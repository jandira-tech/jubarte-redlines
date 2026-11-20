// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The integration-test suite layout the workspace split depends on.
//!
//! Integration tests live in seven suite binaries, `tests/it_<suite>/`, one
//! per layer of the split: a test moves
//! into the suite of the highest layer it references, and only the tests
//! named by [`STANDALONE`] stay their own binary. This guard keeps that
//! layout true: a test file dropped at `tests/` fails here until it moves
//! into a suite (or joins the standalone list), and a file moved into a
//! suite without a `mod` line in that suite's `main.rs` fails here too, so
//! its tests can never silently stop compiling.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Integration tests that stay standalone binaries.
const STANDALONE: &[&str] = &[
    "adoption",
    "bench_fixture_paths_resolve",
    "cli_version",
    "convert_docx_to_pdf",
    "corpus_present",
    "coverage_gates",
    "crate_layering",
    "deny_build_scripts",
    "facade_surface",
    "foreign_alignment",
    "legacy_doc",
    "legacy_doc_convert",
    "md_fixed_point",
    "m_validity_ring1",
    "perf_atom_clone_allocs",
    "perf_contract",
    "perf_serialize_prefix_allocs",
    "readme_coverage",
    "release_metadata",
    "repo_paths_fit_windows",
    "schema_consistency",
    "suite_membership",
    "tuning_audit",
];

fn stem(path: &Path) -> String {
    path.file_stem()
        .expect("file stem")
        .to_string_lossy()
        .into_owned()
}

/// Stems of the `.rs` files directly inside `dir` (subdirectories ignored).
fn rust_stems(dir: &Path) -> BTreeSet<String> {
    let mut stems = BTreeSet::new();
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display())) {
        let path: PathBuf = entry.expect("dir entry").path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            stems.insert(stem(&path));
        }
    }
    stems
}

/// The `mod <name>;` names declared in a suite's `main.rs` (`common`, which
/// is pulled in by `#[path]`, excluded).
fn declared_mods(main_rs: &str) -> BTreeSet<String> {
    let mut mods = BTreeSet::new();
    for line in main_rs.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix("mod ") else {
            continue;
        };
        let Some(name) = rest.strip_suffix(';') else {
            continue;
        };
        if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            mods.insert(name.to_owned());
        }
    }
    mods.remove("common");
    mods
}

#[test]
fn only_listed_tests_are_standalone() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let found = rust_stems(&tests);
    let listed: BTreeSet<String> = STANDALONE.iter().map(|s| (*s).to_owned()).collect();
    assert_eq!(
        found, listed,
        "tests/*.rs must be the standalone list; every other file moves into \
         tests/it_<suite>/ (highest layer referenced, plan A8)"
    );
}

#[test]
fn every_suite_file_is_declared() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut suites: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&tests).expect("read tests/") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() && stem(&path).starts_with("it_") {
            suites.push(path);
        }
    }
    suites.sort();
    assert!(
        !suites.is_empty(),
        "no tests/it_* suites found; T1.1 has not run"
    );
    for suite in &suites {
        let main_rs = suite.join("main.rs");
        let text = fs::read_to_string(&main_rs)
            .unwrap_or_else(|e| panic!("read {}: {e}", main_rs.display()));
        let declared = declared_mods(&text);
        let mut members = rust_stems(suite);
        members.remove("main");
        assert_eq!(
            declared,
            members,
            "{}: every member file needs its `mod <file>;` line, and every \
             `mod` line its file",
            suite.display()
        );
    }
}
