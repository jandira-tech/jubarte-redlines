// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Drive the shipped `docx_to_pdf` entry (library + `jubarte convert` CLI).

use std::io::{Cursor, Write};
use std::process::Command;

use jubarte::convert::{docx_to_pdf, pdf_page_count};
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
