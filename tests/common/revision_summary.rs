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
/// paragraph marks and table changes that have no inline delimiter. A change
/// of several marks prints its joined id on each piece and counts once, as
/// the kind its pieces make together (#1261).
pub fn printed_counts(body: &str) -> [usize; 6] {
    let mut counts = [0; 6];
    for change in printed_changes(body) {
        counts[change.kind] += 1;
    }
    counts
}

/// One change the body prints: its kind, an index into [`KINDS`], and the
/// raw Word marks it stands for.
pub struct Printed {
    pub kind: usize,
    pub marks: usize,
}

/// The kind of one change whose pieces print as `was` and `kind`: a deletion
/// with an insertion is a substitution, a paragraph mark goes with its text.
fn joined(was: usize, kind: usize) -> usize {
    match (was, kind) {
        (a, b) if a == b => a,
        (2, _) | (_, 2) | (0, 1) | (1, 0) => 2,
        (3, other) | (other, 3) => other,
        (a, _) => a,
    }
}

/// The printed changes in reading order, each change of several marks once.
pub fn printed_changes(body: &str) -> Vec<Printed> {
    let joined_id = Regex::new(r"^(?:[A-Za-z0-9_-]+\.)?r[0-9]+(?:\+[0-9]+)+$").unwrap();
    let mut seen: Vec<(String, usize)> = Vec::new();
    let mut out: Vec<(Option<String>, usize)> = Vec::new();
    for (id, kind) in printed_tags(body) {
        if joined_id.is_match(&id) {
            if let Some(at) = seen.iter().position(|(other, _)| *other == id) {
                let slot = seen[at].1;
                out[slot].1 = joined(out[slot].1, kind);
                continue;
            }
            seen.push((id.clone(), out.len()));
            out.push((Some(id), kind));
        } else {
            out.push((None, kind));
        }
    }
    out.into_iter()
        .map(|(id, kind)| {
            let ids = id.map_or(1, |id| id.matches('+').count() + 1);
            Printed {
                kind,
                marks: ids.max(if kind == 2 { 2 } else { 1 }),
            }
        })
        .collect()
}

/// Every complete attributed tag the body prints: its own id and its kind.
fn printed_tags(body: &str) -> Vec<(String, usize)> {
    // The header reports note stories separately, including a note inserted
    // with its body reference (whose attribution is repeated in its definition).
    let body = body
        .split("<!-- footnotes")
        .next()
        .unwrap()
        .split("<!-- endnotes")
        .next()
        .unwrap();
    let mut tags = Vec::new();
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
    let own = |attribution: &str| attribution.split_whitespace().next().unwrap().to_string();
    let inline = Regex::new(&format!(
        r"(\+\+\}}|--\}}|~~\}})[*_`]*\{{({attribution})\}}"
    ))
    .unwrap();
    for cap in inline.captures_iter(body) {
        if !visible(cap.get(0).unwrap().start()) {
            continue;
        }
        let kind = match &cap[1] {
            "++}" => 0,
            "--}" => 1,
            _ => 2,
        };
        tags.push((own(&cap[2]), kind));
    }
    for cap in Regex::new(&format!(r"¶\{{({attribution})\}}"))
        .unwrap()
        .captures_iter(body)
    {
        if visible(cap.get(0).unwrap().start()) {
            tags.push((own(&cap[1]), 3));
        }
    }
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
                tags.push((own(item), kind));
            }
        }
    }
    tags
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

/// In tracked views each substitution represents two raw Word marks, and a
/// change printed with a joined id one mark per id it joins.
#[track_caller]
pub fn assert_raw_total(view: &str) {
    let (_, rest) = view.split_once("---\n").unwrap();
    let (header, body) = rest.split_once("---\n").unwrap();
    let line = header
        .lines()
        .find(|line| line.starts_with("revisions:"))
        .unwrap();
    let raw: usize = line.split_whitespace().nth(1).unwrap().parse().unwrap();
    let marks = printed_changes(body)
        .iter()
        .map(|change| change.marks)
        .sum::<usize>();
    assert_eq!(raw, marks, "{line}\n{body}");
}
