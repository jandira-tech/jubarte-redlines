// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! What jubarte prints is what it parses (docs/superpowers/specs/
//! 2026-10-13-ids-and-markdown-roundtrip.md §2.1): every short id the
//! committed views of `tests/fixtures/md-roundtrip` print parses as a
//! `ShortId` and prints back unchanged. The views are the ones
//! `committed_views_match_the_view` (tests/md_fixed_point.rs) keeps current.
//! Runs over every id and fails at the end with the list.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use jubarte::ids::ShortId;
use regex::Regex;

const FIXTURES: &str = "tests/fixtures/md-roundtrip";

/// The places a view prints an id: the head of an id line, a note's first
/// token, an id followed by an author handle (`r1 @AL`, `footer2.r7 @LW`),
/// a reply's `re c5`, a YAML `id:`, and the blocks an id line lists
/// (`cells p8-p16`, `in c5`).
static PRINTED: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"(?m)^<!-- ([^\s>]+)",
        r"\{>>([^\s<]+)",
        r"([A-Za-z0-9.+:-]+) @[A-Za-z]",
        r"\bre (c\d+)",
        r"\bid: ([A-Za-z0-9.+:-]+)",
        r"\bcells ([a-z0-9.]+-[a-z0-9.]+)",
        r"\bin (c\d+)",
    ]
    .iter()
    .map(|p| Regex::new(p).unwrap())
    .collect()
});

/// Tokens a view prints that are not ids: the note words of a view without
/// ids, and a pair's lone author.
fn not_an_id(token: &str) -> bool {
    matches!(token, "noid" | "-->") || token.starts_with('@')
}

#[test]
fn every_id_the_views_print_parses_and_prints_back() {
    let mut fixtures: Vec<_> = std::fs::read_dir(FIXTURES)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .collect();
    fixtures.sort();
    let mut checked = 0;
    let mut failures: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in &fixtures {
        let text = std::fs::read_to_string(path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        for pattern in PRINTED.iter() {
            for captures in pattern.captures_iter(&text) {
                let token = captures[1].trim_end_matches([',', ';', ':']);
                if not_an_id(token) {
                    continue;
                }
                checked += 1;
                let why = match token.parse::<ShortId>() {
                    Ok(id) if id.to_string() == token => continue,
                    Ok(id) => format!("prints back as {id}"),
                    Err(e) => e.expected.to_string(),
                };
                let list = failures.entry(name.clone()).or_default();
                let line = format!("    ✗ {token}: {why}");
                if !list.contains(&line) {
                    list.push(line);
                }
            }
        }
    }
    assert!(checked > 1000, "only {checked} ids found in {FIXTURES}");
    let count: usize = failures.values().map(Vec::len).sum();
    let report: Vec<String> = failures
        .iter()
        .map(|(name, lines)| format!("  {name}:\n{}", lines.join("\n")))
        .collect();
    assert!(
        failures.is_empty(),
        "{count} distinct printed ids do not round-trip (of {checked} checked):\n{}",
        report.join("\n")
    );
}
