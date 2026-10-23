// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! The style legend an agent view's YAML header prints, read back:
//!
//! ```yaml
//! styles:
//!   Normal: Times New Roman 11pt, after 6pt, line 12pt atLeast, left  # default; …
//!   "#": Heading1, Times New Roman bold 24pt, before 24pt, keep-next
//! ```
//!
//! The default paragraph style's line gives its whole look: font, bold and
//! italic, size, spacing before and after (none printed is none), line
//! spacing (none printed is single), keep-next, keep-lines, a paragraph
//! indent (`indent 0.5in`, and `first-line`, `hanging` or `right` when set),
//! contextual spacing and alignment. A heading's line names its style, its
//! font, size, bold and italic, and only the spacing, keep-next, keep-lines,
//! indent, contextual spacing and alignment that differ from the default
//! style's, which it is based on. Every other paragraph style the body uses
//! follows, keyed by its id, described like a heading (`TOC2: Arial 12pt,
//! after 6pt`). A line ends `, named Text Heading 2` when the style's name
//! is not its id (for a heading `HeadingN`, not `heading N`). A character
//! style a story paragraph names leads with `character` (`LineNumber:
//! character, Calibri 11pt, named line number`), and the writer rebuilds a
//! `w:type="character"` style for it.
//!
//! The writer sets a property only when the legend's value differs from
//! what the style already inherits: document defaults for the default and
//! character styles, and the default paragraph style for other paragraph styles.
//! The default style's spacing is written as the document default itself
//! (`w:pPrDefault`), where Word keeps it.

use std::fmt::Write as _;

use super::from_docx::Element;
use super::xml::{FormatMark, escape};

/// `styles` with a tracked change on each holder the view's
/// `style_changes:` names. A style's two tags name its `w:pPrChange` and
/// `w:rPrChange` in document order; a single tag names both with their
/// shared id, as Word writes them. `docDefaults` takes only a
/// `w:rPrChange`. Each carries empty previous properties as the view names
/// no payload. A holder the part has no style of is left out.
pub(super) fn with_style_changes(styles: &str, changes: &[(String, Vec<FormatMark>)]) -> String {
    let mut out = styles.to_string();
    for (holder, marks) in changes {
        for (index, mark) in marks.iter().enumerate() {
            let date = super::xml::date_attribute(&mark.date);
            let id = mark
                .id
                .map_or_else(String::new, |id| format!(" w:id=\"{id}\""));
            let change = |name: &str, inner: &str| {
                format!(
                    "<w:{name}Change{id} w:author=\"{}\"{date}><w:{inner}/></w:{name}Change>",
                    escape(&mark.author)
                )
            };
            if holder == "docDefaults" {
                with_defaults_change(&mut out, &change("rPr", "rPr"));
                continue;
            }
            let key = format!("w:styleId=\"{}\"", escape(holder));
            let Some(at) = out.find(&key) else {
                continue;
            };
            let Some(end) = out[at..].find("</w:style>").map(|i| at + i) else {
                continue;
            };
            let mut block = out[at..end].to_string();
            // The style's own `w:pPr` and `w:rPr`: the `w:rPr` a `w:pPr`
            // holds is the paragraph mark's (`CT_ParaRPr`), and
            // `CT_Style` puts the style's own after the `w:pPr`.
            if marks.len() != 2 || index == 0 {
                let change = change("pPr", "pPr");
                match own_ppr(&block) {
                    Some(ppr) if ppr.empty() => {
                        block
                            .replace_range(ppr.start..ppr.end, &format!("<w:pPr>{change}</w:pPr>"));
                    }
                    Some(ppr) => block.insert_str(ppr.at(), &change),
                    // The style had no `w:pPr`: one takes the change,
                    // where `CT_Style` puts it, before its `w:rPr`.
                    None => {
                        let at = own_rpr(&block).map_or(block.len(), |rpr| rpr.start);
                        block.insert_str(at, &format!("<w:pPr>{change}</w:pPr>"));
                    }
                }
            }
            if marks.len() != 2 || index == 1 {
                let change = change("rPr", "rPr");
                match own_rpr(&block) {
                    Some(rpr) if rpr.empty() => {
                        block
                            .replace_range(rpr.start..rpr.end, &format!("<w:rPr>{change}</w:rPr>"));
                    }
                    Some(rpr) => block.insert_str(rpr.at(), &change),
                    None => block.push_str(&format!("<w:rPr>{change}</w:rPr>")),
                }
            }
            out.replace_range(at..end, &block);
        }
    }
    out
}

/// `carrier` (a `w:rPrChange`) last in the run defaults' `w:rPr`, making
/// whatever of `w:docDefaults` / `w:rPrDefault` / `w:rPr` the part lacks
/// where the schema puts it: `w:docDefaults` first in `w:styles`,
/// `w:rPrDefault` first in it. A part with no `w:styles` takes nothing.
fn with_defaults_change(out: &mut String, carrier: &str) {
    let run = format!("<w:rPr>{carrier}</w:rPr>");
    let default = format!("<w:rPrDefault>{run}</w:rPrDefault>");
    let Some(defaults) = open_tag(out, "docDefaults", 0..out.len()) else {
        if let Some(styles) = open_tag(out, "styles", 0..out.len()).filter(|t| !t.empty) {
            out.insert_str(
                styles.end,
                &format!("<w:docDefaults>{default}</w:docDefaults>"),
            );
        }
        return;
    };
    if defaults.empty {
        out.replace_range(
            defaults.start..defaults.end,
            &format!("<w:docDefaults>{default}</w:docDefaults>"),
        );
        return;
    }
    let Some(close) = out[defaults.end..]
        .find("</w:docDefaults>")
        .map(|at| defaults.end + at)
    else {
        return;
    };
    let Some(rpr_default) = open_tag(out, "rPrDefault", defaults.end..close) else {
        out.insert_str(defaults.end, &default);
        return;
    };
    if rpr_default.empty {
        out.replace_range(rpr_default.start..rpr_default.end, &default);
        return;
    }
    let Some(default_close) = out[rpr_default.end..close]
        .find("</w:rPrDefault>")
        .map(|at| rpr_default.end + at)
    else {
        return;
    };
    match open_tag(out, "rPr", rpr_default.end..default_close) {
        Some(rpr) if rpr.empty => out.replace_range(rpr.start..rpr.end, &run),
        // The last `</w:rPr>` in the default closes its own `w:rPr`: an
        // earlier one closes a change's previous properties.
        Some(rpr) => {
            if let Some(at) = out[rpr.end..default_close].rfind("</w:rPr>") {
                out.insert_str(rpr.end + at, carrier);
            }
        }
        None => out.insert_str(rpr_default.end, &run),
    }
}

/// Where an element's start tag lies.
struct OpenTag {
    start: usize,
    /// Just past its `>`.
    end: usize,
    /// `<w:name/>`, which holds nothing.
    empty: bool,
}

/// The first `<w:name …>` start tag in `range` of `xml`, attributes and
/// all; `<w:nameMore` is another element.
fn open_tag(xml: &str, name: &str, range: std::ops::Range<usize>) -> Option<OpenTag> {
    let tag = format!("<w:{name}");
    let mut from = range.start;
    while let Some(at) = xml[from..range.end].find(&tag).map(|i| from + i) {
        let after = at + tag.len();
        if xml[after..].starts_with([' ', '>', '/', '\t', '\n', '\r']) {
            let end = after + xml[after..].find('>')? + 1;
            return Some(OpenTag {
                start: at,
                end,
                empty: xml[..end - 1].ends_with('/'),
            });
        }
        from = after;
    }
    None
}

/// One element of a style the view writes a change into, and where that
/// change goes: before the element's close tag, which is where each
/// schema puts its change (`w:pPrChange` last in `CT_PPr`, `w:rPrChange`
/// last in `CT_RPr`).
struct Own {
    start: usize,
    end: usize,
    /// The close tag's own start; `None` for `<w:name/>`, which a change
    /// replaces whole.
    close: Option<usize>,
}

impl Own {
    /// Where a change of this element goes.
    fn at(&self) -> usize {
        self.close.unwrap_or(self.start)
    }

    /// Whether the change replaces the element rather than joining it.
    fn empty(&self) -> bool {
        self.close.is_none()
    }
}

/// The first `<w:name>` element in `block` from `from`, with where its
/// change goes. The element nests: a `w:pPrChange` holds the previous
/// properties in a `w:pPr` of their own.
fn own(block: &str, name: &str, from: usize) -> Option<Own> {
    let open = open_tag(block, name, from..block.len())?;
    if open.empty {
        return Some(Own {
            start: open.start,
            end: open.end,
            close: None,
        });
    }
    let close = close_at(block, name, open.end)?;
    let tag = format!("</w:{name}>");
    Some(Own {
        start: open.start,
        end: close + tag.len(),
        close: Some(close),
    })
}

/// Where the `<w:name>` element that opens just before `open_end` closes:
/// the closes nest, so every open before a close counts unless it is an
/// empty element, which holds nothing.
fn close_at(block: &str, name: &str, open_end: usize) -> Option<usize> {
    let tag = format!("</w:{name}>");
    let mut depth = 1usize;
    let mut at = open_end;
    while let Some(close) = block[at..].find(&tag).map(|i| i + at) {
        depth += usize::from(open_tag(block, name, at..close).is_some_and(|open| !open.empty));
        depth = depth.checked_sub(1)?;
        if depth == 0 {
            return Some(close);
        }
        at = close + tag.len();
    }
    None
}

/// The style's own `w:pPr`: the first one, never the previous properties a
/// `w:pPrChange` holds.
fn own_ppr(block: &str) -> Option<Own> {
    own(block, "pPr", 0)
}

/// The style's own `w:rPr`: `CT_Style` puts it after the `w:pPr`, and the
/// `w:rPr` a `w:pPr` holds is the paragraph mark's (`CT_ParaRPr`), not
/// the style's (#1595 review).
fn own_rpr(block: &str) -> Option<Own> {
    own(block, "rPr", own_ppr(block).map_or(0, |ppr| ppr.end))
}

/// One style the legend describes.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct StyleSpec {
    pub id: String,
    /// The style's name, when the legend gives one (`, named …`).
    pub name: Option<String>,
    /// A character style: the legend's first clause names the kind
    /// (`LineNumber: character, Calibri 11pt`).
    pub character: bool,
    /// The `#` count for a heading.
    pub heading: Option<u32>,
    /// The default paragraph style, which other paragraph styles are based on.
    pub default: bool,
    pub font: String,
    pub bold: bool,
    pub italic: bool,
    /// Half points.
    pub size: u32,
    /// Twips.
    pub before: Option<u32>,
    pub after: Option<u32>,
    /// Twips with `exact` or `atLeast`, or 240ths of a line with `auto`.
    pub line: Option<(u32, &'static str)>,
    pub keep_next: bool,
    pub keep_lines: bool,
    /// Left indent, twips.
    pub left: Option<i64>,
    /// Right indent, twips.
    pub right: Option<i64>,
    /// First-line indent, twips.
    pub first_line: Option<i64>,
    /// Hanging indent, twips.
    pub hanging: Option<i64>,
    pub contextual_spacing: bool,
    pub align: Option<&'static str>,
}

/// The largest font size and spacing the legend can set: Word's 1638pt.
const MOST_POINTS: f64 = 1638.0;

fn points(value: &str, scale: f64) -> Option<u32> {
    let n: f64 = value.strip_suffix("pt")?.parse().ok()?;
    // `as` cannot overflow: the bound is checked first.
    (n.is_finite() && (0.0..=MOST_POINTS).contains(&n)).then(|| (n * scale).round() as u32)
}

/// `0.5in` as twips. Word's indent is at most 22in (31680 twips) either way.
pub(super) fn twips_in(value: &str) -> Option<i64> {
    let n: f64 = value.strip_suffix("in")?.parse().ok()?;
    if !n.is_finite() {
        return None;
    }
    let twips = (n * 1440.0).round();
    // `as` cannot overflow: the bound is checked first.
    (twips.is_finite() && twips.abs() <= 31680.0).then_some(twips as i64)
}

/// `Times New Roman bold italic 11pt`: the font, bold, italic and size.
fn font_clause(clause: &str) -> Option<(String, bool, bool, u32)> {
    let (rest, size) = clause.rsplit_once(' ')?;
    let size = points(size, 2.0)?;
    let mut rest = rest;
    let mut italic = false;
    let mut bold = false;
    if let Some(r) = rest.strip_suffix(" italic") {
        italic = true;
        rest = r;
    }
    if let Some(r) = rest.strip_suffix(" bold") {
        bold = true;
        rest = r;
    }
    (!rest.is_empty()).then(|| (rest.to_string(), bold, italic, size))
}

/// The style a legend value describes; `heading` is the `#` count.
fn spec(id: &str, heading: Option<u32>, default: bool, value: &str) -> Option<StyleSpec> {
    let (value, name) = match value.rsplit_once(", named ") {
        Some((value, name)) => (value, Some(name.to_string())),
        None => (value, None),
    };
    let mut clauses = value.split(", ").peekable();
    // A character style leads with its kind, before the look. The default
    // style is a paragraph one: for it, `character` is no clause we know.
    let character = !default && clauses.next_if(|c| *c == "character").is_some();
    let (font, bold, italic, size) = font_clause(clauses.next()?)?;
    let mut spec = StyleSpec {
        id: id.to_string(),
        name,
        character,
        heading,
        default,
        font,
        bold,
        italic,
        size,
        before: None,
        after: None,
        line: None,
        keep_next: false,
        keep_lines: false,
        left: None,
        right: None,
        first_line: None,
        hanging: None,
        contextual_spacing: false,
        align: None,
    };
    for clause in clauses {
        match clause.split_once(' ') {
            Some(("before", v)) => spec.before = Some(points(v, 20.0)?),
            Some(("after", v)) => spec.after = Some(points(v, 20.0)?),
            Some(("indent", v)) => spec.left = Some(twips_in(v)?),
            // `right` alone is the alignment; with a value, the indent.
            Some(("right", v)) => spec.right = Some(twips_in(v)?),
            Some(("first-line", v)) => spec.first_line = Some(twips_in(v)?),
            Some(("hanging", v)) => spec.hanging = Some(twips_in(v)?),
            Some(("contextual", "spacing")) => spec.contextual_spacing = true,
            Some(("line", v)) => {
                spec.line = Some(match v.split_once(' ') {
                    Some((height, "exact")) => (points(height, 20.0)?, "exact"),
                    Some((height, "atLeast")) => (points(height, 20.0)?, "atLeast"),
                    Some(_) => return None,
                    None => {
                        let ratio: f64 = v.parse().ok()?;
                        if !(ratio.is_finite() && (0.0..=132.0).contains(&ratio)) {
                            return None;
                        }
                        // `as` cannot overflow: the bound is checked first.
                        ((ratio * 240.0).round() as u32, "auto")
                    }
                });
            }
            None if clause == "keep-next" => spec.keep_next = true,
            None if clause == "keep-lines" => spec.keep_lines = true,
            None => {
                spec.align = Some(match clause {
                    "left" => "left",
                    "center" => "center",
                    "right" => "right",
                    "justify" => "both",
                    _ => return None,
                });
            }
            Some(_) => return None,
        }
    }
    Some(spec)
}

/// `line` without the comment the view prints after a value: a `#` after
/// two spaces, outside a quoted string (`text: "ACME  #1"` keeps its
/// `#`; so does `title: Issue #5`).
pub(super) fn without_comment(line: &str) -> &str {
    let mut quoted = false;
    let mut escaped = false;
    let mut spaces = 0;
    for (at, c) in line.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '#' if spaces >= 2 && !quoted => return &line[..at],
            _ => {}
        }
        spaces = if c == ' ' { spaces + 1 } else { 0 };
    }
    line
}

/// A YAML scalar as the legend prints it: plain, or a JSON string.
fn unquote(value: &str) -> Option<String> {
    if value.starts_with('"') {
        serde_json::from_str(value).ok()
    } else {
        Some(value.to_string())
    }
}

/// A table style's borders as its legend entry states them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TableBorders {
    /// `all borders 0.5pt`: every side one single line of this size, in
    /// eighths of a point.
    All(u32),
    /// `borders bottom double 0.75pt`: each side the style sets, in Word's
    /// order (top, left, bottom, right, insideH, insideV).
    Sides(Vec<BorderSide>),
}

/// One side of a `borders …` clause: `top single 1.5pt #FF0000 space 2pt`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct BorderSide {
    /// `top`, `left`, `bottom`, `right`, `insideH` or `insideV`.
    pub side: String,
    /// The `w:val`: `single`, `double`, `nil`, …
    pub val: String,
    /// The `w:sz`, in eighths of a point.
    pub size: Option<u32>,
    /// The `w:color` when it is not `auto`.
    pub color: Option<String>,
    /// The `w:space`, in points, when it is not zero.
    pub space: Option<u32>,
}

/// The sides a table style's `w:tblBorders` holds, in the order Word's
/// schema puts them.
pub(crate) const TABLE_BORDER_SIDES: [&str; 6] =
    ["top", "left", "bottom", "right", "insideH", "insideV"];

/// A `borders …` clause's sides (`borders bottom double 0.75pt`): a side
/// name, its value, then a size in points, a `#RRGGBB` colour and a
/// `space Npt`, each optional. A side the grammar does not know, or one
/// named twice, reads as no clause.
fn border_sides(clause: &str) -> Option<Vec<BorderSide>> {
    let mut tokens = clause.strip_prefix("borders ")?.split(' ').peekable();
    let mut sides: Vec<BorderSide> = Vec::new();
    while let Some(side) = tokens.next() {
        if !TABLE_BORDER_SIDES.contains(&side) || sides.iter().any(|s| s.side == side) {
            return None;
        }
        let val = tokens
            .next()
            .filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_alphanumeric()))?;
        let mut border = BorderSide {
            side: side.to_string(),
            val: val.to_string(),
            size: None,
            color: None,
            space: None,
        };
        while let Some(&token) = tokens.peek() {
            if TABLE_BORDER_SIDES.contains(&token) {
                break;
            }
            tokens.next();
            if let Some(pt) = token.strip_suffix("pt").filter(|_| border.size.is_none()) {
                let pt = pt
                    .parse::<f64>()
                    .ok()
                    .filter(|pt| (0.0..=255.0).contains(pt))?;
                border.size = Some((pt * 8.0).round() as u32);
            } else if let Some(hex) = token.strip_prefix('#').filter(|_| border.color.is_none()) {
                (hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit())).then_some(())?;
                border.color = Some(hex.to_string());
            } else if token == "space" && border.space.is_none() {
                let pt = tokens.next()?.strip_suffix("pt")?.parse::<u32>().ok()?;
                border.space = Some(pt);
            } else {
                return None;
            }
        }
        sides.push(border);
    }
    (!sides.is_empty()).then_some(sides)
}

/// The legend's `table:` style, which every table takes: its id, the
/// borders its entry states and the name it gives the style.
pub(super) fn table_style(yaml: &str) -> Option<(String, Option<TableBorders>, Option<String>)> {
    legend_line(yaml, "table: ")
        .as_deref()
        .and_then(table_style_entry)
        .map(|(entry, _)| entry)
}

/// The `tables:` legend's entries, keyed by their style id: the styles of
/// a document whose tables differ, each with the border and the name its
/// entry states.
pub(super) fn table_legend(
    yaml: &str,
) -> std::collections::BTreeMap<String, (Option<TableBorders>, Option<String>)> {
    let mut out = std::collections::BTreeMap::new();
    let Some(value) = legend_line(yaml, "tables: ") else {
        return out;
    };
    let mut rest = value.as_str();
    while let Some(((id, border, name), next)) = table_style_entry(rest) {
        out.insert(id, (border, name));
        rest = next;
    }
    out
}

/// The `styles:` block's `key: ` line's value, unwrapped from its YAML
/// quoting: `table: X, named My X`'s, `tables: …`'s.
fn legend_line(yaml: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in yaml.lines() {
        if !line.starts_with(' ') {
            inside = line.trim_end() == "styles:";
            continue;
        }
        if let Some(value) = without_comment(line)
            .trim()
            .strip_prefix(key)
            .filter(|_| inside)
            .and_then(|v| unquote(v.trim()))
        {
            return Some(value);
        }
    }
    None
}

/// One `table:`/`tables:` entry — `X, all borders 0.5pt, named My X` —
/// its id, the border it states and the name it gives the style, with
/// the rest of the line after it (the `tables:` legend holds one entry
/// per style). A clause that is none of the grammar's ends the entry.
type TableStyleEntry = (String, Option<TableBorders>, Option<String>);

fn table_style_entry(value: &str) -> Option<(TableStyleEntry, &str)> {
    let (id, mut rest) = take_clause(value);
    let id = unquote(id.trim()).filter(|id| !id.is_empty())?;
    let mut border = None;
    let mut name = None;
    while !rest.is_empty() {
        let (clause, next) = take_clause(rest);
        if border.is_none()
            && let Some(pt) = clause
                .strip_prefix("all borders ")
                .and_then(|pt| pt.strip_suffix("pt"))
                .and_then(|pt| pt.parse::<f64>().ok())
                .filter(|pt| (0.25..=6.0).contains(pt))
        {
            border = Some(TableBorders::All((pt * 8.0).round() as u32));
        } else if border.is_none()
            && let Some(sides) = border_sides(&clause)
        {
            border = Some(TableBorders::Sides(sides));
        } else if name.is_none()
            && let Some(named) = clause
                .strip_prefix("named ")
                .and_then(|named| unquote(named.trim()))
        {
            name = Some(named);
        } else if clause != "borders vary" {
            return Some(((id, border, name), rest));
        }
        rest = next;
    }
    Some(((id, border, name), ""))
}

/// The value's first `, `-separated clause, a quoted one whole (a `"`
/// opens a JSON string, whose `, ` is the name's own), and what follows
/// the separator.
fn take_clause(value: &str) -> (String, &str) {
    let (mut quoted, mut escaped) = (false, false);
    for (i, c) in value.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            ',' if !quoted && value[i + 1..].starts_with(' ') => {
                return (value[..i].to_string(), &value[i + 2..]);
            }
            _ => {}
        }
    }
    (value.to_string(), "")
}

pub(super) fn legend(yaml: &str) -> Vec<StyleSpec> {
    let mut specs = Vec::new();
    let mut inside = false;
    for line in yaml.lines() {
        if !line.starts_with(' ') {
            inside = line.trim_end() == "styles:";
            continue;
        }
        if !inside {
            continue;
        }
        // A YAML comment (`  # …`) is no part of the value.
        let line = without_comment(line).trim();
        let Some((key, value)) = line.split_once(": ") else {
            continue;
        };
        let (Some(key), Some(value)) = (unquote(key), unquote(value.trim())) else {
            continue;
        };
        let spec = if key.bytes().all(|b| b == b'#') {
            let level = u32::try_from(key.len())
                .ok()
                .filter(|n| (1..=6).contains(n));
            level.and_then(|level| {
                let (id, rest) = value.split_once(", ")?;
                spec(&unquote(id)?, Some(level), false, rest)
            })
        } else if matches!(key.as_str(), "table" | "tables" | "changes") {
            None
        } else {
            let default = !specs.iter().any(|s: &StyleSpec| s.default);
            spec(&key, None, default, &value)
        };
        specs.extend(spec);
    }
    specs
}

/// `styles` (a styles part) with each style of `specs` defined as the
/// legend describes it, in place of its own definition. A default style
/// other than `Normal` takes the default from the part's own.
pub(super) fn apply(styles: &str, specs: &[StyleSpec]) -> String {
    let mut defaults = document_defaults(styles);
    let mut out = styles.to_string();
    let base = specs.iter().find(|s| s.default);
    let from_base = base.map(effective);
    if let Some(look) = &from_base {
        out = out.replace(
            "w:type=\"paragraph\" w:default=\"1\"",
            "w:type=\"paragraph\"",
        );
        // The default style's spacing is the document's default, as Word
        // keeps it: a table style's own spacing wins over `docDefaults` in
        // its cells, and would not over the default style's (ISO/IEC 29500
        // style precedence).
        with_default_spacing(&mut out, look);
        defaults.before = look.before;
        defaults.after = look.after;
        defaults.line_twips = look.line_twips;
        defaults.line_rule.clone_from(&look.line_rule);
    }
    for spec in specs {
        out = without_style(&out, &spec.id);
        let inherited = if spec.default || spec.character {
            &defaults
        } else {
            from_base.as_ref().unwrap_or(&defaults)
        };
        // Word ignores a basedOn that names a different kind of style
        // (ISO/IEC 29500, basedOn). Character styles therefore use
        // docDefaults and write their run properties against those.
        let parent = base.filter(|_| !spec.character).map(|b| b.id.as_str());
        let definition = definition(spec, parent, inherited);
        if let Some(at) = out.rfind("</w:styles>") {
            out.insert_str(at, &definition);
        }
    }
    out
}

/// The `w:pPr` children CT_PPrBase puts after `w:spacing`.
const AFTER_SPACING: [&str; 11] = [
    "ind",
    "contextualSpacing",
    "mirrorIndents",
    "suppressOverlap",
    "jc",
    "textDirection",
    "textAlignment",
    "textboxTightWrap",
    "outlineLvl",
    "divId",
    "cnfStyle",
];

/// `out` with `look`'s spacing as the document's paragraph default
/// (`w:docDefaults/w:pPrDefault/w:pPr/w:spacing`) in place of the spacing
/// it had, each element made where the part lacks it: `w:docDefaults`
/// first in `w:styles`, `w:pPrDefault` after `w:rPrDefault`, the spacing
/// where CT_PPrBase puts it. A part with no `w:styles` takes nothing.
fn with_default_spacing(out: &mut String, look: &Given) {
    let mut spacing = String::from("<w:spacing");
    if look.before > 0 {
        let _ = write!(spacing, " w:before=\"{}\"", look.before);
    }
    let _ = write!(
        spacing,
        " w:after=\"{}\" w:line=\"{}\" w:lineRule=\"{}\"/>",
        look.after, look.line_twips, look.line_rule
    );
    let ppr = format!("<w:pPr>{spacing}</w:pPr>");
    let default = format!("<w:pPrDefault>{ppr}</w:pPrDefault>");
    let Some(defaults) = open_tag(out, "docDefaults", 0..out.len()) else {
        if let Some(styles) = open_tag(out, "styles", 0..out.len()).filter(|t| !t.empty) {
            out.insert_str(
                styles.end,
                &format!("<w:docDefaults>{default}</w:docDefaults>"),
            );
        }
        return;
    };
    if defaults.empty {
        out.replace_range(
            defaults.start..defaults.end,
            &format!("<w:docDefaults>{default}</w:docDefaults>"),
        );
        return;
    }
    let Some(close) = out[defaults.end..]
        .find("</w:docDefaults>")
        .map(|at| defaults.end + at)
    else {
        return;
    };
    let Some(ppr_default) = open_tag(out, "pPrDefault", defaults.end..close) else {
        out.insert_str(close, &default);
        return;
    };
    if ppr_default.empty {
        out.replace_range(ppr_default.start..ppr_default.end, &default);
        return;
    }
    let Some(default_close) = out[ppr_default.end..close]
        .find("</w:pPrDefault>")
        .map(|at| ppr_default.end + at)
    else {
        return;
    };
    let Some(own) = open_tag(out, "pPr", ppr_default.end..default_close) else {
        out.insert_str(ppr_default.end, &ppr);
        return;
    };
    if own.empty {
        out.replace_range(own.start..own.end, &ppr);
        return;
    }
    let Some(mut own_close) = out[own.end..default_close]
        .rfind("</w:pPr>")
        .map(|at| own.end + at)
    else {
        return;
    };
    if let Some(old) = open_tag(out, "spacing", own.end..own_close) {
        let end = if old.empty {
            old.end
        } else {
            out[old.end..own_close]
                .find("</w:spacing>")
                .map_or(old.end, |at| old.end + at + "</w:spacing>".len())
        };
        out.replace_range(old.start..end, "");
        own_close -= end - old.start;
    }
    let at = AFTER_SPACING
        .iter()
        .filter_map(|name| open_tag(out, name, own.end..own_close).map(|t| t.start))
        .min()
        .unwrap_or(own_close);
    out.insert_str(at, &spacing);
}

/// `styles` without the definition of style `id`.
fn without_style(styles: &str, id: &str) -> String {
    let key = format!("w:styleId=\"{}\"", escape(id));
    let mut out = String::with_capacity(styles.len());
    let mut rest = styles;
    while let Some(at) = rest.find("<w:style ") {
        let Some(tag_end) = rest[at..].find('>').map(|i| at + i) else {
            break;
        };
        let Some(end) = rest[tag_end..].find("</w:style>").map(|i| tag_end + i + 10) else {
            break;
        };
        out.push_str(&rest[..at]);
        if !rest[at..tag_end].contains(&key) {
            out.push_str(&rest[at..end]);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// What a style already inherits, so a property equal to it is not written.
struct Given {
    font: Option<String>,
    size: Option<u32>,
    bold: bool,
    italic: bool,
    before: u32,
    after: u32,
    line_twips: u32,
    line_rule: String,
    align: String,
    keep_next: bool,
    keep_lines: bool,
    contextual: bool,
    left: Option<i64>,
    right: Option<i64>,
    first_line: Option<i64>,
    hanging: Option<i64>,
}

impl Given {
    /// Word's own values when `docDefaults` names nothing.
    fn implicit() -> Self {
        Self {
            font: None,
            size: None,
            bold: false,
            italic: false,
            before: 0,
            after: 0,
            line_twips: 240,
            line_rule: "auto".to_string(),
            align: "left".to_string(),
            keep_next: false,
            keep_lines: false,
            contextual: false,
            left: None,
            right: None,
            first_line: None,
            hanging: None,
        }
    }
}

/// The legend's value for the default style: an omitted spacing is zero or
/// single, an omitted alignment is left, and an omitted indent is none.
fn effective(spec: &StyleSpec) -> Given {
    let (line_twips, line_rule) = spec.line.unwrap_or((240, "auto"));
    Given {
        font: Some(spec.font.clone()),
        size: Some(spec.size),
        bold: spec.bold,
        italic: spec.italic,
        before: spec.before.unwrap_or(0),
        after: spec.after.unwrap_or(0),
        line_twips,
        line_rule: line_rule.to_string(),
        align: spec.align.unwrap_or("left").to_string(),
        keep_next: spec.keep_next,
        keep_lines: spec.keep_lines,
        contextual: spec.contextual_spacing,
        left: spec.left,
        right: spec.right,
        first_line: spec.first_line,
        hanging: spec.hanging,
    }
}

fn whole(element: &Element, attr: &str) -> Option<u32> {
    let n: f64 = element.attr(attr)?.parse().ok()?;
    let n = n.round();
    // `as` cannot overflow: the bound is checked first.
    (n.is_finite() && (0.0..=u32::MAX as f64).contains(&n)).then_some(n as u32)
}

fn signed(element: &Element, attr: &str) -> Option<i64> {
    let n: f64 = element.attr(attr)?.parse().ok()?;
    let n = n.round();
    // `as` cannot overflow: the bound is checked first.
    (n.is_finite() && n.abs() <= 1.0e15).then_some(n as i64)
}

/// `docDefaults` over Word's implicit values. A part that does not parse
/// keeps the implicit values, so the legend's own properties are still written.
fn document_defaults(styles: &str) -> Given {
    let mut given = Given::implicit();
    let Ok(root) = super::from_docx::parse_xml(styles.as_bytes()) else {
        return given;
    };
    let Some(defaults) = root.child("docDefaults") else {
        return given;
    };
    if let Some(rpr) = defaults.path(&["rPrDefault", "rPr"]) {
        // A theme font wins over the named one, so the named one is not
        // what the style inherits.
        if let Some(fonts) = rpr.child("rFonts") {
            given.font = fonts
                .attr("asciiTheme")
                .is_none()
                .then(|| fonts.attr("ascii").or_else(|| fonts.attr("hAnsi")))
                .flatten()
                .map(str::to_string);
        }
        if let Some(size) = rpr.child("sz").and_then(|sz| whole(sz, "val")) {
            given.size = Some(size);
        }
        if let Some(bold) = rpr.toggle("b") {
            given.bold = bold;
        }
        if let Some(italic) = rpr.toggle("i") {
            given.italic = italic;
        }
    }
    if let Some(ppr) = defaults.path(&["pPrDefault", "pPr"]) {
        if let Some(spacing) = ppr.child("spacing") {
            if let Some(before) = whole(spacing, "before") {
                given.before = before;
            }
            if let Some(after) = whole(spacing, "after") {
                given.after = after;
            }
            if let Some(line) = whole(spacing, "line") {
                given.line_twips = line;
                given.line_rule = spacing.attr("lineRule").unwrap_or("auto").to_string();
            }
        }
        if let Some(ind) = ppr.child("ind") {
            if let Some(left) = signed(ind, "left").or_else(|| signed(ind, "start")) {
                given.left = Some(left);
            }
            if let Some(right) = signed(ind, "right").or_else(|| signed(ind, "end")) {
                given.right = Some(right);
            }
            if let Some(first) = signed(ind, "firstLine") {
                given.first_line = Some(first);
            }
            if let Some(hanging) = signed(ind, "hanging") {
                given.hanging = Some(hanging);
            }
        }
        if let Some(on) = ppr.toggle("keepNext") {
            given.keep_next = on;
        }
        if let Some(on) = ppr.toggle("keepLines") {
            given.keep_lines = on;
        }
        if let Some(on) = ppr.toggle("contextualSpacing") {
            given.contextual = on;
        }
        if let Some(align) = ppr.child("jc").and_then(|jc| jc.attr("val")) {
            given.align = align.to_string();
        }
    }
    given
}

/// An indent the legend sets and the style does not already inherit.
fn indent_attr(spec: Option<i64>, inherited: Option<i64>) -> Option<i64> {
    spec.filter(|value| Some(*value) != inherited)
}

/// The gallery flags (`uiPriority`, `semiHidden`, `unhideWhenUsed`,
/// `qFormat`) Word gives built-in style `id` named `name`, in schema order;
/// none for a style Word does not know.
fn gallery_flags(id: &str, name: &str) -> String {
    let Some(builtin) = super::xml::style_definition(id) else {
        return String::new();
    };
    if !builtin.contains(&format!("<w:name w:val=\"{name}\"/>")) {
        return String::new();
    }
    let mut out = String::new();
    for flag in [
        "<w:uiPriority ",
        "<w:semiHidden/>",
        "<w:unhideWhenUsed/>",
        "<w:qFormat/>",
    ] {
        if let Some(at) = builtin.find(flag) {
            let end = builtin[at..]
                .find("/>")
                .map_or(builtin.len(), |e| at + e + 2);
            out.push_str(&builtin[at..end]);
        }
    }
    out
}

/// The `w:style` element of `spec`, based on style `base` unless it is the
/// default. `inherited` is what the style already has, and a property equal
/// to it is left out.
fn definition(spec: &StyleSpec, base: Option<&str>, inherited: &Given) -> String {
    let id = escape(&spec.id);
    let name = match (&spec.name, spec.heading, spec.id.strip_prefix("Heading")) {
        (Some(name), ..) => escape(name).to_string(),
        (None, Some(_), Some(n)) if n.parse::<u32>().is_ok() => format!("heading {n}"),
        _ => id.to_string(),
    };
    // `w:name` holds no comma (pattern `[^,]*`); a style id may.
    let name = name.replace(',', " ");
    let mut out = if spec.default {
        format!(
            "<w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"{id}\"><w:name w:val=\"{name}\"/><w:qFormat/>"
        )
    } else {
        let mut out = format!(
            "<w:style w:type=\"{}\" w:styleId=\"{id}\"><w:name w:val=\"{name}\"/>",
            if spec.character {
                "character"
            } else {
                "paragraph"
            }
        );
        if let Some(base) = base.filter(|b| *b != spec.id) {
            out.push_str(&format!("<w:basedOn w:val=\"{}\"/>", escape(base)));
            if spec.heading.is_some() {
                out.push_str(&format!("<w:next w:val=\"{}\"/>", escape(base)));
            }
        }
        if spec.heading.is_some() {
            out.push_str("<w:uiPriority w:val=\"9\"/><w:qFormat/>");
        } else {
            out.push_str(&gallery_flags(&spec.id, &name));
        }
        out
    };
    // pPr children in schema order: keepNext, keepLines, spacing, ind,
    // contextualSpacing, jc, outlineLvl. The default style's omitted
    // spacing is zero or single and its omitted alignment is left; any
    // other style's omission inherits, so it is not written.
    let mut ppr = String::new();
    if spec.keep_next && !inherited.keep_next {
        ppr.push_str("<w:keepNext/>");
    }
    if spec.keep_lines && !inherited.keep_lines {
        ppr.push_str("<w:keepLines/>");
    }
    let before = if spec.default {
        spec.before.or(Some(0))
    } else {
        spec.before
    };
    let after = if spec.default {
        spec.after.or(Some(0))
    } else {
        spec.after
    };
    let line = if spec.default {
        spec.line.or(Some((240, "auto")))
    } else {
        spec.line
    };
    let write_before = before.is_some_and(|value| value != inherited.before);
    let write_after = after.is_some_and(|value| value != inherited.after);
    let write_line = line
        .is_some_and(|(twips, rule)| twips != inherited.line_twips || rule != inherited.line_rule);
    if write_before || write_after || write_line {
        ppr.push_str("<w:spacing");
        if write_before {
            ppr.push_str(&format!(" w:before=\"{}\"", before.unwrap_or(0)));
        }
        if write_after {
            ppr.push_str(&format!(" w:after=\"{}\"", after.unwrap_or(0)));
        }
        if write_line {
            let (twips, rule) = line.unwrap_or((240, "auto"));
            ppr.push_str(&format!(" w:line=\"{twips}\" w:lineRule=\"{rule}\""));
        }
        ppr.push_str("/>");
    }
    // The default style's omitted indent is zero, as its spacing is: the
    // legend names it only when it is not, so a default indent from
    // `docDefaults` is cleared (#1498 review).
    let indent = |value: Option<i64>, given: Option<i64>| {
        if spec.default {
            indent_attr(value.or(Some(0)), Some(given.unwrap_or(0)))
        } else {
            indent_attr(value, given)
        }
    };
    let left = indent(spec.left, inherited.left);
    let right = indent(spec.right, inherited.right);
    let first = indent(spec.first_line, inherited.first_line);
    let hanging = indent(spec.hanging, inherited.hanging);
    if left.is_some() || right.is_some() || first.is_some() || hanging.is_some() {
        ppr.push_str("<w:ind");
        if let Some(value) = left {
            ppr.push_str(&format!(" w:left=\"{value}\""));
        }
        if let Some(value) = right {
            ppr.push_str(&format!(" w:right=\"{value}\""));
        }
        if let Some(value) = first {
            ppr.push_str(&format!(" w:firstLine=\"{value}\""));
        }
        if let Some(value) = hanging {
            ppr.push_str(&format!(" w:hanging=\"{value}\""));
        }
        ppr.push_str("/>");
    }
    if spec.contextual_spacing && !inherited.contextual {
        ppr.push_str("<w:contextualSpacing/>");
    }
    let align = if spec.default {
        spec.align.or(Some("left"))
    } else {
        spec.align
    };
    if let Some(align) = align.filter(|value| *value != inherited.align) {
        ppr.push_str(&format!("<w:jc w:val=\"{align}\"/>"));
    }
    if let Some(level) = spec.heading {
        ppr.push_str(&format!("<w:outlineLvl w:val=\"{}\"/>", level - 1));
    }
    if !ppr.is_empty() {
        out.push_str(&format!("<w:pPr>{ppr}</w:pPr>"));
    }
    // rPr children in schema order: rFonts, b, bCs, i, iCs, sz, szCs.
    let mut rpr = String::new();
    if inherited.font.as_deref() != Some(spec.font.as_str()) {
        let font = escape(&spec.font);
        rpr.push_str(&format!(
            "<w:rFonts w:ascii=\"{font}\" w:hAnsi=\"{font}\" w:eastAsia=\"{font}\" w:cs=\"{font}\"/>"
        ));
    }
    let toggle = |on: bool, name: &str| {
        if on {
            format!("<w:{name}/><w:{name}Cs/>")
        } else {
            format!("<w:{name} w:val=\"0\"/><w:{name}Cs w:val=\"0\"/>")
        }
    };
    if spec.bold != inherited.bold {
        rpr.push_str(&toggle(spec.bold, "b"));
    }
    if spec.italic != inherited.italic {
        rpr.push_str(&toggle(spec.italic, "i"));
    }
    if inherited.size != Some(spec.size) {
        rpr.push_str(&format!(
            "<w:sz w:val=\"{size}\"/><w:szCs w:val=\"{size}\"/>",
            size = spec.size
        ));
    }
    if !rpr.is_empty() {
        out.push_str(&format!("<w:rPr>{rpr}</w:rPr>"));
    }
    out.push_str("</w:style>");
    out
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    fn defaults_change(styles: &str) -> String {
        let mark = FormatMark {
            id: Some(7),
            author: "Ana".to_string(),
            date: String::new(),
        };
        with_style_changes(styles, &[("docDefaults".to_string(), vec![mark])])
    }

    const CARRIER: &str = "<w:rPrChange w:id=\"7\" w:author=\"Ana\"><w:rPr/></w:rPrChange>";

    #[test]
    fn a_defaults_change_lands_in_the_defaults_wherever_they_are() {
        // No docDefaults: the change makes one, first in the part (where
        // CT_Styles puts it), never in another style's run properties.
        let out = defaults_change(
            "<w:styles xmlns:w=\"W\"><w:style w:styleId=\"A\"><w:rPr><w:b/></w:rPr></w:style></w:styles>",
        );
        assert_eq!(
            out,
            format!(
                "<w:styles xmlns:w=\"W\"><w:docDefaults><w:rPrDefault><w:rPr>{CARRIER}</w:rPr>\
                 </w:rPrDefault></w:docDefaults><w:style w:styleId=\"A\"><w:rPr><w:b/></w:rPr></w:style></w:styles>"
            )
        );
        // A docDefaults with only paragraph defaults: rPrDefault goes
        // first (CT_DocDefaults is rPrDefault, then pPrDefault).
        let out = defaults_change(
            "<w:styles><w:docDefaults><w:pPrDefault><w:pPr/></w:pPrDefault></w:docDefaults></w:styles>",
        );
        assert_eq!(
            out,
            format!(
                "<w:styles><w:docDefaults><w:rPrDefault><w:rPr>{CARRIER}</w:rPr></w:rPrDefault>\
                 <w:pPrDefault><w:pPr/></w:pPrDefault></w:docDefaults></w:styles>"
            )
        );
        // An empty docDefaults element.
        let out = defaults_change("<w:styles><w:docDefaults/></w:styles>");
        assert_eq!(
            out,
            format!(
                "<w:styles><w:docDefaults><w:rPrDefault><w:rPr>{CARRIER}</w:rPr></w:rPrDefault>\
                 </w:docDefaults></w:styles>"
            )
        );
        // An existing run default takes the change last in its rPr.
        let out = defaults_change(
            "<w:styles><w:docDefaults><w:rPrDefault><w:rPr><w:sz w:val=\"22\"/></w:rPr></w:rPrDefault>\
             </w:docDefaults><w:style w:styleId=\"A\"><w:rPr><w:b/></w:rPr></w:style></w:styles>",
        );
        assert_eq!(
            out,
            format!(
                "<w:styles><w:docDefaults><w:rPrDefault><w:rPr><w:sz w:val=\"22\"/>{CARRIER}</w:rPr>\
                 </w:rPrDefault></w:docDefaults><w:style w:styleId=\"A\"><w:rPr><w:b/></w:rPr></w:style></w:styles>"
            )
        );
    }

    /// A style's `w:pPr` may hold the paragraph mark's own run properties
    /// (`CT_ParaRPr`), which CT_Style puts before the style's `w:rPr`: the
    /// changes belong to the style's own elements, never to the mark's (#1595
    /// review).
    #[test]
    fn a_style_change_lands_in_the_styles_own_ppr_and_rpr() {
        let mark = FormatMark {
            id: Some(9),
            author: "Ana".to_string(),
            date: String::new(),
        };
        let styles = "<w:styles><w:style w:styleId=\"Heading1\"><w:name w:val=\"heading 1\"/>\
                      <w:pPr><w:keepNext/><w:rPr><w:i/></w:rPr></w:pPr>\
                      <w:rPr><w:sz w:val=\"32\"/></w:rPr></w:style></w:styles>";
        assert_eq!(
            with_style_changes(styles, &[("Heading1".to_string(), vec![mark])]),
            "<w:styles><w:style w:styleId=\"Heading1\"><w:name w:val=\"heading 1\"/>\
             <w:pPr><w:keepNext/><w:rPr><w:i/></w:rPr>\
             <w:pPrChange w:id=\"9\" w:author=\"Ana\"><w:pPr/></w:pPrChange></w:pPr>\
             <w:rPr><w:sz w:val=\"32\"/><w:rPrChange w:id=\"9\" w:author=\"Ana\"><w:rPr/></w:rPrChange>\
             </w:rPr></w:style></w:styles>"
        );
    }

    const YAML: &str = "view: tracked\nstyles:\n  Normal: Times New Roman 11pt, after 6pt, line 12pt atLeast, left  # default; unannotated paragraphs use it\n  \"#\": Heading1, Times New Roman bold 24pt, before 24pt, keep-next\n  \"##\": Heading2, Arial bold italic 13.5pt, line 1.08, center\n  table: TableGrid, all borders 0.5pt\nbody: p0-p2\n";

    #[test]
    fn every_other_style_and_its_name_read_and_are_defined() {
        let specs = legend(
            "styles:\n  Normal: Arial 12pt, left  # default\n  \
             \"#\": Heading1, Arial bold 16pt, named Heading 1\n  \
             TextHeading2: Times New Roman 12pt, after 6pt, named Text Heading 2\n  \
             TOC2: Arial 11pt\n",
        );
        let ids: Vec<(&str, Option<&str>, bool)> = specs
            .iter()
            .map(|s| (s.id.as_str(), s.name.as_deref(), s.default))
            .collect();
        assert_eq!(
            ids,
            [
                ("Normal", None, true),
                ("Heading1", Some("Heading 1"), false),
                ("TextHeading2", Some("Text Heading 2"), false),
                ("TOC2", None, false),
            ]
        );
        let styles = apply("<w:styles></w:styles>", &specs);
        assert!(
            styles.contains(
                r#"<w:style w:type="paragraph" w:styleId="TextHeading2"><w:name w:val="Text Heading 2"/><w:basedOn w:val="Normal"/><w:pPr><w:spacing w:after="120"/></w:pPr><w:rPr><w:rFonts w:ascii="Times New Roman""#
            ),
            "{styles}"
        );
        assert!(
            styles.contains(r#"<w:name w:val="Heading 1"/>"#),
            "{styles}"
        );
        assert!(
            styles.contains(r#"w:styleId="TOC2"><w:name w:val="TOC2"/>"#),
            "{styles}"
        );
        assert_eq!(styles.matches("w:default=\"1\"").count(), 1, "{styles}");
    }

    #[test]
    fn a_character_style_comes_back_as_one() {
        let specs = legend(
            "styles:\n  Normal: Calibri 11pt, left\n  \
             LineNumber: character, Calibri 11pt, named line number\n",
        );
        assert_eq!(specs.len(), 2, "the character style must be read");
        let styles = apply("<w:styles></w:styles>", &specs);
        assert!(
            styles.contains(r#"<w:style w:type="character" w:styleId="LineNumber"><w:name w:val="line number"/>"#),
            "{styles}"
        );
        assert_eq!(legend("styles:\n  Normal: character, Calibri 11pt\n"), []);
    }

    #[test]
    fn a_character_style_keeps_its_font_without_a_paragraph_base() {
        // A basedOn of another kind is ignored by Word. Even when the
        // fonts match Normal, they must be written against docDefaults.
        let specs = legend(
            "styles:\n  Normal: Calibri bold italic 11pt, after 8pt, left\n  \
             LineNumber: character, Calibri bold italic 11pt, named line number\n",
        );
        let styles = apply("<w:styles></w:styles>", &specs);
        let character = styles
            .split(r#"<w:style w:type="character""#)
            .nth(1)
            .expect("a character style is written")
            .split("</w:style>")
            .next()
            .unwrap();
        assert!(!character.contains("<w:basedOn"), "{character}");
        assert!(!character.contains("<w:pPr"), "{character}");
        assert!(character.contains(r#"w:ascii="Calibri""#), "{character}");
        assert!(character.contains(r#"<w:sz w:val="22"/>"#), "{character}");
        assert!(
            character.contains("<w:b/>") && character.contains("<w:i/>"),
            "{character}"
        );
    }

    #[test]
    fn the_legend_reads() {
        let specs = legend(YAML);
        assert_eq!(specs.len(), 3, "{specs:?}");
        assert_eq!(
            (
                specs[0].font.as_str(),
                specs[0].size,
                specs[0].after,
                specs[0].line,
                specs[0].align
            ),
            (
                "Times New Roman",
                22,
                Some(120),
                Some((240, "atLeast")),
                Some("left")
            )
        );
        assert_eq!(
            (specs[1].heading, specs[1].bold, specs[1].before),
            (Some(1), true, Some(480))
        );
        assert!(specs[1].keep_next);
        assert!(!specs[0].keep_lines);
        assert!(!specs[0].contextual_spacing);
        assert_eq!(specs[0].left, None);
        assert_eq!(
            (specs[2].italic, specs[2].size, specs[2].line),
            (true, 27, Some((259, "auto")))
        );
        assert_eq!(specs[2].align, Some("center"));
    }

    #[test]
    fn a_comment_ends_at_a_hash_outside_quotes() {
        assert_eq!(without_comment("  a: b  # c"), "  a: b  ");
        assert_eq!(without_comment("title: Issue #5"), "title: Issue #5");
        assert_eq!(
            without_comment("  a: \"x  #1 \\\" #\"  # c"),
            "  a: \"x  #1 \\\" #\"  "
        );
        assert_eq!(without_comment("  \"#\": H1, A 1pt"), "  \"#\": H1, A 1pt");
    }

    #[test]
    fn what_does_not_read_is_left_out() {
        assert_eq!(
            legend("styles:\n  Normal: 11pt\n  \"#\": Heading1, Arial bold 99999pt\n"),
            []
        );
        assert_eq!(legend("styles:\n  \"#######\": Heading7, Arial 11pt\n"), []);
        assert_eq!(legend("styles:\n  table: TableGrid\n"), []);
        assert_eq!(
            legend("styles:\n  Normal: Calibri 11pt, indent 99in, left\n"),
            []
        );
        assert_eq!(
            legend("styles:\n  Normal: Calibri 11pt, indent 0.5, left\n"),
            []
        );
        assert_eq!(
            legend("styles:\n  Normal: Calibri 11pt, contextual wide, left\n"),
            []
        );
    }

    #[test]
    fn a_default_style_with_another_id_takes_the_default() {
        let specs = legend(
            "styles:\n  \"1\": Arial 12pt, left            # default\n  \"#\": 2, Calibri bold 16pt, before 12pt\n",
        );
        assert_eq!(
            specs.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            ["1", "2"]
        );
        let styles = "<w:styles><w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\"><w:name w:val=\"Normal\"/></w:style></w:styles>";
        let out = apply(styles, &specs);
        assert_eq!(out.matches("w:default=\"1\"").count(), 1, "{out}");
        assert!(out.contains("w:default=\"1\" w:styleId=\"1\""), "{out}");
        assert!(
            out.contains("<w:name w:val=\"2\"/><w:basedOn w:val=\"1\"/>"),
            "{out}"
        );
    }

    #[test]
    fn a_definition_replaces_the_styles_own() {
        let styles = "<w:styles><w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\"><w:name w:val=\"Normal\"/></w:style><w:style w:type=\"paragraph\" w:styleId=\"Quote\"><w:name w:val=\"Quote\"/></w:style></w:styles>";
        let out = apply(styles, &legend(YAML));
        assert_eq!(out.matches("w:styleId=\"Normal\"").count(), 1, "{out}");
        assert!(out.contains("w:styleId=\"Quote\""), "{out}");
        let normal = style_element(&out, "Normal");
        // The default style's spacing is the document's default.
        assert!(
            out.starts_with(
                "<w:styles><w:docDefaults><w:pPrDefault><w:pPr>\
                 <w:spacing w:after=\"120\" w:line=\"240\" w:lineRule=\"atLeast\"/>\
                 </w:pPr></w:pPrDefault></w:docDefaults>"
            ),
            "{out}"
        );
        assert!(!normal.contains("<w:spacing"), "{normal}");
        assert!(!normal.contains("<w:jc"), "{normal}");
        assert!(!normal.contains("<w:b"), "{normal}");
        assert!(
            normal.contains(
                "<w:rFonts w:ascii=\"Times New Roman\" w:hAnsi=\"Times New Roman\" \
                 w:eastAsia=\"Times New Roman\" w:cs=\"Times New Roman\"/>"
            ),
            "{normal}"
        );
        assert!(
            normal.contains("<w:sz w:val=\"22\"/><w:szCs w:val=\"22\"/>"),
            "{normal}"
        );
        assert!(
            out.contains("<w:name w:val=\"heading 1\"/><w:basedOn w:val=\"Normal\"/>"),
            "{out}"
        );
    }

    fn style_element(styles: &str, id: &str) -> String {
        let key = format!("w:styleId=\"{id}\"");
        let at = styles
            .find(&key)
            .unwrap_or_else(|| panic!("no {id} in {styles}"));
        let start = styles[..at].rfind("<w:style ").unwrap_or(at);
        let end = styles[at..]
            .find("</w:style>")
            .map(|i| at + i + "</w:style>".len())
            .unwrap_or(styles.len());
        styles[start..end].to_string()
    }

    #[test]
    fn an_indent_contextual_spacing_and_keep_lines_are_written() {
        let specs = legend(
            "styles:\n  \
             Normal: Times New Roman 11pt, indent -0.0035in, hanging 0.007in, justify\n  \
             \"#\": Heading1, Times New Roman bold 16pt, keep-next, keep-lines\n  \
             ListParagraph: Times New Roman 11pt, indent 0.5in, first-line 0.25in, \
             contextual spacing, named List Paragraph\n",
        );
        assert_eq!(specs[0].left, Some(-5));
        assert_eq!(specs[0].hanging, Some(10));
        assert!(specs[1].keep_lines);
        assert_eq!(specs[2].left, Some(720));
        assert_eq!(specs[2].first_line, Some(360));
        assert!(specs[2].contextual_spacing);
        let out = apply("<w:styles></w:styles>", &specs);
        let list = style_element(&out, "ListParagraph");
        assert!(
            list.contains("<w:ind w:left=\"720\" w:firstLine=\"360\"/>"),
            "{list}"
        );
        assert!(list.contains("<w:contextualSpacing/>"), "{list}");
        assert!(!list.contains("<w:rPr"), "{list}");
        let heading = style_element(&out, "Heading1");
        assert!(heading.contains("<w:keepNext/><w:keepLines/>"), "{heading}");
        let normal = style_element(&out, "Normal");
        assert!(
            normal.contains("<w:ind w:left=\"-5\" w:hanging=\"10\"/>"),
            "{normal}"
        );
    }

    #[test]
    fn a_right_indent_is_written_and_a_bare_right_still_aligns() {
        // `right 0.5in` is the style's right indent (#1489); `right` alone
        // is still its alignment, as on id lines.
        let specs = legend(
            "styles:\n  \
             Normal: Times New Roman 11pt, left\n  \
             Quote: Times New Roman 11pt, indent 0.5in, right 0.5in, named Quote Text\n  \
             Flush: Times New Roman 11pt, right\n  \
             Back: Times New Roman 11pt, right 0in\n",
        );
        assert_eq!(specs[1].right, Some(720));
        assert_eq!(specs[2].right, None);
        assert_eq!(specs[2].align, Some("right"));
        assert_eq!(specs[3].right, Some(0));
        let out = apply("<w:styles></w:styles>", &specs);
        let quote = style_element(&out, "Quote");
        assert!(
            quote.contains("<w:ind w:left=\"720\" w:right=\"720\"/>"),
            "{quote}"
        );
        let flush = style_element(&out, "Flush");
        assert!(flush.contains("<w:jc w:val=\"right\"/>"), "{flush}");
        assert!(!flush.contains("<w:ind"), "{flush}");
        // An explicit zero is kept, as `indent 0in` is: the source said it.
        let back = style_element(&out, "Back");
        assert!(back.contains("<w:ind w:right=\"0\"/>"), "{back}");
    }

    #[test]
    fn a_theme_font_in_the_defaults_is_not_the_named_font() {
        // Word draws the theme's font where `asciiTheme` is set, whatever
        // `ascii` says, so a legend naming the `ascii` font still writes it.
        let styles = "<w:styles><w:docDefaults><w:rPrDefault><w:rPr>\
            <w:rFonts w:ascii=\"Aptos\" w:hAnsi=\"Aptos\" w:asciiTheme=\"minorHAnsi\" w:hAnsiTheme=\"minorHAnsi\"/>\
            <w:sz w:val=\"22\"/></w:rPr></w:rPrDefault></w:docDefaults></w:styles>";
        let out = apply(styles, &legend("styles:\n  Normal: Aptos 11pt, left\n"));
        let normal = style_element(&out, "Normal");
        assert!(normal.contains("<w:rFonts w:ascii=\"Aptos\""), "{normal}");
    }

    #[test]
    fn a_style_omits_properties_the_defaults_already_give() {
        let styles = "<w:styles><w:docDefaults><w:rPrDefault><w:rPr>\
            <w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\" w:eastAsia=\"Calibri\" w:cs=\"Calibri\"/>\
            <w:sz w:val=\"22\"/><w:szCs w:val=\"22\"/></w:rPr></w:rPrDefault>\
            <w:pPrDefault><w:pPr><w:spacing w:after=\"160\" w:line=\"259\" w:lineRule=\"auto\"/>\
            </w:pPr></w:pPrDefault></w:docDefaults></w:styles>";
        let specs = legend(
            "styles:\n  \
             Normal: Calibri 11pt, after 8pt, line 1.08, left\n  \
             \"#\": Heading1, Calibri bold 16pt, before 12pt, after 4pt, keep-next, keep-lines\n  \
             ListParagraph: Calibri 11pt, indent 0.5in, contextual spacing, named List Paragraph\n  \
             Quote: Calibri 11pt, left\n",
        );
        let out = apply(styles, &specs);
        assert!(
            out.contains(
                "<w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
                 <w:name w:val=\"Normal\"/><w:qFormat/></w:style>"
            ),
            "{out}"
        );
        let list = style_element(&out, "ListParagraph");
        assert!(list.contains("<w:ind w:left=\"720\"/>"), "{list}");
        assert!(list.contains("<w:contextualSpacing/>"), "{list}");
        assert!(!list.contains("<w:rPr"), "{list}");
        assert!(
            list.contains("<w:name w:val=\"List Paragraph\"/>"),
            "{list}"
        );
        // A built-in style keeps Word's gallery flags: List Paragraph is
        // priority 34 and quick-format, as Word itself writes it.
        assert!(
            list.contains(
                "<w:basedOn w:val=\"Normal\"/><w:uiPriority w:val=\"34\"/><w:qFormat/><w:pPr>"
            ),
            "{list}"
        );
        assert!(
            style_element(&out, "Quote").contains("<w:uiPriority w:val=\"29\"/><w:qFormat/>"),
            "{out}"
        );
        let heading = style_element(&out, "Heading1");
        assert!(
            heading.contains(
                "<w:pPr><w:keepNext/><w:keepLines/><w:spacing w:before=\"240\" w:after=\"80\"/>\
                 <w:outlineLvl w:val=\"0\"/></w:pPr>"
            ),
            "{heading}"
        );
        assert!(
            heading.contains(
                "<w:rPr><w:b/><w:bCs/><w:sz w:val=\"32\"/><w:szCs w:val=\"32\"/></w:rPr>"
            ),
            "{heading}"
        );
        // Quote is not bold, and Normal's legend is not bold either, so no toggle.
        let quote = style_element(&out, "Quote");
        assert!(!quote.contains("<w:rPr"), "{quote}");
        assert!(!quote.contains("<w:pPr"), "{quote}");
    }

    #[test]
    fn the_default_style_s_omitted_indent_is_none() {
        // The default style's line names an indent only when it is not
        // zero, so an omitted one is zero: a template's own default
        // indent may not survive under it (#1498 review).
        let styles = "<w:styles><w:docDefaults><w:pPrDefault><w:pPr>\
            <w:ind w:left=\"360\" w:right=\"720\" w:hanging=\"360\"/>\
            </w:pPr></w:pPrDefault></w:docDefaults></w:styles>";
        let out = apply(
            styles,
            &legend("styles:\n  Normal: Calibri 11pt, left\n  Quote: Calibri 11pt, left\n"),
        );
        let normal = style_element(&out, "Normal");
        assert!(
            normal.contains("<w:ind w:left=\"0\" w:right=\"0\" w:hanging=\"0\"/>"),
            "{normal}"
        );
        // A style based on the default inherits its zeros.
        assert!(!style_element(&out, "Quote").contains("<w:ind"), "{out}");
        // With no default indent there is nothing to clear.
        let out = apply(
            "<w:styles></w:styles>",
            &legend("styles:\n  Normal: Calibri 11pt, left\n"),
        );
        assert!(!style_element(&out, "Normal").contains("<w:ind"), "{out}");
    }

    #[test]
    fn a_run_property_is_written_when_it_differs_from_the_base() {
        let specs =
            legend("styles:\n  Normal: Calibri bold 11pt, left\n  Quote: Calibri 11pt, left\n");
        let out = apply("<w:styles></w:styles>", &specs);
        let quote = style_element(&out, "Quote");
        assert!(
            quote.contains("<w:b w:val=\"0\"/><w:bCs w:val=\"0\"/>"),
            "{quote}"
        );
        assert!(!quote.contains("w:rFonts"), "{quote}");
        assert!(!quote.contains("<w:sz"), "{quote}");
        assert!(!quote.contains("<w:jc"), "{quote}");
    }

    /// The `w:pPrDefault` of a styles part, or nothing.
    fn paragraph_defaults(styles: &str) -> &str {
        let Some(start) = styles.find("<w:pPrDefault") else {
            return "";
        };
        let end = styles[start..]
            .find("</w:pPrDefault>")
            .map_or(styles.len(), |i| start + i + "</w:pPrDefault>".len());
        &styles[start..end]
    }

    #[test]
    fn the_default_style_s_spacing_is_the_documents_default() {
        // Word keeps the body's spacing in `docDefaults`. A table style's
        // own spacing then wins in its cells; over `Normal`'s it would not
        // (ISO/IEC 29500 style precedence), so the cells would grow.
        let styles = "<w:styles><w:docDefaults><w:rPrDefault><w:rPr><w:sz w:val=\"22\"/></w:rPr></w:rPrDefault>\
            <w:pPrDefault><w:pPr><w:spacing w:after=\"160\" w:line=\"259\" w:lineRule=\"auto\"/></w:pPr></w:pPrDefault>\
            </w:docDefaults></w:styles>";
        let out = apply(
            styles,
            &legend(
                "styles:\n  Normal: Calibri 11pt, after 10pt, line 1.15, left\n  \
                 Quote: Calibri 11pt, before 6pt, left\n",
            ),
        );
        assert_eq!(
            paragraph_defaults(&out),
            "<w:pPrDefault><w:pPr><w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>\
             </w:pPr></w:pPrDefault>",
            "{out}"
        );
        let normal = style_element(&out, "Normal");
        assert!(!normal.contains("<w:spacing"), "{normal}");
        // A style based on the default still writes what it sets apart.
        let quote = style_element(&out, "Quote");
        assert!(quote.contains("<w:spacing w:before=\"120\"/>"), "{quote}");
        // An omitted spacing is none and single, as the legend says, so a
        // template's own default does not survive under it.
        let out = apply(styles, &legend("styles:\n  Normal: Calibri 11pt, left\n"));
        assert_eq!(
            paragraph_defaults(&out),
            "<w:pPrDefault><w:pPr><w:spacing w:after=\"0\" w:line=\"240\" w:lineRule=\"auto\"/>\
             </w:pPr></w:pPrDefault>",
            "{out}"
        );
        assert!(
            !style_element(&out, "Normal").contains("<w:spacing"),
            "{out}"
        );
    }

    #[test]
    fn the_default_spacing_lands_where_the_schema_puts_it() {
        let specs = legend(
            "styles:\n  Normal: Calibri 11pt, before 3pt, after 6pt, line 12pt exact, left\n",
        );
        let spacing =
            "<w:spacing w:before=\"60\" w:after=\"120\" w:line=\"240\" w:lineRule=\"exact\"/>";
        // No docDefaults: one is made, first in the part.
        let out = apply("<w:styles><w:latentStyles/></w:styles>", &specs);
        assert!(
            out.starts_with(&format!(
                "<w:styles><w:docDefaults><w:pPrDefault><w:pPr>{spacing}</w:pPr></w:pPrDefault>\
                 </w:docDefaults><w:latentStyles/>"
            )),
            "{out}"
        );
        // Run defaults only: the paragraph defaults follow them.
        let out = apply(
            "<w:styles><w:docDefaults><w:rPrDefault><w:rPr/></w:rPrDefault></w:docDefaults></w:styles>",
            &specs,
        );
        assert!(
            out.contains(&format!(
                "<w:rPrDefault><w:rPr/></w:rPrDefault><w:pPrDefault><w:pPr>{spacing}</w:pPr>\
                 </w:pPrDefault></w:docDefaults>"
            )),
            "{out}"
        );
        // Other paragraph defaults stay, the spacing between them in
        // CT_PPrBase order (after widowControl, before ind).
        let out = apply(
            "<w:styles><w:docDefaults><w:pPrDefault><w:pPr><w:widowControl/>\
             <w:spacing w:after=\"160\" w:line=\"259\" w:lineRule=\"auto\"/><w:ind w:left=\"0\"/></w:pPr>\
             </w:pPrDefault></w:docDefaults></w:styles>",
            &specs,
        );
        assert_eq!(
            paragraph_defaults(&out),
            format!(
                "<w:pPrDefault><w:pPr><w:widowControl/>{spacing}<w:ind w:left=\"0\"/></w:pPr></w:pPrDefault>"
            ),
            "{out}"
        );
        let out = apply(
            "<w:styles><w:docDefaults><w:pPrDefault><w:pPr><w:jc w:val=\"left\"/></w:pPr>\
             </w:pPrDefault></w:docDefaults></w:styles>",
            &specs,
        );
        assert_eq!(
            paragraph_defaults(&out),
            format!("<w:pPrDefault><w:pPr>{spacing}<w:jc w:val=\"left\"/></w:pPr></w:pPrDefault>"),
            "{out}"
        );
        // An empty pPrDefault, or one without its pPr.
        for empty in [
            "<w:pPrDefault/>",
            "<w:pPrDefault></w:pPrDefault>",
            "<w:pPrDefault><w:pPr/></w:pPrDefault>",
        ] {
            let out = apply(
                &format!("<w:styles><w:docDefaults>{empty}</w:docDefaults></w:styles>"),
                &specs,
            );
            assert_eq!(
                paragraph_defaults(&out),
                format!("<w:pPrDefault><w:pPr>{spacing}</w:pPr></w:pPrDefault>"),
                "{out}"
            );
        }
    }
}
