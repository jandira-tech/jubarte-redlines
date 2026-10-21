// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The YAML header of the agent view.

use std::collections::{HashMap, HashSet};

use super::agent::{Handles, RevTag};
use super::ooxml::Element;
use crate::markdown::view_notes::WORD_PAGE_SIZES;

/// Where the owner's name came from.
pub(crate) enum Owner {
    Creator(String),
    LastModifiedBy(String),
    /// `docProps/core.xml` is there but names nobody.
    Unnamed,
    /// No `docProps/core.xml`.
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
    /// The part's own id, its first paragraph's (`header2`): a story printed
    /// paragraph by paragraph names it, then `pN:` for each.
    pub part: String,
    /// The paragraph `text` is, in the part.
    pub first: usize,
    pub text: String,
    /// The paragraphs after it, to the part's last (an empty one too, a
    /// line Word lays out): `text` then prints as a list (`text: [Page 1,
    /// Draft]`, `text: [DRAFT, ""]`).
    pub more: Vec<String>,
    /// `center`, `right`, or `None` for left.
    pub align: Option<&'static str>,
    /// `text` and `more`, one by one: what each paragraph carries itself.
    pub paras: Vec<ParaFact>,
    /// The paragraphs' styles when not all are the story's own
    /// ([`story_style`]): one for all (`style: Normal`), or one per
    /// paragraph from the part's first (`styles: [Header, ListParagraph]`).
    pub styles: Vec<String>,
    pub active: bool,
    /// Tracked view: the part's insertions, deletions and moves `text` prints
    /// no tag for (`header1.r3 @AC`), then its formatting changes.
    pub rev: Vec<String>,
    pub fmt: Vec<String>,
}

/// One paragraph of a header or footer: its own alignment, text colour and
/// borders.
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct ParaFact {
    /// `center`, `right`, or `None` for left.
    pub align: Option<&'static str>,
    /// The colour its text runs set themselves (`w:color`), when they all
    /// set the same one.
    pub color: Option<String>,
    /// Its own borders (`w:pBdr`), as a border prints them (`bottom single
    /// 0.5pt 8496B0 space 4pt`).
    pub border: Option<String>,
}

/// A section of the body (Task 11 fills these).
pub(crate) struct SectionFact {
    /// Paragraph range, inclusive.
    pub first: usize,
    pub last: usize,
    pub sect_pr: Option<Element>,
    pub stories: Vec<StoryFact>,
    pub title_page: bool,
}

pub(crate) struct CommentFact {
    pub id: String,
    pub author: Option<String>,
    pub parent: Option<String>,
    /// A range or reference to it sits in the body or a note. Without one
    /// Word draws no balloon, so it is no open thread.
    pub anchored: bool,
    /// Its first paragraph's style: `CommentText` for Word's own
    /// (`annotation text`), the document default's id for none.
    pub style: String,
}

pub(crate) struct Facts<'a> {
    pub source: &'a str,
    /// The SHA-256 of the bytes the view reads, in hex: the file, or the
    /// in-memory redline `jubarte A B` prints.
    pub sha256: String,
    /// `None` tracked, `Some(true)` accept-all, `Some(false)` reject-all.
    pub resolved: Option<bool>,
    pub comments_inline: bool,
    pub tracking_on: bool,
    pub tags: Vec<RevTag>,
    /// The document's changes, as `jubarte changes` lists them: the one
    /// revision count.
    pub changes: Vec<crate::changes::Change>,
    pub comments: Vec<CommentFact>,
    pub handles: &'a Handles,
    /// Resolved comment ids (`w15:done`), from `agent::Threads`.
    pub done: &'a HashSet<String>,
    pub owner: Owner,
    /// `dc:title`, when the core properties give one.
    pub title: Option<String>,
    pub paragraphs: usize,
    pub tables: usize,
    pub pages: usize,
    pub pages_source: PagesSource,
    /// `range:` line after `body:` when a selection is active (Task 12).
    pub range: Option<String>,
    /// A header or footer holds a change `range`'s `--changed` selects.
    pub stories_changed: bool,
    pub styles: Option<&'a Element>,
    pub theme: Option<&'a Element>,
    pub default_style: Option<&'a str>,
    /// (level, style id) for every heading level used, lowest first.
    pub heading_styles: Vec<(usize, String)>,
    /// Every other paragraph style the body uses, in first-use order.
    pub other_styles: Vec<String>,
    pub table_styles: Vec<String>,
    /// The tables differ in style: their lines name theirs, and the
    /// legend lists them all as `tables:`.
    pub tables_differ: bool,
    /// The body's sections in order; the first one is described in full.
    pub sections: Vec<SectionFact>,
    /// Tracked view: the changes (`rev`, `fmt`) of the notes no definition
    /// line shows, such as Word's footnote separators.
    pub note_changes: (Vec<String>, Vec<String>),
    /// The notes parts (`footnotes`, `endnotes`) the view shows no note
    /// of, such as a part holding only Word's separators.
    pub empty_notes: Vec<&'static str>,
    /// Tracked view of the whole document: the listed changes no other line
    /// prints (`r3 @AC in p0`), such as a text box's deleted picture.
    pub unplaced: Vec<String>,
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

/// `value` as a YAML scalar: plain when every YAML reader, 1.1 or 1.2, takes
/// it back as the same string, else double-quoted (a JSON string is a valid YAML one).
/// `flow` values sit inside `{…}` or `[…]`, where `,`, `[`, `]`, `{` and `}`
/// end them, and where libyaml refuses any `:` and a `?` (`@??`).
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
        && !(flow && value.contains([',', '[', ']', '{', '}', ':', '?']))
        && !RESERVED.contains(&value.to_ascii_lowercase().as_str())
        // One word that opens with a digit, `+` or `.` may be a number, a
        // time (`12:30`, base 60 in YAML 1.1) or a date to some reader.
        && (!value.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '.')
            || value.contains(char::is_whitespace));
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

/// The `revisions:` line's comment: the body's marks and the changes the
/// view shows them as, then formatting (body and styles) and each other
/// story's count: `9 marks shown as 5 changes: 1 insertion, 4
/// substitutions; 2 formatting; 1 in header1`. Empty for no change.
fn revisions_comment(f: &Facts) -> String {
    use crate::changes::ChangeKind;
    let story = |c: &crate::changes::Change| {
        crate::ids::change_id(&c.id).map_or_else(|| "body".to_string(), |id| id.story)
    };
    let marks = f
        .changes
        .iter()
        .filter(|c| story(c) == "body" && c.kind != ChangeKind::Formatting)
        .count();
    let formatting = f
        .changes
        .iter()
        .filter(|c| {
            c.kind == ChangeKind::Formatting && matches!(story(c).as_str(), "body" | "styles")
        })
        .count();
    // A part's group prints where its block does — headers then footers,
    // first, default, even, section by section — not where its relationship
    // happens to sit, which a writer cannot keep.
    let mut counts: HashMap<String, usize> = HashMap::new();
    for c in &f.changes {
        let s = story(c);
        if matches!(s.as_str(), "body" | "styles") {
            continue;
        }
        *counts.entry(s).or_default() += 1;
    }
    let mut order: Vec<String> = Vec::new();
    for section in &f.sections {
        for kind in ["header", "footer"] {
            for ty in ["first", "default", "even"] {
                if let Some(s) = section
                    .stories
                    .iter()
                    .find(|s| s.kind == kind && s.ty == ty && counts.contains_key(&s.part))
                    && !order.contains(&s.part)
                {
                    order.push(s.part.clone());
                }
            }
        }
    }
    for c in &f.changes {
        let s = story(c);
        if counts.contains_key(&s) && !order.contains(&s) {
            order.push(s);
        }
    }
    let others: Vec<(String, usize)> = order
        .into_iter()
        .filter_map(|s| counts.get(&s).map(|n| (s, *n)))
        .collect();
    let mut parts = Vec::new();
    if marks > 0 {
        let mut kinds = Vec::new();
        for (kind, one, many) in [
            ("ins", "insertion", "insertions"),
            ("del", "deletion", "deletions"),
            ("sub", "substitution", "substitutions"),
            ("mark", "paragraph mark", "paragraph marks"),
            ("row", "table row", "table rows"),
            ("cell", "table cell", "table cells"),
        ] {
            let n = f.tags.iter().filter(|t| t.kind == kind).count();
            if n > 0 {
                kinds.push(plural(n, one, many));
            }
        }
        let marks = plural(marks, "mark", "marks");
        parts.push(if f.tags.is_empty() {
            marks
        } else {
            format!(
                "{marks} shown as {}: {}",
                plural(f.tags.len(), "change", "changes"),
                join(&kinds)
            )
        });
    }
    if formatting > 0 {
        parts.push(format!("{formatting} formatting"));
    }
    for (name, n) in others {
        parts.push(format!("{n} in {name}"));
    }
    parts.join("; ")
}

pub(crate) fn render(f: &Facts) -> String {
    let mut out = String::from("---\n");
    kv(
        &mut out,
        &format!("source: {}", scalar(f.source, false)),
        None,
    );
    kv(&mut out, &format!("sha256: {}", f.sha256), None);
    if let Some(title) = &f.title {
        kv(&mut out, &format!("title: {}", scalar(title, false)), None);
    }
    let handle_list: Vec<String> = f
        .handles
        .order
        .iter()
        .filter(|a| {
            f.changes
                .iter()
                .any(|c| c.author.as_deref() == Some(a.as_str()))
        })
        .filter_map(|a| f.handles.by_author.get(a).cloned())
        .collect();
    let by = if handle_list.is_empty() {
        String::new()
    } else {
        format!(" by {}", join(&handle_list))
    };
    let threads: Vec<&CommentFact> = f
        .comments
        .iter()
        .filter(|c| c.parent.is_none() && c.anchored)
        .collect();
    let unanchored: Vec<&CommentFact> = f
        .comments
        .iter()
        .filter(|c| c.parent.is_none() && !c.anchored)
        .collect();
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
                plural(f.changes.len(), "revision", "revisions"),
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
    kv(
        &mut out,
        &format!("revisions: {}", f.changes.len()),
        Some(&revisions_comment(f)),
    );
    if f.comments.is_empty() {
        kv(&mut out, "comments: 0", None);
    } else {
        let open = threads.iter().filter(|c| !f.done.contains(&c.id)).count();
        let mut key = format!("comments: {}", plural(open, "thread open", "threads open"));
        if open < threads.len() {
            key.push_str(&format!(", {} resolved", threads.len() - open));
        }
        if !unanchored.is_empty() {
            key.push_str(&format!(", {} unanchored", unanchored.len()));
        }
        // Comment paragraphs in another style than Word's: the most
        // comments' style for all (`style Normal`), and each other one's
        // (`c6 style Normal`).
        let mut counts: Vec<(&str, usize)> = Vec::new();
        for comment in &f.comments {
            match counts.iter_mut().find(|(s, _)| *s == comment.style) {
                Some((_, n)) => *n += 1,
                None => counts.push((&comment.style, 1)),
            }
        }
        let most = counts
            .iter()
            .fold(None::<(&str, usize)>, |best, &(s, n)| match best {
                Some((_, m)) if m >= n => best,
                _ => Some((s, n)),
            })
            .map_or("CommentText", |(s, _)| s);
        if most != "CommentText" {
            key.push_str(&format!(
                ", style {}",
                crate::markdown::view_notes::style_token(most)
            ));
        }
        for comment in f.comments.iter().filter(|c| c.style != most) {
            key.push_str(&format!(
                ", c{} style {}",
                comment.id,
                crate::markdown::view_notes::style_token(&comment.style)
            ));
        }
        let mut list: Vec<String> = threads
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
        list.extend(unanchored.iter().map(|c| {
            format!(
                "c{} (unanchored: not in the body or a note, Word shows no balloon)",
                c.id
            )
        }));
        // A quoted style id may hold `: ` or ` #`: the value is quoted then.
        let key = match key.split_once(": ") {
            Some((name, value)) => format!("{name}: {}", scalar(value, false)),
            None => key,
        };
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
        Owner::Unnamed => kv(
            &mut out,
            "  document_owner: none",
            Some("docProps/core.xml names no dc:creator or cp:lastModifiedBy"),
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
            .changes
            .iter()
            .filter(|c| c.author.as_deref() == Some(author.as_str()))
            .count();
        let comments = f
            .comments
            .iter()
            .filter(|c| c.author.as_deref() == Some(author.as_str()))
            .count();
        let mut parts = Vec::new();
        if revisions > 0 {
            parts.push(plural(revisions, "revision", "revisions"));
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
        // A header or footer change shows on the headers:/footers: lines,
        // not as a body block.
        let note = f.stories_changed.then(|| {
            if range.ends_with("(none)") {
                "no body block; header and footer text is on the headers:/footers: lines"
            } else {
                "a header or footer change too: its text is on the headers:/footers: lines"
            }
        });
        kv(&mut out, &format!("range: {range}{of}"), note);
    }
    part_two(&mut out, f);
    out.push_str("---\n");
    out
}

/// A twips-valued attribute; Strict files write the universal measure
/// (`72pt`, `1in`) where Transitional writes the integer.
fn twips(e: &Element, attr: &str) -> Option<f64> {
    e.attr(attr).and_then(|v| {
        v.parse::<f64>()
            .ok()
            .or_else(|| crate::util::universal_measure_twips(v))
    })
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
    // Named sizes match on the rounded twip: 210mm is the 11906 an A4
    // spells, not the 11905 a truncating cast reads (#531). The names are
    // Word's own paper list, and the writer writes the same twips back
    // for each (#561); a twip off a named size is not that size.
    let name = match WORD_PAGE_SIZES.iter().find(|&&(_, w, h)| {
        (short.round() as i64, long.round() as i64) == (i64::from(w), i64::from(h))
    }) {
        // Word's paper list names the Tabloid sheet Ledger lying
        // landscape.
        Some(&(name, _, _)) if landscape && name == "Tabloid" => "Ledger".to_string(),
        Some(&(name, _, _)) => name.to_string(),
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
    // Text columns, and their gap when it is not Word's 0.5in.
    if let Some(cols) = sect.child("cols")
        && let Some(num) = cols
            .attr("num")
            .and_then(|n| n.parse::<u32>().ok())
            .filter(|&n| n > 1)
    {
        out.push_str(&format!(", columns {num}"));
        if let Some(gap) = twips(cols, "space").filter(|&g| g != 720.0) {
            out.push_str(&format!(", column gap {}", super::agent::inches(gap)));
        }
        if cols
            .attr("sep")
            .is_some_and(|v| matches!(v, "1" | "true" | "on"))
        {
            out.push_str(", column line");
        }
    }
    // The grid lines snap to; a `w:docGrid` with no type is none.
    if let Some(grid) = sect.child("docGrid")
        && let Some(kind) = match grid.attr("type") {
            Some("lines") => Some("line grid"),
            Some("linesAndChars") => Some("line and char grid"),
            Some("snapToChars") => Some("char grid"),
            _ => None,
        }
        // A character grid needs no line pitch (`snapToChars` sets only
        // the character spacing); the line grids do.
        && let Some(pitch) = match twips(grid, "linePitch").filter(|&p| p > 0.0) {
            Some(pitch) => Some(Some(pitch)),
            None => (kind == "char grid").then_some(None),
        }
    {
        match pitch {
            Some(pitch) => {
                let points = format!("{:.2}", pitch / 20.0);
                let points = points.trim_end_matches('0').trim_end_matches('.');
                out.push_str(&format!(", {kind} {points}pt"));
            }
            None => out.push_str(&format!(", {kind}")),
        }
        if kind != "line grid"
            && let Some(space) = grid
                .attr("charSpace")
                .and_then(|c| c.parse::<i64>().ok())
                .filter(|&c| c != 0)
        {
            out.push_str(&format!(", char space {space}"));
        }
    }
    // How the section starts, when its `w:sectPr` states it: `nextPage`
    // is Word's default, but a document that says so keeps saying it
    // (#528). A value outside Word's five says nothing the view can write.
    if let Some(kind) = sect.child("type").and_then(|t| t.attr("val")).filter(|v| {
        matches!(
            *v,
            "nextPage" | "continuous" | "evenPage" | "oddPage" | "nextColumn"
        )
    }) {
        out.push_str(&format!(", starts {kind}"));
    }
    out
}

/// Resolved paragraph/run properties of a style through its `basedOn`
/// chain and the document defaults.
#[derive(Default)]
struct Resolved {
    character: bool,
    font: Option<String>,
    size_half_points: Option<f64>,
    bold: bool,
    italic: bool,
    before: Option<f64>,
    after: Option<f64>,
    line: Option<(f64, String)>,
    keep_next: bool,
    keep_lines: bool,
    contextual_spacing: bool,
    /// Twips.
    left: Option<f64>,
    first_line: Option<f64>,
    hanging: Option<f64>,
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
            // `w:sz` is half-points (`ST_HpsMeasure`), not twips: a strict
            // `"11pt"` is 22, as the plain `"22"` is.
            r.size_half_points = rpr.child("sz").and_then(|s| {
                s.attr("val").and_then(|v| {
                    v.parse::<f64>()
                        .ok()
                        .or_else(|| crate::util::universal_measure_half_points(v))
                })
            });
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
        r.keep_lines |= ppr.child("keepLines").is_some();
        r.contextual_spacing |= ppr.child("contextualSpacing").is_some();
        if let Some(ind) = ppr.child("ind") {
            if r.left.is_none() {
                r.left = twips(ind, "left").or_else(|| twips(ind, "start"));
            }
            if r.first_line.is_none() {
                r.first_line = twips(ind, "firstLine");
            }
            if r.hanging.is_none() {
                r.hanging = twips(ind, "hanging");
            }
        }
        if r.align.is_none() {
            r.align = ppr
                .child("jc")
                .and_then(|j| j.attr("val"))
                .map(str::to_string);
        }
    }
}

fn resolve(styles: Option<&Element>, theme: Option<&Element>, id: &str) -> Resolved {
    let Some(styles) = styles else {
        return Resolved::default();
    };
    let mut r = Resolved {
        character: style_by_id(styles, id).is_some_and(|s| s.attr("type") == Some("character")),
        ..Resolved::default()
    };
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
/// style), so the legend reads as deviations. Indent, keep-lines and
/// contextual spacing follow keep-next. A zero indent is named only when
/// it differs from the base.
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
    // Character styles carry run properties only. Paragraph defaults
    // (including the writer template's spacing) do not describe them.
    if r.character {
        return format!("character, {}", parts.join(", "));
    }
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
    if r.keep_lines && differs(|x| x.keep_lines.to_string()) {
        parts.push("keep-lines".to_string());
    }
    // Left, then first-line, then hanging: the order the writer reads back.
    if differs(|x| format!("{:?}", x.left))
        && let Some(left) = r.left
        && (base.is_some() || left != 0.0)
    {
        parts.push(format!("indent {}", super::agent::inches(left)));
    }
    if differs(|x| format!("{:?}", x.first_line))
        && let Some(first) = r.first_line
        && (base.is_some() || first != 0.0)
    {
        parts.push(format!("first-line {}", super::agent::inches(first)));
    }
    if differs(|x| format!("{:?}", x.hanging))
        && let Some(hanging) = r.hanging
        && (base.is_some() || hanging != 0.0)
    {
        parts.push(format!("hanging {}", super::agent::inches(hanging)));
    }
    if r.contextual_spacing && differs(|x| x.contextual_spacing.to_string()) {
        parts.push("contextual spacing".to_string());
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

/// `, named Text Heading 2`: the name of style `id` when it is not the one
/// the writer would give it (`expected`).
fn named(styles: Option<&Element>, id: &str, expected: &str) -> String {
    styles
        .and_then(|s| style_by_id(s, id))
        .and_then(|s| s.child("name"))
        .and_then(|n| n.attr("val"))
        .filter(|name| *name != expected)
        .map(|name| format!(", named {name}"))
        .unwrap_or_default()
}

/// A legend `named` value: quoted (a JSON string) when it holds a comma
/// or a quote, so the clause's `, ` separator cannot split it (#549).
fn quoted_name(name: &str) -> String {
    if name.contains(',') || name.contains('"') {
        serde_json::to_string(name).unwrap_or_else(|_| format!("\"{name}\""))
    } else {
        name.to_string()
    }
}

fn table_style_line(styles: Option<&Element>, id: &str) -> String {
    // The style's name, when it is not the one the writer would give the
    // id (`9` is named `Normal Table`, `TableGrid0` `TableGrid`, #528).
    let named = styles
        .and_then(|s| style_by_id(s, id))
        .and_then(|s| s.child("name"))
        .and_then(|n| n.attr("val"))
        .filter(|name| {
            *name
                != super::super::package::word_table_style_name(id)
                    .unwrap_or_else(|| id.to_string())
        })
        .map(|name| format!(", named {}", quoted_name(name)))
        .unwrap_or_default();
    let borders = styles
        .and_then(|s| style_by_id(s, id))
        .and_then(|s| s.path(&["tblPr", "tblBorders"]));
    let Some(borders) = borders else {
        return format!("{id}{named}");
    };
    // The strict `start`/`end` elements name the `left`/`right` borders.
    let sides = [
        &["top"][..],
        &["left", "start"][..],
        &["bottom"][..],
        &["right", "end"][..],
        &["insideH"][..],
        &["insideV"][..],
    ];
    let sizes: Vec<Option<(String, String)>> = sides
        .iter()
        .map(|names| {
            names.iter().find_map(|side| borders.child(side)).map(|b| {
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
                "{id}, all borders {}pt{named}",
                pt.trim_end_matches('0').trim_end_matches('.')
            )
        }
        _ => format!("{id}, borders vary{named}"),
    }
}

/// The style the writer gives a `header` or `footer` paragraph the view
/// names no style for: Word's own.
pub(crate) fn story_style(kind: &str) -> &'static str {
    if kind == "header" { "Header" } else { "Footer" }
}

/// A story prints paragraph by paragraph (`p0: {text: …, color: …}`) when
/// one carries its own colour or borders, or their alignments differ.
fn by_paragraph(s: &StoryFact) -> bool {
    s.paras
        .iter()
        .any(|p| p.color.is_some() || p.border.is_some() || p.align != s.align)
}

/// `p0: {text: …, center, color: "595959", border: bottom single 0.5pt}`,
/// one per paragraph.
fn paragraph_entries(s: &StoryFact) -> Vec<String> {
    std::iter::once(&s.text)
        .chain(&s.more)
        .zip(&s.paras)
        .enumerate()
        .map(|(k, (text, p))| {
            let mut inner = format!("text: {}", scalar(text, true));
            if let Some(align) = p.align {
                inner.push_str(&format!(", {align}"));
            }
            if let Some(color) = &p.color {
                inner.push_str(&format!(", color: {}", scalar(color, true)));
            }
            if let Some(border) = &p.border {
                inner.push_str(&format!(", border: {}", scalar(border, true)));
            }
            format!("p{}: {{{inner}}}", s.first + k)
        })
        .collect()
}

/// A story's entries after its text: its styles, `inactive`, its changes.
fn story_tail(s: &StoryFact) -> Vec<String> {
    let mut tail = Vec::new();
    match s.styles.as_slice() {
        [] => {}
        [style] => tail.push(format!("style: {}", scalar(style, true))),
        styles => {
            let all: Vec<String> = styles.iter().map(|s| scalar(s, true)).collect();
            tail.push(format!("styles: [{}]", all.join(", ")));
        }
    }
    if !s.active {
        tail.push("inactive".to_string());
    }
    for (key, tags) in [("rev", &s.rev), ("fmt", &s.fmt)] {
        if !tags.is_empty() {
            tail.push(format!("{key}: {}", scalar(&tags.join("; "), true)));
        }
    }
    tail
}

/// A story's entry in the first section's block: one line, or, paragraph
/// by paragraph, `key:` then its entries indented below.
fn story_kv(out: &mut String, key: &str, s: &StoryFact, comment: Option<&str>) {
    if !by_paragraph(s) {
        kv(out, &format!("{key}: {}", entry(s)), comment);
        return;
    }
    kv(out, &format!("{key}:"), comment);
    out.push_str(&format!("    id: {}\n", scalar(&s.part, true)));
    for line in paragraph_entries(s).into_iter().chain(story_tail(s)) {
        // A block mapping's flag needs a value.
        let line = if line == "inactive" {
            "inactive: true".to_string()
        } else {
            line
        };
        out.push_str(&format!("    {line}\n"));
    }
}

fn entry(s: &StoryFact) -> String {
    if by_paragraph(s) {
        let items: Vec<String> = std::iter::once(format!("id: {}", scalar(&s.part, true)))
            .chain(paragraph_entries(s))
            .chain(story_tail(s))
            .collect();
        return format!("{{{}}}", items.join(", "));
    }
    let text = if s.more.is_empty() {
        scalar(&s.text, true)
    } else {
        let all: Vec<String> = std::iter::once(&s.text)
            .chain(&s.more)
            .map(|t| scalar(t, true))
            .collect();
        format!("[{}]", all.join(", "))
    };
    let mut inner = format!("id: {}, text: {text}", scalar(&s.id, true));
    if let Some(align) = s.align {
        inner.push_str(&format!(", {align}"));
    }
    for item in story_tail(s) {
        inner.push_str(&format!(", {item}"));
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
                Some(s) => story_kv(
                    out,
                    "  first",
                    s,
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
        if !section.title_page
            && let Some(s) = find("first")
        {
            story_kv(
                out,
                "  first",
                s,
                Some("defined, but different first page is off"),
            );
        }
        match find("default") {
            Some(s) => story_kv(out, "  default", s, None),
            None => kv(out, "  default: none", None),
        }
        if let Some(s) = find("even") {
            story_kv(
                out,
                "  even",
                s,
                if s.active {
                    None
                } else {
                    Some("defined, but even/odd headers are off")
                },
            );
        }
    }
}

/// `{p2-p3, page: …, columns 2, headers: {default: {…}}}`: what a later section
/// changes against the one before it (page setup against the first).
fn section_entry(
    section: &SectionFact,
    previous: &SectionFact,
    first: &SectionFact,
    handles: Option<&Handles>,
) -> String {
    let mut parts = vec![format!("p{}-p{}", section.first, section.last)];
    let setup = |s: &SectionFact| s.sect_pr.as_ref().map(page_setup);
    if setup(section) != setup(first)
        && let Some(page) = setup(section)
    {
        parts.push(format!("page: {page}"));
    }
    if let Some(tags) = handles.and_then(|h| section_changes(section, h)) {
        parts.push(format!("fmt: {}", scalar(&tags, true)));
    }
    for kind in ["header", "footer"] {
        // w:titlePg is the section's own: a section that has it names its
        // first-page story, or `first: none`, even when inherited.
        let mut changed: Vec<String> = section
            .stories
            .iter()
            .filter(|s| s.kind == kind)
            .filter(|s| {
                (s.ty == "first" && section.title_page)
                    || !previous
                        .stories
                        .iter()
                        .any(|p| p.kind == s.kind && p.ty == s.ty && p.id == s.id)
            })
            .map(|s| format!("{}: {}", s.ty, entry(s)))
            .collect();
        if section.title_page
            && !section
                .stories
                .iter()
                .any(|s| s.kind == kind && s.ty == "first")
        {
            changed.insert(0, "first: none".to_string());
        }
        if !changed.is_empty() {
            parts.push(format!("{kind}s: {{{}}}", changed.join(", ")));
        }
    }
    format!("{{{}}}", parts.join(", "))
}

/// The tracked view's tags for a section's own changes (`w:sectPrChange`):
/// `r9 @AC`.
fn section_changes(section: &SectionFact, handles: &Handles) -> Option<String> {
    let sect = section.sect_pr.as_ref()?;
    let tags: Vec<String> = sect
        .children_named("sectPrChange")
        .map(|change| super::agent::tag_of(change, handles))
        .collect();
    (!tags.is_empty()).then(|| super::agent::format_tags(&tags))
}

/// `Normal: styles.r0 @AS, Heading1: styles.r3 @AS`: the styles part's
/// tracked changes by the style that holds them (`docDefaults` for the
/// document defaults), in the tracked view.
fn style_changes(styles: &Element, handles: &Handles) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for holder in styles.elements() {
        let name = match holder.local() {
            "docDefaults" => "docDefaults".to_string(),
            "style" => match holder.attr("styleId") {
                Some(id) => id.to_string(),
                None => continue,
            },
            _ => continue,
        };
        let mut carriers = Vec::new();
        collect(holder, &mut carriers);
        let mut tags: Vec<String> = Vec::new();
        for carrier in carriers {
            let tag = format!("styles#{}", super::agent::tag_of(carrier, handles));
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }
        if !tags.is_empty() {
            found.push((name, super::agent::format_tags(&tags)));
        }
    }
    found
}

/// The elements under `e` that record a tracked change.
fn collect<'e>(e: &'e Element, out: &mut Vec<&'e Element>) {
    for child in e.elements() {
        let local = child.local();
        if (local.ends_with("Change") || matches!(local, "ins" | "del"))
            && child.attr("id").is_some()
        {
            out.push(child);
            continue;
        }
        collect(child, out);
    }
}

fn part_two(out: &mut String, f: &Facts) {
    if let Some(first) = f.sections.first()
        && let Some(sect) = first.sect_pr.as_ref()
    {
        let mut page = page_setup(sect);
        if f.resolved.is_none()
            && let Some(tags) = section_changes(first, f.handles)
        {
            page.push_str(&format!("; fmt {tags}"));
        }
        kv(out, &format!("page: {page}"), None);
    }
    out.push_str("styles:\n");
    let default = f.default_style.unwrap_or("Normal");
    let base = resolve(f.styles, f.theme, default);
    // A style a tracked change names but no paragraph uses still gets a
    // legend line after the used ones, so the view's own docx defines it
    // and the change has a holder there. The changed list's order is the
    // styles part's; the writer keeps it, so the next view names the same
    // order.
    let changes = if f.resolved.is_none() {
        f.styles
            .map(|styles| style_changes(styles, f.handles))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    kv(
        out,
        &format!(
            "  {}: {}",
            scalar(default, false),
            scalar(
                &format!(
                    "{}{}",
                    describe(&base, None),
                    named(f.styles, default, default)
                ),
                false
            )
        ),
        Some("default; unannotated paragraphs use it"),
    );
    for (level, style) in &f.heading_styles {
        // The writer names a `HeadingN` style `heading N`, as Word does.
        let expected = style
            .strip_prefix("Heading")
            .filter(|n| n.parse::<u32>().is_ok())
            .map_or_else(|| style.clone(), |n| format!("heading {n}"));
        kv(
            out,
            &format!(
                "  \"{}\": {}",
                "#".repeat(*level),
                scalar(
                    &format!(
                        "{style}, {}{}",
                        describe(&resolve(f.styles, f.theme, style), Some(&base)),
                        named(f.styles, style, &expected)
                    ),
                    false
                )
            ),
            None,
        );
    }
    for style in &f.other_styles {
        kv(
            out,
            &format!(
                "  {}: {}",
                scalar(style, false),
                scalar(
                    &format!(
                        "{}{}",
                        describe(&resolve(f.styles, f.theme, style), Some(&base)),
                        named(f.styles, style, style)
                    ),
                    false
                )
            ),
            None,
        );
    }
    for (style, _) in &changes {
        if style.as_str() == "docDefaults"
            || style == default
            || f.heading_styles.iter().any(|(_, h)| h == style)
            || f.other_styles.iter().any(|o| o == style)
        {
            continue;
        }
        kv(
            out,
            &format!(
                "  {}: {}",
                scalar(style, false),
                scalar(
                    &format!(
                        "{}{}",
                        describe(&resolve(f.styles, f.theme, style), Some(&base)),
                        named(f.styles, style, style)
                    ),
                    false
                )
            ),
            Some("tracked changes to it"),
        );
    }
    match f.table_styles.as_slice() {
        [] => {}
        [one] if !f.tables_differ => kv(
            out,
            &format!(
                "  table: {}",
                scalar(&table_style_line(f.styles, one), false)
            ),
            None,
        ),
        many => kv(
            out,
            &format!(
                "  tables: {}",
                scalar(
                    &many
                        .iter()
                        .map(|id| table_style_line(f.styles, id))
                        .collect::<Vec<_>>()
                        .join(", "),
                    false
                )
            ),
            None,
        ),
    }
    if !changes.is_empty() {
        // Legend order: the default style first, then the styles the body
        // uses, then the ones only a change names; `docDefaults` leads.
        let order = |id: &str| {
            if id == "docDefaults" {
                return usize::MIN;
            }
            if id == default {
                return 0;
            }
            f.heading_styles
                .iter()
                .position(|(_, h)| h == id)
                .map(|i| i + 1)
                .or_else(|| {
                    f.other_styles
                        .iter()
                        .position(|o| o == id)
                        .map(|i| i + 1 + f.heading_styles.len())
                })
                .unwrap_or_else(|| {
                    changes
                        .iter()
                        .position(|(holder, _)| holder == id)
                        .map_or(usize::MAX, |i| i + usize::MAX / 2)
                })
        };
        let mut ordered = changes.clone();
        ordered.sort_by_key(|(holder, _)| order(holder));
        let list: Vec<String> = ordered
            .iter()
            .map(|(style, tags)| format!("{}: {}", scalar(style, true), scalar(tags, true)))
            .collect();
        kv(
            out,
            &format!("style_changes: {{{}}}", list.join(", ")),
            Some("tracked changes to the styles"),
        );
    }
    let (rev, fmt) = &f.note_changes;
    let mut clauses: Vec<String> = [("rev", rev), ("fmt", fmt)]
        .into_iter()
        .filter(|(_, tags)| !tags.is_empty())
        .map(|(key, tags)| format!("{key}: {}", scalar(&tags.join("; "), true)))
        .collect();
    let changes = !clauses.is_empty();
    if !f.empty_notes.is_empty() {
        clauses.push(format!("empty: [{}]", f.empty_notes.join(", ")));
    }
    if !clauses.is_empty() {
        kv(
            out,
            &format!("notes: {{{}}}", clauses.join(", ")),
            Some(if changes {
                "changes in notes the body does not show (separators)"
            } else {
                "notes parts that hold no note"
            }),
        );
    }
    if !f.unplaced.is_empty() {
        kv(
            out,
            &format!(
                "unplaced: [{}]",
                f.unplaced
                    .iter()
                    .map(|e| scalar(e, true))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Some("changes no other line shows (text boxes, Word's Fallback copies)"),
        );
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
                        section_entry(
                            section,
                            &f.sections[i - 1],
                            first,
                            f.resolved.is_none().then_some(f.handles)
                        )
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

/// How [`story_text`] reads a header or footer's tracked changes.
#[derive(Clone, Copy)]
pub(crate) enum StoryView<'a> {
    /// The agent's tracked view, with the handles and the part's stem:
    /// `{++DRAFT++}{>>header1.r1 @AC<<}`, `{--old--}{>>header1.r2 @AC<<}`;
    /// document text that reads like CriticMarkup is escaped.
    Tagged(&'a super::agent::Handles, &'a str),
    /// Every change accepted: inserted and moved-to runs count, deleted and
    /// moved-from runs do not. With the character styles that hide a run.
    Accept(&'a std::collections::HashSet<String>),
    /// Every change rejected: deleted and moved-from runs count, inserted
    /// and moved-to runs do not.
    Reject(&'a std::collections::HashSet<String>),
}

/// A header/footer paragraph's text with fields as the body prints them,
/// `{CODE}result{/}` (`{PAGE}1{/}`, `{HYPERLINK "https://…"}site{/}`), and a
/// code jubarte does not print whole as its name alone (`{XE}`), as is a
/// code holding a nested field (`{IF}`). A field nested in another's result
/// prints its result. Line breaks are newlines. Runs inside hyperlinks,
/// content controls, smart tags and custom XML count; tracked changes read
/// as `view` says.
/// [`story_text_with`] without drawing capture: the tests' view.
#[cfg(test)]
fn story_text(p: &Element, view: StoryView) -> String {
    story_text_with(p, view, &mut |_, _| None)
}

/// [`story_text`], calling `draw` for each `w:drawing`, `w:pict`, `w:object`
/// and drawing-holding `mc:AlternateContent` element: its Markdown (`None`
/// prints nothing). The `bool` says the element is run content, spliced
/// inside a `w:r` (`true`) or where a `w:r` may sit (`false`, a
/// paragraph-level AlternateContent).
pub(crate) fn story_text_with(
    p: &Element,
    view: StoryView,
    draw: &mut dyn FnMut(&Element, bool) -> Option<String>,
) -> String {
    struct State<'a> {
        out: String,
        /// Open complex fields, outermost first.
        fields: Vec<Open>,
        view: StoryView<'a>,
        /// Inside deleted text the view shows.
        deleting: bool,
        /// The tagged view's last change, while nothing follows it yet.
        last: Option<Shown>,
        /// How many printed fields' results the walk is in.
        in_result: usize,
    }
    /// One side of a change the tagged view printed: its text, its marks'
    /// tags, and their author.
    struct Side {
        text: String,
        tags: Vec<String>,
        author: Option<String>,
    }
    /// A change the tagged view printed from `start` to `end`: an
    /// insertion, a deletion, or the insertion then the deletion Word's
    /// Compare writes for a substitution.
    struct Shown {
        start: usize,
        end: usize,
        new: Option<Side>,
        old: Option<Side>,
    }
    /// A complex field being read.
    #[derive(Default)]
    struct Open {
        /// The instruction so far.
        instr: String,
        /// Past its `separate`: in its result.
        result: bool,
        /// Its code printed whole: `{CODE}`, its result, then `{/}`.
        printed: bool,
        /// A field nested in its instruction, which a code cannot print.
        nested: bool,
        /// Where its result starts in the text, once its code is printed.
        at: usize,
    }
    /// A printed field's end: `{/}`, or nothing when it has no result and
    /// its code is a bare name, which the writer reads as `{PAGE}`. A
    /// field in another's result always ends: `{PAGE}{NUMPAGES}{/}` would
    /// read as a bare PAGE, then NUMPAGES with no result.
    fn field_end(out: &str, at: usize, instr: &str, nested: bool) -> &'static str {
        let bare = !instr.trim().is_empty()
            && instr
                .trim()
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit());
        if bare && !nested && out.len() == at {
            ""
        } else {
            super::FIELD_END
        }
    }
    /// `ins`/`moveTo` → `Some(true)`, `del`/`moveFrom` → `Some(false)`.
    fn inserting(local: &str) -> Option<bool> {
        match local {
            "ins" | "moveTo" => Some(true),
            "del" | "moveFrom" => Some(false),
            _ => None,
        }
    }
    fn code(instr: &str) -> String {
        let name = instr
            .split_whitespace()
            .next()
            .unwrap_or("FIELD")
            .to_uppercase();
        format!("{{{name}}}")
    }
    fn walk(
        parent: &Element,
        s: &mut State,
        draw: &mut dyn FnMut(&Element, bool) -> Option<String>,
    ) {
        for run in parent.elements() {
            match run.local() {
                "drawing" | "pict" | "object" => {
                    if let Some(markdown) = draw(run, true) {
                        s.out.push_str(&markdown);
                    }
                }
                "AlternateContent" if super::drawing::capturable_alternate(run) => {
                    if let Some(markdown) = draw(run, false) {
                        s.out.push_str(&markdown);
                    }
                }
                "AlternateContent" => {
                    if let Some(choice) = run.child("Choice").or_else(|| run.child("Fallback")) {
                        walk(choice, s, draw);
                    }
                }
                "fldSimple" => {
                    let instr = run.attr("instr").unwrap_or("");
                    match super::printed_code(instr).filter(|_| s.fields.is_empty()) {
                        Some(code) => {
                            s.out.push_str(&code);
                            let at = s.out.len();
                            let nested = s.in_result > 0;
                            s.in_result += 1;
                            walk(run, s, draw);
                            s.in_result -= 1;
                            s.out.push_str(field_end(&s.out, at, instr, nested));
                        }
                        None if s.fields.is_empty() => s.out.push_str(&code(instr)),
                        None => walk(run, s, draw),
                    }
                }
                local if inserting(local).is_some() => {
                    let inserted = inserting(local) == Some(true);
                    let tagged = match s.view {
                        StoryView::Tagged(handles, stem) if s.fields.is_empty() => {
                            Some((handles, stem))
                        }
                        _ => None,
                    };
                    let shown = match s.view {
                        StoryView::Tagged(..) => tagged.is_some() || inserted,
                        StoryView::Accept(_) => inserted,
                        StoryView::Reject(_) => !inserted,
                    };
                    if !shown {
                        continue;
                    }
                    let start = s.out.len();
                    let deleting = std::mem::replace(&mut s.deleting, !inserted);
                    walk(run, s, draw);
                    s.deleting = deleting;
                    if let Some((handles, stem)) = tagged {
                        let text = s.out.split_off(start);
                        if !text.is_empty() {
                            let tag = format!("{stem}#{}", super::agent::tag_of(run, handles));
                            s.change(
                                inserted,
                                Side {
                                    text,
                                    tags: vec![tag],
                                    author: run.attr("author").map(str::to_string),
                                },
                            );
                        }
                    }
                }
                "hyperlink" | "smartTag" | "customXml" => walk(run, s, draw),
                "sdt" => {
                    if let Some(content) = run.child("sdtContent") {
                        walk(content, s, draw);
                    }
                }
                "r" => {
                    // A hidden run prints nothing, as in the body.
                    let hidden = match s.view {
                        StoryView::Tagged(handles, _) => &handles.hidden_styles,
                        StoryView::Accept(hidden) | StoryView::Reject(hidden) => hidden,
                    };
                    if super::agent::hidden_run(run.child("rPr"), hidden) {
                        continue;
                    }
                    for child in run.elements() {
                        run_content(child, s, draw);
                    }
                }
                _ => {}
            }
        }
    }
    /// One child of a `w:r` (or of a run-level `mc:Choice`): text, field
    /// characters, breaks, and drawings `draw` renders.
    fn run_content(
        child: &Element,
        s: &mut State,
        draw: &mut dyn FnMut(&Element, bool) -> Option<String>,
    ) {
        match child.local() {
            "drawing" | "pict" | "object" => {
                if let Some(markdown) = draw(child, true) {
                    s.out.push_str(&markdown);
                }
            }
            "AlternateContent" if super::drawing::holds_drawing(child) => {
                if let Some(markdown) = draw(child, true) {
                    s.out.push_str(&markdown);
                }
            }
            "AlternateContent" => {
                if let Some(choice) = child.child("Choice").or_else(|| child.child("Fallback")) {
                    for inner in choice.elements() {
                        run_content(inner, s, draw);
                    }
                }
            }
            _ => {
                {
                    match child.local() {
                        "fldChar" => match child.attr("fldCharType") {
                            Some("begin") => {
                                if let Some(field) = s.fields.last_mut().filter(|f| !f.result) {
                                    field.nested = true;
                                }
                                s.fields.push(Open::default());
                            }
                            Some("separate") => {
                                let outermost = s.fields.len() == 1;
                                if let Some(field) = s.fields.last_mut() {
                                    field.result = true;
                                    if outermost
                                        && !field.nested
                                        && let Some(code) = super::printed_code(&field.instr)
                                    {
                                        field.printed = true;
                                        s.out.push_str(&code);
                                        field.at = s.out.len();
                                    }
                                }
                            }
                            Some("end") => {
                                if let Some(field) = s.fields.pop()
                                    && s.fields.is_empty()
                                {
                                    s.out.push_str(&if field.printed {
                                        field_end(&s.out, field.at, &field.instr, s.in_result > 0)
                                            .to_string()
                                    } else {
                                        code(&field.instr)
                                    });
                                }
                            }
                            _ => {}
                        },
                        "instrText" | "delInstrText" => {
                            if let Some(field) = s.fields.last_mut().filter(|f| !f.result) {
                                field.instr.push_str(&child.text());
                            }
                        }
                        // Only a printed field's result shows, and not
                        // the code of a field nested in it.
                        _ if !(s.fields.is_empty()
                            || s.fields[0].printed && s.fields.iter().all(|f| f.result)) => {}
                        "t" => s.push_text(&child.text()),
                        "delText" if s.deleting => s.push_text(&child.text()),
                        "tab" => s.out.push('\t'),
                        "br" | "cr" => s.out.push('\n'),
                        _ => {}
                    }
                }
            }
        }
    }
    impl State<'_> {
        /// A tagged change, as the body prints its neighbours: marks of one
        /// kind by one author join, and an insertion directly followed by
        /// a deletion is one substitution.
        fn change(&mut self, inserted: bool, side: Side) {
            let at = self.out.len();
            let mut shown = match self.last.take().filter(|last| last.end == at) {
                Some(mut last) => {
                    let joins = |own: &Option<Side>| {
                        own.as_ref().is_some_and(|own| own.author == side.author)
                    };
                    if inserted && last.old.is_none() && joins(&last.new) {
                        let new = last.new.as_mut().expect("checked above");
                        new.text.push_str(&side.text);
                        new.tags.extend(side.tags);
                        last
                    } else if !inserted && joins(&last.old) {
                        let old = last.old.as_mut().expect("checked above");
                        old.text.push_str(&side.text);
                        old.tags.extend(side.tags);
                        last
                    } else if !inserted && last.old.is_none() && last.new.is_some() {
                        last.old = Some(side);
                        last
                    } else {
                        Shown::alone(at, inserted, side)
                    }
                }
                None => Shown::alone(at, inserted, side),
            };
            self.out.truncate(shown.start);
            let tag = |side: &Side| super::agent::join_tags(&side.tags);
            let printed = match (&shown.old, &shown.new) {
                (Some(old), Some(new)) => format!(
                    "{{~~{}~>{}~~}}{{>>{}<<}}",
                    old.text,
                    new.text,
                    super::agent::format_tag(&super::agent::join_tags(&[tag(old), tag(new)]))
                ),
                (None, Some(new)) => format!(
                    "{{++{}++}}{{>>{}<<}}",
                    new.text,
                    super::agent::format_tag(&tag(new))
                ),
                (Some(old), None) => format!(
                    "{{--{}--}}{{>>{}<<}}",
                    old.text,
                    super::agent::format_tag(&tag(old))
                ),
                (None, None) => String::new(),
            };
            self.out.push_str(&printed);
            shown.end = self.out.len();
            self.last = Some(shown);
        }

        /// Document text; the tagged view escapes what reads like a mark,
        /// and every view text that reads like a field code (`\{PAGE}`),
        /// as the body does.
        fn push_text(&mut self, text: &str) {
            let text = escape_field_codes(text);
            match self.view {
                StoryView::Tagged(..) => self.out.push_str(&super::critic::escape(&text)),
                _ => self.out.push_str(&text),
            }
        }
    }
    impl Shown {
        fn alone(start: usize, inserted: bool, side: Side) -> Self {
            let (new, old) = if inserted {
                (Some(side), None)
            } else {
                (None, Some(side))
            };
            Shown {
                start,
                end: start,
                new,
                old,
            }
        }
    }
    let mut s = State {
        out: String::new(),
        fields: Vec::new(),
        view,
        deleting: false,
        last: None,
        in_result: 0,
    };
    walk(p, &mut s, draw);
    s.out.trim().to_string()
}

/// `text` with a backslash before each `{` the writer would read as a
/// field: one that opens what reads like a field code (`{PAGE \* roman}`),
/// a bare name (`{SELLER}`, `{1999}`) or a result's end (`{/}`), so text
/// typed in a header stays text.
fn escape_field_codes(text: &str) -> std::borrow::Cow<'_, str> {
    use crate::markdown::critic::{field_code_close, is_field_code};
    let opens = |at: usize| {
        let after = &text[at + 1..];
        after.starts_with("/}")
            || after.find('}').is_some_and(|end| {
                end > 0
                    && after[..end]
                        .bytes()
                        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            })
            || field_code_close(after.char_indices())
                .is_some_and(|close| is_field_code(&after[..close]))
    };
    if !text.char_indices().any(|(at, c)| c == '{' && opens(at)) {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 2);
    for (at, c) in text.char_indices() {
        if c == '{' && opens(at) {
            out.push('\\');
        }
        out.push(c);
    }
    std::borrow::Cow::Owned(out)
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
    fn a_section_s_universal_measures_read_as_twips() {
        // Strict packages write `595.3pt` where Transitional writes `11906`;
        // none of these are the fallback defaults, so a lost decode says so.
        let strict = parse_xml(
            format!(
                r#"<w:sectPr {W}><w:pgSz w:w="595.3pt" w:h="841.9pt"/><w:pgMar w:top="54pt" w:right="0.75in" w:bottom="54pt" w:left="0.75in" w:header="54pt" w:footer="54pt"/></w:sectPr>"#
            )
            .as_bytes(),
        )
        .unwrap();
        let transitional = parse_xml(
            format!(
                r#"<w:sectPr {W}><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1080" w:right="1080" w:bottom="1080" w:left="1080" w:header="1080" w:footer="1080"/></w:sectPr>"#
            )
            .as_bytes(),
        )
        .unwrap();
        assert_eq!(
            page_setup(&strict),
            "A4 portrait, margins 0.75in, header/footer 0.75in"
        );
        assert_eq!(page_setup(&strict), page_setup(&transitional));
    }

    /// An mm-spelled page is the page its twips twin names: 210mm x 297mm
    /// is A4 once rounded to the twips it is (11905.5 → 11906), not the
    /// `8.268x11.693in` a truncating cast reads from any producer whose
    /// spelling is not Word's own `595.3pt` (#531 review defect 6).
    #[test]
    fn an_mm_spelled_page_is_the_page_its_twips_twin_names() {
        let mm = parse_xml(
            format!(r#"<w:sectPr {W}><w:pgSz w:w="210mm" w:h="297mm"/></w:sectPr>"#).as_bytes(),
        )
        .unwrap();
        let twin = parse_xml(
            format!(r#"<w:sectPr {W}><w:pgSz w:w="11906" w:h="16838"/></w:sectPr>"#).as_bytes(),
        )
        .unwrap();
        assert_eq!(page_setup(&mm), "A4 portrait");
        assert_eq!(page_setup(&mm), page_setup(&twin));
    }

    /// Word's paper list: every size Word names and the writer writes
    /// back exactly is named by the rounded twips Word spells it in — the
    /// short side first, whatever way the sheet lies (#561 review defect
    /// 4). Tabloid is the sheet Word's list names Ledger lying landscape.
    #[test]
    fn word_s_paper_list_names_its_sizes() {
        let cases = [
            (12_240, 15_840, "", "Letter portrait"),
            (12_240, 20_160, "", "Legal portrait"),
            (10_440, 15_120, "", "Executive portrait"),
            (15_840, 24_480, "", "Tabloid portrait"),
            (
                24_480,
                15_840,
                r#" w:orient="landscape""#,
                "Ledger landscape",
            ),
            (16_838, 23_811, "", "A3 portrait"),
            (11_906, 16_838, "", "A4 portrait"),
            (8_391, 11_906, "", "A5 portrait"),
            (14_570, 20_636, "", "B4 (JIS) portrait"),
            (10_318, 14_570, "", "B5 (JIS) portrait"),
        ];
        for (w, h, orient, expected) in cases {
            let sect = parse_xml(
                format!(r#"<w:sectPr {W}><w:pgSz w:w="{w}" w:h="{h}"{orient}/></w:sectPr>"#)
                    .as_bytes(),
            )
            .unwrap();
            assert_eq!(page_setup(&sect), expected, "{w}x{h}");
        }
        // A twip off the named size is not the named size: the rounded
        // match is exact, so an unnamed sheet keeps its inches.
        let off = parse_xml(
            format!(r#"<w:sectPr {W}><w:pgSz w:w="8392" w:h="11906"/></w:sectPr>"#).as_bytes(),
        )
        .unwrap();
        assert_eq!(page_setup(&off), "5.828x8.268in portrait");
    }

    /// A strict table style names its side borders `start`/`end`: the
    /// legend reads them as the `left`/`right` they name, and one side a
    /// twip apart still varies.
    #[test]
    fn a_table_style_s_start_end_borders_read_as_left_right() {
        let borders = |sides: &str| {
            parse_xml(
                format!(
                    r#"<w:styles {W}><w:style w:type="table" w:styleId="T"><w:tblPr><w:tblBorders>{sides}</w:tblBorders></w:tblPr></w:style></w:styles>"#
                )
                .as_bytes(),
            )
            .unwrap()
        };
        let strict = borders(
            r#"<w:top w:val="single" w:sz="4"/><w:start w:val="single" w:sz="4"/><w:bottom w:val="single" w:sz="4"/><w:end w:val="single" w:sz="4"/><w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="4"/>"#,
        );
        let transitional = borders(
            r#"<w:top w:val="single" w:sz="4"/><w:left w:val="single" w:sz="4"/><w:bottom w:val="single" w:sz="4"/><w:right w:val="single" w:sz="4"/><w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="4"/>"#,
        );
        assert_eq!(table_style_line(Some(&strict), "T"), "T, all borders 0.5pt");
        assert_eq!(
            table_style_line(Some(&transitional), "T"),
            table_style_line(Some(&strict), "T")
        );
        let varied = borders(
            r#"<w:top w:val="single" w:sz="4"/><w:start w:val="single" w:sz="8"/><w:bottom w:val="single" w:sz="4"/><w:end w:val="single" w:sz="4"/><w:insideH w:val="single" w:sz="4"/><w:insideV w:val="single" w:sz="4"/>"#,
        );
        assert_eq!(table_style_line(Some(&varied), "T"), "T, borders vary");
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
    fn a_story_field_prints_its_whole_code_and_its_result() {
        // As in the body: `{CODE}result{/}`, so a link's target and text
        // and a field's switches survive the round trip.
        let root = ftr(&format!(
            r#"<w:p><w:r><w:t>Page </w:t></w:r>{PAGE_RUNS}<w:r><w:t> · </w:t></w:r><w:fldSimple w:instr=" HYPERLINK &quot;https://x.example&quot; "><w:r><w:t>site</w:t></w:r></w:fldSimple></w:p>"#
        ));
        assert_eq!(
            story_text(
                story_paragraphs(&root)[0],
                StoryView::Accept(&Default::default())
            ),
            r#"Page {PAGE}1{/} · {HYPERLINK "https://x.example"}site{/}"#
        );
    }

    #[test]
    fn a_story_field_without_a_result_prints_its_bare_code() {
        // The writer's `{PAGE}`: no result, so none is printed; a field
        // with switches keeps the coded form.
        let root = ftr(
            r#"<w:p><w:fldSimple w:instr=" PAGE "/><w:r><w:t> of </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t> </w:t></w:r><w:fldSimple w:instr=" PAGE \* roman "/></w:p>"#,
        );
        assert_eq!(
            story_text(
                story_paragraphs(&root)[0],
                StoryView::Accept(&Default::default())
            ),
            r#"{PAGE} of {NUMPAGES} {PAGE \* roman}{/}"#
        );
    }

    #[test]
    fn a_page_field_in_a_block_content_control_is_found() {
        let root = ftr(&format!(
            "<w:sdt><w:sdtPr/><w:sdtContent><w:p>{PAGE_RUNS}</w:p></w:sdtContent></w:sdt><w:p/>"
        ));
        let paragraphs = story_paragraphs(&root);
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(
            story_text(paragraphs[0], StoryView::Accept(&Default::default())),
            "{PAGE}1{/}"
        );
        assert_eq!(
            story_text(paragraphs[1], StoryView::Accept(&Default::default())),
            ""
        );
    }

    #[test]
    fn runs_inside_inline_wrappers_count_and_deletions_do_not() {
        let root = ftr(&format!(
            r#"<w:p><w:hyperlink><w:r><w:t>Page </w:t></w:r></w:hyperlink><w:sdt><w:sdtContent>{PAGE_RUNS}</w:sdtContent></w:sdt><w:ins><w:r><w:t> of </w:t></w:r></w:ins><w:del><w:r><w:delText>gone</w:delText></w:r></w:del><w:fldSimple w:instr=" numpages "><w:r><w:t>3</w:t></w:r></w:fldSimple></w:p>"#
        ));
        assert_eq!(
            story_text(
                story_paragraphs(&root)[0],
                StoryView::Accept(&Default::default())
            ),
            "Page {PAGE}1{/} of {numpages}3{/}"
        );
    }

    #[test]
    fn paragraphs_number_as_edit_does_tables_in_and_text_boxes_out() {
        // edit numbers a story's every w:p outside text boxes, table cells
        // included, so `footer1.p1` here is the paragraph edit numbers 1.
        let root = ftr(
            r#"<w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:p><w:r><w:pict><w:txbxContent><w:p><w:r><w:t>boxed</w:t></w:r></w:p></w:txbxContent></w:pict></w:r><w:r><w:t>body</w:t></w:r></w:p>"#,
        );
        let paragraphs = story_paragraphs(&root);
        assert_eq!(paragraphs.len(), 2);
        assert_eq!(
            story_text(paragraphs[0], StoryView::Accept(&Default::default())),
            ""
        );
        assert_eq!(
            story_text(paragraphs[1], StoryView::Accept(&Default::default())),
            "body"
        );
    }

    /// pi review r392 F1/F6: values a YAML 1.1 or 1.2 reader takes as a
    /// number, a time or a date are quoted; prose that starts with a digit
    /// stays plain.
    #[test]
    fn number_and_time_like_values_are_quoted() {
        for value in [
            "0x2A",
            "0o17",
            "+.inf",
            ".Inf",
            "12:30",
            "1:30:45",
            "1_000",
            "2026-10-01",
            "42",
            "1e5",
            "+1",
        ] {
            assert_eq!(scalar(value, false), format!("\"{value}\""), "{value}");
        }
        for value in ["1. Introduction", "3 survive", "Page 2", "x0x2A"] {
            assert_eq!(scalar(value, true), value, "{value}");
        }
    }

    /// pi review av2 F18: a line break inside a header paragraph is a line
    /// break in its text, not two words glued together.
    #[test]
    fn a_break_in_a_header_line_is_a_newline() {
        let root = ftr(
            r#"<w:p><w:r><w:t>line one</w:t><w:br/><w:t>line two</w:t><w:cr/><w:t>three</w:t></w:r></w:p>"#,
        );
        assert_eq!(
            story_text(
                story_paragraphs(&root)[0],
                StoryView::Accept(&Default::default())
            ),
            "line one\nline two\nthree"
        );
    }

    /// pi review av2 F18: a field nested in another's instruction does not
    /// replace it; the outer field names the code, its result stays hidden.
    #[test]
    fn a_nested_field_keeps_the_outer_code() {
        let root = ftr(&format!(
            r#"<w:p><w:r><w:t>Page </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> IF </w:instrText></w:r>{PAGE_RUNS}<w:r><w:instrText xml:space="preserve"> > 1 "more" "" </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>more</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t> end</w:t></w:r></w:p>"#
        ));
        assert_eq!(
            story_text(
                story_paragraphs(&root)[0],
                StoryView::Accept(&Default::default())
            ),
            "Page {IF} end"
        );
    }

    /// A paragraph indent, contextual spacing and keep-lines are named;
    /// a style with none of them keeps today's line.
    #[test]
    fn the_legend_names_an_indent_contextual_spacing_and_keep_lines() {
        let xml = format!(
            r#"<w:styles {W}><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:sz w:val="22"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after="160" w:line="259" w:lineRule="auto"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:styleId="Heading1"><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:keepLines/><w:spacing w:before="240" w:after="80"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style><w:style w:styleId="HeadingPlain"><w:basedOn w:val="Normal"/><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="80"/></w:pPr><w:rPr><w:b/><w:sz w:val="32"/></w:rPr></w:style><w:style w:styleId="ListParagraph"><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="720"/><w:contextualSpacing/></w:pPr></w:style><w:style w:styleId="First"><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="720" w:firstLine="360"/></w:pPr></w:style><w:style w:styleId="Hang"><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="720" w:hanging="360"/></w:pPr></w:style><w:style w:styleId="Both"><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="2520" w:hanging="2520"/></w:pPr></w:style><w:style w:styleId="Zero"><w:basedOn w:val="Normal"/><w:pPr><w:ind w:left="0"/></w:pPr></w:style><w:style w:styleId="Plain"><w:basedOn w:val="Normal"/></w:style></w:styles>"#
        );
        let styles = parse_xml(xml.as_bytes()).unwrap();
        let base = resolve(Some(&styles), None, "Normal");
        let line = |id: &str| describe(&resolve(Some(&styles), None, id), Some(&base));
        assert_eq!(
            describe(&base, None),
            "Calibri 11pt, after 8pt, line 1.08, left"
        );
        assert_eq!(
            line("ListParagraph"),
            "Calibri 11pt, indent 0.5in, contextual spacing"
        );
        assert_eq!(
            line("Heading1"),
            "Calibri bold 16pt, before 12pt, after 4pt, keep-next, keep-lines"
        );
        assert_eq!(
            line("HeadingPlain"),
            "Calibri bold 16pt, before 12pt, after 4pt, keep-next"
        );
        assert_eq!(
            line("First"),
            "Calibri 11pt, indent 0.5in, first-line 0.25in"
        );
        assert_eq!(line("Hang"), "Calibri 11pt, indent 0.5in, hanging 0.25in");
        assert_eq!(line("Both"), "Calibri 11pt, indent 1.75in, hanging 1.75in");
        assert_eq!(line("Zero"), "Calibri 11pt, indent 0in");
        assert_eq!(line("Plain"), "Calibri 11pt");
        let flat = Resolved {
            font: Some("Calibri".to_string()),
            size_half_points: Some(22.0),
            left: Some(0.0),
            ..Resolved::default()
        };
        assert_eq!(describe(&flat, None), "Calibri 11pt, left");
    }

    /// A strict style carries its measures as universal measures (`11pt`,
    /// `1in`) and its font size in half-points either way: the legend reads
    /// the same as the transitional integers.
    #[test]
    fn a_style_s_universal_measures_read_as_their_integers() {
        let legend = |rpr: &str, ppr: &str| {
            let xml = format!(
                r#"<w:styles {W}><w:style w:type="paragraph" w:styleId="S"><w:name w:val="S"/><w:pPr>{ppr}</w:pPr><w:rPr>{rpr}</w:rPr></w:style></w:styles>"#
            );
            let styles = parse_xml(xml.as_bytes()).unwrap();
            describe(&resolve(Some(&styles), None, "S"), None)
        };
        assert_eq!(
            legend(
                r#"<w:rFonts w:ascii="Calibri"/><w:sz w:val="16pt"/>"#,
                r#"<w:spacing w:before="10pt" w:after="5pt" w:line="12.95pt" w:lineRule="auto"/><w:ind w:start="0.25in"/>"#
            ),
            legend(
                r#"<w:rFonts w:ascii="Calibri"/><w:sz w:val="32"/>"#,
                r#"<w:spacing w:before="200" w:after="100" w:line="259" w:lineRule="auto"/><w:ind w:left="360"/>"#
            )
        );
    }
}
