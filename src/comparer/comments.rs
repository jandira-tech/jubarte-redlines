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

/// Mapped output intervals keyed by the final comment id. Unmappable comments
/// are absent and therefore fall to orphan cleanup.
type AnchorInterval = (usize, usize);
type AnchoredRanges = HashMap<String, AnchorInterval>;
type FingerprintGroups = HashMap<String, Vec<(String, AnchorInterval)>>;

/// Inject one side's anchor events into the merged body. Returns the ids and
/// mapped intervals that were anchored.
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
) -> AnchoredRanges {
    let Some((src_text, ranges)) = extract_events(src_pkg, src_main) else {
        return HashMap::new();
    };
    if ranges.is_empty() {
        return HashMap::new();
    }
    let (merged_text, mut segs) = collect_segments(dom, result_root, b_side, author);
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
            }
        }
    }
    anchored_ranges
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

fn is_namespace_qname_list(name: &XName) -> bool {
    if name.namespace_name().is_empty() {
        return name.local_name() == "Requires";
    }
    name.namespace_name() == MC::URI
        && matches!(
            name.local_name(),
            "Ignorable"
                | "PreserveAttributes"
                | "PreserveElements"
                | "ProcessContent"
                | "MustUnderstand"
        )
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
        for (name, value) in dom.attributes(element) {
            if !dom.is_namespace_declaration(&name) && !name.namespace_name().is_empty() {
                used_uris.insert(name.namespace_name().to_string());
            }
            if is_namespace_qname_list(&name) {
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
            for (name, value) in dom.attributes(element) {
                if !is_namespace_qname_list(&name) {
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
                let target = part.strip_prefix("word/").unwrap_or(part);
                out.add_document_relationship(out_main, rel_type, target);
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
/// Equal bodies on distinct non-empty ranges are independent comments. Exact
/// duplicate anchors collapse deterministically, and a live non-empty anchor
/// supersedes a stale zero-length revision copy of the same comment.
fn select_anchor_aware_comments(out: &PartFs, anchored: &AnchoredRanges) -> HashSet<String> {
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
        let mut seen_ranges = HashSet::new();
        for (id, range) in candidates {
            if has_nonempty && range.0 == range.1 {
                continue;
            }
            if seen_ranges.insert(*range) {
                keep.insert(id.clone());
            }
        }
    }
    keep
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
    let anchored = select_anchor_aware_comments(out, &anchored);
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
