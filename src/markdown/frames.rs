// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A paragraph's frame (`w:framePr`, CT_FramePr) in the view and back:
//! a header or footer paragraph's `frame: {wrap: none, hAnchor: margin,
//! vAnchor: text, xAlign: right, y: 1}`, the classic page-number frame.
//!
//! The keys are the schema's attribute names and the values its own,
//! lengths in twips as the part writes them, so the frame comes back
//! attribute for attribute. The reader prints the frame the paragraph
//! lays out in (its style chain's, under its own); the writer writes it
//! on the paragraph.

use std::fmt::Write as _;

/// CT_FramePr's attributes, in the schema's order.
pub(crate) const ATTRIBUTES: [&str; 15] = [
    "dropCap",
    "lines",
    "w",
    "h",
    "vSpace",
    "hSpace",
    "wrap",
    "hAnchor",
    "vAnchor",
    "x",
    "xAlign",
    "y",
    "yAlign",
    "hRule",
    "anchorLock",
];

/// Whether `value` is one the schema allows for attribute `name`.
pub(crate) fn valid(name: &str, value: &str) -> bool {
    let one_of = |words: &[&str]| words.contains(&value);
    match name {
        "dropCap" => one_of(&["none", "drop", "margin"]),
        "lines" => value.parse::<i64>().is_ok(),
        "w" | "h" | "vSpace" | "hSpace" => measure(value, false),
        "x" | "y" => measure(value, true),
        "wrap" => one_of(&["auto", "notBeside", "around", "tight", "through", "none"]),
        "hAnchor" | "vAnchor" => one_of(&["text", "margin", "page"]),
        "xAlign" => one_of(&["left", "center", "right", "inside", "outside"]),
        "yAlign" => one_of(&["inline", "top", "center", "bottom", "inside", "outside"]),
        "hRule" => one_of(&["auto", "exact", "atLeast"]),
        "anchorLock" => one_of(&["true", "false", "on", "off", "1", "0"]),
        _ => false,
    }
}

/// A twips measure: whole twips, or a universal measure (`12pt`, `1.5cm`),
/// signed when `signed`.
fn measure(value: &str, signed: bool) -> bool {
    let unsigned = match value.strip_prefix('-') {
        Some(rest) if signed => rest,
        Some(_) => return false,
        None => value,
    };
    if !unsigned.is_empty() && unsigned.bytes().all(|b| b.is_ascii_digit()) {
        return true;
    }
    ["mm", "cm", "in", "pt", "pc", "pi"].iter().any(|unit| {
        unsigned.strip_suffix(unit).is_some_and(|number| {
            let mut parts = number.splitn(2, '.');
            let whole = parts.next().unwrap_or("");
            let fraction = parts.next();
            !whole.is_empty()
                && whole.bytes().all(|b| b.is_ascii_digit())
                && fraction.is_none_or(|f| !f.is_empty() && f.bytes().all(|b| b.is_ascii_digit()))
        })
    })
}

/// The frame of a paragraph: `chain`, a style chain's frames from its
/// base style to the paragraph's own style, then `own`, the paragraph's
/// `w:framePr`, each overlaying the attributes before it. `None` when no
/// one sets a frame; an empty frame when only an empty `w:framePr` does.
/// Only the schema's attributes count, with values it allows.
pub(crate) fn merged<'a>(
    chain: impl IntoIterator<Item = &'a [(String, String)]>,
    own: Option<&'a [(String, String)]>,
) -> Option<Vec<(String, String)>> {
    let mut frame: Option<Vec<(String, String)>> = None;
    for attrs in chain.into_iter().chain(own) {
        let frame = frame.get_or_insert_with(Vec::new);
        for (name, value) in attrs {
            let local = name.rsplit(':').next().unwrap_or(name);
            if !valid(local, value) {
                continue;
            }
            match frame.iter_mut().find(|(n, _)| n == local) {
                Some(slot) => slot.1.clone_from(value),
                None => frame.push((local.to_string(), value.clone())),
            }
        }
    }
    let mut frame = frame?;
    frame.sort_by_key(|(name, _)| ATTRIBUTES.iter().position(|a| a == name));
    Some(frame)
}

/// `{wrap: none, hAnchor: margin, y: 1}`, in the schema's order; `{}`
/// for an empty frame. Every value [`valid`] allows is a plain YAML word.
pub(crate) fn describe(frame: &[(String, String)]) -> String {
    let items: Vec<String> = frame
        .iter()
        .map(|(name, value)| format!("{name}: {value}"))
        .collect();
    format!("{{{}}}", items.join(", "))
}

/// `<w:framePr w:wrap="none" …/>`, attributes in the schema's order.
pub(crate) fn xml(frame: &[(String, String)]) -> String {
    let mut out = String::from("<w:framePr");
    for name in ATTRIBUTES {
        if let Some((_, value)) = frame.iter().find(|(n, _)| n == name) {
            let _ = write!(out, " w:{name}=\"{}\"", super::xml::escape(value));
        }
    }
    out.push_str("/>");
    out
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
        items
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn a_frame_reads_in_the_schemas_order_and_writes_back() {
        let own = pairs(&[
            ("w:y", "1"),
            ("w:wrap", "none"),
            ("w:vAnchor", "text"),
            ("w:hAnchor", "margin"),
            ("w:xAlign", "right"),
        ]);
        let frame = merged([], Some(own.as_slice())).unwrap();
        assert_eq!(
            describe(&frame),
            "{wrap: none, hAnchor: margin, vAnchor: text, xAlign: right, y: 1}"
        );
        assert_eq!(
            xml(&frame),
            "<w:framePr w:wrap=\"none\" w:hAnchor=\"margin\" w:vAnchor=\"text\" w:xAlign=\"right\" w:y=\"1\"/>"
        );
    }

    #[test]
    fn the_paragraph_s_own_frame_overlays_its_style_s() {
        let base = pairs(&[("w:h", "363"), ("w:wrap", "around")]);
        let style = pairs(&[("w:wrap", "notBeside")]);
        let own = pairs(&[("w:wrap", "none"), ("w:x", "-20")]);
        let frame = merged([base.as_slice(), style.as_slice()], Some(own.as_slice())).unwrap();
        assert_eq!(describe(&frame), "{h: 363, wrap: none, x: -20}");
        // A style's frame alone frames the paragraph.
        assert_eq!(
            merged([style.as_slice()], None).map(|f| describe(&f)),
            Some("{wrap: notBeside}".to_string())
        );
        // An empty framePr is a frame; none is none.
        assert_eq!(
            merged([], Some(&[][..])).map(|f| describe(&f)),
            Some("{}".to_string())
        );
        assert_eq!(merged([], None), None);
        // An attribute the schema does not name, or a value it does not
        // allow, is no part of it.
        let odd = pairs(&[("w14:foo", "1"), ("w:y", "2"), ("w:wrap", "sideways")]);
        assert_eq!(
            merged([], Some(odd.as_slice())).map(|f| describe(&f)),
            Some("{y: 2}".to_string())
        );
    }

    #[test]
    fn a_value_reads_only_as_its_attribute_allows() {
        for (name, value) in [
            ("wrap", "none"),
            ("hAnchor", "page"),
            ("x", "-20"),
            ("y", "1"),
            ("w", "2880"),
            ("h", "12.5pt"),
            ("x", "-1.5cm"),
            ("lines", "3"),
            ("anchorLock", "1"),
            ("yAlign", "inline"),
            ("hRule", "atLeast"),
            ("dropCap", "drop"),
        ] {
            assert!(valid(name, value), "{name}={value}");
        }
        for (name, value) in [
            ("wrap", "sideways"),
            ("w", "-5"),
            ("w", "5px"),
            ("x", "1.cm"),
            ("y", ""),
            ("lines", "two"),
            ("color", "red"),
            ("anchorLock", "yes"),
        ] {
            assert!(!valid(name, value), "{name}={value}");
        }
    }
}
