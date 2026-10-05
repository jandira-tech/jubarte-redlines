// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The font report says whether each requested family was substituted, and
//! `jubarte convert --fail-on-substitution` turns a substitution into exit 4.

mod common;

use std::process::Command;

use common::docx::docx;
use jubarte::convert::{FontReportEntry, FontStep, PdfOptions, docx_render_report};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn missing_font_docx() -> Vec<u8> {
    docx(
        r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="NoSuchFont" w:hAnsi="NoSuchFont"/></w:rPr><w:t>hello</w:t></w:r></w:p>"#,
    )
}

fn font_docx(families: &[&str]) -> Vec<u8> {
    let runs: String = families
        .iter()
        .map(|f| {
            format!(
                r#"<w:r><w:rPr><w:rFonts w:ascii="{f}" w:hAnsi="{f}" w:cs="{f}" w:eastAsia="{f}"/></w:rPr><w:t>hello</w:t></w:r>"#
            )
        })
        .collect();
    let mark = families.first().copied().unwrap_or("Carlito");
    docx(&format!(
        r#"<w:p><w:pPr><w:rPr><w:rFonts w:ascii="{mark}" w:hAnsi="{mark}"/></w:rPr></w:pPr>{runs}</w:p>"#
    ))
}

fn convert(input: &[u8], extra: &[&str]) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("in.docx");
    std::fs::write(&path, input).unwrap();
    Command::new(BIN)
        .arg("convert")
        .arg(&path)
        .arg("-o")
        .arg(dir.path().join("out.pdf"))
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn the_report_names_only_the_fonts_the_document_asks_for() {
    // A document without comments paints no balloon, so the balloon
    // label's Times New Roman and the default face are no requests of it.
    // Listed, they failed --fail-on-substitution on a machine without
    // Calibri for a document that asks for Carlito alone.
    let report = docx_render_report(&font_docx(&["Carlito"]), PdfOptions::default()).unwrap();
    let requested: Vec<&str> = report.fonts.iter().map(|f| f.requested.as_str()).collect();
    assert_eq!(requested, ["Carlito"]);
}

#[test]
fn a_bundled_family_is_not_substituted_and_the_flag_passes() {
    let docx = font_docx(&["Carlito"]);
    let report = docx_render_report(&docx, PdfOptions::default()).unwrap();
    assert!(
        report.fonts.iter().all(|f| !f.substituted()),
        "{:?}",
        report.fonts
    );
    let out = convert(&docx, &["--fail-on-substitution"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn every_substitution_is_listed_and_counted() {
    let out = convert(
        &font_docx(&["NoSuchFont", "AlsoMissing"]),
        &["--fail-on-substitution"],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(4), "{stderr}");
    assert!(stderr.contains("substituted: NoSuchFont -> "), "{stderr}");
    assert!(stderr.contains("substituted: AlsoMissing -> "), "{stderr}");
    assert!(stderr.contains("fonts were substituted"), "{stderr}");
}

#[test]
fn a_convert_error_with_the_flag_is_still_exit_1() {
    let out = convert(b"not a docx", &["--fail-on-substitution"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn entry(step: FontStep) -> FontReportEntry {
    FontReportEntry {
        requested: "Family".into(),
        step,
        physical: "Face".into(),
        bold: false,
        italic: false,
        synthetic: false,
    }
}

#[test]
fn only_the_fallback_steps_count_as_substituted() {
    for (step, substituted) in [
        (FontStep::Embedded, false),
        (FontStep::Explicit, false),
        (FontStep::AltName, false),
        (FontStep::Theme, false),
        (FontStep::WordSubstitution, true),
        (FontStep::OpenFallback, true),
        (FontStep::Generic, true),
        (FontStep::Unknown, true),
    ] {
        let entry = entry(step);
        assert_eq!(entry.substituted(), substituted, "{step}");
        let json: serde_json::Value = serde_json::from_str(&entry.to_json()).unwrap();
        assert_eq!(json["substituted"], substituted, "{step}");
    }
}

#[test]
fn a_bundled_face_of_the_requested_family_is_not_a_substitution() {
    let bundled = |requested: &str, physical: &str| FontReportEntry {
        requested: requested.into(),
        physical: physical.into(),
        ..entry(FontStep::OpenFallback)
    };
    assert!(!bundled("Carlito", "Carlito").substituted());
    assert!(!bundled("Carlito", "Carlito-Bold").substituted());
    assert!(!bundled("Liberation Sans", "LiberationSans").substituted());
    assert!(!bundled("carlito, sans-serif", "Carlito").substituted());
    assert!(bundled("Calibri", "Carlito").substituted());
    assert!(bundled("Arial", "LiberationSans").substituted());
    assert!(bundled("Carl", "Carlito").substituted());
}

#[test]
fn synthetic_style_alone_is_not_a_substitution() {
    let entry = FontReportEntry {
        synthetic: true,
        bold: true,
        ..entry(FontStep::Explicit)
    };
    assert!(!entry.substituted());
}

#[test]
fn an_unknown_family_is_reported_as_substituted() {
    let report = docx_render_report(&missing_font_docx(), PdfOptions::default()).unwrap();
    let entry = report
        .fonts
        .iter()
        .find(|f| f.requested == "NoSuchFont")
        .expect("requested family");
    assert!(entry.substituted(), "{entry:?}");
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
    let fonts = json["fonts"].as_array().unwrap();
    let reported = fonts
        .iter()
        .find(|f| f["requested"] == "NoSuchFont")
        .expect("requested family in the JSON report");
    assert_eq!(reported["substituted"], true);
    assert!(fonts.iter().all(|f| f["substituted"].is_boolean()));
}

#[test]
fn fail_on_substitution_exits_4_after_writing_every_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.docx");
    std::fs::write(&input, missing_font_docx()).unwrap();
    let pdf = dir.path().join("out.pdf");
    let report = dir.path().join("report.json");
    let out = Command::new(BIN)
        .arg("convert")
        .arg(&input)
        .arg("-o")
        .arg(&pdf)
        .arg("--report")
        .arg(&report)
        .arg("--fail-on-substitution")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(4), "{stderr}");
    assert!(stderr.contains("substituted: NoSuchFont -> "), "{stderr}");
    assert!(pdf.exists(), "the PDF is written before the exit");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert!(
        json["fonts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["requested"] == "NoSuchFont" && f["substituted"] == true)
    );
}

#[test]
fn without_the_flag_a_substitution_still_exits_0() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.docx");
    std::fs::write(&input, missing_font_docx()).unwrap();
    let out = Command::new(BIN)
        .arg("convert")
        .arg(&input)
        .arg("-o")
        .arg(dir.path().join("out.pdf"))
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!String::from_utf8_lossy(&out.stderr).contains("substituted:"));
}
