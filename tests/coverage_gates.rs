// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1124: every `coverage(off)` sits behind the cfg that enables the
//! `coverage_attribute` feature. Integration tests compile the library
//! without `cfg(test)`, so an attribute gated more loosely than the feature
//! is E0658 in the nightly coverage build, which stable CI never compiles.

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.rs` file below `dir`, nested modules included. Symbolic links
/// (`target` is one under mbx) and build or tool directories are skipped.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            let skip = matches!(
                entry.file_name().to_str(),
                Some("target" | ".git" | "node_modules" | ".venv")
            );
            if !skip {
                rust_files(&path, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// The `.rs` files of the git checkout at `root` that git tracks or would
/// add: an ignored folder never builds. `None` when `root` is no checkout
/// (it has no `.git`); inside one, git must answer, so a failing git never
/// falls back to the walk unseen.
fn checkout_rust_files(root: &Path) -> Option<Vec<PathBuf>> {
    if !root.join(".git").exists() {
        return None;
    }
    // The repository's own ignore rules only: a user's global excludes
    // file must not hide a file of the crate.
    let listed = std::process::Command::new("git")
        .args(["-c", "core.excludesFile=", "ls-files", "-z"])
        .args(["--cached", "--others", "--exclude-standard", "--", "*.rs"])
        .current_dir(root)
        .output()
        .unwrap_or_else(|e| panic!("git ls-files in {}: {e}", root.display()));
    assert!(
        listed.status.success(),
        "git ls-files in {} failed: {}",
        root.display(),
        String::from_utf8_lossy(&listed.stderr)
    );
    let names = String::from_utf8(listed.stdout).expect("git lists UTF-8 paths");
    let mut files = Vec::new();
    for name in names.split('\0').filter(|name| !name.is_empty()) {
        let path = root.join(name);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_file() => files.push(path),
            // A link: the walk below skips links too.
            Ok(_) => {}
            // A tracked file deleted from the checkout.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => panic!("{}: {e}", path.display()),
        }
    }
    Some(files)
}

/// Line number and predicate of each `cfg_attr(PREDICATE, ATTRIBUTE)` whose
/// attribute is `attribute`, however rustfmt splits it over lines.
fn predicates<'a>(text: &'a str, attribute: &str) -> Vec<(usize, &'a str)> {
    let wanted = squeeze(attribute);
    let mut found = Vec::new();
    for (at, _) in text.match_indices("cfg_attr(") {
        let start = at + "cfg_attr(".len();
        let (mut depth, mut comma, mut end) = (1usize, None, None);
        for (i, c) in text[start..].char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(start + i);
                        break;
                    }
                }
                ',' if depth == 1 => comma = Some(start + i),
                _ => {}
            }
        }
        let (Some(comma), Some(end)) = (comma, end) else {
            continue;
        };
        if squeeze(&text[comma + 1..end]) == wanted {
            let line = text[..at].matches('\n').count() + 1;
            found.push((line, text[start..comma].trim()));
        }
    }
    found
}

/// `text` without whitespace.
fn squeeze(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

/// A cfg predicate with the arguments of `all` and `any` sorted, so that
/// the same conditions in another order compare equal.
fn canonical(predicate: &str) -> String {
    let squeezed = squeeze(predicate);
    for head in ["all(", "any("] {
        let Some(inner) = squeezed
            .strip_prefix(head)
            .and_then(|rest| rest.strip_suffix(')'))
        else {
            continue;
        };
        let (mut args, mut depth, mut from) = (Vec::new(), 0usize, 0);
        for (i, c) in inner.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    args.push(canonical(&inner[from..i]));
                    from = i + 1;
                }
                _ => {}
            }
        }
        args.push(canonical(&inner[from..]));
        args.sort();
        return format!("{head}{})", args.join(","));
    }
    squeezed
}

/// The crate roots' feature gate, the number of `coverage(off)` sites in the
/// whole repository, and each site that is not gated like the feature, as
/// `FILE:LINE: PREDICATE`. Only `src/` enables the feature, so a site
/// anywhere else (tests, benches, examples, the binding crates) is loose.
fn audit(root: &Path) -> (String, usize, Vec<String>) {
    let gates: Vec<String> = ["src/lib.rs", "src/bin/jubarte.rs"]
        .iter()
        .filter(|crate_root| root.join(crate_root).exists())
        .flat_map(|crate_root| {
            let text = fs::read_to_string(root.join(crate_root)).unwrap();
            predicates(&text, "feature(coverage_attribute)")
                .into_iter()
                .map(|(_, p)| p.to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(
        !gates.is_empty(),
        "no feature gate found in the crate roots"
    );
    assert!(
        gates.iter().all(|g| canonical(g) == canonical(&gates[0])),
        "the crate roots gate the feature alike: {gates:?}"
    );
    let gate = gates[0].clone();
    let mut files = checkout_rust_files(root).unwrap_or_else(|| {
        let mut files = Vec::new();
        rust_files(root, &mut files);
        files
    });
    files.sort();
    let mut sites = 0;
    let mut loose = Vec::new();
    for path in &files {
        let relative = path.strip_prefix(root).unwrap();
        // This file quotes loose gates as test data.
        if relative == Path::new("tests/coverage_gates.rs") {
            continue;
        }
        let in_src = relative.starts_with("src");
        let text = fs::read_to_string(path).unwrap();
        for (line, predicate) in predicates(&text, "coverage(off)") {
            sites += 1;
            if !in_src || canonical(predicate) != canonical(&gate) {
                // A repo-relative path: `/` between its components on
                // every OS, as the report compares them (#1222).
                let file = relative
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let place = if in_src {
                    ""
                } else {
                    " (outside src/, where no crate enables the feature)"
                };
                loose.push(format!("{file}:{line}: {predicate}{place}"));
            }
        }
    }
    (gate, sites, loose)
}

/// A throwaway crate: its `src/lib.rs` gates the feature as this crate does,
/// and `files` adds `(PATH, TEXT)` beside it.
fn sample_crate(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let mut all = vec![(
        "src/lib.rs",
        "#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]\n",
    )];
    all.extend_from_slice(files);
    for (path, text) in all {
        let path = dir.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    dir
}

#[test]
fn coverage_off_uses_the_feature_gate() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (gate, sites, loose) = audit(&root);
    assert!(sites > 0, "no coverage(off) site found: the scan is broken");
    assert!(
        loose.is_empty(),
        "{} of {sites} coverage(off) sites are not gated like the feature ({gate}):\n{}",
        loose.len(),
        loose.join("\n")
    );
}

#[test]
fn a_gate_rustfmt_splits_over_lines_is_still_a_site() {
    let text =
        "#[cfg_attr(\n    all(coverage_nightly, test),\n    coverage(off)\n)]\nmod tests {}\n";
    assert_eq!(
        predicates(text, "coverage(off)"),
        [(1, "all(coverage_nightly, test)")],
        "a multi-line coverage(off) must not escape the guard"
    );
}

#[test]
fn the_order_of_a_gates_conditions_does_not_loosen_it() {
    let dir = sample_crate(&[(
        "src/a.rs",
        "#[cfg_attr(all(test, coverage_nightly), coverage(off))]\nmod tests {}\n",
    )]);
    let (_, sites, loose) = audit(dir.path());
    assert_eq!((sites, loose), (1, Vec::<String>::new()));
}

#[test]
fn a_loose_gate_outside_src_is_reported() {
    // Integration tests, benches, examples and the binding crates do not
    // enable the feature, so any coverage(off) there is E0658 on nightly.
    let dir = sample_crate(&[(
        "tests/t.rs",
        "#[cfg_attr(coverage_nightly, coverage(off))]\nfn helper() {}\n",
    )]);
    let (_, sites, loose) = audit(dir.path());
    assert_eq!(sites, 1, "{loose:?}");
    assert_eq!(loose.len(), 1, "{loose:?}");
    assert!(loose[0].starts_with("tests/t.rs:1: "), "{loose:?}");
}

#[test]
fn a_folder_git_ignores_is_not_scanned() {
    // An ignored local folder (old evidence, a scratch copy of a crate)
    // never builds, so a loose gate in it is no E0658. A file git would
    // add is still scanned.
    let dir = sample_crate(&[
        (".gitignore", "/archive/\n"),
        (
            "archive/probe/src/lib.rs",
            "#[cfg_attr(coverage_nightly, coverage(off))]\nfn helper() {}\n",
        ),
        (
            "tests/t.rs",
            "#[cfg_attr(coverage_nightly, coverage(off))]\nfn helper() {}\n",
        ),
    ]);
    let init = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir.path())
        .status()
        .expect("git runs");
    assert!(init.success());
    let (_, sites, loose) = audit(dir.path());
    assert_eq!(sites, 1, "{loose:?}");
    assert_eq!(loose.len(), 1, "{loose:?}");
    assert!(loose[0].starts_with("tests/t.rs:1: "), "{loose:?}");
}
