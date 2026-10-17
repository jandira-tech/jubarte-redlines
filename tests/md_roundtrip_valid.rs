// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Every Markdown round-trip fixture reads into a Word-valid docx. The
//! round trip (`md_fixed_point.rs`) counts ids; this gate keeps the ids the
//! reader brings back from making the package invalid (two marks sharing a
//! `w:id`, a comment number claimed twice).

mod common;

use std::path::PathBuf;

use common::validity::check_word_valid_package;
use jubarte::markdown::{DocxOptions, markdown_to_docx};

const FIXTURES: &str = "tests/fixtures/md-roundtrip";

#[test]
fn every_round_trip_fixture_reads_into_a_word_valid_docx() {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(FIXTURES)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("md")))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "no Markdown fixtures in {FIXTURES}");
    let mut failures = Vec::new();
    for path in &paths {
        let markdown = std::fs::read_to_string(path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        match markdown_to_docx(&markdown, &DocxOptions::default()) {
            Ok(result) => {
                let report = check_word_valid_package(&result.docx);
                if !report.ok() {
                    failures.push(format!(
                        "{name}: {} findings, first: {}",
                        report.errors.len(),
                        report.errors[0]
                    ));
                }
            }
            Err(e) => failures.push(format!("{name}: does not read: {e}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
