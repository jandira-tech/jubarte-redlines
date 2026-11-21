// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! What may be tracked in this repository.
//!
//! The repository is public and is meant to stay small and readable: no
//! built program, no log, no scratch file, no note that names a
//! maintainer's machine. Each test here reads `git ls-files` and refuses
//! one kind of clutter. To add a top-level entry or a large file, extend
//! the list it names, in the same pull request, and say why.
//!
//! The tests pass without checking anything where there is no git checkout
//! (a packaged crate has none).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every file and folder allowed at the top of the repository.
const TOP_LEVEL: &[&str] = &[
    ".config",
    ".editorconfig",
    ".gitattributes",
    ".github",
    ".gitignore",
    ".gitleaks.toml",
    ".gitmodules",
    ".shellcheckrc",
    "AGENTS.md",
    "CHANGELOG.md",
    "CITATION.cff",
    "CODE_OF_CONDUCT.md",
    "CONTRIBUTING.md",
    "Cargo.lock",
    "Cargo.lock.license",
    "Cargo.toml",
    "GEMINI.md",
    "KNOWN_ISSUES.md",
    "LICENSE",
    "LICENSES",
    "LICENSES.md",
    "NOTICE",
    "README.crates.md",
    "README.md",
    "REUSE.toml",
    "SECURITY.md",
    "SUPPORT.md",
    "assets",
    "benches",
    "codecov.yml",
    "crates",
    "deny.toml",
    "docs",
    "examples",
    "fuzz",
    "gemini-extension.json",
    "jubarte-python",
    "jubarte-rust-inproc",
    "jubarte-wasm",
    "pyrefly.toml",
    "release_info",
    "ruff.toml",
    "rust-toolchain.toml",
    "rustfmt.toml",
    "schemas",
    "scripts",
    "skills",
    "src",
    "tests",
    "tools",
];

/// A tracked file may be larger than this only under [`LARGE_FILES`].
const SIZE_LIMIT: u64 = 1024 * 1024;

/// Folders whose files may exceed [`SIZE_LIMIT`]: test inputs, and the
/// release evidence and API listings each release adds.
const LARGE_FILES: &[&str] = &["docs/api/", "release_info/", "tests/"];

/// Folders whose files are captured command output, kept as evidence: a
/// log or an empty file there is the record of a run.
const CAPTURED_OUTPUT: &[&str] = &["examples/adoption/"];

/// Files that quote a home-directory path on purpose: the check that finds
/// such paths in built artifacts, its tests, and tests that format a path.
const NAMES_A_HOME: &[&str] = &[
    "crates/jubarte-cli/src/cli.rs",
    "crates/jubarte-render/src/convert/font.rs",
    "crates/jubarte-render/src/convert/settings.rs",
    "scripts/check_npm_paths.py",
    "scripts/test_check_npm_paths.py",
    "scripts/test_gen_docs.py",
    "tests/repo_hygiene.rs",
];

/// Files that name the engine's retired name to forbid it or as history.
const NAMES_THE_RETIRED_NAME: &[&str] = &["AGENTS.md", "CHANGELOG.md", "tests/repo_hygiene.rs"];

/// Files that may name the license the project left at 0.19.0: the
/// history of the change, the notices that state it, another license's
/// text, and tests and review rules about licenses.
const NAMES_THE_FORMER_LICENSE: &[&str] = &[
    "CHANGELOG.md",
    "LICENSES/MPL-2.0.txt",
    "README.crates.md",
    "README.md",
    "docs/adoption/README.md",
    "docs/adoption/anthropic-docx-skill.md",
    "docs/adoption/openai-doc-skill.md",
    "jubarte-python/README.md",
    "jubarte-wasm/cli/README.md",
    "scripts/test_check_evidence_engine.py",
];

/// Test inputs and recorded output: data, not our prose. The text checks
/// skip them; the tests' own source files are checked.
const DATA: &[&str] = &[
    "docs/api/",
    "tests/corpus/",
    "tests/data/",
    "tests/fixtures/",
    "tests/goldens/",
];

/// What the tracked files may weigh together, in bytes. A fresh clone
/// downloads every one of them.
const TREE_BUDGET: u64 = 72 * MIB;

/// What each top-level folder's tracked files may weigh, in bytes. A folder
/// that is not listed, and the files at the root taken together, get
/// [`DEFAULT_FOLDER_BUDGET`]. The figures are the sizes at 0.19.0 plus a
/// tenth: growth past one is a decision someone reviews, not an accident.
const FOLDER_BUDGETS: &[(&str, u64)] = &[
    ("crates", 24 * MIB),
    ("tests", 21 * MIB),
    // two releases of API snapshots: the previous one and the current
    ("docs", 15 * MIB),
    ("release_info", 6 * MIB),
    ("examples", 5 * MIB),
    ("scripts", 3 * MIB / 2),
    ("assets", 3 * MIB / 2),
    ("jubarte-python", 3 * MIB / 2),
];
const DEFAULT_FOLDER_BUDGET: u64 = MIB;
const MIB: u64 = 1024 * 1024;

/// What to do instead of raising a budget.
const WHERE_WEIGHT_GOES: &str = "\
Before raising a budget in tests/repo_hygiene.rs, move the weight out:
  - a third-party or bulky test document: the jubarte-test-corpus submodule,
    or the dataset scripts/fetch_fixtures.py downloads;
  - anything a build produces (packages, binaries, reports): build it in CI;
  - an older release's API snapshot (docs/api) or evidence set
    (release_info): delete it, the previous release's tag still holds it.
    Keep the previous release's and the one being released;
  - plans, notes and logs: they do not belong in the repository.";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The tracked files, or `None` outside a git checkout.
fn tracked() -> Option<Vec<String>> {
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let names: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect();
    (!names.is_empty()).then_some(names)
}

/// The tracked files that are regular files on disk. A submodule is one
/// tracked entry that is a folder, and a sparse checkout may lack a file.
fn tracked_files() -> Option<Vec<(String, PathBuf)>> {
    Some(
        tracked()?
            .into_iter()
            .map(|name| {
                let path = root().join(&name);
                (name, path)
            })
            .filter(|(_, path)| path.is_file())
            .collect(),
    )
}

fn under(name: &str, folders: &[&str]) -> bool {
    folders.iter().any(|folder| name.starts_with(folder))
}

fn report(what: &str, found: &[String]) {
    assert!(
        found.is_empty(),
        "{what}:\n  {}\n(tests/repo_hygiene.rs says what may be tracked)",
        found.join("\n  ")
    );
}

#[test]
fn the_top_level_holds_only_listed_entries() {
    let Some(names) = tracked() else { return };
    let allowed: BTreeSet<&str> = TOP_LEVEL.iter().copied().collect();
    let found: BTreeSet<String> = names
        .iter()
        .map(|name| name.split('/').next().unwrap_or(name).to_owned())
        .filter(|top| !allowed.contains(top.as_str()))
        .collect();
    report(
        "top-level entries that are not in TOP_LEVEL",
        &found.into_iter().collect::<Vec<_>>(),
    );
}

#[test]
fn no_large_file_outside_the_listed_folders() {
    let Some(files) = tracked_files() else { return };
    let found: Vec<String> = files
        .iter()
        .filter(|(name, _)| !under(name, LARGE_FILES))
        .filter_map(|(name, path)| {
            let size = fs::metadata(path).ok()?.len();
            (size > SIZE_LIMIT).then(|| format!("{name} ({} KiB)", size / 1024))
        })
        .collect();
    report("tracked files over 1 MiB outside LARGE_FILES", &found);
}

#[test]
fn the_tree_stays_within_its_size_budgets() {
    let Some(files) = tracked_files() else { return };
    let mut total = 0u64;
    let mut folders: BTreeMap<&str, u64> = BTreeMap::new();
    for (name, path) in &files {
        let Ok(meta) = fs::metadata(path) else {
            continue;
        };
        let folder = name
            .split_once('/')
            .map_or("(files at the root)", |(top, _)| top);
        total += meta.len();
        *folders.entry(folder).or_default() += meta.len();
    }
    let mib = |bytes: u64| format!("{:.1} MiB", bytes as f64 / MIB as f64);
    let mut found: Vec<String> = folders
        .iter()
        .filter_map(|(folder, &size)| {
            let budget = FOLDER_BUDGETS
                .iter()
                .find(|(name, _)| name == folder)
                .map_or(DEFAULT_FOLDER_BUDGET, |&(_, budget)| budget);
            (size > budget).then(|| format!("{folder}: {} of {}", mib(size), mib(budget)))
        })
        .collect();
    if total > TREE_BUDGET {
        found.push(format!(
            "the whole tree: {} of {}",
            mib(total),
            mib(TREE_BUDGET)
        ));
    }
    assert!(
        found.is_empty(),
        "tracked files over their size budget:\n  {}\n{WHERE_WEIGHT_GOES}",
        found.join("\n  ")
    );
}

/// The first bytes of a native executable or library: ELF, Mach-O (both
/// byte orders, 32 and 64 bit, and the universal wrapper) and PE.
fn is_native_binary(head: &[u8]) -> bool {
    const MAGICS: &[&[u8]] = &[
        b"\x7fELF",
        b"\xfe\xed\xfa\xce",
        b"\xfe\xed\xfa\xcf",
        b"\xce\xfa\xed\xfe",
        b"\xcf\xfa\xed\xfe",
        b"\xca\xfe\xba\xbe",
        b"MZ\x90\x00",
    ];
    MAGICS.iter().any(|magic| head.starts_with(magic))
}

#[test]
fn no_built_program_log_or_scratch_file_is_tracked() {
    let Some(files) = tracked_files() else { return };
    let mut found = Vec::new();
    for (name, path) in &files {
        let extension = Path::new(name)
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        let captured = under(name, CAPTURED_OUTPUT);
        if matches!(extension.as_str(), "tmp" | "bak" | "orig" | "rej")
            || name.ends_with(".DS_Store")
            || (extension == "log" && !captured)
        {
            found.push(format!("{name} (a log or scratch file)"));
            continue;
        }
        let Ok(bytes) = fs::read(path) else { continue };
        if bytes.is_empty() && !captured && !name.ends_with("/py.typed") {
            found.push(format!("{name} (empty)"));
        }
        // Test data may hold a program on purpose (a signed archive's payload).
        if is_native_binary(&bytes) && !name.starts_with("tests/") {
            found.push(format!("{name} (a native executable)"));
        }
        // The npm packages are built in CI, never committed.
        if bytes.starts_with(b"\0asm") && !name.starts_with("tests/") {
            found.push(format!("{name} (a WebAssembly module)"));
        }
        if name.starts_with("jubarte-wasm/npm/") && name.contains("/jubarte_wasm") {
            found.push(format!("{name} (wasm-bindgen output)"));
        }
    }
    report("tracked files that are build output or scratch", &found);
}

#[test]
fn no_file_names_a_maintainers_machine_or_the_retired_name() {
    let Some(files) = tracked_files() else { return };
    // Spelled in pieces so this file does not match itself by accident.
    let home_marks = [
        ["/Us", "ers/"].concat(),
        ["/ho", "me/"].concat(),
        ["\\Us", "ers\\"].concat(),
        ["~/", "T/"].concat(),
        ["temp", "/T/"].concat(),
    ];
    let retired = ["ooxmlsdk", "-redline"].concat();
    let mut found = Vec::new();
    for (name, path) in &files {
        if under(name, DATA) {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue; // not UTF-8: a binary file
        };
        for (number, line) in text.lines().enumerate() {
            if !NAMES_A_HOME.contains(&name.as_str())
                && home_marks.iter().any(|mark| line.contains(mark.as_str()))
            {
                found.push(format!(
                    "{name}:{}: a path on someone's machine",
                    number + 1
                ));
            }
            if !NAMES_THE_RETIRED_NAME.contains(&name.as_str()) && line.contains(&retired) {
                found.push(format!("{name}:{}: the retired name {retired}", number + 1));
            }
        }
    }
    report(
        "tracked text that names a machine or the retired name",
        &found,
    );
}

/// The project is Apache-2.0 from 0.19.0. A file that still names the
/// license it had before is one of the few that record the change, or a
/// leftover.
#[test]
fn no_file_names_the_former_license() {
    let Some(files) = tracked_files() else { return };
    let marks = [["AG", "PL"].concat(), ["Aff", "ero"].concat()];
    let mut found = Vec::new();
    for (name, path) in &files {
        if under(name, DATA)
            || name == "NOTICE"
            || name.ends_with("/NOTICE")
            || under(name, NAMES_THE_FORMER_LICENSE)
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue; // not UTF-8: a binary file
        };
        for (number, line) in text.lines().enumerate() {
            if marks.iter().any(|mark| line.contains(mark.as_str())) {
                found.push(format!("{name}:{}", number + 1));
            }
        }
    }
    report("tracked text that names the former license", &found);
}

#[test]
fn every_license_copy_matches_the_root_license() {
    let Some(files) = tracked_files() else { return };
    let Ok(root_license) = fs::read(root().join("LICENSE")) else {
        return;
    };
    let found: Vec<String> = files
        .iter()
        .filter(|(name, _)| name.ends_with("/LICENSE"))
        .filter(|(_, path)| fs::read(path).ok().as_deref() != Some(root_license.as_slice()))
        .map(|(name, _)| name.clone())
        .collect();
    report("LICENSE copies that differ from the root LICENSE", &found);
}
