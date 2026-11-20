// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Comments: range markers, anchors and reply markers, and the comment
//! references a deletion takes along.

use super::*;

/// Comment `id`'s markers under `scope`, anchored on words a deletion then
/// took: the range start leaves the deletion it opens in, and the range
/// end, with the reference run after it, the one it closes in. A marker in
/// the middle of a deletion (the comparer joins deletions that touch into
/// one) splits it in two, the second part under the fresh revision id
/// `next_id`. So the comment wraps the deleted words, as a strike's does,
/// and accepting the deletion keeps the comment's reference.
fn lift_out_of_deletion(dom: &mut Dom, scope: NodeId, id: u32, next_id: &mut u64) {
    let id = id.to_string();
    let marker = |dom: &Dom, local: &str| {
        dom.descendants(scope, Some(&W::name(local)))
            .into_iter()
            .find(|&m| dom.attribute(m, &W::id()) == Some(id.as_str()))
    };
    if let Some(start) = marker(dom, "commentRangeStart") {
        while let Some(del) = dom.parent(start).filter(|&p| dom.name_is(p, &W::del())) {
            if dom.elements(del, None).first() != Some(&start) {
                split_deletion_before(dom, del, start, next_id);
                continue;
            }
            dom.remove(start);
            dom.add_before_self(del, start);
            drop_if_empty(dom, del);
        }
    }
    let Some(end) = marker(dom, "commentRangeEnd") else {
        return;
    };
    let reference = dom.next_element(end).filter(|&run| {
        dom.element(run, &W::comment_reference())
            .is_some_and(|r| dom.attribute(r, &W::id()) == Some(id.as_str()))
    });
    let last = reference.unwrap_or(end);
    while let Some(del) = dom.parent(end).filter(|&p| dom.name_is(p, &W::del())) {
        if dom.elements(del, None).last() != Some(&last) {
            if let Some(&after) = dom.nodes_after_self(last).first() {
                split_deletion_before(dom, del, after, next_id);
            }
            continue;
        }
        dom.remove(end);
        dom.add_after_self(del, end);
        if let Some(run) = reference {
            dom.remove(run);
            dom.add_after_self(end, run);
        }
        drop_if_empty(dom, del);
    }
}

/// Split deletion `del` before its child `at`: `at` and what follows it
/// move into a copy of `del` right after it, under revision id `next_id`.
fn split_deletion_before(dom: &mut Dom, del: NodeId, at: NodeId, next_id: &mut u64) {
    let tail = dom.new_element(W::del());
    for (name, value) in dom.attributes(del) {
        dom.set_attribute_value(tail, &name, Some(&value));
    }
    dom.set_attribute_value(tail, &W::id(), Some(&next_id.to_string()));
    *next_id += 1;
    let mut moved = vec![at];
    moved.extend(dom.nodes_after_self(at));
    for node in moved {
        dom.remove(node);
        dom.add(tail, node);
    }
    dom.add_after_self(del, tail);
}

/// Remove deletion `del` once a lifted marker leaves it no element.
fn drop_if_empty(dom: &mut Dom, del: NodeId) {
    if dom.elements(del, None).is_empty() {
        dom.remove(del);
    }
}

/// One more than the largest `w:id` in the stories of `opened`: a revision
/// id no annotation holds.
fn next_annotation_id(opened: &Opened) -> u64 {
    let largest = |dom: &Dom, root: NodeId| {
        dom.descendants(root, None)
            .into_iter()
            .filter_map(|n| dom.attribute(n, &W::id())?.parse::<u64>().ok())
            .max()
            .unwrap_or(0)
    };
    let mut max = largest(&opened.dom, opened.document);
    for (_, _, part) in opened.story_parts() {
        if let Ok((dom, document, _)) = crate::inspect::parse_part(&opened.pkg, &part) {
            max = max.max(largest(&dom, document));
        }
    }
    max + 1
}

/// `redline` with the markers of each comment in `ids`, on words the
/// comparer deleted, lifted out of those deletions
/// ([`lift_out_of_deletion`]): the comment wraps the deletion,
/// so accepting it keeps the comment's reference.
pub(super) fn lift_deletion_comments(redline: &[u8], ids: &[u32]) -> Result<Vec<u8>, EditError> {
    if ids.is_empty() {
        return Ok(redline.to_vec());
    }
    let mut red = Opened::open(redline).map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
    let mut next_id = next_annotation_id(&red);
    for &id in ids {
        lift_out_of_deletion(&mut red.dom, red.body, id, &mut next_id);
    }
    let xml = red.dom.serialize_document(red.document);
    let main = red.main.clone();
    red.pkg.set_part(&main, xml.into_bytes());
    red.pkg
        .to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
}

/// A paragraph the plan deleted whole: its mark carries the plan author's
/// deletion. Comment reference runs the comparer left beside that deletion
/// join it, so Accept drops the thread instead of parking an empty one on
/// the next paragraph.
pub(super) fn enclose_deleted_paragraph_comment_references(
    redline: &[u8],
    author: &str,
) -> Result<Vec<u8>, EditError> {
    let mut red = Opened::open(redline).map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))?;
    let body_changed = enclose_comment_references(&mut red.dom, red.body, author);
    let stories = red.story_parts();
    let mut rewritten = Vec::new();
    for (_, _, part) in &stories {
        let Some(xml) = red.pkg.part_string(part) else {
            continue;
        };
        let document = red.dom.parse_xdocument(&xml);
        let Some(root) = red.dom.root(document) else {
            continue;
        };
        if enclose_comment_references(&mut red.dom, root, author) {
            rewritten.push((part.clone(), red.dom.serialize_document(document)));
        }
    }
    if !body_changed && rewritten.is_empty() {
        return Ok(redline.to_vec());
    }
    if body_changed {
        let main = red.main.clone();
        let xml = red.dom.serialize_document(red.document);
        red.pkg.set_part(&main, xml.into_bytes());
    }
    for (part, xml) in rewritten {
        red.pkg.set_part(&part, xml.into_bytes());
    }
    red.pkg
        .to_zip()
        .map_err(|e| err("PACKAGE_WRITE", None, e.to_string()))
}

/// Move comment-reference runs that belong only to `root`'s wholly deleted
/// paragraphs into that paragraph's content deletion. Returns whether the
/// tree changed.
fn enclose_comment_references(dom: &mut Dom, root: NodeId, author: &str) -> bool {
    let moves = comment_reference_moves(dom, root, author);
    let changed = !moves.is_empty();
    for (run, del) in moves {
        dom.remove(run);
        dom.add(del, run);
    }
    changed
}

/// The reference runs `enclose_comment_references` moves, each with the
/// deletion it joins. One walk of `root` maps every comment to the
/// paragraphs holding its anchors, so each reference is checked without
/// walking the story again.
fn comment_reference_moves(dom: &Dom, root: NodeId, author: &str) -> Vec<(NodeId, NodeId)> {
    let mut deleted = Vec::new();
    dom.for_each_descendant_element(root, Some(&W::p()), |paragraph| {
        // A paragraph already inside a deletion (a text box in a deleted
        // run) goes with that deletion.
        if deleted_mark_author(dom, paragraph) == Some(author) && !inside_deletion(dom, paragraph) {
            deleted.push(paragraph);
        }
    });
    if deleted.is_empty() {
        return Vec::new();
    }
    let anchors = comment_anchor_paragraphs(dom, root);
    let mut moves = Vec::new();
    for paragraph in deleted {
        let Some(del) = content_deletion(dom, paragraph, author) else {
            continue;
        };
        let mut seen = HashSet::new();
        for index in 0..dom.child_count(paragraph) {
            let run = dom.child_at(paragraph, index);
            if !dom.name_is(run, &W::r()) {
                continue;
            }
            let Some(id) = reference_id(dom, run) else {
                continue;
            };
            if !seen.insert(id) {
                continue;
            }
            // Every anchor of the comment sits in this paragraph.
            let only_here = anchors
                .get(id)
                .is_some_and(|held| held.iter().all(|&at| at == Some(paragraph)));
            if only_here {
                moves.push((run, del));
            }
        }
    }
    moves
}

/// Each comment id under `root`, with the paragraph holding each of its
/// anchors (`None` for an anchor outside any paragraph).
fn comment_anchor_paragraphs(dom: &Dom, root: NodeId) -> HashMap<&str, Vec<Option<NodeId>>> {
    let (start, end, reference) = (
        W::comment_range_start(),
        W::comment_range_end(),
        W::comment_reference(),
    );
    let mut anchors: HashMap<&str, Vec<Option<NodeId>>> = HashMap::new();
    dom.for_each_descendant_element(root, None, |marker| {
        if !(dom.name_is(marker, &start)
            || dom.name_is(marker, &end)
            || dom.name_is(marker, &reference))
        {
            return;
        }
        if let Some(id) = dom.attribute(marker, &W::id()) {
            anchors
                .entry(id)
                .or_default()
                .push(enclosing_paragraph(dom, marker));
        }
    });
    anchors
}

/// The nearest `w:p` around `node`.
fn enclosing_paragraph(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let mut current = dom.parent(node);
    while let Some(at) = current {
        if dom.name_is(at, &W::p()) {
            return Some(at);
        }
        current = dom.parent(at);
    }
    None
}

/// Author of the paragraph mark's `w:del` or `w:moveFrom`, when it has one.
fn deleted_mark_author(dom: &Dom, paragraph: NodeId) -> Option<&str> {
    let props = dom
        .element(paragraph, &W::p_pr())
        .and_then(|props| dom.element(props, &W::r_pr()))?;
    let mark = (0..dom.child_count(props))
        .map(|index| dom.child_at(props, index))
        .find(|&child| is_deletion(dom, child))?;
    dom.attribute(mark, &W::author())
}

/// The plan author's content `w:del` in `paragraph`, preferring the last
/// one that holds deleted text.
fn content_deletion(dom: &Dom, paragraph: NodeId, author: &str) -> Option<NodeId> {
    let mut last = None;
    for index in (0..dom.child_count(paragraph)).rev() {
        let del = dom.child_at(paragraph, index);
        if !dom.name_is(del, &W::del()) || dom.attribute(del, &W::author()) != Some(author) {
            continue;
        }
        let holds_text = dom
            .find_descendant_element(del, Some(&W::del_text()), |text| {
                !dom.value(text).is_empty()
            })
            .is_some();
        if holds_text {
            return Some(del);
        }
        last.get_or_insert(del);
    }
    last
}

/// `w:id` when `run` is only a comment reference (properties allowed).
fn reference_id(dom: &Dom, run: NodeId) -> Option<&str> {
    let mut id = None;
    for index in 0..dom.child_count(run) {
        let child = dom.child_at(run, index);
        if !dom.is_element(child) {
            continue;
        }
        if dom.name_is(child, &W::r_pr()) {
            continue;
        }
        if !dom.name_is(child, &W::comment_reference()) || id.is_some() {
            return None;
        }
        id = dom.attribute(child, &W::id());
    }
    id
}

/// `node`, or an element around it, is a `w:del` or a `w:moveFrom`.
fn inside_deletion(dom: &Dom, node: NodeId) -> bool {
    let mut current = Some(node);
    while let Some(at) = current {
        if is_deletion(dom, at) {
            return true;
        }
        current = dom.parent(at);
    }
    false
}

/// `node` is a `w:del` or a `w:moveFrom`.
fn is_deletion(dom: &Dom, node: NodeId) -> bool {
    dom.name_is(node, &W::del()) || dom.name_is(node, &W::move_from())
}

/// Whether comment `id`'s range in `redline` holds text that is not deleted.
pub(super) fn comment_holds_kept_text(redline: &[u8], id: u32) -> bool {
    let Ok(opened) = Opened::open(redline) else {
        return false;
    };
    let id = id.to_string();
    let parts = std::iter::once(opened.main.clone())
        .chain(opened.story_parts().into_iter().map(|(_, _, part)| part));
    for part in parts {
        let Ok((dom, _, root)) = crate::inspect::parse_part(&opened.pkg, &part) else {
            continue;
        };
        let mut inside = false;
        for node in dom.descendants(root, None) {
            let Some(name) = dom.name(node).filter(|n| n.namespace_name() == W::URI) else {
                continue;
            };
            let is_id = || dom.attribute(node, &W::id()) == Some(id.as_str());
            match name.local_name() {
                "commentRangeStart" if is_id() => inside = true,
                "commentRangeEnd" if is_id() => inside = false,
                "t" if inside => return true,
                _ => {}
            }
        }
    }
    false
}

pub(super) fn existing_comment_ids(opened: &Opened) -> Option<u32> {
    let name = opened.related("comments").into_iter().next()?;
    let xml = opened.pkg.part_string(&name)?;
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document)?;
    dom.descendants(root, Some(&W::comment()))
        .into_iter()
        .filter_map(|c| {
            dom.attribute(c, &W::id())
                .and_then(|v| v.parse::<u32>().ok())
        })
        .max()
}

/// Markup that may sit between two block paragraphs and also inside one.
pub(super) fn is_range_markup(dom: &Dom, node: NodeId) -> bool {
    const RANGES: &[&str] = &[
        "bookmarkStart",
        "bookmarkEnd",
        "commentRangeStart",
        "commentRangeEnd",
        "permStart",
        "permEnd",
        "proofErr",
    ];
    dom.name(node)
        .is_some_and(|n| n.namespace_name() == W::URI && RANGES.contains(&n.local_name()))
}

/// Wrap the projection range `[start, end)` of `paragraph` in comment
/// markers for comment `id`, splitting runs at the boundaries as needed.
pub(super) fn anchor_comment(
    dom: &mut Dom,
    paragraph: NodeId,
    start: usize,
    end: usize,
    id: u32,
    ids: &mut RevisionCounter,
) {
    let id_str = id.to_string();
    let range_start = dom.new_element(W::comment_range_start());
    dom.set_attribute_value(range_start, &W::id(), Some(&id_str));
    let range_end = dom.new_element(W::comment_range_end());
    dom.set_attribute_value(range_end, &W::id(), Some(&id_str));
    let reference_run = dom.new_element(W::r());
    let reference = dom.new_element(W::comment_reference());
    dom.set_attribute_value(reference, &W::id(), Some(&id_str));
    dom.add(reference_run, reference);
    if wrap_range(dom, paragraph, start, end, range_start, range_end, ids) {
        place_reference(dom, range_start, range_end, reference_run);
    } else {
        dom.add(paragraph, range_start);
        dom.add(paragraph, range_end);
        dom.add(paragraph, reference_run);
    }
}

/// Put a comment's reference run right after its end marker, or, when the
/// range starts outside the tracked insertion holding that marker, after
/// the insertion: rejecting it would take a reference inside it, and the
/// comment with it, though only part of the commented text was inserted.
/// A comment wholly on inserted text keeps its reference inside and goes
/// with the text, as in Word.
fn place_reference(dom: &mut Dom, range_start: NodeId, range_end: NodeId, reference_run: NodeId) {
    let mut after = range_end;
    for ancestor in dom.ancestors(range_end, None) {
        if dom.name_is(ancestor, &W::p()) {
            break;
        }
        if (dom.name_is(ancestor, &W::ins()) || dom.name_is(ancestor, &W::move_to()))
            && !dom.ancestors(range_start, None).contains(&ancestor)
        {
            after = ancestor;
        }
    }
    dom.add_after_self(after, reference_run);
}

pub(super) fn comment_marker(dom: &mut Dom, local: &str, id: &str) -> NodeId {
    let marker = dom.new_element(W::name(local));
    dom.set_attribute_value(marker, &W::id(), Some(id));
    marker
}

pub(super) fn comment_reference_run(dom: &mut Dom, id: &str) -> NodeId {
    let run = dom.new_element(W::r());
    let reference = comment_marker(dom, "commentReference", id);
    dom.add(run, reference);
    run
}

/// Comment `id` from the start of paragraph `first` to the end of `last`:
/// the start marker before `first`'s first run, the end marker and the
/// reference run after `last`'s last run (appended to an empty paragraph).
pub(super) fn anchor_span(
    dom: &mut Dom,
    first: NodeId,
    last: NodeId,
    id: u32,
    ids: &mut RevisionCounter,
) {
    let id = id.to_string();
    let range_start = comment_marker(dom, "commentRangeStart", &id);
    let range_end = comment_marker(dom, "commentRangeEnd", &id);
    let reference_run = comment_reference_run(dom, &id);
    let len = project_paragraph(dom, first).text.len();
    let scratch = dom.new_element(W::comment_range_end());
    if len > 0 && wrap_range(dom, first, 0, len, range_start, scratch, ids) {
        dom.remove(scratch);
    } else {
        match dom.element(first, &W::p_pr()) {
            Some(ppr) => dom.add_after_self(ppr, range_start),
            None => dom.add_first(first, range_start),
        }
    }
    let len = project_paragraph(dom, last).text.len();
    let scratch = dom.new_element(W::comment_range_start());
    if len > 0 && wrap_range(dom, last, 0, len, scratch, range_end, ids) {
        dom.remove(scratch);
        place_reference(dom, range_start, range_end, reference_run);
    } else {
        dom.add(last, range_end);
        dom.add(last, reference_run);
    }
}

/// Markers of reply `id` beside those of comment `parent`, as Word writes
/// them: the start after the parent's start, the end after the parent's
/// end, the reference run after the parent's reference run.
pub(super) fn place_reply_markers(dom: &mut Dom, roots: &[NodeId], parent: u32, id: u32) {
    let parent = parent.to_string();
    let id = id.to_string();
    let find = |dom: &Dom, local: &str| {
        roots.iter().find_map(|&root| {
            dom.descendants(root, Some(&W::name(local)))
                .into_iter()
                .find(|&m| dom.attribute(m, &W::id()) == Some(parent.as_str()))
        })
    };
    let (start, end, reference) = (
        find(dom, "commentRangeStart"),
        find(dom, "commentRangeEnd"),
        find(dom, "commentReference"),
    );
    let Some(reference) = reference else {
        return;
    };
    let parent_run = dom
        .parent(reference)
        .filter(|&r| dom.name_is(r, &W::r()))
        .unwrap_or(reference);
    let new_run = comment_reference_run(dom, &id);
    dom.add_after_self(parent_run, new_run);
    let new_end = comment_marker(dom, "commentRangeEnd", &id);
    match end {
        Some(end) => dom.add_after_self(end, new_end),
        None => dom.add_before_self(new_run, new_end),
    }
    let new_start = comment_marker(dom, "commentRangeStart", &id);
    match start {
        Some(start) => dom.add_after_self(start, new_start),
        None => dom.add_before_self(new_end, new_start),
    }
}

/// Remove the range markers and references of the comments `ids`; a run
/// left holding nothing but its properties goes too.
pub(super) fn remove_comment_markers(dom: &mut Dom, roots: &[NodeId], ids: &[String]) {
    for &root in roots {
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for marker in dom.descendants(root, Some(&W::name(local))) {
                if !dom
                    .attribute(marker, &W::id())
                    .is_some_and(|v| ids.iter().any(|id| id == v))
                {
                    continue;
                }
                let run = dom.parent(marker).filter(|&r| dom.name_is(r, &W::r()));
                dom.remove(marker);
                if let Some(run) = run
                    && dom
                        .elements(run, None)
                        .iter()
                        .all(|&c| dom.name_is(c, &W::r_pr()))
                {
                    dom.remove(run);
                }
            }
        }
    }
}

/// Put `open` right before the run holding projection offset `start` and
/// `close` right after the run ending at `end`, splitting runs at both
/// offsets first. False, with nothing placed, when the range holds no run.
pub(super) fn wrap_range(
    dom: &mut Dom,
    paragraph: NodeId,
    start: usize,
    end: usize,
    open: NodeId,
    close: NodeId,
    ids: &mut RevisionCounter,
) -> bool {
    let projection = project_paragraph(dom, paragraph);
    if let Some(seg) = projection
        .segments
        .iter()
        .find(|s| s.start < start && start < s.end)
        .cloned()
    {
        split_run_at(dom, &seg, start, ids);
    }
    let projection = project_paragraph(dom, paragraph);
    if let Some(seg) = projection
        .segments
        .iter()
        .find(|s| s.start < end && end < s.end)
        .cloned()
    {
        split_run_at(dom, &seg, end, ids);
    }
    let projection = project_paragraph(dom, paragraph);
    let first = projection
        .segments
        .iter()
        .find(|s| s.start >= start && s.start < end.max(start + 1))
        .map(|s| run_of(&s.piece));
    let last = projection
        .segments
        .iter()
        .rev()
        .find(|s| s.end <= end && s.end > start)
        .map(|s| run_of(&s.piece));
    match (first, last) {
        (Some(first), Some(last)) if start < end => {
            dom.add_before_self(first, open);
            dom.add_after_self(last, close);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod reference_branch_tests {
    use super::super::deeper_boundary_fixture::{Part, docx, docx_with_sect, part_string};
    use super::*;

    const MARK: &str = r#"<w:pPr><w:rPr><w:del w:id="90" w:author="Ed" w:date="2026-01-01T00:00:00Z"/></w:rPr></w:pPr>"#;

    fn reference(id: u32) -> String {
        format!(
            r#"<w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="{id}"/></w:r>"#
        )
    }

    fn deleted(text: &str) -> String {
        format!(
            r#"<w:del w:id="91" w:author="Ed" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>{text}</w:delText></w:r></w:del>"#
        )
    }

    /// Whether comment `id`'s reference runs all sit inside a `w:del`.
    fn enclosed(xml: &str, id: u32) -> bool {
        inside(xml, id) == count(xml, id)
    }

    fn count(xml: &str, id: u32) -> usize {
        xml.matches(&format!(r#"<w:commentReference w:id="{id}" />"#))
            .count()
    }

    /// How many of comment `id`'s references sit inside a `w:del`.
    fn inside(xml: &str, id: u32) -> usize {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).unwrap();
        dom.descendants(root, Some(&W::comment_reference()))
            .into_iter()
            .filter(|&r| dom.attribute(r, &W::id()) == Some(id.to_string().as_str()))
            .filter(|&r| dom.ancestors(r, Some(&W::del())).len() == 1)
            .count()
    }

    #[test]
    fn references_join_the_deletion_of_their_wholly_deleted_paragraph() {
        let body = format!(
            // p0: wholly deleted by Ed; comment 1 lives only here (its
            // reference appears twice), comment 2 also anchors in p1, the
            // run with two references and the run with text stay put.
            r#"<w:p>{MARK}<w:commentRangeStart w:id="1"/>{}<w:commentRangeEnd w:id="1"/>{}{}<w:r><w:commentReference w:id="3"/><w:commentReference w:id="4"/></w:r><w:r><w:t>x</w:t><w:commentReference w:id="5"/></w:r><w:commentRangeStart w:id="2"/>{}<w:r><w:t>y</w:t></w:r></w:p><w:p><w:commentRangeEnd w:id="2"/><w:r><w:t>tail</w:t></w:r></w:p>"#,
            deleted("gone"),
            reference(1),
            reference(1),
            reference(2),
        );
        let source = docx(&body);
        let out = enclose_deleted_paragraph_comment_references(&source, "Ed").unwrap();
        let xml = part_string(&out, "word/document.xml").unwrap();
        // The first of comment 1's references joins; its repeat stays.
        assert_eq!((inside(&xml, 1), count(&xml, 1)), (1, 2), "{xml}");
        for id in [2, 3, 4, 5] {
            assert_eq!(inside(&xml, id), 0, "{id}: {xml}");
        }
        // Another author's deleted paragraph is left alone: nothing changes.
        let out = enclose_deleted_paragraph_comment_references(&source, "Bo").unwrap();
        assert_eq!(out, source);
    }

    #[test]
    fn a_deletion_without_text_still_takes_the_reference() {
        let body = format!(
            r#"<w:p>{MARK}<w:del w:id="92" w:author="Ed" w:date="2026-01-01T00:00:00Z"><w:r><w:delText></w:delText></w:r></w:del><w:del w:id="93" w:author="Bo" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>b</w:delText></w:r></w:del><w:commentRangeStart w:id="1"/><w:commentRangeEnd w:id="1"/>{}</w:p><w:p>{MARK}<w:commentRangeStart w:id="6"/><w:commentRangeEnd w:id="6"/>{}</w:p><w:p><w:del w:id="94" w:author="Ed" w:date="2026-01-01T00:00:00Z"><w:r><w:pict><w:p>{MARK}{}{}</w:p></w:pict></w:r></w:del></w:p>"#,
            reference(1),
            reference(6),
            deleted("boxed"),
            reference(8),
        );
        let source = docx(&body);
        let out = enclose_deleted_paragraph_comment_references(&source, "Ed").unwrap();
        let xml = part_string(&out, "word/document.xml").unwrap();
        assert!(enclosed(&xml, 1), "{xml}");
        // No content deletion of the author's: the reference stays.
        assert!(!enclosed(&xml, 6), "{xml}");
    }

    #[test]
    fn header_references_join_their_deleted_paragraph_too() {
        let header = format!(
            r#"<w:hdr xmlns:w="{}"><w:p>{MARK}<w:commentRangeStart w:id="4"/>{}<w:commentRangeEnd w:id="4"/>{}</w:p></w:hdr>"#,
            W::URI,
            deleted("head"),
            reference(4)
        );
        let part = Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        };
        let sect = r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#;
        let source = docx_with_sect("<w:p><w:r><w:t>Body.</w:t></w:r></w:p>", &[part], sect);
        let out = enclose_deleted_paragraph_comment_references(&source, "Ed").unwrap();
        assert!(enclosed(&part_string(&out, "word/header1.xml").unwrap(), 4));
        let body = part_string(&out, "word/document.xml").unwrap();
        assert!(body.contains("Body."));
    }

    #[test]
    fn comment_markers_leave_the_deletions_that_took_their_words() {
        let body = format!(
            r#"<w:p><w:del w:id="1" w:author="Ed" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>a</w:delText></w:r><w:commentRangeStart w:id="7"/><w:r><w:delText>b</w:delText></w:r><w:commentRangeEnd w:id="7"/>{}<w:r><w:delText>c</w:delText></w:r></w:del></w:p><w:p><w:del w:id="2" w:author="Ed" w:date="2026-01-01T00:00:00Z"><w:commentRangeStart w:id="8"/><w:r><w:delText>d</w:delText></w:r><w:commentRangeEnd w:id="8"/></w:del></w:p>"#,
            reference(7)
        );
        let source = docx(&body);
        assert_eq!(lift_deletion_comments(&source, &[]).unwrap(), source);
        let out = lift_deletion_comments(&source, &[7, 8, 9]).unwrap();
        let xml = part_string(&out, "word/document.xml").unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            for marker in dom.descendants(root, Some(&W::name(local))) {
                assert!(
                    dom.ancestors(marker, Some(&W::del())).is_empty(),
                    "{local} left in a deletion: {xml}"
                );
            }
        }
        // a, b, c and d stay deleted, b in a fresh deletion of its own.
        let deleted: Vec<String> = dom
            .descendants(root, Some(&W::del()))
            .into_iter()
            .map(|d| {
                dom.descendants(d, Some(&W::del_text()))
                    .into_iter()
                    .map(|t| dom.value(t).to_string())
                    .collect()
            })
            .collect();
        assert_eq!(deleted, ["a", "b", "c", "d"]);
        let ids: HashSet<&str> = dom
            .descendants(root, Some(&W::del()))
            .into_iter()
            .filter_map(|d| dom.attribute(d, &W::id()))
            .collect();
        assert_eq!(ids.len(), 4, "split parts take fresh ids: {xml}");
    }
}
