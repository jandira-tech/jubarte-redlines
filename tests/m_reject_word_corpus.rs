// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Reject All against Word's own, on the neurotic_docx_bench
//! `rejected_tracking` corpus (Word's redlines and Word's Reject All of each;
//! not shipped, the test skips without it): every story paragraph's text and
//! mark state match (`jubarte debug WORD JUB -c text` prints
//! `text identical`).

use std::path::PathBuf;

use jubarte::debug::{Check, Options, report};
use jubarte::document_comparer::reject_revisions;

/// The bench beside this checkout, or beside the main checkout from a
/// worktree.
fn bench() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    [
        root.join("../neurotic_docx_bench"),
        root.join("../../../neurotic_docx_bench"),
    ]
    .into_iter()
    .find(|p| {
        p.join("corpus/word/notices/rejected_tracking_selection.csv")
            .is_file()
    })
}

#[test]
fn reject_matches_words_reject_all_on_words_redlines() {
    let Some(bench) = bench() else {
        eprintln!("skip: neurotic_docx_bench missing");
        return;
    };
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
    for row in rows {
        let fields: Vec<&str> = row.split(',').collect();
        let (Some(docx), Some(id)) = (fields.get(docx_col), fields.get(id_col)) else {
            continue;
        };
        let word = words.join(format!("{id}_rejected_tracking.docx"));
        let redline = bench.join("corpus/word").join(docx);
        if !word.is_file() || !redline.is_file() {
            continue;
        }
        pairs += 1;
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
