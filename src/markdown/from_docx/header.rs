// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The YAML header of the agent view.

use std::collections::HashSet;

use super::agent::{Handles, RevTag};
use super::ooxml::Element;

/// Where the owner's name came from.
pub(crate) enum Owner {
    Creator(String),
    LastModifiedBy(String),
    None,
}

/// Where the page count and markers came from.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PagesSource {
    /// `Options::pages` from the layout pass.
    Layout,
    /// `w:lastRenderedPageBreak` in the file.
    Cached,
    /// Hard page and section breaks only.
    Estimated,
}

/// A header or footer of one section.
#[derive(Clone)]
pub(crate) struct StoryFact {
    /// `header` or `footer`.
    pub kind: &'static str,
    /// `first`, `default`, `even`.
    pub ty: &'static str,
    /// `header2` (the part `header2.xml`, first paragraph), `footer1`, `header2.p2`.
    pub id: String,
    pub text: String,
    /// `center`, `right`, or `None` for left.
    pub align: Option<&'static str>,
    pub active: bool,
}

/// A section of the body (Task 11 fills these).
pub(crate) struct SectionFact {
    /// Paragraph range, inclusive.
    pub first: usize,
    pub last: usize,
    pub sect_pr: Option<Element>,
    pub stories: Vec<StoryFact>,
    pub title_page: bool,
    pub columns: usize,
}

pub(crate) struct CommentFact {
    pub id: String,
    pub author: Option<String>,
    pub parent: Option<String>,
}

pub(crate) struct Facts<'a> {
    pub source: &'a str,
    /// `None` tracked, `Some(true)` accept-all, `Some(false)` reject-all.
    pub resolved: Option<bool>,
    pub comments_inline: bool,
    pub tracking_on: bool,
    pub tags: Vec<RevTag>,
    pub marks: usize,
    pub format_changes: usize,
    /// The author of every formatting change, as stored (`None` when the
    /// change names none).
    pub format_authors: Vec<Option<String>>,
    pub comments: Vec<CommentFact>,
    pub handles: &'a Handles,
    /// Resolved comment ids (`w15:done`), from `agent::Threads`.
    pub done: &'a HashSet<String>,
    pub owner: Owner,
    pub paragraphs: usize,
    pub tables: usize,
    pub pages: usize,
    pub pages_source: PagesSource,
    /// `range:` line after `body:` when a selection is active (Task 12).
    pub range: Option<String>,
    pub styles: Option<&'a Element>,
    pub theme: Option<&'a Element>,
    pub default_style: Option<&'a str>,
    /// (level, style id) for every heading level used, lowest first.
    pub heading_styles: Vec<(usize, String)>,
    pub table_styles: Vec<String>,
    /// The body's sections in order; the first one is described in full.
    pub sections: Vec<SectionFact>,
}

/// `key: value` padded to 35 columns then `# comment`; a key/value longer
/// than 33 characters takes two spaces. An empty comment writes none.
fn kv(out: &mut String, key_value: &str, comment: Option<&str>) {
    match comment.filter(|c| !c.is_empty()) {
        Some(c) if key_value.chars().count() <= 33 => {
            out.push_str(&format!("{key_value:<35}# {c}\n"));
        }
        Some(c) => out.push_str(&format!("{key_value}  # {c}\n")),
        None => {
            out.push_str(key_value);
            out.push('\n');
        }
    }
}

/// `value` as a YAML scalar: plain when a YAML reader takes it back as the
/// same string, else double-quoted (a JSON string is a valid YAML one).
/// `flow` values sit inside `{…}`, where `,`, `[`, `]`, `{` and `}` end them.
pub(crate) fn scalar(value: &str, flow: bool) -> String {
    const RESERVED: [&str; 12] = [
        "true", "false", "yes", "no", "on", "off", "y", "n", "null", "~", ".nan", ".inf",
    ];
    let plain = !value.is_empty()
        && value.trim() == value
        && !value.chars().any(char::is_control)
        && !value.starts_with([
            '-', '?', ':', ',', '[', ']', '{', '}', '#', '&', '*', '!', '|', '>', '\'', '"', '%',
            '@', '`',
        ])
        && !value.contains(": ")
        && !value.contains(" #")
        && !value.ends_with(':')
        && !(flow && value.contains([',', '[', ']', '{', '}']))
        && !RESERVED.contains(&value.to_ascii_lowercase().as_str())
        && value.parse::<f64>().is_err();
    if plain {
        value.to_string()
    } else {
        serde_json::to_string(value).unwrap_or_else(|_| format!("\"{value}\""))
    }
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

fn join(items: &[String]) -> String {
    items.join(", ")
}

pub(crate) fn render(f: &Facts) -> String {
    let mut out = String::from("---\n");
    kv(
        &mut out,
        &format!("source: {}", scalar(f.source, false)),
        None,
    );
    let handle_list: Vec<String> = f
        .handles
        .order
        .iter()
        .filter(|a| {
            f.tags
                .iter()
                .any(|t| t.author.as_deref() == Some(a.as_str()))
        })
        .filter_map(|a| f.handles.by_author.get(a).cloned())
        .collect();
    let by = if handle_list.is_empty() {
        String::new()
    } else {
        format!(" by {}", join(&handle_list))
    };
    let threads: Vec<&CommentFact> = f.comments.iter().filter(|c| c.parent.is_none()).collect();
    match (f.resolved, f.comments_inline) {
        (None, true) => kv(
            &mut out,
            "view: tracked",
            Some("revisions as CriticMarkup, comments inline"),
        ),
        (None, false) => kv(
            &mut out,
            "view: tracked, comments hidden",
            Some(&format!(
                "{} ({}); carrying paragraphs are marked",
                plural(threads.len(), "thread open", "threads open"),
                plural(f.comments.len(), "comment", "comments")
            )),
        ),
        (Some(accept), _) => kv(
            &mut out,
            &format!("view: {}", if accept { "accept-all" } else { "reject-all" }),
            Some(&format!(
                "{}{by} shown as {}; file unchanged",
                plural(f.tags.len(), "revision", "revisions"),
                if accept { "accepted" } else { "rejected" }
            )),
        ),
    }
    if f.tracking_on {
        kv(
            &mut out,
            "track_changes: on",
            Some("w:trackRevisions set; edits are tracked"),
        );
    } else {
        kv(
            &mut out,
            "track_changes: off",
            Some("w:trackRevisions not set; new edits are not tracked unless edit sets it"),
        );
    }
    if f.tags.is_empty() {
        let formatting = (f.format_changes > 0)
            .then(|| plural(f.format_changes, "formatting change", "formatting changes"));
        kv(&mut out, "revisions: 0", formatting.as_deref());
    } else {
        let count = |kind: &str| f.tags.iter().filter(|t| t.kind == kind).count();
        let mut parts = Vec::new();
        for (kind, one, many) in [
            ("ins", "insertion", "insertions"),
            ("del", "deletion", "deletions"),
            ("sub", "substitution", "substitutions"),
            ("mark", "paragraph mark", "paragraph marks"),
            ("row", "table row", "table rows"),
            ("cell", "table cell", "table cells"),
        ] {
            let n = count(kind);
            if n > 0 {
                parts.push(plural(n, one, many));
            }
        }
        let mut marks = plural(f.marks, "Word mark", "Word marks");
        if f.format_changes > 0 {
            marks.push_str(&format!(", {} formatting", f.format_changes));
        }
        kv(
            &mut out,
            &format!("revisions: {}", f.tags.len()),
            Some(&format!("{} ({marks})", join(&parts))),
        );
    }
    if f.comments.is_empty() {
        kv(&mut out, "comments: 0", None);
    } else {
        let open = threads.iter().filter(|c| !f.done.contains(&c.id)).count();
        let mut key = format!("comments: {}", plural(open, "thread open", "threads open"));
        if open < threads.len() {
            key.push_str(&format!(", {} resolved", threads.len() - open));
        }
        let list: Vec<String> = threads
            .iter()
            .map(|t| {
                let replies: Vec<String> = f
                    .comments
                    .iter()
                    .filter(|c| c.parent.as_deref() == Some(&t.id))
                    .map(|c| format!("c{}", c.id))
                    .collect();
                match replies.len() {
                    0 => format!("c{}", t.id),
                    1 => format!("c{} (+ reply {})", t.id, replies[0]),
                    _ => format!("c{} (+ replies {})", t.id, join(&replies)),
                }
            })
            .collect();
        kv(
            &mut out,
            &key,
            Some(&format!(
                "{}: {}",
                plural(f.comments.len(), "comment", "comments"),
                join(&list)
            )),
        );
    }
    out.push_str("authors:\n");
    match &f.owner {
        Owner::Creator(name) => kv(
            &mut out,
            &format!("  document_owner: {}", scalar(name, false)),
            Some("dc:creator"),
        ),
        Owner::LastModifiedBy(name) => kv(
            &mut out,
            &format!("  document_owner: {}", scalar(name, false)),
            Some("cp:lastModifiedBy; no dc:creator"),
        ),
        Owner::None => kv(
            &mut out,
            "  document_owner: none",
            Some("no docProps/core.xml"),
        ),
    }
    for author in &f.handles.order {
        let Some(handle) = f.handles.by_author.get(author) else {
            continue;
        };
        let revisions = f
            .tags
            .iter()
            .filter(|t| t.author.as_deref() == Some(author.as_str()))
            .count();
        let comments = f
            .comments
            .iter()
            .filter(|c| c.author.as_deref() == Some(author.as_str()))
            .count();
        let formatting = f
            .format_authors
            .iter()
            .filter(|a| a.as_deref() == Some(author.as_str()))
            .count();
        let mut parts = Vec::new();
        if revisions > 0 {
            parts.push(plural(revisions, "revision", "revisions"));
        }
        if formatting > 0 {
            parts.push(plural(
                formatting,
                "formatting change",
                "formatting changes",
            ));
        }
        if comments > 0 {
            parts.push(plural(comments, "comment", "comments"));
        }
        if let Some(date) = f.handles.unique_date(author) {
            parts.push(date.to_string());
        } else if let Some(range) = f.handles.date_range(author) {
            parts.push(range);
        }
        kv(
            &mut out,
            &format!("  {handle}: {}", scalar(author, false)),
            Some(&join(&parts)),
        );
    }
    // `p0-pN`, or nothing to name in a body without paragraphs.
    let span = f
        .paragraphs
        .checked_sub(1)
        .map_or_else(|| "no paragraphs".to_string(), |last| format!("p0-p{last}"));
    kv(
        &mut out,
        &format!(
            "body: {span}, {}, {}",
            plural(f.tables, "table", "tables"),
            plural(f.pages, "page", "pages")
        ),
        Some(match f.pages_source {
            PagesSource::Layout => "pages from layout",
            PagesSource::Cached => "pages from Word's cached layout",
            PagesSource::Estimated => "page count estimated from breaks",
        }),
    );
    if let Some(range) = &f.range {
        let of = if f.paragraphs == 0 {
            String::new()
        } else {
            format!(" of {span}")
        };
        kv(&mut out, &format!("range: {range}{of}"), None);
    }
    part_two(&mut out, f);
    out.push_str("---\n");
    out
}

fn twips(e: &Element, attr: &str) -> Option<f64> {
    e.attr(attr).and_then(|v| v.parse::<f64>().ok())
}

/// `Letter portrait, margins 1in, header/footer 0.5in` for a `w:sectPr`.
fn page_setup(sect: &Element) -> String {
    let size = sect.child("pgSz");
    let (w, h) = (
        size.and_then(|s| twips(s, "w")).unwrap_or(12240.0),
        size.and_then(|s| twips(s, "h")).unwrap_or(15840.0),
    );
    let landscape = size.and_then(|s| s.attr("orient")) == Some("landscape") || w > h;
    let (short, long) = if w < h { (w, h) } else { (h, w) };
    let name = match (short as i64, long as i64) {
        (12240, 15840) => "Letter".to_string(),
        (12240, 20160) => "Legal".to_string(),
        (11906, 16838) => "A4".to_string(),
        _ => format!(
            "{}x{}",
            super::agent::inches(w).trim_end_matches("in"),
            super::agent::inches(h)
        ),
    };
    let mut out = format!(
        "{name} {}",
        if landscape { "landscape" } else { "portrait" }
    );
    if let Some(m) = sect.child("pgMar") {
        let side = |a: &str| twips(m, a).unwrap_or(1440.0);
        let (t, r, b, l) = (side("top"), side("right"), side("bottom"), side("left"));
        let inch = super::agent::inches;
        if t == r && r == b && b == l {
            out.push_str(&format!(", margins {}", inch(t)));
        } else {
            out.push_str(&format!(
                ", margins top {}, right {}, bottom {}, left {}",
                inch(t),
                inch(r),
                inch(b),
                inch(l)
            ));
        }
        let (hd, ft) = (
            twips(m, "header").unwrap_or(720.0),
            twips(m, "footer").unwrap_or(720.0),
        );
        if hd == ft {
            out.push_str(&format!(", header/footer {}", inch(hd)));
        } else {
            out.push_str(&format!(", header {}, footer {}", inch(hd), inch(ft)));
        }
        if let Some(g) = twips(m, "gutter").filter(|&g| g > 0.0) {
            out.push_str(&format!(", gutter {}", inch(g)));
        }
    }
    out
}

/// Resolved paragraph/run properties of a style through its `basedOn`
/// chain and the document defaults.
#[derive(Default)]
struct Resolved {
    font: Option<String>,
    size_half_points: Option<f64>,
    bold: bool,
    italic: bool,
    before: Option<f64>,
    after: Option<f64>,
    line: Option<(f64, String)>,
    keep_next: bool,
    align: Option<String>,
}

fn style_by_id<'a>(styles: &'a Element, id: &str) -> Option<&'a Element> {
    styles
        .children_named("style")
        .find(|s| s.attr("styleId") == Some(id))
}

fn font_of(rpr: &Element, theme: Option<&Element>) -> Option<String> {
    let fonts = rpr.child("rFonts")?;
    if let Some(name) = fonts.attr("ascii") {
        return Some(name.to_string());
    }
    let which = fonts
        .attr("asciiTheme")
        .or_else(|| fonts.attr("hAnsiTheme"))?;
    let theme = theme?;
    let mut scheme = Vec::new();
    theme.find_all(
        if which.starts_with("major") {
            "majorFont"
        } else {
            "minorFont"
        },
        &mut scheme,
    );
    scheme
        .first()?
        .child("latin")?
        .attr("typeface")
        .map(str::to_string)
}

fn apply(r: &mut Resolved, ppr: Option<&Element>, rpr: Option<&Element>, theme: Option<&Element>) {
    if let Some(rpr) = rpr {
        if r.font.is_none() {
            r.font = font_of(rpr, theme);
        }
        if r.size_half_points.is_none() {
            r.size_half_points = rpr.child("sz").and_then(|s| twips(s, "val"));
        }
        r.bold |= rpr.toggle("b").unwrap_or(false);
        r.italic |= rpr.toggle("i").unwrap_or(false);
    }
    if let Some(ppr) = ppr {
        if let Some(sp) = ppr.child("spacing") {
            if r.before.is_none() {
                r.before = twips(sp, "before");
            }
            if r.after.is_none() {
                r.after = twips(sp, "after");
            }
            if r.line.is_none()
                && let Some(line) = twips(sp, "line")
            {
                r.line = Some((line, sp.attr("lineRule").unwrap_or("auto").to_string()));
            }
        }
        r.keep_next |= ppr.child("keepNext").is_some();
        if r.align.is_none() {
            r.align = ppr
                .child("jc")
                .and_then(|j| j.attr("val"))
                .map(str::to_string);
        }
    }
}

fn resolve(styles: Option<&Element>, theme: Option<&Element>, id: &str) -> Resolved {
    let mut r = Resolved::default();
    let Some(styles) = styles else { return r };
    let mut current = Some(id.to_string());
    let mut hops = 0;
    while let Some(sid) = current.take() {
        hops += 1;
        if hops > 16 {
            break;
        }
        let Some(style) = style_by_id(styles, &sid) else {
            break;
        };
        apply(&mut r, style.child("pPr"), style.child("rPr"), theme);
        current = style
            .child("basedOn")
            .and_then(|b| b.attr("val"))
            .map(str::to_string);
    }
    if let Some(defaults) = styles.child("docDefaults") {
        apply(
            &mut r,
            defaults.path(&["pPrDefault", "pPr"]),
            defaults.path(&["rPrDefault", "rPr"]),
            theme,
        );
    }
    r
}

fn points(twips: f64) -> String {
    let v = format!("{:.1}", twips / 20.0);
    v.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn align_word(align: Option<&str>) -> &'static str {
    match align {
        Some("center") => "center",
        Some("right" | "end") => "right",
        Some("both" | "distribute") => "justify",
        _ => "left",
    }
}

/// The default style's line prints everything; a heading line always prints
/// its font and size, then only what differs from `base` (the default
/// style), so the legend reads as deviations.
fn describe(r: &Resolved, base: Option<&Resolved>) -> String {
    let differs = |pick: fn(&Resolved) -> String| base.is_none_or(|b| pick(b) != pick(r));
    let mut parts = Vec::new();
    let mut font = r
        .font
        .clone()
        .unwrap_or_else(|| "Times New Roman".to_string());
    if r.bold {
        font.push_str(" bold");
    }
    if r.italic {
        font.push_str(" italic");
    }
    let size = r.size_half_points.unwrap_or(20.0) / 2.0;
    let size = format!("{size:.1}");
    parts.push(format!(
        "{font} {}pt",
        size.trim_end_matches('0').trim_end_matches('.')
    ));
    if differs(|x| format!("{:?}", x.before))
        && let Some(b) = r.before.filter(|&b| b > 0.0)
    {
        parts.push(format!("before {}pt", points(b)));
    }
    if differs(|x| format!("{:?}", x.after))
        && let Some(a) = r.after.filter(|&a| a > 0.0)
    {
        parts.push(format!("after {}pt", points(a)));
    }
    if differs(|x| format!("{:?}", x.line))
        && let Some((line, rule)) = &r.line
    {
        if rule == "auto" {
            let ratio = format!("{:.2}", line / 240.0);
            let ratio = ratio.trim_end_matches('0').trim_end_matches('.');
            if ratio != "1" {
                parts.push(format!("line {ratio}"));
            }
        } else {
            parts.push(format!("line {}pt {rule}", points(*line)));
        }
    }
    if r.keep_next && differs(|x| x.keep_next.to_string()) {
        parts.push("keep-next".to_string());
    }
    match base {
        None => parts.push(align_word(r.align.as_deref()).to_string()),
        Some(b) if align_word(r.align.as_deref()) != align_word(b.align.as_deref()) => {
            parts.push(align_word(r.align.as_deref()).to_string());
        }
        Some(_) => {}
    }
    parts.join(", ")
}

fn table_style_line(styles: Option<&Element>, id: &str) -> String {
    let borders = styles
        .and_then(|s| style_by_id(s, id))
        .and_then(|s| s.path(&["tblPr", "tblBorders"]));
    let Some(borders) = borders else {
        return id.to_string();
    };
    let sides = ["top", "left", "bottom", "right", "insideH", "insideV"];
    let sizes: Vec<Option<(String, String)>> = sides
        .iter()
        .map(|side| {
            borders.child(side).map(|b| {
                (
                    b.attr("val").unwrap_or("").to_string(),
                    b.attr("sz").unwrap_or("").to_string(),
                )
            })
        })
        .collect();
    match sizes.first().cloned().flatten() {
        Some((val, sz))
            if val == "single"
                && sizes
                    .iter()
                    .all(|s| s.as_ref() == Some(&(val.clone(), sz.clone()))) =>
        {
            let pt = sz.parse::<f64>().map(|s| s / 8.0).unwrap_or(0.5);
            let pt = format!("{pt:.2}");
            format!(
                "{id}, all borders {}pt",
                pt.trim_end_matches('0').trim_end_matches('.')
            )
        }
        _ => format!("{id}, borders vary"),
    }
}

fn entry(s: &StoryFact) -> String {
    let mut inner = format!("id: {}, text: {}", s.id, scalar(&s.text, true));
    if let Some(align) = s.align {
        inner.push_str(&format!(", {align}"));
    }
    if !s.active {
        inner.push_str(", inactive");
    }
    format!("{{{inner}}}")
}

/// The `headers:` / `footers:` blocks of the first section.
fn first_section_stories(out: &mut String, section: &SectionFact) {
    for kind in ["header", "footer"] {
        let stories: Vec<&StoryFact> = section.stories.iter().filter(|s| s.kind == kind).collect();
        if stories.is_empty() && !section.title_page {
            continue;
        }
        out.push_str(&format!("{kind}s:\n"));
        let find = |ty: &str| stories.iter().find(|s| s.ty == ty);
        if section.title_page {
            match find("first") {
                Some(s) => kv(
                    out,
                    &format!("  first: {}", entry(s)),
                    Some("page 1 only (different first page)"),
                ),
                None => kv(
                    out,
                    "  first: none",
                    Some(if kind == "footer" {
                        "page 1 shows no page number"
                    } else {
                        "page 1 shows no header"
                    }),
                ),
            }
        }
        match find("default") {
            Some(s) => kv(out, &format!("  default: {}", entry(s)), None),
            None => kv(out, "  default: none", None),
        }
        if let Some(s) = find("even") {
            kv(
                out,
                &format!("  even: {}", entry(s)),
                if s.active {
                    None
                } else {
                    Some("defined, but even/odd headers are off")
                },
            );
        }
    }
}

/// `{p2-p3, headers: {default: {…}}, columns: 2}`: what a later section
/// changes against the one before it (page setup against the first).
fn section_entry(section: &SectionFact, previous: &SectionFact, first: &SectionFact) -> String {
    let mut parts = vec![format!("p{}-p{}", section.first, section.last)];
    let setup = |s: &SectionFact| s.sect_pr.as_ref().map(page_setup);
    if setup(section) != setup(first)
        && let Some(page) = setup(section)
    {
        parts.push(format!("page: {page}"));
    }
    for kind in ["header", "footer"] {
        let changed: Vec<String> = section
            .stories
            .iter()
            .filter(|s| s.kind == kind)
            .filter(|s| {
                !previous
                    .stories
                    .iter()
                    .any(|p| p.kind == s.kind && p.ty == s.ty && p.id == s.id)
            })
            .map(|s| format!("{}: {}", s.ty, entry(s)))
            .collect();
        if !changed.is_empty() {
            parts.push(format!("{kind}s: {{{}}}", changed.join(", ")));
        }
    }
    if section.columns != previous.columns {
        parts.push(format!("columns: {}", section.columns));
    }
    format!("{{{}}}", parts.join(", "))
}

fn part_two(out: &mut String, f: &Facts) {
    if let Some(sect) = f.sections.first().and_then(|s| s.sect_pr.as_ref()) {
        kv(out, &format!("page: {}", page_setup(sect)), None);
    }
    out.push_str("styles:\n");
    let default = f.default_style.unwrap_or("Normal");
    let base = resolve(f.styles, f.theme, default);
    kv(
        out,
        &format!(
            "  {}: {}",
            scalar(default, false),
            scalar(&describe(&base, None), false)
        ),
        Some("default; unannotated paragraphs use it"),
    );
    for (level, style) in &f.heading_styles {
        kv(
            out,
            &format!(
                "  \"{}\": {}",
                "#".repeat(*level),
                scalar(
                    &format!(
                        "{style}, {}",
                        describe(&resolve(f.styles, f.theme, style), Some(&base))
                    ),
                    false
                )
            ),
            None,
        );
    }
    match f.table_styles.as_slice() {
        [] => {}
        [one] => kv(
            out,
            &format!(
                "  table: {}",
                scalar(&table_style_line(f.styles, one), false)
            ),
            None,
        ),
        many => kv(
            out,
            &format!("  tables: {}", scalar(&many.join(", "), false)),
            None,
        ),
    }
    if let Some(first) = f.sections.first() {
        first_section_stories(out, first);
        if f.sections.len() > 1 {
            out.push_str("sections:\n");
            for (i, section) in f.sections.iter().enumerate().skip(1) {
                kv(
                    out,
                    &format!(
                        "  {}: {}",
                        i + 1,
                        section_entry(section, &f.sections[i - 1], first)
                    ),
                    None,
                );
            }
        }
    }
}

/// A header/footer part's paragraphs in document order, numbered as `edit`
/// numbers a story (`inspect::story_paragraph_nodes`): every `w:p` outside
/// text boxes, table cells and content controls included.
pub(crate) fn story_paragraphs(root: &Element) -> Vec<&Element> {
    super::agent::paragraphs(root)
}

/// A header/footer paragraph's text with fields as `{PAGE}`: field codes
/// print, cached results do not. Runs inside hyperlinks, content controls,
/// smart tags, custom XML and insertions count; deleted runs do not.
pub(crate) fn story_text(p: &Element) -> String {
    #[derive(Default)]
    struct State {
        out: String,
        instr: Option<String>,
        in_result: bool,
    }
    fn code(instr: &str) -> String {
        let name = instr
            .split_whitespace()
            .next()
            .unwrap_or("FIELD")
            .to_uppercase();
        format!("{{{name}}}")
    }
    fn walk(parent: &Element, s: &mut State) {
        for run in parent.elements() {
            match run.local() {
                "fldSimple" => s.out.push_str(&code(run.attr("instr").unwrap_or(""))),
                "hyperlink" | "smartTag" | "customXml" | "ins" => walk(run, s),
                "sdt" => {
                    if let Some(content) = run.child("sdtContent") {
                        walk(content, s);
                    }
                }
                "r" => {
                    for child in run.elements() {
                        match child.local() {
                            "fldChar" => match child.attr("fldCharType") {
                                Some("begin") => s.instr = Some(String::new()),
                                Some("separate") => s.in_result = true,
                                Some("end") => {
                                    if let Some(instr) = s.instr.take() {
                                        s.out.push_str(&code(&instr));
                                    }
                                    s.in_result = false;
                                }
                                _ => {}
                            },
                            "instrText" => {
                                if let Some(i) = s.instr.as_mut() {
                                    i.push_str(&child.text());
                                }
                            }
                            "t" if s.instr.is_none() && !s.in_result => {
                                s.out.push_str(&child.text());
                            }
                            "tab" => s.out.push('\t'),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let mut s = State::default();
    walk(p, &mut s);
    s.out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::from_docx::ooxml::parse_xml;

    const W: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main""#;

    fn ftr(inner: &str) -> Element {
        parse_xml(format!("<w:ftr {W}>{inner}</w:ftr>").as_bytes()).unwrap()
    }

    const PAGE_RUNS: &str = r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#;

    /// Review t09-15: header and footer ids must number paragraphs exactly as
    /// `edit` resolves them, so the two walks are checked against each other.
    #[test]
    fn story_paragraphs_agree_with_the_paragraphs_edit_resolves() {
        let xml = format!(
            r#"<w:ftr {W}><w:p/><w:tbl><w:tr><w:tc><w:p/><w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl></w:tc></w:tr></w:tbl><w:sdt><w:sdtContent><w:p/></w:sdtContent></w:sdt><w:p><w:r><w:pict><v:shape xmlns:v="urn:schemas-microsoft-com:vml"><v:textbox><w:txbxContent><w:p/></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p><w:customXml><w:p/></w:customXml></w:ftr>"#
        );
        let ours = story_paragraphs(&parse_xml(xml.as_bytes()).unwrap()).len();
        let mut dom = crate::xmllinq::Dom::new();
        let document = dom.parse_xdocument(&xml);
        let root = dom.root(document).unwrap();
        let edits = crate::inspect::story_paragraph_nodes(&dom, root).len();
        assert_eq!(ours, edits);
        assert_eq!(ours, 6);
    }

    #[test]
    fn a_long_value_aligns_by_characters_not_bytes() {
        let mut ascii = String::new();
        // 32 characters (35 bytes): the padded branch.
        kv(
            &mut ascii,
            "  document_owner: Jorgen Osterbo",
            Some("dc:creator"),
        );
        let mut accented = String::new();
        kv(
            &mut accented,
            "  document_owner: Jørgen Østerbø",
            Some("dc:creator"),
        );
        assert_eq!(
            ascii.find('#'),
            accented.chars().position(|c| c == '#'),
            "{ascii}{accented}"
        );
    }

    #[test]
    fn an_empty_comment_leaves_no_hash() {
        let mut out = String::new();
        kv(&mut out, "  AC: Ann Counsel", Some(""));
        assert_eq!(out, "  AC: Ann Counsel\n");
    }

    #[test]
    fn a_page_field_in_a_block_content_control_is_found() {
        let root = ftr(&format!(
            "<w:sdt><w:sdtPr/><w:sdtContent><w:p>{PAGE_RUNS}</w:p></w:sdtContent></w:sdt><w:p/>"
        ));
        let paragraphs = story_paragraphs(&root);
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(story_text(paragraphs[0]), "{PAGE}");
        assert_eq!(story_text(paragraphs[1]), "");
    }

    #[test]
    fn runs_inside_inline_wrappers_count_and_deletions_do_not() {
        let root = ftr(&format!(
            r#"<w:p><w:hyperlink><w:r><w:t>Page </w:t></w:r></w:hyperlink><w:sdt><w:sdtContent>{PAGE_RUNS}</w:sdtContent></w:sdt><w:ins><w:r><w:t> of </w:t></w:r></w:ins><w:del><w:r><w:delText>gone</w:delText></w:r></w:del><w:fldSimple w:instr=" numpages "><w:r><w:t>3</w:t></w:r></w:fldSimple></w:p>"#
        ));
        assert_eq!(
            story_text(story_paragraphs(&root)[0]),
            "Page {PAGE} of {NUMPAGES}"
        );
    }

    #[test]
    fn paragraphs_number_as_edit_does_tables_in_and_text_boxes_out() {
        // edit numbers a story's every w:p outside text boxes, table cells
        // included, so `footer1.p1` here is edit's `footer1:p:1`.
        let root = ftr(
            r#"<w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:p><w:r><w:pict><w:txbxContent><w:p><w:r><w:t>boxed</w:t></w:r></w:p></w:txbxContent></w:pict></w:r><w:r><w:t>body</w:t></w:r></w:p>"#,
        );
        let paragraphs = story_paragraphs(&root);
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(story_text(paragraphs[0]), "");
        assert_eq!(story_text(paragraphs[1]), "body");
    }
}
