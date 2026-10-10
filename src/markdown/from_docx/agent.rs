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
/// Attribute stamped on every numbered `w:p`: its `body:p:N` index.
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
    /// order, then comment-only authors in part order.
    pub order: Vec<String>,
    /// Author → every `w:date` on their marks and comments.
    pub dates: HashMap<String, BTreeSet<String>>,
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

/// `12@AC`: the element's `w:id` and its author's handle (`??` for an
/// author the legend does not know, which only a malformed file produces).
pub(crate) fn tag_of(element: &Element, handles: &Handles) -> String {
    let id = element.attr("id").unwrap_or("?");
    let handle = handles.of(element.attr("author")).unwrap_or("??");
    format!("{id}@{handle}")
}

/// The handle of a one-author tag (`1+2@AC` → `AC`); `None` for several.
pub(crate) fn handle_of(tag: &str) -> Option<&str> {
    if tag.contains('|') {
        return None;
    }
    tag.rsplit_once('@').map(|(_, h)| h)
}

/// Joins the tags of one logical change: `1@AC` + `2@AC` is `1+2@AC`,
/// `1@AC` + `2@JD` is `1@AC|2@JD`, `1+2@AC` + `3@AC` is `1+2+3@AC`.
pub(crate) fn join_tags(parts: &[String]) -> String {
    if parts.is_empty() {
        return String::new();
    }
    let handles: Vec<Option<&str>> = parts.iter().map(|p| handle_of(p)).collect();
    if handles.iter().all(|h| h.is_some() && *h == handles[0]) {
        let ids: Vec<&str> = parts
            .iter()
            .map(|p| p.rsplit_once('@').map_or(p.as_str(), |(ids, _)| ids))
            .collect();
        format!("{}@{}", ids.join("+"), handles[0].unwrap_or("??"))
    } else {
        parts.join("|")
    }
}

/// `0@AC` → `#0 @AC`; `1+2@AC` → `#1+2 @AC`; `1@AC|2@JD` → `#1 @AC; #2 @JD`.
pub(crate) fn format_tag(tag: &str) -> String {
    tag.split('|')
        .map(|one| match one.rsplit_once('@') {
            Some((ids, handle)) => format!("#{ids} @{handle}"),
            None => format!("#{one}"),
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// A list of tags for an id line: `#0 @AC; #1+2 @AC`.
pub(crate) fn format_tags(tags: &[String]) -> String {
    tags.iter()
        .map(|t| format_tag(t))
        .collect::<Vec<_>>()
        .join("; ")
}

/// The revisions among a paragraph's direct children, in order, as the
/// renderer shows them: neighbouring marks of one kind by one author join
/// (`7+8@AC`), then a deletion directly followed by an insertion (or the
/// reverse) pairs as a substitution, greedily left to right. Bookmarks,
/// proofing marks and comment markers do not break adjacency; a run does.
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
                "hyperlink" | "smartTag" | "customXml" | "fldSimple" => flatten(e, out),
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
        ) {
            continue;
        }
        slots.push(kind_of(e.local()).map(|kind| RevTag {
            kind,
            tag: tag_of(e, handles),
            author: e.attr("author").map(str::to_string),
            date: e.attr("date").map(str::to_string),
        }));
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
    // 2. Pair a deletion with the insertion beside it.
    let mut out = Vec::new();
    let mut i = 0;
    while i < joined.len() {
        let Some(current) = joined[i].take() else {
            i += 1;
            continue;
        };
        let next_kind = joined.get(i + 1).and_then(|n| n.as_ref()).map(|n| n.kind);
        let pair = matches!(
            (current.kind, next_kind),
            ("del", Some("ins")) | ("ins", Some("del"))
        );
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

/// Tags of the formatting changes recorded in a paragraph.
pub(crate) fn format_change_tags(p: &Element, handles: &Handles) -> Vec<String> {
    fn walk(e: &Element, handles: &Handles, out: &mut Vec<String>) {
        for child in e.elements() {
            if child.is("rPrChange") || child.is("pPrChange") {
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

fn walk_revision_authors(
    e: &Element,
    out: &mut Vec<String>,
    dates: &mut HashMap<String, BTreeSet<String>>,
) {
    if (is_revision(e.local()) || e.local().ends_with("PrChange"))
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
    for child in e.elements() {
        walk_revision_authors(child, out, dates);
    }
}

/// Authors of revisions (document order, then footnotes and endnotes) then
/// of comments (comment order),
/// each with a handle: the comment `w:initials` that author wrote, else the
/// uppercase initials of the name's words; a collision appends 2, 3, ….
pub(crate) fn handles(
    document: &Element,
    notes: &[&Element],
    comments: Option<&Element>,
) -> Handles {
    let mut order = Vec::new();
    let mut dates: HashMap<String, BTreeSet<String>> = HashMap::new();
    walk_revision_authors(document, &mut order, &mut dates);
    for root in notes {
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
    pub default_style: Option<&'a str>,
    pub heading: Option<usize>,
    /// The list marker the converter computed (`1.`, `(a)`, `-`).
    pub marker: Option<&'a str>,
    pub page_break: bool,
    /// Accept-all or reject-all view: print the `rev` tags.
    pub resolved: bool,
    /// Comment ids to list (comments hidden).
    pub comments: &'a [String],
    /// The paragraph has no text but keeps its own line for its marks,
    /// revisions or comments: `<!-- p4 empty, break-ins #3 @AC -->`.
    pub empty: bool,
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

/// Twips as inches with up to two decimals: `720` → `0.5in`, `1440` → `1in`.
pub(crate) fn inches(twips: f64) -> String {
    let value = format!("{:.2}", twips / 1440.0);
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

/// The authors of the formatting changes (`*PrChange`) under `e`, text
/// boxes excluded, in document order.
pub(crate) fn format_change_authors(e: &Element) -> Vec<Option<String>> {
    fn walk(e: &Element, out: &mut Vec<Option<String>>) {
        for c in e.elements().filter(|c| !c.is("txbxContent")) {
            if c.local().ends_with("PrChange") {
                out.push(c.attr("author").map(str::to_string));
            }
            walk(c, out);
        }
    }
    let mut out = Vec::new();
    walk(e, &mut out);
    out
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

/// `<!-- p3 justify, first-line 0.5in, comments #c5 -->` (the plan's grammar).
pub(crate) fn id_line(p: &Element, f: &LineFacts, handles: &Handles) -> String {
    let mut head = format!("p{}", f.index);
    if let Some(style) = f.style {
        let implied = match f.heading {
            Some(level) => style == format!("Heading{level}"),
            None => Some(style) == f.default_style || style == "Normal",
        };
        if !implied {
            head.push(' ');
            head.push_str(style);
        }
    }
    let ppr = p.child("pPr");
    let mut clauses: Vec<String> = Vec::new();
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
        for (attr, name) in [
            ("firstLine", "first-line"),
            ("hanging", "hanging"),
            ("left", "left"),
            ("start", "left"),
            ("right", "right"),
            ("end", "right"),
        ] {
            if let Some(v) = ind.attr(attr).and_then(|v| v.parse::<f64>().ok()) {
                clauses.push(format!("{name} {}", inches(v)));
            }
        }
    }
    match f.marker {
        Some("-") => clauses.push("bullet".to_string()),
        Some(label) => clauses.push(format!("num \"{label}\"")),
        None => {}
    }
    if f.page_break {
        clauses.push("page-break".to_string());
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
    if f.resolved {
        let tags: Vec<String> = stamped_revs(p).into_iter().map(|(_, tag)| tag).collect();
        if !tags.is_empty() {
            clauses.push(format!("rev {}", format_tags(&tags)));
        }
    }
    if !f.comments.is_empty() {
        let ids: Vec<String> = f.comments.iter().map(|c| format!("#c{c}")).collect();
        clauses.push(format!("comments {}", ids.join(" ")));
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

/// `<!-- t0 center 3x3, cells p8-p16 by row, header row repeats -->`;
/// `None` for a nested table (not numbered). With `comments` (comments
/// hidden) the cells' comment ids print as `comments #c9 in p3`.
pub(crate) fn table_line(
    tbl: &Element,
    resolved: bool,
    comments: Option<&HashMap<String, Element>>,
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
    let mut merged = false;
    let mut break_ins: Vec<String> = Vec::new();
    let mut break_del: Vec<String> = Vec::new();
    let mut revs: Vec<String> = Vec::new();
    let mut held: Vec<String> = Vec::new();
    for (r, tr) in rows.iter().enumerate() {
        let mut range: Option<(usize, usize)> = None;
        if resolved {
            revs.extend(
                stamped_revs(tr)
                    .into_iter()
                    .map(|(_, tag)| format!("{} in r{r}", format_tag(&tag))),
            );
        }
        for (c, tc) in super::row_cells(tr).enumerate() {
            if resolved {
                revs.extend(
                    stamped_revs(tc)
                        .into_iter()
                        .map(|(_, tag)| format!("{} in r{r}.c{c}", format_tag(&tag))),
                );
            }
            if tc.path(&["tcPr", "gridSpan"]).is_some() || tc.path(&["tcPr", "vMerge"]).is_some() {
                merged = true;
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
                }
                let ids: Vec<String> = comments.map_or_else(Vec::new, |known| {
                    comment_ids(p)
                        .into_iter()
                        .filter(|id| known.contains_key(id))
                        .collect()
                });
                if !ids.is_empty() {
                    let ids: Vec<String> = ids.iter().map(|c| format!("#c{c}")).collect();
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
                    format!("r{i} p{a}")
                } else {
                    format!("r{i} p{a}-p{b}")
                }
            })
            .collect();
        clauses.push(format!("cells {}", rows.join(" ")));
    }
    if rows
        .first()
        .is_some_and(|tr| tr.path(&["trPr", "tblHeader"]).is_some())
    {
        clauses.push("header row repeats".to_string());
    }
    if merged {
        clauses.push("merged cells".to_string());
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
    if !held.is_empty() {
        clauses.push(format!("comments {}", held.join("; ")));
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

/// `#c5 @AC: `, `#c5 @AC resolved: `, `#c6 @AS re #c5: `, or `#c5: ` with
/// no author. `date` is the inline timestamp, when wanted.
pub(crate) fn comment_head(
    id: &str,
    author: Option<&str>,
    date: Option<&str>,
    handles: &Handles,
    threads: &Threads,
) -> String {
    let mut head = format!("#c{id}");
    if let Some(handle) = handles.of(author) {
        head.push_str(&format!(" @{handle}"));
    }
    if let Some(date) = date {
        head.push(' ');
        head.push_str(date);
    }
    if let Some(root) = threads.reply_of.get(id) {
        head.push_str(&format!(" re #c{root}"));
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
    if e.is("txbxContent") {
        return;
    }
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
    for child in e.elements() {
        collect_in(child, handles, out);
    }
}

/// Count of revision elements (`w:ins`, `w:del`, moves, cell marks) and of
/// formatting changes (`*PrChange`) under `body`, text boxes excluded.
pub(crate) fn count_marks(e: &Element) -> (usize, usize) {
    if e.is("txbxContent") {
        return (0, 0);
    }
    let mut marks = usize::from(is_revision(e.local()));
    let mut formats = usize::from(e.local().ends_with("PrChange"));
    for child in e.elements() {
        let (m, f) = count_marks(child);
        marks += m;
        formats += f;
    }
    (marks, formats)
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
        let first = chunk.lines().next().unwrap_or("");
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
fn has_handle(text: &str, handle: &str) -> bool {
    let key = format!("@{handle}");
    text.match_indices(&key).any(|(at, _)| {
        text[at + key.len()..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_alphanumeric())
    })
}

/// Whether a block of the agent view carries a tracked change or a comment:
/// a note in its text, or a mark clause on its id or table line.
fn is_marked(block: &str) -> bool {
    let first = block.lines().next().unwrap_or("");
    block.contains("{>>#")
        || [
            " rev #",
            " comments #",
            " break-ins #",
            " break-del #",
            " fmt #",
        ]
        .iter()
        .any(|key| first.contains(key))
}

/// The selected blocks of `body` joined back, and the `range:` text. A
/// `Select::Changed` author arrives resolved: `@AC` for a known handle, the
/// text as given otherwise (no block holds an unknown author's marks).
pub(crate) fn select_blocks(
    body: &str,
    select: &Select,
    last: usize,
) -> Result<(String, String), String> {
    let blocks = blocks_of(body);
    let keep: Vec<bool>;
    let range: String;
    match select {
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
            range = format!(
                "head {n} ({})",
                span_text(
                    spans.first().map_or(0, |s| s.0),
                    spans.last().map_or(0, |s| s.1)
                )
            );
        }
        Select::Tail(n) => {
            let skip = blocks.len().saturating_sub(*n);
            keep = (0..blocks.len()).map(|i| i >= skip).collect();
            let spans: Vec<(usize, usize)> =
                blocks.iter().skip(skip).filter_map(|b| b.span).collect();
            range = format!(
                "tail {n} ({})",
                span_text(
                    spans.first().map_or(0, |s| s.0),
                    spans.last().map_or(0, |s| s.1)
                )
            );
        }
        Select::Changed { by } => {
            let handle = by.as_deref().map(|by| by.strip_prefix('@'));
            keep = blocks
                .iter()
                .map(|block| {
                    is_marked(&block.text)
                        && match handle {
                            None => true,
                            Some(Some(handle)) => has_handle(&block.text, handle),
                            Some(None) => false,
                        }
                })
                .collect();
            let names: Vec<String> = blocks
                .iter()
                .zip(&keep)
                .filter(|(_, keep)| **keep)
                .filter_map(|(block, _)| match (block.table, block.span) {
                    (Some(t), _) => Some(format!("t{t}")),
                    (None, Some((a, z))) => Some(span_text(a, z)),
                    _ => None,
                })
                .collect();
            range = format!(
                "changed{} ({})",
                by.as_deref()
                    .map_or(String::new(), |by| format!(" by {by}")),
                if names.is_empty() {
                    "none".to_string()
                } else {
                    names.join(", ")
                }
            );
        }
        Select::Picks(picks) => {
            let mut wanted: Vec<(usize, usize)> = Vec::new();
            let mut tables: Vec<usize> = Vec::new();
            let mut names: Vec<(usize, String)> = Vec::new();
            for pick in picks {
                match pick {
                    Pick::Paragraphs { from, to } => {
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
            range = names
                .into_iter()
                .map(|(_, name)| name)
                .collect::<Vec<_>>()
                .join(", ");
        }
    }
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
    Ok((out, range))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markdown::from_docx::ooxml::parse_xml;

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
    fn revision_tags_join_neighbours_then_pair_a_deletion_with_the_insertion_beside_it() {
        let d = doc(
            r#"<w:p><w:ins w:id="0" w:author="Ann Counsel"><w:r><w:t>x</w:t></w:r></w:ins><w:r><w:t> t </w:t></w:r><w:del w:id="1" w:author="Ann Counsel"><w:r><w:delText>a</w:delText></w:r></w:del><w:bookmarkStart w:id="9" w:name="_b"/><w:ins w:id="2" w:author="Ann Counsel"><w:r><w:t>b</w:t></w:r></w:ins><w:r><w:t> u </w:t></w:r><w:del w:id="3" w:author="Ann Counsel"><w:r><w:delText>c</w:delText></w:r></w:del><w:ins w:id="7" w:author="Ann Counsel"><w:r><w:t>d</w:t></w:r></w:ins><w:ins w:id="8" w:author="Ann Counsel"><w:r><w:t>e</w:t></w:r></w:ins></w:p>"#,
        );
        let handles = handles(&d, &[], None);
        let p = d.child("body").unwrap().child("p").unwrap();
        let tags: Vec<String> = revision_tags(p, &handles)
            .into_iter()
            .map(|t| format!("{}:{}", t.kind, t.tag))
            .collect();
        assert_eq!(tags, ["ins:0@AC", "sub:1+2@AC", "sub:3+7+8@AC"]);
    }

    #[test]
    fn tags_always_carry_the_handle_and_format_for_printing() {
        let d = doc(
            r#"<w:p><w:ins w:id="0" w:author="Ann Counsel"><w:r><w:t>a</w:t></w:r></w:ins><w:r><w:t> </w:t></w:r><w:del w:id="1" w:author="John Doe"><w:r><w:delText>b</w:delText></w:r></w:del></w:p>"#,
        );
        let handles = handles(&d, &[], None);
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
        assert_eq!(format_tag("0@AC"), "#0 @AC");
        assert_eq!(format_tag("1+2@AC"), "#1+2 @AC");
        assert_eq!(format_tag("1@AC|2@JD"), "#1 @AC; #2 @JD");
        assert_eq!(handle_of("1+2@AC"), Some("AC"));
    }

    #[test]
    fn handles_prefer_comment_initials_disambiguate_collisions_and_collect_timestamps() {
        let d = doc(
            r#"<w:p><w:ins w:id="0" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/><w:ins w:id="1" w:author="Al Cooper" w:date="2026-10-02T08:00:00Z"/><w:ins w:id="2" w:author="Al Cooper" w:date="2026-10-03T08:30:00Z"/></w:p>"#,
        );
        let comments = parse_xml(format!(r#"<w:comments {W}><w:comment w:id="5" w:author="Arthur Souza Rodrigues" w:initials="AS" w:date="2026-10-09T16:13:00Z"/><w:comment w:id="6" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"/></w:comments>"#).as_bytes()).unwrap();
        let handles = handles(&d, &[], Some(&comments));
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
            r#"<w:p><w:moveFromRangeStart w:id="20" w:name="m"/><w:moveFrom w:id="1" w:author="Ann Counsel"><w:r><w:delText>a</w:delText></w:r></w:moveFrom><w:moveFromRangeEnd w:id="20"/><w:permStart w:id="30"/><w:moveToRangeStart w:id="21" w:name="m"/><w:moveTo w:id="2" w:author="Ann Counsel"><w:r><w:t>b</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="21"/><w:permEnd w:id="30"/></w:p>"#,
        );
        let handles = handles(&d, &[], None);
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
        let handles = handles(&d, &[], None);
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
