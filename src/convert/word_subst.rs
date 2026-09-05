// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-substitution evidence table (plan Step 2d).

use super::font_table::{FontFamilyClass, Pitch};

const TABLE: &str = include_str!("word_substitutions.toml");

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
    s.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(s)
        .to_string()
}

fn parse_string_array(s: &str) -> Vec<String> {
    let s = s.trim().trim_start_matches('[').trim_end_matches(']');
    s.split(',')
        .map(str::trim)
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

pub(crate) fn rows() -> Vec<SubstRow> {
    parse_table(TABLE)
}

/// Look up an evidence-table row for `requested` (after comma-split).
/// `*` is the unknown-family last resort and is not returned here.
pub(crate) fn lookup_physical(requested: &str) -> Option<String> {
    let key = normalize(requested);
    rows()
        .into_iter()
        .find(|r| r.key != "*" && r.key == key)
        .map(|r| r.physical)
}

pub(crate) fn unknown_physical() -> String {
    rows()
        .into_iter()
        .find(|r| r.key == "*")
        .map(|r| r.physical)
        .unwrap_or_else(|| "Cambria".into())
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
