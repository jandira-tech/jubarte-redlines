// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::convert::diff_render`: which pages of two documents differ,
//! pixel for pixel, from one layout pass each.

mod common;

use common::docx::{docx, docx_with_sect_pr, para};
use jubarte::convert::{DiffOptions, PdfOptions, diff_render};

const PAGE_BREAK: &str = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;

fn opts() -> DiffOptions {
    DiffOptions {
        dpi: 50.0,
        pdf: PdfOptions::default(),
        overlay: true,
    }
}

#[test]
fn identical_inputs_change_nothing() {
    let a = docx(&para("Same text."));
    let d = diff_render(&a, &a, &opts()).unwrap();
    assert_eq!(d.pages.len(), 1);
    assert_eq!(d.pages[0].changed_ratio, 0.0);
    assert_eq!(d.pages[0].bbox, None);
    assert_eq!(d.pages[0].only_in, None);
    assert!(d.overlays[0].is_none());
    assert_eq!(d.a.len(), 1);
    assert_eq!(d.b.len(), 1);
    assert_eq!(d.a_report.page_count, 1);
    assert_eq!(d.b_report.page_count, 1);
    assert!(!d.differs());
}

#[test]
fn a_changed_word_changes_a_bounded_region() {
    let a = docx(&para("The fee is ten."));
    let b = docx(&para("The fee is twenty."));
    let d = diff_render(&a, &b, &opts()).unwrap();
    let p = &d.pages[0];
    assert!(p.changed_ratio > 0.0 && p.changed_ratio < 0.05, "{p:?}");
    let [x0, y0, x1, y1] = p.bbox.unwrap();
    assert!(
        x1 > x0 && y1 > y0 && y1 < 120,
        "the change sits on the first line: {p:?}"
    );
    assert!(d.overlays[0].as_ref().unwrap().starts_with(b"\x89PNG"));
    assert!(d.differs());
}

#[test]
fn the_overlay_is_skipped_on_request() {
    let a = docx(&para("The fee is ten."));
    let b = docx(&para("The fee is twenty."));
    let d = diff_render(
        &a,
        &b,
        &DiffOptions {
            overlay: false,
            ..opts()
        },
    )
    .unwrap();
    assert!(d.pages[0].changed_ratio > 0.0);
    assert_eq!(d.overlays, vec![None]);
}

#[test]
fn a_page_present_on_one_side_is_reported_not_skipped() {
    let a = docx(&para("One page."));
    let b = docx(&(para("One page.") + PAGE_BREAK + &para("Second page.")));
    let d = diff_render(&a, &b, &opts()).unwrap();
    assert_eq!(d.pages.len(), 2);
    assert_eq!(d.pages[0].changed_ratio, 0.0);
    assert_eq!(d.pages[1].index, 1);
    assert_eq!(d.pages[1].only_in, Some("b"));
    assert_eq!(d.pages[1].changed_ratio, 1.0);
    assert_eq!(d.pages[1].bbox, Some([0, 0, 425, 550]));
    assert!(d.overlays[1].is_none(), "nothing to paint over on one side");
    assert!(d.differs());

    let reversed = diff_render(&b, &a, &opts()).unwrap();
    assert_eq!(reversed.pages[1].only_in, Some("a"));
}

#[test]
fn pages_of_different_sizes_count_as_wholly_changed() {
    let letter = docx(&para("Same text."));
    let a4 = docx_with_sect_pr(
        &para("Same text."),
        &[],
        r#"<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>"#,
    );
    let d = diff_render(&letter, &a4, &opts()).unwrap();
    assert_eq!(d.pages.len(), 1);
    let p = &d.pages[0];
    assert_eq!(p.changed_ratio, 1.0);
    assert_eq!(p.only_in, None, "both sides have the page");
    assert_eq!(
        p.bbox,
        Some([0, 0, 425, 585]),
        "Letter 425 wide, A4 585 tall at 50 dpi"
    );
    assert!(d.overlays[0].is_none(), "no overlay across page sizes");
    assert!(d.differs());
}

#[test]
fn page_diffs_serialize_with_only_in_when_present() {
    let a = docx(&para("One page."));
    let b = docx(&(para("One page.") + PAGE_BREAK + &para("Second page.")));
    let d = diff_render(&a, &b, &opts()).unwrap();
    let json = serde_json::to_value(&d.pages).unwrap();
    assert_eq!(
        json,
        serde_json::json!([
            {"index": 0, "changed_ratio": 0.0, "bbox": null},
            {"index": 1, "changed_ratio": 1.0, "bbox": [0, 0, 425, 550], "only_in": "b"},
        ])
    );
}

#[test]
fn an_invalid_dpi_is_an_error() {
    let a = docx(&para("A"));
    let e = diff_render(&a, &a, &DiffOptions { dpi: 0.0, ..opts() })
        .err()
        .unwrap();
    assert!(e.to_string().contains("dpi"), "{e}");
}

#[test]
fn the_cli_exits_0_when_equal_5_when_different_and_1_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.docx");
    let b = dir.path().join("b.docx");
    std::fs::write(&a, docx(&para("The fee is ten."))).unwrap();
    std::fs::write(&b, docx(&para("The fee is twenty."))).unwrap();
    let run = |x: &std::path::Path, y: &std::path::Path| {
        std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(["diff-render", "--dpi", "30"])
            .arg(x)
            .arg(y)
            .output()
            .unwrap()
    };
    let same = run(&a, &a);
    assert_eq!(same.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&same.stdout),
        "0 of 1 page differ\n"
    );
    let changed = run(&a, &b);
    assert_eq!(changed.status.code(), Some(5));
    assert!(
        String::from_utf8_lossy(&changed.stdout).starts_with("page 1: "),
        "{changed:?}"
    );
    let missing = run(&a, &dir.path().join("missing.docx"));
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("missing.docx"));
}
