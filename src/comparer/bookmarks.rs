// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Bookmark carryover into the redline body (and headers/footers/notes).
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
//!
//! Public entry points:
//! - [`carry_bookmarks`] — body-level carryover, used after the diff but
//!   before serialization;
//! - [`carry_matching_bookmarks`] — same, with a name predicate and applied
//!   to one part root at a time (body, header, footer, notes).

use std::collections::{HashMap, HashSet};

use super::comments::{Seg, collect_segments, place_after_offset, place_before_offset};
use crate::namespaces::W;
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

/// Word's hidden last-edit bookmark: its Compare never writes one.
const GO_BACK: &str = "_GoBack";

type EndpointAttributes = Vec<(XName, String)>;

/// One source range and its character interval in the source projection.
struct Marker {
    name: String,
    /// Permission endpoints retain every source attribute, including their ids.
    permission: Option<(EndpointAttributes, EndpointAttributes)>,
    start: usize,
    end: usize,
    /// The source contains an actual closing endpoint, rather than an orphan.
    paired: bool,
    /// No text precedes the start in its paragraph. An empty bookmark there
    /// belongs to the paragraph that follows it, not to the text before it.
    leads: bool,
    /// A nonempty start at its paragraph's text end retains the preceding
    /// side, after the adjacent run's outermost inline revision, if any.
    start_trails: bool,
    /// An end at its paragraph's text start retains the following side,
    /// before the adjacent run's outermost inline revision, if any.
    end_leads: bool,
    /// Text offsets cannot encode a body-level endpoint's paragraph gap.
    start_in_body: bool,
    end_in_body: bool,
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
fn extract(pkg: &PartFs, main: &str, permission: bool) -> (Vec<char>, Vec<Marker>) {
    let Some(xml) = pkg.part_string(main) else {
        return (Vec::new(), Vec::new());
    };
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    // The main part's body, or a header, footer or notes part's root.
    let Some(body) = dom.root(d).map(|r| dom.element(r, &W::body()).unwrap_or(r)) else {
        return (Vec::new(), Vec::new());
    };
    let (start, end) = if permission {
        (W::name("permStart"), W::name("permEnd"))
    } else {
        (W::name("bookmarkStart"), W::name("bookmarkEnd"))
    };
    let t = W::t();
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
            let Some(bm) = (if permission {
                Some(id)
            } else {
                dom.attribute(n, &name_attr)
            })
            .filter(|s| !s.is_empty()) else {
                continue;
            };
            if bm == GO_BACK || !names.insert(bm.to_string()) {
                continue;
            }
            let cols = dom.attribute(n, &col_first).map(|first| {
                let last = dom.attribute(n, &col_last).unwrap_or(first);
                (first.to_string(), last.to_string())
            });
            let paragraph = paragraph_of(&dom, n);
            let leads = paragraph.is_none_or(|p| !with_text.contains(&p));
            let start_trails = !leads
                && paragraph.is_some_and(|p| {
                    dom.descendant_nodes(p)
                        .into_iter()
                        .skip_while(|&node| node != n)
                        .skip(1)
                        .all(|node| !dom.name_is(node, &t) || dom.value(node).is_empty())
                });
            let marker = Marker {
                name: bm.to_string(),
                permission: permission.then(|| (dom.attributes(n), Vec::new())),
                start: text.len(),
                end: text.len(),
                paired: false,
                leads,
                start_trails,
                end_leads: false,
                start_in_body: dom.parent(n) == Some(body) && dom.name_is(body, &W::body()),
                end_in_body: false,
                cols,
                in_sdt: !dom.ancestors(n, Some(&W::name("sdt"))).is_empty(),
            };
            open.insert(id.to_string(), marker);
        } else if name == end
            && let Some(mut marker) = open.remove(id)
        {
            if let Some((_, attrs)) = &mut marker.permission {
                *attrs = dom.attributes(n);
            }
            marker.end = text.len();
            marker.paired = true;
            marker.end_leads = paragraph_of(&dom, n).is_some_and(|p| !with_text.contains(&p));
            marker.end_in_body = dom.parent(n) == Some(body) && dom.name_is(body, &W::body());
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
    if let Some((start_attrs, end_attrs)) = &m.permission {
        let start = dom.new_element(W::name("permStart"));
        dom.set_attributes(start, start_attrs.clone());
        let end = dom.new_element(W::name("permEnd"));
        dom.set_attributes(end, end_attrs.clone());
        if end_attrs.is_empty() {
            dom.set_attribute_value(end, &W::id(), Some(&m.name));
        }
        return (start, end);
    }
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

/// Keep a boundary endpoint beside the outermost inline revision containing
/// its adjacent run, on the same side. Never climb past its paragraph.
fn leave_inline_revision(dom: &mut Dom, endpoint: NodeId, after: bool) {
    let revisions = [W::ins(), W::del(), W::name("moveFrom"), W::name("moveTo")];
    let outermost = dom
        .ancestors(endpoint, None)
        .into_iter()
        .take_while(|&ancestor| !dom.name_is(ancestor, &W::p()))
        .filter(|&ancestor| revisions.iter().any(|name| dom.name_is(ancestor, name)))
        .last();
    if let Some(revision) = outermost {
        dom.remove(endpoint);
        if after {
            dom.add_after_self(revision, endpoint);
        } else {
            dom.add_before_self(revision, endpoint);
        }
    }
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
            let Some(s) = map_point(
                src,
                &merged,
                m.start,
                hint,
                if empty { m.leads } else { !m.start_trails },
            ) else {
                continue; // its text is gone from this side of the redline
            };
            hint = s;
            let e = if empty {
                s
            } else {
                map_point(src, &merged, m.end, s, m.end_leads)
                    .filter(|&e| e >= s)
                    .unwrap_or(s)
            };
            (s, e)
        };
        let (start, end) = new_markers(dom, m, *next_id);
        *next_id += 1;
        if s < e {
            // Text offsets alone cannot distinguish the two paragraph sides
            // of a boundary. Restore the source side of a trailing start or
            // leading end, then leave any enclosing inline revisions on that
            // same side, without climbing above the paragraph.
            if m.start_trails {
                place_after_offset(dom, &mut segs, s, start);
                leave_inline_revision(dom, start, true);
            } else {
                place_before_offset(dom, &mut segs, s, start);
            }
            if m.end_leads {
                place_before_offset(dom, &mut segs, e, end);
                leave_inline_revision(dom, end, false);
            } else {
                place_after_offset(dom, &mut segs, e, end);
            }
        } else if m.leads {
            place_before_offset(dom, &mut segs, s, start);
            dom.add_after_self(start, end);
        } else {
            place_after_offset(dom, &mut segs, s, end);
            dom.add_before_self(end, start);
        }
        leave_foreign_controls(dom, m, start, end, s >= e);
        // A body endpoint belongs beside the paragraph, even when its
        // adjacent text now sits in an inline insertion. Offset placement
        // alone would pull it into that paragraph (and that revision).
        for (endpoint, in_body, after) in
            [(start, m.start_in_body, false), (end, m.end_in_body, true)]
        {
            if in_body && let Some(paragraph) = paragraph_of(dom, endpoint) {
                dom.remove(endpoint);
                if after {
                    dom.add_after_self(paragraph, endpoint);
                } else {
                    dom.add_before_self(paragraph, endpoint);
                }
            }
        }
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

/// Carry both source documents' bookmarks into the redline body, then
/// renumber ids above every `w:id` already present so they do not collide
/// with revisions or comments (Word keeps bookmark and revision ids
/// disjoint).
///
/// Word's Compare keeps the union of both documents' body bookmarks by
/// name so a redline's `PAGEREF`/`REF`/`HYPERLINK` fields still resolve.
/// Call after the diff produced `result_root` and before it is serialized.
/// Backs the `--keep-bookmarks` CLI flag path (the comparer post-process
/// step that wires `[`super::comments`]-style anchors into the merged body).
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

/// [`carry_bookmarks`] for the bookmarks whose name passes `keep`, applied
/// to one part root (the body, or a single header, footer or notes part).
///
/// `result_root` is that part's root in the diff result; `main1`/`main2` are
/// the same part's name in each source package. `keep` is the name predicate
/// the caller supplies (e.g. drop `_GoBack` for opaque text-box content).
pub fn carry_matching_bookmarks(
    dom: &mut Dom,
    result_root: NodeId,
    (pkg1, main1): (&PartFs, &str),
    (pkg2, main2): (&PartFs, &str),
    author: &str,
    keep: impl Fn(&str) -> bool,
) {
    let (a_text, mut a_marks) = extract(pkg1, main1, false);
    let (b_text, mut b_marks) = extract(pkg2, main2, false);
    a_marks.retain(|m| keep(&m.name));
    b_marks.retain(|m| keep(&m.name));
    // A surviving start is not an empty range when its original end was
    // deleted. Keep the original pair on the projection containing that text.
    b_marks.retain(|b| b.paired || !a_marks.iter().any(|a| a.name == b.name && a.paired));
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

/// Edit-only permission carry-over. Reuse bookmark projection and content-control
/// boundaries, preferring the revised pair by id and retaining original-only
/// pairs on the deleted side. Compare deliberately does not call this.
pub(crate) fn carry_permissions(
    dom: &mut Dom,
    root: NodeId,
    original: (&PartFs, &str),
    revised: (&PartFs, &str),
    author: &str,
) {
    let (a_text, a_marks) = extract(original.0, original.1, true);
    let (b_text, mut b_marks) = extract(revised.0, revised.1, true);
    b_marks.retain(|b| b.paired || !a_marks.iter().any(|a| a.name == b.name && a.paired));
    let b_ids: HashSet<&str> = b_marks.iter().map(|m| m.name.as_str()).collect();
    let present: HashSet<String> = dom
        .descendants(root, Some(&W::name("permStart")))
        .into_iter()
        .filter_map(|n| dom.attribute(n, &W::id()).map(str::to_string))
        .collect();
    let b_new: Vec<&Marker> = b_marks
        .iter()
        .filter(|m| !present.contains(&m.name))
        .collect();
    let a_only: Vec<&Marker> = a_marks
        .iter()
        .filter(|m| !b_ids.contains(m.name.as_str()) && !present.contains(&m.name))
        .collect();
    inject_side(dom, root, (&b_text, &b_new), true, author, &mut 0);
    inject_side(dom, root, (&a_text, &a_only), false, author, &mut 0);
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn check_boundary_revision(trailing_start: bool) {
        for (outer, inner, b_side, leaf) in [
            ("ins", "moveTo", true, "t"),
            ("moveTo", "ins", true, "t"),
            ("del", "moveFrom", false, "delText"),
            ("moveFrom", "del", false, "delText"),
        ] {
            let revised = format!(
                r#"<w:{outer} w:author="Editor"><w:{inner} w:author="Editor"><w:r><w:{leaf}>{}</w:{leaf}></w:r></w:{inner}></w:{outer}>"#,
                if trailing_start { "foo" } else { "bar" }
            );
            let body = if trailing_start {
                format!("<w:p>{revised}</w:p><w:p><w:r><w:t>bar</w:t></w:r></w:p>")
            } else {
                format!("<w:p><w:r><w:t>foo</w:t></w:r></w:p><w:p>{revised}</w:p>")
            };
            let mut dom = Dom::new();
            // The block-level revision above the paragraphs must remain an
            // ancestor: only inline revision containers are left behind.
            let doc = dom.parse_xdocument(&format!(
                r#"<w:body xmlns:w="{}"><w:{outer} w:author="Editor">{body}</w:{outer}></w:body>"#,
                W::URI
            ));
            let root = dom.root(doc).unwrap();
            let paragraphs = dom.descendants(root, Some(&W::p()));
            let owner = paragraphs[usize::from(!trailing_start)];
            let container = dom.elements(owner, None)[0];
            let marker = Marker {
                name: "Clause".into(),
                permission: None,
                paired: true,
                start: if trailing_start { 3 } else { 0 },
                end: if trailing_start { 6 } else { 3 },
                leads: !trailing_start,
                start_trails: trailing_start,
                end_leads: !trailing_start,
                start_in_body: false,
                end_in_body: false,
                cols: None,
                in_sdt: false,
            };
            inject_side(
                &mut dom,
                root,
                (&"foobar".chars().collect::<Vec<_>>(), &[&marker]),
                b_side,
                "Editor",
                &mut 10,
            );
            let name = if trailing_start {
                W::bookmark_start()
            } else {
                W::bookmark_end()
            };
            let endpoint = dom.descendants(root, Some(&name))[0];
            assert_eq!(
                dom.parent(endpoint),
                Some(owner),
                "{outer}/{inner}: endpoint must sit beside the outermost inline revision"
            );
            let children = dom.elements(owner, None);
            assert_eq!(
                children,
                if trailing_start {
                    vec![container, endpoint]
                } else {
                    vec![endpoint, container]
                },
                "source boundary side must be retained"
            );
        }
    }

    #[test]
    fn trailing_start_leaves_outermost_inline_revision_on_same_side() {
        check_boundary_revision(true);
    }

    #[test]
    fn leading_end_leaves_outermost_inline_revision_on_same_side() {
        check_boundary_revision(false);
    }
}
