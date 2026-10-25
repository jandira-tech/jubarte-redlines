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

use crate::namespaces::{M, W};
use crate::revision_processor::RevisionIds;
use crate::xmllinq::{Dom, NodeId, XName};

/// Settle every cell's final mark under `root`.
pub fn settle_cell_final_marks(dom: &mut Dom, root: NodeId) {
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

/// Whether `paragraph` keeps words once its `kind` revisions resolve:
/// Accept All (`"del"`) keeps what is not deleted or moved away, Reject All
/// (`"ins"`) what is not inserted or moved in. An equation's runs (`m:r`)
/// are words too (codex on #1680).
fn keeps_words(dom: &Dom, paragraph: NodeId, kind: &str) -> bool {
    let gone = if kind == "del" {
        [W::del(), W::move_from()]
    } else {
        [W::ins(), W::move_to()]
    };
    let (math_run, math_r_pr) = (M::r(), M::r_pr());
    dom.descendants(paragraph, None)
        .into_iter()
        .filter(|&n| dom.name_is(n, &W::r()) || dom.name_is(n, &math_run))
        .any(|run| {
            dom.elements(run, None)
                .into_iter()
                .any(|child| !dom.name_is(child, &W::r_pr()) && !dom.name_is(child, &math_r_pr))
                && !dom
                    .ancestors(run, None)
                    .into_iter()
                    .take_while(|&a| a != paragraph)
                    .any(|a| gone.iter().any(|name| dom.name_is(a, name)))
        })
}

/// Whether every row of `table` is inserted or deleted as `kind` says.
fn table_revised(dom: &Dom, table: NodeId, kind: &str) -> bool {
    let rows: Vec<NodeId> = dom
        .descendants(table, Some(&W::tr()))
        .into_iter()
        .filter(|&row| dom.ancestors(row, Some(&W::tbl())).first() == Some(&table))
        .collect();
    !rows.is_empty()
        && rows.iter().all(|&row| {
            dom.element(row, &W::tr_pr()).is_some_and(|pr| {
                dom.element(pr, &crate::validate::revision_name(kind))
                    .is_some()
            })
        })
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
