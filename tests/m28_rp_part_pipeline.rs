// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M28 — RevisionProcessor part-pipeline (plan M-A). A.0 = the doc-order
//! tag-stream walker (`DescendantAndSelfTags` :2397) and the block-content
//! iteration helpers (`IterateBlockContentElements` :1909,
//! `GetParagraphInfo`/`ContentElementsBeforeSelf` :2888) the later
//! transforms (A.2–A.5) consume.

use jubarte::namespaces::W;
use jubarte::revision_processor::{
    Tag, TagType, accept_deleted_and_move_from_paragraph_marks,
    accept_deleted_and_move_from_paragraph_marks_transform,
    accept_deleted_and_moved_from_content_controls, accept_deleted_cells_transform,
    accept_move_from_ranges, accept_paragraph_end_tags_in_move_from_transform,
    accept_revisions_document, accept_revisions_for_element, accept_revisions_for_part_content,
    add_empty_paragraph_to_any_empty_cells, annotate_content_controls_with_run_ids,
    annotate_run_elements_with_id, coalesque_paragraph_end_tags_in_move_from_transform,
    collapse_paragraph_transform, content_elements_before_self, descendant_and_self_tags,
    element_has_tracked_revisions, fix_up_deleted_or_inserted_field_codes_transform,
    get_paragraph_info, iterate_block_content_elements, merge_adjacent_tables_transform,
    reject_revisions_document, remove_rows_left_empty_by_move_from,
};
use jubarte::xmllinq::{Dom, NodeId};

fn body_from(dom: &mut Dom, inner: &str) -> NodeId {
    let xml = format!(
        "<w:document xmlns:w=\"{}\"><w:body>{}</w:body></w:document>",
        W::URI,
        inner
    );
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.element(root, &W::body()).unwrap()
}

/// A.0 — plan-stated assert: `<w:p><w:r/></w:p>` streams as
/// `[Start(p), Empty(r), End(p)]`.
#[test]
fn a0_tag_stream_start_empty_end() {
    let mut d = Dom::new();
    let body = body_from(&mut d, "<w:p><w:r/></w:p>");
    let p = d.elements(body, Some(&W::p()))[0];
    let r = d.elements(p, Some(&W::r()))[0];

    let tags = descendant_and_self_tags(&d, p);
    assert_eq!(
        tags,
        vec![
            Tag {
                element: p,
                tag_type: TagType::Element
            },
            Tag {
                element: r,
                tag_type: TagType::EmptyElement
            },
            Tag {
                element: p,
                tag_type: TagType::EndElement
            },
        ]
    );
}

/// A.0 — C# `Nodes().Any()` counts TEXT nodes: `<w:t>x</w:t>` is NOT an
/// empty element (it opens and closes), even though it has no element
/// children. The root always gets a Start/End pair, never Empty.
#[test]
fn a0_tag_stream_text_node_is_not_empty() {
    let mut d = Dom::new();
    let body = body_from(&mut d, "<w:p><w:r><w:t>x</w:t></w:r></w:p>");
    let p = d.elements(body, Some(&W::p()))[0];
    let r = d.elements(p, Some(&W::r()))[0];
    let t = d.elements(r, Some(&W::t()))[0];

    let tags = descendant_and_self_tags(&d, p);
    let expected = vec![
        (p, TagType::Element),
        (r, TagType::Element),
        (t, TagType::Element),
        (t, TagType::EndElement),
        (r, TagType::EndElement),
        (p, TagType::EndElement),
    ];
    let got: Vec<(NodeId, TagType)> = tags.iter().map(|t| (t.element, t.tag_type)).collect();
    assert_eq!(got, expected);

    // a childless root still yields its own Start/End pair (not Empty)
    let mut d2 = Dom::new();
    let b2 = body_from(&mut d2, "<w:p/>");
    let p2 = d2.elements(b2, Some(&W::p()))[0];
    let tags2 = descendant_and_self_tags(&d2, p2);
    let got2: Vec<(NodeId, TagType)> = tags2.iter().map(|t| (t.element, t.tag_type)).collect();
    assert_eq!(
        got2,
        vec![(p2, TagType::Element), (p2, TagType::EndElement)]
    );
}

/// A.0 — `IterateBlockContentElements` (:1909): doc-order chain of block
/// content (`w:p`/`w:tbl`) with prev/this/next links.
#[test]
fn a0_iterate_block_content_two_paragraphs() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:r><w:t>one</w:t></w:r></w:p><w:p><w:r><w:t>two</w:t></w:r></w:p>",
    );
    let ps = d.elements(body, Some(&W::p()));

    let infos = iterate_block_content_elements(&d, body);
    assert_eq!(infos.len(), 2);
    assert_eq!(infos[0].previous_block_content_element, None);
    assert_eq!(infos[0].this_block_content_element, Some(ps[0]));
    assert_eq!(infos[0].next_block_content_element, Some(ps[1]));
    assert_eq!(infos[1].previous_block_content_element, Some(ps[0]));
    assert_eq!(infos[1].this_block_content_element, Some(ps[1]));
    assert_eq!(infos[1].next_block_content_element, None);

    // no child elements → empty chain
    let mut d2 = Dom::new();
    let b2 = body_from(&mut d2, "");
    assert!(iterate_block_content_elements(&d2, b2).is_empty());
}

/// A.0 — FAITHFUL subtlety: when `this` is a `w:tbl`, the next-search starts
/// at the table's FOLLOWING siblings (`ElementsAfterSelf().DescendantsAndSelf()`),
/// so paragraphs INSIDE the table are skipped — the chain is [tbl, p_after],
/// never the cell paragraph.
#[test]
fn a0_iterate_block_content_table_descendants_skipped() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:tbl><w:tr><w:tc><w:p><w:r><w:t>in</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:p><w:r><w:t>after</w:t></w:r></w:p>",
    );
    let tbl = d.elements(body, Some(&W::name("tbl")))[0];
    let p_after = d.elements(body, Some(&W::p()))[0];

    let infos = iterate_block_content_elements(&d, body);
    let chain: Vec<Option<NodeId>> = infos.iter().map(|i| i.this_block_content_element).collect();
    assert_eq!(
        chain,
        vec![Some(tbl), Some(p_after)],
        "tbl's inner paragraph must not appear in the chain"
    );
}

/// A.0 — `GetParagraphInfo` (:2917): for a child of a block-level content
/// container, `this` = first descendant-or-self in {p, tc, txbxContent},
/// nulled when that first hit is a `tc`/`txbxContent` (e.g. a table child);
/// `previous` = the previous ELEMENT sibling (any kind).
#[test]
fn a0_get_paragraph_info() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:r><w:t>a</w:t></w:r></w:p>\
         <w:tbl><w:tr><w:tc><w:p><w:r><w:t>in</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    );
    let p1 = d.elements(body, Some(&W::p()))[0];
    let tbl = d.elements(body, Some(&W::name("tbl")))[0];

    let pi_p = get_paragraph_info(&d, p1);
    assert_eq!(
        pi_p.this_block_content_element,
        Some(p1),
        "p matches itself"
    );
    assert_eq!(pi_p.previous_block_content_element, None);

    let pi_tbl = get_paragraph_info(&d, tbl);
    assert_eq!(
        pi_tbl.this_block_content_element, None,
        "tbl's first {{p,tc,txbxContent}} descendant is a tc → nulled"
    );
    assert_eq!(pi_tbl.previous_block_content_element, Some(p1));
}

/// A.1 — `FixUpDeletedOrInsertedFieldCodesTransform` (:1354): a `w:r/w:instrText`
/// group strictly between two `w:del`-of-`w:fldChar` groups is wrapped in a new
/// `w:del` and its `w:instrText` becomes `w:delInstrText` (attrs + text kept).
#[test]
fn a1_del_flanked_instr_text_becomes_del_instr_text() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p>\
         <w:del w:author=\"x\"><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r></w:del>\
         <w:r><w:instrText xml:space=\"preserve\">PAGE</w:instrText></w:r>\
         <w:del w:author=\"x\"><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:del>\
         </w:p>",
    );
    let out = fix_up_deleted_or_inserted_field_codes_transform(&mut d, body);

    let p = d.elements(out, Some(&W::p()))[0];
    let kids = d.elements(p, None);
    assert_eq!(kids.len(), 3, "three groups: del, del(new), del");
    for &k in &kids {
        assert_eq!(d.name(k).unwrap(), W::del(), "every group now under w:del");
    }
    // middle: w:del > w:r > w:delInstrText, text + attrs preserved
    let mid_r = d.elements(kids[1], Some(&W::r()))[0];
    let dit = d.element(mid_r, &W::name("delInstrText")).unwrap();
    assert_eq!(d.value(dit), "PAGE");
    assert_eq!(
        d.attribute(
            dit,
            &jubarte::xmllinq::XName::get("space", "http://www.w3.org/XML/1998/namespace")
        ),
        Some("preserve"),
        "instrText attributes carried onto delInstrText"
    );
    assert!(
        d.descendants(p, Some(&W::name("instrText"))).is_empty(),
        "no plain instrText remains in the del-flanked case"
    );
}

/// A.1 — ins-flanked variant: the instrText run group is wrapped in `w:ins`
/// and stays `w:instrText`.
#[test]
fn a1_ins_flanked_instr_text_stays_instr_text_under_ins() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p>\
         <w:ins w:author=\"x\"><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r></w:ins>\
         <w:r><w:instrText>PAGE</w:instrText></w:r>\
         <w:ins w:author=\"x\"><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:ins>\
         </w:p>",
    );
    let out = fix_up_deleted_or_inserted_field_codes_transform(&mut d, body);

    let p = d.elements(out, Some(&W::p()))[0];
    let kids = d.elements(p, None);
    assert_eq!(kids.len(), 3);
    for &k in &kids {
        assert_eq!(d.name(k).unwrap(), W::ins());
    }
    let mid_r = d.elements(kids[1], Some(&W::r()))[0];
    let it = d.element(mid_r, &W::name("instrText")).unwrap();
    assert_eq!(d.value(it), "PAGE");
    assert!(
        d.descendants(p, Some(&W::name("delInstrText"))).is_empty(),
        "ins case must NOT produce delInstrText"
    );
}

/// A.1 — no wrap when the instrText group is at the paragraph boundary or the
/// flanks disagree (del on one side, ins on the other).
#[test]
fn a1_unflanked_or_mixed_instr_text_untouched() {
    // instrText first child (i == 0): untouched
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p>\
         <w:r><w:instrText>PAGE</w:instrText></w:r>\
         <w:del><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:del>\
         </w:p>",
    );
    let out = fix_up_deleted_or_inserted_field_codes_transform(&mut d, body);
    let p = d.elements(out, Some(&W::p()))[0];
    assert_eq!(
        d.name(d.elements(p, None)[0]).unwrap(),
        W::r(),
        "boundary instrText run stays bare"
    );
    assert_eq!(d.descendants(p, Some(&W::name("instrText"))).len(), 1);

    // mixed flanks (del before, ins after): untouched
    let mut d2 = Dom::new();
    let b2 = body_from(
        &mut d2,
        "<w:p>\
         <w:del><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r></w:del>\
         <w:r><w:instrText>PAGE</w:instrText></w:r>\
         <w:ins><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:ins>\
         </w:p>",
    );
    let out2 = fix_up_deleted_or_inserted_field_codes_transform(&mut d2, b2);
    let p2 = d2.elements(out2, Some(&W::p()))[0];
    let kids2 = d2.elements(p2, None);
    assert_eq!(
        d2.name(kids2[1]).unwrap(),
        W::r(),
        "mixed flanks: run stays bare"
    );
    assert_eq!(d2.descendants(p2, Some(&W::name("instrText"))).len(), 1);
}

/// A.2 — `AcceptMoveFromRanges` (:1530): content strictly inside a MATCHED
/// `moveFromRangeStart`/`End` id pair is deleted on accept. The markers
/// themselves survive this pass (AcceptAllOtherRevisions strips them later).
#[test]
fn a2_matched_move_from_range_content_deleted() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:r><w:t>keep1</w:t></w:r></w:p>\
         <w:moveFromRangeStart w:id=\"1\" w:name=\"m1\"/>\
         <w:p><w:r><w:t>moved away</w:t></w:r></w:p>\
         <w:moveFromRangeEnd w:id=\"1\"/>\
         <w:p><w:r><w:t>keep2</w:t></w:r></w:p>",
    );
    let out = accept_move_from_ranges(&mut d, body);

    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(
        texts,
        vec!["keep1", "keep2"],
        "the in-range paragraph is deleted, its neighbors kept"
    );
    assert_eq!(
        d.descendants(out, Some(&W::name("moveFromRangeStart")))
            .len(),
        1,
        "range markers survive this pass"
    );
}

/// A.2 — an UNMATCHED `moveFromRangeStart` (no end with the same id) is
/// inert: nothing is deleted, and the input element is returned unchanged
/// (C# returns the same document without a rebuild).
#[test]
fn a2_unmatched_start_is_inert() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:moveFromRangeStart w:id=\"9\" w:name=\"m9\"/>\
         <w:p><w:r><w:t>still here</w:t></w:r></w:p>",
    );
    let out = accept_move_from_ranges(&mut d, body);
    assert_eq!(out, body, "no matched range → identity, no rebuild");
    assert_eq!(d.descendants(out, Some(&W::t())).len(), 1);
}

/// A.2 — FAITHFUL strictly-inside semantics: an element whose OPEN tag
/// precedes the range start (only its close tag is inside) survives; only
/// elements with both tags inside the range are deleted.
#[test]
fn a2_straddling_element_survives() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p>\
         <w:r><w:t>before</w:t></w:r>\
         <w:moveFromRangeStart w:id=\"2\" w:name=\"m2\"/>\
         <w:r><w:t>inside</w:t></w:r>\
         </w:p>\
         <w:moveFromRangeEnd w:id=\"2\"/>",
    );
    let out = accept_move_from_ranges(&mut d, body);

    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(
        texts,
        vec!["before"],
        "straddling w:p survives (close tag only), fully-inside run deleted"
    );
    assert_eq!(
        d.descendants(out, Some(&W::p())).len(),
        1,
        "the straddling paragraph itself is kept"
    );
}

/// A.3 — `AcceptParagraphEndTagsInMoveFromTransform` (:1610). FAITHFUL-BUG:
/// the C# condition is inverted — the coalescing branch only runs when there
/// is a single all-`Other` group (i.e. nothing to coalesce), so the transform
/// is a deep identity rebuild in practice. The TS port (RevisionProcessor.ts
/// :1313, with a "needs rewritten" note at :971) reproduces it verbatim, so
/// the TS goldens and the C# RP baselines carry this behavior. We port it
/// as-is: a paragraph whose mark sits in an open moveFrom range is NOT
/// coalesced with its successor.
#[test]
fn a3_open_move_from_paragraph_mark_not_coalesced_faithful_bug() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p>\
         <w:moveFromRangeStart w:id=\"1\" w:name=\"m1\"/>\
         <w:r><w:t>first</w:t></w:r>\
         </w:p>\
         <w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:t>second</w:t></w:r></w:p>",
    );
    let out = accept_paragraph_end_tags_in_move_from_transform(&mut d, body);

    let ps = d.elements(out, Some(&W::p()));
    assert_eq!(ps.len(), 2, "faithful: NO coalescing happens (dead branch)");
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["first", "second"]);
    assert_eq!(
        d.descendants(out, Some(&W::name("moveFromRangeStart")))
            .len(),
        1,
        "marker preserved by the identity rebuild"
    );
    assert!(
        d.element(ps[1], &W::p_pr()).is_some(),
        "second paragraph keeps its own pPr"
    );
}

/// A.3 — the "nothing to do" branch (single all-Other group) passes the
/// container through with its children intact.
#[test]
fn a3_all_other_container_unchanged() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:r><w:t>a</w:t></w:r></w:p><w:p><w:r><w:t>b</w:t></w:r></w:p>",
    );
    let out = accept_paragraph_end_tags_in_move_from_transform(&mut d, body);
    assert_eq!(d.elements(out, Some(&W::p())).len(), 2);
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["a", "b"]);
}

/// A.3 — `CollapseParagraphTransform` (:1821), unit-level (the intended
/// consumer path is dead upstream, see FAITHFUL-BUG above; A.5a's grouping
/// machine uses its own CollapseTransform): a `w:p` collapses to its
/// children minus `w:pPr`; non-paragraphs rebuild recursively.
#[test]
fn a3_collapse_paragraph_drops_ppr_keeps_content() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:t>x</w:t></w:r><w:r><w:t>y</w:t></w:r></w:p>",
    );
    let p = d.elements(body, Some(&W::p()))[0];
    let out = collapse_paragraph_transform(&mut d, p);
    assert_eq!(out.len(), 2, "pPr dropped, both runs survive");
    for &n in &out {
        assert_eq!(d.name(n).unwrap(), W::r());
    }

    // wrapped: sdt > sdtContent > p — rebuilds the wrapper, collapses inside
    let mut d2 = Dom::new();
    let b2 = body_from(
        &mut d2,
        "<w:sdt><w:sdtContent><w:p><w:pPr/><w:r><w:t>z</w:t></w:r></w:p></w:sdtContent></w:sdt>",
    );
    let sdt = d2.elements(b2, Some(&W::name("sdt")))[0];
    let out2 = collapse_paragraph_transform(&mut d2, sdt);
    assert_eq!(out2.len(), 1);
    assert!(
        d2.descendants(out2[0], Some(&W::p())).is_empty(),
        "inner paragraph collapsed away"
    );
    assert_eq!(d2.descendants(out2[0], Some(&W::t())).len(), 1);
}

/// A.3 — `CoalesqueParagraphEndTagsInMoveFromTransform` (:2645), unit-level:
/// the first group member's paragraph absorbs its own children plus the
/// collapsed content of the group's subsequent paragraphs (first paragraph's
/// pPr wins; later pPrs are dropped by the collapse).
#[test]
fn a3_coalesque_absorbs_subsequent_collapsed_paragraphs() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:pPr><w:jc w:val=\"left\"/></w:pPr><w:r><w:t>one</w:t></w:r></w:p>\
         <w:p><w:pPr><w:jc w:val=\"right\"/></w:pPr><w:r><w:t>two</w:t></w:r></w:p>\
         <w:p><w:r><w:t>three</w:t></w:r></w:p>",
    );
    let ps = d.elements(body, Some(&W::p()));
    let merged = coalesque_paragraph_end_tags_in_move_from_transform(&mut d, ps[0], &ps);

    assert_eq!(d.name(merged).unwrap(), W::p());
    assert!(
        d.descendants(merged, Some(&W::p())).is_empty(),
        "subsequent paragraphs collapsed, not nested"
    );
    let texts: Vec<String> = d
        .descendants(merged, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["one", "two", "three"]);
    let pprs = d.elements(merged, Some(&W::p_pr()));
    assert_eq!(pprs.len(), 1, "only the first paragraph's pPr survives");
    let jc = d.element(pprs[0], &W::name("jc")).unwrap();
    assert_eq!(d.attribute(jc, &W::val()), Some("left"));
}

/// A.4 — `AcceptDeletedAndMovedFromContentControls` (:2491): a `w:sdt`
/// strictly inside a matched `customXmlDelRangeStart`/`End` pair collapses to
/// its `sdtContent` children (wrappers vanish, content stays).
#[test]
fn a4_del_range_sdt_collapses_to_content() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:customXmlDelRangeStart w:id=\"1\"/>\
         <w:sdt><w:sdtPr/><w:sdtContent><w:p><w:r><w:t>kept</w:t></w:r></w:p></w:sdtContent></w:sdt>\
         <w:customXmlDelRangeEnd w:id=\"1\"/>",
    );
    let out = accept_deleted_and_moved_from_content_controls(&mut d, body);

    assert!(
        d.descendants(out, Some(&W::name("sdt"))).is_empty(),
        "sdt wrapper collapsed"
    );
    assert!(d.descendants(out, Some(&W::name("sdtContent"))).is_empty());
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["kept"], "sdtContent children spliced in place");
    assert_eq!(
        d.descendants(out, Some(&W::name("customXmlDelRangeStart")))
            .len(),
        1,
        "range markers survive this pass"
    );
}

/// A.4 — a `w:sdt` strictly inside a matched `customXmlMoveFromRangeStart`/
/// `End` pair is deleted entirely (moveFrom ranges delete everything inside).
#[test]
fn a4_move_from_range_sdt_deleted() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:r><w:t>before</w:t></w:r></w:p>\
         <w:customXmlMoveFromRangeStart w:id=\"2\"/>\
         <w:sdt><w:sdtContent><w:p><w:r><w:t>moved</w:t></w:r></w:p></w:sdtContent></w:sdt>\
         <w:customXmlMoveFromRangeEnd w:id=\"2\"/>",
    );
    let out = accept_deleted_and_moved_from_content_controls(&mut d, body);

    assert!(d.descendants(out, Some(&W::name("sdt"))).is_empty());
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(
        texts,
        vec!["before"],
        "moveFrom-spanned sdt deleted entirely"
    );
}

/// A.4 — FAITHFUL: del ranges track ONLY `w:sdt` tags — a plain paragraph
/// inside a customXmlDelRange is untouched. And with no matched range at all
/// the input element is returned unchanged (identity).
#[test]
fn a4_del_range_ignores_non_sdt_and_unmatched_is_identity() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:customXmlDelRangeStart w:id=\"3\"/>\
         <w:p><w:r><w:t>plain</w:t></w:r></w:p>\
         <w:customXmlDelRangeEnd w:id=\"3\"/>",
    );
    let out = accept_deleted_and_moved_from_content_controls(&mut d, body);
    assert_eq!(out, body, "no sdt collected → nothing to do → identity");
    assert_eq!(d.descendants(out, Some(&W::t())).len(), 1);

    // unmatched moveFrom start alone is inert too
    let mut d2 = Dom::new();
    let b2 = body_from(
        &mut d2,
        "<w:customXmlMoveFromRangeStart w:id=\"9\"/>\
         <w:sdt><w:sdtContent><w:p/></w:sdtContent></w:sdt>",
    );
    let out2 = accept_deleted_and_moved_from_content_controls(&mut d2, b2);
    assert_eq!(out2, b2);
    assert_eq!(d2.descendants(out2, Some(&W::name("sdt"))).len(), 1);
}

/// A.5a — `AcceptDeletedAndMoveFromParagraphMarksTransform` (:2119), RP005
/// shape: two paragraphs, the FIRST's mark deleted (`pPr/rPr/w:del`) → ONE
/// merged paragraph carrying the SECOND's pPr (g.Last(), the :2271 RP052 fix)
/// and both paragraphs' contents.
#[test]
fn a5a_deleted_mark_merges_into_following_paragraph() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p>\
         <w:pPr><w:jc w:val=\"left\"/><w:rPr><w:del w:author=\"x\"/></w:rPr></w:pPr>\
         <w:r><w:t>one</w:t></w:r>\
         </w:p>\
         <w:p>\
         <w:pPr><w:jc w:val=\"right\"/></w:pPr>\
         <w:r><w:t>two</w:t></w:r>\
         </w:p>",
    );
    let out = accept_deleted_and_move_from_paragraph_marks_transform(&mut d, body);

    let ps = d.elements(out, Some(&W::p()));
    assert_eq!(ps.len(), 1, "the two paragraphs merged into one");
    let texts: Vec<String> = d
        .descendants(ps[0], Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["one", "two"], "both contents in document order");
    let pprs = d.elements(ps[0], Some(&W::p_pr()));
    assert_eq!(pprs.len(), 1, "single merged pPr");
    let jc = d.element(pprs[0], &W::name("jc")).unwrap();
    assert_eq!(
        d.attribute(jc, &W::val()),
        Some("right"),
        "merged pPr comes from the LAST group member (g.Last(), :2271)"
    );
}

/// A.5a — trailing nuke (:2276): a fully-deleted paragraph whose mark is
/// `w:del` and which is the LAST block content in the container is removed
/// entirely.
#[test]
fn a5a_trailing_fully_deleted_paragraph_removed() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:r><w:t>keep</w:t></w:r></w:p>\
         <w:p>\
         <w:pPr><w:rPr><w:del w:author=\"x\"/></w:rPr></w:pPr>\
         <w:del w:author=\"x\"><w:r><w:delText>gone</w:delText></w:r></w:del>\
         </w:p>",
    );
    let out = accept_deleted_and_move_from_paragraph_marks_transform(&mut d, body);

    let ps = d.elements(out, Some(&W::p()));
    assert_eq!(ps.len(), 1, "the trailing fully-deleted paragraph is nuked");
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["keep"]);
}

/// A.5a — a table resets the state machine: a deleted-mark paragraph with
/// surviving (non-deleted) content directly before a `w:tbl` stays a
/// paragraph of its own (no successor to merge into), and the body-level
/// `sectPr` is re-appended at the end of the rebuilt container.
#[test]
fn a5a_table_bounds_group_and_sectpr_preserved() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p>\
         <w:pPr><w:jc w:val=\"center\"/><w:rPr><w:del w:author=\"x\"/></w:rPr></w:pPr>\
         <w:r><w:t>survives</w:t></w:r>\
         </w:p>\
         <w:tbl><w:tr><w:tc><w:tcPr/><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
         <w:sectPr><w:pgSz w:w=\"12240\" w:h=\"15840\"/></w:sectPr>",
    );
    let out = accept_deleted_and_move_from_paragraph_marks_transform(&mut d, body);

    let kids = d.elements(out, None);
    let names: Vec<String> = kids
        .iter()
        .map(|&k| d.name(k).unwrap().local_name().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["p", "tbl", "sectPr"],
        "paragraph kept (content not all deleted), table intact, sectPr re-appended last"
    );
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["survives", "cell"]);
}

/// A.5b — `AnnotateRunElementsWithId` (:1935) + `AnnotateContentControlsWithRunIds`
/// (:1945): runs numbered 0.. in doc order; each sdt gets `pt:RunIds` (its own
/// runs, comma-joined) and `pt:UniqueId`.
#[test]
fn a5b_annotate_ids() {
    use jubarte::namespaces::PT;
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:r><w:t>a</w:t></w:r></w:p>\
         <w:sdt><w:sdtContent><w:p><w:r><w:t>b</w:t></w:r><w:r><w:t>c</w:t></w:r></w:p></w:sdtContent></w:sdt>",
    );
    annotate_run_elements_with_id(&mut d, body);
    annotate_content_controls_with_run_ids(&mut d, body);

    let runs = d.descendants(body, Some(&W::r()));
    let ids: Vec<&str> = runs
        .iter()
        .map(|&r| d.attribute(r, &PT::name("UniqueId")).unwrap())
        .collect();
    assert_eq!(ids, vec!["0", "1", "2"]);
    let sdt = d.descendants(body, Some(&W::name("sdt")))[0];
    assert_eq!(d.attribute(sdt, &PT::name("RunIds")), Some("1,2"));
    assert_eq!(d.attribute(sdt, &PT::name("UniqueId")), Some("0"));
}

/// A.5b — plan assert (RP016 shape): an sdt-wrapped paragraph keeps its sdt
/// wrapper through the full `accept_deleted_and_move_from_paragraph_marks`
/// wrapper. (A.5a's chain rebuild strips the wrapper; A.5b restores it via
/// the whole-paragraph branch.)
#[test]
fn a5b_sdt_wrapped_paragraph_keeps_wrapper() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:sdt><w:sdtPr/><w:sdtContent><w:p><w:r><w:t>solo</w:t></w:r></w:p></w:sdtContent></w:sdt>\
         <w:p><w:r><w:t>after</w:t></w:r></w:p>",
    );
    let out = accept_deleted_and_move_from_paragraph_marks(&mut d, body);

    let sdts = d.descendants(out, Some(&W::name("sdt")));
    assert_eq!(sdts.len(), 1, "sdt wrapper restored");
    let inner_texts: Vec<String> = d
        .descendants(sdts[0], Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(inner_texts, vec!["solo"], "sdt holds its paragraph");
    let sdt_kids: Vec<String> = d
        .elements(sdts[0], None)
        .iter()
        .map(|&k| d.name(k).unwrap().local_name().to_string())
        .collect();
    assert_eq!(
        sdt_kids,
        vec!["sdtPr", "sdtContent"],
        "sdt children in Order_sdt order"
    );
    let all_texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(all_texts, vec!["solo", "after"]);
}

/// A.5b — partial re-wrap: when a deleted paragraph mark merges an
/// sdt-wrapped paragraph with its successor, the sdt is re-created INSIDE the
/// merged paragraph around just its own runs.
#[test]
fn a5b_sdt_rewrapped_around_runs_after_merge() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:sdt><w:sdtPr/><w:sdtContent>\
         <w:p><w:pPr><w:rPr><w:del w:author=\"x\"/></w:rPr></w:pPr><w:r><w:t>one</w:t></w:r></w:p>\
         </w:sdtContent></w:sdt>\
         <w:p><w:r><w:t>two</w:t></w:r></w:p>",
    );
    let out = accept_deleted_and_move_from_paragraph_marks(&mut d, body);

    let ps = d.elements(out, Some(&W::p()));
    assert_eq!(ps.len(), 1, "paragraphs merged");
    let sdts = d.descendants(ps[0], Some(&W::name("sdt")));
    assert_eq!(sdts.len(), 1, "sdt re-created inside the merged paragraph");
    let sdt_texts: Vec<String> = d
        .descendants(sdts[0], Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(sdt_texts, vec!["one"], "sdt wraps only its own runs");
    let all_texts: Vec<String> = d
        .descendants(ps[0], Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(
        all_texts,
        vec!["one", "two"],
        "successor content follows outside the sdt"
    );
}

/// A.6 — `RemoveRowsLeftEmptyByMoveFrom` (:2777): a `w:tr` whose cells all
/// lost their block content (no direct `tc` child in {p, tbl, sdt, del, ins,
/// oMath, oMathPara, moveTo}) is dropped; rows with any surviving block
/// content are kept. (The `contains_move_from` gate is applied by the A.10
/// pipeline, not here.)
#[test]
fn a6_rows_emptied_by_move_from_dropped() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:tbl>\
         <w:tr><w:tc><w:tcPr/><w:p><w:r><w:t>alive</w:t></w:r></w:p></w:tc></w:tr>\
         <w:tr><w:tc><w:tcPr/></w:tc></w:tr>\
         </w:tbl>",
    );
    let out = remove_rows_left_empty_by_move_from(&mut d, body);

    let rows = d.descendants(out, Some(&W::name("tr")));
    assert_eq!(rows.len(), 1, "emptied row dropped, surviving row kept");
    assert_eq!(d.descendants(out, Some(&W::t())).len(), 1);
}

/// A.7 — `AcceptDeletedCellsTransform` (:2674) + `Order_tcPr` (:1466), RP034
/// shape: a `cellDel` cell is dropped, its left-neighbor anchor absorbs it via
/// a widened `gridSpan`, and the rebuilt `tcPr` children follow Order_tcPr
/// (`tcW` before `gridSpan`).
#[test]
fn a7_deleted_cell_dropped_anchor_gridspan_widened() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:tbl><w:tr>\
         <w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>a</w:t></w:r></w:p></w:tc>\
         <w:tc><w:tcPr><w:cellDel w:author=\"x\"/></w:tcPr><w:p/></w:tc>\
         <w:tc><w:p><w:r><w:t>c</w:t></w:r></w:p></w:tc>\
         </w:tr></w:tbl>",
    );
    let out = accept_deleted_cells_transform(&mut d, body);

    let cells = d.descendants(out, Some(&W::name("tc")));
    assert_eq!(cells.len(), 2, "deleted cell dropped");
    assert!(
        d.descendants(out, Some(&W::name("cellDel"))).is_empty(),
        "cellDel gone with its cell"
    );
    let tcpr = d.element(cells[0], &W::name("tcPr")).unwrap();
    let kid_names: Vec<String> = d
        .elements(tcpr, None)
        .iter()
        .map(|&k| d.name(k).unwrap().local_name().to_string())
        .collect();
    assert_eq!(
        kid_names,
        vec!["tcW", "gridSpan"],
        "Order_tcPr: tcW(20) before gridSpan(30)"
    );
    let gs = d.element(tcpr, &W::name("gridSpan")).unwrap();
    assert_eq!(
        d.attribute(gs, &W::val()),
        Some("2"),
        "anchor absorbs the one deleted cell"
    );
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["a", "c"]);
}

/// A.7 — a deleted-cell group with NO preceding anchor (row starts deleted)
/// is dropped outright.
#[test]
fn a7_leading_deleted_cell_dropped_without_anchor() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:tbl><w:tr>\
         <w:tc><w:tcPr><w:cellDel w:author=\"x\"/></w:tcPr><w:p/></w:tc>\
         <w:tc><w:p><w:r><w:t>b</w:t></w:r></w:p></w:tc>\
         </w:tr></w:tbl>",
    );
    let out = accept_deleted_cells_transform(&mut d, body);

    let cells = d.descendants(out, Some(&W::name("tc")));
    assert_eq!(cells.len(), 1);
    assert!(
        d.element(cells[0], &W::name("gridSpan")).is_none()
            && d.element(cells[0], &W::name("tcPr")).is_none(),
        "survivor untouched (Other group passthrough)"
    );
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["b"]);
}

/// A.8 — `MergeAdjacentTablesTransform` (:464) + `FixWidths` (:1484): two
/// adjacent tables with revision marks merge into one whose `tblGrid` is the
/// union of the cumulative grid widths; cell widths are refit and spanning
/// cells get a `gridSpan` over the finer grid.
///
/// M112: clean (no-revision) adjacent tables stay separate — put a `w:ins`
/// on the first table's row so the merge still fires for this A.8 test.
#[test]
fn a8_adjacent_tables_merge_with_unioned_grid() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:tbl>\
         <w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid>\
         <w:tr>\
         <w:trPr><w:ins w:id=\"1\" w:author=\"x\" w:date=\"d\"/></w:trPr>\
         <w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>a1</w:t></w:r></w:p></w:tc>\
         <w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>a2</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl>\
         <w:tbl>\
         <w:tblPr/>\
         <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr>\
         <w:tc><w:tcPr><w:tcW w:w=\"4000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>b1</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl>",
    );
    let out = merge_adjacent_tables_transform(&mut d, body);

    let tbls = d.elements(out, Some(&W::name("tbl")));
    assert_eq!(
        tbls.len(),
        1,
        "adjacent same-kind tables with revision marks merged"
    );
    let grid = d.element(tbls[0], &W::name("tblGrid")).unwrap();
    let cols: Vec<String> = d
        .elements(grid, Some(&W::name("gridCol")))
        .iter()
        .map(|&g| d.attribute(g, &W::name("w")).unwrap().to_string())
        .collect();
    assert_eq!(
        cols,
        vec!["2000", "2000"],
        "unioned grid: diffs of {{2000,4000}}"
    );
    assert!(
        d.element(tbls[0], &W::name("tblPr")).is_some(),
        "first table's tblPr carried over"
    );
    let rows = d.elements(tbls[0], Some(&W::name("tr")));
    assert_eq!(rows.len(), 2, "both tables' rows in the merged table");
    // the second table's single wide cell now spans 2 grid columns
    let b1_tc = d.elements(rows[1], Some(&W::name("tc")))[0];
    let gs = d
        .element(b1_tc, &W::name("tcPr"))
        .and_then(|t| d.element(t, &W::name("gridSpan")))
        .expect("wide cell gets a gridSpan");
    assert_eq!(d.attribute(gs, &W::val()), Some("2"));
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["a1", "a2", "b1"]);
}

/// A.9 — `AddEmptyParagraphToAnyEmptyCells` (:1448): a `w:tc` containing only
/// `tcPr` (no block content) gains an empty `w:p`; non-empty cells untouched.
#[test]
fn a9_empty_cell_gains_paragraph() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:tbl><w:tr>\
         <w:tc><w:tcPr/></w:tc>\
         <w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc>\
         </w:tr></w:tbl>",
    );
    let out = add_empty_paragraph_to_any_empty_cells(&mut d, body);

    let cells = d.descendants(out, Some(&W::name("tc")));
    let empty_cell_kids: Vec<String> = d
        .elements(cells[0], None)
        .iter()
        .map(|&k| d.name(k).unwrap().local_name().to_string())
        .collect();
    assert_eq!(
        empty_cell_kids,
        vec!["tcPr", "p"],
        "empty cell gains a w:p after tcPr"
    );
    assert_eq!(
        d.elements(cells[1], Some(&W::p())).len(),
        1,
        "non-empty cell untouched"
    );
}

/// A.0 — `ContentElementsBeforeSelf` (:2926): previous element siblings,
/// nearest first.
#[test]
fn a0_content_elements_before_self() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p><w:r><w:t>1</w:t></w:r></w:p>\
         <w:p><w:r><w:t>2</w:t></w:r></w:p>\
         <w:p><w:r><w:t>3</w:t></w:r></w:p>",
    );
    let ps = d.elements(body, Some(&W::p()));

    assert_eq!(content_elements_before_self(&d, ps[2]), vec![ps[1], ps[0]]);
    assert!(content_elements_before_self(&d, ps[0]).is_empty());
}

/// A.10 — integration: field code + moveFrom range + deleted paragraph mark +
/// cellDel compose through the full 15-step pipeline.
#[test]
fn a10_pipeline_integration() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:p>\
         <w:del w:author=\"x\"><w:r><w:fldChar w:fldCharType=\"begin\"/></w:r></w:del>\
         <w:r><w:instrText>PAGE</w:instrText></w:r>\
         <w:del w:author=\"x\"><w:r><w:fldChar w:fldCharType=\"end\"/></w:r></w:del>\
         <w:r><w:t>normal</w:t></w:r>\
         </w:p>\
         <w:moveFromRangeStart w:id=\"1\" w:name=\"m1\"/>\
         <w:p><w:r><w:t>movedaway</w:t></w:r></w:p>\
         <w:moveFromRangeEnd w:id=\"1\"/>\
         <w:p><w:pPr><w:rPr><w:del w:author=\"x\"/></w:rPr></w:pPr><w:r><w:t>first</w:t></w:r></w:p>\
         <w:p><w:r><w:t>second</w:t></w:r></w:p>\
         <w:tbl><w:tr>\
         <w:tc><w:tcPr><w:tcW w:w=\"1000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>anchor</w:t></w:r></w:p></w:tc>\
         <w:tc><w:tcPr><w:cellDel w:author=\"x\"/></w:tcPr><w:p/></w:tc>\
         </w:tr></w:tbl>",
    );
    let out = accept_revisions_for_part_content(&mut d, body);

    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(
        texts,
        vec!["normal", "first", "second", "anchor"],
        "field code gone, moved-away paragraph deleted, marks merged, table intact"
    );
    assert!(
        !element_has_tracked_revisions(&d, out),
        "no tracked-revision markup survives a full accept"
    );
    let ps = d.elements(out, Some(&W::p()));
    assert_eq!(ps.len(), 2, "field-code para + merged para");
    let cells = d.descendants(out, Some(&W::name("tc")));
    assert_eq!(cells.len(), 1, "deleted cell dropped");
    let gs = d
        .element(cells[0], &W::name("tcPr"))
        .and_then(|t| d.element(t, &W::name("gridSpan")))
        .expect("anchor widened");
    assert_eq!(d.attribute(gs, &W::val()), Some("2"));
}

/// A.10 — RP-baseline sweep: for every RP002–RP052 fixture with dotnet
/// `-Accepted`/`-Rejected` baselines, our accept/reject text-reconstruction
/// must equal the baseline's. Skips when the git-ignored `Docxodus/` corpus
/// is absent (CI runners).
#[test]
fn a10_rp_baseline_sweep() {
    let rp_dir = std::path::Path::new("tests/corpus/Docxodus/TestFiles/RP");
    if !rp_dir.is_dir() {
        eprintln!("SKIP: Docxodus/TestFiles/RP not present");
        return;
    }

    fn doc_text(bytes: &[u8]) -> String {
        let z = jubarte::opc::PartFs::open(bytes).unwrap();
        let xml = z.part_string("word/document.xml").unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        dom.descendants(root, Some(&W::t()))
            .iter()
            .map(|&t| dom.value(t))
            .collect()
    }

    fn processed_text(bytes: &[u8], reject: bool) -> String {
        let z = jubarte::opc::PartFs::open(bytes).unwrap();
        let xml = z.part_string("word/document.xml").unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        let out = if reject {
            reject_revisions_document(&mut dom, root)
        } else {
            accept_revisions_for_part_content(&mut dom, root)
        };
        dom.descendants(out, Some(&W::t()))
            .iter()
            .map(|&t| dom.value(t))
            .collect()
    }

    let mut inputs: Vec<std::path::PathBuf> = std::fs::read_dir(rp_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            if !name.ends_with(".docx")
                || name.ends_with("-Accepted.docx")
                || name.ends_with("-Rejected.docx")
            {
                return false;
            }
            name.get(2..5)
                .and_then(|n| n.parse::<u32>().ok())
                .is_some_and(|n| (2..=52).contains(&n))
        })
        .collect();
    inputs.sort();
    assert!(
        !inputs.is_empty(),
        "RP corpus present but no inputs matched"
    );

    let mut failures: Vec<String> = Vec::new();
    let mut checked = 0;
    for input in &inputs {
        let base = input.with_extension("");
        let bytes = std::fs::read(input).unwrap();
        for (suffix, reject) in [("-Accepted.docx", false), ("-Rejected.docx", true)] {
            let baseline = std::path::PathBuf::from(format!("{}{}", base.display(), suffix));
            if !baseline.is_file() {
                continue;
            }
            checked += 1;
            let expected = doc_text(&std::fs::read(&baseline).unwrap());
            let got = std::panic::catch_unwind(|| processed_text(&bytes, reject));
            match got {
                Ok(got) if got == expected => {}
                Ok(got) => failures.push(format!(
                    "{} [{}]\n  expected: {:?}\n  got:      {:?}",
                    input.file_name().unwrap().to_string_lossy(),
                    if reject { "reject" } else { "accept" },
                    &expected.chars().take(120).collect::<String>(),
                    &got.chars().take(120).collect::<String>(),
                )),
                Err(_) => failures.push(format!(
                    "{} [{}] PANICKED",
                    input.file_name().unwrap().to_string_lossy(),
                    if reject { "reject" } else { "accept" },
                )),
            }
        }
    }
    eprintln!(
        "RP sweep: {checked} projections checked, {} failed",
        failures.len()
    );
    assert!(
        failures.is_empty(),
        "RP baseline mismatches ({}/{checked}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// A.11 — package-scope accept: a docx carrying `w:ins` in an injected
/// footnotes part, a header part, and an `rPrChange` in its styles part
/// accepts clean on EVERY part (`element_has_tracked_revisions` false per
/// part), through the byte facade.
#[test]
fn a11_package_accept_cleans_every_part() {
    use jubarte::opc::PartFs;

    let original = std::fs::read("tests/fixtures/f4/original.docx").unwrap();
    let mut pkg = PartFs::open(&original).unwrap();

    // Inject a footnotes part with an inserted run, wired via rels + CT.
    let footnotes_xml = format!(
        "<w:footnotes xmlns:w=\"{w}\">\
         <w:footnote w:id=\"1\"><w:p><w:ins w:id=\"9\" w:author=\"x\" w:date=\"2020-01-01T00:00:00Z\">\
         <w:r><w:t>inserted note text</w:t></w:r></w:ins></w:p></w:footnote>\
         </w:footnotes>",
        w = W::URI
    );
    pkg.set_part("word/footnotes.xml", footnotes_xml.into_bytes());
    pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
        "footnotes.xml",
    );
    pkg.add_content_type_override(
        "/word/footnotes.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
    );

    // Inject a header part with tracked insertion (covers headers/footers path).
    let header_xml = format!(
        "<w:hdr xmlns:w=\"{w}\">\
         <w:p><w:ins w:id=\"11\" w:author=\"x\" w:date=\"2020-01-01T00:00:00Z\">\
         <w:r><w:t>header inserted</w:t></w:r></w:ins></w:p>\
         </w:hdr>",
        w = W::URI
    );
    pkg.set_part("word/header1.xml", header_xml.into_bytes());
    pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        "header1.xml",
    );
    pkg.add_content_type_override(
        "/word/header1.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
    );

    // Replace the styles part with one carrying an rPrChange.
    let styles_xml = format!(
        "<w:styles xmlns:w=\"{w}\">\
         <w:style w:type=\"paragraph\" w:styleId=\"Normal\">\
         <w:rPr><w:b/><w:rPrChange w:id=\"3\" w:author=\"x\" w:date=\"2020-01-01T00:00:00Z\"><w:rPr/></w:rPrChange></w:rPr>\
         </w:style></w:styles>",
        w = W::URI
    );
    pkg.set_part("word/styles.xml", styles_xml.into_bytes());

    let dirty = pkg.to_zip().unwrap();
    let accepted = jubarte::document_comparer::accept_revisions(&dirty).unwrap();

    let out = PartFs::open(&accepted).unwrap();
    for part in [
        "word/document.xml",
        "word/footnotes.xml",
        "word/header1.xml",
        "word/styles.xml",
    ] {
        let xml = out
            .part_string(part)
            .unwrap_or_else(|| panic!("{part} present"));
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        assert!(
            !jubarte::revision_processor::element_has_tracked_revisions(&dom, root),
            "{part} still carries tracked revisions after package accept"
        );
    }
    // the inserted footnote text survives the accept
    let fx = out.part_string("word/footnotes.xml").unwrap();
    assert!(
        fx.contains("inserted note text"),
        "accepted ins content kept"
    );
    let hx = out.part_string("word/header1.xml").unwrap();
    assert!(
        hx.contains("header inserted"),
        "accepted header ins content kept"
    );
    // the styles rPr kept the new (accepted) property
    let sx = out.part_string("word/styles.xml").unwrap();
    assert!(
        sx.contains("<w:b"),
        "accepted style keeps the changed property"
    );
}

/// A.11 — package-scope reject: the same package rejects to its ORIGINAL
/// projection — inserted footnote/header content removed, styles rPr
/// reverted to the rPrChange's stored (old) properties.
#[test]
fn a11_package_reject_restores_original() {
    use jubarte::opc::PartFs;

    let original = std::fs::read("tests/fixtures/f4/original.docx").unwrap();
    let mut pkg = PartFs::open(&original).unwrap();
    let footnotes_xml = format!(
        "<w:footnotes xmlns:w=\"{w}\">\
         <w:footnote w:id=\"1\"><w:p><w:ins w:id=\"9\" w:author=\"x\" w:date=\"2020-01-01T00:00:00Z\">\
         <w:r><w:t>inserted note text</w:t></w:r></w:ins></w:p></w:footnote>\
         </w:footnotes>",
        w = W::URI
    );
    pkg.set_part("word/footnotes.xml", footnotes_xml.into_bytes());
    pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
        "footnotes.xml",
    );
    pkg.add_content_type_override(
        "/word/footnotes.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
    );
    let header_xml = format!(
        "<w:hdr xmlns:w=\"{w}\">\
         <w:p><w:ins w:id=\"11\" w:author=\"x\" w:date=\"2020-01-01T00:00:00Z\">\
         <w:r><w:t>header inserted</w:t></w:r></w:ins></w:p>\
         </w:hdr>",
        w = W::URI
    );
    pkg.set_part("word/header1.xml", header_xml.into_bytes());
    pkg.add_document_relationship(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
        "header1.xml",
    );
    pkg.add_content_type_override(
        "/word/header1.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
    );
    let styles_xml = format!(
        "<w:styles xmlns:w=\"{w}\">\
         <w:style w:type=\"paragraph\" w:styleId=\"Normal\">\
         <w:rPr><w:b/><w:rPrChange w:id=\"3\" w:author=\"x\" w:date=\"2020-01-01T00:00:00Z\"><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr>\
         </w:style></w:styles>",
        w = W::URI
    );
    pkg.set_part("word/styles.xml", styles_xml.into_bytes());

    let dirty = pkg.to_zip().unwrap();
    let rejected = jubarte::document_comparer::reject_revisions(&dirty).unwrap();

    let out = PartFs::open(&rejected).unwrap();
    let fx = out.part_string("word/footnotes.xml").unwrap();
    assert!(
        !fx.contains("inserted note text"),
        "rejected ins content removed from footnotes part"
    );
    let hx = out.part_string("word/header1.xml").unwrap();
    assert!(
        !hx.contains("header inserted"),
        "rejected ins content removed from header part"
    );
    let sx = out.part_string("word/styles.xml").unwrap();
    assert!(
        sx.contains("<w:i"),
        "styles rPr reverted to the old properties"
    );
    assert!(!sx.contains("rPrChange"), "no change markup remains");
    for part in [
        "word/document.xml",
        "word/footnotes.xml",
        "word/header1.xml",
        "word/styles.xml",
    ] {
        let xml = out.part_string(part).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        assert!(
            !jubarte::revision_processor::element_has_tracked_revisions(&dom, root),
            "{part} still carries tracked revisions after package reject"
        );
    }
}

// --- gems harvested from open recipe PRs #56/#58 ---

/// Two adjacent, same-kind tables at body scope.
fn two_adjacent_tables_body(dom: &mut Dom) -> NodeId {
    body_from(
        dom,
        "<w:tbl>\
         <w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr>\
         <w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid>\
         <w:tr>\
         <w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>a1</w:t></w:r></w:p></w:tc>\
         <w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>a2</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl>\
         <w:tbl>\
         <w:tblPr/>\
         <w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid>\
         <w:tr>\
         <w:tc><w:tcPr><w:tcW w:w=\"4000\" w:type=\"dxa\"/></w:tcPr><w:p><w:r><w:t>b1</w:t></w:r></w:p></w:tc>\
         </w:tr>\
         </w:tbl>",
    )
}

fn empty_table_cell_body(dom: &mut Dom) -> NodeId {
    body_from(
        dom,
        "<w:tbl><w:tr>\
         <w:tc><w:tcPr/></w:tc>\
         <w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc>\
         </w:tr></w:tbl>",
    )
}

/// CHANGED CODE: the full pipeline's A.9 step (`AddEmptyParagraphToAnyEmptyCells`)
/// now runs through `accept_revisions_document`, so a `w:tc` with only `w:tcPr`
/// gains an empty `w:p`.
#[test]
fn accept_revisions_document_adds_paragraph_to_empty_table_cells() {
    let mut d = Dom::new();
    let body = empty_table_cell_body(&mut d);
    let out = accept_revisions_document(&mut d, body);

    let cells = d.descendants(out, Some(&W::name("tc")));
    assert_eq!(cells.len(), 2);
    assert_eq!(
        d.elements(cells[0], Some(&W::p())).len(),
        1,
        "accept_revisions_document now fills empty cells with an empty paragraph (A.9)"
    );
}

/// PRIOR BEHAVIOR PATH: `accept_revisions_for_element` has no A.9 step, so an
/// empty `w:tc` stays empty — the narrower pipeline's behavior is unchanged
/// by this PR.
#[test]
fn accept_revisions_for_element_leaves_empty_table_cells_untouched() {
    let mut d = Dom::new();
    let body = empty_table_cell_body(&mut d);
    let out = accept_revisions_for_element(&mut d, body);

    let cells = d.descendants(out, Some(&W::name("tc")));
    assert_eq!(cells.len(), 2);
    assert!(
        d.elements(cells[0], Some(&W::p())).is_empty(),
        "the element-scope pipeline never adds paragraphs to empty cells"
    );
}

/// M112 SUPERSESSION (this test originally asserted the pre-M112 PowerTools
/// contract "the full pipeline merges adjacent tables"): Word Compare does
/// not merge clean adjacent tables, and A.10 consumes every revision mark
/// (ins/del/move/cellIns/cellDel) before its merge step runs, so the
/// document-scope pipeline always sees clean tables — adjacent tables stay
/// separate. Merging still fires on direct `merge_adjacent_tables_transform`
/// calls with marked tables (a8).
#[test]
fn accept_revisions_document_keeps_adjacent_tables_separate_post_m112() {
    let mut d = Dom::new();
    let body = two_adjacent_tables_body(&mut d);
    let out = accept_revisions_document(&mut d, body);

    let tbls = d.elements(out, Some(&W::name("tbl")));
    assert_eq!(
        tbls.len(),
        2,
        "M112: clean adjacent tables stay separate through the full A.10 pipeline"
    );
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["a1", "a2", "b1"]);
}

/// PRIOR BEHAVIOR PATH: `accept_revisions_for_element` (:270) is untouched by
/// this PR and still implements only RemoveRsid → AcceptMoveFromMoveTo →
/// AcceptAllOtherRevisions — it does NOT merge adjacent tables. Regression
/// guard so the still-exported, narrower entry point keeps behaving exactly
/// as it did before `accept_revisions_document` was rewired away from it.
#[test]
fn accept_revisions_for_element_leaves_adjacent_tables_unmerged() {
    let mut d = Dom::new();
    let body = two_adjacent_tables_body(&mut d);
    let out = accept_revisions_for_element(&mut d, body);

    let tbls = d.elements(out, Some(&W::name("tbl")));
    assert_eq!(
        tbls.len(),
        2,
        "the element-scope pipeline has no table-merge step, unlike accept_revisions_document"
    );
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(
        texts,
        vec!["a1", "a2", "b1"],
        "content itself is unaffected"
    );
}

/// M112 SUPERSESSION (this test originally asserted the pre-M112 "reject
/// path also merges adjacent tables"): `reject_revisions_document`'s final
/// step is the same A.10 pipeline, whose merge step only ever sees
/// mark-free tables — so the reject path keeps clean adjacent tables
/// separate too.
#[test]
fn reject_revisions_document_keeps_adjacent_tables_separate_post_m112() {
    let mut d = Dom::new();
    let body = two_adjacent_tables_body(&mut d);
    let out = reject_revisions_document(&mut d, body);

    let tbls = d.elements(out, Some(&W::name("tbl")));
    assert_eq!(
        tbls.len(),
        2,
        "M112: clean adjacent tables stay separate through the reject path too"
    );
}

/// A.5b regression (CHANGED CODE): a content control whose ENTIRE content is
/// a deleted-mark, deleted-content paragraph that is the container's LAST
/// block content gets "nuked" whole by A.5a's DeletedRange grouping
/// (`accept_deleted_and_move_from_paragraph_marks_transform`'s `continue` at
/// the "nuke: never attached" arm) — its run never reaches `new_document`.
/// `add_block_level_content_controls` must SKIP such a content control
/// instead of panicking. Before the fix, the old `.map(...).expect(...)`
/// lookup for the (now-missing) run id would panic with "annotated run
/// missing from the transformed document".
#[test]
fn a5b_fully_deleted_content_control_is_skipped_not_panicked() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:sdt><w:sdtPr/><w:sdtContent>\
         <w:p><w:pPr><w:rPr><w:del w:author=\"x\"/></w:rPr></w:pPr>\
         <w:del><w:r><w:delText>gone</w:delText></w:r></w:del></w:p>\
         </w:sdtContent></w:sdt>",
    );

    // must not panic (this call alone is the regression assertion)
    let out = accept_deleted_and_move_from_paragraph_marks(&mut d, body);

    assert!(
        d.descendants(out, Some(&W::name("sdt"))).is_empty(),
        "the whole content control was nuked along with its only paragraph"
    );
    assert!(
        d.elements(out, Some(&W::p())).is_empty(),
        "the fully-deleted last paragraph is dropped, not left behind empty"
    );
}

/// A.5b regression, mixed (CHANGED CODE + PRIOR BEHAVIOR side by side): one
/// content control is fully deleted (nuked, must be skipped gracefully — the
/// fix) while a PRECEDING, untouched content control must still be restored
/// exactly like before (`a5b_sdt_wrapped_paragraph_keeps_wrapper`'s prior
/// behavior). The `filter_map` refactor must not affect sdts whose runs DO
/// survive, even when a later sibling sdt's runs are entirely gone.
#[test]
fn a5b_mixed_surviving_and_fully_deleted_content_controls() {
    let mut d = Dom::new();
    let body = body_from(
        &mut d,
        "<w:sdt><w:sdtPr/><w:sdtContent><w:p><w:r><w:t>keep me</w:t></w:r></w:p></w:sdtContent></w:sdt>\
         <w:sdt><w:sdtPr/><w:sdtContent>\
         <w:p><w:pPr><w:rPr><w:del w:author=\"x\"/></w:rPr></w:pPr>\
         <w:del><w:r><w:delText>gone</w:delText></w:r></w:del></w:p>\
         </w:sdtContent></w:sdt>",
    );

    let out = accept_deleted_and_move_from_paragraph_marks(&mut d, body);

    let sdts = d.descendants(out, Some(&W::name("sdt")));
    assert_eq!(
        sdts.len(),
        1,
        "prior behavior preserved: the surviving sdt is still restored; the \
         fully-deleted one is skipped rather than panicking"
    );
    let texts: Vec<String> = d
        .descendants(out, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .collect();
    assert_eq!(texts, vec!["keep me"]);
}
