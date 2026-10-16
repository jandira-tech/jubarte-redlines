// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A header's or footer's paragraph frame (`w:framePr`): the classic
//! right-aligned PAGE frame the document's section breaks and its page
//! numbers use, written back to a docx that round-trips. A frame prints
//! as a `frame:` key in the paragraph-by-paragraph grammar:
//!
//! ```yaml
//! p0: {text: "{PAGE}1{/}", frame: "(hAnchor=margin,vAnchor=text,wrap=none,xAlign=right,y=1)"}
//! ```
//!
//! The dialect sorts the attributes alphabetically (`hAnchor`, `vAnchor`,
//! `wrap`, `xAlign`, `y`); the writer places them in schema order
//! (`w`, `h`, `hRule`, `hAnchor`, `vAnchor`, `x`, `y`, `xAlign`,
//! `yAlign`, `wrap`, `hSpace`, `vSpace`) so `Word` keeps the order it
//! writes. `jubarte debug A B --check render` sorts the same way the
//! dialect does, so the gate test
//! (`tests/md_fixed_point.rs::markdown_and_docx_round_trip_exactly`)
//! treats both sides as equal.

/// The schema order of `w:framePr` attributes: `w`, `h`, `hRule`,
/// `hAnchor`, `vAnchor`, `x`, `y`, `xAlign`, `yAlign`, `wrap`, `hSpace`,
/// `vSpace`. Unknown attributes follow the known ones in their own
/// alphabetical order, after the known ones.
pub(crate) const SCHEMA: [&str; 12] = [
    "w", "h", "hRule", "hAnchor", "vAnchor", "x", "y", "xAlign", "yAlign", "wrap", "hSpace",
    "vSpace",
];

/// A header's or footer's paragraph frame: a list of `w:framePr`
/// attributes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Frame {
    pub attrs: Vec<(String, String)>,
}

impl Frame {
    /// The frame from `attrs` (a paragraph's own attributes plus its
    /// style chain's), held in alphabetical order: the dialect's
    /// canonical form, and what [`Self::format`] prints.
    pub(crate) fn from_attrs(attrs: Vec<(String, String)>) -> Self {
        Self {
            attrs: alpha(attrs),
        }
    }

    /// The view's `frame: "(hAnchor=margin,vAnchor=text,wrap=none,xAlign=right,y=1)"`
    /// read back. `text` may include a leading or trailing space; an
    /// empty text is `None`; a malformed pair (`hAnchor:`) is `None`.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let inner = text.trim();
        let inner = inner.strip_prefix('(').and_then(|s| s.strip_suffix(')'))?;
        let inner = inner.trim();
        let mut attrs = Vec::new();
        for pair in inner.split(',') {
            let pair = pair.trim();
            if pair.is_empty() {
                continue;
            }
            let (name, value) = pair.split_once('=')?;
            if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
                return None;
            }
            attrs.push((name.to_string(), value.to_string()));
        }
        if attrs.is_empty() {
            return None;
        }
        Some(Self {
            attrs: alpha(attrs),
        })
    }

    /// The view's grammar for the frame: `(hAnchor=margin,vAnchor=text,wrap=none,xAlign=right,y=1)`,
    /// attributes in alphabetical order (`hAnchor`, `vAnchor`, `wrap`,
    /// `xAlign`, `y`).
    pub(crate) fn format(&self) -> String {
        let pairs: Vec<String> = self
            .attrs
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        format!("({})", pairs.join(","))
    }

    /// The frame's `<w:framePr .../>` element: the attributes in schema
    /// order, escaped as XML strings.
    pub(crate) fn xml(&self) -> String {
        let attrs = schema(self.attrs.clone());
        let mut out = String::from("<w:framePr");
        for (name, value) in &attrs {
            out.push_str(&format!(
                " w:{name}=\"{}\"",
                crate::markdown::xml::escape(value)
            ));
        }
        out.push_str("/>");
        out
    }
}

/// `attrs` in alphabetical order (the dialect's order).
fn alpha(mut attrs: Vec<(String, String)>) -> Vec<(String, String)> {
    attrs.sort_by(|a, b| a.0.cmp(&b.0));
    attrs
}

/// `attrs` in [`SCHEMA`] order, unknown attributes last and alphabetically
/// among themselves. The writer's order.
fn schema(mut attrs: Vec<(String, String)>) -> Vec<(String, String)> {
    attrs.sort_by(|(a, _), (b, _)| {
        match (
            SCHEMA.iter().position(|n| n == a),
            SCHEMA.iter().position(|n| n == b),
        ) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.cmp(b),
        }
    });
    attrs
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn from_attrs_orders_alphabetically() {
        let frame = Frame::from_attrs(vec![
            ("wrap".into(), "none".into()),
            ("xAlign".into(), "right".into()),
            ("hAnchor".into(), "margin".into()),
            ("y".into(), "1".into()),
            ("vAnchor".into(), "text".into()),
            ("custom".into(), "kept".into()),
        ]);
        let names: Vec<&str> = frame.attrs.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(
            names,
            ["custom", "hAnchor", "vAnchor", "wrap", "xAlign", "y"]
        );
    }

    #[test]
    fn parse_reads_the_dialect_in_alphabetical_order() {
        let text = "(hAnchor=margin,vAnchor=text,wrap=none,xAlign=right,y=1)";
        let frame = Frame::parse(text).expect("reads");
        let names: Vec<&str> = frame.attrs.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["hAnchor", "vAnchor", "wrap", "xAlign", "y"]);
        assert_eq!(frame.format(), text);
    }

    #[test]
    fn format_prints_the_attrs_in_them() {
        let frame = Frame::from_attrs(vec![
            ("wrap".into(), "none".into()),
            ("y".into(), "1".into()),
            ("xAlign".into(), "right".into()),
            ("vAnchor".into(), "text".into()),
            ("hAnchor".into(), "margin".into()),
        ]);
        assert_eq!(
            frame.format(),
            "(hAnchor=margin,vAnchor=text,wrap=none,xAlign=right,y=1)"
        );
    }

    #[test]
    fn parse_rejects_malformed_text() {
        for bad in [
            "hAnchor=margin",
            "(hAnchor)",
            "(hAnchor,vAnchor=text)",
            "(=value)",
            "(hAnchor=)",
            "()",
            "(hAnchor=margin,vAnchor=text,)",
        ] {
            assert!(Frame::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn xml_writes_schema_ordered_attributes() {
        let frame = Frame::parse("(hAnchor=margin,vAnchor=text,wrap=none,xAlign=right,y=1)")
            .expect("reads");
        assert_eq!(
            frame.xml(),
            r#"<w:framePr w:y="1" w:hAnchor="margin" w:vAnchor="text" w:xAlign="right" w:wrap="none"/>"#
        );
    }
}
