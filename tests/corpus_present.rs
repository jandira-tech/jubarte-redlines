// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! The bench test corpus is a git submodule (`tests/corpus/bench`,
//! jandira-tech/jubarte-test-corpus): third-party Word documents that are
//! not licensed with this repository. A clone made without
//! `--recurse-submodules` has an empty folder there, and every test that
//! reads a corpus document then fails with a missing file. This test says
//! why, and how to fix it.

use std::path::Path;

#[test]
fn bench_corpus_submodule_is_checked_out() {
    let bench = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus/bench");
    let missing: Vec<&str> = [
        "PROVENANCE.txt",
        "neurotic_docx_bench/corpus/word_based/docx_source",
        "_to_improve_accepted_changes",
        "issue_1926",
    ]
    .into_iter()
    .filter(|entry| !bench.join(entry).exists())
    .collect();
    assert!(
        missing.is_empty(),
        "the bench test corpus is not checked out ({} lacks {missing:?}).\n\
         Run: git submodule update --init tests/corpus/bench",
        bench.display()
    );
}
