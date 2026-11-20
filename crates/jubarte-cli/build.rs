// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

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

fn build_version(root: &Path, repo: &Path, version: &str) -> String {
    // The repository root is two up from this crate. Do not accidentally
    // identify an enclosing repository when building an unpacked crates.io
    // package or vendored source inside another checkout: only trust it
    // when the repository's own path for this build script canonicalizes
    // back to the copy being built (plan T4.8).
    if repo.join(".git").exists()
        && repo
            .join("crates/jubarte-cli/build.rs")
            .canonicalize()
            .ok()
            .as_deref()
            == root.join("build.rs").canonicalize().ok().as_deref()
        && let Some(sha) = git(repo, &["rev-parse", "--short", "HEAD"])
        && !sha.is_empty()
    {
        let dirty = git(repo, &["status", "--porcelain", "--untracked-files=normal"])
            .is_some_and(|status| !status.is_empty());
        return format!("{version}+{sha}{}", if dirty { ".dirty" } else { "" });
    }
    version.to_string()
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = root.join("../..");
    println!(
        "cargo:rustc-env=JUBARTE_BUILD_VERSION={}",
        build_version(root, &repo, env!("CARGO_PKG_VERSION"))
    );
    // Rerun only when the commit moves. Watching the tree or the index would
    // rerun this script, and so recompile the crate, after every edit or
    // `git status`; `.dirty` therefore reflects the tree as of the last
    // commit change, which is what a release build from a tag needs.
    println!("cargo:rerun-if-changed=build.rs");
    if repo.join(".git").exists() {
        // --git-path resolves worktree .git files and shared ref storage.
        // Cargo reruns the script on every build while a watched path is
        // missing, and a fresh CI checkout has no packed-refs: watching it
        // there recompiled the crate on every cargo command. The same holds
        // for the branch's loose ref once `git pack-refs` (or `git gc`)
        // removed it, so a packed ref is watched through its nearest
        // existing folder: the next commit writes the loose ref back there.
        for name in ["HEAD", "packed-refs"] {
            if let Some(path) = git(root, &["rev-parse", "--git-path", name])
                && root.join(&path).exists()
            {
                println!("cargo:rerun-if-changed={path}");
            }
        }
        if let Some(reference) = git(root, &["symbolic-ref", "-q", "HEAD"])
            && let Some(path) = git(root, &["rev-parse", "--git-path", &reference])
        {
            let mut watched = Path::new(&path);
            while !root.join(watched).exists() {
                match watched.parent() {
                    Some(parent) if !parent.as_os_str().is_empty() => watched = parent,
                    _ => break,
                }
            }
            if root.join(watched).exists() {
                println!("cargo:rerun-if-changed={}", watched.display());
            }
        }
    }
}
