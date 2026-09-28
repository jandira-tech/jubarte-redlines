// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Every neurotic_docx_bench path a test names must exist.
//!
//! Corpus tests skip when their fixture is missing, so that a checkout
//! without the bench still passes. The cost is that a fixture move turns
//! them into silent passes: the 2026-09-27 corpus reorganization left 170
//! renderer assertions skipping until this check was added. When the bench
//! is checked out beside this repository, a missing path fails here.

use std::path::{Path, PathBuf};

const PREFIX: &str = "\"../neurotic_docx_bench/";

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read source dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Literal `"../neurotic_docx_bench/…"` strings, skipping format templates.
fn bench_literals(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(PREFIX) {
        let body = &rest[start + 1..];
        let Some(end) = body.find('"') else { break };
        let literal = &body[..end];
        if !literal.contains('{') && !literal.contains('\\') {
            found.push(literal);
        }
        rest = &body[end..];
    }
    found
}

#[test]
fn bench_literals_skip_templates_and_keep_plain_paths() {
    let text = r#"let a = "../neurotic_docx_bench/corpus/x.docx";
        let b = format!("../neurotic_docx_bench/corpus/{name}.docx");
        let c = "../neurotic_docx_bench/corpus""#;
    assert_eq!(
        bench_literals(text),
        [
            "../neurotic_docx_bench/corpus/x.docx",
            "../neurotic_docx_bench/corpus"
        ]
    );
}

#[test]
fn every_named_bench_path_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    if !root.join("../neurotic_docx_bench").is_dir() {
        eprintln!("skip: ../neurotic_docx_bench is not checked out");
        return;
    }
    let mut files = Vec::new();
    rust_files(&root.join("tests"), &mut files);
    rust_files(&root.join("src"), &mut files);
    files.sort();
    let mut missing = Vec::new();
    // This file's own samples are not fixtures.
    let this_file = root.join(file!());
    for file in files.iter().filter(|file| **file != this_file) {
        let text = std::fs::read_to_string(file).expect("read rust source");
        for literal in bench_literals(&text) {
            if !root.join(literal).exists() {
                let shown = file.strip_prefix(root).unwrap_or(file);
                missing.push(format!("{}: {literal}", shown.display()));
            }
        }
    }
    missing.dedup();
    assert!(
        missing.is_empty(),
        "{} bench paths named by tests do not exist (their tests skip silently); \
         point them at the file's new place in corpus/word (documents.csv maps old names):\n{}",
        missing.len(),
        missing.join("\n")
    );
}
