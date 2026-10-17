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
//! indent (`indent 0.5in`, and `first-line` or `hanging` when set),
//! contextual spacing and alignment. A heading's line names its style, its
//! font, size, bold and italic, and only the spacing, keep-next, keep-lines,
//! indent, contextual spacing and alignment that differ from the default
//! style's, which it is based on. Every other paragraph style the body uses
//! follows, keyed by its id, described like a heading (`TOC2: Arial 12pt,
//! after 6pt`). A line ends `, named Text Heading 2` when the style's name
//! is not its id (for a heading `HeadingN`, not `heading N`).
//!
//! The writer sets a property only when the legend's value differs from
//! what the style already inherits: the document defaults for the default
//! style, and that style for every other.

use super::from_docx::Element;
use super::xml::escape;

/// One style the legend describes.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct StyleSpec {
    pub id: String,
    /// The style's name, when the legend gives one (`, named …`).
    pub name: Option<String>,
    /// The `#` count for a heading.
    pub heading: Option<u32>,
    /// The default paragraph style, which the others are based on.
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
fn twips_in(value: &str) -> Option<i64> {
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
    let mut clauses = value.split(", ");
    let (font, bold, italic, size) = font_clause(clauses.next()?)?;
    let mut spec = StyleSpec {
        id: id.to_string(),
        name,
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

/// The styles the YAML header's `styles:` block describes: the default
/// style's line (`Normal: …`, its id quoted when it must be), each
/// heading's (`"#": Heading1, …`) and each other style's (`TOC2: …`), after
/// the default's. Lines that do not read are left out.
/// The legend's `table:` style, which every table takes: its id and, when
/// the legend says `all borders 0.5pt`, that border in eighths of a point.
pub(super) fn table_style(yaml: &str) -> Option<(String, Option<u32>, Option<String>)> {
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
) -> std::collections::BTreeMap<String, (Option<u32>, Option<String>)> {
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
type TableStyleEntry = (String, Option<u32>, Option<String>);

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
            border = Some((pt * 8.0).round() as u32);
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
    let defaults = document_defaults(styles);
    let mut out = styles.to_string();
    let base = specs.iter().find(|s| s.default);
    let from_base = base.map(effective);
    if base.is_some() {
        out = out.replace(
            "w:type=\"paragraph\" w:default=\"1\"",
            "w:type=\"paragraph\"",
        );
    }
    for spec in specs {
        out = without_style(&out, &spec.id);
        let inherited = if spec.default {
            &defaults
        } else {
            from_base.as_ref().unwrap_or(&defaults)
        };
        let definition = definition(spec, base.map(|b| b.id.as_str()), inherited);
        if let Some(at) = out.rfind("</w:styles>") {
            out.insert_str(at, &definition);
        }
    }
    out
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
        let mut out =
            format!("<w:style w:type=\"paragraph\" w:styleId=\"{id}\"><w:name w:val=\"{name}\"/>");
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
    let left = indent_attr(spec.left, inherited.left);
    let first = indent_attr(spec.first_line, inherited.first_line);
    let hanging = indent_attr(spec.hanging, inherited.hanging);
    if left.is_some() || first.is_some() || hanging.is_some() {
        ppr.push_str("<w:ind");
        if let Some(value) = left {
            ppr.push_str(&format!(" w:left=\"{value}\""));
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
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

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
        assert!(
            normal.contains("<w:spacing w:after=\"120\" w:line=\"240\" w:lineRule=\"atLeast\"/>"),
            "{normal}"
        );
        assert!(!normal.contains("w:before=\"0\""), "{normal}");
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
}
