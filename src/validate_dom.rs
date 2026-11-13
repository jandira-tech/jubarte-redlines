// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pure WordprocessingML DOM structural predicates shared between the
//! redline comparer and document validator.

use crate::markup_simplifier::{choice_requires_understood, word_understands_namespace};
use crate::namespaces::{M, MC, W};
use crate::xmllinq::{Dom, NodeId, XName};

/// Paragraphs whose content should move ahead of deleted tables when Accept All
/// would otherwise join their inserted words onto an empty or colliding trailing
/// paragraph mark (#1851, #1926).
pub(crate) fn inserted_text_before_deleted_tables(dom: &Dom, body: NodeId) -> Vec<(NodeId, NodeId)> {
    let markers = [
        W::bookmark_start(),
        W::bookmark_end(),
        W::comment_range_start(),
        W::comment_range_end(),
        W::perm_start(),
        W::perm_end(),
        W::proof_err(),
    ];
    let blocks: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&b| !markers.iter().any(|name| dom.name_is(b, name)))
        .collect();
    let mut out = Vec::new();
    for (at, &paragraph) in blocks.iter().enumerate() {
        let right_before_a_table = blocks
            .get(at + 1)
            .is_some_and(|&next| dom.name_is(next, &W::tbl()) && table_revised(dom, next, "del"));
        if !right_before_a_table || !keeps_only_inserted_words(dom, paragraph) {
            continue;
        }
        let mut tables = 0;
        let joined = blocks[at + 1..].iter().copied().find(|&block| {
            if dom.name_is(block, &W::tbl()) && table_revised(dom, block, "del") {
                tables += 1;
                return false;
            }
            !(dom.name_is(block, &W::p())
                && only_mark_deleted(dom, block)
                && !keeps_words(dom, block, "del"))
        });
        let preceded_by_deleted_table = at > 0
            && dom.name_is(blocks[at - 1], &W::tbl())
            && table_revised(dom, blocks[at - 1], "del");
        // Nothing but the insertion: a deleted run of the original's
        // own words, or any other child, keeps the move (#1851).
        let only_inserted_runs = dom
            .elements(paragraph, None)
            .into_iter()
            .filter(|&child| !dom.name_is(child, &W::p_pr()))
            .all(|child| dom.name_is(child, &W::ins()));
        if let Some(joined) = joined.filter(|&joined| {
            tables > 0
                && dom.name_is(joined, &W::p())
                && !mark_carries(dom, joined, &[W::del(), W::move_from()])
                // One deleted table, an empty paragraph after it, no
                // deleted table before, and nothing in this paragraph but
                // the insertion: Word's order, the words stay (#1926).
                // Two tables, a paragraph after the table that already
                // keeps words, or deleted words beside the insertion,
                // still move (#1851).
                && (tables > 1
                    || preceded_by_deleted_table
                    || keeps_words(dom, joined, "del")
                    || !only_inserted_runs)
        }) {
            out.push((paragraph, joined));
        }
    }
    out
}

/// Whether `paragraph`'s mark is deleted, and no more (not inserted, not
/// moved), and it ends no section.
fn only_mark_deleted(dom: &Dom, paragraph: NodeId) -> bool {
    mark_carries(dom, paragraph, &[W::del()])
        && !mark_carries(dom, paragraph, &[W::ins(), W::move_from(), W::move_to()])
        && dom
            .element(paragraph, &W::p_pr())
            .is_none_or(|ppr| dom.element(ppr, &W::sect_pr()).is_none())
}

/// Whether `paragraph`'s mark (`w:pPr/w:rPr`) holds one of `records`.
fn mark_carries(dom: &Dom, paragraph: NodeId, records: &[XName]) -> bool {
    dom.element(paragraph, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| records.iter().any(|name| dom.element(rpr, name).is_some()))
}

/// Whether `paragraph`, its mark deleted, keeps words under Accept All
/// and every one of them sits in its own `w:ins` children.
fn keeps_only_inserted_words(dom: &Dom, paragraph: NodeId) -> bool {
    if !dom.name_is(paragraph, &W::p()) || !only_mark_deleted(dom, paragraph) {
        return false;
    }
    let (inserted, other): (Vec<NodeId>, Vec<NodeId>) = dom
        .elements(paragraph, None)
        .into_iter()
        .filter(|&child| !dom.name_is(child, &W::p_pr()))
        .partition(|&child| dom.name_is(child, &W::ins()));
    inserted.iter().any(|&ins| keeps_words(dom, ins, "del"))
        && !other.iter().any(|&child| keeps_words(dom, child, "del"))
}

/// Whether `node` keeps words once its `kind` revisions resolve: a run in
/// it (`node` itself included; an equation's `m:r` too) with content,
/// under no revision that resolution removes, up to `node`. Accept All
/// (`"del"`) removes what is deleted or moved away, Reject All (`"ins"`)
/// what is inserted or moved in.
pub(crate) fn keeps_words(dom: &Dom, node: NodeId, kind: &str) -> bool {
    let gone = if kind == "del" {
        [W::del(), W::move_from()]
    } else {
        [W::ins(), W::move_to()]
    };
    let (math_run, math_r_pr) = (M::r(), M::r_pr());
    let stop = dom.parent(node);
    std::iter::once(node)
        .chain(dom.descendants(node, None))
        .filter(|&n| dom.name_is(n, &W::r()) || dom.name_is(n, &math_run))
        .any(|run| {
            dom.elements(run, None)
                .into_iter()
                .any(|child| !dom.name_is(child, &W::r_pr()) && !dom.name_is(child, &math_r_pr))
                && !dom
                    .ancestors(run, None)
                    .into_iter()
                    .take_while(|&a| Some(a) != stop)
                    .any(|a| gone.iter().any(|name| dom.name_is(a, name)))
        })
}

/// Whether every row of `table` (its own, not a nested table's) is
/// inserted or deleted as `kind` says.
pub(crate) fn table_revised(dom: &Dom, table: NodeId, kind: &str) -> bool {
    let rows: Vec<NodeId> = dom
        .descendants(table, Some(&W::tr()))
        .into_iter()
        .filter(|&row| dom.ancestors(row, Some(&W::tbl())).first() == Some(&table))
        .collect();
    !rows.is_empty()
        && rows.iter().all(|&row| {
            dom.element(row, &W::tr_pr())
                .is_some_and(|pr| dom.element(pr, &revision_name(kind)).is_some())
        })
}

/// A story's closing paragraph: its last block when that is a paragraph of
/// its own, not one in a content control, a custom XML element or a table.
pub(crate) fn closing_paragraph(dom: &Dom, story: NodeId) -> Option<NodeId> {
    let blocks = [W::p(), W::tbl(), W::sdt(), W::custom_xml()];
    dom.elements(story, None)
        .into_iter()
        .rev()
        .find(|&child| blocks.iter().any(|name| dom.name_is(child, name)))
        .filter(|&last| dom.name_is(last, &W::p()))
}

/// Whether paragraph `p` holds a PAGE or NUMPAGES field: a field code whose
/// first word is one of them. A complex field's code is every
/// `w:instrText` or `w:delInstrText` between its `begin` and its
/// `separate` (or `end`), split across runs as Word often writes it; a
/// `w:fldSimple`'s is its `w:instr`. Code outside any field in `p` (its
/// `begin` in an earlier paragraph) counts piece by piece.
pub(crate) fn holds_page_field(dom: &Dom, p: NodeId) -> bool {
    let page = |code: &str| {
        code.split_whitespace().next().is_some_and(|word| {
            word.eq_ignore_ascii_case("PAGE") || word.eq_ignore_ascii_case("NUMPAGES")
        })
    };
    let (instr_text, del_instr_text) = (W::instr_text(), W::del_instr_text());
    let (fld_char, fld_char_type) = (W::fld_char(), W::fld_char_type());
    let (fld_simple, instr) = (W::fld_simple(), W::instr());
    // The open complex fields' codes, innermost last; `None` once read.
    let mut codes: Vec<Option<String>> = Vec::new();
    for n in dom.descendants(p, None) {
        if dom.name_is(n, &instr_text) || dom.name_is(n, &del_instr_text) {
            match codes.last_mut() {
                Some(Some(code)) => code.push_str(&dom.value(n)),
                Some(None) => {}
                None if page(&dom.value(n)) => return true,
                None => {}
            }
        } else if dom.name_is(n, &fld_char) {
            match dom.attribute(n, &fld_char_type) {
                Some("begin") => codes.push(Some(String::new())),
                Some(kind @ ("separate" | "end")) => {
                    if codes
                        .last_mut()
                        .and_then(Option::take)
                        .is_some_and(|code| page(&code))
                    {
                        return true;
                    }
                    if kind == "end" {
                        codes.pop();
                    }
                }
                _ => {}
            }
        } else if dom.name_is(n, &fld_simple) && dom.attribute(n, &instr).is_some_and(page) {
            return true;
        }
    }
    // A field whose `separate` and `end` lie past the paragraph.
    codes.into_iter().flatten().any(|code| page(&code))
}

/// The revisions (`"ins"`, `"del"`) on `cell`'s final paragraph mark that
/// no row covers, with that paragraph: `CELL_FINAL_MARK_REVISION`'s rule,
/// which the comparer's settle pass applies too. The cell and every cell
/// around it count, through nested tables (an outer row's revision covers
/// a nested table's cells), each with the row that holds it, through any
/// content control between them.
pub(crate) fn uncovered_final_marks(dom: &Dom, cell: NodeId) -> Vec<(NodeId, &'static str)> {
    let Some(last) = last_block_paragraph(dom, cell) else {
        return Vec::new();
    };
    let Some(rpr) = dom
        .element(last, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
    else {
        return Vec::new();
    };
    let cells: Vec<NodeId> = std::iter::once(cell)
        .chain(dom.ancestors(cell, Some(&W::tc())))
        .collect();
    ["ins", "del"]
        .into_iter()
        .filter(|kind| {
            dom.element(rpr, &revision_name(kind)).is_some()
                && !cells.iter().any(|&c| {
                    dom.ancestors(c, Some(&W::tr()))
                        .first()
                        .is_some_and(|&row| row_revised(dom, row, c, kind))
                })
        })
        .map(|kind| (last, kind))
        .collect()
}

/// Whether `row` (or the cell itself) is inserted or deleted as `kind`
/// says: `w:trPr/w:ins` or `w:trPr/w:del`, or the cell's
/// `w:tcPr/w:cellIns` or `w:tcPr/w:cellDel`.
pub(crate) fn row_revised(dom: &Dom, row: NodeId, cell: NodeId, kind: &str) -> bool {
    let cell_kind = if kind == "ins" {
        W::cell_ins()
    } else {
        W::cell_del()
    };
    dom.element(row, &W::tr_pr())
        .is_some_and(|pr| dom.element(pr, &revision_name(kind)).is_some())
        || dom
            .element(cell, &W::tc_pr())
            .is_some_and(|pr| dom.element(pr, &cell_kind).is_some())
}

/// `w:ins` for `"ins"`, `w:del` for `"del"`.
pub(crate) fn revision_name(kind: &str) -> XName {
    if kind == "ins" { W::ins() } else { W::del() }
}

/// The last paragraph among a container's blocks, through block content
/// controls, custom XML elements, and the alternate-content branch Word
/// shows, not into nested tables.
pub(crate) fn last_block_paragraph(dom: &Dom, container: NodeId) -> Option<NodeId> {
    container_blocks(dom, container)
        .into_iter()
        .rev()
        .find(|&block| dom.name_is(block, &W::p()))
}

/// The branch of block-level text alternate content that Word shows.
/// Drawing and VML alternates stay whole, so their paragraphs are not
/// blocks of the cell around them. Anything else is not alternate content.
fn shown_alternate_branch(dom: &Dom, node: NodeId) -> Option<NodeId> {
    if !dom.name_is(node, &MC::alternate_content()) {
        return None;
    }
    // A picture that is itself a branch's block (not one inside a branch's
    // paragraph) leaves the branch Word shows in doubt, so it is skipped.
    let pictured = |branch: NodeId| {
        dom.elements(branch, None)
            .into_iter()
            .any(|child| dom.name_is(child, &W::drawing()) || dom.name_is(child, &W::pict()))
    };
    if dom.elements(node, None).into_iter().any(pictured) {
        return None;
    }
    let understood = word_understands_namespace;
    dom.elements(node, Some(&MC::choice()))
        .into_iter()
        .find(|&choice| choice_requires_understood(dom, choice, understood))
        .or_else(|| dom.element(node, &MC::fallback()))
}

/// The paragraphs and tables of `container` in order, through block
/// content controls, custom XML elements, and the alternate-content branch
/// Word shows, not into nested tables.
pub(crate) fn container_blocks(dom: &Dom, container: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    for child in dom.elements(container, None) {
        if dom.name_is(child, &W::p()) || dom.name_is(child, &W::tbl()) {
            out.push(child);
        } else if dom.name_is(child, &W::sdt()) {
            if let Some(content) = dom.element(child, &W::sdt_content()) {
                out.extend(container_blocks(dom, content));
            }
        } else if dom.name_is(child, &W::custom_xml()) {
            out.extend(container_blocks(dom, child));
        } else if let Some(branch) = shown_alternate_branch(dom, child) {
            out.extend(container_blocks(dom, branch));
        }
    }
    out
}

/// Revision elements whose `w:id` must be unique among elements of the
/// same record type within a part (the SDK's `Sem_UniqueAttributeValue`).
/// Different kinds can share an id: Word pairs a style's `pPrChange` and `rPrChange`.
/// A range end shares its start's id, but two ends of one kind may not
/// share one; OpenXmlValidator lets two `customXmlMoveToRangeEnd` do so.
/// Comment and bookmark ids have other scopes.
pub(crate) fn has_unique_revision_id(name: &XName) -> bool {
    name.namespace_name() == W::URI
        && (name.local_name().ends_with("PrChange")
            || matches!(
                name.local_name(),
                "ins"
                    | "del"
                    | "moveFrom"
                    | "moveTo"
                    | "moveFromRangeStart"
                    | "moveFromRangeEnd"
                    | "moveToRangeStart"
                    | "moveToRangeEnd"
                    | "customXmlInsRangeStart"
                    | "customXmlInsRangeEnd"
                    | "customXmlDelRangeStart"
                    | "customXmlDelRangeEnd"
                    | "customXmlMoveFromRangeStart"
                    | "customXmlMoveFromRangeEnd"
                    | "customXmlMoveToRangeStart"
                    | "tblGridChange"
                    | "tblPrExChange"
                    | "numberingChange"
                    | "cellIns"
                    | "cellDel"
                    | "cellMerge"
            ))
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn unique_revision_id_matches_tracked_change_elements_and_excludes_comments() {
        assert!(has_unique_revision_id(&W::ins()));
        assert!(has_unique_revision_id(&W::del()));
        assert!(has_unique_revision_id(&W::name("pPrChange")));
        assert!(!has_unique_revision_id(&W::name("commentRangeStart")));
    }

    #[test]
    fn closing_paragraph_and_holds_page_field_inspect_body() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
                <w:body>
                    <w:p><w:fldSimple w:instr=" PAGE "/></w:p>
                </w:body>
            </w:document>"#,
        );
        let body = dom.element(dom.root(doc).unwrap(), &W::body()).unwrap();
        let close = closing_paragraph(&dom, body).unwrap();
        assert!(holds_page_field(&dom, close));
        assert!(keeps_words(&dom, close, "del"));
    }
}
