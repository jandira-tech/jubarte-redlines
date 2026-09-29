// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Every copied fixture path a test names must exist.
//!
//! The neurotic_docx_bench documents the tests use are copied into
//! `tests/corpus/neurotic_docx_bench` (same relative paths as in the bench)
//! and Word's accepted redlines into `tests/corpus/_to_improve_accepted_changes`,
//! so no test depends on a checkout beside this one. Many tests still skip
//! when a fixture is missing; this check turns a missing copy into a failure
//! instead of a silent pass (the 2026-09-27 bench reorganization left 170
//! renderer assertions skipping that way).

use std::path::{Path, PathBuf};

const PREFIXES: [&str; 2] = [
    "\"tests/corpus/neurotic_docx_bench/",
    "\"tests/corpus/_to_improve_accepted_changes/",
];

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

/// Literal fixture path strings, skipping format templates.
fn fixture_literals(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    for prefix in PREFIXES {
        let mut rest = text;
        while let Some(start) = rest.find(prefix) {
            let body = &rest[start + 1..];
            let Some(end) = body.find('"') else { break };
            let literal = &body[..end];
            if !literal.contains('{') && !literal.contains('\\') {
                found.push(literal);
            }
            rest = &body[end..];
        }
    }
    found
}

#[test]
fn fixture_literals_skip_templates_and_keep_plain_paths() {
    let text = r#"let a = "tests/corpus/neurotic_docx_bench/corpus/x.docx";
        let b = format!("tests/corpus/neurotic_docx_bench/corpus/{name}.docx");
        let c = "tests/corpus/_to_improve_accepted_changes/y.docx""#;
    assert_eq!(
        fixture_literals(text),
        [
            "tests/corpus/neurotic_docx_bench/corpus/x.docx",
            "tests/corpus/_to_improve_accepted_changes/y.docx"
        ]
    );
}

#[test]
fn every_named_fixture_path_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("tests"), &mut files);
    rust_files(&root.join("src"), &mut files);
    files.sort();
    let mut missing = Vec::new();
    // This file's own samples are not fixtures.
    let this_file = root.join(file!());
    for file in files.iter().filter(|file| **file != this_file) {
        let text = std::fs::read_to_string(file).expect("read rust source");
        for literal in fixture_literals(&text) {
            if !root.join(literal).exists() {
                let shown = file.strip_prefix(root).unwrap_or(file);
                missing.push(format!("{}: {literal}", shown.display()));
            }
        }
    }
    missing.dedup();
    assert!(
        missing.is_empty(),
        "{} fixture paths named by tests do not exist (their tests skip silently); \
         copy the file from neurotic_docx_bench to the same relative path:\n{}",
        missing.len(),
        missing.join("\n")
    );
}
