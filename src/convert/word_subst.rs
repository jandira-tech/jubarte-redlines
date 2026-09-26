// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-substitution evidence table (plan Step 2d).

use std::sync::LazyLock;

use super::font_table::{FontFamilyClass, Pitch};

const TABLE: &str = include_str!("word_substitutions.toml");

/// Parsed once: `lookup_physical` / `unknown_physical` run per resolution.
static ROWS: LazyLock<Vec<SubstRow>> = LazyLock::new(|| parse_table(TABLE));

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SubstRow {
    pub key: String,
    pub physical: String,
    pub stems: Vec<String>,
}

pub(crate) fn parse_table(src: &str) -> Vec<SubstRow> {
    let mut rows = Vec::new();
    let mut key = None;
    let mut physical = None;
    let mut stems = Vec::new();
    let flush = |rows: &mut Vec<SubstRow>,
                 key: &mut Option<String>,
                 physical: &mut Option<String>,
                 stems: &mut Vec<String>| {
        if let (Some(k), Some(p)) = (key.take(), physical.take()) {
            rows.push(SubstRow {
                key: k,
                physical: p,
                stems: std::mem::take(stems),
            });
        } else {
            stems.clear();
        }
    };
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[row]]" {
            flush(&mut rows, &mut key, &mut physical, &mut stems);
            continue;
        }
        if let Some(rest) = line.strip_prefix("key = ") {
            key = Some(unquote(rest));
        } else if let Some(rest) = line.strip_prefix("physical = ") {
            physical = Some(unquote(rest));
        } else if let Some(rest) = line.strip_prefix("stems = ") {
            stems = parse_string_array(rest);
        }
    }
    flush(&mut rows, &mut key, &mut physical, &mut stems);
    rows
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    match s.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        Some(inner) => unescape(inner),
        None => s.to_string(),
    }
}

/// TOML basic-string escapes the table uses (`\"`, `\\`).
fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// A one-line TOML array of basic strings. Commas and escaped quotes
/// inside a string belong to it (the `*` row's stem has both).
fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim().trim_start_matches('[').trim_end_matches(']');
    let mut items = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for c in s.chars() {
        if in_string {
            current.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == ',' {
            items.push(std::mem::take(&mut current));
        } else {
            if c == '"' {
                in_string = true;
            }
            current.push(c);
        }
    }
    items.push(current);
    items
        .iter()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .map(unquote)
        .collect()
}

fn normalize(name: &str) -> String {
    name.trim()
        .to_ascii_lowercase()
        .replace([' ', '-'], "")
        .replace("mt", "")
}

pub(crate) fn rows() -> &'static [SubstRow] {
    &ROWS
}

/// Look up an evidence-table row for `requested` (after comma-split).
/// `*` is the unknown-family last resort and is not returned here.
pub(crate) fn lookup_physical(requested: &str) -> Option<String> {
    let key = normalize(requested);
    rows()
        .iter()
        .find(|r| r.key != "*" && r.key == key)
        .map(|r| r.physical.clone())
}

pub(crate) fn unknown_physical() -> String {
    rows()
        .iter()
        .find(|r| r.key == "*")
        .map_or_else(|| "Cambria".into(), |r| r.physical.clone())
}

pub(crate) fn generic_physical(family: FontFamilyClass, pitch: Pitch) -> &'static str {
    if matches!(pitch, Pitch::Fixed) {
        return "Courier New";
    }
    match family {
        FontFamilyClass::Roman => "Times New Roman",
        FontFamilyClass::Swiss => "Arial",
        FontFamilyClass::Modern => "Courier New",
        FontFamilyClass::Script | FontFamilyClass::Decorative | FontFamilyClass::Auto => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stems_keep_commas_and_escaped_quotes_inside_a_string() {
        let row = rows().iter().find(|r| r.key == "*").expect("* row");
        assert_eq!(
            row.stems,
            vec![
                "quoted CSS list \"Times New Roman\", Times, serif; unknown family without altName"
                    .to_string()
            ]
        );
        assert_eq!(
            parse_string_array(r#"["a, b", "c\\d", e]"#),
            vec!["a, b".to_string(), "c\\d".to_string(), "e".to_string()]
        );
    }

    #[test]
    fn table_contains_required_rows() {
        let rows = rows();
        let keys: Vec<&str> = rows.iter().map(|r| r.key.as_str()).collect();
        assert!(keys.contains(&"inter"));
        assert!(keys.contains(&"dejavusansmono"));
        assert!(keys.contains(&"liberationserif"));
        assert!(keys.contains(&""));
        assert!(keys.contains(&"widelatin"));
        assert!(keys.contains(&"*"));
        assert_eq!(lookup_physical("Inter").as_deref(), Some("Cambria"));
        assert_eq!(
            lookup_physical("DejaVu Sans Mono").as_deref(),
            Some("Verdana")
        );
        assert_eq!(unknown_physical(), "Cambria");
    }
}
