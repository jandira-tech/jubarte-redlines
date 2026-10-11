// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `RenderRequest.pages`: one layout pass for the whole document, PNGs only
//! for the pages asked for.

mod common;

use common::docx::{docx, para};
use jubarte::convert::{ConvertError, PdfOptions, RenderRequest, render};

const PAGE_BREAK: &str = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;

fn three_pages() -> Vec<u8> {
    docx(&(para("A") + PAGE_BREAK + &para("B") + PAGE_BREAK + &para("C")))
}

fn pngs_of(pages: Option<Vec<usize>>) -> Vec<Vec<u8>> {
    render(
        &three_pages(),
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: Some(40.0),
            pages,
        },
    )
    .unwrap()
    .pngs
}

#[test]
fn only_the_requested_pages_are_rasterized_but_the_report_covers_all() {
    let out = render(
        &three_pages(),
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: Some(40.0),
            pages: Some(vec![2]),
        },
    )
    .unwrap();
    assert_eq!(out.report.page_count, 3);
    assert_eq!(out.pngs.len(), 1);
    assert_eq!(out.report.pages[2].text.trim(), "C");
}

#[test]
fn selected_pages_are_the_same_bytes_as_the_full_render() {
    let all = pngs_of(None);
    let picked = pngs_of(Some(vec![2, 0, 2]));
    assert_eq!(picked.len(), 2, "sorted and deduplicated");
    assert_eq!(picked[0], all[0]);
    assert_eq!(picked[1], all[2]);
}

#[test]
fn an_empty_selection_rasterizes_nothing() {
    assert!(pngs_of(Some(Vec::new())).is_empty());
}

#[test]
fn an_out_of_range_page_is_an_error_naming_the_count() {
    let one = docx(&para("A"));
    let e = render(
        &one,
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: Some(40.0),
            pages: Some(vec![5]),
        },
    )
    .unwrap_err();
    assert!(e.to_string().contains("1 page"), "{e}");
    assert!(matches!(
        e,
        ConvertError::PageOutOfRange {
            requested: 5,
            page_count: 1
        }
    ));
}

#[test]
fn the_page_count_in_the_error_is_plural_when_it_should_be() {
    let e = render(
        &three_pages(),
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: Some(40.0),
            pages: Some(vec![3]),
        },
    )
    .unwrap_err();
    assert!(e.to_string().contains("3 pages"), "{e}");
}

#[test]
fn pages_without_png_dpi_still_validate() {
    let e = render(
        &three_pages(),
        PdfOptions::default(),
        RenderRequest {
            pdf: false,
            png_dpi: None,
            pages: Some(vec![9]),
        },
    )
    .unwrap_err();
    assert!(matches!(e, ConvertError::PageOutOfRange { .. }), "{e}");
}

fn jubarte(args: &[&std::ffi::OsStr]) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn the_cli_honours_pages_for_png_output_and_refuses_it_for_other_formats() {
    let dir = tempfile::tempdir().unwrap();
    let src = dir.path().join("in.docx");
    std::fs::write(&src, three_pages()).unwrap();
    let png = jubarte(&[
        "convert".as_ref(),
        src.as_os_str(),
        "-t".as_ref(),
        "png".as_ref(),
        "--dpi".as_ref(),
        "20".as_ref(),
        "--pages".as_ref(),
        "2".as_ref(),
    ]);
    assert_eq!(png.status.code(), Some(0), "{png:?}");
    assert!(dir.path().join("in-page-02.png").exists(), "{png:?}");
    assert!(!dir.path().join("in-page-01.png").exists());

    for to in ["md", "docx", "pdf"] {
        let out = jubarte(&[
            "convert".as_ref(),
            src.as_os_str(),
            "-t".as_ref(),
            to.as_ref(),
            "-o".as_ref(),
            dir.path().join(format!("out.{to}")).as_os_str(),
            "--pages".as_ref(),
            "1".as_ref(),
        ]);
        assert_eq!(out.status.code(), Some(2), "-t {to}: {out:?}");
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains("Usage:"));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("--pages"),
            "-t {to}: {out:?}"
        );
        assert!(!dir.path().join(format!("out.{to}")).exists(), "-t {to}");
    }
}
