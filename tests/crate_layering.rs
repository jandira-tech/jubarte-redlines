// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The workspace dependency DAG the split produced (plan A1, T4.9).
//!
//! Nine packages: the facade at the root and eight library crates under
//! `crates/`. Each `[dependencies]` and `[dev-dependencies]` edge between
//! workspace crates must point downward in the DAG, and the compare and
//! render products must never name each other. The test reads every
//! `crates/*/Cargo.toml` and the root manifest (section by section, not
//! regex) and asserts each edge is allowed.
//!
//! The crate-path scanner below stays: its unit tests protect the
//! `crate::` path-collection machinery `jubarte debug` and the historical
//! evidence rely on, and any future module move reuses it.

use std::collections::BTreeSet;
use std::path::Path;

/// The DAG, lowest first; an edge may point from any crate to one at a
/// smaller index. The compare and render products are peers: neither
/// names the other.
const LAYERS: [&str; 8] = [
    "jubarte-dom",
    "jubarte-opc",
    "jubarte-redlines",
    "jubarte-render",
    "jubarte-doc",
    "jubarte-markdown",
    "jubarte-edit",
    "jubarte-cli",
];

fn rank(layer: &str) -> usize {
    LAYERS
        .iter()
        .position(|known| *known == layer)
        .unwrap_or_else(|| panic!("unknown layer {layer}"))
}

/// Whether `from` may depend on `to`.
fn allowed(from: &str, to: &str) -> bool {
    if from == "jubarte" {
        return true; // the facade names everything
    }
    if from == to {
        return true;
    }
    // The compare and render products are peers: neither names the other.
    if matches!(
        (from, to),
        ("jubarte-redlines", "jubarte-render") | ("jubarte-render", "jubarte-redlines")
    ) {
        return false;
    }
    rank(to) < rank(from)
}

fn is_ident(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Every leading `ident(::ident)*` segment of `text`, and what follows them.
fn path_segments(text: &str) -> (Vec<&str>, &str) {
    let mut segments = Vec::new();
    let mut rest = skip_trivia(text);
    loop {
        let segment = rest.strip_prefix("r#").unwrap_or(rest);
        let end = segment.bytes().take_while(|byte| is_ident(*byte)).count();
        if end == 0 {
            return (segments, rest);
        }
        segments.push(&segment[..end]);
        let after = skip_trivia(&segment[end..]);
        match after.strip_prefix("::") {
            Some(next) => rest = skip_trivia(next),
            None => return (segments, &segment[end..]),
        }
    }
}

/// Every path named after `crate` and its `::`, expanding `crate::{a, b::c}`
/// groups. Rust allows whitespace and comments around the first `::` too.
fn crate_paths(text: &str) -> Vec<Vec<&str>> {
    let mut paths = Vec::new();
    let mut from = 0;
    while let Some(found) = text[from..].find("crate") {
        let start = from + found;
        from = start + "crate".len();
        if start > 0 && is_ident(text.as_bytes()[start - 1]) {
            continue;
        }
        let Some(rest) = skip_trivia(&text[from..]).strip_prefix("::") else {
            continue;
        };
        collect_path(rest, &[], &mut paths);
    }
    paths
}

fn skip_trivia(mut s: &str) -> &str {
    loop {
        s = s.trim_start();
        match comment_len(s) {
            Some(len) => s = &s[len..],
            None => return s,
        }
    }
}

/// The comment `s` opens with, or `None`.
fn comment_len(s: &str) -> Option<usize> {
    if s.starts_with("//") {
        return Some(s.find('\n').map_or(s.len(), |newline| newline + 1));
    }
    if !s.starts_with("/*") {
        return None;
    }
    let mut depth = 1usize;
    let mut i = 2;
    let bytes = s.as_bytes();
    while i + 1 < bytes.len() && depth > 0 {
        if bytes[i] == b'/' && bytes[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
            depth -= 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    Some(if depth == 0 { i } else { s.len() })
}

fn collect_path<'a>(rest: &'a str, prefix: &[&'a str], paths: &mut Vec<Vec<&'a str>>) {
    if let Some(group) = skip_trivia(rest).strip_prefix('{') {
        collect_group(group, prefix, paths);
        return;
    }
    let (segments, remainder) = path_segments(rest);
    if let Some(group) = skip_trivia(remainder).strip_prefix('{') {
        collect_group(group, &([prefix, &segments[..]].concat()), paths);
        return;
    }
    // `self` in a group names the prefix itself.
    let leaf: Vec<&str> = prefix
        .iter()
        .copied()
        .chain(segments)
        .filter(|segment| *segment != "self")
        .take(2)
        .collect();
    if !leaf.is_empty() {
        paths.push(leaf);
    }
}

fn collect_group<'a>(group: &'a str, prefix: &[&'a str], paths: &mut Vec<Vec<&'a str>>) {
    let mut depth = 1usize;
    let mut item_start = 0;
    let mut index = 0;
    while index < group.len() {
        if group.as_bytes()[index] == b'/'
            && let Some(len) = comment_len(&group[index..])
        {
            index += len;
            continue;
        }
        match group.as_bytes()[index] {
            b'{' => depth += 1,
            b'}' | b',' if depth == 1 => {
                let item = group[item_start..index].trim();
                if !item.is_empty() {
                    collect_path(item, prefix, paths);
                }
                item_start = index + 1;
                if group.as_bytes()[index] == b'}' {
                    break;
                }
            }
            b'}' => depth -= 1,
            _ => {}
        }
        index += 1;
    }
}

/// One `[dependencies]`-style table body: the crate names it lists.
fn dependency_names(table: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in table.lines() {
        let line = line.trim();
        if !line.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
            continue;
        }
        let Some(name) = line.split('=').next().map(str::trim) else {
            continue;
        };
        if !name.is_empty() {
            names.push(name.to_owned());
        }
    }
    names
}

/// Every workspace crate each manifest names as a dependency or
/// dev-dependency, as `(from, to)` edges.
fn manifest_edges(root: &Path) -> BTreeSet<(String, String)> {
    let mut manifests = vec![root.join("Cargo.toml")];
    let mut crates = vec![];
    for entry in
        std::fs::read_dir(root.join("crates")).unwrap_or_else(|e| panic!("read crates/: {e}"))
    {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            crates.push(path.join("Cargo.toml"));
        }
    }
    crates.sort();
    manifests.extend(crates);
    let mut edges = BTreeSet::new();
    for manifest in manifests {
        let text = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|e| panic!("read {}: {e}", manifest.display()));
        let from = text
            .split("name = \"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_else(|| panic!("{} names no package", manifest.display()))
            .to_owned();
        for section in ["[dependencies]", "[dev-dependencies]"] {
            let Some(start) = text.find(section) else {
                continue;
            };
            let body = &text[start + section.len()..];
            let end = body.find("\n[").map(|at| &body[..at]).unwrap_or(body);
            for to in dependency_names(end) {
                if LAYERS.contains(&to.as_str()) {
                    edges.insert((from.clone(), to));
                }
            }
        }
    }
    edges
}

#[test]
fn every_workspace_dependency_edge_points_down() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let found = manifest_edges(root);
    assert!(
        !found.is_empty(),
        "no workspace dependency edges found; the manifest scan broke"
    );
    let violations: Vec<String> = found
        .iter()
        .filter(|(from, to)| !allowed(from, to))
        .map(|(from, to)| format!("{from} -> {to}"))
        .collect();
    assert!(
        violations.is_empty(),
        "upward or peer-product workspace dependencies:\n{violations:#?}\n\
         the DAG is {LAYERS:?} (lowest first)"
    );
}

#[test]
fn crate_paths_expand_groups_and_skip_longer_idents() {
    let text = "use crate::{a::b::c, d, e::{f, g}};\nlet x = mycrate::z; crate::h::i();";
    let paths = crate_paths(text);
    assert_eq!(
        paths,
        vec![
            vec!["a", "b"],
            vec!["d"],
            vec!["e", "f"],
            vec!["e", "g"],
            vec!["h", "i"],
        ]
    );
}

#[test]
fn nested_groups_reach_submodule_overrides() {
    for text in [
        "use crate::markdown::{redline, patch};",
        "use crate::{markdown::{redline, unified}};",
        "use crate::markdown::redline::run;",
    ] {
        let paths = crate_paths(text);
        assert!(
            paths.contains(&vec!["markdown", "redline"]),
            "{text}: {paths:?}"
        );
    }
}

#[test]
fn nested_group_regressions_match_direct_imports() {
    for text in [
        "crate::markdown::{redline}",
        "crate::{markdown::{redline}}",
        "crate::markdown::redline",
    ] {
        let paths = crate_paths(text);
        assert_eq!(paths, vec![vec!["markdown", "redline"]], "{text}");
    }
}

#[test]
fn raw_identifier_segments_match_plain_segments() {
    let plain = crate_paths("crate::markdown::patch");
    let raw = crate_paths("crate::markdown::r#patch");
    assert_eq!(raw, plain);
}

#[test]
fn whitespace_around_separators_keeps_full_path() {
    for text in [
        "use crate::{markdown:: redline::{self}};",
        "use crate::markdown:: redline::{self};",
        "use crate::markdown :: redline;",
        "use crate::markdown:: redline;",
        "use crate:: markdown::redline;",
        "use crate :: markdown::redline;",
        "use crate\n\t::markdown::redline;",
        "use crate ::{markdown::redline};",
    ] {
        let paths = crate_paths(text);
        assert_eq!(paths, vec![vec!["markdown", "redline"]], "{text}");
    }
}

#[test]
fn comments_are_whitespace_inside_paths_and_groups() {
    for text in [
        "use crate::markdown/*{,}*/::redline;",
        "use crate::{markdown/*{,}*/::redline};",
        "use crate::{/* } , */ markdown::redline};",
        "use crate::{markdown::{/*{*/redline/*}*/}};",
        "use crate::markdown::redline; // trailing",
        "use crate/*c*/::markdown::redline;",
        "use crate // note\n::markdown::redline;",
        "use crate /* } , */ :: markdown::redline;",
    ] {
        let paths = crate_paths(text);
        assert!(
            paths.contains(&vec!["markdown", "redline"]),
            "{text}: {paths:?}"
        );
    }
}

#[test]
fn crate_paths_expand_nested_groups_past_the_first_segment() {
    // An override such as markdown::redline must be seen in grouped form,
    // or an upward path written as a group slips past the guard.
    let text = "use crate::markdown::{redline, patch};\n\
                use crate::{edit::{inline as i, self}, ids};";
    assert_eq!(
        crate_paths(text),
        vec![
            vec!["markdown", "redline"],
            vec!["markdown", "patch"],
            vec!["edit", "inline"],
            vec!["edit"],
            vec!["ids"],
        ]
    );
}

#[test]
fn a_group_inside_a_group_keeps_its_outer_prefix_first() {
    for text in [
        "use crate::markdown::{redline::{run}};",
        "use crate::{markdown::{redline::{run, Opts}}};",
    ] {
        let paths = crate_paths(text);
        assert!(
            paths
                .iter()
                .all(|path| path == &vec!["markdown", "redline"]),
            "{text}: {paths:?}"
        );
    }
}
