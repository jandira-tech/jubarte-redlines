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
            pages: None,
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
            pages: None,
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

#[test]
fn dpi_outside_the_supported_range_is_an_error() {
    let bytes = letter_with_pages(1);
    for dpi in [0.0, -72.0, f32::NAN, f32::INFINITY, 1e9] {
        let err = docx_to_png(&bytes, PdfOptions::default(), dpi)
            .expect_err("an unusable dpi must not yield an empty or 1-pixel page set");
        assert!(err.to_string().contains("dpi"), "{dpi}: {err}");
    }
    // The bounds themselves are accepted.
    assert_eq!(
        docx_to_png(&bytes, PdfOptions::default(), 1.0)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_page_over_the_pixel_budget_is_an_error_not_a_missing_png() {
    // Word's largest page, 22in square, at 1200 dpi is 26400² pixels: a
    // 2.8 GB buffer whose failed allocation would abort the process.
    let big = r#"<w:p><w:pPr><w:sectPr><w:pgSz w:w="31680" w:h="31680"/></w:sectPr></w:pPr><w:r><w:t>Big</w:t></w:r></w:p>"#;
    let bytes = docx(&(big.to_string() + &para("Letter page.")));
    let err = render(
        &bytes,
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: Some(1200.0),
            pages: None,
        },
    )
    .expect_err("an unpaintable page must fail the request");
    assert!(err.to_string().contains("page 1"), "{err}");
}

#[test]
fn fractional_dpi_rounds_dimensions_up_and_png_contains_painted_text() {
    let source = docx(&para("Visible ink"));
    let out = render(
        &source,
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: Some(24.5),
            pages: None,
        },
    )
    .unwrap();
    assert!(out.pdf.is_none());
    assert_eq!(out.pngs.len(), 1);
    let pixels = image::load_from_memory(&out.pngs[0]).unwrap().to_rgba8();
    assert_eq!(pixels.dimensions(), (209, 270));
    assert!(
        pixels.pixels().all(|p| p[3] == 255),
        "the page background is opaque"
    );
    assert_eq!(pixels.get_pixel(0, 0).0, [255; 4]);
    assert!(
        pixels
            .pixels()
            .any(|p| p[0] < 128 && p[1] < 128 && p[2] < 128),
        "a valid PNG must also contain the rendered text"
    );
    assert!(out.report.pages[0].text.contains("Visible ink"));
}

#[test]
fn report_is_independent_of_requested_output_formats() {
    let source = letter_with_pages(2);
    let report_only = render(
        &source,
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: None,
            pages: None,
        },
    )
    .unwrap();
    let png_only = render(
        &source,
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: Some(12.0),
            pages: None,
        },
    )
    .unwrap();
    let pdf_only = render(
        &source,
        PdfOptions::default(),
        RenderRequest {
            pdf: true,
            png_dpi: None,
            pages: None,
        },
    )
    .unwrap();
    assert_eq!(report_only.report.to_json(), png_only.report.to_json());
    assert_eq!(report_only.report.to_json(), pdf_only.report.to_json());
    assert!(report_only.pdf.is_none() && report_only.pngs.is_empty());
    assert!(png_only.pdf.is_none());
    assert_eq!(png_only.pngs.len(), 2);
    assert!(pdf_only.pngs.is_empty());
    assert_eq!(jubarte::convert::pdf_page_count(&pdf_only.pdf.unwrap()), 2);
}
