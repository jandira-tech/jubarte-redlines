// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! The bench test corpus is a git submodule (`tests/corpus/bench`,
//! jandira-tech/jubarte-test-corpus): third-party Word documents that are
//! not licensed with this repository. A clone made without
//! `--recurse-submodules` has an empty folder there, and every test that
//! reads a corpus document then fails with a missing file. This test says
//! why, and how to fix it.
//!
//! A second set of documents, whose origin is not confirmed, is in neither
//! repository: `tests/corpus/hf_fixtures.sha256` lists them and
//! `scripts/fetch_fixtures.py` downloads them from a public dataset. The
//! second test says so when they are missing.

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

#[test]
fn fetched_fixtures_are_on_disk() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus");
    let manifest = std::fs::read_to_string(corpus.join("hf_fixtures.sha256"))
        .expect("tests/corpus/hf_fixtures.sha256 is tracked");
    let listed: Vec<&str> = manifest
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| line.split_once("  ").map(|(_, name)| name))
        .collect();
    assert!(
        listed.len() > 100,
        "the manifest lists {} files",
        listed.len()
    );
    let missing: Vec<&str> = listed
        .iter()
        .copied()
        .filter(|name| !corpus.join(name).is_file())
        .collect();
    assert!(
        missing.is_empty(),
        "{} of {} downloaded test documents are missing under {} (first: {}).\n\
         Run: python3 scripts/fetch_fixtures.py",
        missing.len(),
        listed.len(),
        corpus.display(),
        missing[0]
    );
}
