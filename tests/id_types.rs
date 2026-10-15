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
/// An HTML comment anywhere on a line: an id line, or the ` <!-- rev … -->`
/// a note definition ends with (pi review of #464).
static COMMENT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<!-- (.*?) -->").unwrap());
/// An expired spelling (`#12`, `#c27`, `r12`, `header1.p2`): the
/// split below would cut it apart, so it is looked for whole.
static EXPIRED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:^|[\s,;(\[{])(#c?[0-9][0-9+]*|[a-z][a-z0-9_]*:(?:rev|p):[0-9]+)").unwrap()
});
/// What an id looks like: lowercase, a number in it, steps joined by `.`,
/// `+` or `-` (`p12`, `footnotes.p2-p3`, `t0.row1`, `r1+2`, `footer2.r7`).
static ID_SHAPED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9]*(?:[.+-][a-z]*[0-9]+)*$").unwrap());

/// Every id-shaped token a view prints where ids live (grok review of
/// #24a: seven narrow patterns missed `cells t1.row0 p194-p208`, `in p195`,
/// `r389 @??`, `body: p0-p172` and the comment lists): the words of each id
/// line and of the YAML header outside quoted text, and the heads of notes
/// (`r1 @AC; r2 @JD`, `c6 @AS re c5:`), never a comment's own text. Notes
/// inside quoted YAML values (`text: "{++x++}{>>footer2.r1660 @AB<<}"`) count.
fn printed_places(view: &str) -> Vec<String> {
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
        for comment in COMMENT.captures_iter(line) {
            places.push(QUOTED.replace_all(&comment[1], " ").into_owned());
        }
    }
    for note in NOTE.captures_iter(view) {
        let head = note[1].split(": ").next().unwrap_or_default();
        places.push(head.to_string());
    }
    places
}

/// Every id-shaped word of [`printed_places`].
fn printed_ids(places: &[String]) -> Vec<String> {
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
        let places = printed_places(&text);
        for expired in places.iter().flat_map(|p| EXPIRED.captures_iter(p)) {
            checked += 1;
            let line = format!("    ✗ {}: an expired spelling is printed", &expired[1]);
            let list = failures.entry(name.clone()).or_default();
            if !list.contains(&line) {
                list.push(line);
            }
        }
        for token in printed_ids(&places) {
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

#[test]
fn places_include_trailing_comments_quoted_notes_and_expired_spellings() {
    let view = "---\nfooter2:\n  text: \"{++x++}{>>footer2.r1660 @AB<<}\"\n---\n\
                [^1]: Text.{>>r10 @AS<<} <!-- rev footnotes.r11 @AS -->\n\
                <!-- p3 rev #12 -->\n";
    let places = printed_places(view);
    let ids = printed_ids(&places);
    for id in ["footer2.r1660", "footnotes.r11", "r10", "p3"] {
        assert!(ids.iter().any(|i| i == id), "{id} not in {ids:?}");
    }
    let expired: Vec<String> = places
        .iter()
        .flat_map(|p| EXPIRED.captures_iter(p))
        .map(|c| c[1].to_string())
        .collect();
    assert_eq!(expired, ["#12"]);
}
