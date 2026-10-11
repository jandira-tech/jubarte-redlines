// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-substitution evidence table (plan Step 2d).

use std::sync::LazyLock;

use super::font_table::{FontEntry, FontFamilyClass};

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

/// The catalogue face Word draws an absent font-table entry in (Word 16
/// probes my6/my7, unique names, both orders). Its panose and pitch play
/// no part: a fixed-pitch Courier-like entry is Calibri too.
/// - charset B1/B2 (Hebrew, Arabic) or a Hebrew bit in `w:sig`: Arial;
/// - family roman: Cambria;
/// - any other family, or none: Calibri.
pub(crate) fn entry_generic(entry: &FontEntry) -> &'static str {
    let rtl_charset = entry
        .charset
        .as_deref()
        .is_some_and(|c| c.eq_ignore_ascii_case("B2") || c.eq_ignore_ascii_case("B1"));
    if rtl_charset || entry.usb[0] & USB0_HEBREW != 0 {
        return "Arial";
    }
    match entry.family {
        FontFamilyClass::Roman => "Cambria",
        _ => "Calibri",
    }
}

const USB0_ARMENIAN: u32 = 1 << 10;
const USB0_HEBREW: u32 = 1 << 11;
const USB2_THAANA: u32 = 1 << (72 - 64);

/// The installed face an absent entry's script coverage picks, whatever
/// its family says: a `w:sig` claiming Armenian is Sylfaen (even beside
/// Hebrew and Arabic bits, as Arial's own sig has them), one claiming
/// Thaana is MV Boli (55fcbe9086's Faruma).
pub(crate) fn script_face(entry: &FontEntry) -> Option<&'static str> {
    if entry.usb[0] & USB0_ARMENIAN != 0 {
        Some("Sylfaen")
    } else if entry.usb[2] & USB2_THAANA != 0 {
        Some("MV Boli")
    } else {
        None
    }
}

/// Word's own stand-in for a family it knows by name, ahead of the font
/// table's altName (Word 16 probes my2–my5, fresh sessions): the family
/// itself or the family and a style word ("Myriad Pro Light"), never a
/// PostScript name ("MyriadPro-Regular" is unknown).
pub(crate) fn word_name_substitute(name: &str) -> Option<&'static str> {
    const MAP: &[(&str, &str)] = &[
        ("myriad pro", "Segoe UI"),
        ("proxima nova", "Tahoma"),
        ("futura", "Century Gothic"),
        ("adobe garamond pro", "Garamond"),
        ("adobe caslon pro", "Palatino Linotype"),
    ];
    let lower = name.trim().to_ascii_lowercase();
    let (family, physical) = MAP.iter().find(|(family, _)| {
        lower
            .strip_prefix(family)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
    })?;
    // Only Myriad's Light keeps its weight: Semibold, Bold and Black are
    // all the regular Segoe UI.
    if *family == "myriad pro" && lower.split(' ').any(|word| word == "light") {
        return Some("Segoe UI Light");
    }
    Some(physical)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
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

    fn entry(xml: &str) -> FontEntry {
        let table = super::super::font_table::parse_font_table_xml(&format!(
            r#"<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:font w:name="E">{xml}</w:font></w:fonts>"#
        ));
        table.get("E").expect("entry").clone()
    }

    #[test]
    fn an_absent_entry_is_cambria_when_roman_and_calibri_otherwise() {
        // Word 16 probes my6/my7: panose and pitch play no part.
        for (xml, want) in [
            (
                r#"<w:family w:val="roman"/><w:pitch w:val="fixed"/>"#,
                "Cambria",
            ),
            (
                r#"<w:panose1 w:val="00000000000000000000"/><w:family w:val="roman"/>"#,
                "Cambria",
            ),
            (
                r#"<w:panose1 w:val="020B0604020202020204"/><w:family w:val="swiss"/>"#,
                "Calibri",
            ),
            (
                r#"<w:panose1 w:val="02070309020205020404"/><w:pitch w:val="fixed"/>"#,
                "Calibri",
            ),
            (
                r#"<w:family w:val="modern"/><w:pitch w:val="variable"/>"#,
                "Calibri",
            ),
            (r#"<w:family w:val="decorative"/>"#, "Calibri"),
            ("", "Calibri"),
            (
                r#"<w:charset w:val="B2"/><w:family w:val="roman"/>"#,
                "Arial",
            ),
            (
                r#"<w:family w:val="roman"/><w:sig w:usb0="00000803" w:usb1="0" w:usb2="0" w:usb3="0" w:csb0="21" w:csb1="0"/>"#,
                "Arial",
            ),
        ] {
            assert_eq!(entry_generic(&entry(xml)), want, "{xml}");
        }
    }

    #[test]
    fn a_sig_claiming_armenian_or_thaana_picks_that_script_face() {
        // Word 16 probes my6/my7: Arial's own sig (Armenian, Hebrew and
        // Arabic bits) is Sylfaen; Faruma's Thaana bit is MV Boli, even on
        // a roman entry.
        let sig = |usb0: &str, usb2: &str| {
            format!(
                r#"<w:family w:val="roman"/><w:sig w:usb0="{usb0}" w:usb1="C000785B" w:usb2="{usb2}" w:usb3="0" w:csb0="1FF" w:csb1="0"/>"#
            )
        };
        assert_eq!(script_face(&entry(&sig("E0002EFF", "9"))), Some("Sylfaen"));
        assert_eq!(
            script_face(&entry(&sig("00000003", "100"))),
            Some("MV Boli")
        );
        assert_eq!(script_face(&entry(&sig("00000803", "0"))), None);
        assert_eq!(script_face(&entry(&sig("20000287", "0"))), None);
    }

    #[test]
    fn word_knows_some_absent_families_by_name_only() {
        // Word 16 probes my4/my5, fresh sessions.
        for (name, want) in [
            ("Myriad Pro", Some("Segoe UI")),
            ("myriad pro", Some("Segoe UI")),
            ("Myriad Pro Semibold", Some("Segoe UI")),
            ("Myriad Pro Cond", Some("Segoe UI")),
            ("Myriad Pro Light", Some("Segoe UI Light")),
            ("Proxima Nova Light", Some("Tahoma")),
            ("Futura PT", Some("Century Gothic")),
            ("Futura Bk BT", Some("Century Gothic")),
            ("Adobe Garamond Pro", Some("Garamond")),
            ("Adobe Caslon Pro", Some("Palatino Linotype")),
            ("MyriadPro-Regular", None),
            ("Myriad Web Pro", None),
            ("Myriad", None),
            ("Adobe Garamond", None),
            ("Garamond Pro", None),
            ("Minion Pro", None),
        ] {
            assert_eq!(word_name_substitute(name), want, "{name}");
        }
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
