// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M35 — comments carryover for every comparer preset.
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

use crate::namespaces::{MC, W, W14};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

/// (part name, content type, relationship type) for the comment part family.
pub(crate) const FAMILY: [(&str, &str, &str); 4] = [
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

fn comment_definition_fingerprint(dom: &Dom, comment: NodeId) -> String {
    let body: String = dom
        .descendants(comment, Some(&W::t()))
        .into_iter()
        .map(|text| dom.value(text))
        .collect();
    let author = dom.attribute(comment, &W::author()).unwrap_or("");
    let date = dom.attribute(comment, &W::date()).unwrap_or("");
    let initials = dom.attribute(comment, &W::name("initials")).unwrap_or("");
    format!(
        "{}\u{0}{author}\u{0}{date}\u{0}{initials}",
        normalized_text(&body)
    )
}

/// (id → definition fingerprint) for every comment. The author, timestamp,
/// and initials are part of logical identity; body text alone is not.
fn comment_id_fingerprint_of(pkg: &PartFs) -> HashMap<String, String> {
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
            dom.attribute(c, &W::name("id"))
                .map(|id| (id.to_string(), comment_definition_fingerprint(&dom, c)))
        })
        .collect()
}

/// True when B carries every one of A's comments by both id and definition
/// fingerprint — the condition under which B's parts can be emitted
/// byte-identical. A numeric-id superset is not sufficient.
fn b_carries_same_comments_as_a(pkg1: &PartFs, pkg2: &PartFs) -> bool {
    let a = comment_id_fingerprint_of(pkg1);
    if a.is_empty() {
        return true;
    }
    let b = comment_id_fingerprint_of(pkg2);
    a.iter()
        .all(|(id, fingerprint)| b.get(id) == Some(fingerprint))
}

fn normalized_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Build an id-independent identity from the comment body and its anchored
/// source context. Body text alone is unsafe: two distinct comments can say
/// the same thing. The bounded pre/inner/post projection distinguishes their
/// logical locations while still matching Word-renumbered copies.
fn comment_anchor_identities(pkg: &PartFs, main: &str) -> HashMap<String, String> {
    let definitions = comment_id_fingerprint_of(pkg);
    let Some((text, ranges)) = extract_events(pkg, main) else {
        return definitions
            .into_iter()
            .map(|(id, definition)| {
                let identity = format!("{definition}\u{0}<unanchored:{id}>");
                (id, identity)
            })
            .collect();
    };
    let chars: Vec<char> = text.chars().collect();
    let ranges_by_id: HashMap<&str, &Range> = ranges
        .iter()
        .map(|range| (range.id.as_str(), range))
        .collect();
    let mut candidates = FingerprintGroups::new();
    for (id, definition) in definitions {
        let Some(range) = ranges_by_id.get(id.as_str()) else {
            candidates
                .entry(definition)
                .or_default()
                .push((id, (usize::MAX, usize::MAX)));
            continue;
        };
        candidates
            .entry(definition)
            .or_default()
            .push((id, (range.start, range.end)));
    }

    let mut identities = HashMap::new();
    for (definition, group) in candidates {
        let has_nonempty = group.iter().any(|(_, (start, end))| end > start);
        let mut seen_anchors = HashSet::new();
        for (id, (raw_start, raw_end)) in group {
            let unanchored = raw_start == usize::MAX;
            let start = raw_start.min(chars.len());
            let end = raw_end.min(chars.len()).max(start);
            if has_nonempty && start == end {
                continue;
            }
            let anchor = if unanchored {
                format!("<unanchored:{id}>")
            } else {
                let pre: String = chars[start.saturating_sub(40)..start].iter().collect();
                let inner: String = chars[start..end].iter().collect();
                let post: String = chars[end..(end + 40).min(chars.len())].iter().collect();
                format!(
                    "{}\u{0}{}\u{0}{}",
                    normalized_text(&pre),
                    normalized_text(&inner),
                    normalized_text(&post)
                )
            };
            if seen_anchors.insert(anchor.clone()) {
                identities.insert(id, format!("{definition}\u{0}{anchor}"));
            }
        }
    }
    identities
}

/// True when B's multiset of anchored comment identities covers A's. Word can
/// renumber a comment set across sequential redline sources, so ids cannot be
/// the key; body-only matching is equally unsafe because repeated prose is
/// common in review comments.
fn b_covers_comment_identities_of_a(
    pkg1: &PartFs,
    main1: &str,
    pkg2: &PartFs,
    main2: &str,
) -> bool {
    let a = comment_anchor_identities(pkg1, main1);
    if a.is_empty() {
        return true;
    }
    let b = comment_anchor_identities(pkg2, main2);
    let mut b_counts: HashMap<String, usize> = HashMap::new();
    for identity in b.values() {
        *b_counts.entry(identity.clone()).or_default() += 1;
    }
    for identity in a.values() {
        match b_counts.get_mut(identity) {
            Some(n) if *n > 0 => *n -= 1,
            _ => return false,
        }
    }
    true
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Start,
    End,
    /// An empty range: start, end and reference together, at the end's place
    /// (a start placed on its own would land in the next paragraph).
    Point,
}

struct Event {
    offset: usize,
    kind: Kind,
    id: String,
    /// Document order of the range's start in its source.
    seq: usize,
}

/// A comment's [start, end) character interval in its source projection.
struct Range {
    id: String,
    start: usize,
    end: usize,
    /// Document order of the start among the source's starts.
    seq: usize,
}

/// The paragraph mark in a projection: a marker after it sits in a later
/// paragraph than the text before it (an end past a table stays past it).
const MARK: char = '\n';

/// Extract anchor ranges + the projection text from a source document's main
/// part. Counted text = every `w:t` character in body order, each paragraph
/// closed by a [`MARK`] (the inputs are post-PreProcessMarkup, i.e.
/// revisions accepted — no live `w:delText`).
fn extract_events(pkg: &PartFs, main: &str) -> Option<(String, Vec<Range>)> {
    let xml = pkg.part_string(main)?;
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d)?;
    let body = dom.element(root, &W::body())?;
    let start = W::name("commentRangeStart");
    let end = W::name("commentRangeEnd");
    let reference = W::name("commentReference");
    let mut references: Vec<(String, usize)> = Vec::new();
    let mut text = String::new();
    let mut offset = 0usize;
    let mut starts: HashMap<String, (usize, usize)> = HashMap::new();
    let mut ranges: Vec<Range> = Vec::new();
    let mut open: Vec<NodeId> = Vec::new();
    for n in dom.descendant_nodes(body) {
        while open
            .last()
            .is_some_and(|&p| !dom.ancestors(n, None).contains(&p))
        {
            open.pop();
            text.push(MARK);
            offset += 1;
        }
        if dom.name_is(n, &W::p()) {
            open.push(n);
        }
        if dom.is_element(n) {
            let name = dom.name(n).unwrap();
            if name == start {
                if let Some(id) = dom.attribute(n, &W::name("id")) {
                    let seq = starts.len();
                    starts.entry(id.to_string()).or_insert((offset, seq));
                }
            } else if name == end
                && let Some(id) = dom.attribute(n, &W::name("id"))
                && let Some(&(s, seq)) = starts.get(id)
            {
                ranges.push(Range {
                    id: id.to_string(),
                    start: s,
                    end: offset,
                    seq,
                });
            } else if name == reference
                && let Some(id) = dom.attribute(n, &W::name("id"))
            {
                references.push((id.to_string(), offset));
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
    text.extend(open.iter().map(|_| MARK));
    // A reference with no range markers is a point comment: an empty range
    // where the reference sits, which is how Word's redline writes it.
    for (id, at) in references {
        if !starts.contains_key(&id) && !ranges.iter().any(|r| r.id == id) {
            ranges.push(Range {
                id,
                start: at,
                end: at,
                seq: usize::MAX, // no start marker
            });
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
        if let Some(pos) =
            find_chars_nearest(merged, &needle, expected(src, merged, r.start, pre.len()))
        {
            let s = pos + pre.len();
            return Some((s, s + inner.len()));
        }
    }
    map_range_ends(src, merged, r)
}

/// Where a needle starting `pre` chars before source offset `at` should sit
/// in `merged`, scaled by the two projections' lengths. Repeated text (a
/// copied section under its own comments) maps to the copy at its own place.
fn expected(src: &[char], merged: &[char], at: usize, pre: usize) -> usize {
    let scaled = if src.is_empty() {
        0
    } else {
        (at as u128 * merged.len() as u128 / src.len() as u128) as usize
    };
    scaled.saturating_sub(pre)
}

/// Longest head or tail of a range that anchors one of its ends.
const END_ANCHOR_CHARS: usize = 40;

/// Map a long range whose text changed inside (a comment over several
/// paragraphs and tables): its start by context + head, its end by tail +
/// context found after the start. The mapped span must stay within half to
/// twice the source span, plus the anchors' slack.
fn map_range_ends(src: &[char], merged: &[char], r: &Range) -> Option<(usize, usize)> {
    let end = r.end.min(src.len());
    let inner = &src[r.start..end];
    if inner.len() <= 2 * END_ANCHOR_CHARS {
        return None;
    }
    let head = &inner[..END_ANCHOR_CHARS];
    let tail = &inner[inner.len() - END_ANCHOR_CHARS..];
    for ctx in [40usize, 20, 10] {
        let pre = &src[r.start.saturating_sub(ctx)..r.start];
        let post = &src[end..(end + ctx).min(src.len())];
        let start_needle: Vec<char> = pre.iter().chain(head).copied().collect();
        let end_needle: Vec<char> = tail.iter().chain(post).copied().collect();
        let target = expected(src, merged, r.start, pre.len());
        let Some(pos) = find_chars_nearest(merged, &start_needle, target) else {
            continue;
        };
        let s = pos + pre.len();
        let Some(tail_at) = find_chars_from(merged, &end_needle, s + head.len()) else {
            continue;
        };
        let e = tail_at + tail.len();
        let span = e - s;
        let slack = 2 * END_ANCHOR_CHARS;
        if span + slack >= inner.len() / 2 && span <= 2 * inner.len() + slack {
            return Some((s, e));
        }
    }
    None
}

/// The occurrence of `needle` nearest to `target` (the earlier one on a tie).
fn find_chars_nearest(haystack: &[char], needle: &[char], target: usize) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut from = 0;
    while let Some(i) = find_chars_from(haystack, needle, from) {
        if best.is_none_or(|b| i.abs_diff(target) < b.abs_diff(target)) {
            best = Some(i);
        }
        if i >= target {
            break;
        }
        from = i + 1;
    }
    best
}

fn find_chars_from(haystack: &[char], needle: &[char], from: usize) -> Option<usize> {
    if needle.len() > haystack.len() || from > haystack.len() - needle.len() {
        return None;
    }
    (from..=haystack.len() - needle.len()).find(|&i| haystack[i..i + needle.len()] == *needle)
}

/// A counted text leaf in the merged body: the `w:t`/`w:delText` element, its
/// run, and its [start, start+len) character interval in the projection.
pub(super) struct Seg {
    leaf: NodeId,
    run: NodeId,
    start: usize,
    len: usize,
}

impl Seg {
    /// A paragraph's [`MARK`]: `leaf` and `run` are the paragraph.
    fn is_mark(&self) -> bool {
        self.leaf == self.run
    }
}

/// Whether a paragraph's mark is on this side: not inserted on A's, not
/// deleted on B's (a moved mark counts at its own end).
fn mark_on_side(dom: &Dom, p: NodeId, b_side: bool) -> bool {
    let changes: &[&str] = if b_side {
        &["del", "moveFrom"]
    } else {
        &["ins", "moveTo"]
    };
    !dom.element(p, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| {
            changes
                .iter()
                .any(|c| dom.element(rpr, &W::name(c)).is_some())
        })
}

/// Collect the merged body's counted leaves for one side's projection.
/// B side: every `w:t` not inside any `w:del` (= B's visible text).
/// A side: `w:t` not inside any `w:ins` + `w:delText` inside a `w:del`
/// authored by the comparer (foreign carried revisions are B's, not A's).
/// With `marks`, each paragraph whose mark is on the side closes with a
/// [`MARK`] segment, as [`extract_events`] projects its source.
pub(super) fn collect_segments(
    dom: &Dom,
    result_root: NodeId,
    b_side: bool,
    author: &str,
    marks: bool,
) -> (String, Vec<Seg>) {
    // The document's body, or a header, footer or notes part's root.
    let body = dom.element(result_root, &W::body()).unwrap_or(result_root);
    let del_text = W::name("delText");
    let (move_from, move_to) = (W::name("moveFrom"), W::name("moveTo"));
    let mut text = String::new();
    let mut segs = Vec::new();
    let mut offset = 0usize;
    let mut open: Vec<NodeId> = Vec::new();
    let close = |p: NodeId, text: &mut String, segs: &mut Vec<Seg>, offset: &mut usize| {
        if mark_on_side(dom, p, b_side) {
            text.push(MARK);
            segs.push(Seg {
                leaf: p,
                run: p,
                start: *offset,
                len: 1,
            });
            *offset += 1;
        }
    };
    for n in dom.descendant_nodes(body) {
        if marks {
            while let Some(&p) = open.last()
                && !dom.ancestors(n, None).contains(&p)
            {
                open.pop();
                close(p, &mut text, &mut segs, &mut offset);
            }
            if dom.name_is(n, &W::p()) {
                open.push(n);
            }
        }
        if !dom.is_text(n) {
            continue;
        }
        let Some(leaf) = dom.parent(n) else { continue };
        let Some(leaf_name) = dom.name(leaf) else {
            continue;
        };
        // moved text is A's at its source (moveFrom), B's at its destination
        let counted = if b_side {
            leaf_name == W::t()
                && !has_ancestor(dom, leaf, &W::del(), None)
                && !has_ancestor(dom, leaf, &move_from, None)
        } else if leaf_name == W::t() {
            !has_ancestor(dom, leaf, &W::ins(), None) && !has_ancestor(dom, leaf, &move_to, None)
        } else {
            leaf_name == del_text
                && (has_ancestor(dom, leaf, &W::del(), Some(author))
                    || has_ancestor(dom, leaf, &move_from, Some(author)))
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
    while let Some(p) = open.pop() {
        close(p, &mut text, &mut segs, &mut offset);
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
    // later leaves of the old run moved with the suffix
    for later in &mut segs[i + 1..] {
        if later.run == run {
            later.run = new_run;
        }
    }

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

/// Start a fresh run (cloned rPr) at `segs[i]`'s leaf when an earlier
/// sibling shares its run, so a node placed before that run sits between
/// the two leaves ("ação 🐋" and "東京" of one run, each followed by its own
/// point comment).
fn split_run_before_leaf(dom: &mut Dom, segs: &mut [Seg], i: usize) {
    if segs[i].is_mark() {
        return;
    }
    let (leaf, run) = (segs[i].leaf, segs[i].run);
    let has_earlier = dom
        .nodes(run)
        .into_iter()
        .take_while(|&c| c != leaf)
        .any(|c| !dom.name_is(c, &W::r_pr()));
    if !has_earlier {
        return;
    }
    let new_run = dom.new_element(W::r());
    if let Some(rpr) = dom.element(run, &W::r_pr()) {
        let c = dom.clone_subtree(rpr);
        dom.add(new_run, c);
    }
    for c in dom.nodes(run).into_iter().skip_while(|&c| c != leaf) {
        dom.remove(c);
        dom.add(new_run, c);
    }
    dom.add_after_self(run, new_run);
    for later in &mut segs[i..] {
        if later.run == run {
            later.run = new_run;
        }
    }
}

/// Put `node` just before the character at merged offset `o`, splitting the
/// run that holds it; past the last character, after the last run.
pub(super) fn place_before_offset(dom: &mut Dom, segs: &mut Vec<Seg>, o: usize, node: NodeId) {
    // segments are contiguous in projection order: both keys are monotonic
    let i = segs.partition_point(|s| s.start + s.len <= o);
    match (i < segs.len()).then_some(i) {
        None => match segs.last() {
            // past the last mark: the end of that paragraph
            Some(last) if last.is_mark() => dom.add(last.leaf, node),
            Some(last) => dom.add_after_self(last.run, node),
            None => {}
        },
        // before a mark: the end of its paragraph
        Some(i) if segs[i].is_mark() => dom.add(segs[i].leaf, node),
        Some(i) if segs[i].start >= o => {
            split_run_before_leaf(dom, segs, i);
            dom.add_before_self(segs[i].run, node);
        }
        Some(i) => {
            let k = o - segs[i].start;
            split_seg(dom, segs, i, k);
            dom.add_before_self(segs[i + 1].run, node);
        }
    }
}

/// Put `node` just after the character ending at merged offset `o`, splitting
/// the run that holds it; before any character, before the first run.
pub(super) fn place_after_offset(dom: &mut Dom, segs: &mut Vec<Seg>, o: usize, node: NodeId) {
    match segs.partition_point(|s| s.start < o).checked_sub(1) {
        None => {
            if let Some(first) = segs.first() {
                dom.add_before_self(first.run, node);
            }
        }
        // after a mark: the start of the next paragraph's content, or the
        // end of the last paragraph
        Some(i) if segs[i].is_mark() => match segs.get(i + 1) {
            Some(next) if next.is_mark() => {
                let p = next.leaf;
                // A paragraph whose content opens with a deletion takes the
                // end inside it, as Word writes a range's end in deleted text.
                let content = dom
                    .elements(p, None)
                    .into_iter()
                    .find(|&c| dom.name(c) != Some(W::p_pr()));
                // A range ending on a deleted mark, before a paragraph with
                // no content and a deleted mark, ends in a deletion of its
                // own there (5f0fed8e2a).
                let ends_deleted = mark_deletion(dom, segs[i].leaf).is_some();
                let content = match (content, mark_deletion(dom, p)) {
                    (None, Some(mark)) if ends_deleted => {
                        let del = dom.clone_subtree(mark);
                        dom.add(p, del);
                        Some(del)
                    }
                    (c, _) => c,
                };
                match (content, dom.element(p, &W::p_pr())) {
                    (Some(del), _) if dom.name(del) == Some(W::del()) => dom.add_first(del, node),
                    (_, Some(ppr)) => dom.add_after_self(ppr, node),
                    (_, None) => dom.add_first(p, node),
                }
            }
            Some(_) => {
                split_run_before_leaf(dom, segs, i + 1);
                dom.add_before_self(segs[i + 1].run, node);
            }
            None => dom.add(segs[i].leaf, node),
        },
        Some(i) if segs[i].start + segs[i].len <= o => {
            // A later leaf of the same run starts after `o`: split there.
            if segs.get(i + 1).is_some_and(|n| n.run == segs[i].run) {
                split_run_before_leaf(dom, segs, i + 1);
                dom.add_before_self(segs[i + 1].run, node);
            } else {
                dom.add_after_self(segs[i].run, node);
            }
        }
        Some(i) => {
            let k = o - segs[i].start;
            split_seg(dom, segs, i, k);
            dom.add_after_self(segs[i].run, node);
        }
    }
}

fn new_anchor(dom: &mut Dom, start: bool, id: &str) -> NodeId {
    let name = if start {
        W::name("commentRangeStart")
    } else {
        W::name("commentRangeEnd")
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

/// Mapped output intervals keyed by the final comment id. Unmappable comments
/// are absent and therefore fall to orphan cleanup.
type AnchorInterval = (usize, usize);
type AnchoredRanges = HashMap<String, AnchorInterval>;
type FingerprintGroups = HashMap<String, Vec<(String, AnchorInterval)>>;

/// Inject one side's anchor events into the merged body. Returns the ids and
/// mapped intervals that were anchored.
fn inject_side(
    dom: &mut Dom,
    result_root: NodeId,
    (src_pkg, src_main): (&PartFs, &str),
    b_side: bool,
    author: &str,
    id_map: &HashMap<String, String>,
    only_ids: Option<&HashSet<String>>,
) -> AnchoredRanges {
    let Some((src_text, ranges)) = extract_events(src_pkg, src_main) else {
        return HashMap::new();
    };
    if ranges.is_empty() {
        return HashMap::new();
    }
    let (merged_text, mut segs) = collect_segments(dom, result_root, b_side, author, true);
    let src_chars: Vec<char> = src_text.chars().collect();
    let merged_chars: Vec<char> = merged_text.chars().collect();
    // map each comment range through context matching, then flatten to
    // events sorted by (offset, source order) so nesting order is preserved
    let mut events: Vec<Event> = Vec::new();
    let mut anchored_ranges = HashMap::new();
    for r in &ranges {
        if let Some(only) = only_ids
            && !only.contains(&r.id)
        {
            continue;
        }
        let Some((s, e)) = map_range(&src_chars, &merged_chars, r) else {
            continue; // unmappable — the comment falls to orphan cleanup
        };
        let out_id = id_map.get(&r.id).cloned().unwrap_or_else(|| r.id.clone());
        anchored_ranges.insert(out_id, (s, e));
        if s == e {
            events.push(Event {
                offset: e,
                kind: Kind::Point,
                id: r.id.clone(),
                seq: r.seq,
            });
            continue;
        }
        events.push(Event {
            offset: s,
            kind: Kind::Start,
            id: r.id.clone(),
            seq: r.seq,
        });
        events.push(Event {
            offset: e,
            kind: Kind::End,
            id: r.id.clone(),
            seq: r.seq,
        });
    }
    let mut order: Vec<usize> = (0..events.len()).collect();
    order.sort_by_key(|&i| (events[i].offset, i));
    // Ranges come in the order they end; starts at one place follow the
    // order they start in (01f3deda92's nested comments).
    for run in order.chunk_by_mut(|&a, &b| {
        let (a, b) = (&events[a], &events[b]);
        a.kind == Kind::Start && b.kind == Kind::Start && a.offset == b.offset
    }) {
        run.sort_by_key(|&i| events[i].seq);
    }

    // The reference run of the last end placed, by offset: a second end at
    // the same place follows it, so the source order holds.
    let mut last_end: Option<(usize, NodeId)> = None;
    for idx in order {
        let ev = &events[idx];
        let out_id = id_map.get(&ev.id).cloned().unwrap_or_else(|| ev.id.clone());
        let o = ev.offset;
        match ev.kind {
            Kind::Start => {
                let anchor = new_anchor(dom, true, &out_id);
                place_before_offset(dom, &mut segs, o, anchor);
            }
            Kind::End | Kind::Point => {
                let anchor = new_anchor(dom, false, &out_id);
                match last_end {
                    Some((at, prev)) if at == o => dom.add_after_self(prev, anchor),
                    _ => place_after_offset(dom, &mut segs, o, anchor),
                }
                if ev.kind == Kind::Point {
                    let start = new_anchor(dom, true, &out_id);
                    dom.add_before_self(anchor, start);
                }
                let refrun = new_reference_run(dom, &out_id);
                dom.add_after_self(anchor, refrun);
                if ev.kind == Kind::Point {
                    step_out_of_deletion_end(dom, anchor);
                }
                last_end = Some((o, refrun));
            }
        }
    }
    anchored_ranges
}

/// The `w:del` recording a paragraph's deleted mark.
fn mark_deletion(dom: &Dom, p: NodeId) -> Option<NodeId> {
    dom.element(p, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .and_then(|rpr| dom.element(rpr, &W::del()))
}

/// A point comment (no range) that lands at the end of deleted text sits
/// just after the deletion in Word's redline, so accepting it keeps the
/// comment (145b9e67e2, ff27140d0a). A range comment's reference stays in
/// the deletion it ends in, as Word writes it (11 of 11 in Word's redlines).
fn step_out_of_deletion_end(dom: &mut Dom, end: NodeId) {
    let Some(del) = dom.parent(end).filter(|&d| dom.name(d) == Some(W::del())) else {
        return;
    };
    let marker = |dom: &Dom, n: NodeId| {
        dom.name(n).is_some_and(|nm| {
            nm == W::name("commentRangeStart")
                || nm == W::name("commentRangeEnd")
                || (nm == W::r()
                    && dom.elements(n, None).into_iter().all(|c| {
                        dom.name(c)
                            .is_some_and(|cn| cn == W::r_pr() || cn == W::name("commentReference"))
                    }))
        })
    };
    let kids = dom.elements(del, None);
    let Some(at) = kids.iter().position(|&k| k == end) else {
        return;
    };
    if !kids[at..].iter().all(|&k| marker(dom, k)) {
        return;
    }
    // The point's own start marker precedes its end; move the markers from
    // there on, in order, after the deletion.
    let from = kids[..at]
        .iter()
        .rposition(|&k| dom.name(k) != Some(W::name("commentRangeStart")))
        .map_or(0, |i| i + 1);
    let moving: Vec<NodeId> = kids[from..].to_vec();
    let mut after = del;
    for n in moving {
        dom.remove(n);
        dom.add_after_self(after, n);
        after = n;
    }
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
                    let target = crate::opc::relative_rel_target(out_main, part);
                    out.add_document_relationship(out_main, rel_type, &target);
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

fn allocate_para_id(used: &mut HashSet<String>, next: &mut u32) -> String {
    loop {
        if *next == 0 || *next >= 0x8000_0000 {
            *next = 1;
        }
        let candidate = format!("{:08X}", *next);
        *next += 1;
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
}

fn allocate_durable_id(used: &mut HashSet<String>, next: &mut u32) -> String {
    loop {
        if *next == 0 {
            *next = 1;
        }
        let candidate = format!("{:08X}", *next);
        *next = next.wrapping_add(1);
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
}

fn rewrite_para_id_references(dom: &mut Dom, root: NodeId, map: &HashMap<String, String>) {
    if map.is_empty() {
        return;
    }
    for element in dom.descendants_and_self(root, None) {
        for (name, value) in dom.attributes(element) {
            if matches!(name.local_name(), "paraId" | "paraIdParent")
                && let Some(replacement) = map.get(&value.to_ascii_uppercase())
            {
                dom.set_attribute_value(element, &name, Some(replacement));
            }
        }
    }
}

fn rewrite_durable_id_references(dom: &mut Dom, root: NodeId, map: &HashMap<String, String>) {
    if map.is_empty() {
        return;
    }
    for element in dom.descendants_and_self(root, None) {
        for (name, value) in dom.attributes(element) {
            if name.local_name() == "durableId"
                && let Some(replacement) = map.get(&value.to_ascii_uppercase())
            {
                dom.set_attribute_value(element, &name, Some(replacement));
            }
        }
    }
}

fn namespace_declarations(dom: &Dom, root: NodeId) -> HashMap<String, String> {
    dom.attributes(root)
        .into_iter()
        .filter(|(name, _)| dom.is_namespace_declaration(name))
        .map(|(name, value)| (name.local_name().to_string(), value))
        .collect()
}

/// The shared MC prefix-list test, for an element that may have no name.
fn is_namespace_qname_list(element: Option<&XName>, name: &XName) -> bool {
    element
        .is_some_and(|element| crate::xmllinq::serialize::is_namespace_prefix_list(element, name))
}

fn qname_token_prefix(token: &str) -> &str {
    token.split_once(':').map_or(token, |(prefix, _)| prefix)
}

fn rewrite_qname_token(token: &str, rewrites: &HashMap<String, String>) -> String {
    let prefix = qname_token_prefix(token);
    let Some(replacement) = rewrites.get(prefix) else {
        return token.to_string();
    };
    token.strip_prefix(prefix).map_or_else(
        || replacement.clone(),
        |suffix| format!("{replacement}{suffix}"),
    )
}

/// A cloned element does not carry namespace declarations inherited from its
/// source part root. Preserve the bindings referenced by MCE QName-list values
/// and the `mc:Ignorable` contract for extension namespaces used in the clone.
/// Conflicting destination prefixes are rebound under a fresh prefix and the
/// QName-list tokens are rewritten consistently.
fn preserve_cloned_namespace_context(
    dom: &mut Dom,
    source_root: NodeId,
    destination_root: NodeId,
    clone: NodeId,
) {
    let source_bindings = namespace_declarations(dom, source_root);
    let mut destination_bindings = namespace_declarations(dom, destination_root);
    let mut used_uris = HashSet::new();
    let mut required_prefixes = HashSet::new();

    for element in dom.descendants_and_self(clone, None) {
        if let Some(name) = dom.name(element)
            && !name.namespace_name().is_empty()
        {
            used_uris.insert(name.namespace_name().to_string());
        }
        let element_name = dom.name(element);
        for (name, value) in dom.attributes(element) {
            if !dom.is_namespace_declaration(&name) && !name.namespace_name().is_empty() {
                used_uris.insert(name.namespace_name().to_string());
            }
            if is_namespace_qname_list(element_name.as_ref(), &name) {
                required_prefixes.extend(
                    value
                        .split_whitespace()
                        .map(qname_token_prefix)
                        .map(str::to_string),
                );
            }
        }
    }

    let source_ignorable: Vec<String> = dom
        .attribute(source_root, &MC::name("Ignorable"))
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    required_prefixes.extend(
        source_ignorable
            .iter()
            .filter(|prefix| {
                source_bindings
                    .get(*prefix)
                    .is_some_and(|uri| used_uris.contains(uri))
            })
            .cloned(),
    );

    let mut required_prefixes: Vec<String> = required_prefixes.into_iter().collect();
    required_prefixes.sort();
    let mut rewrites = HashMap::new();
    for prefix in required_prefixes {
        let Some(uri) = source_bindings.get(&prefix) else {
            continue;
        };
        let chosen = if destination_bindings
            .get(&prefix)
            .is_none_or(|bound_uri| bound_uri == uri)
        {
            prefix.clone()
        } else if let Some(existing) = destination_bindings
            .iter()
            .filter(|(_, bound_uri)| *bound_uri == uri)
            .map(|(bound_prefix, _)| bound_prefix)
            .min()
        {
            existing.clone()
        } else {
            let mut index = 0usize;
            loop {
                let candidate = format!("ns{index}");
                if !destination_bindings.contains_key(&candidate) {
                    break candidate;
                }
                index += 1;
            }
        };
        if destination_bindings.get(&chosen) != Some(uri) {
            dom.set_attribute_value(
                destination_root,
                &XNamespace::xmlns().name(&chosen),
                Some(uri),
            );
            destination_bindings.insert(chosen.clone(), uri.clone());
        }
        if chosen != prefix {
            rewrites.insert(prefix, chosen);
        }
    }

    if !rewrites.is_empty() {
        for element in dom.descendants_and_self(clone, None) {
            let element_name = dom.name(element);
            for (name, value) in dom.attributes(element) {
                if !is_namespace_qname_list(element_name.as_ref(), &name) {
                    continue;
                }
                let rewritten = value
                    .split_whitespace()
                    .map(|token| rewrite_qname_token(token, &rewrites))
                    .collect::<Vec<_>>()
                    .join(" ");
                dom.set_attribute_value(element, &name, Some(&rewritten));
            }
        }
    }

    let mut destination_ignorable: Vec<String> = dom
        .attribute(destination_root, &MC::name("Ignorable"))
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    for prefix in source_ignorable {
        let Some(uri) = source_bindings.get(&prefix) else {
            continue;
        };
        if !used_uris.contains(uri) {
            continue;
        }
        let chosen = rewrites.get(&prefix).unwrap_or(&prefix);
        if !destination_ignorable.contains(chosen) {
            destination_ignorable.push(chosen.clone());
        }
    }
    if !destination_ignorable.is_empty() {
        dom.set_attribute_value(
            destination_root,
            &MC::name("Ignorable"),
            Some(&destination_ignorable.join(" ")),
        );
    }
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
    let mut used_para_ids: HashSet<String> = dom
        .descendants(br, Some(&W::p()))
        .into_iter()
        .filter_map(|p| dom.attribute(p, &W14::name("paraId")).map(str::to_string))
        .map(|value| value.to_ascii_uppercase())
        .collect();
    let mut next_para_id = used_para_ids
        .iter()
        .filter_map(|value| u32::from_str_radix(value, 16).ok())
        .filter(|value| *value < 0x8000_0000)
        .max()
        .map_or(1, |value| value.saturating_add(1));
    let mut para_id_map: HashMap<String, String> = HashMap::new();
    let mut next_id = b_ids
        .iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let b_comment_fingerprints: HashMap<String, String> = dom
        .elements(br, Some(&W::name("comment")))
        .into_iter()
        .filter_map(|c| {
            dom.attribute(c, &id_name)
                .map(|id| (id.to_string(), comment_definition_fingerprint(&dom, c)))
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
        if b_comment_fingerprints.get(&id) == Some(&comment_definition_fingerprint(&dom, c)) {
            continue; // same comment carried on both sides; B's copy wins
        }
        let clone = dom.clone_subtree(c);
        preserve_cloned_namespace_context(&mut dom, ar, br, clone);
        for paragraph in dom.descendants(clone, Some(&W::p())) {
            let Some(para_id) = dom
                .attribute(paragraph, &W14::name("paraId"))
                .map(str::to_string)
            else {
                continue;
            };
            let para_id_key = para_id.to_ascii_uppercase();
            if used_para_ids.contains(&para_id_key) {
                let replacement = para_id_map
                    .entry(para_id_key)
                    .or_insert_with(|| allocate_para_id(&mut used_para_ids, &mut next_para_id));
                dom.set_attribute_value(paragraph, &W14::name("paraId"), Some(replacement));
            } else {
                used_para_ids.insert(para_id_key);
            }
        }
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
    let mut durable_id_map: HashMap<String, String> = HashMap::new();
    for (part, ct, rel_type) in &FAMILY[1..] {
        let Some(ax) = pkg1.part_string(part) else {
            continue;
        };
        let is_comments_ids = *part == "word/commentsIds.xml";
        let is_comments_extensible = *part == "word/commentsExtensible.xml";
        if out.part_string(part).is_none() {
            let mut d = Dom::new();
            let ad = d.parse_xdocument(&ax);
            let Some(ar) = d.root(ad) else {
                continue;
            };
            rewrite_para_id_references(&mut d, ar, &para_id_map);
            if is_comments_extensible {
                rewrite_durable_id_references(&mut d, ar, &durable_id_map);
            }
            out.set_part(part, d.serialize_element(ar).into_bytes());
            out.add_content_type_override(&format!("/{part}"), ct);
            let has_rel = out
                .read_rels_for(out_main)
                .is_some_and(|r| r.items.iter().any(|i| i.rel_type == *rel_type));
            if !has_rel {
                let target = crate::opc::relative_rel_target(out_main, part);
                out.add_document_relationship(out_main, rel_type, &target);
            }
            continue; // fully seeded from A; nothing further to merge
        }
        let Some(bx) = out.part_string(part) else {
            continue;
        };
        let mut d = Dom::new();
        let ad = d.parse_xdocument(&ax);
        let Some(ar) = d.root(ad) else {
            continue;
        };
        rewrite_para_id_references(&mut d, ar, &para_id_map);
        if is_comments_extensible {
            rewrite_durable_id_references(&mut d, ar, &durable_id_map);
        }
        let bd = d.parse_xdocument(&bx);
        let Some(br) = d.root(bd) else {
            continue;
        };
        let key_local_name = if is_comments_extensible {
            "durableId"
        } else {
            "paraId"
        };
        let entry_key = |d: &Dom, e: NodeId| -> Option<String> {
            d.attributes(e)
                .into_iter()
                .find(|(n, _)| n.local_name() == key_local_name)
                .map(|(_, v)| v)
        };
        let mut existing: HashSet<String> = d
            .elements(br, None)
            .into_iter()
            .filter_map(|e| entry_key(&d, e))
            .map(|value| value.to_ascii_uppercase())
            .collect();
        let mut used_durable_ids: HashSet<String> = if is_comments_ids {
            d.elements(br, None)
                .into_iter()
                .filter_map(|e| {
                    d.attributes(e)
                        .into_iter()
                        .find(|(name, _)| name.local_name() == "durableId")
                        .map(|(_, value)| value.to_ascii_uppercase())
                })
                .collect()
        } else {
            HashSet::new()
        };
        let mut next_durable_id = used_durable_ids
            .iter()
            .filter_map(|value| u32::from_str_radix(value, 16).ok())
            .max()
            .map_or(1, |value| value.checked_add(1).unwrap_or(1));
        let mut changed = false;
        for e in d.elements(ar, None) {
            if let Some(k) = entry_key(&d, e)
                && !existing.contains(&k.to_ascii_uppercase())
            {
                let c = d.clone_subtree(e);
                preserve_cloned_namespace_context(&mut d, ar, br, c);
                if is_comments_ids
                    && let Some((durable_name, durable_id)) = d
                        .attributes(c)
                        .into_iter()
                        .find(|(name, _)| name.local_name() == "durableId")
                {
                    let durable_key = durable_id.to_ascii_uppercase();
                    if used_durable_ids.contains(&durable_key) {
                        let replacement = durable_id_map.entry(durable_key).or_insert_with(|| {
                            allocate_durable_id(&mut used_durable_ids, &mut next_durable_id)
                        });
                        d.set_attribute_value(c, &durable_name, Some(replacement));
                    } else {
                        used_durable_ids.insert(durable_key);
                    }
                }
                d.add(br, c);
                existing.insert(k.to_ascii_uppercase());
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
                    dead_para_ids.insert(v.to_ascii_uppercase());
                }
            }
        }
        dom.remove(c);
    }
    out.set_part(
        "word/comments.xml",
        dom.serialize_element(root).into_bytes(),
    );
    let mut dead_durable_ids: HashSet<String> = HashSet::new();
    for (part, _, _) in &FAMILY[1..] {
        let Some(px) = out.part_string(part) else {
            continue;
        };
        let mut d2 = Dom::new();
        let pd = d2.parse_xdocument(&px);
        let Some(pr) = d2.root(pd) else { continue };
        let mut changed = false;
        for e in d2.elements(pr, None) {
            let attributes = d2.attributes(e);
            let dead = attributes.iter().any(|(n, v)| {
                n.local_name() == "paraId" && dead_para_ids.contains(&v.to_ascii_uppercase())
            });
            let dead_by_durable_id = attributes.iter().any(|(name, value)| {
                name.local_name() == "durableId"
                    && dead_durable_ids.contains(&value.to_ascii_uppercase())
            });
            if dead || dead_by_durable_id {
                if *part == "word/commentsIds.xml" {
                    dead_durable_ids.extend(
                        attributes
                            .iter()
                            .filter(|(name, _)| name.local_name() == "durableId")
                            .map(|(_, value)| value.to_ascii_uppercase()),
                    );
                }
                d2.remove(e);
                changed = true;
                continue;
            }
            for (name, value) in attributes {
                if name.local_name() == "paraIdParent"
                    && dead_para_ids.contains(&value.to_ascii_uppercase())
                {
                    d2.set_attribute_value(e, &name, None);
                    changed = true;
                }
            }
        }
        if changed {
            out.set_part(part, d2.serialize_element(pr).into_bytes());
        }
    }
}

/// Select comments using both their definition fingerprint and mapped anchor.
/// Equal bodies on distinct non-empty ranges are independent comments, and so
/// are equal bodies one document stacks on one range (Word keeps each). An
/// A-side comment (`a_side`, the union's appended ids) whose body and range
/// equal a B-side one is B's copy and collapses into it; a live non-empty
/// anchor supersedes a stale zero-length revision copy of the same comment.
fn select_anchor_aware_comments(
    out: &PartFs,
    anchored: &AnchoredRanges,
    a_side: &HashSet<String>,
) -> HashSet<String> {
    let Some(xml) = out.part_string("word/comments.xml") else {
        return anchored.keys().cloned().collect();
    };
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let Some(root) = d.root(doc) else {
        return anchored.keys().cloned().collect();
    };
    let mut groups = FingerprintGroups::new();
    for c in d.elements(root, Some(&W::name("comment"))) {
        let Some(id) = d.attribute(c, &W::name("id")).map(str::to_string) else {
            continue;
        };
        let Some(&range) = anchored.get(&id) else {
            continue;
        };
        let fingerprint = comment_definition_fingerprint(&d, c);
        groups.entry(fingerprint).or_default().push((id, range));
    }

    let mut keep = HashSet::new();
    for candidates in groups.values() {
        let has_nonempty = candidates.iter().any(|(_, (start, end))| end > start);
        let b_ranges: HashSet<_> = candidates
            .iter()
            .filter(|(id, _)| !a_side.contains(id))
            .map(|(_, range)| *range)
            .collect();
        for (id, range) in candidates {
            if has_nonempty && range.0 == range.1 {
                continue;
            }
            if a_side.contains(id) && b_ranges.contains(range) {
                continue;
            }
            keep.insert(id.clone());
        }
    }
    keep
}

/// Entry point — run after the diff produced `result_root` but BEFORE it is
/// serialized into `out` (anchors are injected into the result DOM).
pub fn carry_comments(
    dom: &mut Dom,
    result_root: NodeId,
    (pkg1, main1): (&PartFs, &str),
    (pkg2, main2): (&PartFs, &str),
    (out, out_main): (&mut PartFs, &str),
    author: &str,
) {
    let ids_a = comment_ids_of(pkg1);
    let ids_b = comment_ids_of(pkg2);
    if ids_a.is_empty() && ids_b.is_empty() {
        return;
    }
    let no_map = HashMap::new();
    let mut a_side = HashSet::new();
    let anchored = if ids_b.is_empty() {
        // only A has comments; its parts are already in out (out is A's clone)
        inject_side(
            dom,
            result_root,
            (pkg1, main1),
            false,
            author,
            &no_map,
            None,
        )
    } else if ids_a.is_empty()
        || b_carries_same_comments_as_a(pkg1, pkg2)
        || b_covers_comment_identities_of_a(pkg1, main1, pkg2, main2)
    {
        // B carries the union — parts byte-identical from B. Two gates:
        //   1. id+definition match for every A comment (classic superset).
        //   2. id-independent anchored-identity cover (M213): Word-renumbered
        //      comment sets across redline sources.
        // Bare numeric-id superset alone is still not enough.
        install_parts_from(out, out_main, pkg2);
        inject_side(dom, result_root, (pkg2, main2), true, author, &no_map, None)
    } else {
        // true union: B's parts as base + A-only comments appended
        install_parts_from(out, out_main, pkg2);
        let id_map = union_comments_xml(out, out_main, pkg1);
        let mut anchored =
            inject_side(dom, result_root, (pkg2, main2), true, author, &no_map, None);
        let a_only: HashSet<String> = id_map.keys().cloned().collect();
        a_side = id_map.values().cloned().collect();
        anchored.extend(inject_side(
            dom,
            result_root,
            (pkg1, main1),
            false,
            author,
            &id_map,
            Some(&a_only),
        ));
        anchored
    };
    let anchored = select_anchor_aware_comments(out, &anchored, &a_side);
    // Also strip body anchors for dropped ids so they don't linger orphan-free
    // as range markers without a comments.xml entry (Ring-1).
    strip_unanchored_comment_markers(dom, result_root, &anchored);
    drop_orphans(out, out_main, &anchored);
}

/// Remove commentRangeStart/End/commentReference whose id is not in `keep`.
fn strip_unanchored_comment_markers(dom: &mut Dom, result_root: NodeId, keep: &HashSet<String>) {
    let names = [
        W::name("commentRangeStart"),
        W::name("commentRangeEnd"),
        W::name("commentReference"),
    ];
    let mut dead: Vec<NodeId> = Vec::new();
    for name in names {
        for e in dom.descendants(result_root, Some(&name)) {
            if dom
                .attribute(e, &W::name("id"))
                .is_none_or(|id| !keep.contains(id))
            {
                dead.push(e);
            }
        }
    }
    for e in dead {
        dom.remove(e);
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn texts_and_markers(dom: &Dom, root: NodeId) -> Vec<String> {
        dom.descendants(root, None)
            .into_iter()
            .filter_map(|e| {
                let name = dom.name(e)?;
                if name == W::t() {
                    Some(dom.value(e))
                } else if name == W::name("commentRangeStart") {
                    Some(format!("[{}", dom.attribute(e, &W::name("id"))?))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Splitting a run moves its trailing leaves into the new run; a later
    /// split of one of those leaves must happen in the run that now holds it,
    /// or the suffix lands before the moved text.
    /// A comment over several paragraphs keeps its anchors when text inside
    /// it changed: its ends map by their own context (the 23,600-char range
    /// of docx_lots_of_comments_addition's comments 19 and 20).
    #[test]
    fn a_long_range_with_changed_text_inside_maps_by_its_ends() {
        let chars = |s: &str| s.chars().collect::<Vec<_>>();
        let head = "Word advantage: Track Changes controls, compare/combine, ";
        let tail = "and every table after it stays inside the comment range.";
        let src = chars(&format!(
            "Intro text. {head}middle one two three {tail} Outro."
        ));
        let merged = chars(&format!(
            "Intro text. {head}middle ONE TWO three {tail} Outro."
        ));
        let start = "Intro text. ".chars().count();
        let end = src.len() - " Outro.".chars().count();
        let r = Range {
            id: "19".into(),
            start,
            end,
            seq: 0,
        };
        assert_eq!(map_range(&src, &merged, &r), Some((start, end)));
        // The tail must follow the head: a merged text holding only the tail
        // before the head has nowhere to end the range.
        let swapped = chars(&format!("Intro text. {tail} Outro. {head}middle"));
        assert_eq!(map_range(&src, &swapped, &r), None);
    }

    /// A copied section under its own comment maps to the copy, not to the
    /// first section with the same text (docx_lots_of_comments_addition's
    /// comments 19 and 20 landed on comments 3 and 4's range and were then
    /// collapsed as duplicates).
    #[test]
    fn repeated_text_maps_to_the_occurrence_at_its_own_place() {
        let chars = |s: &str| s.chars().collect::<Vec<_>>();
        let section = "Word advantage: Track Changes controls.";
        let src = chars(&format!("Intro. {section} Middle part. {section} End."));
        let merged = chars(&format!(
            "Intro. {section} Middle part added. {section} End."
        ));
        let second = format!("Intro. {section} Middle part. ").chars().count();
        let len = section.chars().count();
        let r = Range {
            id: "19".into(),
            start: second,
            end: second + len,
            seq: 0,
        };
        let merged_second = format!("Intro. {section} Middle part added. ")
            .chars()
            .count();
        assert_eq!(
            map_range(&src, &merged, &r),
            Some((merged_second, merged_second + len))
        );
    }

    /// Moved text belongs to A at its source and to B at its destination.
    #[test]
    fn each_side_counts_moved_text_once_at_its_own_location() {
        let mut dom = Dom::new();
        let d = dom.parse_xdocument(concat!(
            "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
            "<w:body><w:p><w:moveFrom w:id=\"1\" w:author=\"Redline\"><w:r><w:t>Moved</w:t></w:r></w:moveFrom>",
            "<w:r><w:t>Kept</w:t></w:r>",
            "<w:moveTo w:id=\"2\" w:author=\"Redline\"><w:r><w:t>Moved</w:t></w:r></w:moveTo></w:p>",
            "</w:body></w:document>"
        ));
        let root = dom.root(d).unwrap();
        assert_eq!(
            collect_segments(&dom, root, true, "Redline", false).0,
            "KeptMoved"
        );
        assert_eq!(
            collect_segments(&dom, root, false, "Redline", false).0,
            "MovedKept"
        );
    }

    #[test]
    fn a_second_split_follows_a_leaf_moved_by_the_first() {
        let mut dom = Dom::new();
        let d = dom.parse_xdocument(concat!(
            "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
            "<w:body><w:p><w:r><w:t>Alpha beta</w:t><w:br/><w:t>Gamma delta</w:t></w:r></w:p>",
            "</w:body></w:document>"
        ));
        let root = dom.root(d).unwrap();
        let (text, mut segs) = collect_segments(&dom, root, true, "Redline", false);
        assert_eq!(text, "Alpha betaGamma delta");
        let first = new_anchor(&mut dom, true, "1");
        place_before_offset(&mut dom, &mut segs, 6, first);
        let second = new_anchor(&mut dom, true, "2");
        place_before_offset(&mut dom, &mut segs, 16, second);
        assert_eq!(
            texts_and_markers(&dom, root),
            ["Alpha ", "[1", "beta", "Gamma ", "[2", "delta"]
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod deeper_boundary_tests {
    use super::*;

    const MAIN: &str = "word/document.xml";
    // The same bundled, in-memory package used by comparer::parts tests.
    const PACKAGE: &[u8] = include_bytes!("../../tests/fixtures/relids/image_doc.docx");

    fn xml(local: &str, contents: &str) -> String {
        format!(
            "<w:{local} xmlns:w=\"{}\" xmlns:w14=\"{}\" xmlns:mc=\"{}\" xmlns:x=\"urn:comment-test\">{contents}</w:{local}>",
            W::URI,
            W14::URI,
            MC::URI,
        )
    }

    fn package(comments: Option<&str>, body: Option<&str>) -> PartFs {
        let mut pkg = PartFs::open(PACKAGE).unwrap();
        for (part, _, rel) in FAMILY {
            remove_family_part(&mut pkg, MAIN, part, rel);
        }
        pkg.remove_part(MAIN);
        if let Some(body) = body {
            pkg.set_part(
                MAIN,
                xml("document", &format!("<w:body>{body}</w:body>")).into_bytes(),
            );
        }
        if let Some(comments) = comments {
            pkg.set_part("word/comments.xml", xml("comments", comments).into_bytes());
        }
        pkg
    }

    fn parse(contents: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml("document", &format!("<w:body>{contents}</w:body>")));
        let root = dom.root(doc).unwrap();
        (dom, root)
    }

    fn set(values: &[&str]) -> HashSet<String> {
        values.iter().map(|s| (*s).to_string()).collect()
    }

    fn comment(id: &str, body: &str) -> String {
        format!("<w:comment w:id=\"{id}\"><w:p><w:r><w:t>{body}</w:t></w:r></w:p></w:comment>")
    }

    fn range(start: usize, end: usize) -> Range {
        Range {
            id: "7".into(),
            start,
            end,
            seq: 0,
        }
    }

    fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    fn children(dom: &Dom, parent: NodeId) -> Vec<String> {
        dom.elements(parent, None)
            .into_iter()
            .map(|n| {
                let name = dom.name(n).unwrap();
                format!("{}:{}", name.local_name(), dom.value(n))
            })
            .collect()
    }

    #[test]
    fn missing_and_rootless_parts_are_distinct_empty_boundaries() {
        for contents in [None, Some(""), Some("<!-- no document element -->")] {
            let mut pkg = package(None, None);
            if let Some(contents) = contents {
                pkg.set_part("word/comments.xml", contents.as_bytes().to_vec());
                pkg.set_part(MAIN, contents.as_bytes().to_vec());
            }
            assert!(comment_ids_of(&pkg).is_empty());
            assert!(comment_id_fingerprint_of(&pkg).is_empty());
            assert!(extract_events(&pkg, MAIN).is_none());
            assert!(b_carries_same_comments_as_a(&pkg, &package(None, None)));
            assert!(b_covers_comment_identities_of_a(
                &pkg,
                MAIN,
                &package(None, None),
                MAIN
            ));
            let anchors = HashMap::from([("7".into(), (2, 3))]);
            assert_eq!(
                select_anchor_aware_comments(&pkg, &anchors, &HashSet::new()),
                set(&["7"])
            );
            let before = pkg.part_bytes("word/comments.xml").map(<[u8]>::to_vec);
            drop_orphans(&mut pkg, MAIN, &HashSet::new());
            assert_eq!(pkg.part_bytes("word/comments.xml"), before.as_deref());
            let (mut dom, root) = parse("<w:p><w:r><w:t>Kept</w:t></w:r></w:p>");
            assert!(
                inject_side(
                    &mut dom,
                    root,
                    (&pkg, MAIN),
                    true,
                    "R",
                    &HashMap::new(),
                    None
                )
                .is_empty()
            );
            assert_eq!(collect_segments(&dom, root, true, "R", true).0, "Kept\n");
        }
    }

    #[test]
    fn fingerprints_preserve_metadata_and_join_only_visible_text() {
        let definition = "<w:comment w:id=\"7\" w:author=\"Ada\" w:date=\"fixed\" w:initials=\"AL\"><w:p><w:r><w:t>  one </w:t><w:delText>ignored</w:delText><w:t> two\tthree </w:t></w:r></w:p></w:comment><w:comment><w:p/></w:comment>";
        let pkg = package(Some(definition), None);
        assert_eq!(comment_ids_of(&pkg), set(&["7"]));
        assert_eq!(
            comment_id_fingerprint_of(&pkg),
            HashMap::from([("7".into(), "one two three\0Ada\0fixed\0AL".into())])
        );
        for (attribute, value) in [("author", "Grace"), ("date", "other"), ("initials", "GH")] {
            let changed = definition.replace(
                &format!(
                    "w:{attribute}=\"{}\"",
                    match attribute {
                        "author" => "Ada",
                        "date" => "fixed",
                        _ => "AL",
                    }
                ),
                &format!("w:{attribute}=\"{value}\""),
            );
            assert!(
                !b_carries_same_comments_as_a(&pkg, &package(Some(&changed), None)),
                "{attribute}"
            );
        }
    }

    #[test]
    fn unanchored_identity_includes_id_and_live_ranges_supersede_points() {
        let defs = format!(
            "{}{}{}",
            comment("7", "same"),
            comment("8", "same"),
            comment("9", "same")
        );
        for body in [None, Some("<w:p><w:r><w:t>abc</w:t></w:r></w:p>")] {
            let pkg = package(Some(&defs), body);
            let identities = comment_anchor_identities(&pkg, MAIN);
            assert_eq!(identities.len(), 3);
            for id in ["7", "8", "9"] {
                assert_eq!(identities[id], format!("same\0\0\0\0<unanchored:{id}>"));
            }
        }
        let body = "<w:p><w:commentRangeStart w:id=\"7\"/><w:r><w:t>abc</w:t></w:r><w:commentRangeEnd w:id=\"7\"/><w:r><w:commentReference w:id=\"8\"/></w:r></w:p>";
        let pkg = package(Some(&defs), Some(body));
        assert_eq!(
            comment_anchor_identities(&pkg, MAIN),
            HashMap::from([("7".into(), "same\0\0\0\0\0abc\0".into())])
        );
        assert!(!b_covers_comment_identities_of_a(
            &package(Some(&comment("7", "same")), None),
            MAIN,
            &package(Some(&comment("8", "same")), None),
            MAIN
        ));
    }

    #[test]
    fn extraction_ignores_idless_markers_and_deduplicates_reference_points() {
        let body = "<!-- non-text node --><w:p><w:commentRangeStart/><w:commentRangeEnd/><w:commentRangeEnd w:id=\"missing\"/><w:r><w:commentReference/><w:commentReference w:id=\"p\"/><w:commentReference w:id=\"p\"/><w:t>é🐋</w:t></w:r><w:commentRangeStart w:id=\"r\"/><w:commentRangeStart w:id=\"r\"/><w:r><w:t>x</w:t></w:r><w:commentRangeEnd w:id=\"r\"/><w:r><w:commentReference w:id=\"r\"/></w:r></w:p><w:p/>";
        let (text, ranges) = extract_events(&package(None, Some(body)), MAIN).unwrap();
        assert_eq!(text, "é🐋x\n\n");
        assert_eq!(
            ranges
                .iter()
                .map(|r| (r.id.as_str(), r.start, r.end, r.seq))
                .collect::<Vec<_>>(),
            [("r", 2, 3, 0), ("p", 0, 0, usize::MAX)]
        );
    }

    #[test]
    fn character_search_truth_table_includes_empty_needles_and_ties() {
        for (hay, needle, from, want) in [
            ("abc", "", 0, Some(0)),
            ("abc", "", 3, Some(3)),
            ("abc", "", 4, None),
            ("abc", "bc", 1, Some(1)),
            ("abc", "bc", 2, None),
            ("a", "aa", 0, None),
            ("", "a", 0, None),
            ("abc", "x", 0, None),
        ] {
            assert_eq!(
                find_chars_from(&chars(hay), &chars(needle), from),
                want,
                "{hay:?}/{needle:?}/{from}"
            );
        }
        assert_eq!(find_chars_nearest(&chars("x-x"), &chars("x"), 1), Some(0));
        assert_eq!(find_chars_nearest(&chars("x-x-x"), &chars("x"), 3), Some(2));
        assert_eq!(expected(&[], &chars("abc"), 8, 1), 0);
        assert_eq!(expected(&chars("abcd"), &chars("abcdefgh"), 3, 1), 5);
        assert_eq!(expected(&chars("abcd"), &chars("a"), 1, 9), 0);
        assert_eq!(map_range(&[], &[], &range(0, 0)), None);
        assert_eq!(map_range(&chars("abc"), &chars("xyz"), &range(1, 1)), None);
        assert_eq!(
            map_range_ends(&chars(&"x".repeat(80)), &[], &range(0, 80)),
            None
        );
    }

    #[test]
    fn end_mapping_checks_both_inclusive_span_limits() {
        let source = chars(&format!(
            "{}{}{}",
            "H".repeat(40),
            "I".repeat(320),
            "T".repeat(40)
        ));
        for (span, want) in [
            (119, None),
            (120, Some((0, 120))),
            (880, Some((0, 880))),
            (881, None),
        ] {
            let merged = chars(&format!(
                "{}{}{}",
                "H".repeat(40),
                "M".repeat(span - 80),
                "T".repeat(40)
            ));
            assert_eq!(
                map_range_ends(&source, &merged, &range(0, 400)),
                want,
                "span={span}"
            );
        }
        assert_eq!(
            map_range_ends(&source, &chars(&"H".repeat(40)), &range(0, 400)),
            None
        );
    }

    #[test]
    fn projection_filters_foreign_deletions_and_retains_empty_paragraph_marks() {
        let (dom, root) = parse(
            "<w:p><w:del w:author=\"R\"><w:r><w:t>hidden</w:t><w:delText>ours</w:delText></w:r></w:del><w:del w:author=\"Other\"><w:r><w:delText>foreign</w:delText></w:r></w:del><w:r><w:t>kept</w:t></w:r></w:p><w:p/>",
        );
        assert_eq!(collect_segments(&dom, root, true, "R", true).0, "kept\n\n");
        assert_eq!(
            collect_segments(&dom, root, false, "R", true).0,
            "hiddenourskept\n\n"
        );
        for (change, a, b) in [
            ("ins", false, true),
            ("moveTo", false, true),
            ("del", true, false),
            ("moveFrom", true, false),
        ] {
            let (dom, root) = parse(&format!(
                "<w:p><w:pPr><w:rPr><w:{change}/></w:rPr></w:pPr></w:p>"
            ));
            let p = dom.descendants(root, Some(&W::p()))[0];
            assert_eq!(mark_on_side(&dom, p, false), a, "{change}");
            assert_eq!(mark_on_side(&dom, p, true), b, "{change}");
        }
    }

    #[test]
    fn styled_unicode_split_preserves_properties_and_trailing_nodes() {
        let (mut dom, root) = parse(
            "<w:p><w:r><w:rPr><w:b/></w:rPr><w:t>é🐋Z</w:t><w:tab/><w:t>tail</w:t></w:r></w:p>",
        );
        let (_, mut segs) = collect_segments(&dom, root, true, "R", false);
        split_seg(&mut dom, &mut segs, 0, 2);
        let p = dom.descendants(root, Some(&W::p()))[0];
        assert_eq!(children(&dom, p), ["r:é🐋", "r:Ztail"]);
        assert_eq!(
            children(&dom, segs[1].run),
            ["rPr:", "t:Z", "tab:", "t:tail"]
        );
        assert_eq!(
            segs.iter().map(|s| (s.start, s.len)).collect::<Vec<_>>(),
            [(0, 2), (2, 1), (3, 4)]
        );
        assert_eq!(segs[1].run, segs[2].run);
        for s in &segs[..2] {
            assert!(
                dom.element(dom.element(s.run, &W::r_pr()).unwrap(), &W::name("b"))
                    .is_some()
            );
            assert_eq!(
                dom.attribute(s.leaf, &XNamespace::xml().name("space")),
                Some("preserve")
            );
        }
        split_run_before_leaf(&mut dom, &mut segs, 2);
        assert_eq!(children(&dom, p), ["r:é🐋", "r:Z", "r:tail"]);
        assert!(dom.element(segs[2].run, &W::r_pr()).is_some());
    }

    #[test]
    fn placement_empty_start_and_past_last_boundaries() {
        for (body, offset, after, want) in [
            ("<w:p/>", 1, false, true),
            ("<w:p/>", 0, true, true),
            ("<w:p><w:r><w:t>x</w:t></w:r></w:p>", 2, false, true),
            ("", 0, false, false),
            ("", 0, true, false),
        ] {
            let (mut dom, root) = parse(body);
            let (_, mut segs) = collect_segments(&dom, root, true, "R", true);
            if let Some(i) = segs.iter().position(Seg::is_mark) {
                let leaf = segs[i].leaf;
                split_run_before_leaf(&mut dom, &mut segs, i);
                assert_eq!(segs[i].leaf, leaf);
            }
            let anchor = new_anchor(&mut dom, false, "7");
            if after {
                place_after_offset(&mut dom, &mut segs, offset, anchor);
            } else {
                place_before_offset(&mut dom, &mut segs, offset, anchor);
            }
            assert_eq!(
                dom.parent(anchor).is_some(),
                want,
                "{body}/{offset}/{after}"
            );
            if want {
                let p = dom.descendants(root, Some(&W::p()))[0];
                if after && offset == 0 {
                    let body = dom.element(root, &W::body()).unwrap();
                    assert_eq!(dom.parent(anchor), Some(body));
                    assert_eq!(children(&dom, body), ["commentRangeEnd:", "p:"]);
                } else {
                    assert_eq!(dom.parent(anchor), Some(p));
                    let names = children(&dom, p);
                    assert_eq!(names.last().unwrap(), "commentRangeEnd:");
                }
            }
        }
        let (mut dom, root) = parse("<w:p><w:r><w:t>x</w:t></w:r></w:p>");
        let (_, mut segs) = collect_segments(&dom, root, true, "R", false);
        let anchor = new_anchor(&mut dom, false, "7");
        place_before_offset(&mut dom, &mut segs, 2, anchor);
        assert_eq!(
            children(&dom, dom.descendants(root, Some(&W::p()))[0]),
            ["r:x", "commentRangeEnd:"]
        );
    }

    #[test]
    fn point_after_normal_mark_does_not_clone_next_deleted_mark() {
        let (mut dom, root) =
            parse("<w:p/><w:p><w:pPr><w:rPr><w:del w:author=\"R\"/></w:rPr></w:pPr></w:p>");
        let (_, mut segs) = collect_segments(&dom, root, false, "R", true);
        let anchor = new_anchor(&mut dom, false, "7");
        place_after_offset(&mut dom, &mut segs, 1, anchor);
        let p = dom.descendants(root, Some(&W::p()))[1];
        assert_eq!(children(&dom, p), ["pPr:", "commentRangeEnd:"]);
        assert_eq!(dom.parent(anchor), Some(p));
    }

    #[test]
    fn injection_filters_ids_and_rejects_unmappable_ranges() {
        let source = package(
            None,
            Some(
                "<w:p><w:commentRangeStart w:id=\"7\"/><w:r><w:t>abc</w:t></w:r><w:commentRangeEnd w:id=\"7\"/></w:p>",
            ),
        );
        for (text, filter, want) in [
            ("abc", set(&["other"]), None),
            ("xyz", set(&["7"]), None),
            ("abc", set(&["7"]), Some((0, 3))),
        ] {
            let (mut dom, root) = parse(&format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>"));
            let map = HashMap::from([("7".into(), "42".into())]);
            let anchored = inject_side(
                &mut dom,
                root,
                (&source, MAIN),
                true,
                "R",
                &map,
                Some(&filter),
            );
            assert_eq!(anchored.get("42").copied(), want);
            assert_eq!(anchored.len(), usize::from(want.is_some()));
            assert_eq!(
                dom.descendants(root, Some(&W::name("commentReference")))
                    .len(),
                usize::from(want.is_some())
            );
            assert_eq!(collect_segments(&dom, root, true, "R", false).0, text);
        }
    }

    #[test]
    fn deletion_exit_requires_only_markers_after_the_end() {
        for (tail, moves) in [
            ("<w:commentRangeStart w:id=\"8\"/>", true),
            ("<w:r><w:commentReference w:id=\"7\"/></w:r>", true),
            ("<w:r><w:t>tail</w:t></w:r>", false),
            ("<w:bookmarkEnd w:id=\"3\"/>", false),
        ] {
            let (mut dom, root) = parse(&format!(
                "<w:p><w:del><w:r><w:delText>x</w:delText></w:r><w:commentRangeStart w:id=\"7\"/><w:commentRangeEnd w:id=\"7\"/>{tail}</w:del></w:p>"
            ));
            let end = dom.descendants(root, Some(&W::name("commentRangeEnd")))[0];
            let old_parent = dom.parent(end).unwrap();
            step_out_of_deletion_end(&mut dom, end);
            assert_eq!(dom.parent(end) != Some(old_parent), moves, "{tail}");
            if moves {
                let p = dom.descendants(root, Some(&W::p()))[0];
                assert_eq!(dom.parent(end), Some(p));
                assert_eq!(children(&dom, old_parent), ["r:x"]);
            } else {
                assert_eq!(dom.parent(end), Some(old_parent));
            }
        }
    }

    #[test]
    fn allocation_wrap_and_collision_truth_tables() {
        for initial in [0, 1, 0x8000_0000, u32::MAX] {
            let mut used = set(&["00000001", "00000002"]);
            let mut next = initial;
            assert_eq!(allocate_para_id(&mut used, &mut next), "00000003");
            assert_eq!(next, 4);
            assert_eq!(used, set(&["00000001", "00000002", "00000003"]));
        }
        let mut used = set(&["FFFFFFFF", "00000001"]);
        let mut next = u32::MAX;
        assert_eq!(allocate_durable_id(&mut used, &mut next), "00000002");
        assert_eq!(next, 3);
        let mut next = 0;
        assert_eq!(
            allocate_durable_id(&mut HashSet::new(), &mut next),
            "00000001"
        );
        assert_eq!(next, 2);
    }

    #[test]
    fn reference_rewriting_changes_only_matching_identity_attributes() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument("<root paraId=\"aa\" paraIdParent=\"bb\" durableId=\"aa\" other=\"aa\"><child paraId=\"cc\" durableId=\"cc\"/></root>");
        let root = dom.root(doc).unwrap();
        let map = HashMap::from([("AA".into(), "00000001".into())]);
        rewrite_para_id_references(&mut dom, root, &map);
        rewrite_durable_id_references(&mut dom, root, &map);
        assert_eq!(
            dom.attribute(root, &XName::get("paraId", "")),
            Some("00000001")
        );
        assert_eq!(
            dom.attribute(root, &XName::get("durableId", "")),
            Some("00000001")
        );
        assert_eq!(
            dom.attribute(root, &XName::get("paraIdParent", "")),
            Some("bb")
        );
        assert_eq!(dom.attribute(root, &XName::get("other", "")), Some("aa"));
        let child = dom.elements(root, None)[0];
        assert_eq!(dom.attribute(child, &XName::get("paraId", "")), Some("cc"));
        assert_eq!(
            dom.attribute(child, &XName::get("durableId", "")),
            Some("cc")
        );
        assert_eq!(rewrite_qname_token("missing:item", &map), "missing:item");
        assert_eq!(rewrite_qname_token("AA:item", &map), "00000001:item");
        assert_eq!(rewrite_qname_token("AA", &map), "00000001");
        assert!(!is_namespace_qname_list(None, &MC::name("Ignorable")));
        assert!(!is_namespace_qname_list(
            Some(&W::p()),
            &XName::get("Requires", "")
        ));
        assert!(is_namespace_qname_list(
            Some(&MC::name("Choice")),
            &XName::get("Requires", "")
        ));
    }

    #[test]
    fn namespace_conflicts_reuse_aliases_or_skip_occupied_generated_prefixes() {
        for (destination, chosen) in [
            (
                "xmlns:x=\"urn:other\" xmlns:a=\"urn:source\" xmlns:z=\"urn:source\"",
                "a",
            ),
            ("xmlns:x=\"urn:other\" xmlns:ns0=\"urn:occupied\"", "ns1"),
        ] {
            let mut dom = Dom::new();
            let source = dom.parse_xdocument(&format!("<root xmlns:mc=\"{}\" xmlns:x=\"urn:source\" mc:Ignorable=\"x missing\"><child xmlns:local=\"urn:local\" mc:PreserveElements=\"x:item unbound:item\" plain=\"v\"><x:item/></child></root>", MC::URI));
            let dest = dom.parse_xdocument(&format!("<root {destination}/>"));
            let source = dom.root(source).unwrap();
            let dest = dom.root(dest).unwrap();
            let child = dom.elements(source, None)[0];
            let clone = dom.clone_subtree(child);
            preserve_cloned_namespace_context(&mut dom, source, dest, clone);
            assert_eq!(
                namespace_declarations(&dom, dest)
                    .get(chosen)
                    .map(String::as_str),
                Some("urn:source")
            );
            assert_eq!(dom.attribute(dest, &MC::name("Ignorable")), Some(chosen));
            assert_eq!(
                dom.attribute(clone, &MC::name("PreserveElements")),
                Some(format!("{chosen}:item unbound:item").as_str())
            );
            assert_eq!(dom.attribute(clone, &XName::get("plain", "")), Some("v"));
            assert_eq!(
                namespace_declarations(&dom, clone)
                    .get("local")
                    .map(String::as_str),
                Some("urn:local")
            );
            assert_eq!(
                dom.name(dom.elements(clone, None)[0])
                    .unwrap()
                    .namespace_name(),
                "urn:source"
            );
            assert!(!namespace_declarations(&dom, dest).contains_key("unbound"));
        }
    }

    #[test]
    fn union_missing_or_rootless_primary_parts_preserve_output() {
        for (a_xml, b_xml) in [
            (None, Some("<w:comments/>")),
            (Some("<w:comments/>"), None),
            (Some(""), Some("<w:comments/>")),
            (Some("<w:comments/>"), Some("")),
        ] {
            let mut a = package(None, None);
            let mut b = package(None, None);
            for (pkg, contents) in [(&mut a, a_xml), (&mut b, b_xml)] {
                if let Some(contents) = contents {
                    pkg.set_part("word/comments.xml", contents.as_bytes().to_vec());
                }
            }
            let before = b.part_bytes("word/comments.xml").map(<[u8]>::to_vec);
            assert!(union_comments_xml(&mut b, MAIN, &a).is_empty());
            assert_eq!(b.part_bytes("word/comments.xml"), before.as_deref());
        }
    }

    #[test]
    fn union_skips_idless_and_identical_comments_and_duplicate_auxiliary_keys() {
        let defs = format!("{}<w:comment><w:p/></w:comment>", comment("7", "same"));
        let mut a = package(Some(&defs), None);
        let mut b = package(Some(&comment("7", "same")), None);
        let aux = "<root><entry paraId=\"aa\" durableId=\"10\"/><entry other=\"idless\"/></root>";
        for (part, _, _) in &FAMILY[1..] {
            a.set_part(part, aux.as_bytes().to_vec());
            b.set_part(part, aux.as_bytes().to_vec());
        }
        assert!(union_comments_xml(&mut b, MAIN, &a).is_empty());
        assert_eq!(comment_ids_of(&b), set(&["7"]));
        for (part, _, _) in &FAMILY[1..] {
            assert_eq!(b.part_string(part).as_deref(), Some(aux));
        }
    }

    #[test]
    fn auxiliary_rootless_and_seeded_relationship_boundaries() {
        for mode in [0, 1, 2, 3] {
            let mut a = package(Some(&comment("8", "A")), None);
            let mut b = package(Some(&comment("7", "B")), None);
            let (part, ct, rel) = FAMILY[1];
            a.set_part(
                part,
                if mode == 0 || mode == 1 {
                    Vec::new()
                } else {
                    b"<root><entry paraId=\"aa\"/></root>".to_vec()
                },
            );
            if mode == 1 || mode == 2 {
                b.set_part(part, Vec::new());
            }
            if mode == 3 {
                b.add_document_relationship(MAIN, rel, "commentsExtended.xml");
            }
            let map = union_comments_xml(&mut b, MAIN, &a);
            assert_eq!(map, HashMap::from([("8".into(), "8".into())]));
            match mode {
                0 => assert!(b.part_bytes(part).is_none()),
                1 | 2 => assert_eq!(b.part_bytes(part), Some(&b""[..])),
                _ => {
                    assert_eq!(
                        b.part_string(part).as_deref(),
                        Some("<root><entry paraId=\"aa\" /></root>")
                    );
                    assert_eq!(b.content_type_for(part).as_deref(), Some(ct));
                    assert_eq!(
                        b.read_rels_for(MAIN)
                            .unwrap()
                            .items
                            .iter()
                            .filter(|r| r.rel_type == rel)
                            .count(),
                        1
                    );
                }
            }
        }
    }

    #[test]
    fn comments_ids_accepts_missing_durable_id_without_inventing_one() {
        let mut a = package(Some(&comment("8", "A")), None);
        let mut b = package(Some(&comment("7", "B")), None);
        a.set_part(
            FAMILY[2].0,
            b"<root><entry paraId=\"bb\"/><entry paraId=\"aa\" durableId=\"99\"/><entry/></root>"
                .to_vec(),
        );
        b.set_part(
            FAMILY[2].0,
            b"<root><entry paraId=\"aa\" durableId=\"10\"/></root>".to_vec(),
        );
        assert_eq!(
            union_comments_xml(&mut b, MAIN, &a),
            HashMap::from([("8".into(), "8".into())])
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&b.part_string(FAMILY[2].0).unwrap());
        let entries = dom.elements(dom.root(doc).unwrap(), None);
        assert_eq!(entries.len(), 2);
        assert_eq!(
            dom.attribute(entries[1], &XName::get("paraId", "")),
            Some("bb")
        );
        assert_eq!(
            dom.attribute(entries[1], &XName::get("durableId", "")),
            None
        );
        assert_eq!(
            dom.attribute(entries[0], &XName::get("durableId", "")),
            Some("10")
        );
    }

    #[test]
    fn orphan_cleanup_leaves_live_parent_links_and_unchanged_aux_parts() {
        for aux in [
            None,
            Some(""),
            Some("<root><entry paraId=\"bb\" paraIdParent=\"bb\"/></root>"),
        ] {
            let defs = format!(
                "{}<w:comment w:id=\"8\"><w:p w14:paraId=\"aa\" plain=\"ignored\"><w:r><w:t>dead</w:t></w:r></w:p></w:comment>",
                comment("7", "live")
            );
            let mut pkg = package(Some(&defs), None);
            if let Some(aux) = aux {
                pkg.set_part(FAMILY[1].0, aux.as_bytes().to_vec());
            }
            drop_orphans(&mut pkg, MAIN, &set(&["7"]));
            assert_eq!(comment_ids_of(&pkg), set(&["7"]));
            assert_eq!(pkg.part_string(FAMILY[1].0).as_deref(), aux);
        }
    }

    #[test]
    fn selection_truth_table_distinguishes_points_ranges_and_side_duplicates() {
        let defs = format!(
            "{}{}{}<w:comment><w:p/></w:comment>{}",
            comment("7", "same"),
            comment("8", "same"),
            comment("9", "same"),
            comment("10", "unanchored")
        );
        let pkg = package(Some(&defs), None);
        for (ranges, a_side, want) in [
            ([(0, 0), (0, 0), (0, 0)], set(&[]), set(&["7", "8", "9"])),
            ([(0, 1), (0, 0), (2, 3)], set(&[]), set(&["7", "9"])),
            ([(0, 1), (0, 1), (2, 3)], set(&["8", "9"]), set(&["7", "9"])),
            ([(0, 1), (0, 1), (0, 1)], set(&[]), set(&["7", "8", "9"])),
        ] {
            let anchors = ["7", "8", "9"]
                .into_iter()
                .zip(ranges)
                .map(|(id, r)| (id.to_string(), r))
                .collect();
            assert_eq!(select_anchor_aware_comments(&pkg, &anchors, &a_side), want);
        }
    }

    #[test]
    fn stripping_keeps_only_defined_ids_in_each_marker_kind() {
        let (mut dom, root) = parse(
            "<w:p><w:commentRangeStart w:id=\"7\"/><w:commentRangeStart/><w:commentRangeEnd w:id=\"8\"/><w:commentRangeEnd w:id=\"7\"/><w:r><w:commentReference/><w:commentReference w:id=\"7\"/><w:t>kept</w:t></w:r></w:p>",
        );
        strip_unanchored_comment_markers(&mut dom, root, &set(&["7"]));
        for name in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            let markers = dom.descendants(root, Some(&W::name(name)));
            assert_eq!(markers.len(), 1, "{name}");
            assert_eq!(dom.attribute(markers[0], &W::name("id")), Some("7"));
        }
        assert_eq!(collect_segments(&dom, root, true, "R", false).0, "kept");
    }

    #[test]
    fn projection_rejects_document_text_and_a_detached_leaf_without_a_run() {
        let mut dom = Dom::new();
        let document = dom.new_document();
        let text = dom.new_text("outside any element");
        dom.add(document, text);
        let leaf = dom.new_element(W::t());
        dom.add_text(leaf, "outside any run");
        for root in [document, leaf] {
            for b_side in [false, true] {
                let (text, segments) = collect_segments(&dom, root, b_side, "R", true);
                assert_eq!(text, "");
                assert!(segments.is_empty());
            }
        }
        assert_eq!(dom.parent(text), Some(document));
        assert_eq!(dom.parent(leaf), None);
        assert_eq!(dom.value(leaf), "outside any run");
    }

    #[test]
    fn same_id_and_body_with_different_authors_requires_two_union_definitions() {
        // Equal body text does not collapse comments from different authors.
        let a_definition =
            comment("7", "same").replace("w:id=\"7\"", "w:id=\"7\" w:author=\"Alice\"");
        let b_definition =
            comment("7", "same").replace("w:id=\"7\"", "w:id=\"7\" w:author=\"Bob\"");
        let a = package(Some(&a_definition), None);
        let mut b = package(Some(&b_definition), None);
        assert!(!b_carries_same_comments_as_a(&a, &b));
        assert!(!b_covers_comment_identities_of_a(&a, MAIN, &b, MAIN));
        assert_eq!(
            union_comments_xml(&mut b, MAIN, &a),
            HashMap::from([("7".into(), "8".into())])
        );
        assert_eq!(comment_ids_of(&b), set(&["7", "8"]));
        assert_eq!(
            comment_id_fingerprint_of(&b),
            HashMap::from([
                ("7".into(), "same\0Bob\0\0".into()),
                ("8".into(), "same\0Alice\0\0".into())
            ])
        );
    }

    #[test]
    fn equal_comment_bodies_keep_distinct_dates_and_initials() {
        for (attribute, a_value, b_value) in [
            ("date", "2026-10-07T00:00:00Z", "2026-10-08T00:00:00Z"),
            ("initials", "AA", "BB"),
        ] {
            let definition = |value: &str| {
                comment("7", "same").replace(
                    "w:id=\"7\"",
                    &format!("w:id=\"7\" w:{attribute}=\"{value}\""),
                )
            };
            let a = package(Some(&definition(a_value)), None);
            let mut b = package(Some(&definition(b_value)), None);
            assert_eq!(
                union_comments_xml(&mut b, MAIN, &a),
                HashMap::from([("7".into(), "8".into())]),
                "{attribute}",
            );
            let expected = |value: &str| {
                let (date, initials) = if attribute == "date" {
                    (value, "")
                } else {
                    ("", value)
                };
                format!("same\0\0{date}\0{initials}")
            };
            assert_eq!(
                comment_id_fingerprint_of(&b),
                HashMap::from([
                    ("7".into(), expected(b_value)),
                    ("8".into(), expected(a_value))
                ]),
                "{attribute}",
            );
        }
    }

    #[test]
    fn carry_comments_remaps_colliding_anchor_ids_and_preserves_definition_metadata() {
        let source_body = concat!(
            "<w:p><w:commentRangeStart w:id=\"7\"/>",
            "<w:r><w:t>same</w:t></w:r><w:commentRangeEnd w:id=\"7\"/>",
            "<w:r><w:commentReference w:id=\"7\"/></w:r></w:p>",
        );
        let definition = |author: &str, date: &str, initials: &str| {
            comment("7", "same").replace(
                "w:id=\"7\"",
                &format!(
                    "w:id=\"7\" w:author=\"{author}\" w:date=\"{date}\" w:initials=\"{initials}\""
                ),
            )
        };
        let a = package(
            Some(&definition("Alice", "2026-10-07T00:00:00Z", "AA")),
            Some(source_body),
        );
        let b = package(
            Some(&definition("Bob", "2026-10-08T00:00:00Z", "BB")),
            Some(source_body),
        );
        let mut out = package(None, None);
        let (mut dom, root) = parse("<w:p><w:r><w:t>same</w:t></w:r></w:p>");
        carry_comments(
            &mut dom,
            root,
            (&a, MAIN),
            (&b, MAIN),
            (&mut out, MAIN),
            "Redline",
        );
        assert_eq!(comment_ids_of(&out), set(&["7", "8"]));
        assert_eq!(
            comment_id_fingerprint_of(&out),
            HashMap::from([
                ("7".into(), "same\0Bob\x002026-10-08T00:00:00Z\0BB".into()),
                ("8".into(), "same\0Alice\x002026-10-07T00:00:00Z\0AA".into()),
            ]),
        );
        for marker in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            let ids: Vec<_> = dom
                .descendants(root, Some(&W::name(marker)))
                .into_iter()
                .map(|node| dom.attribute(node, &W::name("id")).unwrap().to_string())
                .collect();
            assert_eq!(ids.len(), 2, "{marker}");
            assert_eq!(
                ids.into_iter().collect::<HashSet<_>>(),
                set(&["7", "8"]),
                "{marker}"
            );
        }
        assert_eq!(
            collect_segments(&dom, root, true, "Redline", false).0,
            "same"
        );
        assert_eq!(
            collect_segments(&dom, root, false, "Redline", false).0,
            "same"
        );
    }
}
