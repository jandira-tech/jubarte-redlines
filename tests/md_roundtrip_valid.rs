// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Every Markdown round-trip fixture reads into a Word-valid docx. The
//! round trip (`md_fixed_point.rs`) counts ids; this gate keeps the ids the
//! reader brings back from making the package invalid (two marks sharing a
//! `w:id`, a comment number claimed twice).

mod common;

use std::collections::HashMap;
use std::path::PathBuf;

use common::validity::check_word_valid_package;
use jubarte::markdown::{DocxOptions, markdown_to_docx};
use jubarte::validate::Finding;
use regex::Regex;

const FIXTURES: &str = "tests/fixtures/md-roundtrip";

/// How many times `markdown` names each revision number, as a tag
/// (`{>>r5 @AC<<}`, `rev r9`, `break-ins r3`, `styles.r0`): a number the
/// view names twice is written twice, as Word's own files repeat an id
/// for one change split across carriers (`r43+43`).
fn named_numbers(markdown: &str) -> HashMap<u32, usize> {
    let numbers = Regex::new(r"[^A-Za-z0-9_]r(\d+(?:\+\d+)*)").unwrap();
    let mut named = HashMap::new();
    for (_, [chain]) in numbers.captures_iter(markdown).map(|c| c.extract()) {
        for number in chain.split('+') {
            if let Ok(number) = number.parse::<u32>() {
                *named.entry(number).or_default() += 1;
            }
        }
    }
    named
}

/// Whether a duplicate revision id is one the view asked for: the number
/// it duplicates is named more than once.
fn asked_for(finding: &Finding, named: &HashMap<u32, usize>) -> bool {
    let duplicate = Regex::new(r"duplicate w:id '(\d+)' on revision markup").unwrap();
    duplicate
        .captures(&finding.message)
        .and_then(|c| c[1].parse::<u32>().ok())
        .and_then(|id| named.get(&id).copied())
        .is_some_and(|times| times > 1)
}

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
        let named = named_numbers(&markdown);
        match markdown_to_docx(&markdown, &DocxOptions::default()) {
            Ok(result) => {
                let report = check_word_valid_package(&result.docx);
                let invalid: Vec<&Finding> = report
                    .findings
                    .iter()
                    .filter(|finding| {
                        !(finding.code == "DUPLICATE_REVISION_ID" && asked_for(finding, &named))
                    })
                    .collect();
                if !invalid.is_empty() {
                    failures.push(format!(
                        "{name}: {} findings, first: {}",
                        invalid.len(),
                        invalid[0].message
                    ));
                }
            }
            Err(e) => failures.push(format!("{name}: does not read: {e}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
