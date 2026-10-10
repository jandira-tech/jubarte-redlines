// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! The agent view: numbering that matches `inspect` and `edit`, revision
//! tags, author handles and timestamps, comment threads, id lines, table
//! lines and block selection.

use std::collections::{BTreeSet, HashMap, HashSet};

use super::ooxml::{Element, Node};

/// Attribute stamped on every numbered `w:p`: its `body:p:N` index.
pub(crate) const INDEX: &str = "jubarteIndex";
/// Attribute stamped on every top-level `w:tbl`: its `t{N}` number.
pub(crate) const TABLE: &str = "jubarteTable";
/// Attribute stamped on a `w:p` that holds revisions: `kind:tag` entries
/// separated by spaces (`ins:0@AC sub:1+2@AC`), recorded before resolution.
pub(crate) const REVS: &str = "jubarteRevs";

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
pub(crate) fn revision_tags(p: &Element, handles: &Handles) -> Vec<RevTag> {
    // One slot per child: a revision, or `None` for anything else that
    // breaks adjacency.
    let mut slots: Vec<Option<RevTag>> = Vec::new();
    for e in p.elements() {
        if matches!(
            e.local(),
            "bookmarkStart" | "bookmarkEnd" | "proofErr" | "commentRangeStart" | "commentRangeEnd"
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
    let mut found = Vec::new();
    p.find_all("rPrChange", &mut found);
    p.find_all("pPrChange", &mut found);
    found.iter().map(|e| tag_of(e, handles)).collect()
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
        let revs: Vec<String> = revision_tags(element, handles)
            .into_iter()
            .map(|t| format!("{}:{}", t.kind, t.tag))
            .collect();
        element.attrs.push((INDEX.to_string(), c.p.to_string()));
        c.p += 1;
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

/// The `kind:tag` entries stamped on a paragraph, as (kind, tag).
pub(crate) fn stamped_revs(p: &Element) -> Vec<(String, String)> {
    p.attr(REVS)
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
    if is_revision(e.local()) || e.local().ends_with("PrChange") {
        if let Some(author) = e.attr("author") {
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
    }
    for child in e.elements() {
        walk_revision_authors(child, out, dates);
    }
}

/// Authors of revisions (document order) then of comments (comment order),
/// each with a handle: the comment `w:initials` that author wrote, else the
/// uppercase initials of the name's words; a collision appends 2, 3, ….
pub(crate) fn handles(document: &Element, comments: Option<&Element>) -> Handles {
    let mut order = Vec::new();
    let mut dates: HashMap<String, BTreeSet<String>> = HashMap::new();
    walk_revision_authors(document, &mut order, &mut dates);
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
    for author in &order {
        let base = initials.get(author).cloned().unwrap_or_else(|| {
            let letters: String = author
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .collect::<String>()
                .to_uppercase();
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
}

/// Twips as inches with up to two decimals: `720` → `0.5in`, `1440` → `1in`.
pub(crate) fn inches(twips: f64) -> String {
    let value = format!("{:.2}", twips / 1440.0);
    let value = value.trim_end_matches('0').trim_end_matches('.');
    format!("{value}in")
}

pub(crate) fn has_page_break(p: &Element) -> bool {
    let mut breaks = Vec::new();
    p.find_all("br", &mut breaks);
    breaks.iter().any(|b| b.attr("type") == Some("page"))
}

pub(crate) fn has_rendered_page_break(p: &Element) -> bool {
    let mut found = Vec::new();
    p.find_all("lastRenderedPageBreak", &mut found);
    !found.is_empty()
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
/// hard page and section breaks, for the cached-break page count.
pub(crate) fn page_counts(e: &Element) -> (usize, usize) {
    if e.is("txbxContent") {
        return (0, 0);
    }
    let mut rendered = 0;
    let mut hard = 0;
    if e.is("p") {
        rendered += usize::from(has_rendered_page_break(e));
        hard += usize::from(has_page_break(e)) + usize::from(has_section_break(e));
    }
    for child in e.elements() {
        let (r, h) = page_counts(child);
        rendered += r;
        hard += h;
    }
    (rendered, hard)
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
        let handles = handles(&d, None);
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
        let handles = handles(&d, None);
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
        let handles = handles(&d, Some(&comments));
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
    fn index_runs_collapse_consecutive_indices() {
        assert_eq!(runs(&[3, 4, 5, 9, 11, 12]), [(3, 5), (9, 9), (11, 12)]);
        assert!(runs(&[]).is_empty());
    }
}
