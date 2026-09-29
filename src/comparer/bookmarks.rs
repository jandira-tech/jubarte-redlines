// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Bookmark carryover.
//!
//! Word's Compare keeps the body bookmarks of both documents — the union by
//! name — so a TOC's `PAGEREF` fields, `REF` cross-references and internal
//! hyperlinks still resolve in the redline. WmlComparer strips every bookmark
//! in PreProcessMarkup, and an updated field whose bookmark is gone prints
//! "Error! Bookmark not defined." (file_21 × file_22: 582 bookmarks, every TOC
//! line broken). Docxodus's IR renderer preserves them as well.
//!
//! Placement reuses the comment anchors' character-offset projection
//! ([`super::comments`]): B's bookmarks land on the merged body's B text,
//! A-only ones on its A text (deleted text included). Each endpoint is found
//! by the text around it, searching forward from the previous bookmark so
//! repeated boilerplate resolves in document order. A bookmark present in
//! both documents is placed once, from B.
//!
//! Text offsets know nothing of content controls, so an endpoint placed inside
//! a `w:sdt` its source bookmark was not in leaves it: a start before the
//! control, an end after it. B's body-level `_Toc` bookmarks around a
//! data-bound title control otherwise land inside it, and Word refuses a
//! bookmark in a plain-text or dropdown control (en 7b649361, 57c181da).

use std::collections::{HashMap, HashSet};

use super::comments::{Seg, collect_segments, place_after_offset, place_before_offset};
use crate::namespaces::W;
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId};

/// Word's hidden last-edit bookmark: its Compare never writes one.
const GO_BACK: &str = "_GoBack";

/// One source bookmark: its name and `[start, end)` character interval in
/// the source's text projection.
struct Marker {
    name: String,
    start: usize,
    end: usize,
    /// No text precedes the start in its paragraph. An empty bookmark there
    /// belongs to the paragraph that follows it, not to the text before it.
    leads: bool,
    /// `w:colFirst`/`w:colLast`, kept verbatim as Word does (Google Docs
    /// writes them on plain paragraph bookmarks).
    cols: Option<(String, String)>,
    /// The start sits in a content control in its source.
    in_sdt: bool,
}

/// The nearest enclosing paragraph of `node`, if any.
fn paragraph_of(dom: &Dom, mut node: NodeId) -> Option<NodeId> {
    let p = W::p();
    while let Some(parent) = dom.parent(node) {
        if dom.name_is(parent, &p) {
            return Some(parent);
        }
        node = parent;
    }
    None
}

/// The source body's text projection (every `w:t` character, as
/// [`collect_segments`] counts them) and its bookmarks in start order.
/// A name's second definition is left out. A start whose end is gone (it sat
/// in tracked-deleted text, removed when revisions were accepted) closes
/// where it starts, as Docxodus closes such an orphan.
fn extract(pkg: &PartFs, main: &str) -> (Vec<char>, Vec<Marker>) {
    let Some(xml) = pkg.part_string(main) else {
        return (Vec::new(), Vec::new());
    };
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    // The main part's body, or a header, footer or notes part's root.
    let Some(body) = dom.root(d).map(|r| dom.element(r, &W::body()).unwrap_or(r)) else {
        return (Vec::new(), Vec::new());
    };
    let (start, end, t) = (W::name("bookmarkStart"), W::name("bookmarkEnd"), W::t());
    let (id_attr, name_attr) = (W::name("id"), W::name("name"));
    let (col_first, col_last) = (W::name("colFirst"), W::name("colLast"));
    let mut text: Vec<char> = Vec::new();
    let mut with_text: HashSet<NodeId> = HashSet::new();
    let mut names: HashSet<String> = HashSet::new();
    let mut open: HashMap<String, Marker> = HashMap::new();
    let mut markers = Vec::new();
    for n in dom.descendant_nodes(body) {
        if dom.is_text(n) {
            if dom.parent(n).is_some_and(|leaf| dom.name_is(leaf, &t)) {
                text.extend(dom.text_value(n).unwrap_or("").chars());
                if let Some(p) = paragraph_of(&dom, n) {
                    with_text.insert(p);
                }
            }
            continue;
        }
        let Some(name) = dom.name(n) else { continue };
        let Some(id) = dom.attribute(n, &id_attr) else {
            continue;
        };
        if name == start {
            let Some(bm) = dom.attribute(n, &name_attr).filter(|s| !s.is_empty()) else {
                continue;
            };
            if bm == GO_BACK || !names.insert(bm.to_string()) {
                continue;
            }
            let cols = dom.attribute(n, &col_first).map(|first| {
                let last = dom.attribute(n, &col_last).unwrap_or(first);
                (first.to_string(), last.to_string())
            });
            let marker = Marker {
                name: bm.to_string(),
                start: text.len(),
                end: text.len(),
                leads: paragraph_of(&dom, n).is_none_or(|p| !with_text.contains(&p)),
                cols,
                in_sdt: !dom.ancestors(n, Some(&W::name("sdt"))).is_empty(),
            };
            open.insert(id.to_string(), marker);
        } else if name == end
            && let Some(mut marker) = open.remove(id)
        {
            marker.end = text.len();
            markers.push(marker);
        }
    }
    markers.extend(open.into_values());
    markers.sort_by_key(|m| m.start);
    (text, markers)
}

/// First occurrence of `needle` at or after `from`, wrapping to the start.
fn find_from(haystack: &[char], needle: &[char], from: usize) -> Option<usize> {
    let last = haystack.len().checked_sub(needle.len())?;
    let from = from.min(last);
    (from..=last)
        .chain(0..from)
        .find(|&i| haystack[i..i + needle.len()] == *needle)
}

/// Map source offset `o` onto the merged projection by the text around it,
/// with shrinking context windows, searching forward from `hint`. When the
/// text on both sides no longer meets (an edit right at the bookmark), the
/// side the marker belongs to decides alone: the text after a start (`after`),
/// the text before an end.
fn map_point(src: &[char], merged: &[char], o: usize, hint: usize, after: bool) -> Option<usize> {
    let windows = [(40usize, 40usize), (20, 20), (10, 10)];
    let one_sided = if after {
        [(0, 30), (0, 15)]
    } else {
        [(30, 0), (15, 0)]
    };
    for (before, behind) in windows.into_iter().chain(one_sided) {
        let pre = &src[o.saturating_sub(before)..o];
        let post = &src[o..(o + behind).min(src.len())];
        if pre.is_empty() && post.is_empty() {
            continue;
        }
        let needle = [pre, post].concat();
        if let Some(pos) = find_from(merged, &needle, hint.saturating_sub(pre.len())) {
            return Some(pos + pre.len());
        }
    }
    None
}

/// The outermost content control around `node`.
fn outermost_sdt(dom: &Dom, node: NodeId) -> Option<NodeId> {
    dom.ancestors(node, Some(&W::name("sdt"))).last().copied()
}

/// Move a placed pair out of the content controls its source bookmark was
/// not in: the start before its outermost control, the end after its own.
/// An empty pair moves as one, to the side it belongs to.
fn leave_foreign_controls(dom: &mut Dom, m: &Marker, start: NodeId, end: NodeId, empty: bool) {
    if m.in_sdt {
        return;
    }
    if empty {
        let Some(sdt) = outermost_sdt(dom, start) else {
            return;
        };
        dom.remove(start);
        dom.remove(end);
        if m.leads {
            dom.add_before_self(sdt, start);
        } else {
            dom.add_after_self(sdt, start);
        }
        dom.add_after_self(start, end);
        return;
    }
    if let Some(sdt) = outermost_sdt(dom, start) {
        dom.remove(start);
        dom.add_before_self(sdt, start);
    }
    if let Some(sdt) = outermost_sdt(dom, end) {
        dom.remove(end);
        dom.add_after_self(sdt, end);
    }
}

/// The `w:bookmarkStart`/`w:bookmarkEnd` pair for `m` under `id`.
fn new_markers(dom: &mut Dom, m: &Marker, id: u32) -> (NodeId, NodeId) {
    let id = id.to_string();
    let start = dom.new_element(W::name("bookmarkStart"));
    dom.set_attribute_value(start, &W::name("id"), Some(&id));
    dom.set_attribute_value(start, &W::name("name"), Some(&m.name));
    if let Some((first, last)) = &m.cols {
        dom.set_attribute_value(start, &W::name("colFirst"), Some(first));
        dom.set_attribute_value(start, &W::name("colLast"), Some(last));
    }
    let end = dom.new_element(W::name("bookmarkEnd"));
    dom.set_attribute_value(end, &W::name("id"), Some(&id));
    (start, end)
}

/// Place one side's bookmarks on the merged body's projection of that side.
fn inject_side(
    dom: &mut Dom,
    result_root: NodeId,
    (src, markers): (&[char], &[&Marker]),
    b_side: bool,
    author: &str,
    next_id: &mut u32,
) {
    if markers.is_empty() {
        return;
    }
    let (merged_text, mut segs): (String, Vec<Seg>) =
        collect_segments(dom, result_root, b_side, author, false);
    if segs.is_empty() {
        return; // no text on this side to anchor to
    }
    let merged: Vec<char> = merged_text.chars().collect();
    let identical = merged == src;
    let mut hint = 0usize;
    for m in markers {
        let (s, e) = if identical {
            (m.start, m.end)
        } else {
            let empty = m.end == m.start;
            let Some(s) = map_point(src, &merged, m.start, hint, !empty || m.leads) else {
                continue; // its text is gone from this side of the redline
            };
            hint = s;
            let e = if empty {
                s
            } else {
                map_point(src, &merged, m.end, s, false)
                    .filter(|&e| e >= s)
                    .unwrap_or(s)
            };
            (s, e)
        };
        let (start, end) = new_markers(dom, m, *next_id);
        *next_id += 1;
        if s < e {
            place_before_offset(dom, &mut segs, s, start);
            place_after_offset(dom, &mut segs, e, end);
        } else if m.leads {
            place_before_offset(dom, &mut segs, s, start);
            dom.add_after_self(start, end);
        } else {
            place_after_offset(dom, &mut segs, s, end);
            dom.add_before_self(end, start);
        }
        leave_foreign_controls(dom, m, start, end, s >= e);
    }
}

/// Drop the `_GoBack` bookmarks that rode in with opaque content (textboxes)
/// and renumber the others above every `w:id` in the body. Returns the names
/// still present and the next free id.
fn settle_present_bookmarks(dom: &mut Dom, result_root: NodeId) -> (HashSet<String>, u32) {
    let id_attr = W::name("id");
    let (start, end) = (W::name("bookmarkStart"), W::name("bookmarkEnd"));
    let mut present: HashSet<String> = HashSet::new();
    let mut markers: Vec<NodeId> = Vec::new();
    let mut go_back: HashSet<String> = HashSet::new();
    let mut max_id = 0u32;
    for e in dom.descendants(result_root, None) {
        let id = dom.attribute(e, &id_attr);
        if let Some(n) = id.and_then(|v| v.parse::<u32>().ok()) {
            max_id = max_id.max(n);
        }
        if dom.name_is(e, &start) {
            let name = dom.attribute(e, &W::name("name")).unwrap_or("");
            if name == GO_BACK {
                go_back.extend(id.map(str::to_string));
            } else {
                present.insert(name.to_string());
            }
            markers.push(e);
        } else if dom.name_is(e, &end) {
            markers.push(e);
        }
    }
    let mut next_id = max_id + 1;
    let mut renumbered: HashMap<String, String> = HashMap::new();
    for e in markers {
        let Some(old) = dom.attribute(e, &id_attr).map(str::to_string) else {
            continue;
        };
        if go_back.contains(&old) {
            dom.remove(e);
            continue;
        }
        let new = renumbered.entry(old).or_insert_with(|| {
            next_id += 1;
            (next_id - 1).to_string()
        });
        dom.set_attribute_value(e, &id_attr, Some(new.as_str()));
    }
    (present, next_id)
}

/// Entry point — run on the finished result body, before it is serialized.
/// Bookmark ids start above every `w:id` already in the body, so they never
/// share a value with a revision or comment id (Word keeps them disjoint).
pub fn carry_bookmarks(
    dom: &mut Dom,
    result_root: NodeId,
    (pkg1, main1): (&PartFs, &str),
    (pkg2, main2): (&PartFs, &str),
    author: &str,
) {
    carry_matching_bookmarks(
        dom,
        result_root,
        (pkg1, main1),
        (pkg2, main2),
        author,
        |_| true,
    );
}

/// [`carry_bookmarks`] for the bookmarks whose name passes `keep`, in the
/// body or in one header, footer or notes part (`result_root` is then that
/// part's root and `main1`/`main2` its name in each source).
pub fn carry_matching_bookmarks(
    dom: &mut Dom,
    result_root: NodeId,
    (pkg1, main1): (&PartFs, &str),
    (pkg2, main2): (&PartFs, &str),
    author: &str,
    keep: impl Fn(&str) -> bool,
) {
    let (a_text, mut a_marks) = extract(pkg1, main1);
    let (b_text, mut b_marks) = extract(pkg2, main2);
    a_marks.retain(|m| keep(&m.name));
    b_marks.retain(|m| keep(&m.name));
    let (present, mut next_id) = settle_present_bookmarks(dom, result_root);
    if a_marks.is_empty() && b_marks.is_empty() {
        return;
    }
    let b_names: HashSet<&str> = b_marks.iter().map(|m| m.name.as_str()).collect();
    let b_new: Vec<&Marker> = b_marks
        .iter()
        .filter(|m| !present.contains(&m.name))
        .collect();
    let a_only: Vec<&Marker> = a_marks
        .iter()
        .filter(|m| !b_names.contains(m.name.as_str()) && !present.contains(&m.name))
        .collect();
    inject_side(
        dom,
        result_root,
        (&b_text, &b_new),
        true,
        author,
        &mut next_id,
    );
    inject_side(
        dom,
        result_root,
        (&a_text, &a_only),
        false,
        author,
        &mut next_id,
    );
}
