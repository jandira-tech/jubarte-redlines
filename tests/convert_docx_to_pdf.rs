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
fn pprchange_ghost_list_does_not_hang_live_text() {
    // file_146 / sample_iter2: live pPr is pBdr+spacing; ListParagraph
    // hanging 320 / numPr lives only in w:pPrChange. first_named
    // descendants steal that ghost so Hello sits at ~90pt (left 36 −
    // hang 18) instead of Word's margin 72.
    let body = "<w:p><w:pPr>\
           <w:spacing w:before=\"300\" w:after=\"80\"/>\
           <w:pPrChange w:id=\"1\" w:author=\"a\">\
             <w:pPr><w:pStyle w:val=\"ListParagraph\"/>\
               <w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr>\
               <w:ind w:left=\"720\" w:hanging=\"360\"/>\
             </w:pPr>\
           </w:pPrChange>\
         </w:pPr>\
         <w:r><w:t>HelloGhost</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert pPrChange ghost");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    let min = xs.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        (70.0..76.0).contains(&min),
        "live text stays at margin 72, not ghost hanging ~90; min={min} xs={xs:?}"
    );
}

#[test]
fn deleted_para_skips_pprchange_list_marker_after_mini_303() {
    // file_146 deleted Ground rules: Word paints the pPrChange en-dash,
    // but shipping that marker (mini 303–305) dropped file_146 −0.018
    // / sample_iter2 −0.018. Keep live pPr only (no ghost numPr).
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
              <w:lvlText w:val=\"%1.\"/>\
              <w:pPr><w:ind w:left=\"720\" w:hanging=\"360\"/></w:pPr></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let body = "<w:p><w:pPr>\
           <w:spacing w:before=\"300\" w:after=\"80\"/>\
           <w:pPrChange w:id=\"1\" w:author=\"a\">\
             <w:pPr><w:pStyle w:val=\"ListParagraph\"/>\
               <w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr>\
               <w:ind w:left=\"720\" w:hanging=\"360\"/>\
             </w:pPr>\
           </w:pPrChange>\
         </w:pPr>\
         <w:del w:id=\"0\" w:author=\"a\">\
           <w:r><w:delText>DelItem</w:delText></w:r></w:del></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering)))
        .expect("convert deleted pPrChange list");
    let text = pdf_winansi_text(&pdf);
    assert!(
        text.contains("DelItem") && !text.contains("1."),
        "mini 303 ghost numPr on del-only was ITT-neg; text={text:?}"
    );
}

#[test]
fn numbered_list_revision_keeps_single_counter_after_mini_310() {
    // potpourri Word All Markup `6.11.` is original+current numbering.
    // Ungated mini 310–312 dropped redline file_21_file_22 −0.1035.
    // Gated `%1.`-only mini 313–315 lifted no-redline +0.0007/+0.0127
    // but redline mean −0.0002 (file_52/53/6/74 −0.006). Keep one
    // counter: Ins+Live+Del paints `1. 2. 3.`, not `1.2.`.
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
              <w:lvlText w:val=\"%1.\"/>\
              <w:pPr><w:ind w:left=\"360\" w:hanging=\"360\"/></w:pPr></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let numpr = "<w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>";
    let body = format!(
        "<w:p>{numpr}<w:ins w:id=\"1\" w:author=\"a\">\
           <w:r><w:t>Alpha</w:t></w:r></w:ins></w:p>\
         <w:p>{numpr}<w:r><w:t>Bravo</w:t></w:r></w:p>\
         <w:p>{numpr}<w:del w:id=\"2\" w:author=\"a\">\
           <w:r><w:delText>Charlie</w:delText></w:r></w:del></w:p>\
         <w:sectPr/>"
    );
    let pdf = docx_to_pdf(&numbering_docx(&body, Some(numbering)))
        .expect("convert list revision markers");
    let text = pdf_winansi_text(&pdf);
    assert!(
        text.contains("3.") && !text.contains("1.2.") && text.contains("Charlie"),
        "mini 310/313 dual orig+current was ITT-neg on redline; text={text:?}"
    );
}

#[test]
fn tbl_header_repeats_on_overflow_page() {
    // file_34 / uipriority: `w:trPr/w:tblHeader` is Word's repeating
    // header. Without it, overflow pages lose Feature/Description.
    let mut rows = String::from(
        "<w:tr><w:trPr><w:tblHeader/></w:trPr>\
           <w:tc><w:p><w:r><w:t>HdrCol</w:t></w:r></w:p></w:tc></w:tr>",
    );
    for i in 0..40 {
        rows.push_str(&format!(
            "<w:tr><w:tc><w:p><w:r><w:t>Body{i:02}</w:t></w:r></w:p></w:tc></w:tr>"
        ));
    }
    let body =
        format!("<w:tbl><w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid>{rows}</w:tbl><w:sectPr/>");
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert tblHeader");
    let pages = pdf_page_count(&pdf);
    let text = pdf_winansi_text(&pdf);
    let n = text.matches("HdrCol").count();
    assert!(
        pages >= 2 && n >= 2 && text.contains("Body00"),
        "tblHeader must repeat on overflow pages; pages={pages} n={n} text={text:?}"
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

fn diagram_docx(body: &str, data_xml: &str) -> Vec<u8> {
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
           xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
           xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
           xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
           xmlns:dgm=\"http://schemas.openxmlformats.org/drawingml/2006/diagram\">\
         <w:body>{body}</w:body></w:document>"
    );
    let types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        <Override PartName=\"/word/diagrams/data1.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.drawingml.diagramData+xml\"/>\
        </Types>";
    let pkg_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let doc_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rIdDm\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramData\" \
          Target=\"diagrams/data1.xml\"/>\
        </Relationships>";
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
    zip.start_file("word/diagrams/data1.xml", opts).unwrap();
    zip.write_all(data_xml.as_bytes()).unwrap();
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
fn page_float_blip_keeps_xml_extent_when_wider_than_page() {
    // image_out_of_folder DeepL banner: 10690522×807396 EMU = 841.77×63.57
    // on A4 (595.3pt). Word paints the overflow (clipped by the page);
    // scaling to page.width squashed it to 595.3×44.96.
    let drawing = blip(
        "10690522",
        "807396",
        "<wp:anchor simplePos=\"0\" relativeHeight=\"1\" behindDoc=\"0\" \
          locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"page\"><wp:posOffset>0</wp:posOffset></wp:positionH>\
          <wp:positionV relativeFrom=\"page\"><wp:posOffset>0</wp:posOffset></wp:positionV>",
        "<wp:wrapNone/></wp:anchor>",
    );
    let body = format!(
        "<w:p><w:r>{drawing}</w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&drawing_docx(&body)).expect("convert wide page-float blip");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("841.77 0 0 63.57"),
        "xml extent 841.77×63.57 must not scale to A4 width; snippet {}",
        text.split("/Im")
            .nth(1)
            .unwrap_or(&text[text.len().saturating_sub(240)..])
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
    assert!(
        rgb_image_has_light_gray_fill(&pdf),
        "clipart CRT is a gray-filled polygon (brush 0xDADADA); 0-based SelectObject paints it black"
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

#[test]
fn tiff_blip_paints_rgb_ink() {
    // Official tiff_image / the <50 1-pagers store word/media/*.tif.
    // image crate is png+jpeg only, so decode_image returns Reserve and
    // Word's TIFF ink never paints (official ITT ~46).
    const TIFF: &[u8] = &[
        0x49, 0x49, 0x2a, 0x00, 0x08, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x00, 0x01, 0x03, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x02, 0x01, 0x03, 0x00, 0x03, 0x00, 0x00, 0x00, 0x86, 0x00, 0x00,
        0x00, 0x03, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x06, 0x01,
        0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x11, 0x01, 0x04, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x8c, 0x00, 0x00, 0x00, 0x15, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x03, 0x00, 0x00, 0x00, 0x16, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
        0x00, 0x17, 0x01, 0x04, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x28, 0x01,
        0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08,
        0x00, 0x08, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    assert_eq!(&TIFF[..4], b"II*\0", "fixture must stay little-endian TIFF");
    let drawing = blip(
        "914400",
        "914400",
        "<wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">",
        "</wp:inline>",
    );
    let docx = drawing_docx_media(
        &format!("<w:p><w:r>{drawing}</w:r><w:r><w:t>AfterTiff</w:t></w:r></w:p><w:sectPr/>"),
        "dot.tif",
        TIFF,
    );
    let pdf = docx_to_pdf(&docx).expect("convert TIFF");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Subtype /Image"),
        "TIFF must be painted as a PDF image, not an empty reserve; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        rgb_image_has_dark_samples(&pdf),
        "decoded TIFF must contain ink"
    );
}

fn rgb_image_has_dark_samples(pdf: &[u8]) -> bool {
    // Match the exact XObject header we emit so embedded TTF bytes cannot
    // be mistaken for an image stream.
    const MARK: &[u8] = b"/Type /XObject /Subtype /Image /Width ";
    let mut from = 0;
    while from + MARK.len() < pdf.len() {
        let Some(rel) = pdf[from..]
            .windows(MARK.len())
            .position(|window| window == MARK)
        else {
            break;
        };
        let at = from + rel;
        let header_end = pdf.len().min(at + 400);
        if let Some(stream_at) = pdf[at..header_end]
            .windows(7)
            .position(|window| window == b"stream\n")
        {
            let data = &pdf[at + stream_at + 7..];
            let end = data
                .windows(9)
                .position(|window| window == b"endstream")
                .unwrap_or(data.len().min(200_000));
            if data[..end].iter().any(|&b| b < 200) {
                return true;
            }
        }
        from = at + MARK.len();
    }
    false
}

fn rgb_image_has_light_gray_fill(pdf: &[u8]) -> bool {
    const MARK: &[u8] = b"/Type /XObject /Subtype /Image /Width ";
    let mut from = 0;
    while from + MARK.len() < pdf.len() {
        let Some(rel) = pdf[from..]
            .windows(MARK.len())
            .position(|window| window == MARK)
        else {
            break;
        };
        let at = from + rel;
        let header_end = pdf.len().min(at + 400);
        if let Some(stream_at) = pdf[at..header_end]
            .windows(7)
            .position(|window| window == b"stream\n")
        {
            let data = &pdf[at + stream_at + 7..];
            let end = data
                .windows(9)
                .position(|window| window == b"endstream")
                .unwrap_or(data.len().min(200_000));
            let rgb = &data[..end];
            let mut gray = 0_u32;
            for pix in rgb.chunks_exact(3) {
                let [r, g, b] = [pix[0], pix[1], pix[2]];
                if (200..=235).contains(&r) && r.abs_diff(g) < 16 && g.abs_diff(b) < 16 {
                    gray += 1;
                }
            }
            if gray >= 80 {
                return true;
            }
        }
        from = at + MARK.len();
    }
    false
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
fn wrap_square_dist_l_keeps_text_left_of_a_right_float() {
    // Strict01 / ole / image_out: wp:anchor distL=114300 (9pt) + wrapSquare
    // bothSides. Word wraps body beside the float, not under it. A 144pt
    // right-aligned float on a 468pt measure leaves text ending before
    // x=396-9=387. Overlay-only layout paints the first line to ~540.
    let img = blip(
        "1828800",
        "1828800",
        "<wp:anchor distT=\"0\" distB=\"0\" distL=\"114300\" distR=\"114300\" simplePos=\"0\" \
           relativeHeight=\"1\" behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
           <wp:positionH relativeFrom=\"margin\"><wp:align>right</wp:align></wp:positionH>\
           <wp:positionV relativeFrom=\"margin\"><wp:align>top</wp:align></wp:positionV>\
           <wp:wrapSquare wrapText=\"bothSides\"/>",
        "</wp:anchor>",
    );
    let words = "alpha ".repeat(80);
    let docx = drawing_docx(&format!(
        "<w:p><w:r>{img}</w:r><w:r><w:t>{words}</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
    ));
    let pdf = docx_to_pdf(&docx).expect("convert wrapSquare distL");
    assert_eq!(pdf_page_count(&pdf), 1, "overlay float must stay 1pp");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(
        xs.iter().any(|x| (70.0..90.0).contains(x)),
        "body still starts at the left margin; xs={xs:?}"
    );
    assert!(
        xs.iter().all(|x| *x < 392.0),
        "Word wraps left of the 144pt right float + 9pt distL (text_right=387); xs={xs:?}"
    );
}

#[test]
fn linked_txbx_part_paints_textbox_text() {
    // mcdoc stores "hello" in word/txbx1.xml, referenced by
    // `<wps:txbx r:txbx="rId6"/>` with no inline txbxContent.
    let body = "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" \
          behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"column\"><wp:align>center</wp:align></wp:positionH>\
          <wp:positionV relativeFrom=\"paragraph\"><wp:posOffset>0</wp:posOffset></wp:positionV>\
          <wp:extent cx=\"2366645\" cy=\"1404620\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"2\" name=\"Text Box 2\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
              <wps:txbx r:txbx=\"rIdTxbx\"/>\
            </wps:wsp>\
          </a:graphicData></a:graphic>\
        </wp:anchor></w:drawing></w:r></w:p><w:sectPr/>";
    let txbx = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:txbx xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:r><w:t>LinkedHello</w:t></w:r></w:p></w:txbx>";
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
           xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
           xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
           xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
         <w:body>{body}</w:body></w:document>"
    );
    let types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        </Types>";
    let pkg_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let doc_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rIdTxbx\" \
          Type=\"http://schemas.microsoft.com/office/2006/relationships/txbx\" \
          Target=\"txbx1.xml\"/>\
        </Relationships>";
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
    zip.start_file("word/txbx1.xml", opts).unwrap();
    zip.write_all(txbx.as_bytes()).unwrap();
    let docx = zip.finish().unwrap().into_inner();
    let pdf = docx_to_pdf(&docx).expect("convert linked txbx");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.60 w"),
        "linked txbx part must paint a box; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn official_mcdoc_paints_the_hello_textbox() {
    // mcdoc is a one-page Word oracle (~40 ITT). "hello" lives in a
    // wrapNone wps:txbx inside mc:AlternateContent. Convert emits only
    // the paragraph end-mark (no 0.60 w box).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/mcdoc.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official mcdoc.docx")).expect("convert mcdoc");
    assert_eq!(pdf_page_count(&pdf), 1, "mcdoc is one A4 page");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.60 w"),
        "hello textbox must stroke; tail {}",
        &text[text.len().saturating_sub(240)..]
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
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
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
fn diagram_data_labels_are_painted() {
    // Strict01 Word p13 paints SmartArt "Item 1/2/3" from
    // word/diagrams/data1.xml (`dgm:relIds r:dm`). Convert skips the
    // drawing, so the last page is endnote-only.
    let body = "<w:p><w:r><w:drawing><wp:inline>\
          <wp:extent cx=\"5486400\" cy=\"3200400\"/>\
          <wp:docPr id=\"1\" name=\"Diagram 1\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.openxmlformats.org/drawingml/2006/diagram\">\
            <dgm:relIds r:dm=\"rIdDm\"/>\
          </a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p><w:sectPr/>";
    let data = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <dgm:dataModel xmlns:dgm=\"http://schemas.openxmlformats.org/drawingml/2006/diagram\" \
           xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
           <dgm:ptList>\
             <dgm:pt><dgm:t><a:p><a:r><a:t>ItemAlpha</a:t></a:r></a:p></dgm:t></dgm:pt>\
             <dgm:pt><dgm:t><a:p><a:r><a:t>ItemBeta</a:t></a:r></a:p></dgm:t></dgm:pt>\
           </dgm:ptList></dgm:dataModel>";
    let pdf = docx_to_pdf(&diagram_docx(body, data)).expect("convert diagram labels");
    let text = String::from_utf8_lossy(&pdf);
    let shows = text.matches(" Tj").count();
    assert!(
        shows >= 2,
        "diagram data labels must paint (got {shows} Tj); tail {}",
        &text[text.len().saturating_sub(280)..]
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
fn bent_connector_paints_an_elbow_not_a_box() {
    // Strict01 Elbow Connector: bentConnector3 + ln/tailEnd. A single
    // mid-height hairline misses Word's L-shaped path.
    let body = "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" \
          behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:extent cx=\"2149522\" cy=\"1207827\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"Elbow Connector 6\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
              <wps:spPr>\
                <a:prstGeom prst=\"bentConnector3\"><a:avLst/></a:prstGeom>\
                <a:ln><a:solidFill><a:srgbClr val=\"4F81BD\"/></a:solidFill>\
                  <a:tailEnd type=\"triangle\"/></a:ln>\
              </wps:spPr>\
            </wps:wsp>\
          </a:graphicData></a:graphic>\
        </wp:anchor></w:drawing></w:r>\
        <w:r><w:t>After</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert bent connector");
    let text = String::from_utf8_lossy(&pdf);
    let vertical = pdf_has_vertical_stroke(&text);
    let strokes = text.matches("1.25 w").count();
    assert!(
        vertical,
        "bentConnector3 must include a vertical elbow segment; strokes={strokes} vertical={vertical} sample {}",
        text.lines()
            .filter(|l| l.contains(" l S") || l.contains("1.25 w"))
            .take(8)
            .collect::<Vec<_>>()
            .join(" | ")
    );
    assert!(
        !text.contains("0.60 w"),
        "must not stroke a rectangle; tail {}",
        &text[text.len().saturating_sub(200)..]
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
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(!xs.is_empty(), "list text must paint; xs={xs:?}");
    let min_x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        (min_x - 90.0).abs() < 4.0,
        "bullet at hanging start 90pt, not margin 72; min_x={min_x} xs={xs:?}"
    );
}

#[test]
fn numbering_lvljc_right_puts_marker_at_gutter_end() {
    // sd_2517 unnamed ilvl 2/5/8 are w:lvlJc=right with hanging=180.
    // Word right-aligns the marker in the hanging gutter (right edge at
    // body start). We left-align at hanging start.
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
              <w:lvlText w:val=\"%1.\"/><w:lvlJc w:val=\"right\"/>\
              <w:pPr><w:ind w:left=\"720\" w:hanging=\"80\"/></w:pPr></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>Hello</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering))).expect("convert lvlJc=right");
    let ones = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(!ones.is_empty(), "marker 1. must paint; xs={ones:?}");
    let min_x = ones.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        min_x < 102.0,
        "lvlJc=right tucks a wider-than-gutter marker so its right hits body 108, not hanging-start 99; min_x={min_x} xs={ones:?}"
    );
}

#[test]
fn numbering_default_suff_tabs_body_to_num_stop() {
    // sd_2517 Título2 is suff=tab (default) + w:tab val=num pos=1800,
    // hanging=1800. We glue `Section 1. Hello` with a space; Word tabs
    // the body to the 90pt hanging indent.
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
              <w:lvlText w:val=\"Section %1\"/>\
              <w:pPr><w:tabs><w:tab w:val=\"num\" w:pos=\"1800\"/></w:tabs>\
                <w:ind w:left=\"1800\" w:hanging=\"1800\"/></w:pPr></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>Hello</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering))).expect("convert num tab");
    let hs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(!hs.is_empty(), "Hello must paint; xs={hs:?}");
    // Glyphs of "Section 1" sit near the left margin; Hello must sit at
    // the 90pt hanging indent (72+90=162), not immediately after "1 ".
    let hello = hs
        .iter()
        .copied()
        .filter(|x| *x > 140.0)
        .fold(f32::INFINITY, f32::min);
    assert!(
        (156.0..170.0).contains(&hello),
        "default suff is tab to num pos 1800 (x=162), not a space; hello={hello} xs={hs:?}"
    );
}

#[test]
fn ppr_rpr_does_not_restyle_bare_runs_after_mini_pprrpr() {
    // ECMA-376 17.3.1.29: w:pPr/w:rPr is the paragraph-mark glyph only.
    // Applying it to bare runs (mini pprrpr) italicized/blued sd_2517
    // TextHeading2 and dropped ITT −0.33 (file_22 too; mean 59.1995→59.1885).
    let body = "<w:p><w:pPr><w:rPr><w:i/><w:color w:val=\"0000FF\"/></w:rPr></w:pPr>\
         <w:r><w:t>Hello</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx(body, None)).expect("convert pPr/rPr lock");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("/Carlito-Italic") && !text.contains("/Calibri-Italic"),
        "paragraph-mark italic must not restyle bare Hello; mini pprrpr −0.33"
    );
    assert!(
        !text.contains("0.000 0.000 1.000 rg"),
        "paragraph-mark 0000FF must not paint bare Hello blue; mini pprrpr −0.33"
    );
}

#[test]
fn keeplines_moves_wrapped_para_off_the_widow_line() {
    // sd_2517 Título1–5 / DocumentTitle set w:keepLines. Letter 792 / 72pt
    // margins leave 648pt. Eight exact-72pt fillers leave 72pt — enough
    // for one line of a two-line keepLines para. Word moves both lines
    // to the next page; we park AlphaOne on the floor (y≈72). Glyphs
    // paint as one-char Tj; unique A/B starters locate each line.
    let mut body = String::new();
    for i in 0..8 {
        body.push_str(&format!(
            "<w:p><w:pPr><w:spacing w:after=\"0\" w:line=\"1440\" w:lineRule=\"exact\"/></w:pPr>\
             <w:r><w:t>Fill{i}</w:t></w:r></w:p>"
        ));
    }
    body.push_str(
        "<w:p><w:pPr><w:keepLines/>\
           <w:spacing w:after=\"0\" w:line=\"1440\" w:lineRule=\"exact\"/></w:pPr>\
         <w:r><w:t>AlphaOne</w:t></w:r><w:r><w:br/></w:r><w:r><w:t>BetaTwo</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&numbering_docx(&body, None)).expect("convert keepLines");
    assert!(
        pdf_page_count(&pdf) >= 2,
        "two-line keepLines plus 8×72pt fillers need a second page; n={}",
        pdf_page_count(&pdf)
    );
    let hay = String::from_utf8_lossy(&pdf);
    let one_y = pdf_cm_tj_xy(&hay, "A")
        .into_iter()
        .map(|(_, y)| y)
        .fold(f32::NEG_INFINITY, f32::max);
    let two_y = pdf_cm_tj_xy(&hay, "B")
        .into_iter()
        .map(|(_, y)| y)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        one_y > 400.0 && two_y > 400.0,
        "keepLines must lift both lines off the 72pt widow slot onto the next page top; AlphaOne={one_y} BetaTwo={two_y}"
    );
    assert!(
        (one_y - two_y).abs() < 80.0,
        "keepLines pair must share a page (72pt exact gap); AlphaOne={one_y} BetaTwo={two_y}"
    );
}

#[test]
fn numbering_suff_nothing_omits_gutter_space() {
    // sd_2517 Título1 `Article %2` is w:suff=nothing. Default space
    // glued `Article One ` onto the heading body.
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
              <w:lvlText w:val=\"%1.\"/><w:suff w:val=\"nothing\"/>\
              <w:pPr><w:ind w:left=\"0\" w:hanging=\"0\"/></w:pPr></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>Hello</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering))).expect("convert suff=nothing");
    let text = pdf_winansi_text(&pdf);
    assert!(
        text.contains("1.Hello"),
        "w:suff=nothing must not append a gutter space; text={text:?}"
    );
}

#[test]
fn en_dash_bullet_stays_concatenated_after_mini_endash() {
    // file_146 ListBullet lvlText is U+2013 (–), hanging=320 / left=640.
    // Word p5: dash at 88, body at 104. Hanging U+2013 (mini 205–208)
    // lifted no-redline +0.044/+0.233 but dropped redline mean
    // 54.5872→54.5825. Keep the concatenated body at ~96.
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"bullet\"/>\
              <w:lvlText w:val=\"\u{2013}\"/>\
              <w:pPr><w:ind w:left=\"640\" w:hanging=\"320\"/></w:pPr></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>EveryPR</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering))).expect("convert en-dash bullet");
    let hay = String::from_utf8_lossy(&pdf);
    let es = pdf_cm_tj_xy(&hay, "E");
    assert!(!es.is_empty(), "EveryPR must paint; es={es:?}");
    let (ex, ey) = es
        .iter()
        .copied()
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap();
    assert!(
        (90.0..100.0).contains(&ex),
        "mini endash hanging was redline ITT-neg; keep concatenated E at ~96; ex={ex} ey={ey} es={es:?}"
    );
}

#[test]
fn official_file_146_en_dash_stays_concatenated_after_mini_endash() {
    // Word p5 wants dash 88 / body 104. mini 205–208 hanging dropped
    // redline mean −0.0047; keep concatenated E at ~96.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let hay = String::from_utf8_lossy(&pdf);
    let dashes = pdf_cm_tj_xy(&hay, "\\226");
    assert!(
        dashes.iter().any(|(x, _)| (80.0..95.0).contains(x)),
        "WinAnsi en-dash (octal 226) must still paint; dashes={dashes:?}"
    );
    let es = pdf_cm_tj_xy(&hay, "E");
    let hung = dashes.iter().any(|(dx, dy)| {
        (80.0..95.0).contains(dx)
            && es
                .iter()
                .any(|(ex, ey)| (ey - dy).abs() < 0.6 && (100.0..110.0).contains(ex))
    });
    assert!(
        !hung,
        "en-dash hanging was mini 205–208 redline ITT-neg; dashes={dashes:?} es={es:?}"
    );
    let concat = dashes.iter().any(|(dx, dy)| {
        (80.0..95.0).contains(dx)
            && es
                .iter()
                .any(|(ex, ey)| (ey - dy).abs() < 0.6 && (90.0..100.0).contains(ex))
    });
    assert!(
        concat,
        "Every PR must stay concatenated at ~96; dashes={dashes:?} es={es:?}"
    );
}

#[test]
fn list_bullet_symbol_rfonts_embeds_symbol_not_body_aptos() {
    // comments / addition* / file_27: ListBullet lvl rFonts are
    // ascii=Symbol. Word Quartz paints SymbolMT •; we inherited
    // Normal=Aptos and painted Aptos •.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
             <w:rPr><w:rFonts w:ascii=\"Aptos\" w:hAnsi=\"Aptos\"/>\
               <w:sz w:val=\"22\"/></w:rPr>\
           </w:style>\
         </w:styles>";
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:abstractNum w:abstractNumId=\"0\">\
             <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"bullet\"/>\
               <w:lvlText w:val=\"\u{F0B7}\"/>\
               <w:rPr><w:rFonts w:ascii=\"Symbol\" w:hAnsi=\"Symbol\"/></w:rPr>\
               <w:pPr><w:ind w:left=\"360\" w:hanging=\"360\"/></w:pPr></w:lvl>\
           </w:abstractNum>\
           <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
         </w:numbering>";
    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>HangItem</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(
        body,
        Some(numbering),
        Some(styles),
    ))
    .expect("convert Symbol bullet");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Symbol") || text.contains("SymbolMT") || text.contains("/LiberationSans"),
        "Word paints ListBullet in Symbol, not body Aptos; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn official_comments_lots_embeds_symbol_for_list_bullets() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments.docx";
    let bytes = std::fs::read(path).expect("official comments-lots");
    let pdf = docx_to_pdf(&bytes).expect("convert comments-lots");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Symbol") || text.contains("SymbolMT") || text.contains("/LiberationSans"),
        "official ListBullet lvl is Symbol; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn symbol_pua_bullet_stays_winansi_bullet_after_mini_108() {
    // Word-faithful Symbol U+00B7 (mini 108) was ITT-wrong: comments-lots
    // −0.016, addition* −0.008, potpourri only +0.006. Keep U+2022 0x95.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
             <w:rPr><w:rFonts w:ascii=\"Aptos\" w:hAnsi=\"Aptos\"/>\
               <w:sz w:val=\"22\"/></w:rPr>\
           </w:style>\
         </w:styles>";
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:abstractNum w:abstractNumId=\"0\">\
             <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"bullet\"/>\
               <w:lvlText w:val=\"\u{F0B7}\"/>\
               <w:rPr><w:rFonts w:ascii=\"Symbol\" w:hAnsi=\"Symbol\"/></w:rPr>\
               <w:pPr><w:ind w:left=\"360\" w:hanging=\"360\"/></w:pPr></w:lvl>\
           </w:abstractNum>\
           <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
         </w:numbering>";
    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>HangItem</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(
        body,
        Some(numbering),
        Some(styles),
    ))
    .expect("convert Symbol PUA bullet");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("\\267"),
        "mini 108 U+00B7 was ITT-wrong on comments-lots; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        text.contains("/Symbol") || text.contains("SymbolMT") || text.contains("/LiberationSans"),
        "Symbol PUA ListBullet must embed SymbolMT (U+F0B7), not Aptos 0x95; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
}

#[test]
fn official_potpourri_symbol_bullet_stays_winansi_after_mini_108() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("\\267"),
        "mini 108 Symbol U+00B7 was ITT-wrong on comments-lots; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
}

fn pdf_tf_xy(pdf: &[u8], tf: &str) -> Vec<(f32, f32)> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tf) {
        let start = from + rel;
        let end = hay.len().min(start + tf.len() + 80);
        let slice = &hay[start..end];
        if let Some(td) = slice.find(" Td") {
            let before = &slice[..td];
            let mut parts = before.rsplit([' ', '\n']);
            let y = parts.next().and_then(|s| s.parse::<f32>().ok());
            let x = parts.next().and_then(|s| s.parse::<f32>().ok());
            if let (Some(x), Some(y)) = (x, y) {
                out.push((x, y));
            }
        }
        from += rel + tf.len();
    }
    out
}

fn pdf_line_min_xs(pdf: &[u8]) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let mut pts = pdf_tf_xy(pdf, "11.04 Tf");
    pts.extend(pdf_tf_xy(pdf, "12.00 Tf"));
    pts.extend(pdf_device_xy(hay.as_ref(), "46 Tf"));
    pts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut lines: Vec<(f32, f32)> = Vec::new();
    for (x, y) in pts {
        if let Some((min_x, ly)) = lines.last_mut()
            && (*ly - y).abs() <= 0.4
        {
            *min_x = min_x.min(x);
        } else {
            lines.push((x, y));
        }
    }
    lines.into_iter().map(|(x, _)| x).collect()
}

fn pdf_line_xs_grouped(pdf: &[u8]) -> Vec<Vec<f32>> {
    pdf_line_xs_in(&String::from_utf8_lossy(pdf))
}

fn pdf_line_xs_in(hay: &str) -> Vec<Vec<f32>> {
    let mut pts = pdf_tf_xy(hay.as_bytes(), "11.04 Tf");
    pts.extend(pdf_tf_xy(hay.as_bytes(), "12.00 Tf"));
    pts.extend(pdf_device_xy(hay, "46 Tf"));
    pts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    let mut lines: Vec<(f32, Vec<f32>)> = Vec::new();
    for (x, y) in pts {
        if let Some((ly, xs)) = lines.last_mut()
            && (*ly - y).abs() <= 0.4
        {
            xs.push(x);
        } else {
            lines.push((y, vec![x]));
        }
    }
    lines
        .into_iter()
        .map(|(_, mut xs)| {
            xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
            xs
        })
        .collect()
}

fn hanging_list_pair_count(lines: &[Vec<f32>]) -> usize {
    lines
        .iter()
        .filter(|xs| {
            let has_gutter = xs.iter().any(|x| (*x - 72.0).abs() < 1.5);
            let has_body = xs.iter().any(|x| (*x - 90.0).abs() < 1.5);
            has_gutter && has_body
        })
        .count()
}

fn list_number_fixture(body: &str) -> Vec<u8> {
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"paragraph\" w:styleId=\"ListNumber\">\
            <w:pPr><w:numPr><w:numId w:val=\"2\"/></w:numPr></w:pPr>\
          </w:style>\
        </w:styles>";
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
              <w:lvlText w:val=\"%1.\"/>\
              <w:pPr><w:ind w:left=\"360\" w:hanging=\"360\"/></w:pPr></w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"2\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    numbering_docx_with_styles(body, Some(numbering), Some(styles))
}

fn potpourri_list_number_body() -> &'static str {
    "<w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/></w:pPr>\
         <w:r><w:t>Preheat</w:t></w:r></w:p>\
       <w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/></w:pPr>\
         <w:r><w:t>Whisk</w:t></w:r></w:p>\
       <w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/>\
         <w:numPr><w:ilvl w:val=\"1\"/><w:numId w:val=\"2\"/></w:numPr></w:pPr>\
         <w:r><w:t>Sift</w:t></w:r></w:p>\
       <w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/>\
         <w:numPr><w:ilvl w:val=\"1\"/><w:numId w:val=\"2\"/></w:numPr></w:pPr>\
         <w:r><w:t>AddSalt</w:t></w:r></w:p>\
       <w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/></w:pPr>\
         <w:r><w:t>Bake</w:t></w:r></w:p>\
       <w:sectPr/>"
}

#[test]
fn missing_ilvl_continues_the_defined_list_level() {
    // potpourri / file_170: abstract only defines ilvl=0. Word ignores
    // the requested ilvl=1 and continues 1. 2. 3. 4. 5. at the same
    // hanging (marker 72 / body 90). Synthesizing a nested 1. 2. at
    // 90/108 is Word-wrong.
    let pdf = docx_to_pdf(&list_number_fixture(potpourri_list_number_body()))
        .expect("convert missing ilvl");
    let lines = pdf_line_xs_grouped(&pdf);
    let pairs = hanging_list_pair_count(&lines);
    let nested = lines
        .iter()
        .filter(|xs| {
            let min = xs.first().copied().unwrap_or(0.0);
            (min - 90.0).abs() < 1.5 && xs.iter().any(|x| (*x - 108.0).abs() < 1.5)
        })
        .count();
    assert!(
        pairs >= 5,
        "Word continues all five items at 72/90; pairs={pairs} lines={lines:?}"
    );
    assert_eq!(
        nested, 0,
        "missing ilvl must not synthesize a nested left=720; lines={lines:?}"
    );
    let mins = pdf_line_min_xs(&pdf);
    assert!(
        mins.iter().filter(|x| (**x - 72.0).abs() < 1.5).count() >= 5,
        "all five items start in the hanging gutter; mins={mins:?}"
    );
}

#[test]
fn official_potpourri_list_number_continues_without_nest() {
    // Word p0: 1.Preheat 2.Whisk 3.Sift 4.Add 5.Bake, all marker@72
    // body@90. We restarted Sift/Add as nested 1. 2. at 90/108.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official potpourri")).expect("convert potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let pages = pdf_content_streams(&pdf);
    let lines = pdf_line_xs_in(&pages[0]);
    let pairs = hanging_list_pair_count(&lines);
    assert!(
        pairs >= 5,
        "Word p0 ListNumber is 1-5 at 72/90; pairs={pairs} lines={lines:?}"
    );
}

#[test]
fn missing_ilvl_on_decimal_list_still_paints_nested_markers() {
    // Kept name: Word does not nest here. ilvl=1 without a defined lvl
    // continues the ilvl=0 counter at the same hanging (see
    // missing_ilvl_continues_the_defined_list_level).
    let pdf = docx_to_pdf(&list_number_fixture(potpourri_list_number_body()))
        .expect("convert missing ilvl");
    let pairs = hanging_list_pair_count(&pdf_line_xs_grouped(&pdf));
    assert!(
        pairs >= 5,
        "undefined ilvl continues 1-5 at 72/90; pairs={pairs}"
    );
}

#[test]
fn official_potpourri_nested_list_indents_child_items() {
    // Kept name: Word p0 is 1-5 at 72/90, not a nested Sift/Add.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official potpourri")).expect("convert potpourri");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "potpourri must emit a page");
    let pairs = hanging_list_pair_count(&pdf_line_xs_in(&pages[0]));
    assert!(
        pairs >= 5,
        "Word p0 ListNumber continues at hanging 72/90; pairs={pairs}"
    );
}

#[test]
fn keep_next_heading_moves_with_the_following_table() {
    // comments-lots Heading1 carries w:keepNext. Word then starts the
    // capability matrix on page 4 (Compatibility on page 5). We orphaned
    // the heading at the bottom of page 3 and began the table there.
    let mut body = String::new();
    for i in 0..45 {
        body.push_str(&format!(
            "<w:p><w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
               <w:r><w:t>Pad{i}</w:t></w:r></w:p>"
        ));
    }
    body.push_str(
        "<w:p><w:pPr><w:keepNext/>\
           <w:spacing w:before=\"0\" w:after=\"0\" w:line=\"240\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"36\"/></w:rPr><w:t>HeadKeep</w:t></w:r></w:p>\
         <w:tbl><w:tblGrid>\
           <w:gridCol w:w=\"4000\"/><w:gridCol w:w=\"4000\"/></w:tblGrid>",
    );
    for i in 0..20 {
        body.push_str(&format!(
            "<w:tr><w:tc><w:p><w:r><w:rPr><w:sz w:val=\"16\"/></w:rPr>\
               <w:t>Cell{i}a</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:rPr><w:sz w:val=\"16\"/></w:rPr>\
               <w:t>Cell{i}b</w:t></w:r></w:p></w:tc></w:tr>"
        ));
    }
    body.push_str(
        "</w:tbl><w:sectPr>\
         <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
         <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/>\
       </w:sectPr>",
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert keepNext");
    let head_ys = pdf_tf_ys(&pdf, "18.00 Tf");
    assert!(
        !head_ys.is_empty(),
        "18pt keepNext heading must paint; pages={}",
        pdf_page_count(&pdf)
    );
    let y = head_ys[0];
    assert!(
        y > 500.0,
        "keepNext heading+table must start on a fresh page, not the leftover strip; y={y} ys={head_ys:?}"
    );
}

#[test]
fn official_comments_lots_positioning_thesis_is_word_tall() {
    // Page-1 "Positioning thesis" is a 1-cell vAlign-center D9EAF7
    // banner. wrap_runs counts 4 lines so the +18pt Demo pad (gated to
    // 2..=3) never fires; Word's box is ~69pt and Prepared-for starts
    // 22pt lower than we paint (align max_shift is 5px).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official comments-lots"))
        .expect("convert official comments-lots");
    assert_eq!(pdf_page_count(&pdf), 9, "Word comments-lots is 9pp");
    let hs = pdf_fill_hs(&pdf, 0.851, 0.918, 0.969);
    let cell_h = hs.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        cell_h >= 64.0,
        "Word Positioning thesis banner is ~69pt D9EAF7, not 4-line 50pt; cell_h={cell_h} fills={hs:?}"
    );
}

#[test]
fn official_comments_lots_stays_nine_pages() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official comments-lots"))
        .expect("convert official comments-lots");
    assert_eq!(
        pdf_page_count(&pdf),
        9,
        "official Word comments-lots is 9pp; boxes={:?} rules={:?}",
        pdf_mediaboxes(&pdf),
        pdf_page_rule_counts(&pdf)
    );
    let pages = footer_page_of_total(&pdf);
    assert_eq!(
        pages,
        vec![
            (1, 9),
            (2, 9),
            (3, 9),
            (4, 9),
            (5, 9),
            (6, 9),
            (7, 9),
            (8, 9),
            (9, 9)
        ],
        "three sectPr without pgNumType start must continue PAGE 1–9, not reset at landscape; pages={pages:?}"
    );
}

#[test]
fn official_comments_lots_lightshading_rows_use_body_line_box() {
    // Word LightShading line=240 + Aptos 10.5: 1-line cells ~12pt.
    // table_row_height_pt used 11.0+5=16pt. Wrapped TableGrid headers
    // still need the 8pt chrome (Compatibility stays on Word page 5).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official comments-lots"))
        .expect("convert official comments-lots");
    let hay = String::from_utf8_lossy(&pdf);
    let hs: Vec<f32> = pdf_fill_boxes_in(&hay, 0.827, 0.875, 0.933)
        .into_iter()
        .filter_map(|(_, _, w, h)| (w > 245.0 && h > 8.0).then_some(h))
        .collect();
    assert!(
        hs.iter().any(|h| (11.0..=14.5).contains(h)),
        "Word 1-line LightShading is ~12pt, not 16pt; hs={hs:?}"
    );
    assert!(
        hs.iter().all(|h| *h < 31.0),
        "2-line LightShading must not grow past 11×2+8; hs={hs:?}"
    );
}

#[test]
fn official_file_27_stays_twelve_pages() {
    // Randomized comments-lots: Word is 12pp, landscape on p7. We emit
    // 13 (landscape on p8). The extra portrait page is a 10-row
    // TableGrid whose every trPr carries w:del (a ghost copy of the
    // capability matrix). Mini 59 rewrote every deleted row to
    // "Deleted Cells" and dropped addition* ~5 ITT — those docs have
    // 1-row / MediumShading fully-deleted tables Word still paints.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_27.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_27"))
        .expect("convert official file_27");
    let boxes = pdf_mediaboxes(&pdf);
    let n = pdf_page_count(&pdf);
    assert_eq!(n, 12, "Word file_27 is 12pp; got {n} boxes={boxes:?}");
    assert!(
        boxes.get(6).is_some_and(|&(w, h)| w > h + 10.0),
        "Word landscape is page 7; boxes={boxes:?}"
    );
}

#[test]
fn official_uipriority_stays_two_pages() {
    // Word 2pp. tblCellMar top/bottom 100 twips was added on top of
    // table_row_pad (8pt), so each of the 5 Feature-table rows was
    // ~31pt instead of ~23pt and Summary spilled onto page 3.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/word_tolerated_misplaced_uipriority.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official uipriority"))
        .expect("convert official uipriority");
    let n = pdf_page_count(&pdf);
    assert_eq!(n, 2, "Word uipriority is 2pp; got {n}");
}

#[test]
fn official_uipriority_lists_heading_stays_on_page_one() {
    // uipriority styles are styleId="2"/"3" with w:name heading 1/2 (not
    // Heading1). is_word_heading_style missed those so Calibri typo×1.15
    // extra ~3pt/heading left "5. Lists" on page 2; Word paints it on p1.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/word_tolerated_misplaced_uipriority.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official uipriority"))
        .expect("convert official uipriority");
    let pages = pdf_content_streams(&pdf);
    assert_eq!(pages.len(), 2, "Word uipriority is 2pp");
    let p1 = pdf_winansi_text(pages[0].as_bytes());
    let p2 = pdf_winansi_text(pages[1].as_bytes());
    assert!(
        p1.contains("5. Lists"),
        "Word Heading1-by-name grid keeps 5. Lists on page 1; p1={p1}"
    );
    assert!(
        p2.contains("5.1 Bullet List"),
        "page 2 starts at 5.1 like Word; p2={p2}"
    );
}

#[test]
fn official_file_34_summary_stays_on_page_two() {
    // Word is 2pp. Arial 12 size×1.15 lands 2pp but mini 86 ITT-wrong
    // (file_34 −0.86, heading_3_center 97→94). Keep typo; allow Word+1.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_34.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_34"))
        .expect("convert official file_34");
    let pages = pdf_content_streams(&pdf);
    assert!(
        pages.len() <= 3,
        "file_34 must not grow past Word+1; got {}",
        pages.len()
    );
    assert!(
        pages.len() >= 2 && pdf_winansi_text(pages[1].as_bytes()).contains("Summary"),
        "Summary must sit on page 2 like Word, not a leftover page 3"
    );
}

#[test]
fn official_file_34_matches_word_two_pages() {
    // Word Quartz auto leading for Arial is size×line_mult (12→13.8pt
    // box, 28→32.2). We used em-box×1.15 (~1.5pt extra per Normal line,
    // ~3pt per Title wrap) so "Text alignment options" spilled onto
    // page 3. Glyph size stays 12pt — Arial paint_size×1.15 was ITT-wrong.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_34.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_34"))
        .expect("convert official file_34");
    let pages = pdf_content_streams(&pdf);
    assert_eq!(
        pages.len(),
        2,
        "Word file_34 is 2pp; leftover page 3 is extra Arial leading; got {}",
        pages.len()
    );
    let p2 = pdf_winansi_text(pages[1].as_bytes());
    assert!(
        p2.contains("Text alignment options"),
        "last summary bullet must stay on Word's page 2; p2={p2}"
    );
}

#[test]
fn table_cell_jc_center_centers_header_text() {
    // file_34 / uipriority: header cells `w:jc center`. Word Feature sits
    // at x=125.3 in a 150pt col (pad 9pt). We always painted at pad_l=81.
    let body = "<w:tbl>\
         <w:tblPr><w:tblW w:w=\"0\" w:type=\"auto\"/>\
           <w:tblCellMar>\
             <w:top w:w=\"100\" w:type=\"dxa\"/><w:left w:w=\"180\" w:type=\"dxa\"/>\
             <w:bottom w:w=\"100\" w:type=\"dxa\"/><w:right w:w=\"180\" w:type=\"dxa\"/>\
           </w:tblCellMar>\
         </w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2996\"/><w:gridCol w:w=\"3026\"/><w:gridCol w:w=\"3004\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:tcPr><w:tcW w:w=\"2996\" w:type=\"dxa\"/></w:tcPr>\
             <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
               <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
                 <w:b/><w:sz w:val=\"24\"/></w:rPr><w:t>Feature</w:t></w:r></w:p></w:tc>\
           <w:tc><w:tcPr><w:tcW w:w=\"3026\" w:type=\"dxa\"/></w:tcPr>\
             <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
               <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
                 <w:b/><w:sz w:val=\"24\"/></w:rPr><w:t>Description</w:t></w:r></w:p></w:tc>\
           <w:tc><w:tcPr><w:tcW w:w=\"3004\" w:type=\"dxa\"/></w:tcPr>\
             <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
               <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
                 <w:b/><w:sz w:val=\"24\"/></w:rPr><w:t>Example</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         <w:tr>\
           <w:tc><w:p><w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:sz w:val=\"24\"/></w:rPr><w:t>Bold</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:sz w:val=\"24\"/></w:rPr><w:t>Makes</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:sz w:val=\"24\"/></w:rPr><w:t>Ex</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl><w:sectPr>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/>\
         </w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert centered header cells");
    let xs = pdf_tf_xs(&pdf, "12.00 Tf");
    let feature = pdf_tj_xy(&String::from_utf8_lossy(&pdf), "F")
        .into_iter()
        .map(|(x, _)| x)
        .min_by(|a, b| a.partial_cmp(b).unwrap());
    let feature_x = feature.or_else(|| xs.iter().copied().min_by(|a, b| a.partial_cmp(b).unwrap()));
    let feature_x = feature_x.expect("Feature F");
    assert!(
        (118.0..=132.0).contains(&feature_x),
        "Word centers Feature in the first col at ~125pt, not pad_l 81; x={feature_x} xs={xs:?}"
    );
}

#[test]
fn official_file_34_table_header_feature_is_centered() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_34.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_34"))
        .expect("convert official file_34");
    assert_eq!(pdf_page_count(&pdf), 2, "Word file_34 is 2pp");
    let pages = pdf_content_streams(&pdf);
    let p2 = &pages[1];
    let fs = pdf_tj_xy(p2, "F");
    assert!(
        fs.iter().any(|(x, _)| (118.0..=132.0).contains(x)),
        "Word Feature is centered at ~125pt; F xs={fs:?}"
    );
}

#[test]
fn official_file_34_heading1_to_body_uses_word_calibri_line_box() {
    // Word Calibri Heading1 (sz=32, before=240, after=120, auto 276) sits
    // 26.3pt above the following Arial 12 body. Calibri typo line is
    // already ~1.22×size; multiplying by line_mult 1.15 again is +3pt
    // per heading and leaves file_34 / uipriority ~1 para low of Word
    // (p2 leftover char-style vs Word "5. Lists").
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_34.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_34"))
        .expect("convert official file_34");
    let h1 = distinct_tf_ys(&pdf, "16.08 Tf");
    let body = distinct_tf_ys(&pdf, "12.00 Tf");
    assert!(
        h1.len() >= 2 && body.len() >= 2,
        "Heading1 16.08 and Arial 12 must paint; h1={h1:?} body={body:?}"
    );
    let heading_y = h1[0];
    let body_y = body
        .iter()
        .copied()
        .find(|y| *y < heading_y - 1.0)
        .expect("Arial body below first Heading1");
    let gap = heading_y - body_y;
    assert!(
        (20.5..=22.5).contains(&gap),
        "Word Heading1→body baseline gap is 21.8pt (Calibri typo line, not typo×1.15), got {gap} h1={heading_y} body={body_y}"
    );
}

fn pdf_has_factory_calibri_11(hay: &str) -> bool {
    hay.contains("/Calibri 11.04 Tf") || hay.contains("/Calibri 46 Tf")
}

#[test]
fn official_file_34_omits_factory_calibri_trailing_space() {
    // Word Quartz file_34 is Arial 12 + Calibri-Bold headings. convert
    // currently appends a synthetic 11.04 Calibri space after every
    // non-empty paragraph (~58 extra glyphs). Word has zero Calibri 11.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_34.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_34"))
        .expect("convert official file_34");
    let hay = String::from_utf8_lossy(&pdf);
    assert!(
        !pdf_has_factory_calibri_11(&hay),
        "file_34 must not paint factory Calibri 11 trailing spaces; Word is Arial"
    );
}

#[test]
fn official_sd_2517_omits_factory_calibri_trailing_space() {
    // Word sd_2517 body is Times/Arial. The same 11.04 Calibri trailer
    // is extra ink on every paragraph of the 107pp fixture.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official sd_2517"))
        .expect("convert official sd_2517");
    let pages = pdf_content_streams(&pdf);
    assert_eq!(pages.len(), 107, "Word sd_2517 is 107pp");
    let body = pages.get(6).expect("article page");
    assert!(
        !pdf_has_factory_calibri_11(body),
        "Times body pages must not grow Calibri 11 trailers"
    );
}

#[test]
fn official_cicero_stays_five_pages_after_mini_92() {
    // Word 5pp with West on page 2 (~28.6pt stacked 80+80). Mini 92
    // matched that pairing and dropped Cicero −0.10 ITT (0 better).
    // Keep 20pt rows / West on page 1; do not grow past Word's 5pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Redline_CiceroDo_v_plate_30.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Cicero"))
        .expect("convert official Cicero");
    assert_eq!(pdf_page_count(&pdf), 5, "Word Cicero is 5pp");
}

#[test]
fn unstyled_tblcellmar_80_stays_replaced_after_mini_92() {
    // Word stacks Cicero 80+80 on 8pt chrome (~28.6). Mini 92 did that
    // and was ITT-wrong on the official fixture itself. Keep max().
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:pPrDefault><w:pPr>\
             <w:spacing w:after=\"120\" w:line=\"240\" w:lineRule=\"atLeast\"/>\
           </w:pPr></w:pPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
           </w:style>\
         </w:styles>";
    let body = "<w:tbl><w:tblPr>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"2\" w:color=\"000000\"/>\
             <w:insideH w:val=\"single\" w:sz=\"2\" w:color=\"000000\"/>\
           </w:tblBorders>\
           <w:tblCellMar>\
             <w:top w:w=\"80\" w:type=\"dxa\"/>\
             <w:bottom w:w=\"80\" w:type=\"dxa\"/>\
           </w:tblCellMar></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
           <w:tr><w:tc><w:p><w:r><w:t>North</w:t></w:r></w:p></w:tc></w:tr>\
           <w:tr><w:tc><w:p><w:r><w:t>South</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert 80+80 replace");
    let mut ys = pdf_tf_ys(&pdf, "11.04 Tf");
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    assert!(ys.len() >= 2, "need North and South baselines; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (18.0..22.0).contains(&gap),
        "80+80 stays replaced (~20pt) after mini 92 ITT; gap={gap} ys={ys:?}"
    );
}

#[test]
fn tblcellmar_vertical_is_not_added_on_top_of_row_pad() {
    // uipriority Feature table: explicit top+bottom 100 twips must
    // replace the generic 8pt row chrome, not stack with it.
    let body = "<w:tbl><w:tblPr>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders>\
           <w:tblCellMar>\
             <w:top w:w=\"100\" w:type=\"dxa\"/>\
             <w:bottom w:w=\"100\" w:type=\"dxa\"/>\
           </w:tblCellMar></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
           <w:tr><w:tc><w:p><w:r><w:t>RowA</w:t></w:r></w:p></w:tc></w:tr>\
           <w:tr><w:tc><w:p><w:r><w:t>RowB</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert tblCellMar");
    let mut ys = pdf_tf_ys(&pdf, "11.04 Tf");
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    assert!(ys.len() >= 2, "need RowA and RowB baselines; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (18.0..26.0).contains(&gap),
        "tblCellMar 10pt + 11×1.15 line ≈ 23pt, not 10+8+12.65≈31; gap={gap} ys={ys:?}"
    );
}

#[test]
fn fully_deleted_tablegrid_still_paints_deltext() {
    // Word addition_removal / file_27 p3 still paints the 10-row
    // deleted TableGrid (Capability / Real-time coauthoring). Omitting
    // it made page count match and destroyed pairing.
    let mut rows = String::new();
    for i in 1..=3 {
        rows.push_str(&format!(
            "<w:tr><w:trPr><w:del w:id=\"{i}\" w:author=\"A\"/></w:trPr>"
        ));
        for col in 0..4 {
            rows.push_str(&format!(
                "<w:tc><w:p><w:del w:id=\"{i}{col}\" w:author=\"A\">\
                   <w:r><w:delText>Capability row {i} col {col}</w:delText></w:r>\
                 </w:del></w:p></w:tc>"
            ));
        }
        rows.push_str("</w:tr>");
    }
    let body = format!(
        "<w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2505\"/><w:gridCol w:w=\"2505\"/>\
           <w:gridCol w:w=\"2508\"/><w:gridCol w:w=\"2552\"/></w:tblGrid>\
         {rows}</w:tbl>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading2\"/></w:pPr>\
           <w:r><w:t>AfterDeleted</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&docx_with_styles(&body, &table_grid_line240_styles()))
        .expect("convert fully-deleted TableGrid");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("Capability row 1"),
        "Word still paints deleted TableGrid delText; painted={painted}"
    );
    assert!(
        painted.contains("AfterDeleted"),
        "following heading must still paint; painted={painted}"
    );
}

/// Footer `Page N of T` runs (each `PAGE` / `NUMPAGES` is its own Tj).
fn footer_page_of_total(pdf: &[u8]) -> Vec<(u32, u32)> {
    let text = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut rest = text.as_ref();
    while let Some(i) = rest.find("(Page )") {
        let after = &rest[i + 7..];
        let Some(n) = next_tj_int(after) else {
            break;
        };
        let Some(of_at) = after.find("( of )") else {
            break;
        };
        let Some(total) = next_tj_int(&after[of_at + 6..]) else {
            break;
        };
        out.push((n, total));
        rest = &after[of_at + 6..];
    }
    out
}

fn next_tj_int(s: &str) -> Option<u32> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            let end = bytes[i + 1..]
                .iter()
                .position(|&b| b == b')')
                .map(|p| i + 1 + p)?;
            let inner = &s[i + 1..end];
            if inner.bytes().all(|b| b.is_ascii_digit()) && !inner.is_empty() {
                return inner.parse().ok();
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }
    None
}

fn pdf_page_rule_counts(pdf: &[u8]) -> Vec<usize> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut rest = hay.as_ref();
    while let Some(i) = rest.find(">>\nstream\n") {
        let after = &rest[i + 10..];
        let Some(end) = after.find("\nendstream") else {
            break;
        };
        let body = &after[..end];
        if body.contains(" Tf") && body.len() < 400_000 {
            // Word Quartz hairlines are `… 0.50 … re f` (width or height).
            out.push(body.matches(" 0.50 ").count() + body.matches("0.50 w").count());
        }
        rest = &after[end + 1..];
    }
    out
}

#[test]
fn official_comments_lots_page_five_has_the_capability_table() {
    // Word p5 starts at the last "Compatibility" row of the 13-row matrix
    // plus the chart. Heading1 keepNext + a drawing-only chart para (no
    // extra Normal line) keep that pairing on 9 pages.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official comments-lots"))
        .expect("convert official comments-lots");
    let rules = pdf_page_rule_counts(&pdf);
    assert!(
        rules.len() >= 5,
        "expected 9 page streams with Tf, got {rules:?}"
    );
    assert!(
        rules[4] >= 16,
        "page 5 must include the Compatibility row (4×4 0.50pt rules); rules={rules:?}"
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
    // Compare distinct baselines — per-glyph ys on the same line are ~0.
    let docx = minimal_docx_body(
        "<w:p><w:pPr><w:pStyle w:val=\"Heading2\"/>\
           <w:spacing w:after=\"720\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>Alpha</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading2\"/>\
           <w:spacing w:before=\"720\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>Bravo</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>",
    );
    let pdf = docx_to_pdf(&docx).expect("convert spacing");
    let ys = distinct_tf_ys(&pdf, "11.04 Tf");
    assert!(ys.len() >= 2, "both lines must paint; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (48.0..=58.0).contains(&gap),
        "Word max(after,before)=36pt plus the 11pt line is ~51pt, not ~87pt sum; gap={gap} ys={ys:?}"
    );
}

#[test]
fn official_heading_2_style_follows_word_inter_para_grid() {
    // heading_2_style_demo: latent Heading2, first line=276 / after=10 from
    // docDefaults, next before=18 after=4 line=240. Word yMin gap is 39.12.
    // Summing after+before opened ~48pt and dropped the 80–89 Calibri pack.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/heading_2_style_demo_id_paraid_overflow.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("heading 2 demo")).expect("convert heading 2 demo");
    let ys = distinct_tf_ys(&pdf, "16.08 Tf");
    assert!(
        ys.len() >= 3,
        "three Heading2 lines must paint; ys={ys:?} pages={}",
        pdf_page_count(&pdf)
    );
    let gap = ys[0] - ys[1];
    assert!(
        (37.0..=42.0).contains(&gap),
        "Word Heading2 grid is ~39pt (max after/before), got {gap} ys={ys:?}"
    );
}

#[test]
fn official_heading_1_style_follows_word_inter_para_grid() {
    // heading_1_style_demo: first after=10, next before=20. Word yMin gap
    // 46.32. Summing made ~55pt.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/heading_1_style_demo_id_paraid_overflow.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("heading 1 demo")).expect("convert heading 1 demo");
    let ys = distinct_tf_ys(&pdf, "20.00 Tf");
    assert!(ys.len() >= 2, "Heading1 lines must paint; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (43.0..=49.0).contains(&gap),
        "Word Heading1 grid is ~46pt (max after/before), got {gap} ys={ys:?}"
    );
}

#[test]
fn body_then_heading1_keeps_sum_after_mini_h1max() {
    // potpourri: Normal after=160 (8pt) + Heading1 before=360 (18pt).
    // Word uses max=18pt; mini 209–212 max dropped no-redline −0.057.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:pPrDefault><w:pPr>\
             <w:spacing w:after=\"160\" w:line=\"240\" w:lineRule=\"auto\"/>\
           </w:pPr></w:pPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
           </w:style>\
           <w:style w:type=\"paragraph\" w:styleId=\"Heading1\">\
             <w:name w:val=\"heading 1\"/>\
             <w:pPr><w:spacing w:before=\"360\" w:after=\"80\"/></w:pPr>\
             <w:rPr><w:sz w:val=\"40\"/></w:rPr>\
           </w:style>\
         </w:styles>";
    let body = "<w:p><w:r><w:t>BodyLine</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"40\"/></w:rPr><w:t>HeadLine</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert body then Heading1");
    let body_ys = distinct_tf_ys(&pdf, "11.04 Tf");
    let head_ys = distinct_tf_ys(&pdf, "20.00 Tf");
    assert!(
        !body_ys.is_empty() && !head_ys.is_empty(),
        "body+heading must paint; body={body_ys:?} head={head_ys:?}"
    );
    let gap = body_ys[0] - head_ys[0];
    assert!(
        (46.0..=52.0).contains(&gap),
        "mini h1max was ITT-neg; keep summed ~48pt gap; gap={gap} body={body_ys:?} head={head_ys:?}"
    );
}

#[test]
fn official_potpourri_heading1_stays_summed_after_mini_h1max() {
    // Word p1 Heading1 20pt y=566.4. Max (mini 209–212) dropped
    // potpourri −1.13 and file_170 −2.31. Keep summed y=558.3; 5pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let pages = pdf_content_streams(&pdf);
    let ys: Vec<f32> = pdf_tf_xy(pages[0].as_bytes(), "20.00 Tf")
        .into_iter()
        .filter(|(x, y)| (68.0..80.0).contains(x) && (500.0..640.0).contains(y))
        .map(|(_, y)| y)
        .collect();
    let y = ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (554.0..561.0).contains(&y),
        "mini h1max was ITT-neg; keep summed Heading1 y=558.3; y={y} ys={ys:?}"
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
    let ys = pdf_tf_ys(&pdf, "11.04 Tf");
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
    let ys = pdf_tf_ys(&pdf, "11.04 Tf");
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
fn section_lvltext_does_not_hang_after_mini_sechang() {
    // Word Título2 hangs `Section 1.01` (lvlText longer than 8 chars) at
    // 90pt with body at 180. Hanging it (mini sechang) packed sd_2517 /
    // file_22 107→106pp and dropped ITT −0.10 each. Keep the 8-char cap.
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:abstractNum w:abstractNumId=\"0\">\
            <w:lvl w:ilvl=\"0\">\
              <w:start w:val=\"1\"/><w:numFmt w:val=\"decimalZero\"/>\
              <w:lvlText w:val=\"Section %1\"/>\
              <w:pPr><w:ind w:left=\"1800\" w:hanging=\"1800\"/></w:pPr>\
            </w:lvl>\
          </w:abstractNum>\
          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
        </w:numbering>";
    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>HelloWorld</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering))).expect("convert Section marker");
    let mut xs = pdf_tf_xs(&pdf, "11.04 Tf");
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let max_gap = xs.windows(2).map(|w| w[1] - w[0]).fold(0.0_f32, f32::max);
    assert!(
        max_gap < 20.0,
        "mini sechang 90pt gutter packed 107→106; max_gap={max_gap} xs={xs:?}"
    );
}

#[test]
fn official_potpourri_symbol_bullet_keeps_gutter_space() {
    // Word ListBullet: Symbol • at x=72, Arial space at 77.5, body at 90.
    // Symbol numbering returns PUA with no trailing space, so the gutter
    // is empty (Apples still at 90). Non-Symbol bullets already append
    // ` `. Do not map PUA→U+00B7 (mini 108 ITT-wrong).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official potpourri")).expect("convert potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let pages = pdf_content_streams(&pdf);
    let pts = pdf_tf_xy(pages[0].as_bytes(), "12.00 Tf");
    let mut by_y: Vec<(f32, Vec<f32>)> = Vec::new();
    for (x, y) in pts {
        if let Some((_, xs)) = by_y.iter_mut().find(|(ly, _)| (*ly - y).abs() < 0.6) {
            xs.push(x);
        } else {
            by_y.push((y, vec![x]));
        }
    }
    let mut bullet_gutter = false;
    for (_, xs) in &mut by_y {
        xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min = xs[0];
        if !(68.0..76.0).contains(&min) || !xs.iter().any(|x| (88.0..96.0).contains(x)) {
            continue;
        }
        // Numbered `1.` has 2+ glyphs in 73–87; a Symbol bullet is only
        // the mark at 72 plus Word's Arial space at ~77.5.
        let mid = xs
            .iter()
            .copied()
            .filter(|x| (73.0..87.0).contains(x))
            .count();
        if mid == 1 && xs.iter().any(|x| (76.0..84.0).contains(x)) {
            bullet_gutter = true;
            break;
        }
    }
    assert!(
        bullet_gutter,
        "Word paints an Arial space at ~77.5 in the ListBullet hanging gutter"
    );
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
        text.contains("372.00") || text.contains("371.75") || text.contains("371.50"),
        "300pt table ends at 72+300=372 (fill hairline centered on the edge); stream tail {}",
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
fn hyperlink_rstyle_paints_styled_teal_and_underline() {
    // potpourri / file_170: Hyperlink is a character style (color 467886
    // + single underline). The run only stores rStyle=Hyperlink. We
    // never applied character styles, so "reference page" painted body
    // black and joined the preceding sentence.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"character\" w:styleId=\"Hyperlink\">\
             <w:name w:val=\"Hyperlink\"/>\
             <w:rPr><w:color w:val=\"467886\" w:themeColor=\"hyperlink\"/>\
               <w:u w:val=\"single\"/></w:rPr>\
           </w:style>\
         </w:styles>";
    let body = "<w:p>\
           <w:r><w:t xml:space=\"preserve\">More information available at our </w:t></w:r>\
           <w:hyperlink>\
             <w:r><w:rPr><w:rStyle w:val=\"Hyperlink\"/></w:rPr>\
               <w:t>reference page</w:t></w:r>\
           </w:hyperlink>\
           <w:r><w:t>.</w:t></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert Hyperlink rStyle");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.275 0.471 0.525 rg"),
        "Hyperlink rStyle must paint #467886; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    let hair: Vec<_> = pdf_fill_rects(&pdf, 0.275, 0.471, 0.525)
        .into_iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 20.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "Hyperlink rStyle must fill an underline in #467886; hair={hair:?}"
    );
}

#[test]
fn toc_hyperlink_stays_paragraph_black_like_word_quartz() {
    // sd_2517 / file_22 Sumrio: TOC \h wraps entries in w:hyperlink +
    // rStyle=Hyperlink (0000FF + underline). Word Save-as-PDF paints the
    // toc paragraph style (black, no underline). Blue TOC wiped ~4
    // contents pages (color_sim=0) on the 107pp pair.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:styleId=\"Sumrio1\">\
             <w:name w:val=\"toc 1\"/>\
             <w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
               <w:b/><w:sz w:val=\"24\"/></w:rPr>\
           </w:style>\
           <w:style w:type=\"character\" w:styleId=\"Hyperlink\">\
             <w:name w:val=\"Hyperlink\"/>\
             <w:rPr><w:color w:val=\"0000FF\"/><w:u w:val=\"single\"/></w:rPr>\
           </w:style>\
         </w:styles>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Sumrio1\"/></w:pPr>\
           <w:hyperlink w:anchor=\"_Toc1\">\
             <w:r><w:rPr><w:rStyle w:val=\"Hyperlink\"/></w:rPr>\
               <w:t>TocEntry</w:t></w:r>\
           </w:hyperlink>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert TOC hyperlink");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("0.000 0.000 1.000 rg") && !text.contains("0.000 0.000 1.000 RG"),
        "Word Quartz TOC is not Hyperlink blue; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        text.contains("0.000 0.000 0.000 rg"),
        "TOC entry must paint paragraph black; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn official_potpourri_hyperlink_paints_styled_teal() {
    // Word Quartz p2: "reference page" is its own teal span (#467886).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.275 0.471 0.525 rg"),
        "official potpourri Hyperlink style is #467886"
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
    let hair: Vec<_> = pdf_fill_rects(&pdf, 0.145, 0.388, 0.922)
        .into_iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 8.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "hyperlink underline must fill 2563EB; hair={hair:?}"
    );
}

#[test]
fn table_cell_underline_does_not_paint_past_the_cell() {
    // file_146 / sample_document github cell: xml:space padding on an
    // underlined hyperlink is kept (in_table) and the stroke ran ~46pt
    // past the 540pt table edge. Word clips at the cell.
    let body = "<w:tbl><w:tblPr><w:tblW w:w=\"2520\" w:type=\"dxa\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"2520\"/></w:tblGrid>\
           <w:tr><w:tc><w:p>\
             <w:hyperlink>\
               <w:r><w:rPr><w:color w:val=\"2563EB\"/><w:u w:val=\"single\"/></w:rPr>\
                 <w:t xml:space=\"preserve\">eigenpal/docx-editor                 </w:t></w:r>\
             </w:hyperlink>\
           </w:p></w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert cell underline clip");
    let rules = pdf_vertical_rule_xs(&pdf);
    let right = rules.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        right.is_finite() && right > 100.0,
        "cell must stroke a right rule; rules={rules:?}"
    );
    let x2s = pdf_rgb_underline_x2s(&pdf, 0.145, 0.388, 0.922);
    assert!(!x2s.is_empty(), "underlined hyperlink must stroke");
    let max_x2 = x2s.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_x2 <= right + 1.5,
        "Word clips cell underline at the right rule; x2={max_x2} right={right} x2s={x2s:?} rules={rules:?}"
    );
}

#[test]
fn table_cell_underline_stops_at_ink_not_xml_space_padding() {
    // sample_iter2 / file_146 github cell: `eigenpal/docx-editor                 `
    // (xml:space) is kept in-table. Word underlines the ink (~114pt,
    // x2=488.9) not through the padding to the cell edge (540).
    let body = "<w:tbl><w:tblPr><w:tblW w:w=\"4680\" w:type=\"dxa\"/></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"4680\"/></w:tblGrid>\
           <w:tr><w:tc><w:p>\
             <w:hyperlink>\
               <w:r><w:rPr><w:color w:val=\"2563EB\"/><w:u w:val=\"single\"/></w:rPr>\
                 <w:t xml:space=\"preserve\">pkg                 </w:t></w:r>\
             </w:hyperlink>\
           </w:p></w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert ink underline");
    let hair: Vec<_> = pdf_fill_rects(&pdf, 0.145, 0.388, 0.922)
        .into_iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 4.0)
        .collect();
    assert!(!hair.is_empty(), "underlined pkg must fill; hair={hair:?}");
    let max_w = hair.iter().map(|(w, _)| *w).fold(0.0_f32, f32::max);
    assert!(
        max_w < 40.0,
        "underline must cover 'pkg' ink (~20pt), not xml:space padding to the 234pt cell; max_w={max_w} hair={hair:?}"
    );
}

#[test]
fn official_sample_iter2_github_underline_stops_before_cell_edge() {
    // Word p1 right-cell hyperlink underline is 374.9–488.9. Ours ran
    // to clip_right 540 through generator xml:space padding.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sample_document_word_repair_of_our_output_iter2_word_repaired_2.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official sample_iter2"))
        .expect("convert sample_iter2");
    assert_eq!(pdf_page_count(&pdf), 7, "Word sample_iter2 is 7pp");
    let pages = pdf_content_streams(&pdf);
    let boxes = pdf_fill_boxes_in(&pages[0], 0.145, 0.388, 0.922);
    let hair: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(x, _, w, h)| *h > 0.0 && *h < 1.6 && *w > 20.0 && *x > 300.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "github cell must fill a 2563EB underline; boxes={boxes:?}"
    );
    let max_x2 = hair
        .iter()
        .map(|(x, _, w, _)| x + w)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_x2 < 510.0,
        "Word github underline ends at 488.9, not cell edge 540; max_x2={max_x2} hair={hair:?}"
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
    let npm_x = pdf_tf_xs(&pdf, "11.04 Tf");
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
    let hello = pdf_tf_xs(&pdf, "11.04 Tf");
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
        text.contains("/LiberationMono") || text.contains("/CourierNew"),
        "Courier New must embed Courier New or Liberation Mono; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/Carlito 11.04 Tf"),
        "Courier run must not paint with Carlito"
    );
}

#[test]
fn inter_maps_to_liberation_serif_not_carlito() {
    // Official Word Quartz substitutes Inter → Cambria. The older
    // Liberation-Serif mapping matched soffice, not the Word oracles.
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Inter\" w:hAnsi=\"Inter\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>InterBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Inter");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Cambria"),
        "Inter must embed Cambria (Word substitute); tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/Carlito 11.04 Tf"),
        "Inter run must not paint with Carlito"
    );
}

#[test]
fn inter_embeds_cambria_like_word() {
    // Official Word Quartz oracles for sample_document / eigenpal substitute
    // missing Inter with Cambria (11.04 body). Mapping Inter to Times/Liberation
    // Serif keeps that 8-stem cluster at ITT ~44.
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Inter\" w:hAnsi=\"Inter\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>InterBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Inter");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Cambria"),
        "Word substitutes Inter → Cambria; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/TimesNewRoman") && !text.contains("/LiberationSerif"),
        "Inter must not paint Times/Liberation Serif vs Word Cambria"
    );
}

#[test]
fn ten_pt_uses_word_300dpi_device_size() {
    // sample_document header is sz=20 (10pt). Word Quartz snaps 10pt to
    // 10.08 (42×0.24); we painted 10.00 and missed the header band.
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"20\"/></w:rPr>\
           <w:t>Ten</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 10pt");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("10.08 Tf"),
        "10pt must snap to Word 300dpi 10.08; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    assert!(
        !text.contains("10.00 Tf"),
        "unrounded 10.00 Tf misses the official Word headers"
    );
}

#[test]
fn eight_pt_stays_unsnapped_after_mini_snap8() {
    // sd_2517 / file_22 cover is Word 7.92 (33×0.24). Snapping 8pt
    // (mini snap8) left those stems ~0 and dropped file_34 −0.011.
    let body = "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:spacing w:val=\"24\"/><w:sz w:val=\"16\"/></w:rPr>\
             <w:t>1363 ipsum eiusmod</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 8pt");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("8.00 Tf"),
        "8pt stays 8.00 after mini snap8; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    assert!(
        !text.contains("7.92 Tf"),
        "7.92 snap was ITT-wrong on file_34"
    );
}

#[test]
fn centered_run_track_is_included_in_line_width() {
    // sd_2517 / file_22 cover: jc=center + rPr spacing (20/24 twips).
    // line_w used hmtx only, so extra leftover/2 started ~16pt right of
    // Word (8pt address 251 vs 235). Paint already adds track between
    // glyphs; measure must match.
    let body = "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:spacing w:val=\"1440\"/><w:sz w:val=\"24\"/></w:rPr>\
             <w:t>AB</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert centered track");
    let hay = String::from_utf8_lossy(&pdf);
    let xs: Vec<f32> = pdf_tf_xy(&pdf, "12.00 Tf")
        .into_iter()
        .map(|(x, _)| x)
        .collect();
    assert!(
        !xs.is_empty(),
        "12pt Arial must emit 12.00 Tf; tail {}",
        &hay[hay.len().saturating_sub(240)..]
    );
    let min_x = xs.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        min_x < 280.0,
        "center leftover must include 72pt track so A starts left of naive ~298; min_x={min_x} xs={xs:?}"
    );
}

#[test]
fn official_sd_2517_cover_tracked_eight_pt_starts_near_word() {
    // Word cover 8pt (sz=16, spacing=24, jc=center) starts at x=235.1.
    // Ours measured without track so the same line started at 251.3.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official sd_2517")).expect("convert sd_2517");
    assert_eq!(pdf_page_count(&pdf), 107, "Word sd_2517 is 107pp");
    let pages = pdf_content_streams(&pdf);
    let xs: Vec<f32> = pdf_tf_xy(pages[0].as_bytes(), "8.00 Tf")
        .into_iter()
        .filter(|(_, y)| (210.0..260.0).contains(y))
        .map(|(x, _)| x)
        .collect();
    let min_x = xs.iter().copied().fold(f32::MAX, f32::min);
    assert!(
        (228.0..242.0).contains(&min_x),
        "Word tracked 8pt cover starts at 235, not 251; min_x={min_x} xs={xs:?}"
    );
}

#[test]
fn eight_pt_atleast_two_forty_stays_nine_point_five_after_mini_atl() {
    // sd_2517 cover 8pt atLeast-240 is Word 12pt. Flooring line_box at
    // spec (mini 203) dropped image_out_of_folder / file_48 −1.68 each
    // (mean 59.15→59.09). Do not set line_mult=1 (Cicero 5→4). Keep 9.5.
    let para = "<w:p><w:pPr><w:jc w:val=\"center\"/>\
           <w:spacing w:line=\"240\" w:lineRule=\"atLeast\" w:before=\"0\" w:after=\"0\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:sz w:val=\"16\"/></w:rPr><w:t>COVEREIGHT</w:t></w:r></w:p>";
    let body = format!(
        "{para}{para}\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert 8pt atLeast");
    let mut ys = pdf_tf_ys(&pdf, "8.00 Tf");
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.4);
    assert!(ys.len() >= 2, "two 8pt cover lines; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (8.8..=10.2).contains(&gap),
        "8pt atLeast-240 stays ~9.5 after mini 203; gap={gap} ys={ys:?}"
    );
}

#[test]
fn ten_point_five_stays_unsnapped_after_mini_110() {
    // Word Quartz snaps 10.5 → 10.56. Painting that (mini 110) dropped
    // I_am_sharing −1.14, comments-lots −1.23, image_out_of_folder −3.23.
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"21\"/></w:rPr>\
           <w:t>TenHalf</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 10.5pt");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("10.50 Tf"),
        "10.5pt stays 10.50 after mini 110; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    assert!(
        !text.contains("10.56 Tf"),
        "10.56 snap is ITT-wrong; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

#[test]
fn official_i_am_sharing_body_stays_ten_point_five_after_mini_110() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/I_am_sharing_Microsoft_Word_vs_Google_Docs_Comprehensive_Proof_with_you.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official I_am_sharing"))
        .expect("convert official I_am_sharing");
    assert_eq!(pdf_page_count(&pdf), 9, "Word I_am_sharing is 9pp");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("10.50 Tf"),
        "mini 110 10.56 snap was ITT-wrong; keep 10.50; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn twenty_pt_stays_unsnapped_after_mini_105() {
    // Word Quartz snaps 20 → 19.92. Painting that (mini 105) dropped
    // potpourri −0.016 and uipriority −0.05; keep 20.00.
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"40\"/></w:rPr>\
           <w:t>Twenty</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 20pt");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("20.00 Tf"),
        "20pt stays 20.00 after mini 105; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    assert!(
        !text.contains("19.92 Tf"),
        "19.92 snap is ITT-wrong; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

#[test]
fn twenty_eight_pt_stays_unsnapped_after_mini_105() {
    // Word Quartz snaps 28 → 28.08. Painting that (mini 105) dropped
    // file_34 −0.02; keep 28.00.
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"56\"/></w:rPr>\
           <w:t>TwentyEight</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 28pt");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("28.00 Tf"),
        "28pt stays 28.00 after mini 105; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    assert!(
        !text.contains("28.08 Tf"),
        "28.08 snap is ITT-wrong; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

#[test]
fn inter_auto_line_uses_em_not_cambria_typo_box() {
    // sample_document / eigenpal: Inter → Cambria. Word Quartz auto
    // leading is size×1.15. Cambria typo lineGap=353 makes
    // (asc+desc+gap)×1.15 ~5.6pt taller, so the 32pt title→12pt
    // subtitle gap is 25pt vs Word 19.5 (144dpi ink_f1 0.41→0.29).
    let body = "<w:p><w:pPr><w:spacing w:after=\"20\" w:line=\"276\" w:lineRule=\"auto\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Inter\" w:hAnsi=\"Inter\"/>\
             <w:sz w:val=\"64\"/></w:rPr><w:t>TitleLine</w:t></w:r></w:p>\
         <w:p><w:pPr><w:spacing w:before=\"0\" w:after=\"0\" w:line=\"276\" w:lineRule=\"auto\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Inter\" w:hAnsi=\"Inter\"/>\
             <w:sz w:val=\"24\"/></w:rPr><w:t>SubLine</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1300\" w:right=\"1440\" w:bottom=\"1300\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Inter title+sub");
    let title_ys = pdf_tf_ys(&pdf, "31.92 Tf");
    let sub_ys = pdf_tf_ys(&pdf, "12.00 Tf");
    assert!(
        !title_ys.is_empty() && !sub_ys.is_empty(),
        "32pt title and 12pt sub must paint; title={title_ys:?} sub={sub_ys:?}"
    );
    let title_y = title_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let sub_y = sub_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let gap = title_y - sub_y;
    assert!(
        (17.5..=21.5).contains(&gap),
        "Word Cambria auto title→12pt gap is ~19.5pt, not typo-box ~25pt; gap={gap} title={title_y} sub={sub_y}"
    );
}

#[test]
fn aptos_maps_to_calibri_metric_not_arial() {
    // Official Word oracles for comments / I_am_sharing / numwords embed
    // Aptos. Aptos is Calibri-metric, not Arial. Mapping it to Liberation
    // Sans painted the wrong width class.
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Aptos\" w:hAnsi=\"Aptos\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>AptosBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Aptos");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Aptos") || text.contains("/Calibri") || text.contains("/Carlito"),
        "Aptos must embed Aptos (or Calibri/Carlito fallback); tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/LiberationSans 11.04 Tf"),
        "Aptos run must not paint Arial-metric Liberation Sans"
    );
}

#[test]
fn calibri_embeds_system_calibri_when_present() {
    // Official track oracles are Word/Calibri (174/199 pdf_source files).
    // Bundled Carlito is metric-compatible but the outlines miss pagefair.
    let path = std::path::Path::new(
        "/Applications/Microsoft Word.app/Contents/Resources/DFonts/Calibri.ttf",
    );
    if !path.is_file() {
        return;
    }
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>CalibriBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Calibri");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Calibri"),
        "Calibri run must embed system Calibri when Word DFonts exist; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
}

#[test]
fn css_font_stack_uses_first_family_verdana() {
    // verdana_font_demo stores ascii="Verdana, Geneva, sans-serif". Matching
    // "sansserif" anywhere mapped the run to Arial; Word embeds Verdana
    // (official ITT ~63).
    let path = std::path::Path::new(
        "/Applications/Microsoft Word.app/Contents/Resources/DFonts/Verdana.ttf",
    );
    if !path.is_file() {
        return;
    }
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Verdana, Geneva, sans-serif\" \
             w:hAnsi=\"Verdana, Geneva, sans-serif\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>VerdanaStack</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Verdana stack");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Verdana"),
        "first family in a CSS stack must embed Verdana; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        !text.contains("/ArialMT") && !text.contains("/LiberationSans"),
        "Verdana stack must not fall through to Arial; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn official_verdana_demo_embeds_verdana_not_arial() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/verdana_font_demo_id_paraid_overflow.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("verdana demo")).expect("convert verdana");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Verdana"),
        "official Verdana demo must embed Verdana; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
}

#[test]
fn open_sans_maps_to_arial_metric_not_calibri() {
    // open_sans_font_demo has no system Open Sans. It is Arial-metric, not
    // Calibri; unknown → Carlito left the cluster at ~65.
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Open Sans\" w:hAnsi=\"Open Sans\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>OpenSansBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Open Sans");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/LiberationSans") || text.contains("/Arial"),
        "Open Sans must embed Arial-metric Sans; tail {}",
        &text[text.len().saturating_sub(280)..]
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
    // Normal=Aptos. Do not put a non-display ascii cache on this style.
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
        text.contains("/Carlito") || text.contains("/Calibri"),
        "Heading1 majorHAnsi=Calibri must embed Calibri/Carlito; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        text.contains("11.04 Tf") || text.contains(" 46 Tf"),
        "Normal Aptos body stays 11pt; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    assert!(
        text.contains("14.00 Tf"),
        "Heading1 sz=28 is 14pt; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

#[test]
fn heading1_aptos_display_ascii_uses_theme_major_calibri() {
    // I_am_sharing Heading1 stores ascii="Aptos Display" *and*
    // asciiTheme=majorHAnsi (Calibri). Word Quartz paints Calibri-Bold
    // 13.9; the Display-family name is a stale cache, not the live face.
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
               <w:rFonts w:asciiTheme=\"majorHAnsi\" w:hAnsiTheme=\"majorHAnsi\" \
                 w:ascii=\"Aptos Display\" w:hAnsi=\"Aptos Display\"/>\
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
         <w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles_and_theme(body, styles, theme))
        .expect("convert Aptos Display Heading1");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Calibri-Bold") || text.contains("/Carlito-Bold"),
        "Word paints I_am_sharing Heading1 as Calibri-Bold, not Aptos Display; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/Aptos-Bold"),
        "Aptos Display cache must not stay Aptos-Bold; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

fn aptos_display_cloud_font() -> bool {
    let Ok(home) = std::env::var("HOME") else {
        return false;
    };
    let dir = std::path::PathBuf::from(home)
        .join("Library/Group Containers/UBF8T346G9.Office/FontCache/4/CloudFonts/Aptos Display");
    dir.is_dir()
}

#[test]
fn heading1_theme_major_aptos_display_embeds_display_not_body_aptos() {
    // potpourri / file_170: Heading1 is asciiTheme=majorHAnsi and
    // theme major latin is Aptos Display. Word Quartz embeds
    // AptosDisplay. Mapping that slot to body Aptos is the wrong face.
    if !aptos_display_cloud_font() {
        return;
    }
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
               <w:b/><w:sz w:val=\"40\"/>\
             </w:rPr>\
           </w:style>\
         </w:styles>";
    let theme = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
           <a:themeElements><a:fontScheme name=\"Office\">\
             <a:majorFont><a:latin typeface=\"Aptos Display\"/></a:majorFont>\
             <a:minorFont><a:latin typeface=\"Aptos\"/></a:minorFont>\
           </a:fontScheme></a:themeElements>\
         </a:theme>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>\
           <w:r><w:t>DisplayHead</w:t></w:r></w:p>\
         <w:p><w:r><w:t>AptosBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles_and_theme(body, styles, theme))
        .expect("convert Aptos Display major");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/AptosDisplay"),
        "theme major Aptos Display must embed AptosDisplay; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
}

#[test]
fn official_potpourri_heading_embeds_aptos_display() {
    // Word potpourri Quartz embeds AptosDisplay for Title/Heading.
    // Paint-only — potpourri stays 5pp.
    if !aptos_display_cloud_font() {
        return;
    }
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/AptosDisplay"),
        "potpourri Heading1 major=Aptos Display must embed AptosDisplay"
    );
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
}

fn calibri_light_dfont() -> bool {
    std::path::Path::new("/Applications/Microsoft Word.app/Contents/Resources/DFonts/calibril.ttf")
        .is_file()
}

#[test]
fn heading1_theme_major_calibri_light_embeds_light_not_regular() {
    // Strict01 family: Title/Heading1/2 are asciiTheme=majorHAnsi and
    // theme major latin is Calibri Light. Word Quartz embeds
    // Calibri-Light. We fell through to Calibri Regular.
    if !calibri_light_dfont() {
        return;
    }
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
             <w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"/>\
               <w:sz w:val=\"22\"/></w:rPr>\
           </w:style>\
           <w:style w:type=\"paragraph\" w:styleId=\"Heading1\">\
             <w:name w:val=\"heading 1\"/>\
             <w:basedOn w:val=\"Normal\"/>\
             <w:rPr>\
               <w:rFonts w:asciiTheme=\"majorHAnsi\" w:hAnsiTheme=\"majorHAnsi\"/>\
               <w:sz w:val=\"32\"/>\
             </w:rPr>\
           </w:style>\
         </w:styles>";
    let theme = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
           <a:themeElements><a:fontScheme name=\"Office\">\
             <a:majorFont><a:latin typeface=\"Calibri Light\"/></a:majorFont>\
             <a:minorFont><a:latin typeface=\"Calibri\"/></a:minorFont>\
           </a:fontScheme></a:themeElements>\
         </a:theme>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>\
           <w:r><w:t>LightHead</w:t></w:r></w:p>\
         <w:p><w:r><w:t>BodyCalibri</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles_and_theme(body, styles, theme))
        .expect("convert Calibri Light major");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Calibri-Light"),
        "theme major Calibri Light must embed Calibri-Light; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
}

#[test]
fn official_strict01_title_embeds_calibri_light() {
    // Word Strict01 Title/Heading1/2 are Calibri-Light. Paint-only —
    // stay 13pp.
    if !calibri_light_dfont() {
        return;
    }
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Calibri-Light"),
        "Strict01 Title/Heading must embed Calibri-Light"
    );
    assert_eq!(pdf_page_count(&pdf), 13, "Word Strict01 is 13pp");
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
fn exact_line_400_is_twenty_pt_box_not_size_times_eleven() {
    // sd_2517 Ttulo1 / DocumentTitle: w:line=400 lineRule=exact is 20pt.
    // We stored line_mult=20/11 then * 16pt Arial (~29pt), so Article One
    // sat +14pt vs Word and the body started +44pt (beyond align 5px).
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:styleId=\"Ttulo1\">\
             <w:name w:val=\"heading 1\"/>\
             <w:pPr><w:spacing w:after=\"0\" w:line=\"400\" w:lineRule=\"exact\"/>\
               <w:jc w:val=\"center\"/></w:pPr>\
             <w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
               <w:b/><w:sz w:val=\"32\"/></w:rPr>\
           </w:style>\
         </w:styles>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Ttulo1\"/></w:pPr>\
           <w:r><w:t>ExactHead</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Ttulo1\"/></w:pPr>\
           <w:r><w:t>ExactNext</w:t></w:r></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert exact 400");
    let ys = distinct_tf_ys(&pdf, "16.08 Tf");
    assert!(ys.len() >= 2, "two exact headings; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (18.0..=22.0).contains(&gap),
        "line=400 exact is 20pt, not 16×(20/11)≈29; gap={gap} ys={ys:?}"
    );
}

#[test]
fn empty_exact_title_para_keeps_style_face_and_size() {
    // sd_2517 / file_22 cover: TitlePage empty w:p has no w:t. Wrapping
    // that as factory Calibri 11 (not Arial 18 / exact 20 / after 24)
    // skipped Word's 18pt spaces and stretched DocumentTitle→date to 88pt
    // via ascent mismatch (Word grid is 44pt).
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:styleId=\"TitlePage\">\
             <w:name w:val=\"Title Page\"/>\
             <w:pPr><w:spacing w:after=\"480\" w:line=\"400\" w:lineRule=\"exact\"/>\
               <w:jc w:val=\"center\"/></w:pPr>\
             <w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
               <w:b/><w:sz w:val=\"36\"/></w:rPr>\
           </w:style>\
         </w:styles>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"TitlePage\"/></w:pPr>\
           <w:r><w:t>CoverTitle</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"TitlePage\"/></w:pPr></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"TitlePage\"/></w:pPr>\
           <w:r><w:t>CoverDate</w:t></w:r></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert empty TitlePage");
    let mut ys = pdf_tf_ys(&pdf, "18.00 Tf");
    if ys.is_empty() {
        ys = pdf_tf_ys(&pdf, "18 Tf");
    }
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.4);
    assert!(
        ys.len() >= 3,
        "empty TitlePage must paint the 18pt Arial space like Word; ys={ys:?}"
    );
    let gap = ys[0] - ys[1];
    assert!(
        (42.0..=46.0).contains(&gap),
        "TitlePage exact 20 + after 24 is 44pt, not Calibri-11 ascent stretch; gap={gap} ys={ys:?}"
    );
}

#[test]
fn official_sd_2517_cover_paints_title_page_empty_eighteen_pt() {
    // Word cover is 7× Arial 18pt (title, 3 spaces, date, 2 spaces).
    // Factory Calibri 11 on empty TitlePage dropped 3 of those spaces.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official sd_2517"))
        .expect("convert official sd_2517");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "sd_2517 must emit pages");
    let p1 = &pages[0];
    let mut ys = pdf_tf_ys(p1.as_bytes(), "18.00 Tf");
    if ys.is_empty() {
        ys = pdf_tf_ys(p1.as_bytes(), "18 Tf");
    }
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.4);
    assert!(
        ys.len() >= 6,
        "Word cover paints 7 Arial 18pt lines (empty TitlePage spaces); got {} ys={ys:?}",
        ys.len()
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
    assert_eq!(
        n, 107,
        "sd_2517 must match Word 107 pages (leftover TextHeading empty w:br), got {n}"
    );
}

#[test]
fn sd_2517_official_word_track_toc_has_dot_leaders() {
    // Official Word Quartz is 107pp; we are 106 because body page-breaks
    // (not TOC). TOC is its own lowerRoman section (i–iv) in both. Word
    // paints Sumrio right-tab leader dots + webHidden PAGEREFs; skipping
    // webHidden dropped both and the 107-page pairing drifted on p2–p5.
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx",
    )
    .expect("official sd_2517 fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert official sd_2517");
    let n = pdf_page_count(&pdf);
    assert_eq!(
        n, 107,
        "official sd_2517 must stay at Word 107 (TOC must not spill to v), got {n}"
    );
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.matches("(.)").count() >= 80,
        "Sumrio right-tab leader=dot must paint on official TOC pages"
    );
    assert!(
        text.contains("(1-1)") || (text.contains("(1)") && text.contains("(-)")),
        "webHidden TOC PAGEREF must paint; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

#[test]
fn sd_2517_official_word_track_is_107_pages() {
    // Word Quartz oracle is 107. Body page-breaks after a full page skip
    // one page (ch1 1-4, ch13 13-9). Do not regress to 111 by treating
    // every empty w:br type=page as a skip.
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx",
    )
    .expect("official sd_2517 fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert official sd_2517");
    let n = pdf_page_count(&pdf);
    assert_eq!(
        n, 107,
        "official Word-track sd_2517 is 107pp; leftover empty w:br after TextHeading misses 1-4 (106) or overshoots (109); got {n}"
    );
}

#[test]
fn official_sd_2517_toc_page_three_reaches_article_eleven() {
    // Word p3 of the Sumrio ends at aliqua 11-1 / lorem 11.01. We treat
    // w:tab pos as page-edge, so the 2520-twip left tab sits at 126pt
    // and is already behind "lorem 1.01"; the first tab fires the
    // 8640-twip dot leader. Extra title wraps put 11-1 on p4 (ITT 39).
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx",
    )
    .expect("official sd_2517 fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert official sd_2517");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 3, "expected 107pp, got {}", pages.len());
    let p3 = pdf_winansi_text(pages[2].as_bytes());
    assert!(
        p3.contains("11-1") || p3.contains("11.01"),
        "Word TOC p3 reaches article 11-1, not ~9.2; p3={p3}"
    );
}

#[test]
fn missing_pageref_paints_error_bookmark_not_defined() {
    // Word Save-as-PDF recomputes PAGEREF. A name with no w:bookmarkStart
    // paints "Error! Bookmark not defined." at wrap time (sd_2517 /
    // file_22 TOC lorem 9.01–9.02). Cached "1" must not stay.
    let body = "\
         <w:p>\
           <w:r><w:t>Missing </w:t></w:r>\
           <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
           <w:r><w:instrText xml:space=\"preserve\"> PAGEREF _Missing \\h </w:instrText></w:r>\
           <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
           <w:r><w:t>1</w:t></w:r>\
           <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
         </w:p>\
         <w:p>\
           <w:bookmarkStart w:id=\"1\" w:name=\"_Here\"/>\
           <w:r><w:t>Target</w:t></w:r>\
           <w:bookmarkEnd w:id=\"1\"/>\
         </w:p>\
         <w:p>\
           <w:r><w:t>Live </w:t></w:r>\
           <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
           <w:r><w:instrText xml:space=\"preserve\"> PAGEREF _Here \\h </w:instrText></w:r>\
           <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
           <w:r><w:t>99</w:t></w:r>\
           <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
         </w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert missing PAGEREF");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("Error! Bookmark not defined."),
        "Word paints Error! Bookmark not defined. for a missing PAGEREF; painted={painted}"
    );
    assert!(
        painted.contains("Target") && painted.contains("Live") && painted.contains("1"),
        "existing PAGEREF must still live-patch to page 1, not cached 99; painted={painted}"
    );
    assert!(
        !painted.contains("99"),
        "cached PAGEREF 99 must not survive; painted={painted}"
    );
}

#[test]
fn missing_pageref_error_is_bold_like_word_quartz() {
    // sd_2517 / file_22 TOC lorem 9.01: Word paints
    // "Error! Bookmark not defined." in bold. We kept the result run's
    // Sumrio2 roman, so the error sat light vs Word's bold wrap.
    let body = "\
         <w:p>\
           <w:r><w:t>Label </w:t></w:r>\
           <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
           <w:r><w:instrText xml:space=\"preserve\"> PAGEREF _Gone \\h </w:instrText></w:r>\
           <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
           <w:r><w:t>1</w:t></w:r>\
           <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert bold Error!");
    let hay = String::from_utf8_lossy(&pdf);
    assert!(
        hay.contains("/Calibri-Bold"),
        "Word field-error is bold; hay tail {}",
        &hay[hay.len().saturating_sub(240)..]
    );
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("Error! Bookmark not defined."),
        "error text still paints; painted={painted}"
    );
}

#[test]
fn official_sd_2517_toc_missing_pagerefs_paint_error_bookmark_not_defined() {
    // Word p3 wraps PAGEREF _Toc218523836/_Toc218523837 as
    // "Error! Bookmark not defined." (lorem 9.01–9.02). Cached 9-1
    // packs an extra TOC row so our p4 started at 11.03 vs Word 11.02.
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx",
    )
    .expect("official sd_2517 fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert official sd_2517");
    let n = pdf_page_count(&pdf);
    assert_eq!(
        n, 107,
        "Word sd_2517 stays 107pp after Error! wrap; got {n}"
    );
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 4, "expected 107pp, got {}", pages.len());
    let p3 = pdf_winansi_text(pages[2].as_bytes());
    assert!(
        p3.contains("Error! Bookmark") && p3.contains("not defined."),
        "Word TOC p3 paints Error! Bookmark not defined. for missing _Toc; p3={p3}"
    );
    assert!(
        p3.contains("11-1") || p3.contains("11.01"),
        "Word TOC p3 still reaches 11.01 after Error! wrap; p3={p3}"
    );
    let p4 = pdf_winansi_text(pages[3].as_bytes());
    assert!(
        p4.contains("11.02") || p4.contains("11-7"),
        "Word TOC p4 starts at lorem 11.02, not 11.03; p4={p4}"
    );
}

#[test]
fn official_sd_2517_sumrio1_wraps_doloret_like_word() {
    // Sumrio1 hanging first line ignored w:right=720 and packed
    // "dolor'et" onto line 1 (x2≈487). Word wraps it so the hanging
    // continuation (x≈216) starts with dolor'et.
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx",
    )
    .expect("official sd_2517 fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert official sd_2517");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 3, "expected 107pp, got {}", pages.len());
    let mut by_y: std::collections::BTreeMap<i32, Vec<(f32, String)>> =
        std::collections::BTreeMap::new();
    for (x, y, ch) in pdf_tf_glyphs(pages[2].as_bytes(), "12.00 Tf") {
        by_y.entry((y * 2.0).round() as i32)
            .or_default()
            .push((x, ch));
    }
    let hanging = by_y
        .values()
        .filter_map(|row| {
            let min_x = row.iter().map(|(x, _)| *x).fold(f32::INFINITY, f32::min);
            if !(210.0..230.0).contains(&min_x) {
                return None;
            }
            let mut row = row.clone();
            row.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            Some(row.into_iter().map(|(_, ch)| ch).collect::<String>())
        })
        .find(|s| s.contains("labore adipiscing") || s.contains("dolor"));
    let hanging = hanging.unwrap_or_default();
    assert!(
        hanging.starts_with("dolor"),
        "Word p3 Sumrio1 continuation starts with dolor'et; hanging={hanging:?}"
    );
}

fn pdf_winansi_text(pdf: &[u8]) -> String {
    pdf_winansi_literals(pdf).concat()
}

fn pdf_winansi_literals(pdf: &[u8]) -> Vec<String> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(") Tj") {
        let end = from + rel;
        if let Some(start) = hay[..end].rfind('(') {
            let inner = &hay[start + 1..end];
            if inner.chars().all(|c| c.is_ascii_graphic() || c == ' ') && !inner.is_empty() {
                out.push(inner.to_string());
            }
        }
        from = end + 4;
    }
    out
}

#[test]
fn sd_2517_article_section_markers_match_word() {
    // Título1 cardinalText `Article %2` + Título2 decimalZero `Section %2.%3`.
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:abstractNum w:abstractNumId=\"2\">\
             <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
               <w:lvlText w:val=\"%1\"/></w:lvl>\
             <w:lvl w:ilvl=\"1\"><w:start w:val=\"1\"/><w:numFmt w:val=\"cardinalText\"/>\
               <w:lvlText w:val=\"Article %2\"/><w:pStyle w:val=\"Heading1\"/></w:lvl>\
             <w:lvl w:ilvl=\"2\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimalZero\"/>\
               <w:lvlText w:val=\"Section %2.%3\"/><w:pStyle w:val=\"Heading2\"/></w:lvl>\
           </w:abstractNum>\
           <w:num w:numId=\"2\"><w:abstractNumId w:val=\"2\"/></w:num>\
         </w:numbering>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/>\
           <w:numPr><w:ilvl w:val=\"1\"/><w:numId w:val=\"2\"/></w:numPr></w:pPr>\
           <w:r><w:t>sit adipiscing</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading2\"/>\
           <w:numPr><w:ilvl w:val=\"2\"/><w:numId w:val=\"2\"/></w:numPr></w:pPr>\
           <w:r><w:t>consectetur ipsum</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering))).expect("convert markers");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("Article One"),
        "Word paints Article One not Article 1; painted={painted}"
    );
    assert!(
        painted.contains("Section 1.01"),
        "Word paints Section 1.01 not Section 1.1; painted={painted}"
    );
}

#[test]
fn official_image_out_of_folder_omits_deepl_textbox() {
    // Word prints the wrapSquare page-anchor PNG only (text is in the
    // pixels). We also emit the wps txbx "Subscribe to DeepL Pro" as
    // vector text (ITT 41). Skip the editor chrome when the drawing
    // already has a picture blip at page origin.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/image_out_of_folder.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official image_out_of_folder"))
        .expect("convert image_out_of_folder");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        !painted.contains("Subscribe to DeepL"),
        "DeepL banner txbx must not paint as body text; painted={painted}"
    );
    assert!(
        painted.contains("Aristoxeni") || painted.contains("Quantum"),
        "body text must still paint; painted={painted}"
    );
}

#[test]
fn official_image_out_of_folder_banner_uses_xml_extent() {
    // wrapSquare page-origin logo.png is 10690522×807396 EMU = 841.77×63.57
    // on A4 (595.3pt). Word paints that overflow (visible left 595×63.5);
    // page-width clamp squashed it to 595.3×44.96.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/image_out_of_folder.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official image_out_of_folder"))
        .expect("convert image_out_of_folder");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("841.77 0 0 63.57"),
        "DeepL banner must keep xml extent 841.77×63.57; snippet {}",
        text.split("/Im")
            .nth(1)
            .unwrap_or(&text[text.len().saturating_sub(240)..])
    );
}

fn pdf_content_streams(pdf: &[u8]) -> Vec<String> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut rest = hay.as_ref();
    while let Some(i) = rest.find(">>\nstream\n") {
        let after = &rest[i + 10..];
        let Some(end) = after.find("\nendstream") else {
            break;
        };
        let body = &after[..end];
        if body.contains(" Tf") && body.len() < 400_000 {
            out.push(body.to_string());
        }
        rest = &after[end + 1..];
    }
    out
}

fn pdf_tf_unique_ys_in(hay: &str, tf: &str) -> Vec<f32> {
    let mut ys = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tf) {
        let start = from + rel;
        let end = hay.len().min(start + tf.len() + 80);
        let slice = &hay[start..end];
        if let Some(td) = slice.find(" Td") {
            let before = &slice[..td];
            let mut parts = before.rsplit([' ', '\n']);
            if let Some(y) = parts.next().and_then(|s| s.parse::<f32>().ok())
                && ys.last().is_none_or(|prev: &f32| (*prev - y).abs() > 0.4)
            {
                ys.push(y);
            }
        }
        from += rel + tf.len();
    }
    ys
}

#[test]
fn tblw_pct_two_hundred_overflows_the_page() {
    // table_bookmark_end Test 5: tblW 10000/5000 = 200%. Word does not
    // shrink that to the content box (we did, so all 5 columns sat inside
    // the 468pt measure). 200% of 468 is 936pt; column 4 starts past 540.
    let body = "<w:tbl><w:tblPr>\
           <w:tblW w:w=\"10000\" w:type=\"pct\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid>\
             <w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/>\
             <w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/>\
             <w:gridCol w:w=\"2000\"/>\
           </w:tblGrid>\
           <w:tr>\
             <w:tc><w:p><w:r><w:t>C1</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>C2</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>C3</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>C4</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>C5</w:t></w:r></w:p></w:tc>\
           </w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 200% pct");
    let xs = pdf_vertical_rule_xs(&pdf);
    assert!(
        xs.iter().any(|x| *x > 550.0),
        "200% of 468pt must overflow past the 540pt right margin; xs={xs:?}"
    );
}

#[test]
fn tblw_dxa_eight_inch_overflows_the_page() {
    // table_bookmark_end Test 2: tblW 12000 twips = 600pt = 8.33in.
    // Measure is 432pt (90+90). Capping dxa packed four 108pt columns;
    // Word keeps 150pt and the fourth cell starts at 90+450=540.
    let body = "<w:tbl><w:tblPr>\
           <w:tblW w:w=\"12000\" w:type=\"dxa\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid>\
             <w:gridCol w:w=\"3000\"/><w:gridCol w:w=\"3000\"/>\
             <w:gridCol w:w=\"3000\"/><w:gridCol w:w=\"3000\"/>\
           </w:tblGrid>\
           <w:tr>\
             <w:tc><w:p><w:r><w:t>C1</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>C2</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>C3</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>C4</w:t></w:r></w:p></w:tc>\
           </w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1800\" w:bottom=\"1440\" w:left=\"1800\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 8.33in dxa");
    let cells = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(
        cells.iter().any(|x| *x > 500.0),
        "12000-twip dxa must put C4 near Word 540, not cap at 432pt; xs={cells:?}"
    );
    let rules = pdf_vertical_rule_xs(&pdf);
    assert!(
        rules.iter().any(|x| *x > 530.0),
        "8.33in dxa right edge must overflow the 522pt margin; rules={rules:?}"
    );
}

#[test]
fn tbl_ind_stays_ignored_after_mini_tblind() {
    // mini 233: applying dxa tblInd (file_146 ±4/5 twips) dropped
    // no-redline mean −0.0025 / median −0.011. Keep indent 0.
    let body = "<w:tbl><w:tblPr>\
           <w:tblInd w:w=\"1440\" w:type=\"dxa\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"2880\"/></w:tblGrid>\
           <w:tr><w:tc><w:p><w:r><w:t>Indented</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert tblInd lock");
    let rules = pdf_vertical_rule_xs(&pdf);
    assert!(
        rules.iter().any(|x| *x > 70.0 && *x < 74.0),
        "mini tblind was ITT-neg; keep table at margin 72, not +72pt; rules={rules:?}"
    );
    assert!(
        !rules.iter().any(|x| *x > 140.0 && *x < 148.0),
        "mini tblind was ITT-neg; must not honor 1440-twip tblInd; rules={rules:?}"
    );
}

#[test]
fn official_table_bookmark_test_two_fourth_col_sits_at_word_540() {
    // Word Test 2 (8.33in): four 150pt columns, R1C4 at x=540. Capping
    // 12000 twips to the 432pt measure packed C4 at ~419 (span 324).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/table_bookmark_end.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official table_bookmark_end"))
        .expect("convert table_bookmark_end");
    assert_eq!(pdf_page_count(&pdf), 2, "Word table_bookmark_end is 2pp");
    let pages = pdf_content_streams(&pdf);
    let mut grouped: Vec<(f32, Vec<f32>)> = Vec::new();
    for (x, y) in pdf_device_xy(&pages[0], "46 Tf") {
        if let Some((ly, xs)) = grouped.last_mut()
            && (*ly - y).abs() <= 0.6
        {
            xs.push(x);
        } else {
            grouped.push((y, vec![x]));
        }
    }
    let test2 = grouped.iter().any(|(_, xs)| {
        let mut v = xs.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut cells = Vec::new();
        for x in v {
            if cells.last().is_none_or(|prev| x - prev > 20.0) {
                cells.push(x);
            }
        }
        cells.len() >= 4 && cells[3] > 520.0 && (cells[3] - cells[0] - 450.0).abs() < 25.0
    });
    assert!(
        test2,
        "Word Test 2 is 150pt×4 ending at 540; grouped={grouped:?}"
    );
}

#[test]
fn official_table_bookmark_end_keeps_seven_tests_on_page_one() {
    // Word: Tests 1–7 on page 1, Test 8 on page 2. The 200% pct table
    // (Test 5) overflows so only ~3 of 5 columns are on the page; we
    // shrank it and the empty Normal after each table ate a line, so
    // Test 7 spilled.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/table_bookmark_end.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official table_bookmark_end"))
        .expect("convert table_bookmark_end");
    let n = pdf_page_count(&pdf);
    assert_eq!(n, 2, "Word table_bookmark_end is 2pp; got {n}");
    let pages = pdf_content_streams(&pdf);
    assert!(
        pages.len() >= 2,
        "expected 2 page streams, got {}",
        pages.len()
    );
    let h2 = pdf_tf_unique_ys_in(&pages[0], "13.00 Tf");
    assert!(
        h2.len() >= 7,
        "Word keeps Tests 1–7 (Heading2 13pt) on page 1; got {} ys={h2:?}",
        h2.len()
    );
    let xs = pdf_vertical_rule_xs(&pdf);
    assert!(
        xs.iter().any(|x| *x > 530.0),
        "Test 5 200% pct must overflow past the 522pt right margin; xs={xs:?}"
    );
}

#[test]
fn official_table_bookmark_end_body_stays_calibri_after_mini_90() {
    // Word Quartz paints table_bookmark_end body as Cambria (factory
    // minorHAnsi → theme minor). Resolving that slot (mini 90) lifted
    // this fixture and file_134 ~+2 ITT but dropped file_2 / file_41
    // −2.5 each (Cambria size×1.15 line vs Word ~14.9). Keep Calibri.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/table_bookmark_end.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official table_bookmark_end"))
        .expect("convert table_bookmark_end");
    assert_eq!(pdf_page_count(&pdf), 2, "Word table_bookmark_end is 2pp");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("/Cambria"),
        "factory Cambria minor stays Calibri after mini 90 ITT; tail {}",
        &text[text.len().saturating_sub(320)..]
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
    let ys = pdf_tf_ys(&pdf, "11.04 Tf");
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
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
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
fn table_cell_keeps_xml_space_padding() {
    // sample/eigenpal npm+github row: each cell is `label` + many
    // xml:space spaces + courier package. collapse_ws made @eigenpal
    // sit 2pt after npm; Word leaves ~33pt (padding is the column).
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4680\"/></w:tblGrid>\
         <w:tr><w:tc><w:p>\
           <w:r><w:t xml:space=\"preserve\">npm               </w:t></w:r>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>\
             <w:sz w:val=\"19\"/></w:rPr>\
             <w:t xml:space=\"preserve\">@eigenpal/pkg</w:t></w:r>\
         </w:p></w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert npm cell");
    let npm = ["11.04 Tf", "10.08 Tf"]
        .iter()
        .flat_map(|tf| pdf_tf_xs(&pdf, tf))
        .fold(f32::INFINITY, f32::min);
    let pkg = ["9.60 Tf", "9.50 Tf", "9.12 Tf"]
        .iter()
        .flat_map(|tf| pdf_tf_xs(&pdf, tf))
        .fold(f32::INFINITY, f32::min);
    assert!(
        npm.is_finite() && pkg.is_finite(),
        "npm label and package must paint; npm={npm} pkg={pkg}"
    );
    assert!(
        pkg - npm > 40.0,
        "cell xml:space padding must push the package name; npm={npm} pkg={pkg}"
    );
}

#[test]
fn footer_xml_space_padding_stays_collapsed_after_mini_88() {
    // Word file_146 footer is `Page       1of       7`. Keeping that
    // padding (mini 88) dropped every sample/file_146 stem ~0.10 ITT.
    let footer = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:r><w:t xml:space=\"preserve\">Page       </w:t></w:r>\
             <w:r><w:t>1</w:t></w:r>\
             <w:r><w:t xml:space=\"preserve\">of       </w:t></w:r>\
             <w:r><w:t>2</w:t></w:r>\
             <w:r><w:t xml:space=\"preserve\">·       eigenpal.com</w:t></w:r></w:p></w:ftr>";
    let body = "<w:p><w:r><w:t>Body</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", footer.to_string())],
    ))
    .expect("convert padded footer");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("Page       "),
        "HF xml:space padding is ITT-wrong on sample/file_146; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

#[test]
fn official_sample_npm_package_sits_after_label() {
    // Word p1 npm badge: npm at ~80, @eigenpal at ~133. Collapsed
    // padding parked the package at npm+2.
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/word_based/docx_source/sample_document_really_repaired_word_repaired.docx",
    )
    .expect("sample_document fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert sample_document");
    let pages = pdf_content_streams(&pdf);
    let p1 = pages[0].as_bytes();
    let npm = pdf_tf_xs(p1, "10.08 Tf")
        .into_iter()
        .filter(|x| (70.0..90.0).contains(x))
        .fold(f32::INFINITY, f32::min);
    let pkg = ["9.60 Tf", "9.50 Tf"]
        .iter()
        .flat_map(|tf| pdf_tf_xs(p1, tf))
        .filter(|x| (120.0..180.0).contains(x))
        .fold(f32::INFINITY, f32::min);
    assert!(
        npm.is_finite() && pkg.is_finite(),
        "sample npm badge must paint; npm={npm} pkg={pkg}"
    );
    assert!(
        pkg - npm > 40.0,
        "Word npm badge keeps preserve spaces; npm={npm} pkg={pkg}"
    );
}

#[test]
fn courier_sz19_stays_nine_point_five_after_mini_99() {
    // Word Quartz snaps 9.5 → 40 ppem → 9.60. Painting that (mini 99)
    // dropped file_175 −0.41 and eigenpal_2 −0.52; keep 9.50.
    let body = "<w:p><w:r><w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>\
           <w:sz w:val=\"19\"/></w:rPr><w:t>CourierNineFive</w:t></w:r></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Courier 9.5");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("9.50 Tf"),
        "sz=19 stays 9.50 after mini 99; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("9.60 Tf"),
        "9.60 snap is ITT-wrong; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn official_file_146_courier_stays_nine_point_five_after_mini_99() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    let text = String::from_utf8_lossy(&pdf);
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    assert!(
        text.contains("9.50 Tf"),
        "official Courier sz=19 stays 9.50 after mini 99"
    );
    assert!(
        !text.contains("9.60 Tf"),
        "9.60 snap is ITT-wrong on file_146"
    );
}

#[test]
fn courier_table_cell_stays_eleven_line_box_after_mini_114() {
    // Word Quartz Courier 9.5 listings sit on ~10.8pt. Painting size×1.15
    // (mini 114) dropped the sample/file_146 family 0.05–0.42 ITT. Keep 12.65.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"3600\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>\
             <w:sz w:val=\"19\"/></w:rPr>\
           <w:t>aaaa bbbb cccc dddd eeee ffff gggg hhhh iiii jjjj kkkk \
llll mmmm nnnn oooo pppp qqqq rrrr ssss tttt uuuu vvvv</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Courier cell");
    let ys = distinct_tf_ys(&pdf, "9.50 Tf");
    assert!(
        ys.len() >= 2,
        "Courier cell must wrap to 2+ lines; ys={ys:?}"
    );
    let gap = (ys[0] - ys[1]).abs();
    assert!(
        (12.4..=12.9).contains(&gap),
        "mini 114 Courier size×1.15 was ITT-wrong; keep 11×1.15=12.65; gap={gap} ys={ys:?}"
    );
}

#[test]
fn official_file_146_cambria_body_uses_word_auto_leading() {
    // Word Inter→Cambria 11 / auto is size×1.15 (~12.65; measured 13.2).
    // Body baselines currently sit ~11.36pt apart (size×1 + 1pt remainder),
    // packing "Serialises to w:ins" onto page 1 (Word starts it on page 2).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_146"))
        .expect("convert official file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let hay = String::from_utf8_lossy(&pdf);
    let mut ys: Vec<f32> = pdf_device_xy(hay.as_ref(), "46 Tf")
        .into_iter()
        .map(|(_, y)| (y * 20.0).round() / 20.0)
        .collect();
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup();
    let gaps: Vec<f32> = ys
        .windows(2)
        .map(|w| w[0] - w[1])
        .filter(|g| (10.0..14.5).contains(g))
        .collect();
    let med = {
        let mut g = gaps.clone();
        g.sort_by(|a, b| a.partial_cmp(b).unwrap());
        g[g.len() / 2]
    };
    assert!(
        med >= 12.4,
        "Cambria 11 auto must be size×1.15 not size×1; median gap={med} gaps={gaps:?}"
    );
}

#[test]
fn official_file_146_stays_seven_pages_after_mini_114() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
}

fn pdf_cm_tj_xy(hay: &str, lit: &str) -> Vec<(f32, f32)> {
    // 11pt paints as `q 0.24 0 0 0.24 x y cm BT /Calibri 46 Tf … 0 0 Td (.) Tj`.
    let needle = format!("({lit}) Tj");
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(&needle) {
        let i = from + rel;
        let window = &hay[i.saturating_sub(160)..i];
        if let Some(cm) = window.rfind("0.24 0 0 0.24 ") {
            let rest = &window[cm + "0.24 0 0 0.24 ".len()..];
            let mut parts = rest.split_whitespace();
            if let (Some(x), Some(y)) = (
                parts.next().and_then(|s| s.parse().ok()),
                parts.next().and_then(|s| s.parse().ok()),
            ) {
                out.push((x, y));
            }
        }
        from = i + needle.len();
    }
    out
}

fn pdf_tj_xy(hay: &str, lit: &str) -> Vec<(f32, f32)> {
    let needle = format!("({lit}) Tj");
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(&needle) {
        let i = from + rel;
        let window = &hay[i.saturating_sub(160)..i];
        let mut last = None;
        let mut wfrom = 0;
        while let Some(td) = window[wfrom..].find(" Td") {
            let before = window[..wfrom + td].trim_end();
            let mut parts = before.rsplitn(3, char::is_whitespace);
            let y = parts.next().and_then(|s| s.parse().ok());
            let x = parts.next().and_then(|s| s.parse().ok());
            if let (Some(x), Some(y)) = (x, y) {
                last = Some((x, y));
            }
            wfrom += td + 3;
        }
        if let Some(xy) = last {
            out.push(xy);
        }
        from = i + needle.len();
    }
    out
}

#[test]
fn official_file_146_footer_numpages_does_not_open_a_hole() {
    // Word footer is "Page 1of 7· eigenpal.com". NUMPAGES is painted as
    // @@N@@ then patched to "7"; advancing x by the mark (~45pt) left a
    // hole so middot sat at 333 vs Word 298 (7 at 291).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let hay = String::from_utf8_lossy(&pdf);
    let footer_seven = pdf_tj_xy(&hay, "7")
        .into_iter()
        .filter(|(_, y)| *y < 80.0)
        .map(|(x, _)| x)
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("footer NUMPAGES 7");
    let mut dots = pdf_tj_xy(&hay, "\\267 ");
    if dots.is_empty() {
        dots = pdf_tj_xy(&hay, "\\267");
    }
    let footer_dot = dots
        .into_iter()
        .filter(|(_, y)| *y < 80.0)
        .map(|(x, _)| x)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .expect("footer middot");
    assert!(
        (footer_dot - footer_seven) > 0.0 && (footer_dot - footer_seven) < 16.0,
        "Word glues 7·; NUMPAGES @@N@@ advance opened a hole; 7={footer_seven} middot={footer_dot} gap={}",
        footer_dot - footer_seven
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
fn page_field_uses_sectpr_lower_roman_start() {
    // sd_2517 TOC section is w:pgNumType w:fmt="lowerRoman" w:start="1".
    // PAGE used the document page index ("2"), so every front-matter
    // footer missed Word's "i" and the 107-page pairing drifted.
    let footer = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText>PAGE</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:t>1</w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p></w:ftr>";
    let body = "<w:p><w:r><w:t>TitlePage</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr w:rsidR=\"00000000\">\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/>\
           <w:type w:val=\"nextPage\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>TocBody</w:t></w:r></w:p>\
         <w:sectPr>\
           <w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/>\
           <w:pgNumType w:fmt=\"lowerRoman\" w:start=\"1\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", footer.to_string())],
    ))
    .expect("convert roman PAGE");
    assert_eq!(pdf_page_count(&pdf), 2, "title + TOC section");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("(i)"),
        "PAGE in a lowerRoman section must paint i, not the document index; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("(2)"),
        "document page 2 must not stay arabic 2 under lowerRoman; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn page_field_continues_across_section_without_start() {
    // comments-lots / I_am_sharing: three sectPr (portrait, landscape,
    // portrait) and no w:pgNumType start. Word continues PAGE (6/7/8/9).
    // apply_section always set section_page = page_num_start (default 1),
    // so the landscape page and the following portrait restart at 1.
    let footer = page_footer_xml();
    let body = "<w:p><w:r><w:t>Alpha</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/>\
           <w:type w:val=\"nextPage\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Land</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:pgSz w:w=\"15840\" w:h=\"12240\" w:orient=\"landscape\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/>\
           <w:type w:val=\"nextPage\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:t>Bravo</w:t></w:r></w:p>\
         <w:sectPr>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", footer)],
    ))
    .expect("convert continued PAGE");
    assert_eq!(pdf_page_count(&pdf), 3, "portrait + landscape + portrait");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("(1)") && text.contains("(2)") && text.contains("(3)"),
        "PAGE must continue 1/2/3 when later sectPr omit start; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

fn heading1_before_480_styles() -> &'static str {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"paragraph\" w:styleId=\"Heading1\">\
            <w:name w:val=\"heading 1\"/>\
            <w:pPr><w:spacing w:before=\"480\" w:after=\"0\"/></w:pPr>\
            <w:rPr><w:b/><w:sz w:val=\"28\"/></w:rPr>\
          </w:style>\
        </w:styles>"
}

#[test]
fn heading_before_applies_after_section_break() {
    // comments-lots Heading1 before=480 twips (24pt). Word still applies
    // that space after a nextPage sectPr (p6/p7 headings sit below overflow
    // page-top headings). We skip before whenever at_page_top, so the
    // landscape + following portrait pack extra bullets and starve page 9.
    let sect = "<w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
         <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/>";
    let heading = "<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>\
         <w:r><w:rPr><w:sz w:val=\"28\"/></w:rPr><w:t>AfterBreak</w:t></w:r></w:p>";
    let after_section = format!(
        "<w:p><w:r><w:t>Alpha</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:type w:val=\"nextPage\"/>{sect}</w:sectPr></w:pPr></w:p>\
         {heading}<w:sectPr>{sect}</w:sectPr>"
    );
    let after_page = format!(
        "<w:p><w:r><w:t>Alpha</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         {heading}<w:sectPr>{sect}</w:sectPr>"
    );
    let styles = heading1_before_480_styles();
    let section_pdf =
        docx_to_pdf(&docx_with_styles(&after_section, styles)).expect("convert section before");
    let page_pdf =
        docx_to_pdf(&docx_with_styles(&after_page, styles)).expect("convert page before");
    assert_eq!(pdf_page_count(&section_pdf), 2);
    assert_eq!(pdf_page_count(&page_pdf), 2);
    let section_y = pdf_tf_ys(&section_pdf, "14.00 Tf")
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max);
    let page_y = pdf_tf_ys(&page_pdf, "14.00 Tf")
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        page_y - section_y >= 20.0,
        "Heading1 before=24pt must apply after nextPage sectPr, not after a page break; section_y={section_y} page_y={page_y}"
    );
}

#[test]
fn heading_after_callout_table_keeps_word_before_gap() {
    // comments-lots p7: Word's yellow 1-cell callout is ~55pt (vAlign
    // center + docDefaults after inside the cell). We used to emit 38pt
    // (3×11×1.15) and then steal the missing 17pt as white after the
    // table (mini 55/56). That packed two extra bullets onto p7 and
    // XOR-missed the tall yellow box.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid>\
         <w:tr><w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"FFF2CC\"/>\
           <w:vAlign w:val=\"center\"/></w:tcPr>\
           <w:p><w:r><w:rPr><w:b/><w:sz w:val=\"22\"/></w:rPr>\
             <w:t>DemoLine</w:t></w:r>\
             <w:r><w:br/></w:r>\
             <w:r><w:t>Open this section in Word, turn on Track Changes, add a comment with an @mention, then export to PDF to show the complete workflow.</w:t></w:r></w:p>\
         </w:tc></w:tr></w:tbl>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"28\"/></w:rPr><w:t>AfterTable</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, heading1_before_480_styles()))
        .expect("convert table then Heading1");
    let yellow = pdf_fill_hs(&pdf, 1.0, 0.949, 0.8);
    let cell_h = yellow.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        cell_h >= 50.0,
        "Word callout cell is ~55pt yellow, not 3×11×1.15=38; cell_h={cell_h} fills={yellow:?}"
    );
    let demo_ys = pdf_tf_ys(&pdf, "11.04 Tf");
    let head_ys = pdf_tf_ys(&pdf, "14.00 Tf");
    assert!(
        !demo_ys.is_empty() && !head_ys.is_empty(),
        "both must paint"
    );
    let head_y = head_ys.into_iter().fold(f32::NEG_INFINITY, f32::max);
    let demo_y = demo_ys
        .into_iter()
        .filter(|y| *y > head_y + 2.0)
        .fold(f32::INFINITY, f32::min);
    let gap = demo_y - head_y;
    assert!(
        gap >= 46.0,
        "taller callout must keep Word's ~49pt last-line-to-H1; demo_y={demo_y} head_y={head_y} gap={gap}"
    );
}

#[test]
fn unstyled_filled_cell_without_valign_stays_compact() {
    // sample/eigenpal code/npm cells are unstyled + filled but not
    // vAlign=center callouts. The 18pt Demo pad (mini 57) dropped those
    // ~4 ITT while comments-lots only needs the vAlign boxes.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid>\
         <w:tr><w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"FFF2CC\"/></w:tcPr>\
           <w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr>\
             <w:t>DemoLine</w:t></w:r>\
             <w:r><w:br/></w:r>\
             <w:r><w:t>Open this section in Word, turn on Track Changes, add a comment with an @mention, then export to PDF to show the complete workflow.</w:t></w:r></w:p>\
         </w:tc></w:tr></w:tbl>\
         <w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>AfterBody</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, heading1_before_480_styles()))
        .expect("convert filled cell without vAlign");
    let yellow = pdf_fill_hs(&pdf, 1.0, 0.949, 0.8);
    let cell_h = yellow.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        cell_h < 45.0,
        "unstyled fill without vAlign must stay 3×11×1.15=38, not Demo 55; cell_h={cell_h} fills={yellow:?}"
    );
}

#[test]
fn two_col_valign_banner_stays_compact() {
    // addition_removal / file_27 page 1: "Positioning thesis" is a
    // 2-cell unstyled D9EAF7 vAlign-center banner. The +18pt pad is
    // for 1-cell yellow Demo callouts (Word ~55). Applied here the
    // banner is 68pt vs Word ~40 and shifts the whole 12pp pairing.
    let body = "<w:tbl><w:tblGrid>\
           <w:gridCol w:w=\"2200\"/><w:gridCol w:w=\"7880\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"D9EAF7\"/>\
             <w:vAlign w:val=\"center\"/></w:tcPr>\
             <w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr>\
               <w:t>Positioning thesis</w:t></w:r></w:p></w:tc>\
           <w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"D9EAF7\"/>\
             <w:vAlign w:val=\"center\"/></w:tcPr>\
             <w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr>\
               <w:t>Word provides the real-time collaboration people expect from modern cloud editors while adding deeper professional document production, formal review, rich desktop features, enterprise security, automation, accessibility, and file-format control.</w:t></w:r></w:p></w:tc>\
         </w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, heading1_before_480_styles()))
        .expect("convert two-col banner");
    let blue = pdf_fill_hs(&pdf, 0.851, 0.918, 0.969);
    let cell_h = blue.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        cell_h > 8.0 && cell_h < 56.0,
        "2-col thesis banner must stay compact, not Demo +18=68; cell_h={cell_h} fills={blue:?}"
    );
}

#[test]
fn official_addition_removal_page_one_thesis_is_compact() {
    // 1-cell D9EAF7 + br wraps to 4 lines. Word ~40pt; +18 Demo pad
    // made 68.6 (4×12.65+18) and shifted the 12pp pairing.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments_addition_removal_redline_removal_v_addition.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official addition_removal"))
        .expect("convert addition_removal");
    let blue = pdf_fill_hs(&pdf, 0.851, 0.918, 0.969);
    let cell_h = blue.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        cell_h > 8.0 && cell_h < 56.0,
        "official page-1 thesis banner must match Word ~40pt, not 68; cell_h={cell_h} fills={blue:?}"
    );
}

#[test]
fn tblprchange_grid_is_not_the_live_grid() {
    // addition_removal capability matrix: tblPrChange stores a 13-col
    // ghost grid (two 5-twip columns). first_named(tblGrid) finds that
    // descendant first; Word uses the table's own 4-col tblGrid.
    let body = "<w:tbl><w:tblPr>\
           <w:tblPrChange w:id=\"1\" w:author=\"A\" w:date=\"2026-01-01T00:00:00Z\">\
             <w:tblPr><w:tblGrid>\
               <w:gridCol w:w=\"5\"/><w:gridCol w:w=\"5\"/>\
               <w:gridCol w:w=\"2505\"/><w:gridCol w:w=\"2505\"/>\
             </w:tblGrid></w:tblPr>\
           </w:tblPrChange>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid>\
             <w:gridCol w:w=\"2505\"/><w:gridCol w:w=\"2505\"/>\
             <w:gridCol w:w=\"2508\"/><w:gridCol w:w=\"2552\"/>\
           </w:tblGrid>\
           <w:tr>\
             <w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>D</w:t></w:r></w:p></w:tc>\
           </w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert tblPrChange grid");
    let xs = pdf_vertical_rule_xs(&pdf);
    let unique: Vec<i32> = {
        let mut u: Vec<i32> = xs.iter().map(|x| (*x * 2.0).round() as i32).collect();
        u.sort();
        u.dedup();
        u
    };
    assert!(
        unique.len() >= 4 && unique.len() <= 6,
        "live 4-col tblGrid must win over tblPrChange 5-twip cols; xs={xs:?}"
    );
    assert!(
        xs.iter().any(|x| *x > 500.0),
        "live 4-col grid (capped at 540) must win; tblPrChange ghost ends ~323; xs={xs:?}"
    );
}

#[test]
fn official_addition_removal_capability_matrix_stays_four_columns() {
    // Word p3 is the 4-col capability matrix only. A tblPrChange 13-col
    // ghost grid wrapped the last header into a hairline column so
    // "2. Evidence" leaked onto page 3 (ITT 36).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments_addition_removal_redline_removal_v_addition.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official addition_removal"))
        .expect("convert addition_removal");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 3, "expected 12pp, got {}", pages.len());
    let xs = pdf_vertical_rule_xs(pages[2].as_bytes());
    let unique: Vec<i32> = {
        let mut u: Vec<i32> = xs.iter().map(|x| (*x / 8.0).round() as i32).collect();
        u.sort();
        u.dedup();
        u
    };
    assert!(
        unique.len() <= 12,
        "Word p3 matrix is 4 columns, not a 13-col ghost; n={} xs={xs:?}",
        unique.len()
    );
    assert!(
        xs.iter().any(|x| (175.0..185.0).contains(x))
            && xs.iter().any(|x| (425.0..440.0).contains(x)),
        "4-col TableGrid (~2505 twips) must be on page 3; xs={xs:?}"
    );
}

#[test]
fn tcprchange_gridspan_does_not_pad_extra_columns() {
    // addition_removal remnant table: live tblGrid is 1684/1868/1747/4781
    // (Word's four content columns). The first three cells store
    // gridSpan=2 only inside tcPrChange. first_named picked that up,
    // occupancy became 7, and the live Bottom-line cell sat in an ~50pt
    // padded column (one word per line) instead of the 4781-twip last col.
    let body = "<w:tbl><w:tblPr>\
           <w:tblW w:w=\"0\" w:type=\"auto\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid>\
             <w:gridCol w:w=\"1684\"/><w:gridCol w:w=\"1868\"/>\
             <w:gridCol w:w=\"1747\"/><w:gridCol w:w=\"4781\"/>\
           </w:tblGrid>\
           <w:tr>\
             <w:tc><w:tcPr><w:tcW w:w=\"2520\" w:type=\"dxa\"/>\
               <w:cellDel w:id=\"1\"/>\
               <w:tcPrChange w:id=\"2\" w:author=\"A\">\
                 <w:tcPr><w:tcW w:w=\"2520\" w:type=\"dxa\"/>\
                   <w:gridSpan w:val=\"2\"/></w:tcPr>\
               </w:tcPrChange></w:tcPr>\
               <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"2520\" w:type=\"dxa\"/>\
               <w:cellDel w:id=\"3\"/>\
               <w:tcPrChange w:id=\"4\" w:author=\"A\">\
                 <w:tcPr><w:tcW w:w=\"2520\" w:type=\"dxa\"/>\
                   <w:gridSpan w:val=\"2\"/></w:tcPr>\
               </w:tcPrChange></w:tcPr>\
               <w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"2520\" w:type=\"dxa\"/>\
               <w:cellDel w:id=\"5\"/>\
               <w:tcPrChange w:id=\"6\" w:author=\"A\">\
                 <w:tcPr><w:tcW w:w=\"2520\" w:type=\"dxa\"/>\
                   <w:gridSpan w:val=\"2\"/></w:tcPr>\
               </w:tcPrChange></w:tcPr>\
               <w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc>\
             <w:tc><w:tcPr><w:tcW w:w=\"10080\" w:type=\"dxa\"/></w:tcPr>\
               <w:p><w:r><w:t>Bottom line Google Docs is excellent for lightweight browser-first collaboration.</w:t></w:r></w:p></w:tc>\
           </w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert tcPrChange gridSpan");
    let xs = pdf_vertical_rule_xs(&pdf);
    let unique: Vec<i32> = {
        let mut u: Vec<i32> = xs.iter().map(|x| (*x / 8.0).round() as i32).collect();
        u.sort();
        u.dedup();
        u
    };
    assert!(
        unique.len() <= 6,
        "live 4-col tblGrid must ignore tcPrChange gridSpan=2; n={} xs={xs:?}",
        unique.len()
    );
    // Word All Markup appends a Deleted Cells column, so the last live
    // col starts ~284 (5-col scale), not ~318 (4-col only). Occupancy 7
    // from tcPrChange gridSpan=2 would park that rule near 490.
    assert!(
        xs.iter().any(|x| (275.0..295.0).contains(x)),
        "last live col starts ~284 after Deleted Cells extra col; xs={xs:?}"
    );
    assert!(
        xs.iter().any(|x| (465.0..490.0).contains(x)),
        "Deleted Cells extra col starts ~476; xs={xs:?}"
    );
}

#[test]
fn official_addition_removal_page_three_has_capability_matrix() {
    // Word p3 is exec summary + the 10-row deleted TableGrid (Capability /
    // Real-time coauthoring / …). omit_fully_deleted_tablegrid dropped
    // that table, so p3 started at the remnant “AI assistance / Bottom
    // line” row and ITT pairing collapsed (36).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments_addition_removal_redline_removal_v_addition.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official addition_removal"))
        .expect("convert addition_removal");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 3, "expected 12pp, got {}", pages.len());
    let p3 = pdf_winansi_text(pages[2].as_bytes());
    assert!(
        p3.contains("Google Docs expectation") || p3.contains("Suggesting mode"),
        "Word p3 paints the deleted capability matrix header/rows, not only the remnant; p3={p3}"
    );
}

#[test]
fn official_addition_removal_paints_deleted_cells_label() {
    // Word p4 remnant: 4 live cols plus a trailing “Deleted Cells”
    // stamp for the three w:cellDel cells. Mini 59 rewrote every
    // trPr/del row to that label and dropped addition* −5 ITT.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments_addition_removal_redline_removal_v_addition.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official addition_removal"))
        .expect("convert addition_removal");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("Deleted Cells"),
        "Word remnant table stamps Deleted Cells for cellDel; painted={painted}"
    );
    assert!(
        painted.contains("coauthoring") || painted.contains("Capability"),
        "trPr/del capability matrix must still paint, not become Deleted Cells; painted={painted}"
    );
}

#[test]
fn celldel_appends_deleted_cells_column() {
    // addition_removal remnant: live cells keep their delText; Word
    // adds a trailing Deleted Cells column for w:cellDel. A trPr/del
    // row (the capability matrix) must not be rewritten to that label.
    let body = "<w:tbl><w:tblPr>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"2520\"/><w:gridCol w:w=\"2520\"/></w:tblGrid>\
           <w:tr>\
             <w:tc><w:tcPr><w:cellDel w:id=\"1\"/></w:tcPr>\
               <w:p><w:del w:id=\"2\"><w:r><w:delText>GoneCol</w:delText></w:r></w:del></w:p></w:tc>\
             <w:tc><w:p><w:r><w:t>LiveCol</w:t></w:r></w:p></w:tc>\
           </w:tr></w:tbl>\
         <w:tbl><w:tblPr><w:tblStyle w:val=\"TableGrid\"/></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
           <w:tr><w:trPr><w:del w:id=\"3\"/></w:trPr>\
             <w:tc><w:p><w:del w:id=\"4\"><w:r><w:delText>RowGone</w:delText></w:r></w:del></w:p></w:tc>\
           </w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert cellDel");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("Deleted Cells"),
        "cellDel row must stamp Deleted Cells; painted={painted}"
    );
    assert!(
        painted.contains("GoneCol") && painted.contains("LiveCol"),
        "cellDel cells keep their text; painted={painted}"
    );
    assert!(
        painted.contains("RowGone"),
        "trPr/del row must keep delText, not become Deleted Cells only; painted={painted}"
    );
}

#[test]
fn unstyled_table_then_body_keeps_compact_after() {
    // sample/eigenpal: unstyled tables sit in body, not before Heading1.
    // Global after=10 (mini 55) added 6pt after every such table.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid>\
         <w:tr><w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"FFF2CC\"/></w:tcPr>\
           <w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr>\
             <w:t>DemoLine</w:t></w:r>\
             <w:r><w:br/></w:r>\
             <w:r><w:t>Open this section in Word, turn on Track Changes, add a comment with an @mention, then export to PDF to show the complete workflow.</w:t></w:r></w:p>\
         </w:tc></w:tr></w:tbl>\
         <w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>AfterBody</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, heading1_before_480_styles()))
        .expect("convert table then body");
    let demo_ys = pdf_tf_ys(&pdf, "11.04 Tf");
    assert!(
        demo_ys.len() >= 2,
        "demo last line and body must paint; ys={demo_ys:?}"
    );
    let mut ys = demo_ys;
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    let gap = ys[0] - ys[1];
    assert!(
        gap < 46.0,
        "unstyled table then Normal must keep 4pt chrome, not docDefaults 10pt; gap={gap} ys={ys:?}"
    );
}

#[test]
fn unstyled_table_then_heading_keeps_four_pt_chrome_after_mini_tblafter() {
    // file_146 heading is 4pt below Word, but dropping unstyled
    // after.max(4) (12 tables × 4pt) packed official file_146 7→6pp.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4680\"/></w:tblGrid>\
         <w:tr><w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"FF0000\"/></w:tcPr>\
           <w:p><w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr>\
             <w:t>CellInk</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:p><w:pPr><w:spacing w:before=\"300\" w:after=\"80\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr>\
             <w:t>HeadAfterTbl</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert table then heading");
    let mut ys = pdf_tf_ys(&pdf, "11.04 Tf");
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.4);
    assert!(
        ys.len() >= 2,
        "CellInk and HeadAfterTbl must paint; ys={ys:?}"
    );
    let gap = ys[0] - ys[1];
    assert!(
        (38.5..42.0).contains(&gap),
        "mini tblafter packed file_146 7→6pp; keep 4pt unstyled table chrome; gap={gap} ys={ys:?}"
    );
}

fn page_footer_xml() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText>PAGE</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:t>1</w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:p></w:ftr>"
        .into()
}

fn chap_page_docx(body: &str) -> Vec<u8> {
    // sd_2517 chapter sections: pgNumType chapStyle=1 start=1. Word PAGE is
    // "{Heading1 number}-{section page}" (1-1, 1-2, 2-1), not bare 1/2.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:styleId=\"Heading1\">\
             <w:name w:val=\"heading 1\"/>\
             <w:pPr><w:outlineLvl w:val=\"0\"/>\
               <w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr>\
             </w:pPr>\
           </w:style>\
         </w:styles>";
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:abstractNum w:abstractNumId=\"0\">\
             <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
               <w:lvlText w:val=\"%1\"/><w:pStyle w:val=\"Heading1\"/></w:lvl>\
           </w:abstractNum>\
           <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
         </w:numbering>";
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
           xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\">\
         <w:body>{body}</w:body></w:document>"
    );
    let types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
        <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
        <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
        <Override PartName=\"/word/document.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/>\
        <Override PartName=\"/word/styles.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/>\
        <Override PartName=\"/word/numbering.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/>\
        <Override PartName=\"/word/footer1.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml\"/>\
        </Types>";
    let pkg_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
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
         <Relationship Id=\"rIdN\" \
           Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering\" \
           Target=\"numbering.xml\"/>\
         <Relationship Id=\"rIdF1\" \
           Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer\" \
           Target=\"footer1.xml\"/>\
         </Relationships>";
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
    zip.start_file("word/styles.xml", opts).unwrap();
    zip.write_all(styles.as_bytes()).unwrap();
    zip.start_file("word/numbering.xml", opts).unwrap();
    zip.write_all(numbering.as_bytes()).unwrap();
    zip.start_file("word/footer1.xml", opts).unwrap();
    zip.write_all(page_footer_xml().as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn page_field_uses_chapstyle_heading_and_section_page() {
    // sd_2517 body sections: w:pgNumType w:start="1" w:chapStyle="1".
    // Word PAGE is 1-1 / 1-2 in chapter 1 and 2-1 after the next Heading 1.
    let sect = "<w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:footer=\"720\"/>\
           <w:pgNumType w:start=\"1\" w:chapStyle=\"1\"/>";
    let body = format!(
        "<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>\
           <w:r><w:t>Alpha</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:r><w:t>MoreAlpha</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr><w:type w:val=\"nextPage\"/>{sect}</w:sectPr></w:pPr></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>\
           <w:r><w:t>Beta</w:t></w:r></w:p>\
         <w:sectPr>{sect}</w:sectPr>"
    );
    let pdf = docx_to_pdf(&chap_page_docx(&body)).expect("convert chapStyle PAGE");
    assert_eq!(pdf_page_count(&pdf), 3, "two chapter-1 pages + chapter 2");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("(1-1)"),
        "first Heading1 page must be 1-1; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        text.contains("(1-2)"),
        "second page of the same chapter must be 1-2; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        text.contains("(2-1)"),
        "next Heading1 section must be 2-1; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
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
fn light_shading_does_not_invent_inside_horizontal_rules() {
    // comments-lots / I_am_sharing LightShading-Accent1 lists only
    // tblBorders top+bottom (sz=8, 4F81BD). stroke_cell treated that as
    // implied insideH so every row got a 0.5pt rule. Word Quartz paints
    // band fills and only the listed outer edges (page 1 is 23 fills, 0
    // strokes).
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"table\" w:styleId=\"LightShading-Accent1\">\
            <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
            <w:tblPr><w:tblBorders>\
              <w:top w:val=\"single\" w:sz=\"8\" w:color=\"4F81BD\"/>\
              <w:bottom w:val=\"single\" w:sz=\"8\" w:color=\"4F81BD\"/>\
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
         <w:tr><w:tc><w:p><w:r><w:t>R0</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>R1</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>R2</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert light shading edges");
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(
        ys.len() <= 2,
        "LightShading top+bottom must not invent insideH rules; ys={ys:?}"
    );
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.310 0.506 0.741 rg"),
        "outer edges must use style 4F81BD fills, not black; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn light_shading_skips_header_when_firstrow_has_no_fill() {
    // Word LightShading-Accent1: firstRow is bold-only. Band1 still
    // starts on the first *body* row (R1), not R0. Banding R0 inverted
    // comments-lots Date / Document purpose / Status vs the Word oracle.
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
    let hay = String::from_utf8_lossy(&pdf);
    let cells: Vec<_> = pdf_fill_boxes_in(&hay, 0.827, 0.875, 0.933)
        .into_iter()
        .filter(|&(_, _, w, h)| w > 100.0 && h >= 14.0)
        .collect();
    assert_eq!(
        cells.len(),
        1,
        "Word: only body R1 is band1 (R0 header + R2 band2 empty); cells={cells:?}"
    );
}

fn light_shading_accent1_styles() -> &'static str {
    // I_am_sharing LightShading-Accent1: table rPr color 365F91, firstRow
    // bold-only. Run w:b val=0 must clear that bold; unstyled cell text
    // keeps 365F91 (Word "Executive / Sales").
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"table\" w:styleId=\"LightShading-Accent1\">\
            <w:rPr><w:color w:val=\"365F91\" w:themeColor=\"accent1\" w:themeShade=\"BF\"/></w:rPr>\
            <w:tblStylePr w:type=\"firstRow\"><w:rPr><w:b/></w:rPr></w:tblStylePr>\
            <w:tblStylePr w:type=\"firstCol\"><w:rPr><w:b/></w:rPr></w:tblStylePr>\
          </w:style>\
        </w:styles>"
}

#[test]
fn table_style_rpr_color_stays_black_after_mini_112() {
    // Word LightShading-Accent1 paints unstyled cells 365F91. Applying
    // that (mini 112) dropped comments-lots −0.34, I_am_sharing −0.28,
    // file_170 −0.76. Keep default black.
    let body = "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"LightShading-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:firstColumn=\"1\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"4000\"/><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:p><w:r><w:rPr><w:b/><w:color w:val=\"1F4E79\"/></w:rPr>\
             <w:t>PrepFor</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:rPr><w:b w:val=\"0\"/></w:rPr>\
             <w:t>ExecNoBold</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, light_shading_accent1_styles()))
        .expect("convert LightShading rPr");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("0.212 0.373 0.569 rg"),
        "mini 112 table-style 365F91 was ITT-wrong; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn table_style_firstrow_bold_stays_on_after_mini_112() {
    let body = "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"LightShading-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:firstColumn=\"1\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"4000\"/><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:p><w:r><w:rPr><w:b/><w:color w:val=\"1F4E79\"/></w:rPr>\
             <w:t>PrepFor</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:rPr><w:b w:val=\"0\"/></w:rPr>\
             <w:t>ExecNoBold</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, light_shading_accent1_styles()))
        .expect("convert firstRow bold override");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Calibri-Bold 46 Tf 0.000 0.000 0.000 rg"),
        "mini 112 firstRow-vs-val=0 was ITT-wrong; keep bold black; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn official_i_am_sharing_executive_stays_black_after_mini_112() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/I_am_sharing_Microsoft_Word_vs_Google_Docs_Comprehensive_Proof_with_you.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official I_am_sharing"))
        .expect("convert official I_am_sharing");
    assert_eq!(pdf_page_count(&pdf), 9, "Word I_am_sharing is 9pp");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("0.212 0.373 0.569 rg"),
        "mini 112 365F91 was ITT-wrong on comments-lots; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

fn medium_shading_accent1_styles() -> &'static str {
    // comments-lots MediumShading1-Accent1: firstRow 4F81BD, band1Horz D3DFEE.
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"table\" w:styleId=\"MediumShading1-Accent1\">\
            <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
            <w:tblPr><w:tblStyleRowBandSize w:val=\"1\"/></w:tblPr>\
            <w:tblStylePr w:type=\"firstRow\">\
              <w:rPr><w:b/></w:rPr>\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"4F81BD\"/></w:tcPr>\
            </w:tblStylePr>\
            <w:tblStylePr w:type=\"band1Horz\">\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"D3DFEE\"/></w:tcPr>\
            </w:tblStylePr>\
          </w:style>\
        </w:styles>"
}

#[test]
fn medium_shading_paints_header_and_band_cell_fills() {
    // comments-lots MediumShading / MediumList: Word Quartz fills the
    // firstRow (4F81BD) and band1Horz body cells (D3DFEE). Direct cell
    // shd 1F4E79 on the header still wins over the style fill.
    let body = "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"MediumShading1-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:noHBand=\"0\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"3000\"/><w:gridCol w:w=\"3000\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"1F4E79\"/></w:tcPr>\
             <w:p><w:r><w:t>H0</w:t></w:r></w:p></w:tc>\
           <w:tc><w:tcPr><w:shd w:val=\"clear\" w:fill=\"1F4E79\"/></w:tcPr>\
             <w:p><w:r><w:t>H1</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>A0</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>A1</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>B0</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>B1</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>C0</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>C1</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, medium_shading_accent1_styles()))
        .expect("convert MediumShading fills");
    let text = String::from_utf8_lossy(&pdf);
    let navy = text.matches("0.122 0.306 0.475 rg").count();
    let band = text.matches("0.827 0.875 0.933 rg").count();
    assert!(
        navy >= 2,
        "header direct 1F4E79 must paint; navy={navy} tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        band >= 2,
        "band1Horz body rows must paint D3DFEE; band={band} tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn medium_shading_cell_fill_covers_each_text_line() {
    // Word Quartz paints cell shd as the cell rect plus one inset fill per
    // text line (tblCellMar 108 twips). comments-lots p3 is 35 D3DFEE
    // fills vs our 9 cell-only rects.
    let body = "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"MediumShading1-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:noHBand=\"0\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>Hdr</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc>\
           <w:p><w:r><w:t>LineOne</w:t></w:r></w:p>\
           <w:p><w:r><w:t>LineTwo</w:t></w:r></w:p>\
         </w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, medium_shading_accent1_styles()))
        .expect("convert MediumShading line fills");
    let text = String::from_utf8_lossy(&pdf);
    let band = text.matches("0.827 0.875 0.933 rg").count();
    assert!(
        band >= 3,
        "banded 2-line cell must paint cell + per-line D3DFEE fills like Word; band={band} tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn medium_list_firstcol_fill_wins_over_row_band() {
    // comments-lots MediumList2-Accent1: firstCol shd FFFFFF sits on top
    // of band1Horz D3DFEE. We painted the whole banded row, including col 0.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"table\" w:styleId=\"MediumList2-Accent1\">\
            <w:tblStylePr w:type=\"firstRow\">\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"4F81BD\"/></w:tcPr>\
            </w:tblStylePr>\
            <w:tblStylePr w:type=\"firstCol\">\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"FFFFFF\"/></w:tcPr>\
            </w:tblStylePr>\
            <w:tblStylePr w:type=\"band1Horz\">\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"D3DFEE\"/></w:tcPr>\
            </w:tblStylePr>\
          </w:style>\
        </w:styles>";
    let body = "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"MediumList2-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:firstColumn=\"1\" w:noHBand=\"0\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"3000\"/><w:gridCol w:w=\"3000\"/></w:tblGrid>\
         <w:tr><w:tc><w:p><w:r><w:t>H0</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>H1</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:p><w:r><w:t>A0</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>A1</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert MediumList firstCol");
    let text = String::from_utf8_lossy(&pdf);
    let band = text.matches("0.827 0.875 0.933 rg").count();
    let white = text.matches("1.000 1.000 1.000 rg").count();
    assert!(
        white >= 1,
        "firstCol FFFFFF must paint over the row band; white={white} band={band}"
    );
    assert!(
        (1..=2).contains(&band),
        "only the non-first banded cell is D3DFEE (cell+line), not the whole row; band={band}"
    );
}

fn grid_table4_accent1_styles() -> &'static str {
    // potpourri / file_170 GridTable4-Accent1: firstRow fill 156082 +
    // rPr color FFFFFF. Header cells have no direct w:color; Word paints
    // Region/Q1 white on the dark fill. We currently leave them black.
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"table\" w:styleId=\"GridTable4-Accent1\">\
            <w:tblPr><w:tblBorders>\
              <w:top w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:left w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:right w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
            </w:tblBorders></w:tblPr>\
            <w:tblStylePr w:type=\"firstRow\">\
              <w:rPr><w:b/><w:color w:val=\"FFFFFF\" w:themeColor=\"background1\"/></w:rPr>\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"156082\"/></w:tcPr>\
            </w:tblStylePr>\
            <w:tblStylePr w:type=\"firstCol\"><w:rPr><w:b/></w:rPr></w:tblStylePr>\
            <w:tblStylePr w:type=\"band1Horz\">\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"C1E4F5\"/></w:tcPr>\
            </w:tblStylePr>\
          </w:style>\
        </w:styles>"
}

#[test]
fn grid_table4_firstrow_header_text_is_white() {
    // Word Quartz: GridTable4-Accent1 firstRow rPr color FFFFFF on the
    // header run even when the cell has no direct w:color (potpourri
    // Region/Q1). Table-level rPr color is still ITT-wrong (mini 112).
    // Glyphs emit one Tj each, so assert the bold header paint color.
    let body = "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"GridTable4-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:firstColumn=\"1\" w:noHBand=\"0\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2337\"/><w:gridCol w:w=\"2337\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>Region</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>Q1</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>North</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>$120k</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, grid_table4_accent1_styles()))
        .expect("convert GridTable4 firstRow color");
    let hay = String::from_utf8_lossy(&pdf);
    let text = pdf_winansi_text(&pdf);
    assert!(
        text.contains("Region") && text.contains("North"),
        "table text missing; got {text}"
    );
    assert!(
        hay.contains("/Calibri-Bold 46 Tf 1.000 1.000 1.000 rg"),
        "Word paints GridTable4 firstRow FFFFFF on 11pt header; tail {}",
        &hay[hay.len().saturating_sub(280)..]
    );
    assert!(
        hay.contains("/Calibri-Bold 46 Tf 0.000 0.000 0.000 rg"),
        "firstCol body (North) stays bold black; tail {}",
        &hay[hay.len().saturating_sub(280)..]
    );
    assert!(
        hay.contains("0.082 0.376 0.510 rg"),
        "firstRow fill 156082 must still paint; tail {}",
        &hay[hay.len().saturating_sub(240)..]
    );
}

#[test]
fn official_potpourri_gridtable_header_region_is_white() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let hay = String::from_utf8_lossy(&pdf);
    assert!(
        hay.contains("/Aptos-Bold 12.00 Tf 1.000 1.000 1.000 rg"),
        "Word GridTable4 header Region/Q1 is Aptos-Bold 12 white; tail {}",
        &hay[hay.len().saturating_sub(280)..]
    );
    assert!(
        hay.contains("/Aptos-Bold 12.00 Tf 0.000 0.000 0.000 rg"),
        "firstCol body North stays bold black"
    );
}

fn grid_table4_accent1_styles_with_firstrow_borders() -> String {
    // Word potpourri GridTable4-Accent1 firstRow tcBorders are 156082
    // (0.48pt), body tblBorders 45B0E1. We currently stroke the header
    // lattice with the body color only.
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"table\" w:styleId=\"GridTable4-Accent1\">\
            <w:tblPr><w:tblBorders>\
              <w:top w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:left w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:right w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
              <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"45B0E1\"/>\
            </w:tblBorders></w:tblPr>\
            <w:tblStylePr w:type=\"firstRow\">\
              <w:rPr><w:b/><w:color w:val=\"FFFFFF\"/></w:rPr>\
              <w:tcPr>\
                <w:tcBorders>\
                  <w:top w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"156082\"/>\
                  <w:left w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"156082\"/>\
                  <w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"156082\"/>\
                  <w:right w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"156082\"/>\
                  <w:insideH w:val=\"nil\"/>\
                  <w:insideV w:val=\"nil\"/>\
                </w:tcBorders>\
                <w:shd w:val=\"clear\" w:fill=\"156082\"/>\
              </w:tcPr>\
            </w:tblStylePr>\
            <w:tblStylePr w:type=\"firstCol\"><w:rPr><w:b/></w:rPr></w:tblStylePr>\
            <w:tblStylePr w:type=\"band1Horz\">\
              <w:tcPr><w:shd w:val=\"clear\" w:fill=\"C1E4F5\"/></w:tcPr>\
            </w:tblStylePr>\
          </w:style>\
        </w:styles>"
        .into()
}

fn grid_table4_two_row_body() -> &'static str {
    "<w:tbl>\
         <w:tblPr><w:tblStyle w:val=\"GridTable4-Accent1\"/>\
           <w:tblLook w:firstRow=\"1\" w:lastRow=\"0\" w:firstColumn=\"1\"\
             w:lastColumn=\"0\" w:noHBand=\"0\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2337\"/><w:gridCol w:w=\"2337\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>Region</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>Q1</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>North</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>$120k</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl><w:sectPr/>"
}

fn accent_dark_hairlines(pdf: &[u8]) -> Vec<(f32, f32, f32, f32)> {
    pdf_fill_boxes_in(&String::from_utf8_lossy(pdf), 0.082, 0.376, 0.510)
        .into_iter()
        .filter(|(_, _, w, h)| *h < 2.0 || *w < 2.0)
        .collect()
}

#[test]
fn grid_table4_firstrow_borders_are_accent_dark() {
    let pdf = docx_to_pdf(&docx_with_styles(
        grid_table4_two_row_body(),
        &grid_table4_accent1_styles_with_firstrow_borders(),
    ))
    .expect("convert GridTable4 firstRow tcBorders");
    let hair = accent_dark_hairlines(&pdf);
    assert!(
        !hair.is_empty(),
        "Word firstRow tcBorders 156082 are 0.5pt hairlines, not body 45B0E1; hair={hair:?}"
    );
}

#[test]
fn official_potpourri_gridtable_firstrow_borders_are_dark() {
    // Word header lattice is 156082 0.48pt (33 fills, 25 of them
    // hairlines). Ours strokes header with body 45B0E1 only. Paint-only
    // — potpourri stays 5pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let hair = accent_dark_hairlines(&pdf);
    assert!(
        hair.len() >= 8,
        "potpourri GridTable4 firstRow must stroke 156082 hairlines; n={} hair={hair:?}",
        hair.len()
    );
}

fn accent_dark_vertical_keys(hair: &[(f32, f32, f32, f32)]) -> Vec<(i32, i32)> {
    let mut keys: Vec<(i32, i32)> = hair
        .iter()
        .filter(|(_, _, w, h)| *w < 2.0 && *h > 4.0)
        .map(|(x, y, _, _)| ((x * 2.0).round() as i32, (y * 2.0).round() as i32))
        .collect();
    keys.sort_unstable();
    keys
}

#[test]
fn grid_table4_firstrow_shared_vertical_stays_stacked_after_mini_sharededge() {
    // Word potpourri header is 25 hairs vs our 48 (left+right at shared x).
    // Skipping firstRow left on col>0 (mini 265) dropped no-redline
    // 59.1966→59.1884 (comments-lots −0.16). Keep stacked.
    let pdf = docx_to_pdf(&docx_with_styles(
        grid_table4_two_row_body(),
        &grid_table4_accent1_styles_with_firstrow_borders(),
    ))
    .expect("convert GridTable4 stacked vertical");
    let keys = accent_dark_vertical_keys(&accent_dark_hairlines(&pdf));
    assert!(
        keys.windows(2).any(|pair| pair[0] == pair[1]),
        "mini shared-edge skip was ITT-neg; keep firstRow left+right stacked; keys={keys:?}"
    );
}

#[test]
fn official_potpourri_gridtable_shared_vertical_stays_stacked_after_mini_sharededge() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let keys = accent_dark_vertical_keys(&accent_dark_hairlines(&pdf));
    assert!(
        keys.windows(2).any(|pair| pair[0] == pair[1]),
        "mini shared-edge skip was ITT-neg (comments-lots −0.16); keep stacked; keys={keys:?}"
    );
}

fn black_vertical_keys(pdf: &[u8]) -> Vec<(i32, i32)> {
    let mut keys: Vec<(i32, i32)> =
        pdf_fill_boxes_in(&String::from_utf8_lossy(pdf), 0.000, 0.000, 0.000)
            .into_iter()
            .filter(|(_, _, w, h)| *w < 2.0 && *h > 4.0)
            .map(|(x, y, _, _)| ((x * 2.0).round() as i32, (y * 2.0).round() as i32))
            .collect();
    keys.sort_unstable();
    keys
}

#[test]
fn tblborders_shared_vertical_stays_stacked_after_mini_inside() {
    // Word paints each insideV once (LIGHT 64 vs 44). Skipping interior
    // left/top (mini 271–272) lifted no-redline +0.015/+0.006 but dropped
    // redline mean −0.005 (file_146 −0.73). Keep stacked.
    let body = "<w:tbl><w:tblPr><w:tblBorders>\
           <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
         </w:tblBorders></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2337\"/><w:gridCol w:w=\"2337\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         <w:tr>\
           <w:tc><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc>\
           <w:tc><w:p><w:r><w:t>D</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert tblBorders lattice");
    let keys = black_vertical_keys(&pdf);
    assert!(
        keys.windows(2).any(|pair| pair[0] == pair[1]),
        "mini insideV skip was redline ITT-neg; keep left+right stacked; keys={keys:?}"
    );
}

fn c1e4f5_cell_fills(pdf: &[u8]) -> Vec<(f32, f32, f32, f32)> {
    pdf_fill_boxes_in(&String::from_utf8_lossy(pdf), 0.757, 0.894, 0.961)
        .into_iter()
        .filter(|(_, _, w, h)| *h > 4.0 && *w > 20.0)
        .collect()
}

fn c1e4f5_height_span(fills: &[(f32, f32, f32, f32)]) -> (f32, f32) {
    let mut min_h = f32::MAX;
    let mut max_h = 0.0_f32;
    for (_, _, _, h) in fills {
        min_h = min_h.min(*h);
        max_h = max_h.max(*h);
    }
    (min_h, max_h)
}

#[test]
fn grid_table4_band_inner_fill_matches_cell_height() {
    // Word potpourri band1Horz C1E4F5 is cell-height × tblCellMar-inset
    // (14.64/14.64), not a shorter per-line inner (ours 16/11).
    let pdf = docx_to_pdf(&docx_with_styles(
        grid_table4_two_row_body(),
        grid_table4_accent1_styles(),
    ))
    .expect("convert GridTable4 band fill");
    let fills = c1e4f5_cell_fills(&pdf);
    assert!(
        fills.len() >= 2,
        "band cell must paint outer+inner C1E4F5; fills={fills:?}"
    );
    let (inner, outer) = c1e4f5_height_span(&fills);
    assert!(
        (outer - inner).abs() < 1.0,
        "Word band inner is cell height, not line_box; inner={inner} outer={outer} fills={fills:?}"
    );
}

#[test]
fn official_potpourri_gridtable_band_inner_is_cell_height() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let fills = c1e4f5_cell_fills(&pdf);
    assert!(
        fills.len() >= 2,
        "potpourri band1Horz must paint inset C1E4F5; fills={fills:?}"
    );
    let (inner, outer) = c1e4f5_height_span(&fills);
    assert!(
        (outer - inner).abs() < 1.5,
        "Word potpourri band inner h=14.64 matches outer; inner={inner} outer={outer} fills={fills:?}"
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
        text.contains("1.000 0.000 0.000 rg"),
        "header pBdr bottom must fill red like Word Quartz; tail {}",
        &text[text.len().saturating_sub(400)..]
    );
    assert!(
        !text.contains("1.000 0.000 0.000 RG"),
        "header pBdr must not stroke; tail {}",
        &text[text.len().saturating_sub(240)..]
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
        text.contains("1.000 0.000 0.000 rg"),
        "body pBdr bottom must fill red like Word Quartz; tail {}",
        &text[text.len().saturating_sub(400)..]
    );
}

#[test]
fn pbdr_sz_three_is_word_quartz_hairline() {
    // file_146 / sample_iter2 heading pBdr sz=3. Word Quartz is a 0.24pt
    // fill (1px @ 300dpi). (sz/8).max(0.4) painted 0.40pt on 45 rules.
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:bottom w:val=\"single\" w:sz=\"3\" w:space=\"1\" w:color=\"FF0000\"/>\
         </w:pBdr></w:pPr>\
         <w:r><w:t>HairRule</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert sz=3 pBdr");
    let bars = pdf_fill_rects(&pdf, 1.0, 0.0, 0.0);
    let h = bars.iter().map(|(_, h)| *h).fold(0.0_f32, f32::max);
    assert!(
        (0.20..0.30).contains(&h),
        "Word sz=3 pBdr is 0.24pt hairline, not 0.40; h={h} bars={bars:?}"
    );
}

#[test]
fn official_file_146_e2e8f0_rules_are_word_hairline() {
    // Word file_146 E2E8F0 bottoms are 0.24pt (70.56–541.44 × 0.24).
    // Ours were 72×468 × 0.40. Keep 7pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let bars = pdf_fill_rects(&pdf, 0.886, 0.910, 0.941);
    let hs: Vec<f32> = bars
        .iter()
        .filter(|(w, _)| *w > 200.0)
        .map(|(_, h)| *h)
        .collect();
    assert!(
        !hs.is_empty(),
        "E2E8F0 heading rules must paint; bars={bars:?}"
    );
    let max_h = hs.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        (0.20..0.32).contains(&max_h),
        "Word E2E8F0 is 0.24pt, not 0.40; max_h={max_h} hs={hs:?}"
    );
}

#[test]
fn pbdr_bottom_stays_content_box_after_mini_outset() {
    // mini 225–228: 1.44pt / 6px@300dpi Quartz outset matched Word
    // file_146 70.56–541.44 but was no-redline mean −0.0001 (file_134
    // −0.003). Keep the content box (72×468).
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:bottom w:val=\"single\" w:sz=\"3\" w:space=\"4\" w:color=\"FF0000\"/>\
         </w:pBdr></w:pPr>\
         <w:r><w:t>OutsetRule</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert content-box pBdr");
    let hay = String::from_utf8_lossy(&pdf);
    let boxes = pdf_fill_boxes_in(&hay, 1.0, 0.0, 0.0);
    let hair: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| *h > 0.0 && *h < 1.6 && *w > 40.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "pBdr must still fill a hairline; boxes={boxes:?}"
    );
    let (x, _, w, _) = hair[0];
    assert!(
        (71.5..72.5).contains(&x),
        "mini outset was ITT-neg; keep pBdr at margin 72; x={x} hair={hair:?}"
    );
    assert!(
        (466.0..469.0).contains(&w),
        "mini outset was ITT-neg; keep pBdr w=468; w={w} hair={hair:?}"
    );
}

#[test]
fn hf_pbdr_stays_content_box_after_mini_hfoutset() {
    // Word file_146 header E2E8F0 is 70.56–541.44, but chrome Quartz
    // 1.44pt outset (mini 244) dropped no-redline mean −0.0001 /
    // median −0.0002. Keep the content box like body pBdr.
    let header = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:pPr><w:pBdr>\
             <w:bottom w:val=\"single\" w:sz=\"2\" w:space=\"1\" w:color=\"E2E8F0\"/>\
           </w:pBdr><w:jc w:val=\"center\"/></w:pPr>\
           <w:r><w:t>Hdr</w:t></w:r></w:p></w:hdr>";
    let body = "<w:p><w:r><w:t>Body</w:t></w:r></w:p>\
         <w:sectPr><w:headerReference w:type=\"default\" r:id=\"rIdH1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdH1", "header", "header1.xml")],
        &[("word/header1.xml", header.to_string())],
    ))
    .expect("convert header pBdr");
    let hay = String::from_utf8_lossy(&pdf);
    let boxes = pdf_fill_boxes_in(&hay, 0.886, 0.910, 0.941);
    let hair: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, y, w, h)| *y > 700.0 && *w > 40.0 && *h < 1.6)
        .collect();
    assert!(!hair.is_empty(), "header pBdr must fill; boxes={boxes:?}");
    let (x, _, w, _) = hair[0];
    assert!(
        (71.5..72.5).contains(&x),
        "mini hfoutset was ITT-neg; keep header pBdr at margin 72; x={x} hair={hair:?}"
    );
    assert!(
        (466.0..469.0).contains(&w),
        "mini hfoutset was ITT-neg; keep header pBdr w=468; w={w} hair={hair:?}"
    );
}

#[test]
fn official_file_146_e2e8f0_stays_content_box_after_mini_outset() {
    // mini 225–228 Quartz 1.44pt outset: no-redline 59.1522/53.4543 vs
    // KEEP 59.1523/53.4544. Keep 72×468; 7pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let hay = String::from_utf8_lossy(&pdf);
    let boxes = pdf_fill_boxes_in(&hay, 0.886, 0.910, 0.941);
    let wide: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, y, w, h)| *w > 200.0 && *h < 0.4 && *y > 80.0 && *y < 720.0)
        .collect();
    assert!(
        !wide.is_empty(),
        "E2E8F0 heading rules must paint; boxes={boxes:?}"
    );
    let min_x = wide.iter().map(|(x, _, _, _)| *x).fold(f32::MAX, f32::min);
    let max_w = wide.iter().map(|(_, _, w, _)| *w).fold(0.0_f32, f32::max);
    assert!(
        (71.5..72.5).contains(&min_x),
        "mini outset was ITT-neg; keep E2E8F0 x=72; min_x={min_x} wide={wide:?}"
    );
    assert!(
        (466.0..469.0).contains(&max_w),
        "mini outset was ITT-neg; keep E2E8F0 w=468; max_w={max_w}"
    );
}

#[test]
fn empty_para_pbdr_stays_painted_after_mini_pbdrskip() {
    // file_146: 17 empty body pBdr paras. Skipping them (mini 217–220)
    // dropped redline mean −0.020. Keep the rule.
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:bottom w:val=\"single\" w:sz=\"3\" w:space=\"4\" w:color=\"FF0000\"/>\
         </w:pBdr><w:spacing w:before=\"300\" w:after=\"80\"/></w:pPr></w:p>\
         <w:p><w:r><w:t>AfterEmpty</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert empty pBdr");
    let bars = pdf_fill_rects(&pdf, 1.0, 0.0, 0.0);
    assert!(
        !bars.is_empty(),
        "mini pbdrskip was redline ITT-neg; keep empty pBdr; bars={bars:?}"
    );
}

#[test]
fn deleted_only_para_pbdr_stays_painted_after_mini_pbdrskip() {
    // Skipping del-only pBdr dropped comments-lots redlines −0.48.
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:bottom w:val=\"single\" w:sz=\"3\" w:space=\"4\" w:color=\"FF0000\"/>\
         </w:pBdr></w:pPr>\
         <w:del w:id=\"0\" w:author=\"A\"><w:r><w:delText>GoneHeading</w:delText></w:r></w:del></w:p>\
         <w:p><w:r><w:t>AfterDel</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert del pBdr");
    let bars = pdf_fill_rects(&pdf, 1.0, 0.0, 0.0);
    assert!(
        !bars.is_empty(),
        "mini pbdrskip was redline ITT-neg; keep del pBdr; bars={bars:?}"
    );
}

#[test]
fn official_file_146_e2e8f0_rule_count_stays_after_mini_pbdrskip() {
    // Word file_146 E2E8F0 sc count is 17. Skipping empty/del pBdr
    // (mini 217–220) was redline ITT-neg. Keep ~40 rules; 7pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let n = pdf_fill_rects(&pdf, 0.886, 0.910, 0.941)
        .iter()
        .filter(|(w, _)| *w > 200.0)
        .count();
    assert!(
        (32..=48).contains(&n),
        "mini pbdrskip was redline ITT-neg; keep ~40 E2E8F0 rules; n={n}"
    );
}

#[test]
fn body_pbdr_top_paints_a_rule() {
    // Strict01 Online Video callout is a 4-edge pBdr box. We only
    // stroked bottom, so the top rule (and the box) never appeared.
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:top w:val=\"single\" w:sz=\"12\" w:space=\"1\" w:color=\"00FF00\"/>\
         </w:pBdr></w:pPr>\
         <w:r><w:t>TopRule</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert pBdr top");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.000 1.000 0.000 rg"),
        "body pBdr top must fill green like Word Quartz; tail {}",
        &text[text.len().saturating_sub(400)..]
    );
}

#[test]
fn body_pbdr_left_paints_a_vertical_rule() {
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:left w:val=\"single\" w:sz=\"12\" w:space=\"4\" w:color=\"0000FF\"/>\
         </w:pBdr></w:pPr>\
         <w:r><w:t>LeftRule</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert pBdr left");
    let verts: Vec<_> = pdf_fill_rects(&pdf, 0.0, 0.0, 1.0)
        .into_iter()
        .filter(|(w, h)| *w > 0.0 && *w < 1.6 && *h > 8.0)
        .collect();
    assert!(
        !verts.is_empty(),
        "body pBdr left must fill a vertical hairline; verts={verts:?}"
    );
}

#[test]
fn pbdr_left_space_sits_outside_the_text_box() {
    // sd_2517 / file_22 TextHeading2 4-edge pBdr: left/right space=4.
    // Word box is 93.36–518.88 vs indent-only 99–513 (5.6pt / 11px past
    // max_shift). space is the gap between border and text; left sits
    // at indent − space. Horizontal pBdr (file_146) is unchanged.
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:left w:val=\"single\" w:sz=\"4\" w:space=\"4\" w:color=\"0000FF\"/>\
         </w:pBdr></w:pPr>\
         <w:r><w:t>SpaceLeft</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert pBdr left space");
    let hay = String::from_utf8_lossy(&pdf);
    let boxes = pdf_fill_boxes_in(&hay, 0.0, 0.0, 1.0);
    let verts: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| *w > 0.0 && *w < 1.6 && *h > 8.0)
        .collect();
    assert!(
        !verts.is_empty(),
        "left pBdr must still fill a vertical; boxes={boxes:?}"
    );
    let x = verts[0].0;
    assert!(
        (66.5..69.5).contains(&x),
        "Word left space=4 is margin 72 − 4pt, not the text x; x={x} verts={verts:?}"
    );
}

#[test]
fn pbdr_four_edge_space_matches_word_textheading2_box() {
    // sd_2517 body section: pgMar left/right 1800 (90pt), ind 180 (9pt),
    // left/right space=4. Word x=93.36. indent-only x=99 is 5.6pt off.
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:top w:val=\"single\" w:sz=\"4\" w:space=\"1\" w:color=\"0000FF\"/>\
           <w:left w:val=\"single\" w:sz=\"4\" w:space=\"4\" w:color=\"0000FF\"/>\
           <w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"1\" w:color=\"0000FF\"/>\
           <w:right w:val=\"single\" w:sz=\"4\" w:space=\"4\" w:color=\"0000FF\"/>\
         </w:pBdr><w:ind w:left=\"180\" w:right=\"180\"/></w:pPr>\
         <w:r><w:t>adipiscing labore do lorem ipsum boxed</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1800\" w:bottom=\"1440\" w:left=\"1800\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 4-edge space");
    let hay = String::from_utf8_lossy(&pdf);
    let boxes = pdf_fill_boxes_in(&hay, 0.0, 0.0, 1.0);
    let verts: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| *w > 0.0 && *w < 1.6 && *h > 8.0)
        .collect();
    assert!(
        verts.len() >= 2,
        "4-edge pBdr must paint left and right; boxes={boxes:?}"
    );
    let min_x = verts.iter().map(|(x, _, _, _)| *x).fold(f32::MAX, f32::min);
    assert!(
        (93.0..96.5).contains(&min_x),
        "Word TextHeading2 box left is ~93.36 (90+9-4), not indent 99; min_x={min_x} verts={verts:?}"
    );
}

#[test]
fn body_pbdr_bottom_is_filled_hairline_like_word_quartz() {
    // file_146 / eigenpal / sample_document: Word Quartz paints pBdr as
    // filled hairline rects (p1 is 4× E2E8F0 0.24pt `f`, 0 strokes). We
    // emitted `w RG … l S`, so color_sim/edge_iou saw a stroked rule
    // against Word's ink-union fill — same miss TableGrid had before
    // FillRect.
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:bottom w:val=\"single\" w:sz=\"3\" w:space=\"4\" w:color=\"E2E8F0\"/>\
         </w:pBdr></w:pPr>\
         <w:r><w:t>Rule</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert filled pBdr");
    let text = String::from_utf8_lossy(&pdf);
    let fills = pdf_fill_rects(&pdf, 0.886, 0.910, 0.941);
    let hair: Vec<_> = fills
        .iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 40.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "pBdr must paint E2E8F0 filled hairlines; fills={fills:?} tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("0.886 0.910 0.941 RG"),
        "pBdr must not stroke the Quartz hairline; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn body_pbdr_follows_paragraph_indent() {
    // comments-lots IntenseQuote: left/right=936 twips (46.8pt). Word
    // paints the accent1 bottom rule at the indent (x≈100, w≈413), not
    // the page margins (x=54, w=504). Full-width extra ink is an unused
    // <50-cluster class (comments-lots family / I_am_sharing / file_9).
    let body = "<w:p><w:pPr><w:pBdr>\
           <w:bottom w:val=\"single\" w:sz=\"12\" w:space=\"4\" w:color=\"FF0000\"/>\
         </w:pBdr><w:ind w:left=\"936\" w:right=\"936\"/></w:pPr>\
         <w:r><w:t>IndentedQuote</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert indented pBdr");
    let hay = String::from_utf8_lossy(&pdf);
    let boxes = pdf_fill_boxes_in(&hay, 1.0, 0.0, 0.0);
    let hair: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| *h > 0.0 && *h < 1.6 && *w > 40.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "indented pBdr must still fill a hairline; boxes={boxes:?}"
    );
    let (x, _, w, _) = hair[0];
    assert!(
        (x - 118.8).abs() < 2.0,
        "pBdr x must be margin 72 + indent 46.8, not page margin 72; x={x} hair={hair:?}"
    );
    assert!(
        (w - 374.4).abs() < 4.0,
        "pBdr w must shrink by left+right indent (468-93.6=374.4), not span the page; w={w} hair={hair:?}"
    );
}

#[test]
fn official_comments_lots_intensequote_rule_follows_indent() {
    // Word p2 IntenseQuote "Tip: In Word…" rule is 99.4–512.6 (ind
    // 936/936 on 54pt margins). Ours painted 54–558, ~90pt of extra
    // accent1 ink on every comments-lots family stem.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official comments-lots"))
        .expect("convert comments-lots");
    assert_eq!(pdf_page_count(&pdf), 9, "Word comments-lots is 9pp");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 2, "comments-lots p2 holds IntenseQuote");
    let boxes = pdf_fill_boxes_in(&pages[1], 0.310, 0.506, 0.741);
    let hair: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| *h > 0.0 && *h < 1.6 && *w > 200.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "p2 IntenseQuote must fill an accent1 hairline; boxes={boxes:?}"
    );
    let (x, _, w, _) = hair[0];
    assert!(
        (95.0..110.0).contains(&x),
        "Word IntenseQuote rule starts at ~100 (54+46.8), not margin 54; x={x} hair={hair:?}"
    );
    assert!(
        (400.0..430.0).contains(&w),
        "Word IntenseQuote rule is ~413pt wide, not full 504pt content; w={w} hair={hair:?}"
    );
}

#[test]
fn single_underline_is_filled_hairline_like_word_quartz() {
    // file_34 p1: Word Quartz paints single `w:u` as filled 0.48pt
    // hairlines (9 `f`, 2 `s` — the strokes are the wave). We emitted
    // ~68 `l S` rules, so edge_iou/color_sim saw strokes against fills.
    let body = "<w:p><w:r><w:rPr><w:u w:val=\"single\"/><w:color w:val=\"FF0000\"/></w:rPr>\
           <w:t>UnderlinedSample</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert filled underline");
    let text = String::from_utf8_lossy(&pdf);
    let hair: Vec<_> = pdf_fill_rects(&pdf, 1.0, 0.0, 0.0)
        .into_iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 40.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "single underline must paint a filled hairline; fills={:?} tail {}",
        pdf_fill_rects(&pdf, 1.0, 0.0, 0.0),
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("1.000 0.000 0.000 RG"),
        "single underline must not stroke; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn strike_is_filled_hairline_like_word_quartz() {
    // Word Quartz deletions / w:strike are filled hairlines, not `l S`.
    let body = "<w:p><w:r><w:rPr><w:strike/><w:color w:val=\"0000FF\"/></w:rPr>\
           <w:t>StruckSample</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert filled strike");
    let text = String::from_utf8_lossy(&pdf);
    let hair: Vec<_> = pdf_fill_rects(&pdf, 0.0, 0.0, 1.0)
        .into_iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 40.0)
        .collect();
    assert!(
        !hair.is_empty(),
        "strike must paint a filled hairline; fills={:?} tail {}",
        pdf_fill_rects(&pdf, 0.0, 0.0, 1.0),
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("0.000 0.000 1.000 RG"),
        "strike must not stroke; tail {}",
        &text[text.len().saturating_sub(240)..]
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
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("(2)"),
        "NUMPAGES must paint real count 2 as WinAnsi; cached 9 still? {}",
        text.contains("(9)")
    );
    assert!(
        !text.contains("(9)"),
        "cached NUMPAGES 9 must not remain after patch"
    );
}

#[test]
fn page_and_numpages_paint_when_field_has_no_cached_result() {
    // I_am_sharing footer is `Page {PAGE} of {NUMPAGES}` with empty
    // result slots (separate then end, no w:t). We dropped both so
    // every page painted "Page  of". Word prints 1..N of N.
    let footer = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p>\
             <w:r><w:t xml:space=\"preserve\">Page </w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText>PAGE</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
             <w:r><w:t xml:space=\"preserve\"> of </w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText>NUMPAGES</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
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
    .expect("convert empty field result");
    assert_eq!(pdf_page_count(&pdf), 2);
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("(1)") && text.contains("(2)"),
        "empty PAGE/NUMPAGES result must still paint 1 and 2; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn official_i_am_sharing_footer_paints_page_n_of_n() {
    // Official Word footer is "Page 1 of 9". The field result slots are
    // empty; we painted "Page  of" on every page (ITT ~48).
    let bytes = std::fs::read(
        "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/I_am_sharing_Microsoft_Word_vs_Google_Docs_Comprehensive_Proof_with_you.docx",
    )
    .expect("official I_am_sharing fixture");
    let pdf = docx_to_pdf(&bytes).expect("convert I_am_sharing");
    let pairs = footer_page_of_total(&pdf);
    assert!(
        pairs.iter().any(|&(n, t)| n == 1 && t == 9)
            && pairs.iter().any(|&(n, t)| n == 9 && t == 9),
        "Word Page N of 9 must paint; pairs={pairs:?}"
    );
}

#[test]
fn right_footer_numpages_aligns_to_digit_width_not_placeholder() {
    // I_am_sharing / comments-lots: w:jc=right "Page N of T". chrome
    // measures NUMPAGES as "@@N@@" (~45pt) then patches to "9" (~6pt),
    // so "Page" starts at 470 instead of Word 509 (right edge 558).
    let footer = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:pPr><w:jc w:val=\"right\"/></w:pPr>\
             <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr>\
               <w:t xml:space=\"preserve\">Page </w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText>PAGE</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
             <w:r><w:t xml:space=\"preserve\"> of </w:t></w:r>\
             <w:r><w:fldChar w:fldCharType=\"begin\"/></w:r>\
             <w:r><w:instrText>NUMPAGES</w:instrText></w:r>\
             <w:r><w:fldChar w:fldCharType=\"separate\"/></w:r>\
             <w:r><w:fldChar w:fldCharType=\"end\"/></w:r>\
           </w:p></w:ftr>";
    let body = "<w:p><w:r><w:t>Alpha</w:t></w:r></w:p>\
         <w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdF1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"936\" w:right=\"1080\" w:bottom=\"936\" w:left=\"1080\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdF1", "footer", "footer1.xml")],
        &[("word/footer1.xml", footer.to_string())],
    ))
    .expect("convert right NUMPAGES footer");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    let footer_xs: Vec<f32> = xs.iter().copied().filter(|x| *x > 400.0).collect();
    let min_x = footer_xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        min_x > 495.0,
        "right 'Page 1 of 1' must start near Word 509, not 470 from @@N@@ width; min_x={min_x} footer_xs={footer_xs:?}"
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
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("(2)"),
        "Inter NUMPAGES must paint 2 in the serif face as WinAnsi"
    );
    assert!(
        !text.contains("@@N@@"),
        "@@N@@ must not leak when the footer face is not Carlito"
    );
}

#[test]
fn watermark_sdt_is_not_painted_as_header_chrome() {
    // Strict01 header2 is a Watermarks SDT (VML + Fallback txbx "CONFIDENTIAL"
    // sz=72). collect_hf_runs walks every descendant <w:p>, so the inner
    // textbox becomes a 36pt header line and shoves the body down. Word
    // paints a rotated silver watermark and keeps body at the top margin.
    let header = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:hdr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" \
           xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" \
           xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" \
           xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" \
           xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
           <w:sdt><w:sdtPr><w:docPartObj>\
             <w:docPartGallery w:val=\"Watermarks\"/></w:docPartObj></w:sdtPr>\
           <w:sdtContent><w:p><w:r><w:drawing><wp:anchor behindDoc=\"1\">\
             <wp:positionH relativeFrom=\"margin\"><wp:align>center</wp:align></wp:positionH>\
             <wp:positionV relativeFrom=\"margin\"><wp:align>center</wp:align></wp:positionV>\
             <wp:extent cx=\"6703695\" cy=\"1675765\"/><wp:wrapNone/>\
             <wp:docPr id=\"1\" name=\"PowerPlusWaterMarkObject1\"/>\
             <a:graphic><a:graphicData \
               uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
               <wps:wsp><wps:txbx><w:txbxContent><w:p><w:r>\
                 <w:rPr><w:color w:val=\"C0C0C0\"/><w:sz w:val=\"72\"/></w:rPr>\
                 <w:t>CONFIDENTIAL</w:t></w:r></w:p></w:txbxContent></wps:txbx>\
               </wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></w:r></w:p>\
           </w:sdtContent></w:sdt></w:hdr>";
    let body = "<w:p><w:r><w:t>AfterMark</w:t></w:r></w:p>\
         <w:sectPr>\
           <w:headerReference w:type=\"default\" r:id=\"rIdH1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdH1", "header", "header1.xml")],
        &[("word/header1.xml", header.to_string())],
    ))
    .expect("convert watermark header");
    let text = String::from_utf8_lossy(&pdf);
    let body_ys = pdf_tf_ys(&pdf, "11.04 Tf");
    assert!(
        !body_ys.is_empty(),
        "body AfterMark must paint; {body_ys:?}"
    );
    let body_y = body_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    // Letter 792, top 72, Calibri 11.04 win-ascent ≈ 10.5 → baseline ≈ 709.5.
    // A 36pt header band drops that below 690.
    assert!(
        body_y > 700.0,
        "watermark must not consume the header band; body baseline {body_y} ys={body_ys:?}"
    );
    assert!(
        text.contains("CONFIDENTIAL"),
        "Word paints the watermark string; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    assert!(
        text.contains("0.753 0.753 0.753") || text.contains("0.752 0.752 0.752"),
        "Word watermark fill is silver C0C0C0; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
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
        !text.contains("21.12 Tf"),
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
        !text.contains("21.12 Tf"),
        "later-section / orphan 21pt chrome must not concatenate; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn titlepg_uses_first_header_on_page_one_then_default() {
    // header_no_rels: titlePg + type=first on page 1, type=default after.
    // Ranking default over first painted "My header" on Word's first page.
    let body = "<w:p><w:r><w:t>PageOneBody</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:r><w:t>PageTwoBody</w:t></w:r></w:p>\
         <w:sectPr>\
           <w:headerReference w:type=\"default\" r:id=\"rIdH1\"/>\
           <w:headerReference w:type=\"first\" r:id=\"rIdH2\"/>\
           <w:titlePg/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" \
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[
            ("rIdH1", "header", "header1.xml"),
            ("rIdH2", "header", "header2.xml"),
        ],
        &[
            ("word/header1.xml", hf_part("hdr", 22, "DefaultHdr")),
            ("word/header2.xml", hf_part("hdr", 22, "FirstHdr")),
        ],
    ))
    .expect("convert titlePg headers");
    assert_eq!(pdf_page_count(&pdf), 2, "body + page break is 2pp");
    let pages = pdf_content_streams(&pdf);
    let p1 = pdf_winansi_text(pages[0].as_bytes());
    let p2 = pdf_winansi_text(pages[1].as_bytes());
    assert!(
        p1.contains("FirstHdr"),
        "titlePg page 1 uses type=first; p1={p1}"
    );
    assert!(
        !p1.contains("DefaultHdr"),
        "titlePg page 1 must not use type=default; p1={p1}"
    );
    assert!(
        p2.contains("DefaultHdr"),
        "later pages use type=default; p2={p2}"
    );
    assert!(
        !p2.contains("FirstHdr"),
        "later pages must not keep type=first; p2={p2}"
    );
}

#[test]
fn official_header_no_rels_page_one_uses_first_header() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/header_no_rels.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official header_no_rels"))
        .expect("convert header_no_rels");
    assert_eq!(pdf_page_count(&pdf), 3, "Word header_no_rels is 3pp");
    let pages = pdf_content_streams(&pdf);
    let p1 = pdf_winansi_text(pages[0].as_bytes());
    assert!(
        p1.contains("first page") || p1.contains("firstpage"),
        "Word p1 header is My first page header; p1={p1}"
    );
}

fn pdf_rgb_rule_widths(pdf: &[u8], r: f32, g: f32, b: f32) -> Vec<f32> {
    let mut out: Vec<f32> = pdf_fill_rects(pdf, r, g, b)
        .into_iter()
        .filter_map(|(w, h)| (h > 0.0 && h < 1.6 && w > 4.0).then_some(w))
        .collect();
    let needle = format!("{r:.3} {g:.3} {b:.3} RG");
    let hay = String::from_utf8_lossy(pdf);
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

fn pdf_rgb_underline_x2s(pdf: &[u8], r: f32, g: f32, b: f32) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let fill_needle = format!("{r:.3} {g:.3} {b:.3} rg ");
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(&fill_needle) {
        let rest = &hay[from + rel + fill_needle.len()..];
        let end = rest.find(" re f").unwrap_or(0);
        let parts: Vec<&str> = rest[..end].split_whitespace().collect();
        if parts.len() >= 4
            && let (Ok(x), Ok(w), Ok(h)) = (
                parts[0].parse::<f32>(),
                parts[2].parse::<f32>(),
                parts[3].parse::<f32>(),
            )
            && h > 0.0
            && h < 1.6
            && w > 4.0
        {
            out.push(x + w);
        }
        from += rel + fill_needle.len();
    }
    let needle = format!("{r:.3} {g:.3} {b:.3} RG");
    from = 0;
    while let Some(rel) = hay[from..].find(&needle) {
        let window = &hay[from + rel..hay.len().min(from + rel + 420)];
        if let Some(ls) = window.find(" l S") {
            let before = &window[..ls];
            if let Some(mi) = before.rfind(" m ") {
                let parts: Vec<&str> = before[mi.saturating_sub(24)..].split_whitespace().collect();
                if let Some(i) = parts.iter().position(|t| *t == "m")
                    && i + 2 < parts.len()
                    && let Ok(x2) = parts[i + 1].parse::<f32>()
                {
                    out.push(x2);
                }
            }
        }
        from += rel + needle.len();
    }
    out
}

fn pdf_vertical_rule_xs(pdf: &[u8]) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(" re f\n") {
        let start = hay[..from + rel].rfind('\n').map_or(0, |i| i + 1);
        let line = &hay[start..from + rel];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5
            && let (Ok(x), Ok(_y), Ok(w), Ok(h)) = (
                parts[parts.len() - 4].parse::<f32>(),
                parts[parts.len() - 3].parse::<f32>(),
                parts[parts.len() - 2].parse::<f32>(),
                parts[parts.len() - 1].parse::<f32>(),
            )
            && w < 1.5
            && h > 6.0
        {
            out.push(x + w / 2.0);
        }
        from += rel + 5;
    }
    from = 0;
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

fn pdf_vertical_rule_hs(pdf: &[u8]) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(" re f\n") {
        let start = hay[..from + rel].rfind('\n').map_or(0, |i| i + 1);
        let line = &hay[start..from + rel];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5
            && let (Ok(w), Ok(h)) = (
                parts[parts.len() - 2].parse::<f32>(),
                parts[parts.len() - 1].parse::<f32>(),
            )
            && w < 1.5
            && h > 6.0
        {
            out.push(h);
        }
        from += rel + 5;
    }
    from = 0;
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
            out.push((y1 - y2).abs());
        }
        from += rel + 4;
    }
    out
}

fn pdf_horiz_rule_ys(pdf: &[u8]) -> Vec<f32> {
    // Word Quartz table rules are filled hairline rects
    // (`rg x y w 0.50 re f`). Older strokes are `m … l S`.
    let hay = String::from_utf8_lossy(pdf);
    let mut raw = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(" re f\n") {
        let start = hay[..from + rel].rfind('\n').map_or(0, |i| i + 1);
        let line = &hay[start..from + rel];
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4
            && let (Ok(y), Ok(w), Ok(h)) = (
                parts[parts.len() - 3].parse::<f32>(),
                parts[parts.len() - 2].parse::<f32>(),
                parts[parts.len() - 1].parse::<f32>(),
            )
            && h > 0.0
            && h < 1.6
            && w > 40.0
        {
            raw.push(y + h * 0.5);
        }
        from += rel + 5;
    }
    from = 0;
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

fn pdf_fill_hs(pdf: &[u8], r: f32, g: f32, b: f32) -> Vec<f32> {
    pdf_fill_rects(pdf, r, g, b)
        .into_iter()
        .filter_map(|(w, h)| (w > 100.0 && h > 8.0).then_some(h))
        .collect()
}

fn pdf_fill_ws(pdf: &[u8], r: f32, g: f32, b: f32) -> Vec<f32> {
    pdf_fill_rects(pdf, r, g, b)
        .into_iter()
        .map(|(w, _)| w)
        .collect()
}

fn pdf_fill_rects(pdf: &[u8], r: f32, g: f32, b: f32) -> Vec<(f32, f32)> {
    let needle = format!("{r:.3} {g:.3} {b:.3} rg ");
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(&needle) {
        let rest = &hay[from + rel + needle.len()..];
        let end = rest.find(" re f").unwrap_or(0);
        let parts: Vec<&str> = rest[..end].split_whitespace().collect();
        if parts.len() >= 4
            && let Ok(h) = parts[3].parse::<f32>()
            && let Ok(w) = parts[2].parse::<f32>()
        {
            out.push((w, h));
        }
        from += rel + needle.len();
    }
    out
}

fn pdf_has_filled_polygon(hay: &str) -> bool {
    hay.contains(" h f") || hay.contains("\nh f") || hay.contains(" h f\n")
}

fn pdf_has_cubic(hay: &str) -> bool {
    let tokens: Vec<&str> = hay.split_whitespace().collect();
    tokens.windows(7).any(|w| {
        w[6] == "c"
            && w[0].parse::<f32>().is_ok()
            && w[1].parse::<f32>().is_ok()
            && w[2].parse::<f32>().is_ok()
            && w[3].parse::<f32>().is_ok()
            && w[4].parse::<f32>().is_ok()
            && w[5].parse::<f32>().is_ok()
    })
}

fn pdf_has_wavy_stroke(hay: &str) -> bool {
    // Wave underline: short strokes with both dx and dy (not a rule, not a stem).
    let tokens: Vec<&str> = hay.split_whitespace().collect();
    let mut n = 0u32;
    let mut i = 0;
    while i + 6 < tokens.len() {
        if tokens[i + 2] == "m"
            && tokens[i + 5] == "l"
            && let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
                tokens[i].parse::<f32>(),
                tokens[i + 1].parse::<f32>(),
                tokens[i + 3].parse::<f32>(),
                tokens[i + 4].parse::<f32>(),
            )
        {
            let dx = (x2 - x1).abs();
            let dy = (y2 - y1).abs();
            if dx > 0.8 && dx < 8.0 && dy > 0.35 && dy < 3.0 {
                n += 1;
            }
        }
        i += 1;
    }
    n >= 4
}

fn pdf_has_vertical_stroke(hay: &str) -> bool {
    let tokens: Vec<&str> = hay.split_whitespace().collect();
    let mut i = 0;
    while i + 5 < tokens.len() {
        if tokens[i + 2] == "m"
            && tokens[i + 5] == "l"
            && let (Ok(x1), Ok(y1), Ok(x2), Ok(y2)) = (
                tokens[i].parse::<f32>(),
                tokens[i + 1].parse::<f32>(),
                tokens[i + 3].parse::<f32>(),
                tokens[i + 4].parse::<f32>(),
            )
            && (x1 - x2).abs() < 0.6
            && (y1 - y2).abs() > 4.0
        {
            return true;
        }
        i += 1;
    }
    false
}

fn pdf_literal_y(hay: &str, tf: &str, lit: &str) -> Option<f32> {
    if let Some(i) = hay.find(lit) {
        let before = &hay[i.saturating_sub(220)..i];
        if let Some(cm) = before.rfind("0.24 0 0 0.24 ") {
            let rest = &before[cm + 14..];
            let mut parts = rest.split_whitespace();
            let _x = parts.next();
            if let Some(y) = parts.next().and_then(|s| s.parse().ok()) {
                return Some(y);
            }
        }
    }
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tf) {
        let abs = from + rel;
        let window = &hay[abs..hay.len().min(abs + tf.len() + 96)];
        if window.contains(lit)
            && let Some(td) = window.find(" Td")
        {
            let before = &window[..td];
            let mut parts = before.rsplit([' ', '\n']);
            if let Some(y) = parts.next().and_then(|s| s.parse::<f32>().ok()) {
                return Some(y);
            }
        }
        from = abs + tf.len();
    }
    None
}

fn pdf_tf_ys(pdf: &[u8], tf: &str) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let mut ys = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tf) {
        let start = from + rel;
        let end = hay.len().min(start + tf.len() + 80);
        let slice = &hay[start..end];
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
    let ppem = match tf {
        "11.04 Tf" => Some("46 Tf"),
        "16.08 Tf" => Some("67 Tf"),
        "10.08 Tf" => Some("42 Tf"),
        _ => None,
    };
    if let Some(ppem) = ppem {
        ys.extend(
            pdf_device_xy(hay.as_ref(), ppem)
                .into_iter()
                .map(|(_, y)| y),
        );
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
    let header_ys = pdf_tf_ys(&pdf, "11.04 Tf");
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
fn later_section_without_hf_refs_keeps_prior_header() {
    // comments-lots: only the first sectPr lists headerReference. Word
    // inherits that header onto the landscape and following portrait
    // sections. We cleared chrome and dropped the running head (p6–p9).
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"32\"/></w:rPr>\
           <w:t>PortraitBody</w:t></w:r></w:p>\
         <w:p><w:pPr><w:sectPr>\
           <w:headerReference w:type=\"default\" r:id=\"rIdH1\"/>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr></w:pPr></w:p>\
         <w:p><w:r><w:rPr><w:sz w:val=\"32\"/></w:rPr>\
           <w:t>LandBody</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"15840\" w:h=\"12240\" w:orient=\"landscape\"/></w:sectPr>";
    let pdf = docx_to_pdf(&hf_docx(
        body,
        &[("rIdH1", "header", "header1.xml")],
        &[("word/header1.xml", hf_part("hdr", 22, "KeepHeader"))],
    ))
    .expect("convert inherited header");
    assert_eq!(pdf_page_count(&pdf), 2);
    let header_ys = pdf_tf_ys(&pdf, "11.04 Tf");
    // Landscape page is 612pt tall; header baseline is ~565 (612-36-ascent).
    // Portrait-only chrome sits near 745 and must not be the only hit.
    assert!(
        header_ys.iter().any(|&y| (540.0..590.0).contains(&y)),
        "inherited header must paint on the 612pt landscape page; ys={header_ys:?}"
    );
}

#[test]
fn normal_default_survives_later_nolist_default() {
    // comments-lots styles.xml: Normal w:default=1 (Aptos 10.5) then
    // TableNormal and NoList also w:default=1. Taking the last default
    // left unstyled paras on docDefaults Calibri 11 (title 30pt Calibri
    // instead of Aptos).
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:rPrDefault><w:rPr>\
             <w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"/>\
             <w:sz w:val=\"22\"/>\
           </w:rPr></w:rPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
             <w:rPr><w:rFonts w:ascii=\"Aptos\" w:hAnsi=\"Aptos\"/>\
               <w:sz w:val=\"21\"/></w:rPr>\
           </w:style>\
           <w:style w:type=\"table\" w:default=\"1\" w:styleId=\"TableNormal\">\
             <w:name w:val=\"Normal Table\"/>\
           </w:style>\
           <w:style w:type=\"numbering\" w:default=\"1\" w:styleId=\"NoList\">\
             <w:name w:val=\"No List\"/>\
           </w:style>\
         </w:styles>";
    let body = "<w:p><w:r><w:t>NormalBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert Normal default");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("10.50 Tf"),
        "Normal sz=21 must win over later NoList default; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        text.contains("/Aptos"),
        "Normal ascii Aptos must win over docDefaults Calibri"
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

fn pdf_font_ascent(pdf: &[u8], font: &str) -> Option<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let needle = format!("/FontName /{font}");
    let i = hay.find(&needle)?;
    let window = &hay[i..hay.len().min(i + 400)];
    let a = window.find("/Ascent ")?;
    window[a + 8..].split_whitespace().next()?.parse().ok()
}

fn pdf_title_box_top(pdf: &[u8], tf: &str, font: &str, page_h: f32) -> Option<f32> {
    let baseline = pdf_tf_ys(pdf, tf)
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max);
    if !baseline.is_finite() {
        return None;
    }
    let size: f32 = tf.split_whitespace().next()?.parse().ok()?;
    let ascent = pdf_font_ascent(pdf, font)?;
    Some(page_h - baseline - ascent * size / 1000.0)
}

#[test]
fn cambria_title_font_ascent_is_thousand_unit() {
    // file_146 / 175 / 176: Cambria is 2048 UPM. Word Quartz writes
    // FontDescriptor Ascent/FontBBox in 1000-unit glyph space, so the
    // title box top sits on pgMar top (1300 twips = 65pt). We emitted
    // raw 2048-unit Ascent (~1590) and fitz/ITT saw the title at y=44.
    let body = "<w:p><w:r><w:rPr><w:rFonts w:ascii=\"Cambria\" w:hAnsi=\"Cambria\"/>\
           <w:b/><w:sz w:val=\"64\"/></w:rPr><w:t>TitleCambria</w:t></w:r></w:p>\
         <w:sectPr>\
           <w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1300\" w:right=\"1440\" w:bottom=\"1300\" w:left=\"1440\" \
             w:header=\"708\" w:footer=\"708\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Cambria title");
    let ascent = pdf_font_ascent(&pdf, "Cambria-Bold").unwrap_or(0.0);
    assert!(
        (600.0..1100.0).contains(&ascent),
        "PDF Ascent must be 1000-unit, not 2048-unit; ascent={ascent}"
    );
    let top = pdf_title_box_top(&pdf, "31.92 Tf", "Cambria-Bold", 792.0).expect("title box");
    assert!(
        (top - 65.0).abs() < 4.0,
        "Word title box top is pgMar 65pt; top={top}"
    );
}

#[test]
fn official_file_146_title_box_sits_on_top_margin() {
    // Word Quartz: Cambria-Bold 31.92 title bbox y=65.20 (top=1300).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let top = pdf_title_box_top(&pdf, "31.92 Tf", "Cambria-Bold", 792.0).expect("file_146 title");
    assert!(
        (top - 65.2).abs() < 4.0,
        "official title box must sit on Word top=65; top={top}"
    );
}

fn pdf_device_xy(hay: &str, ppem_tf: &str) -> Vec<(f32, f32)> {
    let mut out = Vec::new();
    let mut from = 0;
    let needle = "0.24 0 0 0.24 ";
    while let Some(rel) = hay[from..].find(needle) {
        let rest = &hay[from + rel + needle.len()..];
        let mut parts = rest.split_whitespace();
        let x = parts.next().and_then(|s| s.parse().ok());
        let y = parts.next().and_then(|s| s.parse().ok());
        let cm = parts.next();
        if cm == Some("cm")
            && let (Some(x), Some(y)) = (x, y)
        {
            let window = &rest[..rest.len().min(160)];
            if window.contains(ppem_tf) {
                out.push((x, y));
            }
        }
        from += rel + needle.len();
    }
    out
}

fn pdf_tf_xs(pdf: &[u8], tf: &str) -> Vec<f32> {
    let hay = String::from_utf8_lossy(pdf);
    let mut xs = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tf) {
        let start = from + rel;
        let end = hay.len().min(start + tf.len() + 80);
        let slice = &hay[start..end];
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
    let ppem = match tf {
        "11.04 Tf" => Some("46 Tf"),
        "16.08 Tf" => Some("67 Tf"),
        "10.08 Tf" => Some("42 Tf"),
        _ => None,
    };
    if let Some(ppem) = ppem {
        xs.extend(
            pdf_device_xy(hay.as_ref(), ppem)
                .into_iter()
                .map(|(x, _)| x),
        );
    }
    xs
}

fn pdf_tf_glyphs(pdf: &[u8], tf: &str) -> Vec<(f32, f32, String)> {
    let hay = String::from_utf8_lossy(pdf);
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(tf) {
        let start = from + rel;
        let window = &hay[start..hay.len().min(start + tf.len() + 120)];
        if let Some(td) = window.find(" Td") {
            let before = &window[..td];
            let mut parts = before.rsplit([' ', '\n']);
            let y = parts.next().and_then(|s| s.parse::<f32>().ok());
            let x = parts.next().and_then(|s| s.parse::<f32>().ok());
            let after = &window[td + 3..];
            if let (Some(x), Some(y)) = (x, y)
                && let Some(tj) = after.find(") Tj")
                && let Some(paren) = after[..tj].rfind('(')
            {
                let inner = &after[paren + 1..tj];
                if inner.chars().all(|c| c.is_ascii_graphic() || c == ' ') && !inner.is_empty() {
                    out.push((x, y, inner.to_string()));
                }
            }
        }
        from += rel + tf.len();
    }
    out
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
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
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
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
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

fn endnotes_docx(body: &str, notes: &str) -> Vec<u8> {
    let endnotes = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:endnotes xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         {notes}</w:endnotes>"
    );
    hf_docx(
        body,
        &[("rIdEn", "endnotes", "endnotes.xml")],
        &[("word/endnotes.xml", endnotes)],
    )
}

fn footnotes_docx(body: &str, notes: &str) -> Vec<u8> {
    let footnotes = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:footnotes xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
         {notes}</w:footnotes>"
    );
    hf_docx(
        body,
        &[("rIdFn", "footnotes", "footnotes.xml")],
        &[("word/footnotes.xml", footnotes)],
    )
}

#[test]
fn cover_pages_gallery_starts_on_its_own_page() {
    // Word's Cover Pages building block sits mid-flow (Strict01 landscape
    // p5) and occupies that page alone. The gallery already ships a
    // trailing page break; without a break *before* the SDT the cover
    // shares the previous page (5 landscape instead of 6).
    let body = "<w:p><w:r><w:t>BeforeCover</w:t></w:r></w:p>\
         <w:sdt><w:sdtPr><w:docPartObj>\
           <w:docPartGallery w:val=\"Cover Pages\"/><w:docPartUnique/>\
         </w:docPartObj></w:sdtPr>\
         <w:sdtContent>\
           <w:p><w:r><w:t>CoverOnly</w:t></w:r></w:p>\
           <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         </w:sdtContent></w:sdt>\
         <w:p><w:r><w:t>AfterCover</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert cover gallery");
    assert_eq!(
        pdf_page_count(&pdf),
        3,
        "cover gallery is its own page (before / cover / after)"
    );
}

#[test]
fn referenced_endnote_is_painted() {
    // Word dumps endnotes after the body on the same page when they fit
    // (Strict01 p13 is SmartArt + "This is an endnote.").
    let body = "<w:p><w:r><w:t>BodyLine</w:t></w:r>\
           <w:r><w:endnoteReference w:id=\"1\"/></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"612pt\" w:h=\"792pt\"/></w:sectPr>";
    let notes = "<w:endnote w:type=\"separator\" w:id=\"-1\"><w:p/></w:endnote>\
         <w:endnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p/></w:endnote>\
         <w:endnote w:id=\"1\"><w:p><w:r><w:t>This is an endnote.</w:t></w:r></w:p></w:endnote>";
    let pdf = docx_to_pdf(&endnotes_docx(body, notes)).expect("convert endnote");
    assert_eq!(pdf_page_count(&pdf), 1, "short body + endnote share a page");
    let text = String::from_utf8_lossy(&pdf);
    let shows = text.matches(" Tj").count();
    assert!(
        shows >= 2,
        "endnote text must paint in addition to the body; got {shows} Tj"
    );
}

#[test]
fn footnotes_stay_unpainted_after_mini_94() {
    // Word paints potpourri / file_170 footnote bodies in the bottom
    // margin. Mini 94 did that and dropped potpourri −0.15 / file_170
    // −0.23 ITT (0 better). Keep footnotes.xml silent.
    let body = "<w:p><w:r><w:t>BodyLine</w:t></w:r>\
           <w:r><w:rPr><w:vertAlign w:val=\"superscript\"/></w:rPr>\
             <w:footnoteReference w:id=\"1\"/></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let notes = "<w:footnote w:type=\"separator\" w:id=\"-1\"><w:p/></w:footnote>\
         <w:footnote w:type=\"continuationSeparator\" w:id=\"0\"><w:p/></w:footnote>\
         <w:footnote w:id=\"1\"><w:p><w:r><w:t>Footnote one lives here.</w:t></w:r></w:p></w:footnote>";
    let pdf = docx_to_pdf(&footnotes_docx(body, notes)).expect("convert footnote");
    assert_eq!(pdf_page_count(&pdf), 1, "short body stays one page");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        !painted.contains("Footnote one lives here"),
        "footnote bodies stay unpainted after mini 94 ITT; painted={painted}"
    );
}

#[test]
fn footnote_reference_stays_unpainted_after_mini_102() {
    // Word paints a superscript "1" at w:footnoteReference. Doing that
    // (mini 102) dropped potpourri −0.014 and file_170 −0.016; 0 better.
    let body = "<w:p><w:r><w:t>SeeNoteHere.</w:t></w:r>\
           <w:r><w:rPr><w:vertAlign w:val=\"superscript\"/></w:rPr>\
             <w:footnoteReference w:id=\"1\"/></w:r></w:p>\
         <w:sectPr/>";
    let notes = "<w:footnote w:type=\"separator\" w:id=\"-1\"><w:p/></w:footnote>\
         <w:footnote w:id=\"1\"><w:p><w:r><w:t>Footnote body stays off.</w:t></w:r></w:p></w:footnote>";
    let pdf = docx_to_pdf(&footnotes_docx(body, notes)).expect("convert fn ref");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("SeeNoteHere"),
        "body must paint; painted={painted}"
    );
    assert!(
        !painted.contains('1'),
        "in-text footnote marker is ITT-wrong after mini 102; painted={painted}"
    );
}

#[test]
fn official_potpourri_stays_five_pages_without_footnote_ink_after_mini_94() {
    // Word p1 y≈708 has "Footnote one…". Painting it (mini 94) was
    // ITT-wrong. Keep 5pp and no footnote-body literals.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "need page streams");
    let p1 = pdf_winansi_text(pages[0].as_bytes());
    assert!(
        !p1.contains("Footnote one") && !p1.contains("introductory paragraph"),
        "potpourri footnote body stays unpainted after mini 94; p1={p1}"
    );
}

#[test]
fn official_strict01_matches_word_thirteen_pages() {
    // Official no_comments Word oracle is 13 pages: 3 portrait, 6
    // landscape, 4 portrait. The shipped converter emits 11 (3+5+3) —
    // pagefair then zeros the unpaired pages (score ~33).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let docx = std::fs::read(path).expect("official Strict01.docx");
    let pdf = docx_to_pdf(&docx).expect("convert official Strict01");
    let boxes = pdf_mediaboxes(&pdf);
    let n = pdf_page_count(&pdf);
    assert_eq!(
        n, 13,
        "Word Strict01 is 13pp (3P+6L+4P); got {n} boxes={boxes:?}"
    );
    let land = boxes.iter().filter(|&&(w, h)| w > h + 10.0).count();
    assert_eq!(
        land, 6,
        "mid-doc landscape section is six pages; land={land} boxes={boxes:?}"
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
    let ys = pdf_tf_ys(&pdf, "11.04 Tf");
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
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
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
fn table_cell_interior_empty_para_stays_skipped_after_mini_78() {
    // Word file_146 code listing wants the interior blank (~24pt).
    // Keeping interior empties (mini empty) was file_146 +3.0 but
    // eigenpal_2 −8.3 / sample −2.5 and median 51.20→50.57. Skip all
    // empty cell paras, including interior.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"9360\"/></w:tblGrid>\
         <w:tr><w:tc>\
           <w:p><w:r><w:rPr><w:rFonts w:ascii=\"Courier New\"/>\
             <w:sz w:val=\"19\"/></w:rPr><w:t>importCss</w:t></w:r></w:p>\
           <w:p></w:p>\
           <w:p><w:r><w:rPr><w:rFonts w:ascii=\"Courier New\"/>\
             <w:sz w:val=\"19\"/></w:rPr><w:t>functionApp</w:t></w:r></w:p>\
           <w:p></w:p>\
         </w:tc></w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert interior empty cell para");
    let ys = distinct_tf_ys(&pdf, "9.50 Tf");
    assert_eq!(ys.len(), 2, "no painted blank line; ys={ys:?}");
    let gap = (ys[0] - ys[1]).abs();
    assert!(
        gap < 20.0,
        "mini 78/empty interior blank was ITT-wrong; gap={gap} ys={ys:?}"
    );
}

#[test]
fn table_cell_trailing_empty_para_is_not_a_blank_line() {
    // mini 78: shipping trailing empty <w:p> doubled every Word cell.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"3600\"/></w:tblGrid>\
         <w:tr><w:tc>\
           <w:p><w:r><w:rPr><w:rFonts w:ascii=\"Courier New\"/>\
             <w:sz w:val=\"19\"/></w:rPr><w:t>OnlyLine</w:t></w:r></w:p>\
           <w:p></w:p>\
         </w:tc></w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert trailing empty cell para");
    let ys = distinct_tf_ys(&pdf, "9.50 Tf");
    assert_eq!(
        ys.len(),
        1,
        "trailing empty cell para must not add a line; ys={ys:?}"
    );
}

#[test]
fn table_cell_empty_pbdr_paints_signature_rule_after_mini_78() {
    // file_146 / sample_iter2 Sign-off cells: EigenPal, Maintainer
    // (after=320), empty <w:p> with pBdr bottom E2E8F0 (the signature
    // line), then "signature · date". mini 78 skipped every empty cell
    // para, so Word's rule vanished and both Sign-off tables packed
    // onto page 6 (Word page 7 is the second table). Keep pBdr empties;
    // plain interior empties stay skipped.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4680\"/></w:tblGrid>\
         <w:tr><w:tc>\
           <w:p><w:r><w:t>EigenPal</w:t></w:r></w:p>\
           <w:p><w:pPr><w:spacing w:after=\"320\"/></w:pPr>\
             <w:r><w:t>Maintainer</w:t></w:r></w:p>\
           <w:p><w:pPr><w:pBdr>\
             <w:bottom w:val=\"single\" w:sz=\"3\" w:space=\"1\" w:color=\"E2E8F0\"/>\
           </w:pBdr><w:spacing w:after=\"60\"/></w:pPr></w:p>\
           <w:p><w:r><w:t>signature</w:t></w:r></w:p>\
         </w:tc></w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert empty pBdr cell para");
    let bars = pdf_fill_rects(&pdf, 0.886, 0.910, 0.941);
    assert!(
        bars.iter()
            .any(|(w, h)| *w > 100.0 && (0.20..0.40).contains(h)),
        "Word Sign-off empty pBdr is an E2E8F0 hairline; bars={bars:?}"
    );
    let hay = String::from_utf8_lossy(&pdf);
    let e_y = pdf_cm_tj_xy(&hay, "E")
        .into_iter()
        .map(|(_, y)| y)
        .fold(0.0_f32, f32::max);
    let s_y = pdf_cm_tj_xy(&hay, "s")
        .into_iter()
        .map(|(_, y)| y)
        .fold(0.0_f32, f32::max);
    let gap = (e_y - s_y).abs();
    assert!(
        gap > 30.0,
        "empty pBdr must occupy a line between EigenPal and signature; E={e_y} s={s_y} gap={gap}"
    );
}

#[test]
fn table_cell_after_320_stays_line_box_after_mini_300() {
    // Word Sign-off Maintainer after=320 is 16pt. Honoring cell after
    // ≥10pt (mini 300–302) was no-redline +0.008/+0.016 but redline
    // mean −0.010 (file_78_file_79 −0.49). Keep the \\n line box.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"4680\"/></w:tblGrid>\
         <w:tr><w:tc>\
           <w:p><w:pPr><w:spacing w:after=\"320\"/></w:pPr>\
             <w:r><w:t>AlphaGap</w:t></w:r></w:p>\
           <w:p><w:r><w:t>BetaGap</w:t></w:r></w:p>\
         </w:tc></w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert cell after=320");
    let hay = String::from_utf8_lossy(&pdf);
    let a_y = pdf_cm_tj_xy(&hay, "A")
        .into_iter()
        .map(|(_, y)| y)
        .fold(0.0_f32, f32::max);
    let b_y = pdf_cm_tj_xy(&hay, "B")
        .into_iter()
        .map(|(_, y)| y)
        .fold(0.0_f32, f32::max);
    let gap = (a_y - b_y).abs();
    assert!(
        gap < 20.0,
        "mini 300 cell after=320 was RL ITT-neg; keep line box; A={a_y} B={b_y} gap={gap}"
    );
}

#[test]
fn table_cell_listing_after_80_stays_collapsed_after_mini_188() {
    // Code-listing cells use after=80 (4pt) on every line. Honoring that
    // globally is mini 188 (median −2.20). Keep the 9.5 line box.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"9360\"/></w:tblGrid>\
         <w:tr><w:tc>\
           <w:p><w:pPr><w:spacing w:after=\"80\"/></w:pPr>\
             <w:r><w:rPr><w:rFonts w:ascii=\"Courier New\"/>\
               <w:sz w:val=\"19\"/></w:rPr><w:t>importCss</w:t></w:r></w:p>\
           <w:p><w:pPr><w:spacing w:after=\"80\"/></w:pPr>\
             <w:r><w:rPr><w:rFonts w:ascii=\"Courier New\"/>\
               <w:sz w:val=\"19\"/></w:rPr><w:t>functionApp</w:t></w:r></w:p>\
         </w:tc></w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert listing after=80");
    let ys = distinct_tf_ys(&pdf, "9.50 Tf");
    assert_eq!(ys.len(), 2, "two listing lines; ys={ys:?}");
    let gap = (ys[0] - ys[1]).abs();
    assert!(
        gap < 20.0,
        "after=80 listing must stay a 9.5 line box; gap={gap} ys={ys:?}"
    );
}

#[test]
fn official_file_146_second_signoff_table_is_on_page_seven() {
    // Word p6 ends with the first Sign-off table + the next heading;
    // p7 is the duplicate EigenPal/Contributor table. Empty pBdr
    // signature lines (not after=320 — mini 300 RL −0.010) overflow
    // table 2. Keep 7pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let pages = pdf_content_streams(&pdf);
    assert_eq!(pages.len(), 7, "Word file_146 is 7 content streams");
    let p6 = pdf_winansi_text(pages[5].as_bytes());
    let p7 = pdf_winansi_text(pages[6].as_bytes());
    assert!(
        p6.contains("EigenPal"),
        "Word p6 still has the first Sign-off table; p6 len {}",
        p6.len()
    );
    assert!(
        p7.contains("EigenPal") && p7.len() < 800,
        "Word p7 is the second Sign-off table only; p7={p7} len={}",
        p7.len()
    );
}

#[test]
fn cell_tcmar_left_overrides_tblcellmar() {
    // file_146 / 175 / 176 / sample code listing: tblCellMar left=10 twips
    // but the cell has tcMar left=200 (10pt). Word paints Courier at
    // 72+10=82; we used the table 10-twip pad (x≈72.5).
    let body = "<w:tbl><w:tblPr>\
           <w:tblW w:w=\"9360\" w:type=\"dxa\"/>\
           <w:tblCellMar>\
             <w:left w:w=\"10\" w:type=\"dxa\"/><w:right w:w=\"10\" w:type=\"dxa\"/>\
           </w:tblCellMar></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"9360\"/></w:tblGrid>\
           <w:tr><w:tc><w:tcPr>\
             <w:tcMar>\
               <w:top w:w=\"120\" w:type=\"dxa\"/><w:left w:w=\"200\" w:type=\"dxa\"/>\
               <w:bottom w:w=\"120\" w:type=\"dxa\"/><w:right w:w=\"200\" w:type=\"dxa\"/>\
             </w:tcMar></w:tcPr>\
             <w:p><w:r><w:rPr>\
               <w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>\
               <w:sz w:val=\"19\"/></w:rPr>\
               <w:t>ImportLine</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert tcMar cell");
    let xs = pdf_tf_xs(&pdf, "9.50 Tf");
    assert!(!xs.is_empty(), "Courier 9.5 must paint; xs={xs:?}");
    let x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        (80.0..86.0).contains(&x),
        "tcMar 200 twips must sit at Word 82, not tblCellMar 10 twips (72.5); x={x} xs={xs:?}"
    );
}

#[test]
fn tblcellmar_start_end_stays_default_after_mini_marse() {
    // Cicero: tblCellMar start/end=160. Mapping to left/right (mini
    // 221–224) dropped Cicero −0.027 ITT. Keep default 108 twips.
    let body = "<w:tbl><w:tblPr>\
           <w:tblW w:w=\"9360\" w:type=\"dxa\"/>\
           <w:tblCellMar>\
             <w:top w:w=\"80\" w:type=\"dxa\"/>\
             <w:start w:w=\"1440\" w:type=\"dxa\"/>\
             <w:bottom w:w=\"80\" w:type=\"dxa\"/>\
             <w:end w:w=\"1440\" w:type=\"dxa\"/>\
           </w:tblCellMar></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"9360\"/></w:tblGrid>\
           <w:tr><w:tc><w:p><w:r><w:t>PadCell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert start/end mar");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(!xs.is_empty(), "PadCell must paint; xs={xs:?}");
    let x = xs.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        (74.0..82.0).contains(&x),
        "mini marse was Cicero ITT-neg; keep default 108 twips (x=77); x={x} xs={xs:?}"
    );
}

#[test]
fn official_file_146_code_import_uses_cell_tcmar() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let pages = pdf_content_streams(&pdf);
    let xs = pdf_tf_xs(pages[0].as_bytes(), "9.50 Tf");
    assert!(
        xs.iter().any(|x| (80.0..86.0).contains(x)),
        "Word code listing starts at 82.6 (tcMar 200); xs={xs:?}"
    );
}

#[test]
fn table_courier_nine_point_five_keeps_eleven_pt_line_box_after_mini_c9() {
    // Word file_146 listing inner is ~10.80pt. Painting Courier 9.5×1.15
    // without growing the 11pt row (mini c9) dropped the 3pp sample/
    // eigenpal cluster −0.13 (median 53.43→53.30). Keep 11×1.15.
    let lines = (0..3)
        .map(|i| {
            format!(
                "<w:p><w:r><w:rPr>\
                   <w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>\
                   <w:sz w:val=\"19\"/></w:rPr>\
                   <w:t>import{i}</w:t></w:r></w:p>"
            )
        })
        .collect::<String>();
    let body = format!(
        "<w:tbl><w:tblPr><w:tblW w:w=\"9360\" w:type=\"dxa\"/></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"9360\"/></w:tblGrid>\
           <w:tr><w:tc><w:tcPr>\
             <w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F1F5F9\"/>\
             <w:tcMar>\
               <w:top w:w=\"120\" w:type=\"dxa\"/><w:left w:w=\"200\" w:type=\"dxa\"/>\
               <w:bottom w:w=\"120\" w:type=\"dxa\"/><w:right w:w=\"200\" w:type=\"dxa\"/>\
             </w:tcMar></w:tcPr>\
             {lines}</w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert courier listing");
    let boxes = pdf_fill_boxes_in(&String::from_utf8_lossy(&pdf), 0.945, 0.961, 0.976);
    let inner: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| *w > 80.0 && (8.0..16.0).contains(h))
        .collect();
    assert!(!inner.is_empty(), "listing inner fills; boxes={boxes:?}");
    let max_h = inner.iter().map(|(_, _, _, h)| *h).fold(0.0_f32, f32::max);
    assert!(
        (12.4..=12.9).contains(&max_h),
        "Courier 9.5 listing stays 11×1.15=12.65 after mini c9; max_h={max_h} inner={inner:?}"
    );
}

#[test]
fn cell_tcmar_top_bottom_lengthens_row_and_insets_first_line() {
    // sample_iter2 / file_146 npm row: no tblCellMar, but the cell has
    // tcMar top=100 / bottom=100 (5+5pt) and left=160. Word's F8FAFC
    // outer is 22.32pt with a 5.04pt pad strip above the inner line
    // fill. We used generic 8pt bottom chrome (20.65) and painted the
    // inner line at the cell top.
    let body = "<w:tbl><w:tblPr>\
           <w:tblW w:w=\"4680\" w:type=\"dxa\"/></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"4680\"/></w:tblGrid>\
           <w:tr><w:tc><w:tcPr>\
             <w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F8FAFC\"/>\
             <w:tcMar>\
               <w:top w:w=\"100\" w:type=\"dxa\"/><w:left w:w=\"160\" w:type=\"dxa\"/>\
               <w:bottom w:w=\"100\" w:type=\"dxa\"/><w:right w:w=\"160\" w:type=\"dxa\"/>\
             </w:tcMar></w:tcPr>\
             <w:p><w:r><w:t>npm</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert tcMar v");
    let boxes = pdf_fill_boxes_in(&String::from_utf8_lossy(&pdf), 0.973, 0.980, 0.988);
    let outer: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| *h > 16.0 && *w > 100.0)
        .collect();
    assert!(
        !outer.is_empty(),
        "F8FAFC cell fill must paint; boxes={boxes:?}"
    );
    let max_h = outer.iter().map(|(_, _, _, h)| *h).fold(0.0_f32, f32::max);
    assert!(
        (21.5..=24.5).contains(&max_h),
        "tcMar 100+100 must replace 8pt chrome: 11×1.15+10≈22.65, not 20.65; max_h={max_h} outer={outer:?}"
    );
    let inner: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| (10.0..16.0).contains(h) && *w > 80.0)
        .collect();
    assert!(
        !inner.is_empty(),
        "Word paints an inset line fill; boxes={boxes:?}"
    );
    let outer_top = outer
        .iter()
        .map(|(x, y, _, h)| (*x, y + h))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();
    let inner_top = inner
        .iter()
        .map(|(x, y, _, h)| (*x, y + h))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();
    let pad = outer_top.1 - inner_top.1;
    assert!(
        (4.0..6.5).contains(&pad),
        "tcMar top 100 twips is a 5pt strip above the line fill; pad={pad} outer_top={outer_top:?} inner_top={inner_top:?}"
    );
}

#[test]
fn cell_tcmar_80_stays_flush_after_mini_pill80() {
    // Word 1E293B inner is pad_t below outer. Insetting when 80+80==
    // chrome (mini 257–260) was no-redline +0.033/+0.102 but redline
    // mean −0.005 (file_34 −0.25). Keep flush tops.
    let body = "<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/></w:tblGrid>\
         <w:tr><w:tc><w:tcPr>\
           <w:shd w:val=\"clear\" w:fill=\"1E293B\"/>\
           <w:tcMar>\
             <w:top w:w=\"80\" w:type=\"dxa\"/><w:left w:w=\"140\" w:type=\"dxa\"/>\
             <w:bottom w:w=\"80\" w:type=\"dxa\"/><w:right w:w=\"140\" w:type=\"dxa\"/>\
           </w:tcMar></w:tcPr>\
           <w:p><w:r><w:rPr><w:sz w:val=\"19\"/></w:rPr><w:t>Prop</w:t></w:r></w:p>\
         </w:tc></w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 1E293B pill");
    let boxes = pdf_fill_boxes_in(&String::from_utf8_lossy(&pdf), 0.118, 0.161, 0.231);
    let outer: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| *h > 16.0 && *w > 40.0)
        .collect();
    let inner: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(_, _, w, h)| (8.0..16.0).contains(h) && *w > 40.0)
        .collect();
    assert!(!outer.is_empty(), "pill outer fill; outer={outer:?}");
    assert!(!inner.is_empty(), "pill inner fill; inner={inner:?}");
    let max_h = outer.iter().map(|(_, _, _, h)| *h).fold(0.0_f32, f32::max);
    assert!(
        (18.0..23.0).contains(&max_h),
        "80+80 must not grow past chrome row (~20.6); max_h={max_h} outer={outer:?}"
    );
    let outer_top = outer
        .iter()
        .map(|(_, y, _, h)| y + h)
        .fold(0.0_f32, f32::max);
    let inner_top = inner
        .iter()
        .map(|(_, y, _, h)| y + h)
        .fold(0.0_f32, f32::max);
    let pad = outer_top - inner_top;
    assert!(
        pad.abs() < 1.0,
        "mini pill80 ITT-neg on redline; keep flush tops; pad={pad} outer={outer:?} inner={inner:?}"
    );
}

#[test]
fn official_file_146_pills_stay_flush_after_mini_pill80() {
    // Word p1 1E293B inner is pad_t below outer. mini 257–260 inset
    // dropped redline mean −0.005. Keep flush; 7pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let pages = pdf_content_streams(&pdf);
    let mut found = false;
    for (i, page) in pages.iter().enumerate() {
        let boxes = pdf_fill_boxes_in(page, 0.118, 0.161, 0.231);
        let outer: Vec<_> = boxes
            .iter()
            .copied()
            .filter(|(_, _, w, h)| *h > 16.0 && *w > 40.0)
            .collect();
        let inner: Vec<_> = boxes
            .iter()
            .copied()
            .filter(|(_, _, w, h)| (8.0..16.0).contains(h) && *w > 40.0)
            .collect();
        if outer.is_empty() || inner.is_empty() {
            continue;
        }
        found = true;
        let outer_top = outer
            .iter()
            .map(|(_, y, _, h)| y + h)
            .fold(0.0_f32, f32::max);
        let inner_top = inner
            .iter()
            .map(|(_, y, _, h)| y + h)
            .fold(0.0_f32, f32::max);
        let pad = outer_top - inner_top;
        assert!(
            pad.abs() < 1.0,
            "page {i} mini pill80 ITT-neg; keep flush 1E293B tops; pad={pad} outer={outer:?} inner={inner:?}"
        );
    }
    assert!(found, "file_146 must still paint 1E293B pills");
}

#[test]
fn official_sample_iter2_npm_row_uses_cell_tcmar_top() {
    // Word p1 F8FAFC npm/github cells: outer 22.32pt, inner line starts
    // 5.04pt below the cell top (tcMar 100/100). Ours was 20.65 with
    // the inner fill flush to the cell top.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sample_document_word_repair_of_our_output_iter2_word_repaired_2.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official sample_iter2"))
        .expect("convert sample_iter2");
    assert_eq!(pdf_page_count(&pdf), 7, "Word sample_iter2 is 7pp");
    let pages = pdf_content_streams(&pdf);
    let boxes = pdf_fill_boxes_in(&pages[0], 0.973, 0.980, 0.988);
    let outer: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(x, _, w, h)| *h > 16.0 && *w > 100.0 && *x < 100.0)
        .collect();
    assert!(
        !outer.is_empty(),
        "npm F8FAFC cell must fill; boxes={boxes:?}"
    );
    let max_h = outer.iter().map(|(_, _, _, h)| *h).fold(0.0_f32, f32::max);
    assert!(
        (21.5..=24.5).contains(&max_h),
        "Word npm row is 22.32pt (tcMar 5+5), not 20.65 chrome; max_h={max_h} outer={outer:?}"
    );
    let inner: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(x, _, w, h)| (10.0..16.0).contains(h) && *w > 80.0 && *x < 100.0)
        .collect();
    assert!(!inner.is_empty(), "npm inner line fill; boxes={boxes:?}");
    let outer_top = outer
        .iter()
        .map(|(_, y, _, h)| y + h)
        .fold(f32::NEG_INFINITY, f32::max);
    let inner_top = inner
        .iter()
        .map(|(_, y, _, h)| y + h)
        .fold(f32::NEG_INFINITY, f32::max);
    let pad = outer_top - inner_top;
    assert!(
        (4.0..6.5).contains(&pad),
        "Word npm pad strip is 5.04pt; pad={pad} outer_top={outer_top} inner_top={inner_top}"
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
    let ys = pdf_tf_ys(&pdf, "11.04 Tf");
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
fn tcpr_nowrap_keeps_a_single_line_row() {
    // table_bookmark / file_134 Test 3 R3C3: w:noWrap + a 63-char
    // sentence in a ~115pt grid. Word keeps one line (overflows);
    // wrapping to 2–3 lines drops the row off Word's pairing.
    let body = "<w:tbl><w:tblPr><w:tblW w:w=\"1440\" w:type=\"dxa\"/>\
           <w:tblBorders>\
             <w:top w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"4\" w:color=\"000000\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"1440\"/></w:tblGrid>\
           <w:tr><w:tc><w:tcPr><w:noWrap/></w:tcPr>\
             <w:p><w:r><w:t>This cell will not wrap text with spaces until it does</w:t></w:r></w:p>\
           </w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert noWrap cell");
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(
        ys.len() >= 2,
        "cell must stroke top and bottom rules, ys={ys:?}"
    );
    let gap = ys[0] - ys[1];
    assert!(
        (16.0..=23.0).contains(&gap),
        "w:noWrap must stay a 1-line row (~20pt), not wrapped 2–3 lines; gap={gap} ys={ys:?}"
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
fn footer_baseline_sits_above_the_footer_margin() {
    // ECMA w:footer is the distance from the page bottom to the *bottom*
    // of the footer. Word comments-lots Aptos 10.5 / footer=720 has the
    // line top at y=743 (baseline ~39pt). We used footer as the baseline
    // (Td 36), so the cap-height sat at 736 — 7pt high on every page of
    // the comments / I_am_sharing cluster.
    let footer = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:p><w:pPr><w:jc w:val=\"right\"/></w:pPr>\
             <w:r><w:rPr><w:sz w:val=\"22\"/></w:rPr><w:t>PageMark</w:t></w:r></w:p>\
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
    .expect("convert footer y");
    let ys = pdf_tf_ys(&pdf, "11.04 Tf");
    let footer_ys: Vec<f32> = ys.iter().copied().filter(|y| *y < 60.0).collect();
    assert!(
        !footer_ys.is_empty(),
        "11pt footer must paint near the bottom; ys={ys:?}"
    );
    let y = footer_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        (38.0..48.0).contains(&y),
        "footer=720 baseline must sit above 36pt by the descender (Word ~39); y={y} footer_ys={footer_ys:?}"
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
        text.contains("11.04 Tf") || text.contains(" 46 Tf"),
        "base run stays 11pt; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    let super_ys = pdf_tf_ys(&pdf, "7.15 Tf");
    assert!(
        !super_ys.is_empty(),
        "superscript must paint at 65% (7.15pt); tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    let base_ys = pdf_tf_ys(&pdf, "11.04 Tf");
    assert!(!base_ys.is_empty(), "base x must paint");
    let super_y = super_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let base_y = base_ys.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        super_y > base_y + 2.0,
        "superscript must sit above the baseline; super={super_y} base={base_y}"
    );
}

#[test]
fn rev_bar_sits_half_an_inch_from_the_page_edge() {
    // Word file_146 / eigenpal / sample Save-as-PDF paints the
    // changed-line mark at x=36 (half of the default 72pt left
    // margin). margin_l-10 sat at 62 and overlapped the title.
    let pdf = docx_to_pdf(&minimal_docx_body(
        "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r>\
           <w:t>fresh insert that must carry a change bar</w:t>\
         </w:r></w:ins></w:p><w:sectPr/>",
    ))
    .expect("convert ins rev bar");
    let xs = pdf_vertical_rule_xs(&pdf);
    assert!(
        xs.iter().any(|x| (34.0..38.0).contains(x)),
        "Word change bar is 0.5in from the page edge; xs={xs:?}"
    );
    assert!(
        xs.iter().all(|x| (*x - 62.0).abs() > 0.5),
        "must not still sit at margin_l-10=62; xs={xs:?}"
    );
}

#[test]
fn rev_bar_stays_half_margin_after_mini_revx() {
    // CiceroDo Word is margin_l-36=54 at left=90pt, but shipping that
    // (mini revx) dropped comments-lots family −0.36 to −0.49 and
    // no-redline mean −0.044. Keep margin_l/2 (x=45).
    let pdf = docx_to_pdf(&minimal_docx_body(
        "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r>\
           <w:t>fresh insert that must carry a change bar</w:t>\
         </w:r></w:ins></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1800\" w:bottom=\"1440\" w:left=\"1800\"/></w:sectPr>",
    ))
    .expect("convert 90pt-margin ins rev bar");
    let xs = pdf_vertical_rule_xs(&pdf);
    assert!(
        xs.iter().any(|x| (43.0..47.0).contains(x)),
        "mini revx margin_l-36 was ITT-wrong; keep x=45; xs={xs:?}"
    );
}

#[test]
fn adjacent_rev_paras_share_one_change_bar() {
    // Word CiceroDo paints one ~395pt changed-line mark through a run
    // of revised paras, not a 12pt tick per paragraph.
    let pdf = docx_to_pdf(&minimal_docx_body(
        "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r>\
           <w:t>first revised paragraph</w:t></w:r></w:ins></w:p>\
         <w:p><w:ins w:id=\"2\" w:author=\"a\"><w:r>\
           <w:t>second revised paragraph</w:t></w:r></w:ins></w:p>\
         <w:sectPr/>",
    ))
    .expect("convert adjacent ins paras");
    let hs = pdf_vertical_rule_hs(&pdf);
    let tall = hs.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        tall > 22.0,
        "adjacent ins paras must merge into one bar, not two ~12pt ticks; hs={hs:?}"
    );
}

#[test]
fn rev_bar_stays_split_across_empty_spacer_after_mini_emptymerge() {
    // Word file_146 p2 merges 19pt empty spacers. Slack 40 (mini 279)
    // lifted file_146 +0.039 / redline +0.015 but no-redline median
    // −0.006 and Cicero −0.037. Keep 16pt ticks.
    let pdf = docx_to_pdf(&minimal_docx_body(
        "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r>\
           <w:t>first revised paragraph</w:t></w:r></w:ins></w:p>\
         <w:p><w:pPr><w:spacing w:before=\"60\" w:after=\"60\"/></w:pPr></w:p>\
         <w:p><w:ins w:id=\"2\" w:author=\"a\"><w:r>\
           <w:t>second revised paragraph</w:t></w:r></w:ins></w:p>\
         <w:sectPr/>",
    ))
    .expect("convert ins paras with empty spacer");
    let hs = pdf_vertical_rule_hs(&pdf);
    let tall = hs.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        tall < 28.0,
        "mini empty-spacer merge was ITT-neg (Cicero −0.037); keep split ticks; hs={hs:?}"
    );
}

#[test]
fn official_cicerodo_p2_has_a_tall_change_bar() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Redline_CiceroDo_v_plate_30.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official CiceroDo")).expect("convert CiceroDo");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 2, "Word CiceroDo is 5pp");
    let hs = pdf_vertical_rule_hs(pages[1].as_bytes());
    let tall = hs.iter().copied().fold(0.0_f32, f32::max);
    assert!(
        tall > 60.0,
        "adjacent revised paras merge (Word p2 is 395pt; 12pt ticks are wrong); hs={hs:?}"
    );
}

#[test]
fn official_file_146_rev_bar_is_half_inch() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    let pages = pdf_content_streams(&pdf);
    let xs = pdf_vertical_rule_xs(pages[0].as_bytes());
    assert!(
        xs.iter().any(|x| (34.0..38.0).contains(x)),
        "Word file_146 title change bar is x=36; xs={xs:?}"
    );
}

#[test]
fn rev_bar_is_word_quartz_filled_rect() {
    // Word file_146 / CiceroDo Quartz paints a 0.72pt (3px@300dpi) fill
    // `36 726.96 m 36.72 726.96 l 36.72 688.56 l 36 688.56 l h f`,
    // left-aligned at the change-bar x — not a 0.75pt stroke
    // (`0.75 w 0.000 0.000 0.000 RG 36.00 … l S`).
    let pdf = docx_to_pdf(&minimal_docx_body(
        "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r>\
           <w:t>fresh insert that must carry a change bar</w:t>\
         </w:r></w:ins></w:p><w:sectPr/>",
    ))
    .expect("convert ins rev bar");
    let hay = String::from_utf8_lossy(&pdf);
    let bars: Vec<_> = pdf_fill_boxes_in(&hay, 0.0, 0.0, 0.0)
        .into_iter()
        .filter(|(x, _, w, h)| *x < 40.0 && *w < 1.5 && *h > 6.0)
        .collect();
    assert!(
        bars.iter()
            .any(|(x, _, w, _)| (35.9..36.1).contains(x) && (0.70..0.74).contains(w)),
        "Word change bar is a 0.72pt fill left-aligned at x=36; bars={bars:?}"
    );
    assert!(
        !hay.contains("0.75 w 0.000 0.000 0.000 RG"),
        "must not keep the 0.75pt stroked change bar"
    );
}

#[test]
fn official_file_146_rev_bar_is_filled_hairline() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let pages = pdf_content_streams(&pdf);
    let bars: Vec<_> = pdf_fill_boxes_in(&pages[0], 0.0, 0.0, 0.0)
        .into_iter()
        .filter(|(x, _, w, h)| (35.9..36.1).contains(x) && *w < 1.5 && *h > 6.0)
        .collect();
    assert!(
        bars.iter().any(|(_, _, w, _)| (0.70..0.74).contains(w)),
        "Word file_146 title change bar is a 0.72pt fill at x=36; bars={bars:?}"
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
    // Word Quartz on addition_removal / file_176 paints del AND first-author
    // ins #D13438 (0.820 0.204 0.220). soffice gold on ins was an ITT miss.
    assert!(
        text.contains("0.820 0.204 0.220"),
        "delText / first-author ins must paint Word markup red; pdf snippet {}",
        text[text.find("rg").unwrap_or(0)..]
            .chars()
            .take(200)
            .collect::<String>()
    );
    assert!(
        !text.contains("0.753 0.565 0.000"),
        "must not keep soffice gold on first-author ins"
    );
}

#[test]
fn ins_without_track_revisions_uses_word_red() {
    // file_176 / CiceroDo / file_19: Word Save-as-PDF paints first-author
    // ins #D13438 even when settings.xml has no w:trackRevisions (4800+
    // gold chars vs Word red). soffice gold is an ITT miss on that cluster.
    let body = "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r><w:t>fresh</w:t></w:r></w:ins></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert ins");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.820 0.204 0.220"),
        "Word first-author ins is #D13438 without trackRevisions; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("0.753 0.565 0.000"),
        "must not keep soffice gold #C09000 on Word-oracle ins"
    );
}

#[test]
fn ins_thirty_two_pt_underline_stays_hairline_after_mini_ul32() {
    // Word file_146 title ins is 2.4pt. all-u / size≥20 / 28pt+ were
    // ITT-neg (mini 197/199/238: green_underline 90.4→89.2; mean
    // 59.1612→59.1552). Keep 0.6pt.
    let body = "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r><w:rPr>\
           <w:b/><w:color w:val=\"0F172A\"/><w:sz w:val=\"64\"/></w:rPr>\
           <w:t>TitleIns</w:t></w:r></w:ins></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 32pt ins");
    let bars = pdf_fill_rects(&pdf, 0.820, 0.204, 0.220);
    let max_h = bars
        .iter()
        .filter(|(w, _)| *w > 20.0)
        .map(|(_, h)| *h)
        .fold(0.0_f32, f32::max);
    assert!(
        (0.45..0.9).contains(&max_h),
        "32pt ins underline stays 0.6pt after mini ul32 ITT; max_h={max_h} bars={bars:?}"
    );
}

#[test]
fn twenty_pt_ins_underline_stays_hairline_after_mini_ul20() {
    // size≥20 (mini 199) dropped no-redline mean −0.007. 20pt stays
    // the 0.6pt hairline; only 28pt+ titles take size×0.075.
    let body = "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r><w:rPr>\
           <w:sz w:val=\"40\"/></w:rPr><w:t>TwentyPtIns</w:t></w:r></w:ins></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 20pt ins");
    let bars = pdf_fill_rects(&pdf, 0.820, 0.204, 0.220);
    let max_h = bars
        .iter()
        .filter(|(w, _)| *w > 20.0)
        .map(|(_, h)| *h)
        .fold(0.0_f32, f32::max);
    assert!(
        (0.45..0.9).contains(&max_h),
        "20pt ins underline stays 0.6pt after mini ul20; max_h={max_h} bars={bars:?}"
    );
}

#[test]
fn eleven_pt_ins_underline_stays_hairline_after_mini_ulthick() {
    // mini-run 197: size×0.075 on 11pt u dropped green_underline 90.4→89.2.
    let body = "<w:p><w:ins w:id=\"1\" w:author=\"a\"><w:r><w:t>fresh</w:t></w:r></w:ins></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert 11pt ins");
    let bars = pdf_fill_rects(&pdf, 0.820, 0.204, 0.220);
    let max_h = bars
        .iter()
        .filter(|(w, _)| *w > 8.0)
        .map(|(_, h)| *h)
        .fold(0.0_f32, f32::max);
    assert!(
        (0.45..0.9).contains(&max_h),
        "11pt ins underline stays 0.6pt hairline; max_h={max_h} bars={bars:?}"
    );
}

#[test]
fn official_file_146_title_ins_underline_stays_hairline_after_mini_ul32() {
    // Word title ins bar is 2.4pt; 28pt+ scaling was ITT-neg on mean
    // (mini 238). Keep 0.6pt; file_146 stays 7pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_146.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_146")).expect("convert file_146");
    assert_eq!(pdf_page_count(&pdf), 7, "Word file_146 is 7pp");
    let pages = pdf_content_streams(&pdf);
    let boxes = pdf_fill_boxes_in(&pages[0], 0.820, 0.204, 0.220);
    let title: Vec<_> = boxes
        .iter()
        .copied()
        .filter(|(x, _, w, h)| *x < 80.0 && *w > 150.0 && *h > 0.3 && *h < 1.2)
        .collect();
    assert!(
        !title.is_empty(),
        "title ins bar must stay a hairline after mini ul32; boxes={boxes:?}"
    );
}

#[test]
fn official_file_176_ins_is_word_red_not_soffice_gold() {
    // Word file_176 (~4800 ins chars) is #D13438 with trackRevisions off.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_176.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_176")).expect("convert file_176");
    let hay = String::from_utf8_lossy(&pdf);
    let gold = hay.matches("0.753 0.565 0.000").count();
    let red = hay.matches("0.820 0.204 0.220").count();
    assert_eq!(gold, 0, "soffice gold must not paint file_176 ins");
    assert!(
        red > 20,
        "Word first-author ins must be #D13438; red={red} gold={gold}"
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
    let wide = pdf_rgb_rule_widths(&exploded, 209.0 / 255.0, 52.0 / 255.0, 56.0 / 255.0);
    let slim = pdf_rgb_rule_widths(&compact, 209.0 / 255.0, 52.0 / 255.0, 56.0 / 255.0);
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
    // Word colors tracked changes by author. First author is #D13438,
    // second is blue #0040A0 (soffice gold on first-author was ITT-wrong).
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
        text.contains("0.820 0.204 0.220"),
        "first author must paint Word #D13438; tail {}",
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
fn second_author_del_stays_word_red_after_mini_authordel() {
    // Word file_146 sara.k/thomas.v del is teal/gray. Painting those
    // with the ins palette (mini 239) dropped no-redline median
    // 53.4615→53.4464. Keep always-red del.
    let body = "<w:p>\
           <w:del w:id=\"0\" w:author=\"first\"><w:r>\
             <w:delText>gone</w:delText></w:r></w:del>\
           <w:del w:id=\"1\" w:author=\"second\"><w:r>\
             <w:delText>also</w:delText></w:r></w:del>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert two-author del");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.820 0.204 0.220"),
        "del stays Word #D13438; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("0.000 0.251 0.627"),
        "second-author del must not take ins blue (mini 239 ITT-neg)"
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
fn table_grid_borders_are_filled_rects_like_word_quartz() {
    // comments-lots p4 TableGrid: Word Quartz paints sz=4 rules as 0.48pt
    // filled rects (190 fills / 0 strokes). We stroked 0.50pt lines
    // (`l S`), so color_sim saw a black grid against Word's ink-union fills.
    let body = format!(
        "{}<w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>",
        three_row_grid_table()
    );
    let pdf = docx_to_pdf(&docx_with_styles(&body, &table_grid_line240_styles()))
        .expect("convert TableGrid fills");
    let text = String::from_utf8_lossy(&pdf);
    let fills = text.matches("0.000 0.000 0.000 rg").count();
    let hair = text.matches(" 0.50 re f").count();
    let strokes = text.matches("0.50 w 0.000 0.000 0.000 RG").count();
    assert!(
        fills >= 4 && hair >= 4,
        "TableGrid sz=4 must paint filled 0.50pt rules like Word Quartz; \
         fills={fills} hair={hair} strokes={strokes} tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        strokes == 0,
        "TableGrid must not stroke 0.50pt lines; strokes={strokes}"
    );
}

#[test]
fn explicit_none_tblborders_do_not_inherit_tablegrid() {
    // file_22 / sd_2517: the TOC table lists TableGrid/Tabelacomgrade but
    // then writes tblBorders with every edge val=none sz=0. Word treats
    // that as a real no-grid (107pp of body text, no black lattice).
    // Parsing all-none as None fell through to the style's sz=4 grid.
    let body = "<w:tbl><w:tblPr>\
           <w:tblStyle w:val=\"TableGrid\"/>\
           <w:tblBorders>\
             <w:top w:val=\"none\" w:sz=\"0\" w:space=\"0\" w:color=\"auto\"/>\
             <w:left w:val=\"none\" w:sz=\"0\" w:space=\"0\" w:color=\"auto\"/>\
             <w:bottom w:val=\"none\" w:sz=\"0\" w:space=\"0\" w:color=\"auto\"/>\
             <w:right w:val=\"none\" w:sz=\"0\" w:space=\"0\" w:color=\"auto\"/>\
             <w:insideH w:val=\"none\" w:sz=\"0\" w:space=\"0\" w:color=\"auto\"/>\
             <w:insideV w:val=\"none\" w:sz=\"0\" w:space=\"0\" w:color=\"auto\"/>\
           </w:tblBorders></w:tblPr>\
           <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
           <w:tr><w:tc><w:p><w:r><w:t>R0</w:t></w:r></w:p></w:tc></w:tr>\
           <w:tr><w:tc><w:p><w:r><w:t>R1</w:t></w:r></w:p></w:tc></w:tr>\
           <w:tr><w:tc><w:p><w:r><w:t>R2</w:t></w:r></w:p></w:tc></w:tr>\
         </w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, &table_grid_line240_styles()))
        .expect("convert none-override table");
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(
        ys.is_empty(),
        "explicit tblBorders val=none must not inherit TableGrid; ys={ys:?}"
    );
}

#[test]
fn cell_tcborders_sz0_suppresses_table_grid() {
    // file_34 / uipriority: tblBorders is a sz=4 auto grid, but every
    // cell then lists tcBorders sz=0. Word paints no lattice (the
    // highlight/wavy demo table is fill+text only).
    let body = "<w:tbl><w:tblPr><w:tblBorders>\
           <w:top w:val=\"single\" w:sz=\"4\" w:color=\"auto\"/>\
           <w:left w:val=\"single\" w:sz=\"4\" w:color=\"auto\"/>\
           <w:bottom w:val=\"single\" w:sz=\"4\" w:color=\"auto\"/>\
           <w:right w:val=\"single\" w:sz=\"4\" w:color=\"auto\"/>\
           <w:insideH w:val=\"single\" w:sz=\"4\" w:color=\"auto\"/>\
           <w:insideV w:val=\"single\" w:sz=\"4\" w:color=\"auto\"/>\
         </w:tblBorders></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"3000\"/><w:gridCol w:w=\"3000\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:tcPr><w:tcBorders>\
             <w:top w:val=\"single\" w:sz=\"0\" w:color=\"000000\"/>\
             <w:left w:val=\"single\" w:sz=\"0\" w:color=\"000000\"/>\
             <w:bottom w:val=\"single\" w:sz=\"0\" w:color=\"000000\"/>\
             <w:right w:val=\"single\" w:sz=\"0\" w:color=\"000000\"/>\
           </w:tcBorders></w:tcPr>\
             <w:p><w:r><w:t>A</w:t></w:r></w:p></w:tc>\
           <w:tc><w:tcPr><w:tcBorders>\
             <w:top w:val=\"single\" w:sz=\"0\" w:color=\"CCCCCC\"/>\
             <w:left w:val=\"single\" w:sz=\"0\" w:color=\"CCCCCC\"/>\
             <w:bottom w:val=\"single\" w:sz=\"0\" w:color=\"CCCCCC\"/>\
             <w:right w:val=\"single\" w:sz=\"0\" w:color=\"CCCCCC\"/>\
           </w:tcBorders></w:tcPr>\
             <w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>\
         </w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert sz0 cells");
    let ys = pdf_horiz_rule_ys(&pdf);
    assert!(
        ys.is_empty(),
        "tcBorders sz=0 must suppress the table grid; ys={ys:?}"
    );
}

#[test]
fn cell_tcborders_override_table_black_with_gray() {
    // CiceroDo / file_19: tblBorders are black sz=2, but every cell
    // restates CCCCCC sz=8. Word Quartz paints the gray 1pt lattice.
    let body = "<w:tbl><w:tblPr><w:tblBorders>\
           <w:top w:val=\"single\" w:sz=\"2\" w:color=\"000000\"/>\
           <w:start w:val=\"single\" w:sz=\"2\" w:color=\"000000\"/>\
           <w:bottom w:val=\"single\" w:sz=\"2\" w:color=\"000000\"/>\
           <w:end w:val=\"single\" w:sz=\"2\" w:color=\"000000\"/>\
           <w:insideH w:val=\"single\" w:sz=\"2\" w:color=\"000000\"/>\
           <w:insideV w:val=\"single\" w:sz=\"2\" w:color=\"000000\"/>\
         </w:tblBorders></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"3000\"/><w:gridCol w:w=\"3000\"/></w:tblGrid>\
         <w:tr>\
           <w:tc><w:tcPr><w:tcBorders>\
             <w:top w:val=\"single\" w:sz=\"8\" w:color=\"CCCCCC\"/>\
             <w:start w:val=\"single\" w:sz=\"8\" w:color=\"CCCCCC\"/>\
             <w:bottom w:val=\"single\" w:sz=\"8\" w:color=\"CCCCCC\"/>\
             <w:end w:val=\"single\" w:sz=\"8\" w:color=\"CCCCCC\"/>\
           </w:tcBorders></w:tcPr>\
             <w:p><w:r><w:t>G</w:t></w:r></w:p></w:tc>\
           <w:tc><w:tcPr><w:tcBorders>\
             <w:top w:val=\"single\" w:sz=\"8\" w:color=\"CCCCCC\"/>\
             <w:left w:val=\"single\" w:sz=\"8\" w:color=\"CCCCCC\"/>\
             <w:bottom w:val=\"single\" w:sz=\"8\" w:color=\"CCCCCC\"/>\
             <w:right w:val=\"single\" w:sz=\"8\" w:color=\"CCCCCC\"/>\
           </w:tcBorders></w:tcPr>\
             <w:p><w:r><w:t>H</w:t></w:r></w:p></w:tc>\
         </w:tr></w:tbl><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert gray cell borders");
    let gray = pdf_fill_rects(&pdf, 0.800, 0.800, 0.800);
    let gray_hair: Vec<_> = gray
        .iter()
        .filter(|(w, h)| (*h > 0.0 && *h < 1.6 && *w > 40.0) || (*w > 0.0 && *w < 1.6 && *h > 8.0))
        .collect();
    assert!(
        !gray_hair.is_empty(),
        "cell tcBorders CCCCCC sz=8 must paint gray hairlines; gray={gray:?}"
    );
    let black_hair = pdf_fill_rects(&pdf, 0.0, 0.0, 0.0)
        .into_iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 40.0)
        .count();
    assert_eq!(
        black_hair, 0,
        "gray cell borders must replace the black tblBorders lattice, not paint both"
    );
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

fn demo_docdefaults_styles() -> &'static str {
    // Same factory stub as the high-gap *_demo_id_paraid_overflow fixtures:
    // Calibri 11 / after=200 / line=276 and an empty default Normal.
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
     <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
       <w:docDefaults>\
         <w:rPrDefault><w:rPr>\
           <w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\"/>\
           <w:sz w:val=\"22\"/>\
         </w:rPr></w:rPrDefault>\
         <w:pPrDefault><w:pPr>\
           <w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>\
         </w:pPr></w:pPrDefault>\
       </w:docDefaults>\
       <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
         <w:name w:val=\"Normal\"/>\
       </w:style>\
     </w:styles>"
}

fn distinct_tf_ys(pdf: &[u8], tf: &str) -> Vec<f32> {
    let mut out: Vec<f32> = Vec::new();
    for y in pdf_tf_ys(pdf, tf) {
        if out.last().is_none_or(|prev| (*prev - y).abs() > 0.4) {
            out.push(y);
        }
    }
    out
}

#[test]
fn calibri_11pt_first_baseline_uses_win_ascent() {
    // Official no_comments Word oracles place Calibri with usWinAscent
    // 1950/2048 (green_bold / file_71 first baseline 82.56 from the top).
    let body = "<w:p><w:r><w:t>WinAscent</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, demo_docdefaults_styles()))
        .expect("convert calibri baseline");
    let ys = distinct_tf_ys(&pdf, "11.04 Tf");
    assert!(
        !ys.is_empty(),
        "expected 11pt text, tail {}",
        String::from_utf8_lossy(&pdf)
            .split_at(pdf.len().saturating_sub(200))
            .1
    );
    let y = ys[0];
    // 792 - 72 - 1950/2048*11 ≈ 709.53
    assert!(
        (708.8..=710.2).contains(&y),
        "Word win-ascent baseline is ~709.5pt, got {y} ys={ys:?}"
    );
}

#[test]
fn missing_heading4_does_not_keep_docdefaults_after() {
    // heading_4_style_demo: pStyle Heading4 is absent from styles.xml. Word
    // applies latent after=0 and keeps the next para's before=280. Official
    // Word oracle gap is 30pt (83.28 → 113.28 from the top).
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Heading4\"/><w:spacing w:line=\"276\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:b/><w:sz w:val=\"24\"/></w:rPr><w:t>Head Four</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading4\"/>\
           <w:spacing w:before=\"280\" w:after=\"80\" w:line=\"240\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:b/><w:sz w:val=\"24\"/></w:rPr><w:t>Next Head</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, demo_docdefaults_styles()))
        .expect("convert missing Heading4");
    let ys = distinct_tf_ys(&pdf, "12.00 Tf");
    assert!(ys.len() >= 2, "expected two 12pt lines, ys={ys:?}");
    let delta = ys[0] - ys[1];
    assert!(
        (27.0..=32.0).contains(&delta),
        "latent Heading4 after=0 + before=14pt is ~30pt, got {delta} ys={ys:?}"
    );
}

#[test]
fn missing_heading3_same_style_gap_matches_word() {
    // heading_3_center / heading_3_style Word oracles: 85.20 → 119.76 (Δ 34.6).
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Heading3\"/><w:spacing w:line=\"276\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:b/><w:sz w:val=\"28\"/></w:rPr><w:t>Head Three</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Heading3\"/>\
           <w:spacing w:before=\"320\" w:after=\"80\" w:line=\"240\"/></w:pPr>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
             <w:b/><w:sz w:val=\"28\"/></w:rPr><w:t>Next Head</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, demo_docdefaults_styles()))
        .expect("convert missing Heading3");
    let ys = distinct_tf_ys(&pdf, "14.00 Tf");
    assert!(ys.len() >= 2, "expected two 14pt lines, ys={ys:?}");
    let delta = ys[0] - ys[1];
    assert!(
        (32.0..=36.0).contains(&delta),
        "Heading3 Word gap is ~34.6pt, got {delta} ys={ys:?}"
    );
}

#[test]
fn docdefaults_minor_hansi_aptos_embeds_liberation_sans() {
    // comments / I_am_sharing / numwords: rPrDefault is asciiTheme=minorHAnsi
    // with no ascii name. Theme minor latin is Aptos. Skipping that slot
    // left the body on Carlito while soffice embeds Liberation Sans.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:rPrDefault><w:rPr>\
             <w:rFonts w:asciiTheme=\"minorHAnsi\" w:hAnsiTheme=\"minorHAnsi\"/>\
             <w:sz w:val=\"22\"/>\
           </w:rPr></w:rPrDefault></w:docDefaults>\
         </w:styles>";
    let theme = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
           <a:themeElements><a:fontScheme name=\"Office\">\
             <a:majorFont><a:latin typeface=\"Aptos Display\"/></a:majorFont>\
             <a:minorFont><a:latin typeface=\"Aptos\"/></a:minorFont>\
           </a:fontScheme></a:themeElements>\
         </a:theme>";
    let body = "<w:p><w:r><w:t>AptosBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles_and_theme(body, styles, theme))
        .expect("convert Aptos minor theme");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Aptos") || text.contains("/Calibri") || text.contains("/Carlito"),
        "minorHAnsi Aptos must embed Aptos/Calibri/Carlito; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        !text.contains("/LiberationSans 11.04 Tf"),
        "Aptos theme body must not paint Liberation Sans"
    );
}

#[test]
fn factory_cambria_minor_stays_calibri_after_mini_90() {
    // Word Quartz paints factory minorHAnsi → Cambria. Mini 90 applied
    // that slot and was ITT-wrong on file_2 / file_41 (−2.5) vs
    // table_bookmark / file_134 (+2). Keep the Aptos-only gate.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:docDefaults><w:rPrDefault><w:rPr>\
             <w:rFonts w:asciiTheme=\"minorHAnsi\" w:hAnsiTheme=\"minorHAnsi\"/>\
             <w:sz w:val=\"22\"/>\
           </w:rPr></w:rPrDefault></w:docDefaults>\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
           </w:style>\
         </w:styles>";
    let theme = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\">\
           <a:themeElements><a:fontScheme name=\"Office\">\
             <a:majorFont><a:latin typeface=\"Calibri\"/></a:majorFont>\
             <a:minorFont><a:latin typeface=\"Cambria\"/></a:minorFont>\
           </a:fontScheme></a:themeElements>\
         </a:theme>";
    let body = "<w:p><w:r><w:t>ThemeMinorBody</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles_and_theme(body, styles, theme))
        .expect("convert Cambria minor theme");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("/Cambria"),
        "factory Cambria minor stays Calibri after mini 90 ITT; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        text.contains("/Calibri") || text.contains("/Carlito"),
        "factory theme body stays Calibri/Carlito; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
}

fn first_line_max_x(pdf: &[u8]) -> f32 {
    let hay = String::from_utf8_lossy(pdf);
    let mut first_y = f32::NEG_INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    // 11pt Calibri is Word-device 46ppem: `q 0.24 0 0 0.24 x y cm BT /F 46 Tf`.
    let mut from = 0;
    while let Some(rel) = hay[from..].find(" 0.24 0 0 0.24 ") {
        let rest = &hay[from + rel + 15..];
        let parts: Vec<&str> = rest.split_whitespace().take(3).collect();
        if parts.len() >= 3
            && parts[2].starts_with("cm")
            && let (Ok(x), Ok(y)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>())
        {
            if y > first_y + 0.4 {
                first_y = y;
                max_x = x;
            } else if (y - first_y).abs() <= 0.4 {
                max_x = max_x.max(x);
            }
        }
        from += rel + 15;
    }
    if max_x.is_finite() {
        return max_x;
    }
    from = 0;
    while let Some(rel) = hay[from..].find("11.04 Tf") {
        let slice = &hay[from + rel..from + rel + 80.min(hay.len() - from - rel)];
        if let Some(td) = slice.find(" Td") {
            let before = &slice[..td];
            let mut parts = before.rsplit([' ', '\n']);
            let y = parts.next().and_then(|s| s.parse::<f32>().ok());
            let x = parts.next().and_then(|s| s.parse::<f32>().ok());
            if let (Some(x), Some(y)) = (x, y) {
                if y > first_y + 0.4 {
                    first_y = y;
                    max_x = x;
                } else if (y - first_y).abs() <= 0.4 {
                    max_x = max_x.max(x);
                }
            }
        }
        from += rel + 8;
    }
    max_x
}

#[test]
fn jc_both_spreads_first_wrapped_line_vs_left() {
    // Strict01 / sd_2517 use w:jc=both. Word justifies every line except
    // the last. Treating "both" as left leaves a ragged right edge.
    let words = "alpha bravo charlie delta echo foxtrot golf hotel india \
         juliet kilo lima mike november oscar papa quebec";
    let wrap = |jc: &str| {
        let body = format!(
            "<w:p><w:pPr><w:jc w:val=\"{jc}\"/></w:pPr>\
               <w:r><w:t>{words}</w:t></w:r></w:p>\
             <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
               <w:pgMar w:top=\"1440\" w:right=\"2880\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
        );
        docx_to_pdf(&minimal_docx_body(&body)).expect("convert")
    };
    let left = first_line_max_x(&wrap("left"));
    let both = first_line_max_x(&wrap("both"));
    assert!(
        both > left + 8.0,
        "justified first line must sit right of left-aligned, both={both} left={left}"
    );
}

#[test]
fn justified_wrapped_line_reaches_the_right_margin() {
    // Word Quartz justify is leftover / inter-word gaps (TJ ≈ -55 at
    // 11.04pt → +0.61pt per space). Counting the trailing wrap space in
    // line_w and as a gap left official justify_alignment ~81 (color_sim 0);
    // Word's last ink on that line sits at ~536.5 with the right margin at 540.
    let words = "This document demonstrates justified text alignment which \
         spreads text evenly across the full width of the line, creating \
         clean left and right edges that are perfect for formal documents \
         and publications.";
    let body = format!(
        "<w:p><w:pPr><w:jc w:val=\"both\"/></w:pPr>\
           <w:r><w:t>{words}</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&docx_with_styles(&body, demo_docdefaults_styles())).expect("convert");
    let last_x = first_line_max_x(&pdf);
    assert!(
        (538.5..=541.5).contains(&last_x),
        "justified wrapped line must reach the 540pt right margin; last_x={last_x}"
    );
}

#[test]
fn eight_pt_run_uses_eight_pt_ascent_not_eleven() {
    // small_font_size_demo: sz=16 (8pt). Line metrics used fold(11.0, max),
    // so 8pt text sat on the 11pt baseline (709.5 vs Word 712.3) and the
    // 11pt line box opened a 25pt gap vs Word's 21pt.
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"16\"/></w:rPr>\
           <w:t>Small Eight</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, demo_docdefaults_styles())).expect("convert 8pt");
    let ys = distinct_tf_ys(&pdf, "8.00 Tf");
    assert!(!ys.is_empty(), "expected 8pt text, ys={ys:?}");
    let y = ys[0];
    // 792 - 72 - 1950/2048*8 ≈ 712.38 (official Word small_font 79.68 from top)
    assert!(
        (711.8..=713.0).contains(&y),
        "8pt win-ascent baseline is ~712.4; got {y} ys={ys:?}"
    );
}

fn word_tf(pt: f32) -> String {
    // Word Save-as-PDF: integer ppem at 300 dpi, 72/300 = 0.24 user units.
    format!("{:.2} Tf", (pt * 25.0 / 6.0).round() * 0.24)
}

#[test]
fn sixteen_pt_uses_word_300dpi_device_size() {
    // font_size_18_demo Word oracle paints 16.08pt (67×0.24), not 16.00.
    // Unrounded 16pt left that stem at ~78 vs office2pdf 97.
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"32\"/></w:rPr>\
           <w:t>Sixteen</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, demo_docdefaults_styles())).expect("convert");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains(&word_tf(16.0)) || text.contains(" 67 Tf"),
        "16pt must snap to Word 300dpi 16.08; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("16.00 Tf"),
        "unrounded 16.00 Tf misses the official Word oracles"
    );
}

#[test]
fn paragraph_paints_a_default_size_end_mark() {
    // Official Word oracles put an 11pt space after every paragraph
    // (font_size_18: 16.08pt run + 11.04pt space on the same baseline).
    // Missing that mark is why 16pt Carlito/Calibri demos sat at ~78 vs
    // office2pdf 97 even after Y and the face matched.
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"32\"/></w:rPr>\
           <w:t>Sixteen</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, demo_docdefaults_styles())).expect("convert");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("16.08 Tf") || text.contains(" 67 Tf"),
        "16pt run must paint at Word 16.08 / 67ppem; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
    assert!(
        text.contains("11.04 Tf") || text.contains(" 46 Tf"),
        "Word end-of-para mark is default 11.04 / 46ppem; tail {}",
        &text[text.len().saturating_sub(200)..]
    );
}

#[test]
fn calibri_ascii_pdf_embeds_winansi_truetype_like_word() {
    // Official Word Quartz oracles (font_size_12 / italic_text / …) embed
    // Calibri as WinAnsi TrueType. Identity-H CID is rasterized unhinted
    // by MuPDF, so CIEDE2000 on the ink-union exceeds 20 and color_sim
    // becomes 0 — a 15-point ITT wipe on the 80–89 Calibri 1-page pack.
    let body = "<w:p><w:r><w:t>Font Size 12 Demo</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Encoding /WinAnsiEncoding"),
        "Word oracles use WinAnsi Calibri; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        text.contains("/Subtype /TrueType"),
        "Word oracles use simple TrueType, not CID Type0; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        !text.contains("/Encoding /Identity-H"),
        "Identity-H CID on an ASCII Calibri page is unhinted in MuPDF; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn calibri_11_uses_word_quartz_46ppem_device() {
    // file_151 / 80–89 Calibri pack: Word Quartz paints 11pt as 46ppem
    // inside a 0.24 (300dpi) cm so MuPDF hints like the oracle. User-space
    // 11.04 Tf is hinted at 22ppem (144dpi) and ink-union XOR is ΔE≈12.
    let body = "<w:p><w:r><w:t>Project Proposal</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.24 0 0 0.24") && text.contains(" 46 Tf"),
        "Word Quartz hints 11pt Calibri at 46ppem then scales 0.24; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn calibri_te_uses_hmtx_advance_like_word() {
    // Word Quartz places Calibri "Te" on hmtx (T=998/2048×11.04≈5.38pt).
    // rustybuzz GPOS/kern shrinks T by ~1pt; the 80–89 pack's color_sim
    // then goes to 0 because ink-union XOR is black-vs-white.
    let body = "<w:p><w:r><w:t>Te</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(xs.len() >= 2, "expected T and e x positions, xs={xs:?}");
    let dt = xs[1] - xs[0];
    assert!(
        (5.20..=5.50).contains(&dt),
        "T advance must be Calibri hmtx ~5.38pt, got {dt} xs={xs:?}"
    );
}

#[test]
fn eleven_pt_calibri_uses_word_quartz_device_track() {
    // Official Word Quartz oracles write Tc≈-0.0015 at 11.04pt, so T→e is
    // 5.36 not raw hmtx 5.38. Skipping that 0.017pt/glyph leaves the
    // 80–89 Calibri 1-pagers ~1pt wide and color_sim=0.
    let body = "<w:p><w:r><w:t>Te</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(xs.len() >= 2, "expected T and e, xs={xs:?}");
    let dt = xs[1] - xs[0];
    assert!(
        (5.345..=5.375).contains(&dt),
        "Word 11.04 Tc-0.0015 makes T→e≈5.36, got {dt} xs={xs:?}"
    );
}

#[test]
fn sixteen_pt_calibri_uses_word_quartz_device_track() {
    // font_size_24/18 Word oracles: 16.08pt (67ppem) with Tc≈-0.0018, so
    // F→o is 7.35 not linear 7.39. That residual is the 16pt color_sim wipe.
    let body = "<w:p><w:r><w:rPr><w:sz w:val=\"32\"/></w:rPr>\
           <w:t>Fo</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"\
             w:header=\"720\" w:footer=\"720\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, demo_docdefaults_styles())).expect("convert");
    let xs = pdf_tf_xs(&pdf, "16.08 Tf");
    assert!(xs.len() >= 2, "expected F and o, xs={xs:?}");
    let dt = xs[1] - xs[0];
    assert!(
        (7.330..=7.370).contains(&dt),
        "Word 16.08 Tc-0.0018 makes F→o≈7.35, got {dt} xs={xs:?}"
    );
}

#[test]
fn theme_color_accent1_paints_office_blue() {
    // Word stores w:color w:themeColor="accent1" with val=auto. Ignoring
    // the theme slot left headings/hyperlinks black vs Word's 4F81BD.
    let body = "<w:p><w:r><w:rPr><w:color w:val=\"auto\" w:themeColor=\"accent1\"/></w:rPr>\
           <w:t>Accent</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert themeColor");
    let text = String::from_utf8_lossy(&pdf);
    // 4F81BD → 79/129/189
    assert!(
        text.contains("0.310 0.506 0.741 rg"),
        "accent1 must paint Office blue 4F81BD; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

fn letter_line_paras(n: usize) -> String {
    (0..n)
        .map(|i| format!("<w:p><w:r><w:t>Line{i}</w:t></w:r></w:p>"))
        .collect()
}

#[test]
fn page_break_after_a_full_page_leaves_a_blank() {
    // sd_2517 ch1/13: Word leaves 1-4 / 13-9 empty because the previous
    // page is full and the next child is an empty `w:br type=page`. We
    // swallowed that skip (106pp). Emitting every blank break para was
    // 111pp — only a *full* previous page must skip.
    let mut n = 1;
    while n < 80 {
        let pdf = docx_to_pdf(&minimal_docx_body(&format!(
            "{}<w:sectPr/>",
            letter_line_paras(n)
        )))
        .expect("probe fill");
        if pdf_page_count(&pdf) >= 2 {
            break;
        }
        n += 1;
    }
    assert!(
        (20..80).contains(&n),
        "could not fill one letter page with short paras, n={n}"
    );
    let full = n - 1;
    // Last line must still fit; its after-spacing overflows the floor
    // (sd_2517 TextHeading after on a full page). A 0-height page-break
    // para then starts on the next page and breaks again.
    let body = format!(
        "{}<w:p><w:pPr><w:spacing w:after=\"480\"/></w:pPr>\
           <w:r><w:rPr><w:sz w:val=\"16\"/></w:rPr><w:t>Tail</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:r><w:t>AfterBreak</w:t></w:r></w:p><w:sectPr/>",
        letter_line_paras(full)
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert full+break");
    assert_eq!(
        pdf_page_count(&pdf),
        3,
        "full page + empty page break must skip a page (content, blank, AfterBreak); full={full}"
    );
}

#[test]
fn empty_page_break_after_table_does_not_skip_a_blank() {
    // file_78 / file_196: leftover in (0,22) after a table or cover
    // drawing must not invent a blank page (−6/−10 ITT).
    let rows = (0..8)
        .map(|i| {
            format!(
                "<w:tr><w:tc><w:p><w:r><w:t>R{i} xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx</w:t></w:r></w:p></w:tc></w:tr>"
            )
        })
        .collect::<String>();
    let body = format!(
        "<w:tbl><w:tblGrid><w:gridCol w:w=\"9000\"/></w:tblGrid>{rows}</w:tbl>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:r><w:t>AfterTableBreak</w:t></w:r></w:p><w:sectPr/>"
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert table+break");
    assert_eq!(
        pdf_page_count(&pdf),
        2,
        "table then empty page break is content + AfterTableBreak, no invented blank"
    );
}

#[test]
fn official_file_78_stays_three_pages() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_78.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_78"))
        .expect("convert official file_78");
    assert_eq!(
        pdf_page_count(&pdf),
        3,
        "Word file_78 is 3pp; leftover skip after its table must not add a blank"
    );
}

#[test]
fn official_file_196_stays_thirteen_pages() {
    // leftover in (0,22) after a cover/table extra-skipped this redline
    // stem (−10 ITT). Word is 13pp; do not invent a blank.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_196.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_196"))
        .expect("convert official file_196");
    assert_eq!(
        pdf_page_count(&pdf),
        13,
        "Word file_196 is 13pp; leftover empty w:br must not add a blank"
    );
}

#[test]
fn official_file_22_is_107_pages() {
    // Same TextHeading leftover skips as sd_2517 (Word 107, PAGEREF 1-8).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_22.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_22"))
        .expect("convert official file_22");
    let n = pdf_page_count(&pdf);
    assert_eq!(
        n, 107,
        "Word file_22 is 107pp; leftover TextHeading empty w:br must skip 1-4 without extra blanks; got {n}"
    );
}

#[test]
fn official_file_22_toc_lorem_105_is_page_one_eight() {
    // Word Quartz Times 12 / line=240 body is ~13.92pt. Typo 12.71 packed
    // chapter 1 so TOC 1.05 painted 1-5. Word is 1-8.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_22.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_22"))
        .expect("convert official file_22");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 2, "expected TOC p2, got {}", pages.len());
    let toc = pdf_winansi_text(pages[1].as_bytes());
    assert!(
        toc.contains("1.05") && toc.contains("1-8"),
        "Word TOC lorem 1.05 is 1-8, not 1-5; toc={toc}"
    );
}

#[test]
fn page_break_before_starts_a_new_page() {
    let body = "<w:p><w:r><w:t>One</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pageBreakBefore/></w:pPr>\
           <w:r><w:t>Two</w:t></w:r></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert pbb");
    assert_eq!(
        pdf_page_count(&pdf),
        2,
        "w:pageBreakBefore must start a new page"
    );
}

#[test]
fn rpr_position_raises_baseline() {
    let body = "<w:p>\
         <w:r><w:t>X</w:t></w:r>\
         <w:r><w:rPr><w:position w:val=\"12\"/></w:rPr><w:t>Y</w:t></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert position");
    let ys = pdf_tf_ys(&pdf, "11.04 Tf");
    let unique: std::collections::BTreeSet<i32> =
        ys.iter().map(|y| (*y * 10.0).round() as i32).collect();
    assert!(
        unique.len() >= 2,
        "w:position 12 (6pt) must raise Y above X, ys={ys:?}"
    );
}

#[test]
fn double_underline_paints_two_rules() {
    let body = "<w:p><w:r><w:rPr><w:u w:val=\"double\"/></w:rPr>\
           <w:t>Hi</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert double u");
    let hair: Vec<_> = pdf_fill_rects(&pdf, 0.0, 0.0, 0.0)
        .into_iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 4.0)
        .collect();
    assert!(
        hair.len() >= 2,
        "double underline must fill two hairlines, hair={hair:?}"
    );
}

#[test]
fn caps_run_is_uppercase() {
    let body = "<w:p><w:r><w:rPr><w:caps/></w:rPr><w:t>ab</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert caps");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(xs.len() >= 2, "caps must still paint two glyphs, xs={xs:?}");
    // Uppercase A/B are wider than a/b; a lower-only run is narrower.
    let lower = pdf_tf_xs(
        &docx_to_pdf(&minimal_docx_body(
            "<w:p><w:r><w:t>ab</w:t></w:r></w:p><w:sectPr/>",
        ))
        .expect("lower"),
        "11.04 Tf",
    );
    assert!(
        (xs[1] - xs[0]) > (lower[1] - lower[0]),
        "caps AB must be wider than ab, caps={} lower={}",
        xs[1] - xs[0],
        lower[1] - lower[0]
    );
}

#[test]
fn rpr_w_scale_widens_advances() {
    let normal = "<w:p><w:r><w:t>AB</w:t></w:r></w:p><w:sectPr/>";
    let wide = "<w:p><w:r><w:rPr><w:w w:val=\"200\"/></w:rPr>\
           <w:t>AB</w:t></w:r></w:p><w:sectPr/>";
    let a = pdf_tf_xs(
        &docx_to_pdf(&minimal_docx_body(normal)).expect("normal"),
        "11.04 Tf",
    );
    let b = pdf_tf_xs(
        &docx_to_pdf(&minimal_docx_body(wide)).expect("wide"),
        "11.04 Tf",
    );
    assert!(a.len() >= 2 && b.len() >= 2, "xs {a:?} {b:?}");
    assert!(
        (b[1] - b[0]) > (a[1] - a[0]) * 1.5,
        "w:w 200 must roughly double the A→B advance, n={} w={}",
        a[1] - a[0],
        b[1] - b[0]
    );
}

#[test]
fn rpr_spacing_tracks_glyphs_apart() {
    // Title / Heading char styles carry w:spacing on rPr (twips).
    let tight = "<w:p><w:r><w:t>AB</w:t></w:r></w:p><w:sectPr/>";
    let loose = "<w:p><w:r><w:rPr><w:spacing w:val=\"200\"/></w:rPr>\
           <w:t>AB</w:t></w:r></w:p><w:sectPr/>";
    let a = pdf_tf_xs(
        &docx_to_pdf(&minimal_docx_body(tight)).expect("tight"),
        "11.04 Tf",
    );
    let b = pdf_tf_xs(
        &docx_to_pdf(&minimal_docx_body(loose)).expect("loose"),
        "11.04 Tf",
    );
    assert!(a.len() >= 2 && b.len() >= 2, "xs tight={a:?} loose={b:?}");
    let d_tight = a[1] - a[0];
    let d_loose = b[1] - b[0];
    assert!(
        d_loose > d_tight + 8.0,
        "200 twips tracking is +10pt, tight={d_tight} loose={d_loose}"
    );
}

#[test]
fn explicit_left_tab_uses_ppr_position() {
    // ECMA-376: w:tab pos is from the left margin. 2880 twips = 144pt,
    // plus 1440-twip margin → B at 216pt. Page-edge 144pt is what made
    // sd_2517's 2520-twip Sumrio left tab sit behind "lorem 1.01".
    let body = "<w:p><w:pPr><w:tabs><w:tab w:val=\"left\" w:pos=\"2880\"/></w:tabs></w:pPr>\
         <w:r><w:t>A</w:t></w:r><w:r><w:tab/></w:r><w:r><w:t>B</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert explicit tab");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(
        xs.iter().any(|&x| (214.0..=218.0).contains(&x)),
        "B after a 144pt-from-margin left tab must sit at 216, got xs={xs:?}"
    );
}

#[test]
fn default_tab_advances_half_inch() {
    // Word's factory tab stop is 720 twips (0.5in). We collapsed w:tab to a
    // single space, so "A\\tB" sat at ~80pt instead of 108pt.
    let body = "<w:p><w:r><w:t>A</w:t></w:r><w:r><w:tab/></w:r>\
         <w:r><w:t>B</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert tab");
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    let max_x = xs.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        max_x >= 107.0,
        "B after a default tab must sit at 108pt, got max_x={max_x} xs={xs:?}"
    );
}

#[test]
fn toc_webhidden_right_tab_paints_pageref_and_dot_leader() {
    // sd_2517 Sumrio1/2: right tab at 8640 twips with leader=dot. Word
    // print/PDF shows the webHidden PAGEREF tab + "5-3"; we dropped both
    // so TOC lines had no dots and no page numbers.
    let body = "<w:p><w:pPr>\
           <w:tabs>\
             <w:tab w:val=\"left\" w:pos=\"2520\"/>\
             <w:tab w:val=\"right\" w:leader=\"dot\" w:pos=\"8640\"/>\
           </w:tabs>\
           <w:ind w:left=\"2520\" w:right=\"720\" w:hanging=\"2520\"/>\
         </w:pPr>\
         <w:r><w:t>lorem 1.01</w:t></w:r>\
         <w:r><w:tab/></w:r>\
         <w:r><w:t>consectetur ipsum</w:t></w:r>\
         <w:r><w:rPr><w:webHidden/></w:rPr><w:tab/></w:r>\
         <w:r><w:rPr><w:webHidden/></w:rPr><w:t>5-3</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert TOC tab");
    let text = String::from_utf8_lossy(&pdf);
    // paint_run emits one WinAnsi literal per glyph, so "5-3" is (5)(-)(3).
    assert!(
        text.contains("(5)") && text.contains("(-)") && text.contains("(3)"),
        "webHidden PAGEREF result must paint in print/PDF; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        text.matches("(.)").count() >= 6,
        "right tab leader=dot must paint leader dots; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    let xs = pdf_tf_xs(&pdf, "11.04 Tf");
    assert!(
        xs.iter().any(|&x| (488.0..510.0).contains(&x)),
        "page number must right-align to margin+432pt (504), got xs={xs:?}"
    );
}

#[test]
fn toc_dot_leader_stops_a_space_before_the_page_number() {
    // sd_2517 / file_22 Sumrio 1-3: Word paints `...... 1-3` (a space
    // before the PAGEREF). paint_tab_leader filled to dest-0.35em so
    // the last dot sat on the number (`.....1-3`).
    let body = "<w:p><w:pPr>\
           <w:tabs>\
             <w:tab w:val=\"left\" w:pos=\"2520\"/>\
             <w:tab w:val=\"right\" w:leader=\"dot\" w:pos=\"8640\"/>\
           </w:tabs>\
           <w:ind w:left=\"2520\" w:right=\"720\" w:hanging=\"2520\"/>\
         </w:pPr>\
         <w:r><w:t>lorem 1.03</w:t></w:r>\
         <w:r><w:tab/></w:r>\
         <w:r><w:t>sed adipiscing do consectetur ipsum</w:t></w:r>\
         <w:r><w:tab/></w:r>\
         <w:r><w:t>1-3</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert TOC space");
    let hay = String::from_utf8_lossy(&pdf);
    let dots = pdf_cm_tj_xy(&hay, ".");
    let ones = pdf_cm_tj_xy(&hay, "1");
    assert!(!dots.is_empty() && !ones.is_empty(), "dots+1 must paint");
    let (nx, ny) = ones
        .iter()
        .copied()
        .filter(|(x, _)| *x > 400.0)
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .expect("right-tab page number");
    let last_dot = dots
        .iter()
        .copied()
        .filter(|(_, y)| (y - ny).abs() < 1.0)
        .map(|(x, _)| x)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        last_dot.is_finite(),
        "leader dots on the PAGEREF line; dots={dots:?} num=({nx},{ny})"
    );
    let gap = nx - last_dot;
    assert!(
        gap >= 5.5,
        "Word leaves ~space+dot (~6pt left-edge) before 1-3, not ~1.35em jam; gap={gap} last_dot={last_dot} num_x={nx}"
    );
}

#[test]
fn official_sd_2517_toc_one_three_not_jammed_into_dots() {
    // Word p2 `sed adipiscing… ...... 1-3`; ours jammed `.....1-3`.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/sd_2517_localized_heading_styles.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official sd_2517")).expect("convert sd_2517");
    assert_eq!(pdf_page_count(&pdf), 107, "Word sd_2517 is 107pp");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 2, "TOC is page 2");
    let hay = &pages[1];
    // 12pt TOC uses user-space Td, not the 11pt 0.24 cm device track.
    let num = pdf_tj_xy(hay, "1-3");
    let dots = pdf_tj_xy(hay, ".");
    let (nx, ny) = num
        .iter()
        .copied()
        .find(|(x, _)| *x > 400.0)
        .expect("TOC 1-3 PAGEREF");
    let last_dot = dots
        .iter()
        .copied()
        .filter(|(_, y)| (y - ny).abs() < 1.5)
        .map(|(x, _)| x)
        .fold(f32::NEG_INFINITY, f32::max);
    let gap = nx - last_dot;
    assert!(
        gap >= 5.5,
        "official 1-3 must not sit on the last leader dot; gap={gap} last_dot={last_dot} num=({nx},{ny})"
    );
}

#[test]
fn toc_right_tab_keeps_pageref_on_last_line() {
    // Word sd_2517 Sumrio wraps the title and puts the right-tab
    // PAGEREF on the last line (`incididunt........ 4-1`). Pinning it
    // to the first line reserved page-number width up front, wrapped
    // early, and left p3 at 9.2 vs Word 11-1.
    let title = "consectetur adipiscing elit sed do eiusmod tempor incididunt \
         labore dolore magna aliqua ipsum";
    let body = format!(
        "<w:p><w:pPr>\
           <w:tabs>\
             <w:tab w:val=\"left\" w:pos=\"2520\"/>\
             <w:tab w:val=\"right\" w:leader=\"dot\" w:pos=\"8640\"/>\
           </w:tabs>\
           <w:ind w:left=\"2520\" w:right=\"720\" w:hanging=\"2520\"/>\
         </w:pPr>\
         <w:r><w:t>lorem 1.01</w:t></w:r>\
         <w:r><w:tab/></w:r>\
         <w:r><w:t>{title}</w:t></w:r>\
         <w:r><w:rPr><w:webHidden/></w:rPr><w:tab/></w:r>\
         <w:r><w:rPr><w:webHidden/></w:rPr><w:t>5-3</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert TOC wrap");
    let ys = pdf_tf_ys(&pdf, "11.04 Tf");
    assert!(ys.len() >= 8, "long TOC title must wrap; ys={ys:?}");
    let min_y = ys.iter().copied().fold(f32::INFINITY, f32::min);
    let hay = String::from_utf8_lossy(&pdf);
    let five_y = pdf_literal_y(&hay, "11.04 Tf", "(5)");
    assert!(
        five_y.is_some_and(|y| (y - min_y).abs() < 0.6),
        "PAGEREF 5-3 must sit on the last wrapped TOC line; five_y={five_y:?} min_y={min_y} ys={ys:?}"
    );
}

#[test]
fn toc_wrap_column_honors_right_indent() {
    // file_22 / sd_2517 Sumrio2: left=2520 hanging=2520 right=720,
    // right tab 8640, TOC section pgMar 1800. first_w used the tab
    // edge (~412pt) and packed Word's wrapped 3.03 onto one line.
    let desc = "sed lorem lorem ipsum magna elit dolore lorem adipiscing elit lorem";
    let body = format!(
        "<w:p><w:pPr>\
           <w:tabs>\
             <w:tab w:val=\"left\" w:pos=\"2520\"/>\
             <w:tab w:val=\"right\" w:leader=\"dot\" w:pos=\"8640\"/>\
           </w:tabs>\
           <w:ind w:left=\"2520\" w:right=\"720\" w:hanging=\"2520\"/>\
         </w:pPr>\
         <w:r><w:rPr><w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\"/>\
           <w:sz w:val=\"24\"/></w:rPr><w:t>lorem 3.03</w:t></w:r>\
         <w:r><w:tab/></w:r>\
         <w:r><w:rPr><w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\"/>\
           <w:sz w:val=\"24\"/></w:rPr><w:t>{desc}</w:t></w:r>\
         <w:r><w:tab/></w:r>\
         <w:r><w:t>3-2</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1800\" w:bottom=\"1440\" w:left=\"1800\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert TOC right-ind wrap");
    let ys = pdf_tf_ys(&pdf, "12.00 Tf");
    let unique: std::collections::BTreeSet<i32> =
        ys.iter().map(|y| (*y * 2.0).round() as i32).collect();
    assert!(
        unique.len() >= 2,
        "Sumrio2 w:right=720 must wrap this Times 12 line like Word 3.03; ys={ys:?}"
    );
}

#[test]
fn toc_sumrio1_first_line_honors_right_indent() {
    // sd_2517 / file_22 Sumrio1 (Arial Bold 12, hanging 2520, right=720,
    // right tab 8640, TOC pgMar 1800, no leading label tab). first_w used
    // the tab edge (~412pt) so "dolor'et" packed onto line 1 (ends x≈487).
    // Word wraps it (line 1 x2≈439). Cap first_w by content minus right
    // indent (budget 396pt, ends x≈486).
    let desc = "aliqua aliqua incididunt elit sit ut adipiscing incididunt dolore \
         dolor'et labore adipiscing";
    let body = format!(
        "<w:p><w:pPr>\
           <w:tabs>\
             <w:tab w:val=\"left\" w:pos=\"2520\"/>\
             <w:tab w:val=\"right\" w:leader=\"dot\" w:pos=\"8640\"/>\
           </w:tabs>\
           <w:ind w:left=\"2520\" w:right=\"720\" w:hanging=\"2520\"/>\
         </w:pPr>\
         <w:r><w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
           <w:b/><w:sz w:val=\"24\"/></w:rPr><w:t>{desc}</w:t></w:r>\
         <w:r><w:tab/></w:r>\
         <w:r><w:t>6-1</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1800\" w:bottom=\"1440\" w:left=\"1800\"/></w:sectPr>"
    );
    let pdf = docx_to_pdf(&minimal_docx_body(&body)).expect("convert Sumrio1 wrap");
    let mut by_y: std::collections::BTreeMap<i32, Vec<(f32, String)>> =
        std::collections::BTreeMap::new();
    for (x, y, ch) in pdf_tf_glyphs(&pdf, "12.00 Tf") {
        by_y.entry((y * 2.0).round() as i32)
            .or_default()
            .push((x, ch));
    }
    let hanging = by_y
        .values()
        .find_map(|row| {
            let min_x = row.iter().map(|(x, _)| *x).fold(f32::INFINITY, f32::min);
            if !(210.0..230.0).contains(&min_x) {
                return None;
            }
            let mut row = row.clone();
            row.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            Some(row.into_iter().map(|(_, ch)| ch).collect::<String>())
        })
        .unwrap_or_default();
    assert!(
        hanging.starts_with("dolor"),
        "Word hanging line starts with dolor'et, not labore; hanging={hanging:?}"
    );
}

fn sumrio2_styles() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/></w:style>\
           <w:style w:type=\"paragraph\" w:styleId=\"Sumrio2\">\
             <w:name w:val=\"toc 2\"/>\
             <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/>\
               <w:ind w:left=\"2520\" w:right=\"720\" w:hanging=\"2520\"/></w:pPr>\
             <w:rPr><w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\"/>\
               <w:sz w:val=\"24\"/></w:rPr></w:style>\
         </w:styles>"
        .into()
}

#[test]
fn sumrio_auto_line_follows_times_hhea_not_typo() {
    // Sumrio2 is TNR 12 / line=240 auto. Word Quartz sd_2517 TOC
    // baselines are 13.68–13.92pt (Times hhea 1825+443+87 = 13.80).
    // Typo 1420+442+307 = 12.71 packs 35 lorem lines onto p3 and
    // ends at 11-8; Word packs 31 and ends at 11-1. Do not use
    // size×1.0 (the previous test) and do not change body Times —
    // that overshoots 107pp.
    let body = "<w:p><w:pPr><w:pStyle w:val=\"Sumrio2\"/></w:pPr>\
           <w:r><w:t>lorem 1.01 title one</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"Sumrio2\"/></w:pPr>\
           <w:r><w:t>lorem 1.02 title two</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf = docx_to_pdf(&docx_with_styles(body, &sumrio2_styles())).expect("convert Sumrio2");
    let mut ys = pdf_tf_ys(&pdf, "12.00 Tf");
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.4);
    assert!(ys.len() >= 2, "two Sumrio2 lines must paint; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (13.4..14.3).contains(&gap),
        "Sumrio2 line=240 must be Times hhea ~13.8 (Word 13.92), not typo 12.71; gap={gap} ys={ys:?}"
    );
}

fn times_normal_styles() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
             <w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/></w:pPr>\
             <w:rPr><w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\"/>\
               <w:sz w:val=\"24\"/></w:rPr></w:style>\
         </w:styles>"
        .into()
}

#[test]
fn times_body_auto_line_stays_typo_so_sd_2517_is_107() {
    // Word Quartz Times 12 / line=240 is ~13.8. Applying that to
    // Normal still blows official file_22 107→116 after leftover skip
    // / live PAGEREF. Body Times stays typo 12.71; TOC size×1.15.
    let body = "<w:p><w:r><w:t>alpha body line</w:t></w:r></w:p>\
         <w:p><w:r><w:t>bravo body line</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf =
        docx_to_pdf(&docx_with_styles(body, &times_normal_styles())).expect("convert Times body");
    let mut ys = pdf_tf_ys(&pdf, "12.00 Tf");
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.4);
    assert!(ys.len() >= 2, "two Times body lines must paint; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (12.3..13.2).contains(&gap),
        "Times body must stay typo ~12.71 (13.8 is 116pp); gap={gap} ys={ys:?}"
    );
}

fn arial_normal_styles() -> String {
    "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/>\
             <w:pPr><w:spacing w:after=\"0\" w:line=\"276\" w:lineRule=\"auto\"/></w:pPr>\
             <w:rPr><w:rFonts w:ascii=\"Arial\" w:hAnsi=\"Arial\"/>\
               <w:sz w:val=\"24\"/></w:rPr></w:style>\
         </w:styles>"
        .into()
}

#[test]
fn arial_12_auto_line_is_size_times_line_mult() {
    // Word Quartz Arial 12 / line=276 is size×1.15 ~13.8 (file_34 body
    // dy 13.7–13.9, 2pp). em-box×1.15 (~15.1) was the 3pp extra. Mini 86
    // forbade paint_size×1.15 (glyph 12→13.8), not the line box.
    let body = "<w:p><w:r><w:t>alpha arial line</w:t></w:r></w:p>\
         <w:p><w:r><w:t>bravo arial line</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\"/></w:sectPr>";
    let pdf =
        docx_to_pdf(&docx_with_styles(body, &arial_normal_styles())).expect("convert Arial 12");
    let mut ys = pdf_tf_ys(&pdf, "12.00 Tf");
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.4);
    assert!(ys.len() >= 2, "two Arial 12 lines must paint; ys={ys:?}");
    let gap = ys[0] - ys[1];
    assert!(
        (13.5..=14.2).contains(&gap),
        "Arial 12 auto-276 line box is size×1.15 ~13.8 like Word file_34; gap={gap} ys={ys:?}"
    );
}

#[test]
fn toc_hanging_first_line_uses_width_to_the_right_tab() {
    // sd_2517 TOC: left=2520 hanging=2520, right tab 8640, pgMar 1800.
    // First line starts in the hanging gutter. Capping wrap at
    // content-indent (270pt) broke "eiusmod tempor" onto line 2;
    // Word keeps that Sumrio1 heading on one line through the tab.
    let title = "aliqua ipsum sed lorem incididunt ipsum eiusmod tempor";
    let body = format!(
        "<w:p><w:pPr><w:pStyle w:val=\"Sumrio2\"/>\
           <w:tabs>\
             <w:tab w:val=\"left\" w:pos=\"2520\"/>\
             <w:tab w:val=\"right\" w:leader=\"dot\" w:pos=\"8640\"/>\
           </w:tabs>\
           <w:ind w:left=\"2520\" w:right=\"720\" w:hanging=\"2520\"/></w:pPr>\
         <w:r><w:t>{title}</w:t></w:r>\
         <w:r><w:tab/></w:r>\
         <w:r><w:t>3-1</w:t></w:r></w:p>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/>\
           <w:pgMar w:top=\"1440\" w:right=\"1800\" w:bottom=\"1440\" w:left=\"1800\"/></w:sectPr>"
    );
    let pdf =
        docx_to_pdf(&docx_with_styles(&body, &sumrio2_styles())).expect("convert hanging TOC");
    let mut ys = pdf_tf_ys(&pdf, "12.00 Tf");
    ys.sort_by(|a, b| b.partial_cmp(a).unwrap());
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.4);
    assert_eq!(
        ys.len(),
        1,
        "Word keeps this Sumrio heading on one line; ys={ys:?}"
    );
}

#[test]
fn vanish_run_is_not_painted() {
    let body = "<w:p><w:r><w:t>See</w:t></w:r>\
         <w:r><w:rPr><w:vanish/></w:rPr><w:t>HIDDEN</w:t></w:r>\
         <w:r><w:t>Me</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert vanish");
    let n = pdf_tf_xs(&pdf, "11.04 Tf").len();
    // "See" + "Me" is 5 glyphs. Including "HIDDEN" would be 11.
    assert!(
        n <= 6,
        "vanished HIDDEN must not paint, n={n} xs={:?}",
        pdf_tf_xs(&pdf, "11.04 Tf")
    );
}

#[test]
fn rpr_shd_fill_paints_a_fill_behind_text() {
    let body = "<w:p><w:r><w:rPr><w:shd w:val=\"clear\" w:fill=\"FFFF00\"/></w:rPr>\
           <w:t>Hi</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert rPr shd");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("1.000 1.000 0.000 rg") && text.contains(" re f"),
        "w:shd fill FFFF00 must fill a rect; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn highlight_yellow_paints_a_fill_behind_text() {
    let body = "<w:p><w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr>\
           <w:t>Hi</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert highlight");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("1.000 1.000 0.000 rg") && text.contains(" re f"),
        "w:highlight yellow must fill a rect; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

fn yellow_highlight_bands(pdf: &[u8]) -> Vec<(f32, f32, f32, f32)> {
    pdf_fill_boxes_in(&String::from_utf8_lossy(pdf), 1.0, 1.0, 0.0)
        .into_iter()
        .filter(|(_, _, w, h)| *h > 4.0 && *w > *h)
        .collect()
}

#[test]
fn adjacent_same_color_highlights_stay_split_after_mini_hlmerge() {
    // Word potpourri yellow is ONE band [282, 281, 245.8, 14.64].
    // Merging abutting same-color run highlights (mini 245–248) dropped
    // no-redline potpourri −0.0012 and file_170 −0.0007; redline 0-delta.
    // Keep per-run fills (106.8+79.4+60.5).
    let body = "<w:p>\
           <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr>\
             <w:t xml:space=\"preserve\">yellow-highlighted, </w:t></w:r>\
           <w:r><w:rPr><w:highlight w:val=\"yellow\"/><w:strike/></w:rPr>\
             <w:t xml:space=\"preserve\">strikethrough, </w:t></w:r>\
           <w:r><w:rPr><w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\"/>\
             <w:highlight w:val=\"yellow\"/></w:rPr>\
             <w:t>monospace.</w:t></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert adjacent yellow");
    let bands = yellow_highlight_bands(&pdf);
    assert_eq!(
        bands.len(),
        3,
        "mini hlmerge ITT-neg; keep per-run yellow; bands={bands:?}"
    );
    assert!(
        bands.iter().all(|b| b.2 < 150.0),
        "no merged ~246pt yellow after mini hlmerge; bands={bands:?}"
    );
}

#[test]
fn nonadjacent_highlights_stay_split() {
    let body = "<w:p>\
           <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr>\
             <w:t>Hello</w:t></w:r>\
           <w:r><w:t xml:space=\"preserve\"> gap </w:t></w:r>\
           <w:r><w:rPr><w:highlight w:val=\"yellow\"/></w:rPr>\
             <w:t>World</w:t></w:r>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert split yellow");
    let bands = yellow_highlight_bands(&pdf);
    assert_eq!(
        bands.len(),
        2,
        "nonadjacent yellows stay split; bands={bands:?}"
    );
}

#[test]
fn official_potpourri_yellow_stays_three_bands_after_mini_hlmerge() {
    // Word n=1 yellow per decorated page (w≈246). Merging (mini 245–248)
    // dropped potpourri 44.6245→44.6233. Keep three per-run fills; 5pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/potpourritest.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official potpourri"))
        .expect("convert official potpourri");
    assert_eq!(pdf_page_count(&pdf), 5, "Word potpourri is 5pp");
    let pages = pdf_content_streams(&pdf);
    let mut decorated = 0usize;
    for (i, page) in pages.iter().enumerate() {
        let bands: Vec<_> = pdf_fill_boxes_in(page, 1.0, 1.0, 0.0)
            .into_iter()
            .filter(|(_, _, w, h)| *h > 4.0 && *w > 40.0)
            .collect();
        if bands.is_empty() {
            continue;
        }
        decorated += 1;
        assert_eq!(
            bands.len(),
            3,
            "page {i} mini hlmerge ITT-neg; keep 3 yellows; bands={bands:?}"
        );
        assert!(
            bands.iter().all(|b| b.2 < 150.0),
            "page {i} must not ship a merged ~246pt yellow; bands={bands:?}"
        );
    }
    assert!(
        decorated >= 1,
        "potpourri must still paint yellow highlight"
    );
}

#[test]
fn ppr_shd_fill_paints_a_paragraph_band() {
    // Word w:pPr/w:shd fills the paragraph extents (content width), not
    // the glyph box. Strict01 "Video provides…" is fill=ED7D31.
    let body = "<w:p><w:pPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"ED7D31\"/></w:pPr>\
           <w:r><w:t>Hi</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert pPr shd");
    let hs = pdf_fill_hs(&pdf, 0.929, 0.490, 0.192);
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !hs.is_empty(),
        "pPr shd ED7D31 must paint a content-wide band; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
}

#[test]
fn ppr_rpr_shd_is_not_a_paragraph_band() {
    // file_71: pPr/rPr/shd is paragraph-mark / default-run shading, not
    // w:pPr/w:shd extents. Mini 116 treated the descendant as a
    // content-wide 92D050 band and dropped file_71 99.04→75.89.
    let body = "<w:p><w:pPr><w:rPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"92D050\"/></w:rPr></w:pPr>\
           <w:r><w:t>Hi</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert pPr/rPr shd");
    let hs = pdf_fill_hs(&pdf, 0.573, 0.816, 0.314);
    assert!(
        hs.is_empty(),
        "pPr/rPr/shd must not paint a content-wide band; hs={hs:?}"
    );
}

#[test]
fn official_file_71_has_no_green_paragraph_band() {
    // Control stem at 99. 92D050 is rPr shd (glyph box, size×1.2≈13.25).
    // A content-wide band (w>400) is ITT-wrong (mini 116: 99→76).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_71.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_71"))
        .expect("convert official file_71");
    let wide = pdf_fill_ws(&pdf, 0.573, 0.816, 0.314);
    assert!(
        wide.iter().all(|&w| w < 400.0),
        "file_71 must not grow a content-wide 92D050 band; ws={wide:?}"
    );
}

#[test]
fn ppr_shd_style_paints_yellow_paragraph_band() {
    // file_34 / uipriority Highlighted Style: pPr shd FFFF00 on the
    // named style, not on the instance pPr. Run highlight is glyph-wide.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
          <w:style w:type=\"paragraph\" w:styleId=\"HighlightedStyle\">\
            <w:name w:val=\"Highlighted Style\"/>\
            <w:pPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"FFFF00\"/>\
              <w:spacing w:before=\"120\" w:after=\"120\"/></w:pPr>\
            <w:rPr><w:b/><w:sz w:val=\"24\"/></w:rPr>\
          </w:style>\
        </w:styles>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"HighlightedStyle\"/></w:pPr>\
           <w:r><w:t>CustomHighlight</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_styles(body, styles)).expect("convert style shd");
    let hs = pdf_fill_hs(&pdf, 1.0, 1.0, 0.0);
    assert!(
        !hs.is_empty(),
        "HighlightedStyle pPr shd FFFF00 must paint a content-wide yellow band"
    );
}

#[test]
fn official_file_34_highlighted_style_paints_yellow_band() {
    // Word paints HighlightedStyle as a full-column yellow band. Run
    // highlight on "Yellow highlight" is glyph-wide (w<100) and must
    // not satisfy this. Paint-only — file_34 stays Word+1 pages.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_34.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_34"))
        .expect("convert official file_34");
    let hs = pdf_fill_hs(&pdf, 1.0, 1.0, 0.0);
    assert!(
        !hs.is_empty(),
        "file_34 HighlightedStyle must paint a yellow paragraph band; pages={}",
        pdf_page_count(&pdf)
    );
    assert!(
        pdf_page_count(&pdf) <= 3,
        "file_34 must stay Word+1; got {}",
        pdf_page_count(&pdf)
    );
}

#[test]
fn official_strict01_video_para_paints_orange_band() {
    // Direct pPr shd fill=ED7D31 on the Online Video paragraph. Word
    // paints the paragraph extents; do not change Strict01's 13pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let hs = pdf_fill_hs(&pdf, 0.929, 0.490, 0.192);
    assert!(
        !hs.is_empty(),
        "Strict01 Online Video pPr shd ED7D31 must paint an orange paragraph band"
    );
    assert_eq!(pdf_page_count(&pdf), 13, "Word Strict01 is 13pp");
}

fn comments_docx(body: &str, comments_xml: &str) -> Vec<u8> {
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
        <Override PartName=\"/word/comments.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml\"/>\
        </Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let doc_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rIdComments\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments\" \
          Target=\"comments.xml\"/>\
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
    zip.start_file("word/comments.xml", opts).unwrap();
    zip.write_all(comments_xml.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

fn comments_part(id: &str, author: &str, text: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:comments xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:comment w:id=\"{id}\" w:author=\"{author}\" w:initials=\"A\">\
             <w:p><w:r><w:t>{text}</w:t></w:r></w:p>\
           </w:comment>\
         </w:comments>"
    )
}

#[derive(Debug, Clone)]
struct PdfNote {
    page: usize,
    x: f32,
    y: f32,
    contents: String,
    author: String,
}

fn pdf_objects(pdf: &[u8]) -> Vec<String> {
    let hay = String::from_utf8_lossy(pdf);
    hay.split("endobj")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn pdf_notes(pdf: &[u8]) -> Vec<PdfNote> {
    let objs = pdf_objects(pdf);
    let mut notes = Vec::new();
    let mut annot_page: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    let mut page_n = 0usize;
    for obj in &objs {
        if obj.contains("/Type /Page") && !obj.contains("/Type /Pages") {
            page_n += 1;
            if let Some(idx) = obj.find("/Annots") {
                let rest = &obj[idx..];
                for cap in rest.split(|c: char| !c.is_ascii_digit()) {
                    if cap.is_empty() {
                        continue;
                    }
                    if let Ok(id) = cap.parse::<usize>()
                        && id > 0
                    {
                        annot_page.insert(id, page_n);
                    }
                }
            }
        }
    }
    for obj in &objs {
        if !obj.contains("/Type /Annot") || !obj.contains("/Subtype /Text") {
            continue;
        }
        let id = obj
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let page = *annot_page.get(&id).unwrap_or(&1);
        let contents = pdf_literal_after(obj, "/Contents").unwrap_or_default();
        let author = pdf_literal_after(obj, "/T ").unwrap_or_default();
        let (x, y) = pdf_rect_xy(obj).unwrap_or((0.0, 0.0));
        notes.push(PdfNote {
            page,
            x,
            y,
            contents,
            author,
        });
    }
    notes
}

fn pdf_literal_after(hay: &str, key: &str) -> Option<String> {
    let idx = hay.find(key)?;
    let rest = &hay[idx + key.len()..];
    let start = rest.find('(')?;
    let mut out = String::new();
    let bytes = &rest.as_bytes()[start + 1..];
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b')' => break,
            b'\\' if i + 1 < bytes.len() => {
                out.push(bytes[i + 1] as char);
                i += 2;
            }
            b => {
                out.push(b as char);
                i += 1;
            }
        }
    }
    Some(out)
}

fn pdf_rect_xy(hay: &str) -> Option<(f32, f32)> {
    let idx = hay.find("/Rect")?;
    let rest = &hay[idx + 5..];
    let start = rest.find('[')?;
    let end = rest.find(']')?;
    let nums: Vec<f32> = rest[start + 1..end]
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    (nums.len() >= 2).then_some((nums[0], nums[1]))
}

#[test]
fn shipped_docx_to_pdf_places_comment_on_range_page() {
    let body = "<w:p><w:r><w:t>Alpha on page one</w:t></w:r></w:p>\
         <w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>\
         <w:p><w:commentRangeStart w:id=\"7\"/>\
           <w:r><w:t>Bravo</w:t></w:r>\
           <w:commentRangeEnd w:id=\"7\"/>\
           <w:r><w:rPr><w:rStyle w:val=\"CommentReference\"/></w:rPr>\
             <w:commentReference w:id=\"7\"/></w:r>\
         </w:p><w:sectPr/>";
    let comments = comments_part("7", "Ada Lovelace", "Second page note");
    let pdf = docx_to_pdf(&comments_docx(body, &comments)).expect("convert comments");
    assert!(pdf.starts_with(b"%PDF"));
    assert!(
        pdf_page_count(&pdf) >= 2,
        "fixture is two pages, got {}",
        pdf_page_count(&pdf)
    );
    let notes = pdf_notes(&pdf);
    assert_eq!(
        notes.len(),
        1,
        "one DOCX comment must become one PDF note; notes={notes:?}"
    );
    let note = &notes[0];
    assert!(
        note.contents.contains("Second page note"),
        "comment body must be preserved; {note:?}"
    );
    assert!(
        note.author.contains("Ada Lovelace"),
        "author must be preserved; {note:?}"
    );
    assert_eq!(
        note.page, 2,
        "comment must sit on the range page, not page 1; {note:?}"
    );
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("Bravo"),
        "range text is body ink; painted={painted}"
    );
    assert!(
        !painted.contains("Second page note"),
        "comment body must not paint extra body ink vs comment-stripped oracles; painted={painted}"
    );
    // Bravo is the first (and only) text on page 2, so it sits at the
    // default body origin. Content streams are separate PDF objects, so
    // glyph page ids are not reliable — the annot's /Annots page is.
    assert!(
        note.x > 60.0 && note.x < 100.0,
        "annot x must be at Bravo's left-origin; {note:?}"
    );
    assert!(
        note.y > 680.0 && note.y < 740.0,
        "annot y must be at Bravo's first-line baseline; {note:?}"
    );
}

#[test]
fn shipped_docx_to_pdf_migrates_word_based_comments() {
    let path = "../neurotic_docx_bench/corpus/word_based/docx_source/comments.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("word_based comments.docx"))
        .expect("convert word_based comments");
    assert!(pdf.starts_with(b"%PDF"));
    assert!(pdf_page_count(&pdf) >= 1);
    let notes = pdf_notes(&pdf);
    assert!(
        notes.iter().any(|n| n.contents.contains("tachyon")),
        "comment body from comments.xml must land in a PDF note; notes={notes:?}"
    );
    assert!(
        notes
            .iter()
            .any(|n| n.author.contains("Michael Williamson")),
        "author must round-trip; notes={notes:?}"
    );
    assert!(
        notes.iter().all(|n| n.page == 1),
        "fixture is one page; notes={notes:?}"
    );
    let painted = pdf_winansi_text(&pdf);
    assert!(painted.contains("Ouch"), "range text stays body ink");
    assert!(
        !painted.contains("tachyon"),
        "comment body is annot-only, not extra raster ink; painted={painted}"
    );
}

#[test]
fn shipped_docx_to_pdf_paints_ins_underline_and_del_strike() {
    let body = "<w:p>\
         <w:del w:id=\"0\" w:author=\"Pat\"><w:r><w:delText>gone</w:delText></w:r></w:del>\
         <w:r><w:t xml:space=\"preserve\"> </w:t></w:r>\
         <w:ins w:id=\"1\" w:author=\"Pat\"><w:r><w:t>fresh</w:t></w:r></w:ins>\
         </w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert ins/del");
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("gone"),
        "delText is still painted; {painted}"
    );
    assert!(painted.contains("fresh"), "ins text is painted; {painted}");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.820 0.204 0.220 RG") || text.contains("0.820 0.204 0.220"),
        "Word del ink is #D13438 strike; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    assert!(
        text.contains("0.820 0.204 0.220 RG") || text.contains("0.820 0.204 0.220"),
        "by-author ins underline is Word #D13438; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn gradfill_first_stop_paints_lummed_accent() {
    // Strict01 cover Rectangle 466: accent1 lumMod=20000 lumOff=80000.
    let body = "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" \
          behindDoc=\"1\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"page\"><wp:align>center</wp:align></wp:positionH>\
          <wp:positionV relativeFrom=\"page\"><wp:align>center</wp:align></wp:positionV>\
          <wp:extent cx=\"7383780\" cy=\"4000000\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"Wash\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
              <wps:spPr><a:prstGeom prst=\"rect\"/>\
                <a:gradFill><a:gsLst>\
                  <a:gs pos=\"0\"><a:schemeClr val=\"accent1\">\
                    <a:lumMod val=\"20000\"/><a:lumOff val=\"80000\"/></a:schemeClr></a:gs>\
                </a:gsLst></a:gradFill>\
                <a:ln><a:noFill/></a:ln></wps:spPr>\
            </wps:wsp>\
          </a:graphicData></a:graphic>\
        </wp:anchor></w:drawing></w:r>\
        <w:r><w:t>AfterWash</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert gradFill");
    let hs = pdf_fill_hs(&pdf, 0.862, 0.901, 0.948);
    assert!(
        !hs.is_empty(),
        "gradFill first stop must paint lummed accent1; tail {}",
        {
            let t = String::from_utf8_lossy(&pdf);
            t[t.len().saturating_sub(280)..].to_string()
        }
    );
}

#[test]
fn theme_filled_wrapnone_rect_paints_accent_fill() {
    // Strict01 Rectangle 1: wrapNone, fillRef accent1, no a:noFill.
    let body = "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" \
          behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"column\"><wp:posOffset>40943</wp:posOffset></wp:positionH>\
          <wp:positionV relativeFrom=\"paragraph\"><wp:posOffset>95534</wp:posOffset></wp:positionV>\
          <wp:extent cx=\"1692323\" cy=\"1030406\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"Rectangle 1\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
              <wps:spPr><a:xfrm><a:ext cx=\"1692323\" cy=\"1030406\"/></a:xfrm>\
                <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></wps:spPr>\
              <wps:style><a:lnRef idx=\"2\"><a:schemeClr val=\"accent1\"/></a:lnRef>\
                <a:fillRef idx=\"1\"><a:schemeClr val=\"accent1\"/></a:fillRef></wps:style>\
            </wps:wsp>\
          </a:graphicData></a:graphic>\
        </wp:anchor></w:drawing></w:r>\
        <w:r><w:t>AfterFill</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert filled rect");
    let hs = pdf_fill_hs(&pdf, 0.310, 0.506, 0.741);
    assert!(
        !hs.is_empty(),
        "accent1 filled wrapNone rect must paint; tail {}",
        {
            let t = String::from_utf8_lossy(&pdf);
            t[t.len().saturating_sub(240)..].to_string()
        }
    );
}

#[test]
fn inline_nofill_rectangle_reserves_flow_without_stroke() {
    // Strict01 Rectangle 3: inline, a:noFill. Word keeps the 167pt hole
    // above the chart; stroking the box is ITT-wrong.
    let body = "<w:p><w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\">\
          <wp:extent cx=\"5104263\" cy=\"2122227\"/>\
          <wp:docPr id=\"1\" name=\"Rectangle 3\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
              <wps:spPr><a:xfrm><a:ext cx=\"5104263\" cy=\"2122227\"/></a:xfrm>\
                <a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom>\
                <a:noFill/><a:ln><a:noFill/></a:ln></wps:spPr>\
            </wps:wsp>\
          </a:graphicData></a:graphic>\
        </wp:inline></w:drawing></w:r></w:p>\
        <w:p><w:r><w:t>AfterHole</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert nofill inline");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        !text.contains("0.60 w"),
        "nofill inline must not stroke; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("401.91") && !text.contains("167.10"),
        "must not paint the 402x167 box; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
    let painted = pdf_winansi_text(&pdf);
    assert!(
        painted.contains("AfterHole"),
        "body after spacer; {painted}"
    );
    assert_eq!(pdf_page_count(&pdf), 1);
    let hay = String::from_utf8_lossy(&pdf);
    let ys = pdf_cm_ys(&hay);
    let y = ys.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(
        y < 620.0,
        "Word keeps the 167pt inline hole; AfterHole must sit below it, ys={ys:?}"
    );
}

fn pdf_cm_ys(hay: &str) -> Vec<f32> {
    let mut ys = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(" 0.24 0 0 0.24 ") {
        let rest = &hay[from + rel + 15..];
        let parts: Vec<&str> = rest.split_whitespace().take(3).collect();
        if parts.len() >= 3
            && parts[2].starts_with("cm")
            && let Ok(y) = parts[1].parse::<f32>()
        {
            ys.push(y);
        }
        from += rel + 15;
    }
    ys
}

#[test]
fn size_rel_page_percent_overrides_extent() {
    // Strict01 cover Rectangle 466: tiny leftover extent, wp14:sizeRel 80%×50%
    // of the page. Using extent paints a 8pt speck; Word uses the page %.
    let body = "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"1\" \
          behindDoc=\"1\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"page\"><wp:align>center</wp:align></wp:positionH>\
          <wp:positionV relativeFrom=\"page\"><wp:align>center</wp:align></wp:positionV>\
          <wp:extent cx=\"100000\" cy=\"100000\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"Wash\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
              <wps:spPr><a:prstGeom prst=\"rect\"/>\
                <a:solidFill><a:srgbClr val=\"4F81BD\"/></a:solidFill>\
                <a:ln><a:noFill/></a:ln></wps:spPr>\
            </wps:wsp>\
          </a:graphicData></a:graphic>\
          <wp14:sizeRelH xmlns:wp14=\"http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing\" \
            relativeFrom=\"page\"><wp14:pctWidth>80%</wp14:pctWidth></wp14:sizeRelH>\
          <wp14:sizeRelV xmlns:wp14=\"http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing\" \
            relativeFrom=\"page\"><wp14:pctHeight>50%</wp14:pctHeight></wp14:sizeRelV>\
        </wp:anchor></w:drawing></w:r>\
        <w:r><w:t>AfterSizeRel</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert sizeRel");
    let ws = pdf_fill_ws(&pdf, 0.310, 0.506, 0.741);
    let hs = pdf_fill_hs(&pdf, 0.310, 0.506, 0.741);
    assert!(
        ws.iter().any(|w| *w > 400.0),
        "80% of letter 612pt is ~490; got ws={ws:?} hs={hs:?}"
    );
    assert!(
        hs.iter().any(|h| *h > 300.0),
        "50% of letter 792pt is ~396; got ws={ws:?} hs={hs:?}"
    );
}

#[test]
fn higher_relative_height_paints_after_lower_fill() {
    // Strict01 cover: white Rectangle 468 (z=251653632) must paint BEFORE
    // dark Rectangle 467 (z=251656704) so the abstract header stays visible.
    let body = "<w:p><w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"10\" \
          behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"page\"><wp:posOffset>0</wp:posOffset></wp:positionH>\
          <wp:positionV relativeFrom=\"page\"><wp:posOffset>0</wp:posOffset></wp:positionV>\
          <wp:extent cx=\"1270000\" cy=\"1270000\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"1\" name=\"DarkOnTop\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
              <wps:spPr><a:prstGeom prst=\"rect\"/>\
                <a:solidFill><a:srgbClr val=\"1F497D\"/></a:solidFill>\
                <a:ln><a:noFill/></a:ln></wps:spPr>\
            </wps:wsp>\
          </a:graphicData></a:graphic>\
        </wp:anchor></w:drawing></w:r>\
        <w:r><w:drawing><wp:anchor simplePos=\"0\" relativeHeight=\"5\" \
          behindDoc=\"0\" locked=\"0\" layoutInCell=\"1\" allowOverlap=\"1\">\
          <wp:positionH relativeFrom=\"page\"><wp:posOffset>0</wp:posOffset></wp:positionH>\
          <wp:positionV relativeFrom=\"page\"><wp:posOffset>0</wp:posOffset></wp:positionV>\
          <wp:extent cx=\"1270000\" cy=\"1270000\"/>\
          <wp:wrapNone/>\
          <wp:docPr id=\"2\" name=\"WhiteUnder\"/>\
          <a:graphic><a:graphicData \
            uri=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
            <wps:wsp xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\">\
              <wps:spPr><a:prstGeom prst=\"rect\"/>\
                <a:solidFill><a:srgbClr val=\"FFFFFF\"/></a:solidFill>\
                <a:ln><a:noFill/></a:ln></wps:spPr>\
            </wps:wsp>\
          </a:graphicData></a:graphic>\
        </wp:anchor></w:drawing></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert z-order");
    let text = String::from_utf8_lossy(&pdf);
    let white = text.find("1.000 1.000 1.000 rg");
    let dark = text.find("0.122 0.286 0.490 rg");
    assert!(
        white.is_some() && dark.is_some(),
        "both fills must paint; {text}"
    );
    assert!(
        white.unwrap() < dark.unwrap(),
        "lower relativeHeight (white) must paint under higher (dark); white={white:?} dark={dark:?}"
    );
}

#[test]
fn official_strict01_cover_wash_uses_page_size_rel() {
    // Word p5 landscape wash is wp14:sizeRel 95%×95% of 792×612 (~752×581).
    // Extent-only is the leftover 581×752 portrait box.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let rects = pdf_fill_rects(&pdf, 0.862, 0.901, 0.948);
    let max_w = rects.iter().map(|(w, _)| *w).fold(0.0_f32, f32::max);
    let max_h = rects.iter().map(|(_, h)| *h).fold(0.0_f32, f32::max);
    assert!(
        max_w > 700.0,
        "cover wash must be ~95% of landscape width 792; rects={rects:?}"
    );
    assert!(
        max_h > 500.0 && max_h < 650.0,
        "cover wash must be ~95% of landscape height 612, not portrait extent 752; rects={rects:?}"
    );
    assert_eq!(pdf_page_count(&pdf), 13, "Word Strict01 is 13pp");
}

#[test]
fn official_strict01_clipart_is_on_word_page_three() {
    // Word p2 is body text; p3 is the WMF clipart + "Two". Extra p1 flow
    // (no Rectangle 3 hole) parked the picture on p2 and left p3 empty.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    assert_eq!(pdf_page_count(&pdf), 13, "Word Strict01 is 13pp");
    let pages = pdf_content_streams(&pdf);
    assert!(pages.len() >= 3, "need page streams; n={}", pages.len());
    let p2_img = pages[1].contains("/Im") && pages[1].contains(" Do");
    let p3_img = pages[2].contains("/Im") && pages[2].contains(" Do");
    assert!(
        p3_img,
        "Word p3 paints the WMF clipart (/Im Do); p3 tail {}",
        &pages[2][pages[2].len().saturating_sub(240)..]
    );
    assert!(
        !p2_img,
        "Word p2 is text-only; clipart must not stay on page 2"
    );
}

#[test]
fn numbering_start_indent_nests_ilvl_past_listparagraph() {
    // Strict01: numbering uses w:ind w:start="18pt"/"36pt" (not w:left).
    // ListParagraph also carries w:ind w:start="720". Numbering must win
    // so ilvl 1 sits 18pt to the right of ilvl 0.
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:style w:type=\"paragraph\" w:styleId=\"ListParagraph\">\
             <w:basedOn w:val=\"Normal\"/>\
             <w:pPr><w:ind w:start=\"720\"/><w:contextualSpacing/></w:pPr>\
           </w:style>\
         </w:styles>";
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
           <w:abstractNum w:abstractNumId=\"0\">\
             <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
               <w:lvlText w:val=\"%1)\"/>\
               <w:pPr><w:ind w:start=\"18pt\" w:hanging=\"18pt\"/></w:pPr></w:lvl>\
             <w:lvl w:ilvl=\"1\"><w:start w:val=\"1\"/><w:numFmt w:val=\"lowerLetter\"/>\
               <w:lvlText w:val=\"%2)\"/>\
               <w:pPr><w:ind w:start=\"36pt\" w:hanging=\"18pt\"/></w:pPr></w:lvl>\
           </w:abstractNum>\
           <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
         </w:numbering>";
    let body = "<w:p><w:pPr><w:pStyle w:val=\"ListParagraph\"/>\
         <w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>ParentItem</w:t></w:r></w:p>\
         <w:p><w:pPr><w:pStyle w:val=\"ListParagraph\"/>\
         <w:numPr><w:ilvl w:val=\"1\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>\
         <w:r><w:t>ChildItem</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&numbering_docx_with_styles(
        body,
        Some(numbering),
        Some(styles),
    ))
    .expect("convert nested start-indent");
    let lines = pdf_line_xs_grouped(&pdf);
    assert!(lines.len() >= 2, "parent + child lines; lines={lines:?}");
    let parent = lines[0][0];
    let child = lines[1][0];
    assert!(
        child > parent + 12.0,
        "ilvl 1 start=36pt must sit right of ilvl 0 start=18pt; parent={parent} child={child} lines={lines:?}"
    );
}

#[test]
fn official_strict01_nested_list_indents_ilvl() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "need page 1");
    let lines = pdf_line_xs_in(&pages[0]);
    let mins: Vec<f32> = lines
        .iter()
        .filter(|xs| !xs.is_empty())
        .map(|xs| xs[0])
        .collect();
    assert!(
        mins.windows(2).any(|w| (w[1] - w[0]).abs() > 12.0),
        "Strict01 page 1 must nest ilvl 1 (~18pt) under ilvl 0; mins={mins:?}"
    );
}

#[test]
fn official_strict01_chart_paints_bottom_legend() {
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("Series 1") || text.contains("(Series 1)"),
        "Word chart legend is Series 1/2/3 under the plot; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn official_strict01_page1_accent_fill_stays_above_the_chart() {
    // wrapNone Rectangle 1 / Right Arrow sit in the 167pt hole above the
    // chart. Without the hole they paint on top of Chart Title.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "need page 1 stream");
    let p1 = &pages[0];
    let fills = pdf_fill_boxes_in(p1, 0.310, 0.506, 0.741);
    let bars = pdf_fill_boxes_in(p1, 0.930, 0.490, 0.190);
    let wrap_y = fills
        .iter()
        .filter(|(_, _, w, h)| *w > 80.0 && *h > 40.0)
        .map(|(_, y, _, _)| *y)
        .fold(f32::NEG_INFINITY, f32::max);
    let chart_top = bars
        .iter()
        .map(|(_, y, _, h)| *y + *h)
        .fold(f32::NEG_INFINITY, f32::max);
    assert!(
        wrap_y.is_finite(),
        "page 1 must still paint the wrapNone accent fill; fills={fills:?}"
    );
    assert!(
        chart_top.is_finite(),
        "page 1 must paint chart bars; bars={bars:?}"
    );
    assert!(
        wrap_y > chart_top + 8.0,
        "wrapNone fill y={wrap_y} must sit above chart bar top {chart_top}; fills={fills:?} bars={bars:?}"
    );
}

#[test]
fn official_strict01_right_arrow_is_a_chevron() {
    // Word rightArrow is a filled chevron (pointed head). Two FillRects
    // (shaft + square head) paint a T and wipe page-1 edge_iou.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "need page 1");
    assert!(
        pdf_has_filled_polygon(&pages[0]),
        "rightArrow must fill a polygon (h f), not only re rects; tail {}",
        &pages[0][pages[0].len().saturating_sub(400)..]
    );
}

#[test]
fn official_strict01_curved_connector_is_a_cubic() {
    // Word curvedConnector3 (flipV) is two cubics, not a 3-segment polyline.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "need page 1");
    assert!(
        pdf_has_cubic(&pages[0]),
        "curvedConnector3 must stroke cubics (c); tail {}",
        &pages[0][pages[0].len().saturating_sub(400)..]
    );
}

#[test]
fn official_strict01_bent_connector_has_a_triangle_head() {
    // Word bentConnector3 tailEnd=triangle is a second filled polygon on page 1
    // (the first is the rightArrow chevron).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "need page 1");
    let n = pages[0].matches(" h f").count();
    assert!(
        n >= 2,
        "need rightArrow chevron + tailEnd triangle (h f count={n}); tail {}",
        &pages[0][pages[0].len().saturating_sub(400)..]
    );
}

#[test]
fn official_file_34_paints_wavy_underline() {
    // Word `w:u val="wave"` is a sine-like stroke. A straight Line under
    // "wavy underline" wipes file_34 edge_iou on the formatting line.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_34.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_34"))
        .expect("convert official file_34");
    let pages = pdf_content_streams(&pdf);
    assert!(!pages.is_empty(), "need page 1");
    assert!(
        pdf_has_wavy_stroke(&pages[0]),
        "wave underline must stroke short diagonal segments, not only y1=y2; tail {}",
        &pages[0][pages[0].len().saturating_sub(400)..]
    );
}

#[test]
fn official_file_34_single_underlines_are_filled_not_stroked() {
    // Word Quartz p1 single `w:u` is filled 0.48pt hairlines. Mini 0.6pt
    // `l S` (~68 strokes) fought that ink-union. Wave on the same page
    // stays stroked (official_file_34_paints_wavy_underline).
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_34.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official file_34"))
        .expect("convert official file_34");
    let hair: Vec<_> = pdf_fill_rects(&pdf, 0.0, 0.0, 0.0)
        .into_iter()
        .filter(|(w, h)| *h > 0.0 && *h < 1.6 && *w > 40.0)
        .collect();
    assert!(
        hair.len() >= 4,
        "file_34 single underlines must fill hairlines like Word; hair={hair:?}"
    );
    let pages = pdf_content_streams(&pdf);
    assert!(
        pdf_has_wavy_stroke(&pages[0]),
        "wave underline must still stroke"
    );
}

#[test]
fn georgia_run_embeds_georgia_not_times() {
    // file_34 / uipriority: Word Quartz embeds Georgia. convert folds
    // Georgia into the Times serif slot (LiberationSerif / Times).
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Georgia\" w:hAnsi=\"Georgia\"/>\
             <w:sz w:val=\"24\"/></w:rPr>\
           <w:t>GeorgiaSample</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Georgia");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Georgia"),
        "Georgia run must embed Georgia, not Times; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        !text.contains("/TimesNewRoman") && !text.contains("/LiberationSerif"),
        "Georgia must not fall through to Times; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn consolas_run_embeds_consolas_not_courier() {
    // Word Quartz embeds Consolas (Strict01 keyword-search line,
    // potpourri/file_170 Consolas-BoldItalic). convert currently folds
    // Consolas into the Courier New mono slot.
    let body = "<w:p><w:r>\
           <w:rPr><w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\"/>\
             <w:sz w:val=\"22\"/></w:rPr>\
           <w:t>ConsolasKeyword</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert Consolas");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Consolas"),
        "Consolas run must embed Consolas, not Courier; tail {}",
        &text[text.len().saturating_sub(320)..]
    );
    assert!(
        !text.contains("/CourierNew") && !text.contains("/LiberationMono"),
        "Consolas must not fall through to Courier; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

#[test]
fn official_strict01_diagram_paints_accent1_roundrects() {
    // Word SmartArt on page 13 is three accent1-filled roundRects
    // (Item 1/2/3). convert currently dumps the labels with no fills.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let pages = pdf_content_streams(&pdf);
    assert_eq!(pages.len(), 13, "Word Strict01 is 13pp");
    let last = pages.last().expect("page 13");
    let n = last.matches("0.310 0.506 0.741 rg").count();
    assert!(
        n >= 3,
        "SmartArt must fill 3 accent1 (4F81BD) roundRects; n={n} tail {}",
        &last[last.len().saturating_sub(400)..]
    );
}

#[test]
fn official_strict01_diagram_roundrects_are_polygons() {
    // Word roundRect adj=16667 (r = min(w,h)/6). Sharp `re` boxes wipe
    // p13 edge_iou at the 9pt corners.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/Strict01.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official Strict01"))
        .expect("convert official Strict01");
    let pages = pdf_content_streams(&pdf);
    assert_eq!(pages.len(), 13, "Word Strict01 is 13pp");
    let last = pages.last().expect("page 13");
    assert!(
        last.contains("0.310 0.506 0.741 rg"),
        "need accent1 fills on p13; tail {}",
        &last[last.len().saturating_sub(280)..]
    );
    assert!(
        !accent1_fill_is_sharp_rect(last),
        "accent1 SmartArt must be roundRect polygons (h f), not re; tail {}",
        &last[last.len().saturating_sub(400)..]
    );
    assert!(
        pdf_has_filled_polygon(last),
        "roundRect must close a filled polygon; tail {}",
        &last[last.len().saturating_sub(280)..]
    );
}

fn accent1_fill_is_sharp_rect(hay: &str) -> bool {
    let needle = "0.310 0.506 0.741 rg ";
    let mut from = 0;
    while let Some(rel) = hay[from..].find(needle) {
        let rest = &hay[from + rel + needle.len()..];
        let until = rest.find('\n').unwrap_or(rest.len().min(80));
        if rest[..until].contains(" re f") {
            return true;
        }
        from += rel + needle.len();
    }
    false
}

fn pdf_fill_boxes_in(hay: &str, r: f32, g: f32, b: f32) -> Vec<(f32, f32, f32, f32)> {
    let needle = format!("{r:.3} {g:.3} {b:.3} rg ");
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(rel) = hay[from..].find(&needle) {
        let rest = &hay[from + rel + needle.len()..];
        let end = rest.find(" re f").unwrap_or(0);
        let parts: Vec<&str> = rest[..end].split_whitespace().collect();
        if parts.len() >= 4
            && let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
                parts[0].parse::<f32>(),
                parts[1].parse::<f32>(),
                parts[2].parse::<f32>(),
                parts[3].parse::<f32>(),
            )
        {
            out.push((x, y, w, h));
        }
        from += rel + needle.len();
    }
    out
}

#[test]
fn vml_imagedata_paints_embedded_png() {
    // Strict01 Choice Requires=v OLE preview is v:imagedata, not a:blip.
    let body = "<w:p><w:r><w:pict xmlns:v=\"urn:schemas-microsoft-com:vml\">\
          <v:shape style=\"width:72pt;height:72pt\">\
            <v:imagedata r:id=\"rIdImg\"/>\
          </v:shape></w:pict></w:r>\
          <w:r><w:t>AfterVml</w:t></w:r></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&drawing_docx(body)).expect("convert VML imagedata");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("/Subtype /Image"),
        "v:imagedata must paint a PDF image; tail {}",
        &text[text.len().saturating_sub(280)..]
    );
}

fn docx_with_settings(body: &str, settings: &str) -> Vec<u8> {
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
        <Override PartName=\"/word/settings.xml\" \
          ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml\"/>\
        </Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rId1\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" \
          Target=\"word/document.xml\"/>\
        </Relationships>";
    let doc_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
        <Relationship Id=\"rIdSettings\" \
          Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings\" \
          Target=\"settings.xml\"/>\
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
    zip.start_file("word/settings.xml", opts).unwrap();
    zip.write_all(settings.as_bytes()).unwrap();
    zip.finish().unwrap().into_inner()
}

#[test]
fn track_revisions_ins_uses_word_first_author_red() {
    // file_27 / randomized clones: Word Save-as-PDF with markup paints
    // first-author ins as #D13438, not soffice gold.
    let settings = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:settings xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
        <w:trackRevisions/></w:settings>";
    let body = "<w:p><w:ins w:id=\"1\" w:author=\"Ada\"><w:r><w:t>HelloIns</w:t></w:r></w:ins></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_settings(body, settings)).expect("convert markup ins");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.820 0.204 0.220"),
        "trackRevisions ins must be Word red #D13438; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("0.753 0.565 0.000"),
        "must not keep soffice gold on markup PDFs"
    );
}

#[test]
fn track_revisions_does_not_add_a_page() {
    // A 2in balloon gutter wrapped file_27's 30pt title and blew 12→14pp.
    // Markup color must not change pagination vs Word.
    let settings = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:settings xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
        <w:trackRevisions/></w:settings>";
    let body = "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
         <w:ins w:id=\"1\" w:author=\"Ada\"><w:r>\
         <w:rPr><w:b/><w:sz w:val=\"60\"/></w:rPr>\
         <w:t>Microsoft Word vs. Google Docs</w:t></w:r></w:ins></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_settings(body, settings)).expect("convert markup title");
    assert_eq!(
        pdf_page_count(&pdf),
        1,
        "30pt title must stay one line / one page"
    );
}

#[test]
fn without_track_revisions_ins_uses_word_red() {
    let body = "<w:p><w:ins w:id=\"1\" w:author=\"Ada\"><w:r><w:t>FreshRed</w:t></w:r></w:ins></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&minimal_docx_body(body)).expect("convert plain ins");
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.820 0.204 0.220"),
        "no-markup ins is still Word #D13438; tail {}",
        &text[text.len().saturating_sub(240)..]
    );
    assert!(
        !text.contains("0.753 0.565 0.000"),
        "must not keep soffice gold on Word-oracle ins"
    );
}

#[test]
fn official_file_27_markup_gutter_and_red_ins() {
    // Word Save-as-PDF All Markup: letter MediaBox, content scaled into the
    // left ~415pt, 0.949 gray pasteboard on the right (~188×578). Shrinking
    // wrap via margin_r += 144 wrapped the 30pt title and blew 12→14pp.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source_randomized/file_27.docx";
    let pdf =
        docx_to_pdf(&std::fs::read(path).expect("official file_27")).expect("convert file_27");
    assert_eq!(pdf_page_count(&pdf), 12, "Word file_27 is 12pp");
    let boxes = pdf_mediaboxes(&pdf);
    assert!(
        boxes
            .first()
            .is_some_and(|&(w, h)| (w - 612.0).abs() < 1.0 && (h - 792.0).abs() < 1.0),
        "markup pane is inside letter, not a wider page; boxes={boxes:?}"
    );
    let hs = pdf_fill_hs(&pdf, 0.949, 0.949, 0.949);
    assert!(
        hs.iter().any(|h| *h > 400.0),
        "file_27 Word PDF has a markup balloon column; hs={hs:?}"
    );
    let text = String::from_utf8_lossy(&pdf);
    assert!(
        text.contains("0.820 0.204 0.220"),
        "file_27 ins title is Word red"
    );
}

#[test]
fn track_revisions_few_dels_do_not_paint_balloon_pane() {
    // file_9 / addition_removal: 54 dels, Word stays 0.24 cm / no pane.
    // file_27 is 103 dels. A single del must not shrink the page.
    let settings = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:settings xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
        <w:trackRevisions/></w:settings>";
    let body = "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr>\
         <w:ins w:id=\"1\" w:author=\"Ada\"><w:r>\
         <w:rPr><w:b/><w:sz w:val=\"60\"/></w:rPr>\
         <w:t>Microsoft Word vs. Google Docs</w:t></w:r></w:ins></w:p>\
         <w:p><w:del w:id=\"2\" w:author=\"Ada\"><w:r>\
         <w:delText>gone</w:delText></w:r></w:del></w:p><w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_settings(body, settings)).expect("convert few dels");
    assert_eq!(pdf_page_count(&pdf), 1, "30pt title stays one page");
    let hs = pdf_fill_hs(&pdf, 0.949, 0.949, 0.949);
    assert!(
        !hs.iter().any(|h| *h > 400.0),
        "Word only pasteboards All-Markup when del volume is high (file_27); hs={hs:?}"
    );
}

#[test]
fn official_comments_lots_has_no_markup_pane() {
    // comments-lots has no trackRevisions; Word stays 0.24 cm / no pasteboard.
    let path = "../neurotic_docx_bench/corpus/no_comments_pdf_was_generated_by_word/docx_source/docx_lots_of_comments.docx";
    let pdf = docx_to_pdf(&std::fs::read(path).expect("official comments-lots"))
        .expect("convert comments-lots");
    assert_eq!(pdf_page_count(&pdf), 9, "Word comments-lots is 9pp");
    let hs = pdf_fill_hs(&pdf, 0.949, 0.949, 0.949);
    assert!(
        !hs.iter().any(|h| *h > 400.0),
        "comments-lots must not grow a markup pane; hs={hs:?}"
    );
}

#[test]
fn track_revisions_ins_only_does_not_paint_balloon_pane() {
    // file_6: trackRevisions + ins, no del. Word stays 0.24 cm / no pane.
    let settings = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
        <w:settings xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
        <w:trackRevisions/></w:settings>";
    let body = "<w:p><w:ins w:id=\"1\" w:author=\"Ada\"><w:r><w:t>HelloIns</w:t></w:r></w:ins></w:p>\
         <w:sectPr/>";
    let pdf = docx_to_pdf(&docx_with_settings(body, settings)).expect("convert ins-only");
    let hs = pdf_fill_hs(&pdf, 0.949, 0.949, 0.949);
    assert!(
        !hs.iter().any(|h| *h > 400.0),
        "ins-only markup stays full-page; hs={hs:?}"
    );
}

/// Every `/Font` resource key on every page, in document order.
fn font_resource_keys(pdf: &[u8]) -> Vec<Vec<String>> {
    let text = String::from_utf8_lossy(pdf);
    let mut pages = Vec::new();
    let mut rest = text.as_ref();
    while let Some(at) = rest.find("/Font <<") {
        rest = &rest[at + "/Font <<".len()..];
        let Some(end) = rest.find(">>") else { break };
        let dict = &rest[..end];
        rest = &rest[end..];
        pages.push(
            dict.split_whitespace()
                .filter(|t| t.starts_with('/'))
                .map(|t| t.to_string())
                .collect::<Vec<_>>(),
        );
    }
    pages
}

/// CodeRabbit PR#4: `font_res` keyed each entry with `face.pdf_name()`, which
/// `sanitize_pdf_name` derives by mapping every non-alphanumeric byte to `-`.
/// Two override faces whose PostScript names differ only in punctuation
/// (`Foo_Bar` / `Foo.Bar`) collapse to one key, so the page resource dictionary
/// held a duplicate and a reader bound one of the faces to the wrong glyph
/// mapping. Names stay readable — the suite reads face selection straight off
/// the content stream — but no page dictionary may hold a duplicate key.
#[test]
fn font_resource_keys_are_unique_per_page() {
    let pdf = docx_to_pdf(&minimal_docx(&["Alpha beta gamma"], None)).expect("convert");
    let pages = font_resource_keys(&pdf);
    assert!(!pages.is_empty(), "no /Font resource dictionary emitted");
    for keys in &pages {
        let mut sorted = keys.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            keys.len(),
            "duplicate key in a page /Font dictionary: {keys:?}"
        );
    }
}

/// The page dictionary lists the faces the page paints, not every face in the
/// document: a `Tf` operator may only name a font the page resources declare.
#[test]
fn page_font_dict_covers_every_font_the_page_selects() {
    let pdf = docx_to_pdf(&minimal_docx(&["Alpha beta gamma"], None)).expect("convert");
    let text = String::from_utf8_lossy(&pdf);
    let declared: Vec<String> = font_resource_keys(&pdf).into_iter().flatten().collect();
    assert!(!declared.is_empty(), "no /Font resource dictionary emitted");
    let mut selected = 0usize;
    for chunk in text.split("BT ").skip(1) {
        let Some(tf) = chunk.split(" Tf").next() else {
            continue;
        };
        let Some(name) = tf.split_whitespace().next() else {
            continue;
        };
        if !name.starts_with('/') {
            continue;
        }
        selected += 1;
        assert!(
            declared.iter().any(|d| d == name),
            "content stream selects {name}, which no page /Font dictionary declares"
        );
    }
    assert!(selected > 0, "no Tf operator found in the content stream");
}
