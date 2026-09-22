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

/// `src/convert/**/*.rs` lines whose text matches a word-boundary `mini`
/// (the plan.md grep; excludes `minimum`).
fn mini_sites(root: &Path) -> Vec<(String, usize, String)> {
    let dir = root.join("src/convert");
    let mut sites = Vec::new();
    let mut files = Vec::new();
    rust_files(&dir, &mut files);
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

struct AuditRow {
    file: String,
    symbol: String,
    class: char,
    disposition: String,
}

/// Markdown pipe-table rows keyed by `file:line`, plus the raw rows.
/// Line numbers are informational: stacked convert PRs insert above a
/// site and bitrot exact keys. Disposition cells store the source excerpt.
fn audit_table(body: &str) -> (HashMap<String, (char, String)>, Vec<AuditRow>) {
    let mut by_line = HashMap::new();
    let mut rows = Vec::new();
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
        let symbol = cells[2].trim().to_string();
        let class = cells[3].chars().next().unwrap_or('?').to_ascii_lowercase();
        let disposition = cells[4].to_string();
        if file.starts_with("src/") && line_no.chars().all(|c| c.is_ascii_digit()) {
            by_line.insert(format!("{file}:{line_no}"), (class, disposition.clone()));
            rows.push(AuditRow {
                file: file.to_string(),
                symbol,
                class,
                disposition,
            });
        }
    }
    (by_line, rows)
}

fn excerpt_hit<'a>(rows: &'a [AuditRow], file: &str, excerpt: &str) -> Option<&'a AuditRow> {
    if excerpt.is_empty() {
        return None;
    }
    rows.iter().find(|row| {
        if row.file != file {
            return false;
        }
        if row.disposition.contains(excerpt) {
            return true;
        }
        // Comments get a parenthetical in a later PR; the table symbol
        // still names the site (Aptos-only gate after #111).
        let sym = row.symbol.trim();
        !sym.is_empty() && excerpt.contains(sym)
    })
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
    let (by_line, rows) = audit_table(&body);
    let mut missing = Vec::new();
    for (file, line, excerpt) in &sites {
        let key = format!("{file}:{line}");
        let hit = by_line
            .get(&key)
            .map(|(class, disp)| (*class, disp.as_str()))
            .or_else(|| {
                excerpt_hit(&rows, file, excerpt).map(|row| (row.class, row.disposition.as_str()))
            });
        match hit {
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
fn tuning_audit_excerpt_covers_shifted_line() {
    // Stacked convert PRs insert above a mini site; file:line keys go stale.
    let body = "\
| file | line | symbol | class | disposition |\n\
|---|---|---|---|\n\
| src/convert/mod.rs | 1 | mini 0 | b | KEEP. // unique-shifted-excerpt |\n";
    let (by_line, rows) = audit_table(body);
    assert!(
        !by_line.contains_key("src/convert/mod.rs:99"),
        "shifted line must not hit the stale file:line key"
    );
    let hit = excerpt_hit(&rows, "src/convert/mod.rs", "// unique-shifted-excerpt")
        .expect("disposition stores the source excerpt");
    assert_eq!(hit.class, 'b');
}

/// Step 8 helpers: each needs its own parsed table row, in the file that
/// defines it, with an allowed class. Prose mentions do not count.
const STEP8_HELPERS: [(&str, &str); 4] = [
    ("word_device_track", "a"),
    ("word_device_paint", "a"),
    ("word_device_pt", "a"),
    ("apply_latent_ppr", "bc"),
];

fn helper_problems(root: &Path, rows: &[AuditRow]) -> Vec<String> {
    let mut problems = Vec::new();
    for (symbol, classes) in STEP8_HELPERS {
        let Some(row) = rows.iter().find(|row| row.symbol == symbol) else {
            problems.push(format!("{symbol}: no table row"));
            continue;
        };
        if !classes.contains(row.class) {
            problems.push(format!("{symbol}: class {} not in {classes}", row.class));
        }
        if row.disposition.is_empty() {
            problems.push(format!("{symbol}: empty disposition"));
        }
        let defines = fs::read_to_string(root.join(&row.file))
            .is_ok_and(|text| text.contains(&format!("fn {symbol}(")));
        if !defines {
            problems.push(format!("{symbol}: {} does not define it", row.file));
        }
    }
    problems
}

#[test]
fn tuning_audit_starts_with_word_device_and_heading_gap() {
    let root = crate_root();
    let body = fs::read_to_string(root.join("TUNING_AUDIT.md")).unwrap_or_default();
    assert!(
        !body.is_empty(),
        "TUNING_AUDIT.md must exist (plan Step 8 starts with word_device_* and heading gap)"
    );
    let (_, rows) = audit_table(&body);
    let problems = helper_problems(&root, &rows);
    assert!(problems.is_empty(), "Step 8 helper rows: {problems:#?}");
}

#[test]
fn helper_rows_reject_prose_only_reclassified_and_misfiled_helpers() {
    // #109: substring needles passed with the word_device_pt row deleted
    // (the intro prose names it) and with it reclassified.
    let root = crate_root();
    let full = fs::read_to_string(root.join("TUNING_AUDIT.md")).unwrap_or_default();
    let without = |needle: &str| {
        full.lines()
            .filter(|l| !(l.trim_start().starts_with('|') && l.contains(needle)))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let (_, rows) = audit_table(&without("| word_device_pt |"));
    assert!(
        helper_problems(&root, &rows)
            .iter()
            .any(|p| p.starts_with("word_device_pt: no table row")),
        "a deleted row is caught even though prose still names the helper"
    );
    let (_, mut rows) = audit_table(&full);
    for row in &mut rows {
        if row.symbol == "word_device_pt" {
            row.class = 'b';
        }
        if row.symbol == "apply_latent_ppr" {
            row.file = "src/convert/font.rs".into();
        }
    }
    let problems = helper_problems(&root, &rows);
    assert!(
        problems
            .iter()
            .any(|p| p.starts_with("word_device_pt: class b")),
        "{problems:?}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.starts_with("apply_latent_ppr: src/convert/font.rs does not define")),
        "{problems:?}"
    );
}
