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
    pub date: Option<String>,
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

/// `key: value` padded to 35 columns then `# comment`; a longer key/value
/// takes two spaces.
fn kv(out: &mut String, key_value: &str, comment: Option<&str>) {
    match comment {
        Some(c) if key_value.len() <= 33 => out.push_str(&format!("{key_value:<35}# {c}\n")),
        Some(c) => out.push_str(&format!("{key_value}  # {c}\n")),
        None => {
            out.push_str(key_value);
            out.push('\n');
        }
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
    kv(&mut out, &format!("source: {}", f.source), None);
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
        kv(&mut out, "revisions: 0", None);
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
            &format!("  document_owner: {name}"),
            Some("dc:creator"),
        ),
        Owner::LastModifiedBy(name) => kv(
            &mut out,
            &format!("  document_owner: {name}"),
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
            &format!("  {handle}: {author}"),
            Some(&join(&parts)),
        );
    }
    let last = f.paragraphs.saturating_sub(1);
    kv(
        &mut out,
        &format!(
            "body: p0-p{last}, {}, {}",
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
        kv(&mut out, &format!("range: {range} of p0-p{last}"), None);
    }
    part_two(&mut out, f);
    out.push_str("---\n");
    out
}

/// Page setup, styles, headers, footers and sections (Task 11).
fn part_two(_out: &mut String, _f: &Facts) {}
