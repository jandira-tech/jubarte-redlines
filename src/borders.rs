// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Table, cell and paragraph borders as a view prints them, read back, and
//! as the writer writes them: the borders Word passes come back as they were
//! (Arthur, 2026-10-15: "whatever word passes, we render").
//!
//! A border reads `single 0.5pt 000000`: Word's `w:val`, its width in points
//! (`w:sz` is eighths of a point), its colour when it is not automatic, its
//! theme colour after that (`theme accent1 tint 99 shade BF`, Word's
//! `w:themeColor`, `w:themeTint` and `w:themeShade`), Word's border
//! effects (`shadow`, `frame`: `w:shadow`, `w:frame`), and `space 1pt` when
//! it keeps a gap. Sides that share a border print as one
//! group (`top,bottom single 1.5pt`), groups joined by ` + `; a table's group
//! naming all six of its sides drops their names (`borders single 0.5pt`).
//! A side Word turns off (`nil`, `none`) is `none`. `start` and `end` are
//! read as `left` and `right`.

/// A table's sides (`w:tblBorders`), in schema order.
pub(crate) const TABLE_SIDES: [&str; 6] = ["top", "left", "bottom", "right", "insideH", "insideV"];

/// A cell's sides (`w:tcBorders`), in schema order.
pub(crate) const CELL_SIDES: [&str; 8] = [
    "top", "left", "bottom", "right", "insideH", "insideV", "tl2br", "tr2bl",
];

/// A paragraph's sides (`w:pBdr`), in schema order.
pub(crate) const PARAGRAPH_SIDES: [&str; 6] = ["top", "left", "bottom", "right", "between", "bar"];

/// One side's border.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Border {
    /// Word's `w:val` (`single`, `double`, `dashed`, …); `none` for a side
    /// turned off.
    pub val: String,
    /// `w:sz`, in eighths of a point.
    pub size: Option<u32>,
    /// `w:color`, when not automatic.
    pub color: Option<String>,
    /// `w:space`, in points, when not zero.
    pub space: Option<u32>,
    /// `w:themeColor` (`accent1`), and its `w:themeTint` and
    /// `w:themeShade` (two hex digits), beside the fallback `color`.
    pub theme: Option<String>,
    pub tint: Option<String>,
    pub shade: Option<String>,
    /// `w:shadow` and `w:frame` on.
    pub shadow: bool,
    pub frame: bool,
}

impl Border {
    /// A border from its element's attributes.
    pub(crate) fn new(
        val: &str,
        size: Option<&str>,
        color: Option<&str>,
        space: Option<&str>,
    ) -> Self {
        let off = matches!(val, "nil" | "none");
        Self {
            val: if off { "none".into() } else { val.into() },
            size: size.and_then(|s| s.parse().ok()).filter(|_| !off),
            color: color
                .filter(|c| !off && !c.eq_ignore_ascii_case("auto"))
                .map(str::to_string),
            space: space
                .and_then(|s| s.parse().ok())
                .filter(|&s| !off && s > 0),
            theme: None,
            tint: None,
            shade: None,
            shadow: false,
            frame: false,
        }
    }

    /// The border with its element's theme colour attributes.
    pub(crate) fn with_theme(
        mut self,
        theme: Option<&str>,
        tint: Option<&str>,
        shade: Option<&str>,
    ) -> Self {
        if self.val == "none" {
            return self;
        }
        self.theme = theme.filter(|t| is_theme_name(t)).map(str::to_string);
        self.tint = tint.filter(|t| is_hex_byte(t)).map(str::to_string);
        self.shade = shade.filter(|t| is_hex_byte(t)).map(str::to_string);
        self
    }

    /// The border with its element's `w:shadow` and `w:frame`.
    pub(crate) fn with_effects(mut self, shadow: Option<&str>, frame: Option<&str>) -> Self {
        if self.val == "none" {
            return self;
        }
        let on = |v: Option<&str>| matches!(v, Some("1" | "true" | "on"));
        self.shadow = on(shadow);
        self.frame = on(frame);
        self
    }

    fn text(&self) -> String {
        let mut text = self.val.clone();
        if let Some(size) = self.size {
            text.push(' ');
            text.push_str(&points(size));
            text.push_str("pt");
        }
        if let Some(color) = &self.color {
            text.push(' ');
            text.push_str(color);
        }
        for (word, value) in [
            ("theme", &self.theme),
            ("tint", &self.tint),
            ("shade", &self.shade),
        ] {
            if let Some(value) = value {
                text.push_str(&format!(" {word} {value}"));
            }
        }
        for (word, on) in [("shadow", self.shadow), ("frame", self.frame)] {
            if on {
                text.push(' ');
                text.push_str(word);
            }
        }
        if let Some(space) = self.space {
            text.push_str(&format!(" space {space}pt"));
        }
        text
    }

    fn parse(words: &[&str]) -> Option<Self> {
        let (val, rest) = words.split_first()?;
        if !val.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return None;
        }
        let mut border = Self {
            val: (*val).to_string(),
            size: None,
            color: None,
            space: None,
            theme: None,
            tint: None,
            shade: None,
            shadow: false,
            frame: false,
        };
        let mut rest = rest.iter();
        while let Some(word) = rest.next() {
            if *word == "space" {
                let pt = rest.next()?.strip_suffix("pt")?;
                border.space = Some(pt.parse().ok()?);
            } else if *word == "theme" {
                let name = rest.next().filter(|n| is_theme_name(n))?;
                border.theme = Some((*name).to_string());
            } else if *word == "tint" || *word == "shade" {
                let value = rest.next().filter(|v| is_hex_byte(v))?;
                let slot = if *word == "tint" {
                    &mut border.tint
                } else {
                    &mut border.shade
                };
                *slot = Some((*value).to_string());
            } else if *word == "shadow" {
                border.shadow = true;
            } else if *word == "frame" {
                border.frame = true;
            } else if let Some(pt) = word.strip_suffix("pt") {
                let pt: f64 = pt.parse().ok()?;
                border.size = Some(eighths(pt)?);
            } else if word.len() == 6 && word.chars().all(|c| c.is_ascii_hexdigit()) {
                border.color = Some((*word).to_string());
            } else {
                return None;
            }
        }
        Some(border)
    }

    /// The side's element as the writer writes it.
    fn xml(&self, side: &str) -> String {
        if self.val == "none" {
            return format!("<w:{side} w:val=\"nil\"/>");
        }
        let mut xml = format!("<w:{side} w:val=\"{}\"", self.val);
        if let Some(size) = self.size {
            xml.push_str(&format!(" w:sz=\"{size}\""));
        }
        xml.push_str(&format!(" w:space=\"{}\"", self.space.unwrap_or(0)));
        if let Some(color) = &self.color {
            xml.push_str(&format!(" w:color=\"{color}\""));
        }
        for (name, value) in [
            ("themeColor", &self.theme),
            ("themeTint", &self.tint),
            ("themeShade", &self.shade),
        ] {
            if let Some(value) = value {
                xml.push_str(&format!(" w:{name}=\"{value}\""));
            }
        }
        for (name, on) in [("shadow", self.shadow), ("frame", self.frame)] {
            if on {
                xml.push_str(&format!(" w:{name}=\"1\""));
            }
        }
        xml.push_str("/>");
        xml
    }
}

/// A table's or a cell's borders, side by side, in schema order.
pub(crate) type Sides = Vec<(&'static str, Border)>;

/// The side Word names `name`, as the view names it (`start` is `left`,
/// `end` is `right`), when `sides` has it.
pub(crate) fn side(name: &str, sides: &[&'static str]) -> Option<&'static str> {
    let name = match name {
        "start" => "left",
        "end" => "right",
        other => other,
    };
    sides.iter().copied().find(|&s| s == name)
}

/// `found` (side, border) in schema order, a side named twice (`left` and
/// `start`) kept once, the first time.
pub(crate) fn ordered(found: &[(&'static str, Border)], sides: &[&'static str]) -> Sides {
    sides
        .iter()
        .filter_map(|&s| found.iter().find(|(f, _)| *f == s).cloned())
        .collect()
}

/// The text a line prints for `borders`: groups of sides that share a
/// border, in the order their first side comes.
pub(crate) fn describe(borders: &Sides, all: &[&'static str]) -> String {
    let mut groups: Vec<(Vec<&'static str>, &Border)> = Vec::new();
    for (side, border) in borders {
        match groups.iter_mut().find(|(_, b)| *b == border) {
            Some((sides, _)) => sides.push(side),
            None => groups.push((vec![side], border)),
        }
    }
    groups
        .iter()
        .map(|(sides, border)| {
            if sides.as_slice() == all {
                border.text()
            } else {
                format!("{} {}", sides.join(","), border.text())
            }
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

/// `text` (as [`describe`] prints it) read back; a group without sides
/// covers `all`. `None` when a group does not read.
pub(super) fn parse(text: &str, all: &[&'static str]) -> Option<Sides> {
    let mut found = Vec::new();
    for group in text.split(" + ") {
        let words: Vec<&str> = group.split_whitespace().collect();
        let named: Option<Vec<&'static str>> = words
            .first()?
            .split(',')
            .map(|name| all.iter().copied().find(|&s| s == name))
            .collect();
        let (sides, border) = match named {
            Some(sides) => (sides, Border::parse(&words[1..])?),
            None => (all.to_vec(), Border::parse(&words)?),
        };
        for side in sides {
            found.push((side, border.clone()));
        }
    }
    Some(ordered(&found, all))
}

/// The sides' elements, in schema order, for `w:tblBorders` or
/// `w:tcBorders`.
pub(super) fn xml(borders: &Sides) -> String {
    borders
        .iter()
        .map(|(side, border)| border.xml(side))
        .collect()
}

/// A `w:themeColor` value as the view prints it: one word of letters and
/// digits (`accent1`, `text1`, `followedHyperlink`).
fn is_theme_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric())
}

/// A `w:themeTint` or `w:themeShade` value: two hex digits.
fn is_hex_byte(value: &str) -> bool {
    value.len() == 2 && value.chars().all(|c| c.is_ascii_hexdigit())
}

/// Eighths of a point as points: `4` is `0.5`, `12` is `1.5`.
fn points(eighths: u32) -> String {
    let pt = format!("{:.3}", f64::from(eighths) / 8.0);
    pt.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn eighths(pt: f64) -> Option<u32> {
    let eighths = (pt * 8.0).round();
    // In range, the cast keeps the value.
    (0.0..=f64::from(u16::MAX))
        .contains(&eighths)
        .then_some(eighths as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_border_prints_and_reads_back() {
        for (sides, text) in [
            (
                vec![(
                    "top",
                    Border::new("single", Some("4"), Some("000000"), Some("0")),
                )],
                "top single 0.5pt 000000",
            ),
            (
                vec![("left", Border::new("nil", Some("4"), Some("FF0000"), None))],
                "left none",
            ),
            (
                vec![(
                    "bottom",
                    Border::new("double", Some("3"), Some("auto"), Some("1")),
                )],
                "bottom double 0.375pt space 1pt",
            ),
            (
                vec![("insideV", Border::new("dashed", None, None, None))],
                "insideV dashed",
            ),
        ] {
            assert_eq!(describe(&sides, &TABLE_SIDES), text);
            assert_eq!(parse(text, &TABLE_SIDES), Some(sides));
        }
    }

    #[test]
    fn all_six_sides_drop_their_names_and_groups_keep_their_order() {
        let thin = Border::new("single", Some("4"), None, None);
        let all: Sides = TABLE_SIDES.iter().map(|&s| (s, thin.clone())).collect();
        assert_eq!(describe(&all, &TABLE_SIDES), "single 0.5pt");
        assert_eq!(parse("single 0.5pt", &TABLE_SIDES), Some(all));
        let cell: Sides = ["top", "left", "bottom", "right"]
            .iter()
            .map(|&s| (s, thin.clone()))
            .collect();
        assert_eq!(
            describe(&cell, &CELL_SIDES),
            "top,left,bottom,right single 0.5pt"
        );
        let mixed = vec![
            ("top", thin.clone()),
            ("left", Border::new("nil", None, None, None)),
            ("bottom", thin),
        ];
        assert_eq!(
            describe(&mixed, &TABLE_SIDES),
            "top,bottom single 0.5pt + left none"
        );
        assert_eq!(
            parse("top,bottom single 0.5pt + left none", &TABLE_SIDES),
            Some(mixed)
        );
    }

    #[test]
    fn start_and_end_are_left_and_right_and_text_that_does_not_read_is_none() {
        assert_eq!(side("start", &TABLE_SIDES), Some("left"));
        assert_eq!(side("end", &CELL_SIDES), Some("right"));
        assert_eq!(side("tl2br", &TABLE_SIDES), None);
        for bad in [
            "",
            "top",
            "top single 0.5",
            "single 0.5pt red",
            "top single space",
        ] {
            assert_eq!(parse(bad, &TABLE_SIDES), None, "{bad}");
        }
    }

    #[test]
    fn the_writer_writes_word_s_attributes() {
        let sides = vec![
            (
                "top",
                Border::new("single", Some("4"), Some("000000"), None),
            ),
            ("left", Border::new("none", None, None, None)),
        ];
        assert_eq!(
            xml(&sides),
            "<w:top w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"000000\"/><w:left w:val=\"nil\"/>"
        );
    }
}
