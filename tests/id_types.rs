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

/// A quoted YAML or id-line string (a header's text, a style name): content,
/// not ids.
static QUOTED: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#""(?:[^"\\]|\\.)*""#).unwrap());
/// A note, `{>>…<<}`.
static NOTE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?s)\{>>(.*?)<<\}").unwrap());
/// What an id looks like: lowercase, a number in it, steps joined by `.`,
/// `+` or `-` (`p12`, `footnotes.p2-p3`, `t0.row1`, `r1+2`, `footer2.r7`).
static ID_SHAPED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9]*(?:[.+-][a-z]*[0-9]+)*$").unwrap());

/// Every id-shaped token a view prints where ids live (grok review of
/// #24a: seven narrow patterns missed `cells t1.row0 p194-p208`, `in p195`,
/// `r389 @??`, `body: p0-p172` and the comment lists): the words of each id
/// line and of the YAML header outside quoted text, and the heads of notes
/// (`r1 @AC; r2 @JD`, `c6 @AS re c5:`), never a comment's own text.
fn printed_ids(view: &str) -> Vec<String> {
    let mut places: Vec<String> = Vec::new();
    let mut lines = view.lines();
    if view.starts_with("---\n") {
        lines.next();
        let mut authors = false;
        for line in lines.by_ref() {
            if line == "---" {
                break;
            }
            // The authors: table holds people's names (`document_owner:
            // jharrop2`), not ids.
            if !line.starts_with(' ') {
                authors = line == "authors:";
            }
            if !authors {
                places.push(QUOTED.replace_all(line, " ").into_owned());
            }
        }
    }
    let body: Vec<&str> = lines.collect();
    for line in &body {
        if let Some(inner) = line.strip_prefix("<!-- ") {
            places.push(QUOTED.replace_all(inner, " ").into_owned());
        }
    }
    for note in NOTE.captures_iter(&body.join("\n")) {
        let head = note[1].split(": ").next().unwrap_or_default();
        places.push(head.to_string());
    }
    places
        .iter()
        .flat_map(|place| place.split(|c: char| c.is_whitespace() || ",;:(){}[]".contains(c)))
        .filter(|word| ID_SHAPED.is_match(word) && word.bytes().any(|b| b.is_ascii_digit()))
        .map(str::to_string)
        .collect()
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
        for token in printed_ids(&text) {
            checked += 1;
            let why = match token.parse::<ShortId>() {
                Ok(id) if id.to_string() == token => continue,
                Ok(id) => format!("prints back as {id}"),
                Err(e) => e.to_string(),
            };
            let list = failures.entry(name.clone()).or_default();
            let line = format!("    ✗ {token}: {why}");
            if !list.contains(&line) {
                list.push(line);
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
