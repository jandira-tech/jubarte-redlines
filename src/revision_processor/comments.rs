// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Comments after Accept All / Reject All, as Word saves them.
//!
//! A nonempty comment range whose characters all go with accepted deletions
//! (or rejected insertions) is removed before resolving the story, even when
//! its reference would survive. Originally empty and partially surviving ranges
//! stay. A comment whose reference goes is also gone: its range markers, its `comments.xml`
//! entry and its `commentsExtended` / `commentsIds` / `commentsExtensible`
//! entries; with none left, the comment parts are not written. The comments
//! left keep their ids, as their bookmarks do: Word's save numbers them
//! afresh, but an id is what a later reply or edit names (#1264).
//! `people.xml` keeps the authors of the comments left (no revision
//! survives Accept or Reject All), and goes when it keeps nobody.

use std::collections::{HashMap, HashSet};

use crate::comparer::comments::FAMILY;
use crate::namespaces::{W, W14, W15};
use crate::opc::PartFs;
use crate::xmllinq::{Dom, NodeId, XName};

const CID: &str = "http://schemas.microsoft.com/office/word/2016/wordml/cid";
const CEX: &str = "http://schemas.microsoft.com/office/word/2018/wordml/cex";
const PEOPLE: (&str, &str) = (
    "word/people.xml",
    "http://schemas.microsoft.com/office/2011/relationships/people",
);

/// One parsed part, written back by [`Parsed::store`].
pub(super) struct Parsed {
    name: String,
    pub(super) dom: Dom,
    doc: NodeId,
    pub(super) root: NodeId,
}

impl Parsed {
    pub(super) fn load(pkg: &PartFs, name: &str) -> Option<Self> {
        let xml = pkg.part_string(name)?;
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc)?;
        Some(Self {
            name: name.to_string(),
            dom,
            doc,
            root,
        })
    }

    pub(super) fn store(self, pkg: &mut PartFs) {
        pkg.set_part(
            &self.name,
            self.dom.serialize_document(self.doc).into_bytes(),
        );
    }

    /// Descendants named `local` in the W namespace.
    pub(super) fn w(&self, local: &str) -> Vec<NodeId> {
        self.dom.descendants(self.root, Some(&W::name(local)))
    }
}

/// Remove every comment range whose entire content is deleted, including
/// its reference run. Replies have their own ranges beside the parent's in
/// Word's shape; they go with the same content. Package pruning removes all
/// corresponding definitions and metadata after story resolution.
pub(super) fn drop_wholly_deleted_comments(dom: &mut Dom, root: NodeId) {
    let dropped = super::bookmarks::wholly_deleted_ranges(
        dom,
        root,
        &W::comment_range_start(),
        &W::comment_range_end(),
    );
    if dropped.is_empty() {
        return;
    }
    for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
        for marker in dom.descendants(root, Some(&W::name(local))) {
            if dom
                .attribute(marker, &W::id())
                .is_some_and(|id| dropped.contains(id))
            {
                remove_marker(dom, marker);
            }
        }
    }
}

/// Collapse wholly removed ranges at their original position before deletion.
/// The scan also serves rejection, after its revisions have been reversed.
/// Partial ranges are deliberately left to the ordinary settlement rules.
pub(super) fn keep_wholly_deleted_comments(dom: &mut Dom, root: NodeId) -> Vec<String> {
    let dropped = super::bookmarks::wholly_deleted_ranges(
        dom,
        root,
        &W::comment_range_start(),
        &W::comment_range_end(),
    );
    let order = dom.descendants(root, None);
    let mut groups: Vec<(NodeId, Vec<String>)> = Vec::new();
    let mut destinations = Vec::new();
    for start in dom.descendants(root, Some(&W::comment_range_start())) {
        let Some(id) = dom
            .attribute(start, &W::id())
            .filter(|id| dropped.contains(*id))
            .map(str::to_string)
        else {
            continue;
        };
        // Place the empty range among paragraph children, outside every
        // revision/hyperlink wrapper that is about to disappear.
        let at = order
            .iter()
            .position(|&n| n == start)
            .expect("range start in story");
        let first_character = order[at + 1..]
            .iter()
            .copied()
            .take_while(|&n| {
                !(dom.name(n) == Some(W::comment_range_end())
                    && dom.attribute(n, &W::id()) == Some(id.as_str()))
            })
            .find(|&n| {
                dom.name(n)
                    .is_some_and(|name| super::bookmarks::is_character(dom, n, &name))
            });
        let mut boundary = first_character.unwrap_or(start);
        while let Some(parent) = dom.parent(boundary) {
            if dom.name(parent) == Some(W::p()) {
                break;
            }
            boundary = parent;
        }
        // A matched move range later discards plain runs between its markers.
        // Put the rescued range before that range too, so the same empty
        // anchor survives both the wrapper deletion (`moveFrom`) and the
        // plain-run deletion the range marks. (#1279)
        let character_at = order
            .iter()
            .position(|&n| n == boundary)
            .expect("character in story");
        if let Some(range_start) = order[..character_at]
            .iter()
            .enumerate()
            .find_map(|(i, &n)| {
                if dom.name(n) != Some(W::move_from_range_start()) {
                    return None;
                }
                let id = dom.attribute(n, &W::id())?;
                let end = order[i + 1..].iter().position(|&e| {
                    dom.name(e) == Some(W::move_from_range_end())
                        && dom.attribute(e, &W::id()) == Some(id)
                })? + i
                    + 1;
                (end > character_at).then_some(n)
            })
        {
            boundary = range_start;
        }
        let Some(paragraph) = dom.parent(boundary) else {
            continue;
        };
        // A row/table is dropped before its children are visited, and a
        // deleted cell may be dropped or merged. Rescue outside the outermost
        // such container, never into another paragraph that is going too.
        let destination = removed_container(dom, paragraph).and_then(|container| {
            let at = order.iter().position(|&n| n == container)?;
            let survives =
                |n: NodeId| dom.name(n) == Some(W::p()) && removed_container(dom, n).is_none();
            order[..at]
                .iter()
                .rev()
                .copied()
                .find(|&n| survives(n))
                .map(|p| (p, false))
                .or_else(|| {
                    order[at + 1..]
                        .iter()
                        .copied()
                        .find(|&n| survives(n))
                        .map(|p| (p, true))
                })
        });
        if removed_container(dom, paragraph).is_some() && destination.is_none() {
            continue;
        }
        if let Some((paragraph, at_start)) = destination {
            // A temporary paragraph child provides the same insertion point
            // used by an in-paragraph collapse. pPr must remain first.
            if let Some((_, point)) = destinations
                .iter()
                .find(|(key, _)| *key == (paragraph, at_start))
            {
                boundary = *point;
            } else {
                let point = dom.new_element(W::r());
                if at_start
                    && let Some(first) = dom
                        .elements(paragraph, None)
                        .into_iter()
                        .find(|&n| dom.name(n) != Some(W::p_pr()))
                {
                    dom.add_before_self(first, point);
                } else {
                    dom.add(paragraph, point);
                }
                destinations.push(((paragraph, at_start), point));
                boundary = point;
            }
        }
        if let Some((_, ids)) = groups.iter_mut().find(|(at, _)| *at == boundary) {
            ids.push(id);
        } else {
            groups.push((boundary, vec![id]));
        }
    }
    let mut kept = Vec::new();
    for (boundary, ids) in groups {
        // A temporary sibling survives removing a boundary that is itself a
        // marker; it is removed after all starts, ends and references move.
        let point = dom.new_element(W::r());
        dom.add_before_self(boundary, point);
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for marker in dom.descendants(root, Some(&W::name(local))) {
                if !dom
                    .attribute(marker, &W::id())
                    .is_some_and(|id| ids.iter().any(|i| i == id))
                {
                    continue;
                }
                if local == "commentReference" {
                    // Copy just the reference and its run properties: a
                    // shared text run must never carry deleted text along.
                    let run = dom.new_element(W::r());
                    if let Some(parent) = dom.parent(marker)
                        && let Some(pr) = dom.element(parent, &W::r_pr())
                    {
                        let pr = dom.clone_subtree(pr);
                        dom.add(run, pr);
                    }
                    let reference = dom.clone_subtree(marker);
                    dom.add(run, reference);
                    remove_marker(dom, marker);
                    dom.add_before_self(point, run);
                } else {
                    dom.remove(marker);
                    dom.add_before_self(point, marker);
                }
            }
        }
        dom.remove(point);
        kept.extend(ids);
    }
    for (_, point) in destinations {
        dom.remove(point);
    }
    kept
}

/// The outermost container removed by the trailing accept pass. Rejection
/// reverses ins/del (including row and cell marks) before this scan.
fn removed_container(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let mut removed = None;
    let mut at = Some(node);
    while let Some(n) = at {
        let name = dom.name(n);
        if (name == Some(W::tr()) && super::has_path(dom, n, &W::tr_pr(), &W::del()))
            || (name == Some(W::tc()) && super::has_path(dom, n, &W::tc_pr(), &W::cell_del()))
            || (name == Some(W::tbl()) && super::table_rows_all_deleted(dom, n))
        {
            removed = Some(n);
        }
        at = dom.parent(n);
    }
    removed
}

/// A reference-only run goes with its marker; a run containing other content
/// keeps that content. Range markers normally are siblings of runs.
fn remove_marker(dom: &mut Dom, marker: NodeId) {
    let run = dom.parent(marker).filter(|&p| dom.name(p) == Some(W::r()));
    dom.remove(marker);
    if let Some(run) = run
        && dom
            .elements(run, None)
            .into_iter()
            .all(|e| dom.name(e) == Some(W::r_pr()))
    {
        dom.remove(run);
    }
}

/// Publish the rescued threads (`kept`, source `w:id`s) on the lowest free
/// comment ids, so a `kept c8` receipt's document shows `c0` (#1279).
/// Survivors keep their ids, as every settlement keeps them (#1264); a
/// rescued thread keeps its source id only when it is still the lowest
/// free one. Skipped while any revision is pending: surviving annotations
/// still name the changes that hold them.
pub(crate) fn renumber_rescued_comments(pkg: &mut PartFs, kept: &[String]) {
    if kept.is_empty() {
        return;
    }
    let kept_ids: HashSet<String> = kept.iter().cloned().collect();
    let mut story_parts: Vec<String> = super::revision_bearing_parts(pkg)
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    story_parts.sort();
    story_parts.dedup();
    // Story order decides: the first-seen rescued id gets the lowest free
    // published id, so a thread's root keeps a lower id than its reply.
    let mut used: HashSet<u32> = HashSet::new();
    let mut story_order: Vec<u32> = Vec::new();
    for part in &story_parts {
        let Some(story) = Parsed::load(pkg, part) else {
            continue;
        };
        if super::element_has_tracked_revisions(&story.dom, story.root) {
            return;
        }
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for m in story.w(local) {
                let Some(id_str) = story.dom.attribute(m, &W::id()) else {
                    continue;
                };
                let Ok(id) = id_str.parse::<u32>() else {
                    continue;
                };
                used.insert(id);
                if kept_ids.contains(id_str) && !story_order.contains(&id) {
                    story_order.push(id);
                }
            }
        }
    }
    if story_order.is_empty() {
        return;
    }
    let Some(mut comments_xml) = Parsed::load(pkg, FAMILY[0].0) else {
        return;
    };
    for c in comments_xml.w("comment") {
        let Some(id_str) = comments_xml.dom.attribute(c, &W::id()) else {
            continue;
        };
        let Ok(id) = id_str.parse::<u32>() else {
            continue;
        };
        if !story_order.contains(&id) {
            used.insert(id);
        }
    }
    // A rescued thread may keep its own source id even though `used` holds
    // it; every other id in `used` is reserved.
    let mut rename: HashMap<u32, u32> = HashMap::new();
    for source in &story_order {
        let mut next = 0u32;
        while next != *source && used.contains(&next) {
            next += 1;
        }
        if next != *source {
            rename.insert(*source, next);
        }
        used.insert(next);
    }
    if rename.is_empty() {
        return;
    }
    // Apply the rename to stories and comments.xml.
    for part in &story_parts {
        let Some(mut story) = Parsed::load(pkg, part) else {
            continue;
        };
        let mut changed = false;
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for m in story.w(local) {
                let Some(value) = story.dom.attribute(m, &W::id()) else {
                    continue;
                };
                let Ok(value) = value.parse::<u32>() else {
                    continue;
                };
                if let Some(&new) = rename.get(&value) {
                    story
                        .dom
                        .set_attribute_value(m, &W::id(), Some(&new.to_string()));
                    changed = true;
                }
            }
        }
        if changed {
            story.store(pkg);
        }
    }
    let mut comments_changed = false;
    for c in comments_xml.w("comment") {
        let Some(value) = comments_xml.dom.attribute(c, &W::id()) else {
            continue;
        };
        let Ok(value) = value.parse::<u32>() else {
            continue;
        };
        if let Some(&new) = rename.get(&value) {
            comments_xml
                .dom
                .set_attribute_value(c, &W::id(), Some(&new.to_string()));
            comments_changed = true;
        }
    }
    if comments_changed {
        comments_xml.store(pkg);
    }
}

/// Drop the comments Accept / Reject All left without a reference, prune the
/// comment family and `people.xml` (module docs). Return their ids, so
/// callers can report precisely what went.
pub(crate) fn prune_orphan_comments(pkg: &mut PartFs, story_parts: &[String]) -> Vec<String> {
    let Some(mut comments) = Parsed::load(pkg, FAMILY[0].0) else {
        return Vec::new();
    };
    // The comments part itself is not an anchor story: a `commentReference`
    // inside a comment body keeps no comment alive, and pruning this Parsed
    // copy would race the `comments` one. Marker fix-ups reach it through
    // `word_save::tidy` instead.
    let stories: Vec<Parsed> = story_parts
        .iter()
        .filter(|p| p.as_str() != FAMILY[0].0)
        .filter_map(|p| Parsed::load(pkg, p))
        .collect();
    let id = W::id();
    let referenced: HashSet<String> = stories
        .iter()
        .flat_map(|s| {
            s.w("commentReference")
                .into_iter()
                .filter_map(|r| s.dom.attribute(r, &id).map(str::to_string))
                .collect::<Vec<_>>()
        })
        .collect();

    let mut kept: HashSet<String> = HashSet::new();
    let mut dropped = false;
    let mut removed = Vec::new();
    let mut dropped_paras: HashSet<String> = HashSet::new();
    let mut authors: HashSet<String> = HashSet::new();
    let para_id = W14::name("paraId");
    for c in comments.w("comment") {
        let cid = comments.dom.attribute(c, &id).unwrap_or("").to_string();
        if referenced.contains(&cid) {
            kept.insert(cid);
            if let Some(a) = comments.dom.attribute(c, &W::author()) {
                authors.insert(a.to_string());
            }
        } else {
            removed.push(cid);
            for p in comments.dom.descendants(c, Some(&W::p())) {
                if let Some(pid) = comments.dom.attribute(p, &para_id) {
                    dropped_paras.insert(pid.to_string());
                }
            }
            comments.dom.remove(c);
            dropped = true;
        }
    }
    // Anchors of the dropped comments go (their commentReference went with
    // the deleted text by construction).
    for mut s in stories {
        let mut changed = false;
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for m in s.w(local) {
                if !s.dom.attribute(m, &id).is_some_and(|v| kept.contains(v)) {
                    remove_marker(&mut s.dom, m);
                    changed = true;
                }
            }
        }
        changed |= collapse_cut_ranges(&mut s);
        if changed {
            s.store(pkg);
        }
    }

    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    if kept.is_empty() {
        for (part, _, rel_type) in FAMILY {
            remove_part(pkg, &main, part, rel_type);
        }
    } else if dropped {
        // An untouched part keeps its bytes.
        comments.store(pkg);
        prune_by_para(pkg, &dropped_paras);
    }
    prune_people(pkg, &main, &authors);
    removed
}

/// A comment whose reference stayed while resolved content took one end of
/// its range (resolving some changes and keeping others can cut a range
/// that way) gets the missing marker back where the range was cut short:
/// the start just before the surviving end, the end just before the
/// reference's run. True when a marker was added.
fn collapse_cut_ranges(s: &mut Parsed) -> bool {
    let id = W::id();
    let ids_of = |s: &Parsed, local: &str| -> HashSet<String> {
        s.w(local)
            .into_iter()
            .filter_map(|m| s.dom.attribute(m, &id).map(str::to_string))
            .collect()
    };
    let (starts, ends) = (ids_of(s, "commentRangeStart"), ids_of(s, "commentRangeEnd"));
    let mut changed = false;
    for reference in s.w("commentReference") {
        let Some(cid) = s.dom.attribute(reference, &id).map(str::to_string) else {
            continue;
        };
        let run = s
            .dom
            .parent(reference)
            .filter(|&r| s.dom.name(r) == Some(W::r()))
            .unwrap_or(reference);
        let end = if ends.contains(&cid) {
            s.w("commentRangeEnd")
                .into_iter()
                .find(|&e| s.dom.attribute(e, &id) == Some(cid.as_str()))
        } else if starts.contains(&cid) {
            let e = marker(&mut s.dom, "commentRangeEnd", &cid);
            s.dom.add_before_self(run, e);
            changed = true;
            Some(e)
        } else {
            None
        };
        if !starts.contains(&cid)
            && let Some(end) = end
        {
            let start = marker(&mut s.dom, "commentRangeStart", &cid);
            s.dom.add_before_self(end, start);
            changed = true;
        }
    }
    changed
}

fn marker(dom: &mut Dom, local: &str, cid: &str) -> NodeId {
    let m = dom.new_element(W::name(local));
    dom.set_attribute_value(m, &W::id(), Some(cid));
    m
}

/// Drop the commentsExtended / commentsIds entries of the dropped comments'
/// paragraphs and the commentsExtensible entries of their durable ids.
fn prune_by_para(pkg: &mut PartFs, dropped_paras: &HashSet<String>) {
    if dropped_paras.is_empty() {
        return;
    }
    if let Some(mut ext) = Parsed::load(pkg, FAMILY[1].0) {
        let key = W15::name("paraId");
        for e in ext.dom.descendants(ext.root, Some(&W15::name("commentEx"))) {
            if ext
                .dom
                .attribute(e, &key)
                .is_some_and(|p| dropped_paras.contains(p))
            {
                ext.dom.remove(e);
            }
        }
        ext.store(pkg);
    }
    let mut durable: HashSet<String> = HashSet::new();
    if let Some(mut ids) = Parsed::load(pkg, FAMILY[2].0) {
        let (key, dur) = (XName::get("paraId", CID), XName::get("durableId", CID));
        for e in ids
            .dom
            .descendants(ids.root, Some(&XName::get("commentId", CID)))
        {
            if ids
                .dom
                .attribute(e, &key)
                .is_some_and(|p| dropped_paras.contains(p))
            {
                if let Some(d) = ids.dom.attribute(e, &dur) {
                    durable.insert(d.to_string());
                }
                ids.dom.remove(e);
            }
        }
        ids.store(pkg);
    }
    if let Some(mut cex) = Parsed::load(pkg, FAMILY[3].0) {
        let key = XName::get("durableId", CEX);
        for e in cex
            .dom
            .descendants(cex.root, Some(&XName::get("commentExtensible", CEX)))
        {
            if cex
                .dom
                .attribute(e, &key)
                .is_some_and(|d| durable.contains(d))
            {
                cex.dom.remove(e);
            }
        }
        cex.store(pkg);
    }
}

/// Keep the people who authored a comment left; drop the part when none is.
fn prune_people(pkg: &mut PartFs, main: &str, authors: &HashSet<String>) {
    let Some(mut people) = Parsed::load(pkg, PEOPLE.0) else {
        return;
    };
    let author = W15::name("author");
    let mut left = 0;
    for p in people
        .dom
        .descendants(people.root, Some(&W15::name("person")))
    {
        if people
            .dom
            .attribute(p, &author)
            .is_some_and(|a| authors.contains(a))
        {
            left += 1;
        } else {
            people.dom.remove(p);
        }
    }
    if left == 0 {
        remove_part(pkg, main, PEOPLE.0, PEOPLE.1);
    } else {
        people.store(pkg);
    }
}

/// Remove `part`, the main part's relationship to it and its content type.
fn remove_part(pkg: &mut PartFs, main: &str, part: &str, rel_type: &str) {
    if pkg.part_bytes(part).is_none() {
        return;
    }
    pkg.remove_part(part);
    pkg.remove_relationships_by_type(main, rel_type);
    pkg.remove_content_type_override(&format!("/{part}"));
}
