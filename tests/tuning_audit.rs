// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Plan Step 8: every convert `mini` comment site (and the named Word-device
//! / heading-gap helpers) must appear in `TUNING_AUDIT.md` with class a|b|c
//! and a disposition.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// `src/convert/*.rs` lines whose text matches a word-boundary `mini`
/// (the plan.md grep; excludes `minimum`).
fn mini_sites(root: &Path) -> Vec<(String, usize, String)> {
    let dir = root.join("src/convert");
    let mut sites = Vec::new();
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    files.sort();
    for path in files {
        let rel = path
            .strip_prefix(root)
            .expect("under crate root")
            .to_string_lossy()
            .replace('\\', "/");
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
        for (i, line) in text.lines().enumerate() {
            if mini_word(line) {
                sites.push((rel.clone(), i + 1, line.trim().to_string()));
            }
        }
    }
    sites
}

fn mini_word(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let mut rest = lower.as_str();
    while let Some(idx) = rest.find("mini") {
        let before_ok = idx == 0
            || rest
                .as_bytes()
                .get(idx - 1)
                .is_some_and(|b| !b.is_ascii_alphanumeric() && *b != b'_');
        let after = rest.get(idx + 4..).unwrap_or("");
        let after_ok = after
            .as_bytes()
            .first()
            .is_none_or(|b| !b.is_ascii_alphanumeric() && *b != b'_');
        if before_ok && after_ok {
            return true;
        }
        rest = rest.get(idx + 4..).unwrap_or("");
    }
    false
}

/// Markdown pipe-table rows keyed by `file:line`.
fn audit_rows(body: &str) -> HashMap<String, (char, String)> {
    let mut rows = HashMap::new();
    for line in body.lines() {
        let line = line.trim();
        if !line.starts_with('|') || line.contains("---") {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() < 5 {
            continue;
        }
        if cells[0].eq_ignore_ascii_case("file") {
            continue;
        }
        let file = cells[0];
        let line_no = cells[1];
        let class = cells[3].chars().next().unwrap_or('?').to_ascii_lowercase();
        let disposition = cells[4].to_string();
        if file.starts_with("src/") && line_no.chars().all(|c| c.is_ascii_digit()) {
            rows.insert(format!("{file}:{line_no}"), (class, disposition));
        }
    }
    rows
}

#[test]
fn tuning_audit_covers_every_mini_comment_site() {
    let root = crate_root();
    let sites = mini_sites(&root);
    assert!(
        !sites.is_empty(),
        "expected mini comment sites under src/convert"
    );
    let path = root.join("TUNING_AUDIT.md");
    let body = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        path.is_file() && !body.is_empty(),
        "TUNING_AUDIT.md must exist at crate root (plan Step 8)"
    );
    let rows = audit_rows(&body);
    let mut missing = Vec::new();
    for (file, line, excerpt) in &sites {
        let key = format!("{file}:{line}");
        match rows.get(&key) {
            None => missing.push(format!("{key}  {excerpt}")),
            Some((class, disp)) => {
                assert!(
                    matches!(class, 'a' | 'b' | 'c'),
                    "{key} class must be a|b|c, got {class}"
                );
                assert!(!disp.is_empty(), "{key} disposition must be non-empty");
            }
        }
    }
    assert!(
        missing.is_empty(),
        "TUNING_AUDIT.md missing {} mini site(s):\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn tuning_audit_starts_with_word_device_and_heading_gap() {
    let body = fs::read_to_string(crate_root().join("TUNING_AUDIT.md")).unwrap_or_default();
    assert!(
        !body.is_empty(),
        "TUNING_AUDIT.md must exist (plan Step 8 starts with word_device_* and heading gap)"
    );
    for needle in [
        "word_device_track",
        "word_device_paint",
        "word_device_pt",
        "apply_latent_ppr",
    ] {
        assert!(
            body.contains(needle),
            "Step 8 must audit {needle}; table body missing the symbol"
        );
    }
    let mut saw_device_a = false;
    let mut saw_latent = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("word_device_track") || lower.contains("word_device_paint") {
            assert!(
                lower.contains("| a |") || lower.contains("| a|"),
                "word_device_* is class (a) font-metric substitute: {line}"
            );
            saw_device_a = true;
        }
        if lower.contains("apply_latent_ppr") {
            assert!(
                lower.contains("| c |") || lower.contains("| c|") || lower.contains("| b |"),
                "heading-gap apply_latent_ppr needs class b or c: {line}"
            );
            saw_latent = true;
        }
    }
    assert!(saw_device_a, "word_device_* rows must be class a");
    assert!(saw_latent, "apply_latent_ppr row must exist");
}
