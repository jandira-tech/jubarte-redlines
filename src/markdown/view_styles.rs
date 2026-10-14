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
//! spacing (none printed is single) and alignment. A heading's line names
//! its style, its font, size, bold and italic, and only the spacing,
//! keep-next and alignment that differ from the default style's, which it
//! is based on. Every other paragraph style the body uses follows, keyed by
//! its id, described like a heading (`TOC2: Arial 12pt, after 6pt`). A line
//! ends `, named Text Heading 2` when the style's name is not its id (for a
//! heading `HeadingN`, not `heading N`).

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
    pub align: Option<&'static str>,
}

/// The largest font size and spacing the legend can set: Word's 1638pt.
const MOST_POINTS: f64 = 1638.0;

fn points(value: &str, scale: f64) -> Option<u32> {
    let n: f64 = value.strip_suffix("pt")?.parse().ok()?;
    // `as` cannot overflow: the bound is checked first.
    (n.is_finite() && (0.0..=MOST_POINTS).contains(&n)).then(|| (n * scale).round() as u32)
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
        align: None,
    };
    for clause in clauses {
        match clause.split_once(' ') {
            Some(("before", v)) => spec.before = Some(points(v, 20.0)?),
            Some(("after", v)) => spec.after = Some(points(v, 20.0)?),
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
    let mut out = styles.to_string();
    let base = specs.iter().find(|s| s.default);
    if base.is_some() {
        out = out.replace(
            "w:type=\"paragraph\" w:default=\"1\"",
            "w:type=\"paragraph\"",
        );
    }
    for spec in specs {
        out = without_style(&out, &spec.id);
        let definition = definition(spec, base.map(|b| b.id.as_str()));
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

/// The `w:style` element of `spec`, based on style `base` unless it is the
/// default.
fn definition(spec: &StyleSpec, base: Option<&str>) -> String {
    let id = escape(&spec.id);
    let name = match (&spec.name, spec.heading, spec.id.strip_prefix("Heading")) {
        (Some(name), ..) => escape(name).to_string(),
        (None, Some(_), Some(n)) if n.parse::<u32>().is_ok() => format!("heading {n}"),
        _ => id.to_string(),
    };
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
        }
        out
    };
    // pPr children in schema order: keepNext, spacing, jc, outlineLvl.
    let mut ppr = String::new();
    if spec.keep_next {
        ppr.push_str("<w:keepNext/>");
    }
    // The default style says all its spacing; a heading only what differs.
    let explicit = spec.default;
    let before = spec.before.or(explicit.then_some(0));
    let after = spec.after.or(explicit.then_some(0));
    let line = spec.line.or(explicit.then_some((240, "auto")));
    if before.is_some() || after.is_some() || line.is_some() {
        ppr.push_str("<w:spacing");
        if let Some(b) = before {
            ppr.push_str(&format!(" w:before=\"{b}\""));
        }
        if let Some(a) = after {
            ppr.push_str(&format!(" w:after=\"{a}\""));
        }
        if let Some((l, rule)) = line {
            ppr.push_str(&format!(" w:line=\"{l}\" w:lineRule=\"{rule}\""));
        }
        ppr.push_str("/>");
    }
    if let Some(align) = spec.align {
        ppr.push_str(&format!("<w:jc w:val=\"{align}\"/>"));
    }
    if let Some(level) = spec.heading {
        ppr.push_str(&format!("<w:outlineLvl w:val=\"{}\"/>", level - 1));
    }
    if !ppr.is_empty() {
        out.push_str(&format!("<w:pPr>{ppr}</w:pPr>"));
    }
    // rPr children in schema order: rFonts, b, bCs, i, iCs, sz, szCs.
    let font = escape(&spec.font);
    let toggle = |on: bool, name: &str| {
        if on {
            format!("<w:{name}/><w:{name}Cs/>")
        } else {
            format!("<w:{name} w:val=\"0\"/><w:{name}Cs w:val=\"0\"/>")
        }
    };
    out.push_str(&format!(
        "<w:rPr><w:rFonts w:ascii=\"{font}\" w:hAnsi=\"{font}\" w:eastAsia=\"{font}\" w:cs=\"{font}\"/>{}{}<w:sz w:val=\"{size}\"/><w:szCs w:val=\"{size}\"/></w:rPr></w:style>",
        toggle(spec.bold, "b"),
        toggle(spec.italic, "i"),
        size = spec.size,
    ));
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
        assert!(out.contains("<w:spacing w:before=\"0\" w:after=\"120\" w:line=\"240\" w:lineRule=\"atLeast\"/><w:jc w:val=\"left\"/>"), "{out}");
        assert!(
            out.contains("<w:name w:val=\"heading 1\"/><w:basedOn w:val=\"Normal\"/>"),
            "{out}"
        );
        assert!(
            out.contains("<w:b w:val=\"0\"/><w:bCs w:val=\"0\"/><w:i w:val=\"0\"/>"),
            "{out}"
        );
    }
}
