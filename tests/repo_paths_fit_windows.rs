// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Every tracked path must check out on Windows. Git for Windows stops at
//! MAX_PATH (260) for the full path, and the Actions runner checks out under
//! `D:\a\jubarte-redlines\jubarte-redlines\` (40 characters), so the v0.10.1
//! Windows build failed on 59 bench redlines named
//! `<a>_<48 hex>__vs__<b>_<48 hex>_redline_<h>.docx` (up to 224 characters)
//! and the GitHub release job was skipped.

use std::path::PathBuf;
use std::process::Command;

/// Repository-relative length allowed; leaves room for a longer checkout
/// root (a fork's name, a nested workspace).
const MAX_REL_PATH: usize = 200;

#[test]
fn every_tracked_path_fits_a_windows_checkout() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(&root)
        .output()
        .expect("git ls-files");
    assert!(out.status.success(), "git ls-files failed");
    let listing = String::from_utf8(out.stdout).expect("utf-8 paths");
    let paths: Vec<&str> = listing.split('\0').filter(|p| !p.is_empty()).collect();
    assert!(
        !paths.is_empty(),
        "no tracked files under {}",
        root.display()
    );
    let long: Vec<String> = paths
        .iter()
        .filter(|p| p.chars().count() > MAX_REL_PATH)
        .map(|p| format!("{} {p}", p.chars().count()))
        .collect();
    assert!(
        long.is_empty(),
        "{} tracked paths exceed {MAX_REL_PATH} characters:\n{}",
        long.len(),
        long.join("\n")
    );
}
