//! M35 — comments carryover (word mode, settings-gated).
//!
//! Word's Compare carries comments through the redline
//! (parity/_scratch/comments_carryover_forensics.md):
//!   1. Parts: union of both sides' `w:comment` sets. When B's set ⊇ A's,
//!      B's four comment parts are emitted byte-identical; when only one side
//!      has comments, that side's are carried.
//!   2. Anchors (`commentRangeStart`/`commentRangeEnd`/`commentReference`)
//!      are re-emitted at the equivalent text position in the merged body and
//!      survive del/ins wrapping (GT keeps anchors around `w:delText` inside
//!      `w:del`).
//!   3. Id collisions between A-only and B-only comments are renumbered
//!      consistently (commentsExtended is keyed by paraId, not comment id).
//!   4. Never an orphaned comments part: a comment whose anchors can't be
//!      carried is dropped from the part.
//!
//! The anchor pass is a character-offset projection, not an atomize
//! flow-through: the merged body's non-deleted text equals B's text and its
//! non-inserted text (plain `w:t` + comparer `w:delText`) equals A's, so each
//! side's anchors are re-injected by character position, located by context
//! matching (robust to content loss elsewhere); an unmappable range falls to
//! rule 4.

use std::collections::{HashMap, HashSet};

use crate::namespaces::W;
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

/// (part name, content type, relationship type) for the comment part family.
const FAMILY: [(&str, &str, &str); 4] = [
    (
        "word/comments.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
    ),
    (
        "word/commentsExtended.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml",
        "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
    ),
    (
        "word/commentsIds.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml",
        "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds",
    ),
    (
        "word/commentsExtensible.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtensible+xml",
        "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible",
    ),
];

fn comment_ids_of(pkg: &PartFs) -> HashSet<String> {
    let Some(xml) = pkg.part_string("word/comments.xml") else {
        return HashSet::new();
    };
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(d) else {
        return HashSet::new();
    };
    dom.elements(root, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| dom.attribute(c, &W::name("id")).map(str::to_string))
        .collect()
}

/// (id → concatenated w:t text) for every comment in a package's comments.xml.
/// Used to tell a genuine comment superset (B carries A's same-id same-text
/// comments plus its own) from a numeric-id superset where the shared ids are
/// DIFFERENT comments authored independently — the latter must go through the
/// collision-renumbering union path, not the byte-identical fast path.
fn comment_id_text_of(pkg: &PartFs) -> HashMap<String, String> {
    let Some(xml) = pkg.part_string("word/comments.xml") else {
        return HashMap::new();
    };
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(d) else {
        return HashMap::new();
    };
    dom.elements(root, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| {
            dom.attribute(c, &W::name("id")).map(|id| {
                let text: String = dom
                    .descendants(c, Some(&W::t()))
                    .into_iter()
                    .map(|t| dom.value(t))
                    .collect();
                (id.to_string(), text)
            })
        })
        .collect()
}

/// True when B carries every one of A's comments by BOTH id and text — the
/// condition under which B's comment parts can be emitted byte-identical. A
/// bare numeric-id superset is NOT sufficient: two independently-authored
/// comments can share an id (commonly 0) with different bodies, and treating
/// that as a superset would silently drop A's comment (PR #81 review).
fn b_carries_same_comments_as_a(pkg1: &PartFs, pkg2: &PartFs) -> bool {
    let a = comment_id_text_of(pkg1);
    if a.is_empty() {
        return true;
    }
    let b = comment_id_text_of(pkg2);
    a.iter().all(|(id, text)| b.get(id) == Some(text))
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Start,
    End,
}

struct Event {
    offset: usize,
    kind: Kind,
    id: String,
}

/// A comment's [start, end) character interval in its source projection.
struct Range {
    id: String,
    start: usize,
    end: usize,
}

/// Extract anchor ranges + the projection text from a source document's main
/// part. Counted text = every `w:t` character in body order (the inputs are
/// post-PreProcessMarkup, i.e. revisions accepted — no live `w:delText`).
fn extract_events(pkg: &PartFs, main: &str) -> Option<(String, Vec<Range>)> {
    let xml = pkg.part_string(main)?;
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d)?;
    let body = dom.element(root, &W::body())?;
    let start = W::name("commentRangeStart");
    let end = W::name("commentRangeEnd");
    let mut text = String::new();
    let mut offset = 0usize;
    let mut starts: HashMap<String, usize> = HashMap::new();
    let mut ranges: Vec<Range> = Vec::new();
    for n in dom.descendant_nodes(body) {
        if dom.is_element(n) {
            let name = dom.name(n).unwrap();
            if name == start {
                if let Some(id) = dom.attribute(n, &W::name("id")) {
                    starts.entry(id.to_string()).or_insert(offset);
                }
            } else if name == end
                && let Some(id) = dom.attribute(n, &W::name("id"))
                && let Some(s) = starts.get(id)
            {
                ranges.push(Range {
                    id: id.to_string(),
                    start: *s,
                    end: offset,
                });
            }
        } else if dom.is_text(n)
            && dom
                .parent(n)
                .and_then(|p| dom.name(p))
                .is_some_and(|pn| pn == W::t())
        {
            let t = dom.text_value(n).unwrap_or("");
            text.push_str(t);
            offset += t.chars().count();
        }
    }
    Some((text, ranges))
}

/// Map a source-projection range onto the merged projection by context
/// matching: search for `pre + inner + post` with shrinking context windows,
/// then bare `inner`. Tolerates content loss elsewhere in the document (the
/// whole-document offsets need not line up). Returns merged (start, end).
fn map_range(src: &[char], merged: &[char], r: &Range) -> Option<(usize, usize)> {
    let inner = &src[r.start..r.end.min(src.len())];
    for ctx in [40usize, 20, 10, 0] {
        if ctx == 0 && inner.is_empty() {
            return None; // a zero-length range needs context to place
        }
        let pre = &src[r.start.saturating_sub(ctx)..r.start];
        let post = &src[r.end.min(src.len())..(r.end + ctx).min(src.len())];
        let mut needle: Vec<char> = Vec::with_capacity(pre.len() + inner.len() + post.len());
        needle.extend_from_slice(pre);
        needle.extend_from_slice(inner);
        needle.extend_from_slice(post);
        if needle.is_empty() {
            continue;
        }
        if let Some(pos) = find_chars(merged, &needle) {
            let s = pos + pre.len();
            return Some((s, s + inner.len()));
        }
    }
    None
}

fn find_chars(haystack: &[char], needle: &[char]) -> Option<usize> {
    if needle.len() > haystack.len() {
        return None;
    }
    (0..=haystack.len() - needle.len()).find(|&i| haystack[i..i + needle.len()] == *needle)
}

/// A counted text leaf in the merged body: the `w:t`/`w:delText` element, its
/// run, and its [start, start+len) character interval in the projection.
struct Seg {
    leaf: NodeId,
    run: NodeId,
    start: usize,
    len: usize,
}

/// Collect the merged body's counted leaves for one side's projection.
/// B side: every `w:t` not inside any `w:del` (= B's visible text).
/// A side: `w:t` not inside any `w:ins` + `w:delText` inside a `w:del`
/// authored by the comparer (foreign carried revisions are B's, not A's).
fn collect_segments(
    dom: &Dom,
    result_root: NodeId,
    b_side: bool,
    author: &str,
) -> (String, Vec<Seg>) {
    let Some(body) = dom.element(result_root, &W::body()) else {
        return (String::new(), Vec::new());
    };
    let del_text = W::name("delText");
    let mut text = String::new();
    let mut segs = Vec::new();
    let mut offset = 0usize;
    for n in dom.descendant_nodes(body) {
        if !dom.is_text(n) {
            continue;
        }
        let Some(leaf) = dom.parent(n) else { continue };
        let Some(leaf_name) = dom.name(leaf) else {
            continue;
        };
        let counted = if b_side {
            leaf_name == W::t() && !has_ancestor(dom, leaf, &W::del(), None)
        } else if leaf_name == W::t() {
            !has_ancestor(dom, leaf, &W::ins(), None)
        } else {
            leaf_name == del_text && has_ancestor(dom, leaf, &W::del(), Some(author))
        };
        if !counted {
            continue;
        }
        let Some(run) = dom.parent(leaf) else {
            continue;
        };
        let t = dom.text_value(n).unwrap_or("");
        let len = t.chars().count();
        text.push_str(t);
        segs.push(Seg {
            leaf,
            run,
            start: offset,
            len,
        });
        offset += len;
    }
    (text, segs)
}

fn has_ancestor(dom: &Dom, node: NodeId, name: &XName, author: Option<&str>) -> bool {
    dom.ancestors(node, Some(name))
        .into_iter()
        .any(|a| author.is_none_or(|au| dom.attribute(a, &W::author()).unwrap_or("") == au))
}

/// Split `segs[i]` after `k` characters (0 < k < len): the leaf keeps the
/// prefix; a fresh run (cloned rPr, plus the leaf's trailing run siblings)
/// takes the suffix. `segs[i]` becomes the prefix seg; the suffix seg is
/// inserted after it.
fn split_seg(dom: &mut Dom, segs: &mut Vec<Seg>, i: usize, k: usize) {
    let seg = &segs[i];
    let (leaf, run, start) = (seg.leaf, seg.run, seg.start);
    let text = dom.value(leaf);
    let split_byte = text
        .char_indices()
        .nth(k)
        .map(|(b, _)| b)
        .unwrap_or(text.len());
    let (prefix, suffix) = text.split_at(split_byte);
    let (prefix, suffix) = (prefix.to_string(), suffix.to_string());
    let leaf_name = dom.name(leaf).unwrap();
    let space = XNamespace::xml().name("space");

    dom.remove_nodes(leaf);
    dom.add_text(leaf, &prefix);
    dom.set_attribute_value(leaf, &space, Some("preserve"));

    let new_run = dom.new_element(W::r());
    if let Some(rpr) = dom.element(run, &W::r_pr()) {
        let c = dom.clone_subtree(rpr);
        dom.add(new_run, c);
    }
    let new_leaf = dom.new_element(leaf_name);
    dom.set_attribute_value(new_leaf, &space, Some("preserve"));
    let new_text = dom.new_text(&suffix);
    dom.add(new_leaf, new_text);
    dom.add(new_run, new_leaf);
    // trailing siblings of the leaf stay after the split point
    let mut after = false;
    for c in dom.nodes(run) {
        if c == leaf {
            after = true;
            continue;
        }
        if after {
            dom.remove(c);
            dom.add(new_run, c);
        }
    }
    dom.add_after_self(run, new_run);

    let old_len = segs[i].len;
    segs[i].len = k;
    segs.insert(
        i + 1,
        Seg {
            leaf: new_leaf,
            run: new_run,
            start: start + k,
            len: old_len - k,
        },
    );
}

fn new_anchor(dom: &mut Dom, kind: Kind, id: &str) -> NodeId {
    let name = match kind {
        Kind::Start => W::name("commentRangeStart"),
        Kind::End => W::name("commentRangeEnd"),
    };
    let e = dom.new_element(name);
    dom.set_attribute_value(e, &W::name("id"), Some(id));
    e
}

/// The GT reference-run shape: `w:rStyle CommentReference` + `w:commentReference`.
fn new_reference_run(dom: &mut Dom, id: &str) -> NodeId {
    let r = dom.new_element(W::r());
    let rpr = dom.new_element(W::r_pr());
    let style = dom.new_element(W::name("rStyle"));
    dom.set_attribute_value(style, &W::val(), Some("CommentReference"));
    dom.add(rpr, style);
    dom.add(r, rpr);
    let cref = dom.new_element(W::name("commentReference"));
    dom.set_attribute_value(cref, &W::name("id"), Some(id));
    dom.add(r, cref);
    r
}

/// Inject one side's anchor events into the merged body. Returns the ids that
/// were anchored (unmappable ranges are skipped — orphan cleanup drops them).
#[allow(clippy::too_many_arguments)]
fn inject_side(
    dom: &mut Dom,
    result_root: NodeId,
    src_pkg: &PartFs,
    src_main: &str,
    b_side: bool,
    author: &str,
    id_map: &HashMap<String, String>,
    only_ids: Option<&HashSet<String>>,
) -> HashSet<String> {
    let Some((src_text, ranges)) = extract_events(src_pkg, src_main) else {
        return HashSet::new();
    };
    if ranges.is_empty() {
        return HashSet::new();
    }
    let (merged_text, mut segs) = collect_segments(dom, result_root, b_side, author);
    let src_chars: Vec<char> = src_text.chars().collect();
    let merged_chars: Vec<char> = merged_text.chars().collect();

    // map each comment range through context matching, then flatten to
    // events sorted by (offset, source order) so nesting order is preserved
    let mut events: Vec<Event> = Vec::new();
    for r in &ranges {
        if let Some(only) = only_ids
            && !only.contains(&r.id)
        {
            continue;
        }
        let Some((s, e)) = map_range(&src_chars, &merged_chars, r) else {
            continue; // unmappable — the comment falls to orphan cleanup
        };
        events.push(Event {
            offset: s,
            kind: Kind::Start,
            id: r.id.clone(),
        });
        events.push(Event {
            offset: e,
            kind: Kind::End,
            id: r.id.clone(),
        });
    }
    let mut order: Vec<usize> = (0..events.len()).collect();
    order.sort_by_key(|&i| (events[i].offset, i));

    let mut anchored = HashSet::new();
    for idx in order {
        let ev = &events[idx];
        let out_id = id_map.get(&ev.id).cloned().unwrap_or_else(|| ev.id.clone());
        let o = ev.offset;
        match ev.kind {
            Kind::Start => {
                let anchor = new_anchor(dom, Kind::Start, &out_id);
                match segs.iter().position(|s| s.start + s.len > o) {
                    None => {
                        if let Some(last) = segs.last() {
                            dom.add_after_self(last.run, anchor);
                        }
                    }
                    Some(i) if segs[i].start >= o => {
                        dom.add_before_self(segs[i].run, anchor);
                    }
                    Some(i) => {
                        let k = o - segs[i].start;
                        split_seg(dom, &mut segs, i, k);
                        dom.add_before_self(segs[i + 1].run, anchor);
                    }
                }
            }
            Kind::End => {
                let anchor = new_anchor(dom, Kind::End, &out_id);
                match segs.iter().rposition(|s| s.start < o) {
                    None => {
                        if let Some(first) = segs.first() {
                            dom.add_before_self(first.run, anchor);
                        }
                    }
                    Some(i) if segs[i].start + segs[i].len <= o => {
                        dom.add_after_self(segs[i].run, anchor);
                    }
                    Some(i) => {
                        let k = o - segs[i].start;
                        split_seg(dom, &mut segs, i, k);
                        dom.add_after_self(segs[i].run, anchor);
                    }
                }
                let refrun = new_reference_run(dom, &out_id);
                dom.add_after_self(anchor, refrun);
                anchored.insert(out_id);
            }
        }
    }
    anchored
}

/// Copy `src`'s comment family into `out` (overwriting), wire content-type
/// overrides + main-document rels, and drop any family part `src` lacks.
fn install_parts_from(out: &mut PartFs, out_main: &str, src: &PartFs) {
    for (part, ct, rel_type) in FAMILY {
        match src.part_bytes(part).map(<[u8]>::to_vec) {
            Some(bytes) => {
                out.set_part(part, bytes);
                out.add_content_type_override(&format!("/{part}"), ct);
                let has_rel = out
                    .read_rels_for(out_main)
                    .is_some_and(|r| r.items.iter().any(|i| i.rel_type == rel_type));
                if !has_rel {
                    let target = part.strip_prefix("word/").unwrap_or(part);
                    out.add_document_relationship(out_main, rel_type, target);
                }
            }
            None => remove_family_part(out, out_main, part, rel_type),
        }
    }
}

fn remove_family_part(out: &mut PartFs, out_main: &str, part: &str, rel_type: &str) {
    out.remove_part(part);
    out.remove_content_type_override(&format!("/{part}"));
    out.remove_relationships_by_type(out_main, rel_type);
}

/// Merge A's comments into a B-based comments.xml for the union case,
/// renumbering A ids that collide with B's. Returns the A→out id map.
fn union_comments_xml(out: &mut PartFs, out_main: &str, pkg1: &PartFs) -> HashMap<String, String> {
    let mut id_map = HashMap::new();
    let (Some(bx), Some(ax)) = (
        out.part_string("word/comments.xml"),
        pkg1.part_string("word/comments.xml"),
    ) else {
        return id_map;
    };
    let mut dom = Dom::new();
    let bd = dom.parse_xdocument(&bx);
    let ad = dom.parse_xdocument(&ax);
    let (Some(br), Some(ar)) = (dom.root(bd), dom.root(ad)) else {
        return id_map;
    };
    let id_name = W::name("id");
    let b_ids: HashSet<String> = dom
        .elements(br, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| dom.attribute(c, &id_name).map(str::to_string))
        .collect();
    let mut next_id = b_ids
        .iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let b_comment_text: HashMap<String, String> = dom
        .elements(br, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| {
            dom.attribute(c, &id_name)
                .map(|id| (id.to_string(), dom.value(c)))
        })
        .collect();
    // A's own ids that will be KEPT as-is (non-colliding, distinct-text comments).
    // A renumber must not hand out an id in this set either, or two output
    // comments end up sharing an id (Word then treats them as one comment and
    // the anchor lookup becomes ambiguous). Concrete case: B ids {0,1,2}, A ids
    // {0,1,2,3,4} → renumber hands out 3,4,5 for A's 0,1,2 collisions, then A's
    // own 3,4 are kept verbatim and collide with the renumbers.
    let a_kept_ids: HashSet<String> = dom
        .elements(ar, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| dom.attribute(c, &id_name).map(str::to_string))
        .filter(|id| !b_ids.contains(id))
        .collect();
    // Reserved id set the renumber must avoid: B's ids plus A's kept ids.
    let mut reserved: HashSet<String> = b_ids.clone();
    reserved.extend(a_kept_ids.iter().cloned());
    for c in dom.elements(ar, Some(&W::name("comment"))) {
        let Some(id) = dom.attribute(c, &id_name).map(str::to_string) else {
            continue;
        };
        if b_comment_text.get(&id) == Some(&dom.value(c)) {
            continue; // same comment carried on both sides; B's copy wins
        }
        let clone = dom.clone_subtree(c);
        if b_ids.contains(&id) {
            // id collision with a DIFFERENT B comment — renumber A's copy to a
            // free id (not in B, not kept by another A comment, not already
            // handed out to a prior renumber).
            while reserved.contains(&next_id.to_string()) {
                next_id += 1;
            }
            let new_id = next_id.to_string();
            next_id += 1;
            reserved.insert(new_id.clone());
            dom.set_attribute_value(clone, &id_name, Some(&new_id));
            id_map.insert(id.clone(), new_id);
        } else {
            id_map.insert(id.clone(), id.clone());
        }
        dom.add(br, clone);
    }
    out.set_part("word/comments.xml", dom.serialize_element(br).into_bytes());
    // aux parts: append A entries whose paraId key is absent from B's.
    // When B lacks an aux part entirely, `install_parts_from(out, pkg2)` has
    // already removed it from `out` — seed the part from A first so A-only
    // comments keep their commentsExtended/Ids/Extensible metadata (PR #81).
    for (part, ct, rel_type) in &FAMILY[1..] {
        let Some(ax) = pkg1.part_string(part) else {
            continue;
        };
        if out.part_string(part).is_none() {
            out.set_part(part, ax.into_bytes());
            out.add_content_type_override(&format!("/{part}"), ct);
            let has_rel = out
                .read_rels_for(out_main)
                .is_some_and(|r| r.items.iter().any(|i| i.rel_type == *rel_type));
            if !has_rel {
                let target = part.strip_prefix("word/").unwrap_or(part);
                out.add_document_relationship(out_main, rel_type, target);
            }
            continue; // fully seeded from A; nothing further to merge
        }
        let Some(bx) = out.part_string(part) else {
            continue;
        };
        let mut d = Dom::new();
        let bd = d.parse_xdocument(&bx);
        let ad = d.parse_xdocument(&ax);
        let (Some(br), Some(ar)) = (d.root(bd), d.root(ad)) else {
            continue;
        };
        let para_key = |d: &Dom, e: NodeId| -> Option<String> {
            d.attributes(e)
                .into_iter()
                .find(|(n, _)| n.local_name() == "paraId")
                .map(|(_, v)| v)
        };
        let existing: HashSet<String> = d
            .elements(br, None)
            .into_iter()
            .filter_map(|e| para_key(&d, e))
            .collect();
        let mut changed = false;
        for e in d.elements(ar, None) {
            if let Some(k) = para_key(&d, e)
                && !existing.contains(&k)
            {
                let c = d.clone_subtree(e);
                d.add(br, c);
                changed = true;
            }
        }
        if changed {
            out.set_part(part, d.serialize_element(br).into_bytes());
        }
    }
    id_map
}

/// Drop unanchored comments from the part family; if none remain, remove the
/// family entirely (rule 4 — no orphaned parts).
fn drop_orphans(out: &mut PartFs, out_main: &str, anchored: &HashSet<String>) {
    let Some(xml) = out.part_string("word/comments.xml") else {
        return;
    };
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(d) else { return };
    let comments = dom.elements(root, Some(&W::name("comment")));
    let orphan: Vec<NodeId> = comments
        .iter()
        .copied()
        .filter(|&c| {
            dom.attribute(c, &W::name("id"))
                .is_none_or(|id| !anchored.contains(id))
        })
        .collect();
    if orphan.is_empty() {
        return;
    }
    if orphan.len() == comments.len() {
        for (part, _, rel_type) in FAMILY {
            remove_family_part(out, out_main, part, rel_type);
        }
        return;
    }
    // paraIds of the removed comments' paragraphs key the aux-part entries
    let mut dead_para_ids: HashSet<String> = HashSet::new();
    for &c in &orphan {
        for p in dom.descendants(c, Some(&W::p())) {
            for (n, v) in dom.attributes(p) {
                if n.local_name() == "paraId" {
                    dead_para_ids.insert(v);
                }
            }
        }
        dom.remove(c);
    }
    out.set_part(
        "word/comments.xml",
        dom.serialize_element(root).into_bytes(),
    );
    for (part, _, _) in &FAMILY[1..] {
        let Some(px) = out.part_string(part) else {
            continue;
        };
        let mut d2 = Dom::new();
        let pd = d2.parse_xdocument(&px);
        let Some(pr) = d2.root(pd) else { continue };
        let mut changed = false;
        for e in d2.elements(pr, None) {
            let dead = d2
                .attributes(e)
                .into_iter()
                .any(|(n, v)| n.local_name() == "paraId" && dead_para_ids.contains(&v));
            if dead {
                d2.remove(e);
                changed = true;
            }
        }
        if changed {
            out.set_part(part, d2.serialize_element(pr).into_bytes());
        }
    }
}

/// Entry point — run after the diff produced `result_root` but BEFORE it is
/// serialized into `out` (anchors are injected into the result DOM).
#[allow(clippy::too_many_arguments)]
pub fn carry_comments(
    dom: &mut Dom,
    result_root: NodeId,
    pkg1: &PartFs,
    main1: &str,
    pkg2: &PartFs,
    main2: &str,
    out: &mut PartFs,
    out_main: &str,
    author: &str,
) {
    let ids_a = comment_ids_of(pkg1);
    let ids_b = comment_ids_of(pkg2);
    if ids_a.is_empty() && ids_b.is_empty() {
        return;
    }
    let no_map = HashMap::new();
    let anchored = if ids_b.is_empty() {
        // only A has comments; its parts are already in out (out is A's clone)
        inject_side(dom, result_root, pkg1, main1, false, author, &no_map, None)
    } else if ids_a.is_empty() || b_carries_same_comments_as_a(pkg1, pkg2) {
        // B carries the union — parts byte-identical from B (gated on matching
        // id AND text for every A comment; a bare numeric-id superset is not
        // enough — same-id independently-authored comments would be dropped).
        install_parts_from(out, out_main, pkg2);
        inject_side(dom, result_root, pkg2, main2, true, author, &no_map, None)
    } else {
        // true union: B's parts as base + A-only comments appended
        install_parts_from(out, out_main, pkg2);
        let id_map = union_comments_xml(out, out_main, pkg1);
        let mut anchored = inject_side(dom, result_root, pkg2, main2, true, author, &no_map, None);
        let a_only: HashSet<String> = id_map.keys().cloned().collect();
        anchored.extend(inject_side(
            dom,
            result_root,
            pkg1,
            main1,
            false,
            author,
            &id_map,
            Some(&a_only),
        ));
        anchored
    };
    drop_orphans(out, out_main, &anchored);
}
