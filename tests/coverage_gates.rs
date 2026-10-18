// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1124: every `coverage(off)` sits behind the cfg that enables the
//! `coverage_attribute` feature. Integration tests compile the library
//! without `cfg(test)`, so an attribute gated more loosely than the feature
//! is E0658 in the nightly coverage build, which stable CI never compiles.

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.rs` file below `dir`, nested modules included.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Line number and predicate of each `cfg_attr(PREDICATE, attribute)`.
fn predicates<'a>(text: &'a str, attribute: &str) -> Vec<(usize, &'a str)> {
    let tail = format!(", {attribute})");
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let rest = &line[line.find("cfg_attr(")? + "cfg_attr(".len()..];
            Some((i + 1, rest[..rest.rfind(&tail)?].trim()))
        })
        .collect()
}

#[test]
fn coverage_off_uses_the_feature_gate() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let gates: Vec<String> = ["src/lib.rs", "src/bin/jubarte.rs"]
        .iter()
        .flat_map(|crate_root| {
            let text = fs::read_to_string(root.join(crate_root)).unwrap();
            predicates(&text, "feature(coverage_attribute)")
                .into_iter()
                .map(|(_, p)| p.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(gates.len(), 2, "one feature gate per crate root: {gates:?}");
    assert_eq!(gates[0], gates[1], "the crate roots gate the feature alike");
    let gate = &gates[0];
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    files.sort();
    let mut sites = 0;
    let mut loose = Vec::new();
    for path in &files {
        let text = fs::read_to_string(path).unwrap();
        for (line, predicate) in predicates(&text, "coverage(off)") {
            sites += 1;
            if predicate != gate {
                let file = path.strip_prefix(&root).unwrap().display();
                loose.push(format!("{file}:{line}: {predicate}"));
            }
        }
    }
    assert!(sites > 0, "no coverage(off) site found: the scan is broken");
    assert!(
        loose.is_empty(),
        "{} of {sites} coverage(off) sites are not gated like the feature ({gate}):\n{}",
        loose.len(),
        loose.join("\n")
    );
}
