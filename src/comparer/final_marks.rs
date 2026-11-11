// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A table cell's final paragraph mark carries no revision its row does
//! not: Word opens such a cell, but Accept All or Reject All never returns
//! (#1669). Word's Compare pairs a cell's two final marks and puts the
//! revision on the mark before (Word 16 oracles on #1669). This pass
//! settles the finished redline, part by part, by the validator's own rule
//! ([`crate::validate::uncovered_final_marks`]), so the two cannot
//! disagree (#1684).
//!
//! An inserted final mark changes places with the nearest earlier mark that
//! is not inserted: that mark becomes the inserted one, and Reject All
//! joins its words to the final mark, which records that paragraph's
//! original properties. A deleted final mark changes places with the
//! nearest earlier mark that is not deleted: Accept All joins that
//! paragraph's words to the final mark, which takes its properties and
//! records its own. When the earlier mark carries the other revision, its
//! words join the paragraph after it, whose mark is then the cell's
//! original (or revised) one. Paragraphs carrying the same revision, and
//! tables every row of which carries it, lie between. With no such mark the
//! final mark loses its revision, as Word writes a cell a row gains.
//!
//! A move joins, once resolved, the paragraphs after that mark to its own.
//! When one of them keeps words that resolution keeps (inserted or
//! unchanged ones under Accept All, deleted or unchanged ones under Reject
//! All), it is a paragraph of its own in that version: the final mark loses
//! its revision instead, as Word writes a paragraph inserted over a
//! deleted nested table (Word 16 oracle on #1680, 2026-10-23). The
//! comparer leaves such a revision where it drops an empty paragraph
//! (#1686); moving it joined two of the revised cell's paragraphs.
//!
//! A body that ends in a table never emits the cell's inserted mark
//! (#1683). This pass writes it on the paragraph before. The main part's
//! sweep moves an inserted closing paragraph back across a deleted table
//! onto the body's empty close first
//! ([`crate::comparer::finalize::pair_closing_insert_across_deleted_table`]).

use crate::namespaces::W;
use crate::revision_processor::RevisionIds;
use crate::validate::{keeps_words, table_revised};
use crate::xmllinq::{Dom, NodeId, XName};

/// Settle every cell's final mark under `root`.
pub fn settle_cell_final_marks(dom: &mut Dom, root: NodeId) {
    // The inserted mark was never written, so the loop below has nothing
    // to move (#1683, case 1). Mint it before that loop.
    mint_cell_insert_when_body_ends_in_table(dom, root);
    let cells = dom.descendants(root, Some(&W::tc()));
    let mut ids = None;
    for cell in cells {
        // Each step clears the final mark, or joins one paragraph away.
        for _ in 0..=dom.descendants(cell, Some(&W::p())).len() {
            let Some(&(last, kind)) = crate::validate::uncovered_final_marks(dom, cell).first()
            else {
                break;
            };
            settle(dom, root, cell, last, kind, &mut ids);
        }
    }
}

/// One step towards settling `cell`, whose final paragraph `last` carries
/// `kind` (`"ins"` or `"del"`).
fn settle(
    dom: &mut Dom,
    root: NodeId,
    cell: NodeId,
    last: NodeId,
    kind: &str,
    ids: &mut Option<RevisionIds>,
) {
    let other = if kind == "ins" { "del" } else { "ins" };
    let blocks = crate::validate::container_blocks(dom, cell);
    let Some(at) = blocks.iter().position(|&b| b == last) else {
        return;
    };
    let partner = blocks[..at].iter().rev().copied().find_map(|block| {
        if dom.name_is(block, &W::tbl()) {
            return if table_revised(dom, block, kind) {
                None
            } else {
                Some(None)
            };
        }
        (!mark_revised_as(dom, block, kind)).then_some(Some(block))
    });
    // Paragraphs the move would join to the partner's, which keep words of
    // their own once resolved.
    let partner = partner.filter(|partner| {
        let from = partner.map_or(0, |p| {
            blocks.iter().position(|&b| b == p).map_or(0, |i| i + 1)
        });
        !blocks[from..=at]
            .iter()
            .any(|&block| dom.name_is(block, &W::p()) && keeps_words(dom, block, kind))
    });
    let revision = mark_revision(dom, last, kind).expect("an uncovered final mark");
    let mut stamp = Stamp {
        root,
        ids,
        author: dom.attribute(revision, &W::author()).map(str::to_string),
        date: dom.attribute(revision, &W::date()).map(str::to_string),
    };
    match partner.flatten() {
        Some(partner) if !mark_revised_as(dom, partner, other) => {
            if kind == "ins" {
                take_inserted_mark(dom, partner, last, &mut stamp);
            } else {
                take_deleted_mark(dom, partner, last, &mut stamp);
            }
        }
        Some(partner) => {
            // The next paragraph, past tables the same revision removes.
            let next = blocks
                .iter()
                .position(|&b| b == partner)
                .and_then(|i| {
                    blocks[i + 1..]
                        .iter()
                        .copied()
                        .find(|&b| !(dom.name_is(b, &W::tbl()) && table_revised(dom, b, kind)))
                })
                // Words do not leave their content control or custom XML
                // element; an empty paragraph has none to move.
                .filter(|&n| {
                    dom.name_is(n, &W::p())
                        && (dom.parent(n) == dom.parent(partner)
                            || dom
                                .nodes(partner)
                                .into_iter()
                                .all(|c| dom.name_is(c, &W::p_pr())))
                });
            match next {
                Some(next) => join(dom, partner, next, kind, &mut stamp),
                None => unrevise(dom, last, kind),
            }
        }
        None => unrevise(dom, last, kind),
    }
}

/// #1683, case 1. The cell's last paragraph is an insertion whose mark
/// was never revised, because the body ends in this table (only deleted
/// paragraphs follow it). Word writes `w:ins` on the paragraph before.
/// A live paragraph after the table keeps the mark the comparer already
/// emitted, and the loop below moves that one.
fn mint_cell_insert_when_body_ends_in_table(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    // Minting writes into cells only, so the body's children stay put:
    // list them once, not once per cell (jules on #2101).
    let body_kids = dom.elements(body, None);
    let cells = dom.descendants(root, Some(&W::tc()));
    let mut ids: Option<RevisionIds> = None;
    for cell in cells {
        let Some(table) = nearest_table(dom, cell) else {
            continue;
        };
        let Some(at) = body_kids.iter().position(|&child| child == table) else {
            continue;
        };
        let tail = &body_kids[at + 1..];
        if !body_tail_is_only_deletions(dom, tail) || !body_tail_holds_paired_mark(dom, tail) {
            continue;
        }
        let blocks = crate::validate::container_blocks(dom, cell);
        let Some((&last, earlier)) = blocks.split_last() else {
            continue;
        };
        let Some(&previous) = earlier.last() else {
            continue;
        };
        if !dom.name_is(last, &W::p())
            || !dom.name_is(previous, &W::p())
            || !mark_is_unrevised(dom, last)
            || !mark_is_unrevised(dom, previous)
            || !content_is_only_insertions(dom, last)
        {
            continue;
        }
        let Some(source) = dom.find_descendant_element(last, Some(&W::ins()), |_| true) else {
            continue;
        };
        let Some(author) = dom.attribute(source, &W::author()).map(str::to_string) else {
            continue;
        };
        let date = dom.attribute(source, &W::date()).map(str::to_string);
        let id = ids
            .get_or_insert_with(|| {
                let mut used = std::collections::HashSet::new();
                dom.for_each_descendant_element(root, None, |element| {
                    if let Some(id) = dom
                        .attribute(element, &W::id())
                        .and_then(|v| v.parse::<u64>().ok())
                    {
                        used.insert(id);
                    }
                });
                RevisionIds::from_used(used)
            })
            .allocate();
        let mark = dom.new_element(W::ins());
        dom.set_attribute_value(mark, &W::id(), Some(&id.to_string()));
        dom.set_attribute_value(mark, &W::author(), Some(&author));
        if let Some(date) = &date {
            dom.set_attribute_value(mark, &W::date(), Some(date));
        }
        let properties = mark_properties(dom, previous);
        dom.add_first(properties, mark);
    }
}

/// The nearest `w:tbl` above `cell`, walked parent by parent (jules on
/// #2101).
fn nearest_table(dom: &Dom, cell: NodeId) -> Option<NodeId> {
    let mut node = dom.parent(cell);
    while let Some(current) = node {
        if dom.name_is(current, &W::tbl()) {
            return Some(current);
        }
        node = dom.parent(current);
    }
    None
}

/// A range or proofing marker a body may hold between blocks: it carries
/// no text and no revision, so the body's tail reads past it (cursor on
/// #2101).
pub(crate) fn is_inert_body_marker(dom: &Dom, node: NodeId) -> bool {
    [
        "bookmarkStart",
        "bookmarkEnd",
        "commentRangeStart",
        "commentRangeEnd",
        "permStart",
        "permEnd",
        "proofErr",
    ]
    .iter()
    .any(|name| dom.name_is(node, &W::name(name)))
}

/// Nothing in the body after the table is live: only `w:sectPr`, inert
/// markers, and paragraphs whose text is deleted.
fn body_tail_is_only_deletions(dom: &Dom, tail: &[NodeId]) -> bool {
    tail.iter().all(|&child| {
        dom.name_is(child, &W::sect_pr())
            || is_inert_body_marker(dom, child)
            || paragraph_is_deletion_only(dom, child)
    })
}

/// A deleted paragraph after the table keeps a live mark: the story-final
/// pairing gave it the cell's mark, which is why the cell's inserted mark
/// was never written. Without that pairing the mark is deleted with its
/// words and the cell's final paragraph is the original's own, filled
/// (codex on #2101).
fn body_tail_holds_paired_mark(dom: &Dom, tail: &[NodeId]) -> bool {
    tail.iter()
        .any(|&child| dom.name_is(child, &W::p()) && mark_is_unrevised(dom, child))
}

/// A paragraph Word deleted: no insertion, no live text, and a deletion
/// in its content or on its mark.
fn paragraph_is_deletion_only(dom: &Dom, paragraph: NodeId) -> bool {
    if !dom.name_is(paragraph, &W::p()) || mark_revised_as(dom, paragraph, "ins") {
        return false;
    }
    if dom
        .find_descendant_element(paragraph, Some(&W::ins()), |_| true)
        .is_some()
    {
        return false;
    }
    let live_text = dom
        .find_descendant_element(paragraph, Some(&W::t()), |text| {
            !dom.value(text).trim().is_empty()
        })
        .is_some();
    if live_text {
        return false;
    }
    mark_revised_as(dom, paragraph, "del")
        || dom
            .find_descendant_element(paragraph, Some(&W::del()), |_| true)
            .is_some()
}

/// No inserted, deleted, or move revision on `paragraph`'s mark.
fn mark_is_unrevised(dom: &Dom, paragraph: NodeId) -> bool {
    !mark_revised_as(dom, paragraph, "ins") && !mark_revised_as(dom, paragraph, "del")
}

/// Every child past `w:pPr` is a `w:ins`, and there is at least one.
fn content_is_only_insertions(dom: &Dom, paragraph: NodeId) -> bool {
    let mut saw = false;
    for child in dom.elements(paragraph, None) {
        if dom.name_is(child, &W::p_pr()) {
            continue;
        }
        if !dom.name_is(child, &W::ins()) {
            return false;
        }
        saw = true;
    }
    saw
}

/// Whether `paragraph`'s mark is revised as `kind` (`"ins"` or `"del"`)
/// says, a mark moved in (`w:moveTo`) or away (`w:moveFrom`) counting as
/// its resolution does, as in [`keeps_words`]: a mark holds one of the
/// four, so a moved one is no partner for a second (codex on #1680).
fn mark_revised_as(dom: &Dom, paragraph: NodeId, kind: &str) -> bool {
    let records = if kind == "del" {
        [W::del(), W::move_from()]
    } else {
        [W::ins(), W::move_to()]
    };
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| records.iter().any(|name| dom.element(rpr, name).is_some()))
}

/// The `kind` revision on `paragraph`'s mark (`w:pPr/w:rPr`).
fn mark_revision(dom: &Dom, paragraph: NodeId, kind: &str) -> Option<NodeId> {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .and_then(|rpr| dom.element(rpr, &crate::validate::revision_name(kind)))
}

/// The final mark `last`'s insertion moves to `partner`'s mark. Rejected,
/// `partner`'s words join `last`'s mark, which records `partner`'s
/// properties and mark formatting where they differ from its own; a
/// change record `partner` holds moves to it.
fn take_inserted_mark(dom: &mut Dom, partner: NodeId, last: NodeId, stamp: &mut Stamp<'_>) {
    let revision = mark_revision(dom, last, "ins").expect("an inserted final mark");
    dom.remove(revision);
    record_original(dom, partner, last, stamp);
    let rpr = mark_properties(dom, partner);
    dom.add_first(rpr, revision);
}

/// The final mark `last`'s deletion moves to `partner`'s mark. Accepted,
/// `partner`'s words join `last`'s mark, which takes `partner`'s properties
/// and mark formatting and records its own where they differ.
fn take_deleted_mark(dom: &mut Dom, partner: NodeId, last: NodeId, stamp: &mut Stamp<'_>) {
    let revision = mark_revision(dom, last, "del").expect("a deleted final mark");
    dom.remove(revision);
    take_properties(dom, partner, last, stamp);
    let rpr = mark_properties(dom, partner);
    match dom.element(rpr, &W::ins()) {
        Some(ins) => dom.add_after_self(ins, revision),
        None => dom.add_first(rpr, revision),
    }
}

/// `head`, whose mark carries the revision other than `kind`, joins `next`,
/// whose mark carries `kind`: its words move to the start of `next`, whose
/// mark is no longer revised. An inserted `next` keeps its properties and
/// records `head`'s original ones; a deleted `next` takes the inserted
/// `head`'s and records its own.
fn join(dom: &mut Dom, head: NodeId, next: NodeId, kind: &str, stamp: &mut Stamp<'_>) {
    if let Some(revision) = mark_revision(dom, next, kind) {
        dom.remove(revision);
    }
    if kind == "ins" {
        record_original(dom, head, next, stamp);
    } else {
        take_properties(dom, head, next, stamp);
    }
    let words: Vec<NodeId> = dom
        .nodes(head)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::p_pr()))
        .collect();
    let anchor = dom.element(next, &W::p_pr());
    for word in words.into_iter().rev() {
        dom.remove(word);
        match anchor {
            Some(ppr) => dom.add_after_self(ppr, word),
            None => dom.add_first(next, word),
        }
    }
    dom.remove(head);
}

/// `last`'s mark loses its `kind` revision.
fn unrevise(dom: &mut Dom, last: NodeId, kind: &str) {
    if let Some(revision) = mark_revision(dom, last, kind) {
        dom.remove(revision);
    }
}

/// `to` records `from`'s original properties and mark formatting: the
/// change records `from` holds move to `to`, or, where `from` holds none,
/// new records keep `from`'s own where they differ from `to`'s. A record
/// `to` already held goes.
fn record_original(dom: &mut Dom, from: NodeId, to: NodeId, stamp: &mut Stamp<'_>) {
    if let Some(change) = paragraph_change(dom, to) {
        dom.remove(change);
    }
    if let Some(change) = paragraph_change(dom, from) {
        dom.remove(change);
        let ppr = paragraph_properties(dom, to);
        dom.add(ppr, change);
    } else if serialized(dom, &bare_properties(dom, from))
        != serialized(dom, &bare_properties(dom, to))
    {
        let record = stamp.record(
            dom,
            W::p_pr_change(),
            W::p_pr(),
            &bare_properties(dom, from),
        );
        let ppr = paragraph_properties(dom, to);
        dom.add(ppr, record);
    }
    if let Some(change) = mark_change(dom, to) {
        dom.remove(change);
    }
    if let Some(change) = mark_change(dom, from) {
        dom.remove(change);
        let rpr = mark_properties(dom, to);
        dom.add(rpr, change);
    } else if serialized(dom, &mark_formatting(dom, from))
        != serialized(dom, &mark_formatting(dom, to))
    {
        let record = stamp.record(
            dom,
            W::r_pr_change(),
            W::r_pr(),
            &mark_formatting(dom, from),
        );
        let rpr = mark_properties(dom, to);
        dom.add(rpr, record);
    }
}

/// `to` takes `from`'s properties and mark formatting, and records its own
/// original ones (its change records, or what it held) where they differ.
fn take_properties(dom: &mut Dom, from: NodeId, to: NodeId, stamp: &mut Stamp<'_>) {
    let copies = |dom: &mut Dom, nodes: Vec<NodeId>| -> Vec<NodeId> {
        nodes.into_iter().map(|n| dom.clone_subtree(n)).collect()
    };
    let original = original_properties(dom, to);
    let original = copies(dom, original);
    let wanted = bare_properties(dom, from);
    let ppr = paragraph_properties(dom, to);
    for child in bare_properties(dom, to) {
        dom.remove(child);
    }
    for child in wanted.into_iter().rev() {
        let copy = dom.clone_subtree(child);
        dom.add_first(ppr, copy);
    }
    if let Some(change) = paragraph_change(dom, to) {
        dom.remove(change);
    }
    if serialized(dom, &original) != serialized(dom, &bare_properties(dom, to)) {
        let record = stamp.record(dom, W::p_pr_change(), W::p_pr(), &original);
        dom.add(ppr, record);
    }
    let original = original_mark_formatting(dom, to);
    let original = copies(dom, original);
    let wanted = mark_formatting(dom, from);
    let rpr = mark_properties(dom, to);
    for child in mark_formatting(dom, to) {
        dom.remove(child);
    }
    if let Some(change) = mark_change(dom, to) {
        dom.remove(change);
    }
    for child in wanted {
        let copy = dom.clone_subtree(child);
        dom.add(rpr, copy);
    }
    if serialized(dom, &original) != serialized(dom, &mark_formatting(dom, to)) {
        let record = stamp.record(dom, W::r_pr_change(), W::r_pr(), &original);
        dom.add(rpr, record);
    }
}

/// `paragraph`'s original properties: what its `w:pPrChange` recorded, or
/// its own.
fn original_properties(dom: &Dom, paragraph: NodeId) -> Vec<NodeId> {
    match paragraph_change(dom, paragraph).and_then(|c| dom.element(c, &W::p_pr())) {
        Some(recorded) => dom.elements(recorded, None),
        None => bare_properties(dom, paragraph),
    }
}

/// `paragraph`'s original mark formatting: what its mark's `w:rPrChange`
/// recorded, or its own.
fn original_mark_formatting(dom: &Dom, paragraph: NodeId) -> Vec<NodeId> {
    match mark_change(dom, paragraph).and_then(|c| dom.element(c, &W::r_pr())) {
        Some(recorded) => dom.elements(recorded, None),
        None => mark_formatting(dom, paragraph),
    }
}

/// `paragraph`'s properties without the mark's own (`w:rPr`), its section
/// or a change record.
fn bare_properties(dom: &Dom, paragraph: NodeId) -> Vec<NodeId> {
    dom.element(paragraph, &W::p_pr())
        .map(|ppr| dom.elements(ppr, None))
        .unwrap_or_default()
        .into_iter()
        .filter(|&child| {
            !dom.name_is(child, &W::r_pr())
                && !dom.name_is(child, &W::sect_pr())
                && !dom.name_is(child, &W::p_pr_change())
        })
        .collect()
}

/// The formatting of `paragraph`'s mark (`w:pPr/w:rPr`), without its
/// revisions or a change record.
fn mark_formatting(dom: &Dom, paragraph: NodeId) -> Vec<NodeId> {
    const RECORDS: [&str; 5] = ["ins", "del", "moveFrom", "moveTo", "rPrChange"];
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .map(|rpr| dom.elements(rpr, None))
        .unwrap_or_default()
        .into_iter()
        .filter(|&child| {
            !dom.name(child)
                .is_some_and(|n| n.namespace_name() == W::URI && RECORDS.contains(&n.local_name()))
        })
        .collect()
}

/// `paragraph`'s `w:pPr/w:pPrChange`.
fn paragraph_change(dom: &Dom, paragraph: NodeId) -> Option<NodeId> {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::p_pr_change()))
}

/// `paragraph`'s mark's `w:pPr/w:rPr/w:rPrChange`.
fn mark_change(dom: &Dom, paragraph: NodeId) -> Option<NodeId> {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .and_then(|rpr| dom.element(rpr, &W::r_pr_change()))
}

/// `children` as XML, to compare property sets.
fn serialized(dom: &Dom, children: &[NodeId]) -> Vec<String> {
    children.iter().map(|&c| dom.serialize_element(c)).collect()
}

/// `paragraph`'s `w:pPr`, created when missing.
fn paragraph_properties(dom: &mut Dom, paragraph: NodeId) -> NodeId {
    match dom.element(paragraph, &W::p_pr()) {
        Some(ppr) => ppr,
        None => {
            let ppr = dom.new_element(W::p_pr());
            dom.add_first(paragraph, ppr);
            ppr
        }
    }
}

/// `paragraph`'s mark properties (`w:pPr/w:rPr`), created when missing,
/// before the paragraph's section and change record.
fn mark_properties(dom: &mut Dom, paragraph: NodeId) -> NodeId {
    let ppr = paragraph_properties(dom, paragraph);
    if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
        return rpr;
    }
    let rpr = dom.new_element(W::r_pr());
    let after = dom
        .elements(ppr, None)
        .into_iter()
        .find(|&c| dom.name_is(c, &W::sect_pr()) || dom.name_is(c, &W::p_pr_change()));
    match after {
        Some(after) => dom.add_before_self(after, rpr),
        None => dom.add(ppr, rpr),
    }
    rpr
}

/// The stamp of the change records a step writes: the moved revision's
/// author and date, and the part's next free revision id.
struct Stamp<'a> {
    root: NodeId,
    ids: &'a mut Option<RevisionIds>,
    author: Option<String>,
    date: Option<String>,
}

impl Stamp<'_> {
    /// A `record` (`w:pPrChange`, `w:rPrChange`) holding a `holder` with
    /// copies of `children`.
    fn record(
        &mut self,
        dom: &mut Dom,
        record: XName,
        holder: XName,
        children: &[NodeId],
    ) -> NodeId {
        // Clear of every id in the part, annotations' too, and within the
        // schema's signed 32-bit range (codex on #1680, round 10).
        let next = self
            .ids
            .get_or_insert_with(|| {
                RevisionIds::from_used(
                    dom.descendants(self.root, None)
                        .into_iter()
                        .filter_map(|e| dom.attribute(e, &W::id())?.parse::<u64>().ok())
                        .collect(),
                )
            })
            .allocate();
        let node = dom.new_element(record);
        dom.set_attribute_value(node, &W::id(), Some(&next.to_string()));
        if let Some(author) = &self.author {
            dom.set_attribute_value(node, &W::author(), Some(author));
        }
        if let Some(date) = &self.date {
            dom.set_attribute_value(node, &W::date(), Some(date));
        }
        let inner = dom.new_element(holder);
        for &child in children {
            let copy = dom.clone_subtree(child);
            dom.add(inner, copy);
        }
        dom.add(node, inner);
        node
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use crate::namespaces::M;

    /// A bookmark end Word leaves between the inserted paragraph and the
    /// deleted table is a marker, not a block: the inserted paragraph
    /// still moves onto the body's empty close (cursor on #2101).
    #[test]
    fn a_marker_before_the_deleted_table_does_not_block_the_close() {
        let ins = r#"w:id="1" w:author="Ann" w:date="2026-10-01T00:00:00Z""#;
        let del = r#"w:author="Ann" w:date="2026-10-01T00:00:00Z""#;
        let xml = format!(
            r#"<w:document xmlns:w="{}"><w:body><w:p><w:r><w:t>Text.</w:t></w:r></w:p><w:p><w:pPr><w:rPr><w:ins {ins}/></w:rPr></w:pPr><w:ins w:id="2" w:author="Ann"><w:r><w:t>Final para.</w:t></w:r></w:ins></w:p><w:bookmarkEnd w:id="9"/><w:tbl><w:tr><w:tc><w:p><w:pPr><w:rPr><w:del w:id="3" {del}/></w:rPr></w:pPr><w:del w:id="4" {del}><w:r><w:delText>Cell.</w:delText></w:r></w:del></w:p></w:tc></w:tr></w:tbl><w:p/><w:sectPr/></w:body></w:document>"#,
            W::URI
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        crate::comparer::finalize::pair_closing_insert_across_deleted_table(
            &mut dom,
            body,
            &format!(
                r#"<w:document xmlns:w="{}"><w:body><w:p><w:r><w:t>Final para.</w:t></w:r></w:p><w:sectPr/></w:body></w:document>"#,
                W::URI
            ),
        );
        let kids: Vec<String> = dom
            .elements(body, None)
            .into_iter()
            .map(|k| dom.name(k).unwrap().local_name().to_string())
            .collect();
        assert_eq!(kids, ["p", "bookmarkEnd", "tbl", "p", "sectPr"]);
        let close = dom
            .elements(body, Some(&W::p()))
            .into_iter()
            .last()
            .unwrap();
        assert!(dom.serialize_element(close).contains("Final para."));
    }

    /// A paragraph that holds only an equation keeps it once resolved, as
    /// a paragraph of words does: the final mark loses its revision, and
    /// no paragraph joins another (codex on #1680).
    #[test]
    fn an_equation_keeps_its_paragraph_as_words_do() {
        let mark = |id: u32| {
            format!(
                r#"<w:pPr><w:rPr><w:del w:id="{id}" w:author="Ann" w:date="2026-10-01T00:00:00Z"/></w:rPr></w:pPr>"#
            )
        };
        let (equation, last) = (mark(7), mark(8));
        let xml = format!(
            r#"<w:document xmlns:w="{}" xmlns:m="{}"><w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>Alpha</w:t></w:r></w:p><w:p>{equation}<m:oMath><m:r><m:t>x</m:t></m:r></m:oMath></w:p><w:p>{last}</w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#,
            W::URI,
            M::URI
        );
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        settle_cell_final_marks(&mut dom, root);
        let paragraphs = dom.descendants(root, Some(&W::p()));
        let deleted: Vec<bool> = paragraphs
            .iter()
            .map(|&p| mark_revision(&dom, p, "del").is_some())
            .collect();
        assert_eq!(deleted, [false, true, false]);
        assert_eq!(dom.descendants(paragraphs[1], Some(&M::o_math())).len(), 1);
    }
}
