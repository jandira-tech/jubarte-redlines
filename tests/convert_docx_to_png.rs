// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::convert::render`: PNG pages, page count and page text from one
//! layout pass, without a PDF round trip or an external rasterizer.

mod common;

use common::docx::{docx, para};
use jubarte::convert::{PdfOptions, RenderRequest, docx_render_report, docx_to_png, render};

fn letter_with_pages(n: usize) -> Vec<u8> {
    let mut body = String::new();
    for i in 0..n {
        body.push_str(&para(&format!("Page {} starts here.", i + 1)));
        if i + 1 < n {
            body.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
        }
    }
    docx(&body)
}

#[test]
fn png_pages_match_the_layout_page_count_and_dpi() {
    let bytes = letter_with_pages(3);
    let pngs = docx_to_png(&bytes, PdfOptions::default(), 36.0).unwrap();
    assert_eq!(pngs.len(), 3);
    for png in &pngs {
        assert_eq!(&png[1..4], b"PNG");
        // US Letter at 36 dpi: 8.5in × 11in → 306 × 396 pixels (IHDR big-endian).
        let w = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
        let h = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
        assert_eq!((w, h), (306, 396));
    }
}

#[test]
fn render_report_lists_page_count_and_page_text() {
    let bytes = letter_with_pages(2);
    let report = docx_render_report(&bytes, PdfOptions::default()).unwrap();
    assert_eq!(report.page_count, 2);
    assert_eq!(report.pages.len(), 2);
    assert!(
        report.pages[0].text.contains("Page 1 starts here."),
        "{:?}",
        report.pages[0].text
    );
    assert!(
        report.pages[1].text.contains("Page 2 starts here."),
        "{:?}",
        report.pages[1].text
    );
    assert!(!report.pages[1].text.contains("Page 1"));
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
    assert_eq!(json["page_count"], 2);
    assert_eq!(json["pages"][1]["index"], 1);
    assert!(json["fonts"].is_array());
}

#[test]
fn one_render_pass_can_produce_pdf_pngs_and_report_together() {
    let bytes = letter_with_pages(2);
    let out = render(
        &bytes,
        PdfOptions::default(),
        RenderRequest {
            pdf: true,
            png_dpi: Some(24.0),
        },
    )
    .unwrap();
    let pdf = out.pdf.expect("pdf requested");
    assert!(pdf.starts_with(b"%PDF"));
    assert_eq!(jubarte::convert::pdf_page_count(&pdf), 2);
    assert_eq!(out.pngs.len(), 2);
    assert_eq!(out.report.page_count, 2);
    let none = render(
        &bytes,
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: None,
        },
    )
    .unwrap();
    assert!(none.pdf.is_none() && none.pngs.is_empty());
    assert_eq!(none.report.page_count, 2);
}

#[test]
fn malformed_input_is_an_error_not_a_panic() {
    assert!(docx_to_png(b"nope", PdfOptions::default(), 72.0).is_err());
    assert!(docx_render_report(b"nope", PdfOptions::default()).is_err());
}
