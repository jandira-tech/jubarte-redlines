// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The agent view: numbering that matches `inspect` and `edit`, revision
//! tags, author handles and timestamps, comment threads, id lines, table
//! lines and block selection.

use std::collections::{BTreeSet, HashMap, HashSet};

use super::ooxml::{Element, Node};
use crate::markdown::{Pick, Select};

// The stamps start with U+E000, which no XML name can hold, so a file
// cannot forge them.
/// Attribute stamped on every numbered `w:p`: its `pN` index.
pub(crate) const INDEX: &str = "\u{E000}jubarteIndex";
/// Attribute stamped on every top-level `w:tbl`: its `t{N}` number.
pub(crate) const TABLE: &str = "\u{E000}jubarteTable";
/// Attribute stamped on a `w:p` that holds revisions: `kind:tag` entries
/// separated by spaces (`ins:0@AC sub:1+2@AC fmt:3@JD mark-ins:4@AC`),
/// recorded before resolution.
pub(crate) const REVS: &str = "\u{E000}jubarteRevs";

/// Author handles, order and timestamps for tags, notes and the header.
#[derive(Debug, Default, Clone)]
pub(crate) struct Handles {
    /// Author name as stored → handle (`Ann Counsel` → `AC`).
    pub by_author: HashMap<String, String>,
    /// Authors in first-appearance order: revision authors in document
    /// order, then comment-only authors in part order, then authors only the
    /// styles' changes name.
    pub order: Vec<String>,
    /// Author → every `w:date` on their marks and comments.
    pub dates: HashMap<String, BTreeSet<String>>,
    /// Comment ids whose ranges the view prints (`{==…==}`) and bookmark
    /// names it prints (`<a id>`): either one between two marks keeps them
    /// apart, as the text shows them.
    pub shown_ranges: HashSet<String>,
    pub shown_bookmarks: HashSet<String>,
    /// Character styles whose runs are hidden (`w:vanish`): such a run
    /// prints nothing, as a directly hidden one.
    pub hidden_styles: HashSet<String>,
}

impl Handles {
    pub(crate) fn of(&self, author: Option<&str>) -> Option<&str> {
        author
            .and_then(|a| self.by_author.get(a))
            .map(String::as_str)
    }

    /// The one timestamp all of the author's marks and comments share.
    pub(crate) fn unique_date(&self, author: &str) -> Option<&str> {
        let dates = self.dates.get(author)?;
        (dates.len() == 1)
            .then(|| dates.iter().next().map(String::as_str))
            .flatten()
    }

    /// `first..last` days of an author with several timestamps.
    pub(crate) fn date_range(&self, author: &str) -> Option<String> {
        let dates = self.dates.get(author)?;
        let day = |d: &String| d.get(..10).unwrap_or(d).to_string();
        match (dates.iter().next(), dates.iter().next_back()) {
            (Some(first), Some(last)) if dates.len() > 1 => {
                Some(format!("{}..{}", day(first), day(last)))
            }
            _ => None,
        }
    }

    /// Whether a note of this author needs its timestamp inline (`--dates`).
    pub(crate) fn needs_date(&self, author: Option<&str>) -> bool {
        author.is_some_and(|a| self.dates.get(a).is_some_and(|d| d.len() > 1))
    }
}

/// One logical revision as the id line and the header count it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RevTag {
    /// `ins`, `del`, `sub`, `mark` (paragraph mark), `row`, `cell`.
    pub kind: &'static str,
    /// Internal form: `0@AC`, `1+2@AC`, `1@AC|2@JD`.
    pub tag: String,
    pub author: Option<String>,
    pub date: Option<String>,
}

fn is_revision(local: &str) -> bool {
    matches!(
        local,
        "ins" | "del" | "moveTo" | "moveFrom" | "cellIns" | "cellDel"
    )
}

fn kind_of(local: &str) -> Option<&'static str> {
    match local {
        "ins" | "moveTo" => Some("ins"),
        "del" | "moveFrom" => Some("del"),
        _ => None,
    }
}

/// A change mark that holds no text (`<w:ins/>`, or runs of formatting
/// only): the view shows nothing for it, so it neither joins nor splits.
fn hollow(e: &Element) -> bool {
    kind_of(e.local()).is_some()
        && e.elements()
            .all(|r| r.local() == "r" && r.elements().all(|c| c.local() == "rPr"))
}

/// A run the text prints nothing for: vanished, or holding only empty
/// text and page breaks (the page break is the paragraph's id line). It
/// keeps no two marks apart, as in the text.
fn silent_run(e: &Element, handles: &Handles) -> bool {
    if e.local() != "r" {
        return false;
    }
    if hidden_run(e.child("rPr"), &handles.hidden_styles) {
        return true;
    }
    e.elements().all(|c| match c.local() {
        // The text prints no soft hyphen and no symbol font's glyph.
        "rPr" | "lastRenderedPageBreak" | "softHyphen" | "sym" => true,
        "t" | "delText" => c.text().is_empty(),
        "br" | "cr" => c.attr("type") == Some("page"),
        _ => false,
    })
}

/// Whether a run of properties `rpr` is hidden: its own `w:vanish`, else
/// its character style's (one of `hidden`).
pub(crate) fn hidden_run(rpr: Option<&Element>, hidden: &HashSet<String>) -> bool {
    rpr.and_then(|r| r.toggle("vanish")).unwrap_or_else(|| {
        rpr.and_then(|r| r.child("rStyle"))
            .and_then(|s| s.attr("val"))
            .is_some_and(|id| hidden.contains(id))
    })
}

/// Whether `mark` holds the start of a shown comment range that ends past
/// it, and the end of one that starts before it.
fn half_ranges(mark: &Element, handles: &Handles) -> (bool, bool) {
    fn walk(e: &Element, handles: &Handles, starts: &mut Vec<String>, ends: &mut Vec<String>) {
        for child in e.elements() {
            let shown = || {
                child
                    .attr("id")
                    .filter(|id| handles.shown_ranges.contains(*id))
                    .map(str::to_string)
            };
            match child.local() {
                "commentRangeStart" => starts.extend(shown()),
                "commentRangeEnd" => ends.extend(shown()),
                _ => walk(child, handles, starts, ends),
            }
        }
    }
    let (mut starts, mut ends) = (Vec::new(), Vec::new());
    walk(mark, handles, &mut starts, &mut ends);
    (
        starts.iter().any(|id| !ends.contains(id)),
        ends.iter().any(|id| !starts.contains(id)),
    )
}

/// `12@AC`: the element's `w:id` and its author's handle (`??` for an
/// author the legend does not know: a malformed file, or Word's
/// `w:tblGridChange`, which has no author).
pub(crate) fn tag_of(element: &Element, handles: &Handles) -> String {
    let id = element.attr("id").map_or_else(|| "?".to_string(), tag_id);
    let handle = handles.of(element.attr("author")).unwrap_or("??");
    format!("{id}@{handle}")
}

/// `12@Ana%20Lima%20(2026-…)` for the plain view: the element's `w:id` and
/// its `Author (date)` note in place of a handle, spelled ([`tag_id`],
/// spaces too) so that tags split on none of it.
pub(crate) fn plain_tag_of(element: &Element, note: &str) -> String {
    let id = element.attr("id").map_or_else(|| "?".to_string(), tag_id);
    format!("{id}@{}", tag_id(note).replace(' ', "%20"))
}

/// A `w:id` as an internal tag holds it: the characters tags join and
/// split on (`+`, `|`, `@`, `#`, `?`) and `%` itself spelled `%XX`, so a
/// malformed id such as `1+2` stays one id. [`untag_id`] reverses it.
fn tag_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len());
    for c in id.chars() {
        if matches!(c, '+' | '|' | '@' | '#' | '?' | '%') {
            out.push_str(&format!("%{:02X}", u32::from(c)));
        } else {
            out.push(c);
        }
    }
    out
}

/// The `w:id` one piece of an internal tag spells ([`tag_id`]).
fn untag_id(piece: &str) -> String {
    let mut out = String::with_capacity(piece.len());
    let mut rest = piece;
    while let Some(at) = rest.find('%') {
        out.push_str(&rest[..at]);
        match rest
            .get(at + 1..at + 3)
            .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        {
            Some(byte) => {
                out.push(char::from(byte));
                rest = &rest[at + 3..];
            }
            None => {
                out.push('%');
                rest = &rest[at + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// One internal tag's parts: `header1#1+2@AC` → (`Some("header1")`,
/// `1+2`, `AC`). The story is absent in the body's tags. Internal only:
/// [`format_tag`] prints it.
pub(crate) fn split_tag(one: &str) -> (Option<&str>, &str, &str) {
    let (head, handle) = one.rsplit_once('@').unwrap_or((one, "??"));
    match head.split_once('#') {
        Some((story, ids)) => (Some(story), ids, handle),
        None => (None, head, handle),
    }
}

/// The handle of a one-author tag (`1+2@AC` → `AC`); `None` for several.
pub(crate) fn handle_of(tag: &str) -> Option<&str> {
    if tag.contains('|') {
        return None;
    }
    Some(split_tag(tag).2)
}

/// Joins the tags of one logical change: `1@AC` + `2@AC` is `1+2@AC`,
/// `1@AC` + `2@JD` is `1@AC|2@JD`, `1+2@AC` + `3@AC` is `1+2+3@AC`. Tags
/// join only within one story and one handle.
pub(crate) fn join_tags(parts: &[String]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    let keys: Vec<Option<(Option<&str>, &str)>> = parts
        .iter()
        .map(|p| {
            (!p.contains('|')).then(|| {
                let (story, _, handle) = split_tag(p);
                (story, handle)
            })
        })
        .collect();
    match keys[0] {
        Some((story, handle)) if keys.iter().all(|k| *k == keys[0]) => {
            let ids: Vec<&str> = parts.iter().map(|p| split_tag(p).1).collect();
            let story = story.map_or(String::new(), |s| format!("{s}#"));
            format!("{story}{}@{handle}", ids.join("+"))
        }
        _ => parts.join("|"),
    }
}

/// `0@AC` → `r0 @AC`; `1+2@AC` → `r1+2 @AC`; `header1#2@JD` →
/// `header1.r2 @JD`; `1@AC|2@JD` → `r1 @AC; r2 @JD`. An id the short form
/// cannot spell prints its long id, one per id: every printed id is one
/// `accept --id` takes. A mark without a `w:id` (only a malformed file has
/// one) prints `noid`: it names no change, and no id parser reads it.
pub(crate) fn format_tag(tag: &str) -> String {
    format_tag_by(tag, |handle| format!(" @{handle}"))
}

/// A plain-view tag ([`plain_tag_of`]) as its note prints it: the ids,
/// then the `Author (date)` note when there is one (`r1+2 Ana Lima
/// (2026-09-29T14:05:00Z)`, `r7`).
pub(crate) fn format_plain_tag(tag: &str) -> String {
    format_tag_by(tag, |note| match untag_id(note) {
        note if note.is_empty() => note,
        note => format!(" {note}"),
    })
}

/// [`format_tag`] with `by` spelling what follows the ids.
fn format_tag_by(tag: &str, by: impl Fn(&str) -> String) -> String {
    tag.split('|')
        .map(|one| {
            let (story, ids, handle) = split_tag(one);
            if ids.split('+').any(|w| w == "?") {
                let story = story.map_or(String::new(), |s| format!("{s}."));
                return format!("{story}noid{}", by(handle));
            }
            let changes: Vec<crate::ids::ChangeId> = ids
                .split('+')
                .map(|w| crate::ids::ChangeId::new(story.unwrap_or("body"), untag_id(w)))
                .collect();
            let short: Vec<String> = changes.iter().map(crate::ids::ChangeId::short).collect();
            if short.iter().all(|s| !s.contains(":rev:")) {
                let numbers: Vec<&str> = changes.iter().map(|c| c.wid.as_str()).collect();
                let head = short[0]
                    .strip_suffix(changes[0].wid.as_str())
                    .unwrap_or("r");
                format!("{head}{}{}", numbers.join("+"), by(handle))
            } else {
                short
                    .iter()
                    .map(|s| format!("{s}{}", by(handle)))
                    .collect::<Vec<_>>()
                    .join("; ")
            }
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// A list of tags for an id line: `r0 @AC; r1+2 @AC`.
pub(crate) fn format_tags(tags: &[String]) -> String {
    tags.iter()
        .map(|t| format_tag(t))
        .collect::<Vec<_>>()
        .join("; ")
}

/// The revisions among a paragraph's direct children, in order, as the
/// renderer shows them: neighbouring marks of one kind by one author join
/// (`7+8@AC`), then an insertion directly followed by a deletion pairs as
/// a substitution, greedily left to right. A run breaks adjacency, and so
/// does a comment range or a bookmark the text prints
/// ([`Handles::shown_ranges`]); proofing marks and the rest do not.
/// Revisions inside links, content controls and simple fields count.
pub(crate) fn revision_tags(p: &Element, handles: &Handles) -> Vec<RevTag> {
    // One slot per child: a revision, or `None` for anything else that
    // breaks adjacency.
    // Links, content controls, smart tags, custom XML and simple fields are
    // transparent: the renderer prints their runs inline, so their
    // revisions count and neighbour the ones around them.
    fn flatten<'a>(parent: &'a Element, out: &mut Vec<&'a Element>) {
        for e in parent.elements() {
            match e.local() {
                "hyperlink" | "smartTag" | "customXml" | "fldSimple" | "bdo" | "dir" => {
                    flatten(e, out);
                }
                "sdt" => {
                    if let Some(content) = e.child("sdtContent") {
                        flatten(content, out);
                    }
                }
                _ => out.push(e),
            }
        }
    }
    let mut children = Vec::new();
    flatten(p, &mut children);
    let mut slots: Vec<Option<RevTag>> = Vec::new();
    for e in children {
        // A range or a bookmark the text prints sits between the marks.
        let shown = match e.local() {
            "commentRangeStart" | "commentRangeEnd" => e
                .attr("id")
                .is_some_and(|id| handles.shown_ranges.contains(id)),
            "bookmarkStart" => e
                .attr("name")
                .is_some_and(|name| handles.shown_bookmarks.contains(name)),
            _ => false,
        };
        if shown {
            slots.push(None);
            continue;
        }
        if matches!(
            e.local(),
            "bookmarkStart"
                | "bookmarkEnd"
                | "proofErr"
                | "commentRangeStart"
                | "commentRangeEnd"
                | "moveFromRangeStart"
                | "moveFromRangeEnd"
                | "moveToRangeStart"
                | "moveToRangeEnd"
                | "permStart"
                | "permEnd"
                | "customXmlInsRangeStart"
                | "customXmlInsRangeEnd"
                | "customXmlDelRangeStart"
                | "customXmlDelRangeEnd"
                | "customXmlMoveFromRangeStart"
                | "customXmlMoveFromRangeEnd"
                | "customXmlMoveToRangeStart"
                | "customXmlMoveToRangeEnd"
        ) || hollow(e)
            || silent_run(e, handles)
        {
            continue;
        }
        // A mark holding one end of a shown comment range: the text
        // prints the range across its edge, which parts it from the mark
        // on that side.
        let (opens, closes) = if kind_of(e.local()).is_some() {
            half_ranges(e, handles)
        } else {
            (false, false)
        };
        if closes {
            slots.push(None);
        }
        slots.push(kind_of(e.local()).map(|kind| RevTag {
            kind,
            tag: tag_of(e, handles),
            author: e.attr("author").map(str::to_string),
            date: e.attr("date").map(str::to_string),
        }));
        if opens {
            slots.push(None);
        }
    }
    // 1. Join neighbours of one kind by one author.
    let mut joined: Vec<Option<RevTag>> = Vec::new();
    for slot in slots {
        match (joined.last_mut(), slot) {
            (Some(Some(last)), Some(next))
                if last.kind == next.kind && last.author == next.author =>
            {
                last.tag = join_tags(&[last.tag.clone(), next.tag]);
            }
            (_, slot) => joined.push(slot),
        }
    }
    // 2. Pair an insertion with the deletion after it.
    let mut out = Vec::new();
    let mut i = 0;
    while i < joined.len() {
        let Some(current) = joined[i].take() else {
            i += 1;
            continue;
        };
        let next_kind = joined.get(i + 1).and_then(|n| n.as_ref()).map(|n| n.kind);
        // Word Compare's order: the insertion, then the deletion. A
        // deletion before an insertion stays two changes, as the text
        // shows it.
        let pair = matches!((current.kind, next_kind), ("ins", Some("del")));
        if pair {
            let next = joined[i + 1].take().expect("checked above");
            let (old, new) = if current.kind == "del" {
                (current, next)
            } else {
                (next, current)
            };
            out.push(RevTag {
                kind: "sub",
                tag: join_tags(&[old.tag, new.tag]),
                author: old.author,
                date: old.date,
            });
            i += 2;
        } else {
            out.push(current);
            i += 1;
        }
    }
    out
}

/// Tracked paragraph mark: the `w:ins` / `w:del` under `w:pPr/w:rPr`.
pub(crate) fn mark_tags(p: &Element, handles: &Handles) -> (Vec<String>, Vec<String>) {
    let mut ins = Vec::new();
    let mut del = Vec::new();
    if let Some(rpr) = p.path(&["pPr", "rPr"]) {
        for e in rpr.elements() {
            match kind_of(e.local()) {
                Some("ins") => ins.push(tag_of(e, handles)),
                Some("del") => del.push(tag_of(e, handles)),
                _ => {}
            }
        }
    }
    (ins, del)
}

/// Tags of the formatting changes recorded in a paragraph (its runs' and
/// its own properties, its list number), a text box Word stores twice read
/// once.
pub(crate) fn format_change_tags(p: &Element, handles: &Handles) -> Vec<String> {
    fn walk(e: &Element, handles: &Handles, out: &mut Vec<String>) {
        for child in shown_children(e) {
            if child.is("rPrChange") || child.is("pPrChange") || child.is("numberingChange") {
                out.push(tag_of(child, handles));
            } else {
                walk(child, handles, out);
            }
        }
    }
    let mut found = Vec::new();
    walk(p, handles, &mut found);
    found
}

struct Counter {
    p: usize,
    t: usize,
}

/// Numbers every `w:p` under `body` as `inspect::body_paragraph_nodes`
/// does (document order, text boxes excluded), numbers top-level tables,
/// and records each paragraph's revision tags. Returns (paragraphs, tables).
pub(crate) fn stamp(body: &mut Element, handles: &Handles) -> (usize, usize) {
    let mut counter = Counter { p: 0, t: 0 };
    stamp_in(body, &mut counter, handles, false);
    (counter.p, counter.t)
}

fn stamp_in(element: &mut Element, c: &mut Counter, handles: &Handles, in_cell: bool) {
    if element.is("p") {
        // Content revisions, then formatting changes, then the paragraph
        // mark: everything resolution may strip.
        let (mark_ins, mark_del) = mark_tags(element, handles);
        let revs: Vec<String> = revision_tags(element, handles)
            .into_iter()
            .map(|t| format!("{}:{}", t.kind, t.tag))
            .chain(
                format_change_tags(element, handles)
                    .into_iter()
                    .map(|t| format!("fmt:{t}")),
            )
            .chain(mark_ins.into_iter().map(|t| format!("mark-ins:{t}")))
            .chain(mark_del.into_iter().map(|t| format!("mark-del:{t}")))
            .collect();
        element.attrs.push((INDEX.to_string(), c.p.to_string()));
        c.p += 1;
        if !revs.is_empty() {
            element.attrs.push((REVS.to_string(), revs.join(" ")));
        }
    }
    // Row and cell revisions, which resolution strips with the `trPr` and
    // `tcPr` markers: the table line names them by row and cell.
    let held = match element.local() {
        "tr" => element.child("trPr").map(|pr| ("row", pr)),
        "tc" => element.child("tcPr").map(|pr| ("cell", pr)),
        _ => None,
    };
    if let Some((kind, pr)) = held {
        let revs: Vec<String> = pr
            .elements()
            .filter(|m| matches!(m.local(), "ins" | "del" | "cellIns" | "cellDel"))
            .map(|m| format!("{kind}:{}", tag_of(m, handles)))
            .collect();
        if !revs.is_empty() {
            element.attrs.push((REVS.to_string(), revs.join(" ")));
        }
    }
    if element.is("tbl") && !in_cell {
        element.attrs.push((TABLE.to_string(), c.t.to_string()));
        c.t += 1;
    }
    let in_cell = in_cell || element.is("tc");
    for child in element.children.iter_mut() {
        if let Node::Element(child) = child {
            if child.is("txbxContent") {
                continue;
            }
            stamp_in(child, c, handles, in_cell);
        }
    }
}

/// The `kind:tag` entries stamped on a paragraph, row or cell, as (kind,
/// tag).
pub(crate) fn stamped_revs(element: &Element) -> Vec<(String, String)> {
    element
        .attr(REVS)
        .map(|revs| {
            revs.split(' ')
                .filter_map(|r| {
                    r.split_once(':')
                        .map(|(k, t)| (k.to_string(), t.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Whether `e` holds a tracked change, by the author `by` names when given:
/// a resolved `@AC` handle, or a full name.
pub(crate) fn revised_by(e: &Element, by: Option<&str>, handles: &Handles) -> bool {
    e.elements().any(|c| {
        let revision = matches!(
            c.local(),
            "ins" | "del" | "moveFrom" | "moveTo" | "cellIns" | "cellDel" | "cellMerge"
        ) || c.local().ends_with("PrChange")
            || c.local() == "tblGridChange";
        let author = c.attr("author");
        (revision
            && by.is_none_or(|by| match by.strip_prefix('@') {
                Some(handle) => handles.of(author) == Some(handle),
                None => author == Some(by),
            }))
            || revised_by(c, by, handles)
    })
}

/// The comment ids a range or a reference under `e` anchors.
pub(crate) fn comment_anchors(e: &Element, out: &mut HashSet<String>) {
    for c in e.elements() {
        if matches!(c.local(), "commentRangeStart" | "commentReference")
            && let Some(id) = c.attr("id")
        {
            out.insert(id.to_string());
        }
        comment_anchors(c, out);
    }
}

/// The children of `e` the view renders: of an `mc:AlternateContent`, only
/// the `mc:Choice` (else the `mc:Fallback`), so a text box Word stores twice
/// counts once.
fn shown_children(e: &Element) -> Vec<&Element> {
    if e.is("AlternateContent") {
        e.child("Choice")
            .or_else(|| e.child("Fallback"))
            .into_iter()
            .collect()
    } else {
        e.elements().collect()
    }
}

/// Whether `e` holds text the view shows: Word's Fallback copy of a text
/// box is not shown.
/// Whether a row's cells hold text inside changes of their own: Word's
/// shape for a row it inserts or deletes, the row's mark and each run's.
/// The row's mark then goes on the table line (`rev r80 @AL in t0.row1`)
/// and the text keeps its own tags; a row whose text has none lends it the
/// row's tag instead.
pub(crate) fn row_text_marked(tr: &Element) -> bool {
    super::row_cells(tr).any(text_marked)
}

/// Whether a cell's own text (not a nested table's) is inside changes of
/// its own: a cell's insertion or deletion then stays on the table line,
/// as a row's does.
pub(crate) fn text_marked(e: &Element) -> bool {
    e.elements().any(|c| {
        (is_revision(c.local()) && shows_text(c)) || (c.local() != "tbl" && text_marked(c))
    })
}

/// Whether `e` holds a tracked change inside a tracked change: CriticMarkup
/// cannot nest one, so a container's mark merged into the nest's outer
/// change would lose it — the text keeps its own marks instead, as the
/// agent view's marked text does.
pub(crate) fn nested_marked(e: &Element) -> bool {
    e.elements().any(|c| {
        (is_revision(c.local()) && any_revision(c)) || (c.local() != "tbl" && nested_marked(c))
    })
}

/// Whether any element below `e` is a tracked change.
fn any_revision(e: &Element) -> bool {
    e.elements()
        .any(|c| is_revision(c.local()) || any_revision(c))
}

fn shows_text(e: &Element) -> bool {
    shown_children(e).into_iter().any(|c| {
        matches!(c.local(), "t" | "delText" | "instrText" | "delInstrText") || shows_text(c)
    })
}

fn walk_revision_authors(
    e: &Element,
    out: &mut Vec<String>,
    dates: &mut HashMap<String, BTreeSet<String>>,
) {
    // Every kind `jubarte changes` lists: marks, cell merges, and
    // formatting (`*PrChange`, `numberingChange`, `tblGridChange`).
    if (is_revision(e.local()) || e.local() == "cellMerge" || e.local().ends_with("Change"))
        && let Some(author) = e.attr("author")
    {
        if !out.iter().any(|a| a == author) {
            out.push(author.to_string());
        }
        if let Some(date) = e.attr("date") {
            dates
                .entry(author.to_string())
                .or_default()
                .insert(date.to_string());
        }
    }
    // Every child, a text box's `mc:Fallback` copy too: `jubarte changes`
    // lists a change only the Fallback holds, so its author needs a handle.
    // A Fallback twin repeats the Choice's authors, already in `out`.
    for child in e.elements() {
        walk_revision_authors(child, out, dates);
    }
}

/// Authors of revisions (document order, then the other stories: footnotes,
/// endnotes, headers and footers) then of comments (comment order), then of
/// the styles' changes, each with a handle: the comment `w:initials` that author wrote, else the
/// uppercase initials of the name's words; a collision appends 2, 3, ….
pub(crate) fn handles(
    document: &Element,
    stories: &[&Element],
    comments: Option<&Element>,
    styles: Option<&Element>,
) -> Handles {
    let mut order = Vec::new();
    let mut dates: HashMap<String, BTreeSet<String>> = HashMap::new();
    walk_revision_authors(document, &mut order, &mut dates);
    for root in stories {
        walk_revision_authors(root, &mut order, &mut dates);
    }
    let mut initials: HashMap<String, String> = HashMap::new();
    if let Some(comments) = comments {
        for c in comments.children_named("comment") {
            let Some(author) = c.attr("author") else {
                continue;
            };
            if !order.iter().any(|a| a == author) {
                order.push(author.to_string());
            }
            if let Some(date) = c.attr("date") {
                dates
                    .entry(author.to_string())
                    .or_default()
                    .insert(date.to_string());
            }
            if let Some(i) = c.attr("initials").map(str::trim).filter(|i| !i.is_empty()) {
                initials
                    .entry(author.to_string())
                    .or_insert_with(|| i.to_string());
            }
        }
    }
    if let Some(styles) = styles {
        walk_revision_authors(styles, &mut order, &mut dates);
    }
    let mut by_author = HashMap::new();
    let mut taken: HashSet<String> = HashSet::new();
    // A handle is letters and digits only: it sits inside tags (`1+2@AC`,
    // `1@AC|2@JD`), space-separated stamps and `{>>…<<}` notes.
    let clean = |s: &str| -> String { s.chars().filter(|c| c.is_alphanumeric()).collect() };
    for author in &order {
        let base = initials
            .get(author)
            .map(|i| clean(i))
            .filter(|i| !i.is_empty())
            .unwrap_or_else(|| {
                let letters = clean(
                    &author
                        .split_whitespace()
                        .filter_map(|w| w.chars().find(|c| c.is_alphanumeric()))
                        .collect::<String>()
                        .to_uppercase(),
                );
                if letters.is_empty() {
                    "??".to_string()
                } else {
                    letters
                }
            });
        let mut handle = base.clone();
        let mut n = 2;
        while !taken.insert(handle.clone()) {
            handle = format!("{base}{n}");
            n += 1;
        }
        by_author.insert(author.clone(), handle);
    }
    Handles {
        by_author,
        order,
        dates,
        shown_ranges: HashSet::new(),
        shown_bookmarks: HashSet::new(),
        hidden_styles: HashSet::new(),
    }
}

/// Consecutive runs of sorted indices: `[3,4,5,9]` → `[(3,5),(9,9)]`.
pub(crate) fn runs(indices: &[usize]) -> Vec<(usize, usize)> {
    let mut out: Vec<(usize, usize)> = Vec::new();
    for &i in indices {
        match out.last_mut() {
            Some((_, end)) if *end + 1 == i => *end = i,
            _ => out.push((i, i)),
        }
    }
    out
}

/// What the id line of a paragraph says beyond its index.
pub(crate) struct LineFacts<'a> {
    pub index: usize,
    pub style: Option<&'a str>,
    pub heading: Option<usize>,
    /// The list marker the converter computed (`1.`, `(a)`, `-`).
    pub marker: Option<&'a str>,
    /// The list level Word gives the paragraph (`ilvl`): `level 1`. The
    /// view prints no list indentation, since each id line ends the
    /// Markdown list.
    pub level: usize,
    /// The paragraph's own `w:outlineLvl`, when no style carries the
    /// level (`outline 2`): the `#` count alone would give the writer's
    /// `HeadingN`. Levels 7-9, which no `#` reaches, are carried by the
    /// clause alone.
    pub outline: Option<usize>,
    pub page_break: bool,
    /// Accept-all or reject-all view: print the `rev` tags.
    pub resolved: bool,
    /// Comment ids to list (comments hidden).
    pub comments: &'a [String],
    /// Comment threads whose range runs on past this paragraph, inline or
    /// hidden: `in c5`.
    pub spans: &'a [String],
    /// The paragraph has no text but keeps its own line for its marks,
    /// revisions or comments: `<!-- p4 empty, break-ins r3 @AC -->`.
    pub empty: bool,
    /// Tracked view: the paragraph's changes its text prints no tag for,
    /// `true` for an insertion (see [`unshown_tags`]).
    pub unshown: &'a [(bool, String)],
}

/// The tags of the insertions, deletions and moves in paragraph `p` whose
/// id no note in `shown` (the paragraph's rendered text) prints: a deleted
/// object, break, field character or empty mark has no text to wrap. The
/// tracked view's id line names them (`rev r12 @AC`, a deletion `rev-del
/// r13 @AC`), so every change reads from the view. The paragraph mark's
/// own changes (`break-ins`, `break-del`) and formatting (`fmt`) have
/// their own clauses; a text box renders its own blocks, and Word's
/// Fallback copy is the Choice's twin. Each entry's `bool` says the mark
/// inserts (`ins`, `moveTo`) rather than deletes (`del`, `moveFrom`).
pub(crate) fn unshown_tags(p: &Element, shown: &str, handles: &Handles) -> Vec<(bool, String)> {
    fn walk(e: &Element, handles: &Handles, out: &mut Vec<(bool, String, String)>) {
        for child in shown_children(e) {
            match child.local() {
                "pPr" | "txbxContent" => {}
                // A formatting change's own history (`w:rPrChange/w:rPr/w:del`)
                // is no change `jubarte changes` lists.
                local if local.ends_with("PrChange") => {}
                local @ ("ins" | "del" | "moveFrom" | "moveTo") => {
                    if let Some(id) = child.attr("id") {
                        let inserted = matches!(local, "ins" | "moveTo");
                        out.push((inserted, id.to_string(), tag_of(child, handles)));
                    }
                    walk(child, handles, out);
                }
                _ => walk(child, handles, out),
            }
        }
    }
    let mut carriers = Vec::new();
    walk(p, handles, &mut carriers);
    if carriers.is_empty() {
        return Vec::new();
    }
    let printed = printed_wids(shown, "body");
    let mut unshown: Vec<(bool, String)> = Vec::new();
    for (inserted, id, tag) in carriers {
        if !printed.contains(&id) && !unshown.iter().any(|(_, t)| *t == tag) {
            unshown.push((inserted, tag));
        }
    }
    unshown
}

/// The `w:id`s of `story`'s changes the tag notes in `shown` print
/// (`{>>r12 @AC<<}`, `{>>header1.r1+2 @AC<<}`); a comment's note holds the
/// author's words and no tag.
fn printed_wids(shown: &str, story: &str) -> HashSet<String> {
    let mut printed = HashSet::new();
    for (at, _) in shown.match_indices("{>>") {
        let note = &shown[at + 3..];
        let note = &note[..note.find("<<}").unwrap_or(note.len())];
        if comment_word(note.split(' ').next().unwrap_or("")).is_some() {
            continue;
        }
        for tag in note.split("; ") {
            let token = tag.split(' ').next().unwrap_or("");
            if let Ok(ids) = crate::ids::parse_change_ids(token) {
                printed.extend(
                    ids.into_iter()
                        .filter(|id| id.story == story)
                        .map(|id| id.wid),
                );
            }
        }
    }
    printed
}

/// The ids (`r12`, `header1.r1`) of every change the view prints a tag for:
/// an id, or ids joined by `+`, followed by an `@` handle. Only where the
/// writer prints tags: the body's `{>>…<<}` notes (not a comment's, which
/// holds the author's words) and `<!-- … -->` lines, and the header without
/// its headers' and footers' `text:` values. Document text that reads like
/// a tag (`ping r5 @ops`) prints no change.
pub(crate) fn printed_change_ids(head: &str, body: &str) -> HashSet<String> {
    let mut spans: Vec<String> = Vec::new();
    tag_spans(body, true, &mut spans);
    let mut authors = false;
    for line in head.lines() {
        // The author legend, the title, the source path and the owner are
        // free text: a name or title that reads like a tag prints nothing.
        if line == "authors:" {
            authors = true;
            continue;
        }
        if authors && line.starts_with("  ") {
            continue;
        }
        authors = false;
        if line.starts_with("source: ") || line.starts_with("title: ") {
            continue;
        }
        let (rest, texts) = split_story_text(line);
        spans.extend(tag_fields(&rest));
        for text in texts {
            tag_spans(&text, false, &mut spans);
        }
    }
    let mut printed = HashSet::new();
    for span in &spans {
        let words: Vec<&str> = span
            .split(|c: char| c.is_whitespace() || ",;{}<>()[]\"".contains(c))
            .filter(|w| !w.is_empty())
            .collect();
        printed.extend(
            words
                .windows(2)
                .filter(|pair| pair[1].len() > 1 && pair[1].starts_with('@'))
                .filter_map(|pair| crate::ids::parse_change_ids(pair[0]).ok())
                .flatten()
                .map(|id| id.short()),
        );
    }
    printed
}

/// The values of a header line's tag fields: the `style_changes:` map
/// whole, else each `rev:` and `fmt:` value and a page's `; fmt` tail. A
/// quoted value runs to its closing quote, a plain one to the next `,` or
/// `}`, the page's tail to the line's end.
fn tag_fields(line: &str) -> Vec<String> {
    if let Some(map) = line.strip_prefix("style_changes: ") {
        return vec![map.to_string()];
    }
    let mut out = Vec::new();
    if let Some((_, tail)) = line.split_once("; fmt ") {
        out.push(tail.to_string());
    }
    for key in ["rev: ", "fmt: "] {
        let mut rest = line;
        while let Some(at) = rest.find(key) {
            let value = &rest[at + key.len()..];
            let end = match value.strip_prefix('"') {
                Some(quoted) => 1 + quoted.find('"').map_or(quoted.len(), |i| i + 1),
                None => value.find([',', '}']).unwrap_or(value.len()),
            };
            out.push(value[..end].to_string());
            rest = &value[end..];
        }
    }
    out
}

/// The tag notes (`{>>r5 @AC<<}`, a comment's skipped) in `text`, and with
/// `lines` its `<!-- … -->` lines' contents.
fn tag_spans(text: &str, lines: bool, out: &mut Vec<String>) {
    let pairs: &[(&str, &str)] = if lines {
        &[("{>>", "<<}"), ("<!-- ", " -->")]
    } else {
        &[("{>>", "<<}")]
    };
    for &(open, close) in pairs {
        let mut rest = text;
        while let Some(at) = rest.find(open) {
            let after = &rest[at + open.len()..];
            let end = after.find(close).unwrap_or(after.len());
            let span = &after[..end];
            let comment =
                open == "{>>" && comment_word(span.split(' ').next().unwrap_or("")).is_some();
            if !comment {
                out.push(span.to_string());
            }
            rest = &after[end..];
        }
    }
}

/// A header line without its `text: …` values (a header's or footer's own
/// words, which keep their tag notes), and those values: a plain value runs
/// to the next `,` or `}`, a quoted one to its closing quote, a list (a
/// story of several paragraphs) to its `]` outside quotes.
fn split_story_text(line: &str) -> (String, Vec<String>) {
    let mut out = String::with_capacity(line.len());
    let mut texts = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find("text: ") {
        out.push_str(&rest[..at]);
        let value = &rest[at + "text: ".len()..];
        // The end of the quoted values in `s`, scanning past them, at the
        // first `stop` outside quotes.
        let past_quotes = |s: &str, stop: char| {
            let (mut quoted, mut escaped) = (false, false);
            s.char_indices()
                .find(|&(_, c)| {
                    let done = c == stop && !quoted;
                    if quoted {
                        quoted = c != '"' || escaped;
                        escaped = c == '\\' && !escaped;
                    } else {
                        quoted = c == '"';
                    }
                    done
                })
                .map_or(s.len(), |(i, _)| i + 1)
        };
        let end = if let Some(quoted) = value.strip_prefix('"') {
            let mut escaped = false;
            let close = quoted
                .char_indices()
                .find(|&(_, c)| {
                    let done = c == '"' && !escaped;
                    escaped = c == '\\' && !escaped;
                    done
                })
                .map_or(quoted.len(), |(i, _)| i + 1);
            1 + close
        } else if let Some(list) = value.strip_prefix('[') {
            1 + past_quotes(list, ']')
        } else {
            value.find([',', '}']).unwrap_or(value.len())
        };
        texts.push(value[..end].to_string());
        rest = &value[end..];
    }
    out.push_str(rest);
    (out, texts)
}

/// A change on the header's `unplaced:` line: `del r3 @AC in p0`, the
/// kind first so the writer gives the mark the same (`ins` by omission).
pub(crate) fn unplaced_entry(change: &crate::changes::Change, handles: &Handles) -> String {
    let handle = handles.of(change.author.as_deref()).unwrap_or("??");
    let kind = match change.kind {
        crate::changes::ChangeKind::Deletion => "del ",
        crate::changes::ChangeKind::Move
            if change.move_side == Some(crate::changes::MoveSide::From) =>
        {
            "del "
        }
        crate::changes::ChangeKind::Formatting => "fmt ",
        _ => "ins ",
    };
    match change.paragraph.as_deref() {
        Some(p) => format!(
            "{kind}{} @{handle} in {}",
            change.r#ref,
            crate::ids::short_paragraph(p)
        ),
        None => format!("{kind}{} @{handle}", change.r#ref),
    }
}

/// A whole story part's changes (a header, a footer, one note) whose id
/// `shown` prints no tag for, as printed tags: insertions, deletions and
/// moves, each `true` for an insertion (`rev`/`rev-del`), then formatting
/// (`fmt`), each with its paragraph when one holds it (`in footer1.p1`).
/// The view shows a header by one paragraph and a note by its text, so
/// these name the rest.
/// A change `story_unshown` names: whether it inserts, and its tag.
pub(crate) type StoryRev = (bool, String);
/// A formatting change `story_unshown` names: its paragraph when one
/// holds it, and its tag.
pub(crate) type StoryFmt = (Option<usize>, String);

pub(crate) fn story_unshown(
    root: &Element,
    story: &str,
    shown: &str,
    skip: &HashSet<String>,
    handles: &Handles,
) -> (Vec<StoryRev>, Vec<StoryFmt>) {
    // `at` is the carrier's paragraph, counted as [`paragraphs`] does, so a
    // formatting change lands on the paragraph it is in (`in footer1.p1`).
    fn walk<'e>(
        e: &'e Element,
        at: Option<usize>,
        counted: bool,
        next: &mut usize,
        out: &mut Vec<(Option<usize>, bool, bool, &'e Element)>,
    ) {
        for child in shown_children(e) {
            let local = child.local();
            if local == "p" && counted {
                let index = *next;
                *next += 1;
                walk(child, Some(index), counted, next, out);
                continue;
            }
            if matches!(local, "ins" | "del" | "moveFrom" | "moveTo") {
                out.push((at, false, matches!(local, "ins" | "moveTo"), child));
            } else if local.ends_with("PrChange")
                || matches!(local, "tblGridChange" | "numberingChange")
            {
                out.push((at, true, true, child));
                continue;
            } else if matches!(local, "cellIns" | "cellDel" | "cellMerge") {
                out.push((at, false, local == "cellIns", child));
            }
            walk(child, at, counted && local != "txbxContent", next, out);
        }
    }
    let mut carriers = Vec::new();
    walk(root, None, true, &mut 0, &mut carriers);
    let printed = printed_wids(shown, story);
    let (mut rev, mut fmt) = (Vec::new(), Vec::new());
    for (at, formatting, inserted, carrier) in carriers {
        let Some(id) = carrier.attr("id") else {
            continue;
        };
        if printed.contains(id) || skip.contains(id) {
            continue;
        }
        let tag = format_tag(&format!("{story}#{}", tag_of(carrier, handles)));
        if formatting {
            if !fmt.iter().any(|(_, t)| *t == tag) {
                fmt.push((at, tag));
            }
        } else if !rev.iter().any(|(_, t)| *t == tag) {
            rev.push((inserted, tag));
        }
    }
    (rev, fmt)
}

/// A header's or footer's paragraph marks that are tracked: `break-del
/// footer1.r25 @BC in footer1.p1` each, and their `w:id`s.
pub(crate) fn story_paragraph_marks(
    root: &Element,
    story: &str,
    handles: &Handles,
) -> (Vec<String>, HashSet<String>) {
    let mut entries = Vec::new();
    let mut ids = HashSet::new();
    for (index, p) in paragraphs(root).into_iter().enumerate() {
        for mark in p
            .path(&["pPr", "rPr"])
            .into_iter()
            .flat_map(Element::elements)
        {
            let word = match mark.local() {
                "ins" => "break-ins",
                "del" => "break-del",
                _ => continue,
            };
            let Some(id) = mark.attr("id") else { continue };
            ids.insert(id.to_string());
            let tag = format_tag(&format!("{story}#{}", tag_of(mark, handles)));
            let at = crate::ids::short_paragraph(&format!("{story}:p:{index}"));
            entries.push(format!("{word} {tag} in {at}"));
        }
    }
    (entries, ids)
}

/// An empty paragraph that still says something the `pN empty` run would
/// lose: a tracked mark, a formatting change or, resolved, a revision.
pub(crate) fn holds_revision_facts(p: &Element, resolved: bool, handles: &Handles) -> bool {
    let (ins, del) = mark_tags(p, handles);
    !ins.is_empty()
        || !del.is_empty()
        || !format_change_tags(p, handles).is_empty()
        || (resolved && !stamped_revs(p).is_empty())
}

/// The comment ids a paragraph's ranges and references name, in document
/// order, each once.
pub(crate) fn comment_ids(p: &Element) -> Vec<String> {
    fn walk(e: &Element, out: &mut Vec<String>) {
        for child in e.elements() {
            if child.is("commentRangeStart") || child.is("commentReference") {
                if let Some(id) = child.attr("id")
                    && !out.iter().any(|c| c == id)
                {
                    out.push(id.to_string());
                }
            } else {
                walk(child, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(p, &mut out);
    out
}

/// Twips as inches with the fewest decimals (two at least, four at most)
/// that read back to the same twips: `720` → `0.5in`, `1440` → `1in`,
/// `1134` (2cm) → `0.7875in`. Four decimals are always enough: they are off
/// by 0.072 twips at most.
pub(crate) fn inches(twips: f64) -> String {
    let inches = twips / 1440.0;
    let value = (2..=4)
        .map(|places| format!("{inches:.places$}"))
        .find(|text| {
            text.parse::<f64>()
                .is_ok_and(|v| (v * 1440.0).round() == twips.round())
        })
        .unwrap_or_else(|| format!("{inches:.4}"));
    let value = value.trim_end_matches('0').trim_end_matches('.');
    format!("{value}in")
}

/// Whether a descendant of `e` outside text boxes matches `hit`: a break
/// inside a text box turns no page of the body.
fn holds(e: &Element, hit: &dyn Fn(&Element) -> bool) -> bool {
    e.elements()
        .any(|c| !c.is("txbxContent") && (hit(c) || holds(c, hit)))
}

/// How many elements under `e` (text boxes excluded) `hit` matches.
fn count(e: &Element, hit: &dyn Fn(&Element) -> bool) -> usize {
    e.elements()
        .filter(|c| !c.is("txbxContent"))
        .map(|c| usize::from(hit(c)) + count(c, hit))
        .sum()
}

/// The first page break of `p` inside an insertion or deletion: the
/// change's kind (`ins`, `del`; a move as its side) and tag.
fn tracked_page_break(p: &Element, handles: &Handles) -> Option<(&'static str, String)> {
    fn walk(
        e: &Element,
        within: Option<(&'static str, &Element)>,
        handles: &Handles,
    ) -> Option<(&'static str, String)> {
        for child in shown_children(e) {
            if child.is("pPr") || child.is("txbxContent") || child.local().ends_with("PrChange") {
                continue;
            }
            if is_page_break(child) {
                if let Some((kind, change)) = within {
                    return Some((kind, tag_of(change, handles)));
                }
                continue;
            }
            let within = kind_of(child.local()).map_or(within, |kind| Some((kind, child)));
            if let Some(found) = walk(child, within, handles) {
                return Some(found);
            }
        }
        None
    }
    walk(p, None, handles)
}

fn is_page_break(e: &Element) -> bool {
    e.is("br") && e.attr("type") == Some("page")
}

pub(crate) fn has_page_break(p: &Element) -> bool {
    holds(p, &is_page_break)
}

pub(crate) fn has_rendered_page_break(p: &Element) -> bool {
    holds(p, &|e| e.is("lastRenderedPageBreak"))
}

/// The hard page breaks in `p`: a paragraph can hold several.
pub(crate) fn page_breaks(p: &Element) -> usize {
    count(p, &is_page_break)
}

/// The cached page breaks in `p`: a paragraph that runs over three pages
/// holds two.
pub(crate) fn rendered_page_breaks(p: &Element) -> usize {
    count(p, &|e| e.is("lastRenderedPageBreak"))
}

/// Backslash-escapes the characters CommonMark would act on inside a run of
/// document text, so literal `*`, `` ` ``, `~~`, `==`, `\`, `[^` and
/// tag-like `<` read back as themselves. `_` is escaped only at a word edge
/// (`a_b` stays; `_x_` becomes `\_x\_`); `=` only beside its twin; `~`
/// and `^` beside their twin or at a word edge (`H~2~O` stays). CriticMarkup delimiters are `critic::escape`'s job, after this.
pub(crate) fn escape_markdown(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let at = |i: usize| chars.get(i).copied();
    let before = |i: usize| i.checked_sub(1).and_then(at);
    let mut out = String::with_capacity(text.len() + 8);
    for (i, &c) in chars.iter().enumerate() {
        let next = at(i + 1);
        let escape = match c {
            '\\' | '*' | '`' => true,
            '_' => {
                !(before(i).is_some_and(char::is_alphanumeric)
                    && next.is_some_and(char::is_alphanumeric))
            }
            '=' => before(i) == Some(c) || next == Some(c),
            // One `~` or `^` marks subscript or superscript at a word
            // edge; inside a word, or between spaces, it opens nothing.
            '~' | '^' if before(i) == Some('[') => false,
            '~' | '^' => {
                let word = |c: Option<char>| c.is_some_and(char::is_alphanumeric);
                let space = |c: Option<char>| c.is_none_or(char::is_whitespace);
                before(i) == Some(c)
                    || next == Some(c)
                    || !((word(before(i)) && word(next)) || (space(before(i)) && space(next)))
            }
            '<' => next.is_some_and(|n| n.is_ascii_alphabetic() || n == '/' || n == '!'),
            '[' => next == Some('^'),
            // A `]` before `(`, `[` or `:` would close a link, an image, a
            // reference or a reference definition.
            ']' => matches!(next, Some('(' | '[' | ':')),
            '&' => entity_follows(&chars[i + 1..]),
            // A literal `{/}`, or `{` before a field code and `}`, would
            // read back as a field.
            '{' => field_follows(&chars[i + 1..]),
            _ => false,
        };
        if escape {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Whether `rest` (the text after a `{`) is `/}` or a field code then `}`,
/// which the writer reads as a body field.
fn field_follows(rest: &[char]) -> bool {
    use crate::markdown::critic::{field_code_close, is_field_code};
    field_code_close(rest.iter().copied().enumerate()).is_some_and(|close| {
        let code: String = rest[..close].iter().collect();
        code == "/" || is_field_code(&code)
    })
}

/// Whether `rest` (the text after a `&`) reads as an entity or character
/// reference (`copy;`, `#35;`, `#x23;`), which would decode on the way back.
/// A name HTML does not define (`foo;`) is text to CommonMark: the parser
/// itself says which names decode.
fn entity_follows(rest: &[char]) -> bool {
    let numeric = rest.first() == Some(&'#');
    let body = if numeric { &rest[1..] } else { rest };
    let name = body
        .iter()
        .take_while(|c| c.is_ascii_alphanumeric())
        .count();
    if name == 0 || body.get(name) != Some(&';') {
        return false;
    }
    if numeric {
        return true;
    }
    let reference: String = std::iter::once('&')
        .chain(body[..=name].iter().copied())
        .collect();
    let read: String = pulldown_cmark::Parser::new(&reference)
        .filter_map(|event| match event {
            pulldown_cmark::Event::Text(text) => Some(text.into_string()),
            _ => None,
        })
        .collect();
    read != reference
}

/// A heading's text with a trailing run of `#` escaped: CommonMark reads a
/// run of `#` at the end of an ATX heading, after a space or alone, as a
/// closing sequence and drops it.
pub(crate) fn escape_closing_hashes(text: &str) -> String {
    let trimmed = text.trim_end();
    let body = trimmed.trim_end_matches('#');
    if body.len() == trimmed.len() || !(body.is_empty() || body.ends_with([' ', '\t'])) {
        return text.to_string();
    }
    format!("{body}\\{}", &text[body.len()..])
}

/// Escapes a block-opening mark at the start of one line of a paragraph:
/// `#` (one to six, then a space or the end), `>`, `|`, a list bullet (`-`
/// or `+` then a space), an ordered marker (digits then `.` or `)` then a
/// space or the end), or a line of dashes or of `=` (a setext underline or a
/// rule).
/// A leading `*` is already `\*` from `escape_markdown`.
pub(crate) fn escape_block_start(line: &str) -> String {
    let bytes = line.as_bytes();
    // A block opener may sit after up to three spaces; four open an indented
    // code block, whose text shows as written. Paragraph lines arrive trimmed
    // (`tidy_inline`), so this guards callers that keep their indent.
    let indent = bytes.iter().take_while(|&&b| b == b' ').count();
    if (1..=3).contains(&indent) {
        let (spaces, rest) = line.split_at(indent);
        return format!("{spaces}{}", escape_block_start(rest));
    }
    let hashes = bytes.iter().take_while(|&&b| b == b'#').count();
    let opens = ((1..=6).contains(&hashes) && bytes.get(hashes).is_none_or(|&b| b == b' '))
        || matches!(bytes.first(), Some(b'>' | b'|'))
        || (matches!(bytes.first(), Some(b'-' | b'+')) && bytes.get(1).is_none_or(|&b| b == b' '))
        || (bytes.first() == Some(&b'-') && line.trim_end().bytes().all(|b| b == b'-'))
        || (bytes.first() == Some(&b'=') && line.trim_end().bytes().all(|b| b == b'='));
    if opens {
        return format!("\\{line}");
    }
    let digits = bytes.iter().take_while(|b| b.is_ascii_digit()).count();
    if (1..=9).contains(&digits)
        && matches!(bytes.get(digits), Some(b'.' | b')'))
        && bytes.get(digits + 1).is_none_or(|&b| b == b' ')
    {
        return format!("{}\\{}", &line[..digits], &line[digits..]);
    }
    line.to_string()
}

/// `escape_block_start` on every line: after a hard break an ATX heading or
/// a list item can interrupt a paragraph.
pub(crate) fn escape_line_starts(text: &str) -> String {
    text.split('\n')
        .map(escape_block_start)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The tables under `e` in document order, text boxes excluded.
pub(crate) fn tables(e: &Element) -> Vec<&Element> {
    fn walk<'a>(e: &'a Element, out: &mut Vec<&'a Element>) {
        for c in e.elements().filter(|c| !c.is("txbxContent")) {
            if c.is("tbl") {
                out.push(c);
            }
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    walk(e, &mut out);
    out
}

/// The paragraphs under `e` in document order, text boxes excluded.
pub(crate) fn paragraphs(e: &Element) -> Vec<&Element> {
    fn walk<'a>(e: &'a Element, out: &mut Vec<&'a Element>) {
        for c in e.elements() {
            if c.is("p") {
                out.push(c);
            }
            if !c.is("txbxContent") {
                walk(c, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(e, &mut out);
    out
}

/// Indices of the paragraphs whose section break starts a new page: the
/// section after them is not `continuous` or `nextColumn` (a section's
/// `w:type` says how that section starts).
pub(crate) fn page_sections(body: &Element) -> HashSet<usize> {
    let ends: Vec<(usize, &Element)> = paragraphs(body)
        .into_iter()
        .filter_map(|p| Some((p.attr(INDEX)?.parse().ok()?, p.path(&["pPr", "sectPr"])?)))
        .collect();
    let last = body.child("sectPr");
    ends.iter()
        .enumerate()
        .filter(|(k, _)| {
            let next = ends.get(k + 1).map(|(_, s)| *s).or(last);
            !next
                .and_then(|s| s.path(&["type"]))
                .and_then(|t| t.attr("val"))
                .is_some_and(|v| v == "continuous" || v == "nextColumn")
        })
        .map(|(_, (index, _))| *index)
        .collect()
}

/// A table's cached page breaks: whether its first paragraph opens a page,
/// and how many more pages its other paragraphs turn (named at the next
/// block, since a marker cannot go inside a table).
pub(crate) fn table_breaks(tbl: &Element, cached: bool) -> (bool, usize) {
    let ps = paragraphs(tbl);
    if cached {
        let first = ps.first().is_some_and(|p| has_rendered_page_break(p));
        let all: usize = ps.iter().map(|p| rendered_page_breaks(p)).sum();
        (first, all - usize::from(first))
    } else {
        (false, ps.iter().map(|p| page_breaks(p)).sum())
    }
}

pub(crate) fn has_section_break(p: &Element) -> bool {
    p.path(&["pPr", "sectPr"]).is_some()
}

/// `<!-- head -->` or `<!-- head clause, clause -->`.
pub(crate) fn line(head: &str, clauses: &[String]) -> String {
    if clauses.is_empty() {
        format!("<!-- {head} -->")
    } else {
        format!("<!-- {head} {} -->", clauses.join(", "))
    }
}

/// `<!-- p3 justify, first-line 0.5in, comments c5 -->` (the plan's grammar).
/// A paragraph that states a style names it, the default too: the statement
/// is the document's own, not the view's to drop (#528). A heading's own
/// `HeadingN` with text needs no name, the `#` count says it; an empty
/// paragraph has no `#`, so even that one is named.
pub(crate) fn id_line(p: &Element, f: &LineFacts, handles: &Handles) -> String {
    let mut head = format!("p{}", f.index);
    if let Some(style) = f.style {
        let implied = !f.empty
            && f.heading
                .is_some_and(|level| style == format!("Heading{level}"));
        if !implied {
            head.push(' ');
            head.push_str(&crate::markdown::view_notes::style_token(style));
        }
    }
    let ppr = p.child("pPr");
    let mut clauses: Vec<String> = Vec::new();
    if let Some(outline) = f.outline {
        clauses.push(format!("outline {outline}"));
    }
    if f.empty {
        clauses.push("empty".to_string());
    }
    if let Some(jc) = ppr
        .and_then(|pr| pr.child("jc"))
        .and_then(|j| j.attr("val"))
    {
        clauses.push(
            match jc {
                "center" => "center",
                "right" | "end" => "right",
                "both" | "distribute" => "justify",
                _ => "left",
            }
            .to_string(),
        );
    }
    if let Some(ind) = ppr.and_then(|pr| pr.child("ind")) {
        // One clause per side: the strict `start`/`end` only when the
        // transitional `left`/`right` is absent, as the layout reads them.
        for (attrs, name) in [
            (&["firstLine"][..], "first-line"),
            (&["hanging"][..], "hanging"),
            (&["left", "start"][..], "left"),
            (&["right", "end"][..], "right"),
        ] {
            if let Some(v) = attrs.iter().find_map(|attr| ind.attr(attr)).and_then(|v| {
                v.parse::<f64>()
                    .ok()
                    .or_else(|| crate::util::universal_measure_twips(v))
            }) {
                clauses.push(format!("{name} {}", inches(v)));
            }
        }
    }
    match f.marker {
        Some("-") => clauses.push("bullet".to_string()),
        // `\` and `"` escaped, so the quoted label always parses.
        Some(label) => clauses.push(format!(
            "num \"{}\"",
            label.replace('\\', "\\\\").replace('"', "\\\"")
        )),
        None => {}
    }
    if f.marker.is_some() && f.level > 0 {
        clauses.push(format!("level {}", f.level));
    }
    // A page break in a tracked change names it: `rev` would say only
    // that some change shows no text, not that it is the break's.
    let tracked_break = (!f.resolved && f.page_break)
        .then(|| tracked_page_break(p, handles))
        .flatten();
    match &tracked_break {
        Some((kind, tag)) => clauses.push(format!("page-break-{kind} {}", format_tag(tag))),
        None if f.page_break => clauses.push("page-break".to_string()),
        None => {}
    }
    if has_section_break(p) {
        clauses.push("section-break".to_string());
    }
    let (ins, del) = mark_tags(p, handles);
    if !ins.is_empty() {
        clauses.push(format!("break-ins {}", format_tags(&ins)));
    }
    if !del.is_empty() {
        clauses.push(format!("break-del {}", format_tags(&del)));
    }
    let fmt = format_change_tags(p, handles);
    if !fmt.is_empty() {
        clauses.push(format!("fmt {}", format_tags(&fmt)));
    }
    let unshown: Vec<(bool, String)> = f
        .unshown
        .iter()
        .filter(|(_, tag)| tracked_break.as_ref().is_none_or(|(_, b)| b != tag))
        .cloned()
        .collect();
    if !f.resolved {
        for inserted in [true, false] {
            let tags: Vec<String> = unshown
                .iter()
                .filter(|(ins, _)| *ins == inserted)
                .map(|(_, tag)| tag.clone())
                .collect();
            if !tags.is_empty() {
                let word = if inserted { "rev" } else { "rev-del" };
                clauses.push(format!("{word} {}", format_tags(&tags)));
            }
        }
    }
    if f.resolved {
        let tags: Vec<String> = stamped_revs(p).into_iter().map(|(_, tag)| tag).collect();
        if !tags.is_empty() {
            clauses.push(format!("rev {}", format_tags(&tags)));
        }
    }
    if !f.comments.is_empty() {
        let ids: Vec<String> = f.comments.iter().map(|c| format!("c{c}")).collect();
        clauses.push(format!("comments {}", ids.join(" ")));
    }
    if !f.spans.is_empty() {
        let ids: Vec<String> = f.spans.iter().map(|c| format!("c{c}")).collect();
        clauses.push(format!("in {}", ids.join(" ")));
    }
    line(&head, &clauses)
}

/// `<!-- p19 empty -->` / `<!-- p19-p23 empty -->` lines for pending empties.
pub(crate) fn empty_lines(indices: &[usize]) -> Vec<String> {
    runs(indices)
        .into_iter()
        .map(|(a, b)| {
            if a == b {
                format!("<!-- p{a} empty -->")
            } else {
                format!("<!-- p{a}-p{b} empty -->")
            }
        })
        .collect()
}

/// The page marker `paginate` writes, for the cached-break fallback.
pub(crate) fn page_marker(page: usize, total: usize) -> String {
    format!("<!-- page {page} of {total} -->")
}

/// Paragraphs (text boxes excluded) holding `w:lastRenderedPageBreak`, and
/// hard page breaks plus section breaks that start a page, for the
/// cached-break page count.
pub(crate) fn page_counts(body: &Element) -> (usize, usize) {
    let ps = paragraphs(body);
    let rendered = ps.iter().map(|p| rendered_page_breaks(p)).sum();
    let hard = ps.iter().map(|p| page_breaks(p)).sum::<usize>() + page_sections(body).len();
    (rendered, hard)
}

/// The table line's merged cells, row by row: `t0.row0.cell1 across 2`
/// for a cell spanning two grid columns, `t0.row0.cell0 down 3` for a
/// vertical merge starting there and covering three rows (Word's
/// `gridSpan` and `vMerge`). `grid` holds each row's cells as grid column,
/// columns spanned and vertical merge.
fn merged_cells(t: &str, grid: &[Vec<(usize, usize, Option<bool>)>]) -> Vec<String> {
    let mut out = Vec::new();
    for (r, cells) in grid.iter().enumerate() {
        for (c, &(column, span, vertical)) in cells.iter().enumerate() {
            let mut said = Vec::new();
            if span > 1 {
                said.push(format!("across {span}"));
            }
            if vertical == Some(true) {
                let down = 1 + grid[r + 1..]
                    .iter()
                    .take_while(|row| {
                        row.iter()
                            .any(|&(at, _, v)| at == column && v == Some(false))
                    })
                    .count();
                if down > 1 {
                    said.push(format!("down {down}"));
                }
            }
            if !said.is_empty() {
                out.push(format!("t{t}.row{r}.cell{c} {}", said.join(" ")));
            }
        }
    }
    out
}

/// The sides `props` (`w:tblBorders`, `w:tcBorders`, `w:pBdr`) names among
/// `all`, in schema order.
pub(super) fn own_borders(
    props: Option<&Element>,
    all: &[&'static str],
) -> crate::markdown::borders::Sides {
    use crate::markdown::borders;
    let found: Vec<_> = props
        .into_iter()
        .flat_map(|e| e.elements())
        .filter_map(|side| {
            let name = borders::side(side.local(), all)?;
            let border = borders::Border::new(
                side.attr("val")?,
                side.attr("sz"),
                side.attr("color"),
                side.attr("space"),
            )
            .with_theme(
                side.attr("themeColor"),
                side.attr("themeTint"),
                side.attr("themeShade"),
            );
            Some((name, border))
        })
        .collect();
    borders::ordered(&found, all)
}

/// The borders `tbl` and its cells carry themselves, as its line prints them
/// (`borders single 0.5pt 000000`, `cell-borders bottom single 1pt in
/// t0.row0`): what Word passes comes back as it was.
fn border_clauses(tbl: &Element, t: &str, rows: &[&Element]) -> Vec<String> {
    use crate::markdown::borders::{self, CELL_SIDES, TABLE_SIDES};
    let read = own_borders;
    let mut clauses = Vec::new();
    let own = read(tbl.path(&["tblPr", "tblBorders"]), &TABLE_SIDES);
    if !own.is_empty() {
        clauses.push(format!("borders {}", borders::describe(&own, &TABLE_SIDES)));
    }
    // A row's own table borders (`w:tblPrEx/w:tblBorders`), the rows that
    // share one in one group: `row-borders top double 0.5pt in t0.row1`.
    let mut row_groups: Vec<(String, Vec<String>)> = Vec::new();
    for (r, tr) in rows.iter().enumerate() {
        let sides = read(tr.path(&["tblPrEx", "tblBorders"]), &TABLE_SIDES);
        if sides.is_empty() {
            continue;
        }
        let text = borders::describe(&sides, &TABLE_SIDES);
        let at = format!("t{t}.row{r}");
        match row_groups.iter_mut().find(|(g, _)| *g == text) {
            Some((_, rows)) => rows.push(at),
            None => row_groups.push((text, vec![at])),
        }
    }
    if !row_groups.is_empty() {
        let groups: Vec<String> = row_groups
            .iter()
            .map(|(text, at)| format!("{text} in {}", at.join(" ")))
            .collect();
        clauses.push(format!("row-borders {}", groups.join("; ")));
    }
    // Each cell's borders, as text, by row; the cells that share one print
    // as one group, a whole row as the row, every cell as `all`.
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|tr| {
            super::row_cells(tr)
                .map(|tc| {
                    let sides = read(tc.path(&["tcPr", "tcBorders"]), &CELL_SIDES);
                    borders::describe(&sides, &CELL_SIDES)
                })
                .collect()
        })
        .collect();
    let mut texts: Vec<&String> = Vec::new();
    for text in cells.iter().flatten().filter(|t| !t.is_empty()) {
        if !texts.contains(&text) {
            texts.push(text);
        }
    }
    let groups: Vec<String> = texts
        .iter()
        .map(|text| {
            if cells.iter().flatten().all(|c| c == *text) {
                return format!("{text} in all");
            }
            let mut at = Vec::new();
            for (r, row) in cells.iter().enumerate() {
                if !row.is_empty() && row.iter().all(|c| c == *text) {
                    at.push(format!("t{t}.row{r}"));
                    continue;
                }
                for (c, cell) in row.iter().enumerate() {
                    if cell == *text {
                        at.push(format!("t{t}.row{r}.cell{c}"));
                    }
                }
            }
            format!("{text} in {}", at.join(" "))
        })
        .collect();
    if !groups.is_empty() {
        clauses.push(format!("cell-borders {}", groups.join("; ")));
    }
    clauses
}

/// `<!-- t0 center 3x3, cells p8-p16 by row, p9 center, header row repeats -->`
/// (`aligned`: the cell paragraphs aligned against their column);
/// `None` for a nested table (not numbered). With `comments` (comments
/// hidden) the cells' comment ids print as `comments c9 in p3`; `spans`
/// are the comment threads whose range runs through the table (`in c5`).
pub(crate) fn table_line(
    tbl: &Element,
    style: Option<&str>,
    aligned: &[String],
    resolved: bool,
    comments: Option<&HashMap<String, Element>>,
    spans: &[String],
    handles: &Handles,
) -> Option<String> {
    let t = tbl.attr(TABLE)?;
    let rows: Vec<&Element> = super::table_rows(tbl);
    let cols = tbl
        .child("tblGrid")
        .map(|g| g.children_named("gridCol").count())
        .filter(|&c| c > 0)
        .unwrap_or_else(|| {
            rows.iter()
                .map(|r| super::row_cells(r).count())
                .max()
                .unwrap_or(0)
        });
    let mut head = format!("t{t}");
    if let Some(jc) = tbl.path(&["tblPr", "jc"]).and_then(|j| j.attr("val")) {
        match jc {
            "center" => head.push_str(" center"),
            "right" | "end" => head.push_str(" right"),
            _ => {}
        }
    }
    // The size opens the clauses: `t0 center 3x3, cells …`.
    let mut clauses: Vec<String> = vec![format!("{}x{}", rows.len(), cols)];
    let mut per_row: Vec<(usize, usize)> = Vec::new();
    let mut uniform = true;
    // Each row's cells: grid column, columns spanned, and vertical merge
    // (`Some(true)` where one starts, `Some(false)` where one continues).
    let mut grid: Vec<Vec<(usize, usize, Option<bool>)>> = Vec::new();
    let mut break_ins: Vec<String> = Vec::new();
    let mut break_del: Vec<String> = Vec::new();
    let mut revs: Vec<String> = Vec::new();
    // A row's or cell's own deletion and a cell's merge: `rev` alone is an
    // insertion.
    let mut rev_del: Vec<String> = Vec::new();
    let mut rev_merge: Vec<String> = Vec::new();
    let mut fmt: Vec<String> = Vec::new();
    let mut held: Vec<String> = Vec::new();
    let mut held_ids: Vec<String> = Vec::new();
    let mut gaps: Vec<String> = Vec::new();
    // The tracked view names the table's own property changes (Word's
    // `tblPrChange`, `tblGridChange`, `trPrChange`, `tblPrExChange`,
    // `tcPrChange`) and its cell insertions, deletions and merges, where
    // they sit: the cells' text shows none of them.
    let changes_in = |props: Option<&Element>, at: &str, out: &mut Vec<String>| {
        for change in props
            .into_iter()
            .flat_map(|e| e.elements())
            .filter(|c| c.local().ends_with("Change"))
        {
            out.push(format!("{}{at}", format_tag(&tag_of(change, handles))));
        }
    };
    if !resolved {
        changes_in(tbl.child("tblPr"), "", &mut fmt);
        changes_in(tbl.child("tblGrid"), "", &mut fmt);
    }
    for (r, tr) in rows.iter().enumerate() {
        let mut range: Option<(usize, usize)> = None;
        if resolved {
            revs.extend(
                stamped_revs(tr)
                    .into_iter()
                    .map(|(_, tag)| format!("{} in t{t}.row{r}", format_tag(&tag))),
            );
        } else {
            let at = format!(" in t{t}.row{r}");
            changes_in(tr.child("trPr"), &at, &mut fmt);
            changes_in(tr.child("tblPrEx"), &at, &mut fmt);
            // A row's insertion or deletion tags its cells' text; a row
            // with no text, or whose text has marks of its own, names it
            // here.
            if !shows_text(tr) || row_text_marked(tr) {
                for mark in tr
                    .child("trPr")
                    .into_iter()
                    .flat_map(|e| e.elements())
                    .filter(|e| matches!(e.local(), "ins" | "del"))
                {
                    let tag = format!("{}{at}", format_tag(&tag_of(mark, handles)));
                    if mark.local() == "del" {
                        rev_del.push(tag);
                    } else {
                        revs.push(tag);
                    }
                }
            }
        }
        let grid_skip = |name: &str| {
            tr.path(&["trPr", name])
                .and_then(|g| g.attr("val"))
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0)
        };
        let mut column = grid_skip("gridBefore");
        // Grid columns a row leaves out before or after its cells: the
        // view prints them as empty cells, which are no cells. A row whose
        // cells stop short of the grid without `gridAfter` leaves the rest
        // out the same way.
        let filled: usize = super::row_cells(tr)
            .map(|tc| {
                tc.path(&["tcPr", "gridSpan"])
                    .and_then(|g| g.attr("val"))
                    .and_then(|v| v.parse::<usize>().ok())
                    .unwrap_or(1)
                    .clamp(1, 64)
            })
            .sum();
        let after = grid_skip("gridAfter");
        // A row whose cells and gaps take more grid columns than the grid
        // has, with a cell that prints nothing, holds a cell the view
        // cannot tell from its padding: its paragraph count, per row,
        // brings it back. Cells that all print text place themselves.
        if column + filled + after > cols && super::row_cells(tr).any(|tc| !shows_text(tc)) {
            uniform = false;
        }
        let after = after.max(cols.saturating_sub(column + filled));
        let mut gap = Vec::new();
        if column > 0 {
            gap.push(format!("before {column}"));
        }
        if after > 0 {
            gap.push(format!("after {after}"));
        }
        if !gap.is_empty() {
            gaps.push(format!("t{t}.row{r} {}", gap.join(" ")));
        }
        let mut cells = Vec::new();
        for (c, tc) in super::row_cells(tr).enumerate() {
            let span = tc
                .path(&["tcPr", "gridSpan"])
                .and_then(|g| g.attr("val"))
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(1)
                .clamp(1, 64);
            let vertical = tc
                .path(&["tcPr", "vMerge"])
                .map(|v| v.attr("val") == Some("restart"));
            cells.push((column, span, vertical));
            column += span;
            if resolved {
                revs.extend(
                    stamped_revs(tc)
                        .into_iter()
                        .map(|(_, tag)| format!("{} in t{t}.row{r}.cell{c}", format_tag(&tag))),
                );
            } else {
                let at = format!(" in t{t}.row{r}.cell{c}");
                let props = tc.child("tcPr");
                changes_in(props, &at, &mut fmt);
                // Word's own cell mark is `w:ins`/`w:del` in `w:tcPr` as
                // often as `w:cellIns`/`w:cellDel` (both read); a merge is
                // `w:cellMerge`.
                for cell_change in props.into_iter().flat_map(|e| e.elements()).filter(|e| {
                    matches!(
                        e.local(),
                        "ins" | "del" | "cellIns" | "cellDel" | "cellMerge"
                    )
                }) {
                    let tag = format!("{}{at}", format_tag(&tag_of(cell_change, handles)));
                    match cell_change.local() {
                        "del" | "cellDel" => rev_del.push(tag),
                        "cellMerge" => rev_merge.push(tag),
                        _ => revs.push(tag),
                    }
                }
            }
            let mut ps = Vec::new();
            tc.find_all("p", &mut ps);
            let idx: Vec<usize> = ps
                .iter()
                .filter_map(|p| p.attr(INDEX)?.parse().ok())
                .collect();
            if idx.len() != 1 {
                uniform = false;
            }
            for p in &ps {
                let Some(i) = p.attr(INDEX) else { continue };
                let (ins, del) = mark_tags(p, handles);
                break_ins.extend(ins.iter().map(|t| format!("{} in p{i}", format_tag(t))));
                break_del.extend(del.iter().map(|t| format!("{} in p{i}", format_tag(t))));
                if resolved {
                    revs.extend(
                        stamped_revs(p)
                            .into_iter()
                            .map(|(_, tag)| format!("{} in p{i}", format_tag(&tag))),
                    );
                } else {
                    // A cell's paragraph has no id line to carry its own
                    // formatting changes.
                    fmt.extend(
                        format_change_tags(p, handles)
                            .iter()
                            .map(|t| format!("{} in p{i}", format_tag(t))),
                    );
                }
                let ids: Vec<String> = comments.map_or_else(Vec::new, |known| {
                    comment_ids(p)
                        .into_iter()
                        .filter(|id| known.contains_key(id))
                        .collect()
                });
                if !ids.is_empty() {
                    held_ids.extend(ids.iter().cloned());
                    let ids: Vec<String> = ids.iter().map(|c| format!("c{c}")).collect();
                    held.push(format!("{} in p{i}", ids.join(" ")));
                }
            }
            if let (Some(&a), Some(&b)) = (idx.first(), idx.last()) {
                range = Some(match range {
                    None => (a, b),
                    Some((start, _)) => (start, b),
                });
            }
        }
        if let Some(r) = range {
            per_row.push(r);
        }
        grid.push(cells);
    }
    if uniform {
        if let (Some(&(a, _)), Some(&(_, b))) = (per_row.first(), per_row.last()) {
            clauses.push(format!("cells p{a}-p{b} by row"));
        }
    } else {
        let rows: Vec<String> = per_row
            .iter()
            .enumerate()
            .map(|(i, (a, b))| {
                if a == b {
                    format!("t{t}.row{i} p{a}")
                } else {
                    format!("t{t}.row{i} p{a}-p{b}")
                }
            })
            .collect();
        clauses.push(format!("cells {}", rows.join(" ")));
    }
    clauses.extend(aligned.iter().cloned());
    if rows
        .first()
        .is_some_and(|tr| tr.path(&["trPr", "tblHeader"]).is_some())
    {
        clauses.push("header row repeats".to_string());
    }
    if let Some(style) = style {
        clauses.push(format!(
            "style {}",
            crate::markdown::view_notes::style_token(style)
        ));
    }
    clauses.extend(border_clauses(tbl, t, &rows));
    let merged = merged_cells(t, &grid);
    if !merged.is_empty() {
        clauses.push(format!("merged {}", merged.join("; ")));
    }
    if !gaps.is_empty() {
        clauses.push(format!("gaps {}", gaps.join("; ")));
    }
    if !break_ins.is_empty() {
        clauses.push(format!("break-ins {}", break_ins.join("; ")));
    }
    if !break_del.is_empty() {
        clauses.push(format!("break-del {}", break_del.join("; ")));
    }
    if !revs.is_empty() {
        clauses.push(format!("rev {}", revs.join("; ")));
    }
    if !rev_del.is_empty() {
        clauses.push(format!("rev-del {}", rev_del.join("; ")));
    }
    if !rev_merge.is_empty() {
        clauses.push(format!("rev-merge {}", rev_merge.join("; ")));
    }
    if !fmt.is_empty() {
        clauses.push(format!("fmt {}", fmt.join("; ")));
    }
    if !held.is_empty() {
        clauses.push(format!("comments {}", held.join("; ")));
    }
    // A comment a cell holds is named once, where it is held.
    let ids: Vec<String> = spans
        .iter()
        .filter(|c| !held_ids.contains(c))
        .map(|c| format!("c{c}"))
        .collect();
    if !ids.is_empty() {
        clauses.push(format!("in {}", ids.join(" ")));
    }
    Some(line(&head, &clauses))
}

/// Comment threads from `commentsExtended.xml`.
#[derive(Debug, Default, Clone)]
pub(crate) struct Threads {
    /// Reply id → root id (`w15:paraIdParent`; Word threads are flat).
    pub reply_of: HashMap<String, String>,
    /// Ids marked `w15:done="1"`.
    pub done: HashSet<String>,
}

/// `commentEx` names a comment by the `w14:paraId` of its last paragraph.
pub(crate) fn threads(comments: Option<&Element>, extended: Option<&Element>) -> Threads {
    let mut t = Threads::default();
    let (Some(comments), Some(extended)) = (comments, extended) else {
        return t;
    };
    let by_para: HashMap<String, String> = comments
        .children_named("comment")
        .filter_map(|c| {
            let id = c.attr("id")?;
            let para = c.children_named("p").last()?.attr("paraId")?;
            Some((para.to_string(), id.to_string()))
        })
        .collect();
    for ex in extended.children_named("commentEx") {
        let Some(id) = ex.attr("paraId").and_then(|p| by_para.get(p)) else {
            continue;
        };
        if ex.attr("done").is_some_and(|d| d == "1" || d == "true") {
            t.done.insert(id.clone());
        }
        if let Some(parent) = ex.attr("paraIdParent").and_then(|p| by_para.get(p)) {
            t.reply_of.insert(id.clone(), parent.clone());
        }
    }
    t
}

/// `c5 @AC: `, `c5 @AC resolved: `, `c6 @AS re c5: `, or `c5: ` with
/// no author. `date` is the inline timestamp, when wanted.
pub(crate) fn comment_head(
    id: &str,
    author: Option<&str>,
    date: Option<&str>,
    handles: &Handles,
    threads: &Threads,
) -> String {
    let mut head = format!("c{id}");
    if let Some(handle) = handles.of(author) {
        head.push_str(&format!(" @{handle}"));
    }
    if let Some(date) = date {
        head.push(' ');
        head.push_str(date);
    }
    if let Some(root) = threads.reply_of.get(id) {
        head.push_str(&format!(" re c{root}"));
    } else if threads.done.contains(id) {
        head.push_str(" resolved");
    }
    head.push_str(": ");
    head
}

/// Every logical revision in the stamped body: inline tags per paragraph,
/// tracked paragraph marks, tracked rows and cells.
pub(crate) fn collect_revisions(body: &Element, handles: &Handles) -> Vec<RevTag> {
    let mut out = Vec::new();
    collect_in(body, handles, &mut out);
    out
}

fn collect_in(e: &Element, handles: &Handles, out: &mut Vec<RevTag>) {
    let attribution = |m: &Element| {
        (
            m.attr("author").map(str::to_string),
            m.attr("date").map(str::to_string),
        )
    };
    if e.is("p") {
        out.extend(revision_tags(e, handles));
        if let Some(rpr) = e.path(&["pPr", "rPr"]) {
            for m in rpr.elements().filter(|m| kind_of(m.local()).is_some()) {
                let (author, date) = attribution(m);
                out.push(RevTag {
                    kind: "mark",
                    tag: tag_of(m, handles),
                    author,
                    date,
                });
            }
        }
    }
    if e.is("tr")
        && let Some(trpr) = e.child("trPr")
    {
        for m in trpr.elements().filter(|m| kind_of(m.local()).is_some()) {
            let (author, date) = attribution(m);
            out.push(RevTag {
                kind: "row",
                tag: tag_of(m, handles),
                author,
                date,
            });
        }
    }
    if e.is("tc")
        && let Some(tcpr) = e.child("tcPr")
    {
        for m in tcpr
            .elements()
            .filter(|m| matches!(m.local(), "cellIns" | "cellDel"))
        {
            let (author, date) = attribution(m);
            out.push(RevTag {
                kind: "cell",
                tag: tag_of(m, handles),
                author,
                date,
            });
        }
    }
    for child in shown_children(e) {
        collect_in(child, handles, out);
    }
}

/// One block of the rendered body: its lines, the paragraph span it covers
/// and, for a table, its number.
struct Block {
    text: String,
    span: Option<(usize, usize)>,
    table: Option<usize>,
    /// The `<!-- page N of M -->` line that preceded it, if any.
    page: Option<String>,
}

/// Every `pN` in a table or empty-run line, for its span.
fn numbers_after(line: &str, key: &str) -> Vec<usize> {
    let Some(at) = line.find(key) else {
        return Vec::new();
    };
    line[at + key.len()..]
        .split(|c: char| !c.is_ascii_digit() && c != 'p')
        .filter_map(|piece| piece.strip_prefix('p')?.parse().ok())
        .collect()
}

/// Splits a rendered body on blank lines into blocks keyed by their id
/// lines; a page marker attaches to the block after it; a block with no id
/// line (notes, footnote definitions) attaches to the block before it.
fn blocks_of(body: &str) -> Vec<Block> {
    let mut blocks: Vec<Block> = Vec::new();
    let mut page: Option<String> = None;
    for chunk in body.split("\n\n").filter(|c| !c.trim().is_empty()) {
        // A nested list item's id line, and a page marker between nested
        // items, are indented with the item.
        let first = chunk.lines().next().unwrap_or("").trim_start();
        if first.starts_with("<!-- page ") {
            page = Some(first.to_string());
            continue;
        }
        let head = first.strip_prefix("<!-- ").unwrap_or("");
        if let Some(rest) = head.strip_prefix('p') {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(start) = digits.parse::<usize>() {
                let end = rest
                    .strip_prefix(&digits)
                    .and_then(|r| r.strip_prefix("-p"))
                    .map(|r| {
                        r.chars()
                            .take_while(char::is_ascii_digit)
                            .collect::<String>()
                    })
                    .and_then(|d| d.parse::<usize>().ok())
                    .unwrap_or(start);
                blocks.push(Block {
                    text: chunk.to_string(),
                    span: Some((start, end)),
                    table: None,
                    page: page.take(),
                });
                continue;
            }
        }
        if let Some(rest) = head.strip_prefix('t') {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            if let Ok(table) = digits.parse::<usize>() {
                let cells = numbers_after(first, "cells ");
                let span = match (cells.iter().min(), cells.iter().max()) {
                    (Some(&a), Some(&b)) => Some((a, b)),
                    _ => None,
                };
                blocks.push(Block {
                    text: chunk.to_string(),
                    span,
                    table: Some(table),
                    page: page.take(),
                });
                continue;
            }
        }
        match blocks.last_mut() {
            Some(last) => {
                last.text.push_str("\n\n");
                last.text.push_str(chunk);
            }
            None => blocks.push(Block {
                text: chunk.to_string(),
                span: None,
                table: None,
                page: page.take(),
            }),
        }
    }
    blocks
}

fn span_text(a: usize, b: usize) -> String {
    if a == b {
        format!("p{a}")
    } else {
        format!("p{a}-p{b}")
    }
}

/// `@HH` in `text` where the handle ends (the next char is not
/// alphanumeric).
fn names_handle(text: &str, handle: &str) -> bool {
    let key = format!("@{handle}");
    text.match_indices(&key).any(|(at, _)| {
        text[at + key.len()..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric())
    })
}

/// The id line a block opens with (`<!-- p3 … -->`, `<!-- t0 … -->`), or
/// `""` for a block that opens with document text. A nested list item's
/// id line is indented with its item.
fn id_line_of(block: &str) -> &str {
    block
        .lines()
        .next()
        .map(str::trim_start)
        .filter(|line| line.starts_with("<!-- "))
        .unwrap_or("")
}

/// The number a comment word names: `c5`, or the 0.12 spelling `#c5`.
fn comment_word(word: &str) -> Option<&str> {
    let id = word.strip_prefix('#').unwrap_or(word).strip_prefix('c')?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
}

/// Whether a block holds a mark of `handle`: on its id or table line, in the
/// tag of a note (`{>>r4 @AC<<}`, `{>>c6 @AS re c5: …<<}`; the comment text
/// after the colon is the author's words, not a tag), or as the author of a
/// hidden comment its line names. Document text never counts: the view
/// escapes any `{>>` in it.
fn has_handle(block: &str, handle: &str, comment_handles: &HashMap<String, String>) -> bool {
    let first = id_line_of(block);
    if names_handle(first, handle) {
        return true;
    }
    let hidden = first
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .filter_map(comment_word)
        .any(|id| comment_handles.get(id).is_some_and(|h| h == handle));
    hidden
        || block.match_indices("{>>").any(|(at, _)| {
            let note = &block[at + 3..];
            let end = note.find("<<}").unwrap_or(note.len());
            let tag = note[..end].split(':').next().unwrap_or("");
            names_handle(tag, handle)
        })
}

/// Whether a block of the agent view carries a tracked change or a comment:
/// a note in its text (the view escapes any `{>>` in document text), or a
/// mark clause on its id or table line (`in c5` for a block inside a
/// comment range whose note is elsewhere).
fn is_marked(block: &str) -> bool {
    block.contains("{>>") || has_mark_clause(id_line_of(block))
}

/// Whether an id or table line holds a mark clause: `rev` (`rev-del`,
/// `rev-merge`), `fmt`, `break-ins` or `break-del` followed by a tag (`r3 @AC`), or `comments`
/// or `in` followed by a comment id (`c5`). A style or a list label that
/// reads `rev` is not one.
fn has_mark_clause(line: &str) -> bool {
    let words: Vec<&str> = line.split_whitespace().collect();
    words.windows(3).any(|w| {
        matches!(
            w[0],
            "rev" | "rev-del" | "rev-merge" | "fmt" | "break-ins" | "break-del"
        ) && w[2].starts_with('@')
    }) || words.windows(2).any(|w| {
        matches!(w[0], "comments" | "in")
            && comment_word(w[1].trim_end_matches([',', ';'])).is_some()
    })
}

/// The selected blocks of `body` joined back, the `range:` text, and the
/// body paragraphs the kept blocks span (`pN` numbers, ascending). A
/// `Select::Changed` author arrives resolved: `@AC` for a known handle, the
/// text as given otherwise (no block holds an unknown author's marks).
/// `notes` (the `[^1]: …` definitions) keeps those the kept blocks cite,
/// and under `Select::Changed` those holding a selected mark, named in the
/// range by their label.
pub(crate) fn select_blocks(
    body: &str,
    notes: &mut Vec<String>,
    select: &Select,
    last: Option<usize>,
    comment_handles: &HashMap<String, String>,
) -> Result<(String, String, Vec<usize>), String> {
    let blocks = blocks_of(body);
    let keep: Vec<bool>;
    let mut changed_notes: Vec<String> = Vec::new();
    let range: String = match select {
        Select::Head(n) | Select::Tail(n) if *n == 0 => {
            return Err(format!(
                "{} needs a count above 0",
                if matches!(select, Select::Head(_)) {
                    "head"
                } else {
                    "tail"
                }
            ));
        }
        Select::Head(n) => {
            keep = (0..blocks.len()).map(|i| i < *n).collect();
            let spans: Vec<(usize, usize)> =
                blocks.iter().take(*n).filter_map(|b| b.span).collect();
            format!(
                "head {n} ({})",
                span_text(
                    spans.first().map_or(0, |s| s.0),
                    spans.last().map_or(0, |s| s.1)
                )
            )
        }
        Select::Tail(n) => {
            let skip = blocks.len().saturating_sub(*n);
            keep = (0..blocks.len()).map(|i| i >= skip).collect();
            let spans: Vec<(usize, usize)> =
                blocks.iter().skip(skip).filter_map(|b| b.span).collect();
            format!(
                "tail {n} ({})",
                span_text(
                    spans.first().map_or(0, |s| s.0),
                    spans.last().map_or(0, |s| s.1)
                )
            )
        }
        Select::Changed { by } => {
            let handle = by.as_deref().map(|by| by.strip_prefix('@'));
            let selected = |text: &str| {
                is_marked(text)
                    && match handle {
                        None => true,
                        Some(Some(handle)) => has_handle(text, handle, comment_handles),
                        Some(None) => false,
                    }
            };
            keep = blocks.iter().map(|block| selected(&block.text)).collect();
            for note in notes.iter().filter(|note| selected(note)) {
                if let Some(label) = note_label(note) {
                    changed_notes.push(label.to_string());
                }
            }
            let names: Vec<String> = blocks
                .iter()
                .zip(&keep)
                .filter(|(_, keep)| **keep)
                .filter_map(|(block, _)| match (block.table, block.span) {
                    (Some(t), _) => Some(format!("t{t}")),
                    (None, Some((a, z))) => Some(span_text(a, z)),
                    _ => None,
                })
                .chain(changed_notes.iter().map(|label| format!("[^{label}]")))
                .collect();
            format!(
                "changed{} ({})",
                by.as_deref()
                    .map_or(String::new(), |by| format!(" by {by}")),
                if names.is_empty() {
                    "none".to_string()
                } else {
                    names.join(", ")
                }
            )
        }
        Select::Picks(picks) => {
            let mut wanted: Vec<(usize, usize)> = Vec::new();
            let mut tables: Vec<usize> = Vec::new();
            let mut names: Vec<(usize, String)> = Vec::new();
            for pick in picks {
                match pick {
                    Pick::Paragraphs { from, to } => {
                        let Some(last) = last else {
                            return Err("the body has no paragraphs".to_string());
                        };
                        let to = to.unwrap_or(last);
                        for n in [*from, to] {
                            if n > last {
                                return Err(format!("p{n} is past the last paragraph p{last}"));
                            }
                        }
                        wanted.push((*from, to));
                        names.push((*from, span_text(*from, to)));
                    }
                    Pick::Table(t) => {
                        let Some(block) = blocks.iter().find(|b| b.table == Some(*t)) else {
                            return Err(format!("t{t} is not a table of this document"));
                        };
                        tables.push(*t);
                        names.push((block.span.map_or(0, |s| s.0), format!("t{t}")));
                    }
                }
            }
            // Accepting or rejecting can join a paragraph into the one
            // before it, which then holds its index.
            for &(from, to) in &wanted {
                let found = blocks
                    .iter()
                    .any(|b| b.span.is_some_and(|(a, z)| a <= to && from <= z));
                if !found
                    && let Some((a, _)) = blocks
                        .iter()
                        .filter_map(|b| b.span)
                        .filter(|&(a, _)| a < from)
                        .max_by_key(|&(a, _)| a)
                {
                    return Err(format!(
                        "p{from} is part of p{a} in this view (accepting or rejecting joined them); pick p{a}, or read the tracked view"
                    ));
                }
            }
            keep = blocks
                .iter()
                .map(|b| {
                    b.table.is_some_and(|t| tables.contains(&t))
                        || b.span.is_some_and(|(a, z)| {
                            wanted.iter().any(|&(from, to)| a <= to && from <= z)
                        })
                })
                .collect();
            names.sort_by_key(|(at, _)| *at);
            names
                .into_iter()
                .map(|(_, name)| name)
                .collect::<Vec<_>>()
                .join(", ")
        }
    };
    let mut out = String::new();
    let mut page_due: Option<&str> = None;
    let mut page_written: Option<&str> = None;
    for (block, keep) in blocks.iter().zip(&keep) {
        if let Some(page) = &block.page {
            page_due = Some(page.as_str());
        }
        if !keep {
            continue;
        }
        if let Some(page) = page_due.take()
            && page_written != Some(page)
        {
            out.push_str(page);
            out.push_str("\n\n");
            page_written = Some(page);
        }
        out.push_str(&block.text);
        out.push_str("\n\n");
    }
    // `convert` ends the Markdown with its newline.
    out.truncate(out.trim_end_matches('\n').len());
    notes.retain(|note| {
        note_label(note).is_some_and(|label| {
            changed_notes.iter().any(|l| l == label) || out.contains(&format!("[^{label}]"))
        })
    });
    let mut paragraphs: Vec<usize> = blocks
        .iter()
        .zip(&keep)
        .filter(|(_, keep)| **keep)
        .filter_map(|(block, _)| block.span)
        .flat_map(|(a, z)| a..=z)
        .collect();
    paragraphs.sort_unstable();
    paragraphs.dedup();
    Ok((out, range, paragraphs))
}

/// The label of a note definition, `1` for `[^1]: …`. A note inserted or
/// deleted with its reference carries its label inside the mark
/// (`{++[^1]: …++}`).
pub(crate) fn note_label(note: &str) -> Option<&str> {
    let note = ["{++", "{--"]
        .into_iter()
        .find_map(|mark| note.strip_prefix(mark))
        .unwrap_or(note);
    note.strip_prefix("[^")?
        .split_once("]:")
        .map(|(label, _)| label)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::from_docx::ooxml::parse_xml;

    /// empryo review of #476: a nested item's indented id line opens its
    /// own block, and an indented page marker still attaches to the block
    /// after it, so `-p`, `--head`, `--tail` and `--changed` see the item.
    #[test]
    fn an_indented_id_line_opens_its_own_block() {
        let body = "<!-- p0 ListParagraph num \"1.\" -->\n1. One.\n\n\
                    \x20  <!-- page 2 of 2 -->\n\n\
                    \x20  <!-- p1 bullet -->\n   - Sub.\n\n<!-- p2 -->\nTwo.";
        let blocks = blocks_of(body);
        let spans: Vec<_> = blocks.iter().map(|block| block.span).collect();
        assert_eq!(spans, [Some((0, 0)), Some((1, 1)), Some((2, 2))]);
        assert_eq!(blocks[1].page.as_deref(), Some("<!-- page 2 of 2 -->"));
    }

    /// A nested list item's id line is indented with its item; its mark
    /// clause and its handle still select the block.
    #[test]
    fn an_indented_id_line_still_marks_its_block() {
        let block = "   <!-- p5 ListParagraph bullet, break-ins r3 @AS -->\n   - Sub.";
        assert!(is_marked(block));
        assert!(has_handle(block, "AS", &HashMap::new()));
        assert!(!has_handle(block, "AC", &HashMap::new()));
    }

    /// pi review of Task 5 F4: a clause key is a mark only with its tag or
    /// comment id after it, so a style or list label reading `rev` keeps no
    /// block under `--changed`.
    #[test]
    fn mark_clauses_need_a_tag_or_a_comment_id() {
        for marked in [
            "<!-- p0 rev r1 @AC -->",
            "<!-- p3 Quote, fmt r5 @AC; r6 @AC -->",
            "<!-- p5 comments c5 c6 -->",
            "<!-- p1 comments #c5 -->",
            "<!-- p2 in c9 -->",
            "<!-- p4 empty, break-ins r3 @AC -->",
            "<!-- t0 2x2, cells p0-p3 by row, fmt r12 @AS in t0.row1 -->",
            "<!-- t0 1x2, cells p0-p1 by row, comments c9 in p1 -->",
        ] {
            assert!(has_mark_clause(marked), "{marked}");
        }
        for plain in [
            "<!-- p0 rev -->",
            "<!-- p0 fmt, justify -->",
            "<!-- p0 comments -->",
            "<!-- p0 num \"See rev 3\" -->",
            "<!-- p0 in -->",
            "<!-- t0 1x1, cells p0-p0 by row -->",
        ] {
            assert!(!has_mark_clause(plain), "{plain}");
        }
    }

    /// CodeRabbit #392: a note inserted or deleted with its reference
    /// carries its label inside the mark.
    #[test]
    fn note_label_reads_through_the_mark_around_an_inserted_or_deleted_note() {
        assert_eq!(note_label("[^1]: text"), Some("1"));
        assert_eq!(note_label("{++[^2]: added++}{>>r5 @AC<<}"), Some("2"));
        assert_eq!(note_label("{--[^3]: gone--}{>>r6 @AC<<}"), Some("3"));
        assert_eq!(note_label("text [^4]: not a definition"), None);
    }

    #[test]
    fn escape_markdown_escapes_only_what_commonmark_acts_on() {
        for (text, escaped) in [
            ("a_b snake_case", "a_b snake_case"),
            ("_x_", "\\_x\\_"),
            ("x < y <br> </u> <!--", "x < y \\<br> \\</u> \\<!--"),
            ("2*3 `t` \\", "2\\*3 \\`t\\` \\\\"),
            ("~~a~~ ~x == y = z", "\\~\\~a\\~\\~ \\~x \\=\\= y = z"),
            ("a ^b^ c ~d~ e", "a \\^b\\^ c \\~d\\~ e"),
            ("H~2~O 2^10 x^2^ a ~ b ^", "H~2~O 2^10 x^2\\^ a ~ b ^"),
            ("[^1] [x]", "\\[^1] [x]"),
            // An entity reference would decode on the way back; a name
            // HTML does not define (`&foo;`, `AT&T;`) stays as it is.
            (
                "&copy; &#35; &#x23; AT&T & co &; &foo; AT&T; &amp;",
                "\\&copy; \\&#35; \\&#x23; AT&T & co &; &foo; AT&T; \\&amp;",
            ),
            // A `]` right before `(`, `[` or `:` would close a link, an
            // image, a reference or a reference definition.
            (
                "[a](b) ![i](p.png) [r]: /u [c][d]",
                "[a\\](b) ![i\\](p.png) [r\\]: /u [c\\][d]",
            ),
            ("[sic] (a) x]", "[sic] (a) x]"),
        ] {
            assert_eq!(escape_markdown(text), escaped, "{text}");
        }
    }

    #[test]
    fn escape_closing_hashes_escapes_a_closing_sequence_only() {
        for (text, escaped) in [
            ("C #", "C \\#"),
            ("#", "\\#"),
            ("C ##", "C \\##"),
            ("C#", "C#"),
            ("C# x", "C# x"),
            ("plain", "plain"),
        ] {
            assert_eq!(escape_closing_hashes(text), escaped, "{text}");
        }
    }

    #[test]
    fn escape_block_start_escapes_block_openers_only() {
        for (line, escaped) in [
            ("# x", "\\# x"),
            ("###### x", "\\###### x"),
            ("####### x", "####### x"),
            ("#tag", "#tag"),
            ("> q", "\\> q"),
            ("| a |", "\\| a |"),
            ("- x", "\\- x"),
            ("+ x", "\\+ x"),
            ("-x", "-x"),
            ("---", "\\---"),
            ("-", "\\-"),
            ("1. one", "1\\. one"),
            ("10) ten", "10\\) ten"),
            ("1.5 litres", "1.5 litres"),
            ("1.", "1\\."),
            ("plain", "plain"),
            // CommonMark lets a block opener sit after up to three spaces.
            ("   =", "   \\="),
            ("  # Fees", "  \\# Fees"),
            (" > q", " \\> q"),
            ("   - x", "   \\- x"),
            ("  1. one", "  1\\. one"),
            // Four spaces (or a tab) inside a paragraph are plain text.
            ("    # x", "    # x"),
            ("\t- x", "\t- x"),
        ] {
            assert_eq!(escape_block_start(line), escaped, "{line}");
        }
        assert_eq!(escape_line_starts("a  \n# b"), "a  \n\\# b");
    }

    const W: &str = r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main""#;

    fn body(inner: &str) -> Element {
        parse_xml(format!("<w:body {W}>{inner}</w:body>").as_bytes()).unwrap()
    }

    fn doc(inner: &str) -> Element {
        parse_xml(format!("<w:document {W}><w:body>{inner}</w:body></w:document>").as_bytes())
            .unwrap()
    }

    fn indices(e: &Element, out: &mut Vec<String>) {
        if e.is("p") {
            out.push(e.attr(INDEX).unwrap_or("-").to_string());
        }
        for child in e.elements() {
            indices(child, out);
        }
    }

    #[test]
    fn stamps_paragraphs_in_document_order_cells_included_text_boxes_excluded() {
        let mut body = body(
            r#"<w:p/><w:tbl><w:tr><w:tc><w:p/><w:p/></w:tc></w:tr></w:tbl><w:p><w:r><w:pict><w:txbxContent><w:p/></w:txbxContent></w:pict></w:r></w:p><w:sdt><w:sdtContent><w:p/></w:sdtContent></w:sdt>"#,
        );
        let (paragraphs, tables) = stamp(&mut body, &Handles::default());
        assert_eq!((paragraphs, tables), (5, 1));
        let mut seen = Vec::new();
        indices(&body, &mut seen);
        assert_eq!(seen, ["0", "1", "2", "3", "-", "4"]);
        assert_eq!(body.child("tbl").unwrap().attr(TABLE), Some("0"));
    }

    /// A strict `w:ind` writes universal measures (`72pt`, `1in`) where
    /// Transitional writes twips; the clause prints the same inches.
    #[test]
    fn a_universal_measure_indent_on_the_id_line_reads_as_twips() {
        let p = body(
            r#"<w:p><w:pPr><w:ind w:start="72pt" w:end="36pt" w:firstLine="18pt"/></w:pPr><w:r><w:t>x</w:t></w:r></w:p>"#,
        );
        let p = p.child("p").unwrap();
        let facts = LineFacts {
            index: 0,
            style: None,
            outline: None,
            heading: None,
            marker: None,
            level: 0,
            page_break: false,
            resolved: false,
            comments: &[],
            spans: &[],
            empty: false,
            unshown: &[],
        };
        let line = id_line(p, &facts, &Handles::default());
        assert_eq!(line, "<!-- p0 first-line 0.25in, left 1in, right 0.5in -->");
    }

    /// A parsable-but-absurd measure is not a measure the schema spells
    /// (`1e300pt`, `nanpt`): the clause drops, as it did before the Strict
    /// reader, instead of printing 250-digit inches or `NaNin`
    /// (#531 review defect 5).
    #[test]
    fn an_absurd_measure_indent_drops_its_clause() {
        for start in ["1e300pt", "nanpt", "999999999999999999999999999in"] {
            let p = body(&format!(
                r#"<w:p><w:pPr><w:ind w:start="{start}" w:end="36pt"/></w:pPr><w:r><w:t>x</w:t></w:r></w:p>"#
            ));
            let p = p.child("p").unwrap();
            let facts = LineFacts {
                index: 0,
                style: None,
                outline: None,
                heading: None,
                marker: None,
                level: 0,
                page_break: false,
                resolved: false,
                comments: &[],
                spans: &[],
                empty: false,
                unshown: &[],
            };
            let line = id_line(p, &facts, &Handles::default());
            assert_eq!(line, "<!-- p0 right 0.5in -->", "{start}: {line}");
        }
    }

    #[test]
    fn nested_tables_are_not_numbered() {
        let mut body = body(
            r#"<w:tbl><w:tr><w:tc><w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl></w:tc></w:tr></w:tbl><w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl>"#,
        );
        let (_, tables) = stamp(&mut body, &Handles::default());
        assert_eq!(tables, 2);
        // `find_all` stops at a match; nested tables need a full walk.
        fn all_tables<'a>(e: &'a Element, out: &mut Vec<&'a Element>) {
            if e.is("tbl") {
                out.push(e);
            }
            for child in e.elements() {
                all_tables(child, out);
            }
        }
        let mut all = Vec::new();
        all_tables(&body, &mut all);
        let numbered: Vec<Option<&str>> = all.iter().map(|t| t.attr(TABLE)).collect();
        assert_eq!(numbered, [Some("0"), None, Some("1")]);
    }

    #[test]
    fn revision_tags_join_neighbours_then_pair_an_insertion_with_the_deletion_after_it() {
        // Word Compare's substitution is the insertion, then the deletion;
        // a deletion before an insertion stays two changes.
        let d = doc(
            r#"<w:p><w:ins w:id="0" w:author="Ann Counsel"><w:r><w:t>x</w:t></w:r></w:ins><w:r><w:t> t </w:t></w:r><w:ins w:id="2" w:author="Ann Counsel"><w:r><w:t>b</w:t></w:r></w:ins><w:bookmarkStart w:id="9" w:name="_b"/><w:del w:id="1" w:author="Ann Counsel"><w:r><w:delText>a</w:delText></w:r></w:del><w:r><w:t> u </w:t></w:r><w:ins w:id="7" w:author="Ann Counsel"><w:r><w:t>d</w:t></w:r></w:ins><w:ins w:id="8" w:author="Ann Counsel"><w:r><w:t>e</w:t></w:r></w:ins><w:del w:id="3" w:author="Ann Counsel"><w:r><w:delText>c</w:delText></w:r></w:del><w:r><w:t> v </w:t></w:r><w:del w:id="4" w:author="Ann Counsel"><w:r><w:delText>f</w:delText></w:r></w:del><w:ins w:id="5" w:author="Ann Counsel"><w:r><w:t>g</w:t></w:r></w:ins></w:p>"#,
        );
        let handles = handles(&d, &[], None, None);
        let p = d.child("body").unwrap().child("p").unwrap();
        let tags: Vec<String> = revision_tags(p, &handles)
            .into_iter()
            .map(|t| format!("{}:{}", t.kind, t.tag))
            .collect();
        assert_eq!(
            tags,
            [
                "ins:0@AC",
                "sub:1+2@AC",
                "sub:3+7+8@AC",
                "del:4@AC",
                "ins:5@AC"
            ]
        );
    }

    #[test]
    fn revision_tags_pass_over_empty_marks_wherever_they_sit() {
        // An empty mark shows nothing: the id line's `rev` clause names it,
        // and it neither joins nor splits the changes around it.
        let d = doc(
            r#"<w:p><w:r><w:t>Empty </w:t></w:r><w:ins w:id="8" w:author="Ann Counsel"><w:r><w:t>a</w:t></w:r></w:ins><w:ins w:id="9" w:author="Ann Counsel"/><w:r><w:t> end.</w:t></w:r><w:ins w:id="10" w:author="Ann Counsel"><w:r><w:rPr><w:b/></w:rPr></w:r></w:ins><w:del w:id="11" w:author="Ann Counsel"><w:r><w:delText>b</w:delText></w:r></w:del><w:ins w:id="12" w:author="Ann Counsel"/><w:del w:id="13" w:author="Ann Counsel"><w:r><w:delText>c</w:delText></w:r></w:del></w:p>"#,
        );
        let handles = handles(&d, &[], None, None);
        let p = d.child("body").unwrap().child("p").unwrap();
        let tags: Vec<String> = revision_tags(p, &handles)
            .into_iter()
            .map(|t| format!("{}:{}", t.kind, t.tag))
            .collect();
        assert_eq!(tags, ["ins:8@AC", "del:11+13@AC"]);
    }

    #[test]
    fn inches_read_back_to_the_same_twips() {
        // 2cm margins are 1134 twips: `0.79in` came back as 1138.
        assert_eq!(inches(1134.0), "0.7875in");
        assert_eq!(inches(1417.0), "0.984in");
        assert_eq!(inches(720.0), "0.5in");
        assert_eq!(inches(1440.0), "1in");
        for twips in 0..=31_680u32 {
            let text = inches(f64::from(twips));
            let back: f64 = text.strip_suffix("in").unwrap().parse().unwrap();
            assert_eq!((back * 1440.0).round(), f64::from(twips), "{text}");
        }
    }

    #[test]
    fn a_shown_range_or_bookmark_between_marks_keeps_them_apart() {
        // The text prints `{++a++}{=={--b--}==}{>>c0<<}`: two changes, not
        // a substitution, and `{++a++}<a id="m"></a>{++b++}` two insertions.
        // A range or bookmark the view does not print joins as before.
        let d = doc(
            r#"<w:p><w:ins w:id="1" w:author="Ann Counsel"><w:r><w:t>a</w:t></w:r></w:ins><w:commentRangeStart w:id="0"/><w:del w:id="2" w:author="Ann Counsel"><w:r><w:delText>b</w:delText></w:r></w:del><w:commentRangeEnd w:id="0"/><w:r><w:t> </w:t></w:r><w:ins w:id="3" w:author="Ann Counsel"><w:r><w:t>c</w:t></w:r></w:ins><w:bookmarkStart w:id="0" w:name="m"/><w:ins w:id="4" w:author="Ann Counsel"><w:r><w:t>d</w:t></w:r></w:ins><w:r><w:t> </w:t></w:r><w:ins w:id="5" w:author="Ann Counsel"><w:r><w:t>e</w:t></w:r></w:ins><w:commentRangeStart w:id="7"/><w:bookmarkStart w:id="1" w:name="n"/><w:del w:id="6" w:author="Ann Counsel"><w:r><w:delText>f</w:delText></w:r></w:del></w:p>"#,
        );
        let mut handles = handles(&d, &[], None, None);
        let p = d.child("body").unwrap().child("p").unwrap();
        let tags = |handles: &Handles| -> Vec<String> {
            revision_tags(p, handles)
                .into_iter()
                .map(|t| format!("{}:{}", t.kind, t.tag))
                .collect()
        };
        assert_eq!(tags(&handles), ["sub:2+1@AC", "ins:3+4@AC", "sub:6+5@AC"]);
        handles.shown_ranges.insert("0".into());
        handles.shown_bookmarks.insert("m".into());
        assert_eq!(
            tags(&handles),
            ["ins:1@AC", "del:2@AC", "ins:3@AC", "ins:4@AC", "sub:6+5@AC"]
        );
    }

    #[test]
    fn tags_always_carry_the_handle_and_format_for_printing() {
        let d = doc(
            r#"<w:p><w:ins w:id="0" w:author="Ann Counsel"><w:r><w:t>a</w:t></w:r></w:ins><w:r><w:t> </w:t></w:r><w:del w:id="1" w:author="John Doe"><w:r><w:delText>b</w:delText></w:r></w:del></w:p>"#,
        );
        let handles = handles(&d, &[], None, None);
        assert_eq!(handles.by_author["Ann Counsel"], "AC");
        assert_eq!(handles.by_author["John Doe"], "JD");
        let p = d.child("body").unwrap().child("p").unwrap();
        let tags: Vec<String> = revision_tags(p, &handles)
            .into_iter()
            .map(|t| t.tag)
            .collect();
        assert_eq!(tags, ["0@AC", "1@JD"]);
        assert_eq!(join_tags(&["1@AC".into(), "2@AC".into()]), "1+2@AC");
        assert_eq!(join_tags(&["1@AC".into(), "2@JD".into()]), "1@AC|2@JD");
        assert_eq!(join_tags(&["1+2@AC".into(), "3@AC".into()]), "1+2+3@AC");
        assert_eq!(format_tag("0@AC"), "r0 @AC");
        assert_eq!(format_tag("1+2@AC"), "r1+2 @AC");
        assert_eq!(format_tag("1@AC|2@JD"), "r1 @AC; r2 @JD");
        assert_eq!(format_tag("footnotes#1+2@AC"), "footnotes.r1+2 @AC");
        assert_eq!(format_tag("1@AC|header1#2@JD"), "r1 @AC; header1.r2 @JD");
        // An id the short form cannot spell prints its long id.
        assert_eq!(format_tag("007@AC"), "body:rev:007 @AC");
        assert_eq!(
            format_tag("Header_1#4+5@AC"),
            "Header_1:rev:4 @AC; Header_1:rev:5 @AC"
        );
        assert_eq!(format_tag("?@??"), "noid @??");
        assert_eq!(format_tag("footnotes#?@AC"), "footnotes.noid @AC");
        assert!(crate::ids::parse_change_ids("noid").is_err());
        assert_eq!(handle_of("1+2@AC"), Some("AC"));
        assert_eq!(handle_of("header1#1@AC"), Some("AC"));
        assert_eq!(
            join_tags(&["header1#1@AC".into(), "header1#2@AC".into()]),
            "header1#1+2@AC"
        );
        assert_eq!(
            join_tags(&["1@AC".into(), "header1#2@AC".into()]),
            "1@AC|header1#2@AC"
        );
    }

    #[test]
    fn handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps() {
        let d = doc(
            r#"<w:p><w:ins w:id="0" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/><w:ins w:id="1" w:author="Al Cooper" w:date="2026-10-02T08:00:00Z"/><w:ins w:id="2" w:author="Al Cooper" w:date="2026-10-03T08:30:00Z"/></w:p>"#,
        );
        let comments = parse_xml(format!(r#"<w:comments {W}><w:comment w:id="5" w:author="Arthur Souza Rodrigues" w:initials="AS" w:date="2026-10-09T16:13:00Z"/><w:comment w:id="6" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:comments>"#).as_bytes()).unwrap();
        let handles = handles(&d, &[], Some(&comments), None);
        assert_eq!(
            handles.order,
            ["Ann Counsel", "Al Cooper", "Arthur Souza Rodrigues"]
        );
        assert_eq!(handles.by_author["Ann Counsel"], "AC");
        assert_eq!(handles.by_author["Al Cooper"], "AC2");
        assert_eq!(handles.by_author["Arthur Souza Rodrigues"], "AS");
        assert_eq!(
            handles.unique_date("Ann Counsel"),
            Some("2026-10-01T09:00:00Z")
        );
        assert_eq!(handles.unique_date("Al Cooper"), None);
        assert_eq!(
            handles.date_range("Al Cooper"),
            Some("2026-10-02..2026-10-03".to_string())
        );
        assert_eq!(handles.unique_date("Nobody"), None);
    }

    #[test]
    fn range_markers_do_not_break_a_move_or_a_substitution() {
        let d = doc(
            r#"<w:p><w:moveToRangeStart w:id="21" w:name="m"/><w:moveTo w:id="2" w:author="Ann Counsel"><w:r><w:t>b</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="21"/><w:permStart w:id="30"/><w:moveFromRangeStart w:id="20" w:name="m"/><w:moveFrom w:id="1" w:author="Ann Counsel"><w:r><w:delText>a</w:delText></w:r></w:moveFrom><w:moveFromRangeEnd w:id="20"/><w:permEnd w:id="30"/></w:p>"#,
        );
        let handles = handles(&d, &[], None, None);
        let p = d.child("body").unwrap().child("p").unwrap();
        let tags: Vec<String> = revision_tags(p, &handles)
            .into_iter()
            .map(|t| format!("{}:{}", t.kind, t.tag))
            .collect();
        assert_eq!(tags, ["sub:1+2@AC"]);
    }

    #[test]
    fn joining_no_tags_is_empty() {
        assert_eq!(join_tags(&[]), "");
    }

    #[test]
    fn format_changes_list_in_document_order() {
        let d = doc(
            r#"<w:p><w:pPr><w:pPrChange w:id="1" w:author="Ann Counsel"><w:pPr/></w:pPrChange></w:pPr><w:r><w:rPr><w:b/><w:rPrChange w:id="2" w:author="Ann Counsel"><w:rPr/></w:rPrChange></w:rPr><w:t>x</w:t></w:r></w:p>"#,
        );
        let handles = handles(&d, &[], None, None);
        let p = d.child("body").unwrap().child("p").unwrap();
        assert_eq!(format_change_tags(p, &handles), ["1@AC", "2@AC"]);
    }

    #[test]
    fn a_stamp_cannot_be_forged_by_the_file() {
        let mut body = body(r#"<w:p w:jubarteIndex="99"/>"#);
        stamp(&mut body, &Handles::default());
        assert_eq!(body.child("p").unwrap().attr(INDEX), Some("0"));
    }

    #[test]
    fn index_runs_collapse_consecutive_indices() {
        assert_eq!(runs(&[3, 4, 5, 9, 11, 12]), [(3, 5), (9, 9), (11, 12)]);
        assert!(runs(&[]).is_empty());
    }
}
