// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Accept All against Word's own, on the local `_to_improve_accepted_changes`
//! corpus (Word's redlines and Word's Accept All of each; not shipped, the
//! test skips without it): every story paragraph's text and mark state match
//! (`jubarte debug WORD JUB -c text` prints `text identical`).

use std::path::PathBuf;

use jubarte::debug::{Check, Options, report};
use jubarte::document_comparer::accept_revisions;

/// The corpus in this checkout, or in the main checkout from a worktree.
fn corpus() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    [
        root.join("_to_improve_accepted_changes"),
        root.join("../../_to_improve_accepted_changes"),
    ]
    .into_iter()
    .find(|p| p.join("_word_sot_accepted_docx").is_dir())
}

#[test]
fn accept_matches_words_accept_all_on_words_redlines() {
    let Some(dir) = corpus() else {
        eprintln!("skip: _to_improve_accepted_changes missing");
        return;
    };
    let opts = Options {
        checks: vec![Check::Text],
        ..Default::default()
    };
    let mut pairs = 0;
    let mut differ = Vec::new();
    for entry in std::fs::read_dir(dir.join("_word_sot_accepted_docx")).unwrap() {
        let word = entry.unwrap().path();
        let redline = dir
            .join("_original_unaccepted_docx")
            .join(word.file_name().unwrap());
        if word.extension().is_none_or(|e| e != "docx") || !redline.is_file() {
            continue;
        }
        pairs += 1;
        let ours = accept_revisions(&std::fs::read(&redline).unwrap()).unwrap();
        let out = report(&std::fs::read(&word).unwrap(), Some(&ours), &opts).unwrap();
        if out != "text identical\n" {
            differ.push(format!("{}:\n{out}", word.display()));
        }
    }
    assert!(pairs > 0, "no pairs in {}", dir.display());
    assert!(
        differ.is_empty(),
        "{} of {pairs} differ:\n{}",
        differ.len(),
        differ.join("\n")
    );
}
