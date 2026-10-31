// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::path::Path;
use std::process::Command;

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8(output.stdout).ok()?.trim().to_string())
}

fn build_version(root: &Path, version: &str) -> String {
    // Do not accidentally identify an enclosing repository when building an
    // unpacked crates.io package or vendored source inside another checkout.
    if root.join(".git").exists()
        && let Some(sha) = git(root, &["rev-parse", "--short", "HEAD"])
        && !sha.is_empty()
    {
        let dirty = git(root, &["status", "--porcelain", "--untracked-files=normal"])
            .is_some_and(|status| !status.is_empty());
        return format!("{version}+{sha}{}", if dirty { ".dirty" } else { "" });
    }
    version.to_string()
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!(
        "cargo:rustc-env=JUBARTE_BUILD_VERSION={}",
        build_version(root, env!("CARGO_PKG_VERSION"))
    );
    // Rerun only when the commit moves. Watching the tree or the index would
    // rerun this script, and so recompile the crate, after every edit or
    // `git status`; `.dirty` therefore reflects the tree as of the last
    // commit change, which is what a release build from a tag needs.
    println!("cargo:rerun-if-changed=build.rs");
    if root.join(".git").exists() {
        // --git-path resolves worktree .git files and shared ref storage.
        for name in ["HEAD", "packed-refs"] {
            if let Some(path) = git(root, &["rev-parse", "--git-path", name]) {
                println!("cargo:rerun-if-changed={path}");
            }
        }
        if let Some(reference) = git(root, &["symbolic-ref", "-q", "HEAD"])
            && let Some(path) = git(root, &["rev-parse", "--git-path", &reference])
        {
            println!("cargo:rerun-if-changed={path}");
        }
    }
}
