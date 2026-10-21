// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! Independent oracle for attributed marks in a printed view.
use regex::Regex;

pub const KINDS: [&str; 6] = [
    "insertion",
    "deletion",
    "substitution",
    "paragraph mark",
    "table row",
    "table cell",
];

/// Independent text oracle: count complete attributed marks, never ids in an
/// `re` chain or the two ids joined by a substitution. Id-line clauses carry
/// paragraph marks and table changes that have no inline delimiter.
pub fn printed_counts(body: &str) -> [usize; 6] {
    // The header reports note stories separately, including a note inserted
    // with its body reference (whose attribution is repeated in its definition).
    let body = body
        .split("<!-- footnotes")
        .next()
        .unwrap()
        .split("<!-- endnotes")
        .next()
        .unwrap();
    let mut counts = [0; 6];
    // An attribution is grammar, not arbitrary text after an @. In
    // particular, a backslash before its closer makes it literal text.
    let id =
        r"(?:(?:[A-Za-z0-9_-]+\.)?(?:r[0-9]+(?:\+[0-9]+)*|noid)|[A-Za-z0-9_-]+:rev:[^\s{}\\]+)";
    let handle = r"(?:[\p{L}\p{N}]+|\?\?)";
    let date = r"[0-9]{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12][0-9]|3[01])T(?:[01][0-9]|2[0-3]):[0-5][0-9](?::[0-5][0-9](?:\.[0-9]+)?)?(?:Z|[+-][0-9]{2}:[0-5][0-9])?";
    let part = format!(r"{id} @{handle}(?: {date})?");
    let attribution = format!(r"{part}(?: re {part})*");
    let escaped = |at: usize| {
        body[..at]
            .bytes()
            .rev()
            .take_while(|&byte| byte == b'\\')
            .count()
            % 2
            != 0
    };
    // Comment text is opaque, even if it quotes a complete attributed
    // change. Attribution-looking text inside a comment adds no marks.
    let comments = Regex::new(r"(?s)\{>>(?:\\.|[^\\])*?<<\}")
        .unwrap()
        .find_iter(body)
        .filter(|found| !escaped(found.start()))
        .map(|found| found.range())
        .collect::<Vec<_>>();
    let visible = |at: usize| !escaped(at) && !comments.iter().any(|range| range.contains(&at));
    let inline = Regex::new(&format!(r"(\+\+\}}|--\}}|~~\}})[*_`]*\{{{attribution}\}}")).unwrap();
    for cap in inline.captures_iter(body) {
        if !visible(cap.get(0).unwrap().start()) {
            continue;
        }
        counts[match &cap[1] {
            "++}" => 0,
            "--}" => 1,
            _ => 2,
        }] += 1;
    }
    counts[3] += Regex::new(&format!(r"¶\{{{attribution}\}}"))
        .unwrap()
        .find_iter(body)
        .filter(|found| visible(found.start()))
        .count();
    let tag = Regex::new(&format!(r"^{attribution}(?: in [A-Za-z0-9_.-]+)?$")).unwrap();
    for line in body
        .lines()
        .map(str::trim_start)
        .filter(|line| line.starts_with("<!-- "))
    {
        for clause in line.trim_end_matches(" -->").split(", ") {
            let words = clause.split_whitespace().collect::<Vec<_>>();
            let Some(at) = words.iter().position(|word| {
                matches!(
                    *word,
                    "break-ins"
                        | "break-del"
                        | "rev"
                        | "rev-del"
                        | "rev-merge"
                        | "page-break-ins"
                        | "page-break-del"
                )
            }) else {
                continue;
            };
            let key = words[at];
            let rest = words[at + 1..].join(" ");
            // Each semicolon item is one printed tag, even when it joins ids.
            for item in rest.split("; ") {
                if !tag.is_match(item) {
                    continue;
                }
                let kind = if key.starts_with("break-") {
                    3
                } else if item.contains(".cell") {
                    5
                } else if item.contains(".row") {
                    4
                } else {
                    usize::from(key.ends_with("del"))
                };
                counts[kind] += 1;
            }
        }
    }
    counts
}

#[track_caller]
pub fn assert_summary(view: &str) {
    let (_, rest) = view.split_once("---\n").unwrap();
    let (header, body) = rest.split_once("---\n").unwrap();
    let counts = if header
        .lines()
        .any(|line| line.starts_with("view: accept-all") || line.starts_with("view: reject-all"))
    {
        [0; 6]
    } else {
        printed_counts(body)
    };
    let line = header
        .lines()
        .find(|line| line.starts_with("revisions:"))
        .unwrap();
    let mut stated = [0; 6];
    let mut total = 0;
    if let Some((_, summary)) = line.split_once(" shown as ") {
        total = summary
            .split_whitespace()
            .next()
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let kinds = summary
            .split_once(": ")
            .unwrap()
            .1
            .split(';')
            .next()
            .unwrap();
        for entry in kinds.split(", ") {
            let (n, kind) = entry.split_once(' ').unwrap();
            let kind = kind.strip_suffix('s').unwrap_or(kind);
            let at = KINDS.iter().position(|name| *name == kind).unwrap();
            stated[at] = n.parse().unwrap();
        }
    }
    assert_eq!(
        (total, stated),
        (counts.iter().sum::<usize>(), counts),
        "{line}\n{body}"
    );
}

/// In tracked views each substitution represents two raw Word marks.
#[track_caller]
pub fn assert_raw_total(view: &str) {
    let (_, rest) = view.split_once("---\n").unwrap();
    let (header, body) = rest.split_once("---\n").unwrap();
    let line = header
        .lines()
        .find(|line| line.starts_with("revisions:"))
        .unwrap();
    let raw: usize = line.split_whitespace().nth(1).unwrap().parse().unwrap();
    let counts = printed_counts(body);
    assert_eq!(
        raw,
        counts.iter().sum::<usize>() + counts[2],
        "{line}\n{body}"
    );
}
