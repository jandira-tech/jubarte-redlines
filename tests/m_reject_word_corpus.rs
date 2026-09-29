// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Reject All against Word's own, on the neurotic_docx_bench
//! `rejected_tracking` corpus copied into `tests/corpus` (Word's redlines and
//! Word's Reject All of each): every listed pair is present,
//! and every story paragraph's text and mark state match (`jubarte debug
//! WORD JUB -c text` prints `text identical`).

use std::path::PathBuf;

use jubarte::debug::{Check, Options, report};
use jubarte::document_comparer::reject_revisions;

/// The bench fixtures copied into this repository.
fn bench() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/neurotic_docx_bench")
}

#[test]
fn reject_matches_words_reject_all_on_words_redlines() {
    let bench = bench();
    let words = bench.join("grok_run/wr0928/rejected_tracking/docx");
    let selection =
        std::fs::read_to_string(bench.join("corpus/word/notices/rejected_tracking_selection.csv"))
            .unwrap();
    let mut rows = selection.lines();
    let header: Vec<&str> = rows.next().unwrap().split(',').collect();
    let col = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let (docx_col, id_col) = (col("docx"), col("id"));
    let opts = Options {
        checks: vec![Check::Text],
        ..Default::default()
    };
    let mut pairs = 0;
    let mut differ = Vec::new();
    // Every listed pair is tested: a malformed row or a missing file is a
    // failure, never a silent skip.
    for row in rows.filter(|r| !r.trim().is_empty()) {
        let fields: Vec<&str> = row.split(',').collect();
        let (Some(docx), Some(id)) = (fields.get(docx_col), fields.get(id_col)) else {
            differ.push(format!("malformed row: {row}"));
            continue;
        };
        let word = words.join(format!("{id}_rejected_tracking.docx"));
        let redline = bench.join("corpus/word").join(docx);
        pairs += 1;
        let missing: Vec<String> = [&word, &redline]
            .into_iter()
            .filter(|p| !p.is_file())
            .map(|p| p.display().to_string())
            .collect();
        if !missing.is_empty() {
            differ.push(format!("{id}: missing {}", missing.join(", ")));
            continue;
        }
        let ours = reject_revisions(&std::fs::read(&redline).unwrap()).unwrap();
        let out = report(&std::fs::read(&word).unwrap(), Some(&ours), &opts).unwrap();
        if out != "text identical\n" {
            differ.push(format!("{id}:\n{out}"));
        }
    }
    assert!(pairs > 0, "no pairs in {}", bench.display());
    assert!(
        differ.is_empty(),
        "{} of {pairs} differ:\n{}",
        differ.len(),
        differ.join("\n")
    );
}
