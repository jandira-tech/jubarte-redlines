//! Port of `RevisionProcessor.ts` — accept-revisions pipeline (M3).
//!
//! Scope (per the implementation plan): the ACCEPT path only. Reject,
//! consolidate, and the HTML/markdown surfaces are out of scope.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::markup_simplifier::remove_rsid_transform;
use crate::namespaces::{M, PT, W};
use crate::xmllinq::{Dom, NodeId, XName};

/// Local names of `RevisionProcessor.TrackedRevisionsElements` (W namespace).
const TRACKED_REVISION_LOCALS: &[&str] = &[
    "cellDel",
    "cellIns",
    "cellMerge",
    "customXmlDelRangeEnd",
    "customXmlDelRangeStart",
    "customXmlInsRangeEnd",
    "customXmlInsRangeStart",
    "customXmlMoveFromRangeEnd",
    "customXmlMoveFromRangeStart",
    "customXmlMoveToRangeEnd",
    "customXmlMoveToRangeStart",
    "del",
    "delInstrText",
    "delText",
    "ins",
    "moveFrom",
    "moveFromRangeEnd",
    "moveFromRangeStart",
    "moveTo",
    "moveToRangeEnd",
    "moveToRangeStart",
    "numberingChange",
    "pPrChange",
    "rPrChange",
    "sectPrChange",
    "tblGridChange",
    "tblPrChange",
    "tblPrExChange",
    "tcPrChange",
    "trPrChange",
];

static TRACKED_REVISION_LOCAL_SET: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| TRACKED_REVISION_LOCALS.iter().copied().collect());

/// `RevisionProcessor.TrackedRevisionsElements` — the element names whose
/// presence indicates the document carries tracked revisions.
pub fn tracked_revisions_elements() -> Vec<XName> {
    TRACKED_REVISION_LOCALS.iter().map(|l| W::name(l)).collect()
}

/// True if any descendant (or `root` itself) is a tracked-revision element.
/// Port of `PartHasTrackedRevisions` applied to a single element tree.
///
/// ACCEPT-SCAN-01: non-allocating DFS + static local-name set (no per-call
/// `Vec<XName>` / full `descendants_and_self` materialization).
pub fn element_has_tracked_revisions(dom: &Dom, root: NodeId) -> bool {
    fn walk(dom: &Dom, id: NodeId) -> bool {
        if let Some(name) = dom.name(id)
            && name.namespace_name() == W::URI
            && TRACKED_REVISION_LOCAL_SET.contains(name.local_name())
        {
            return true;
        }
        for i in 0..dom.child_count(id) {
            if walk(dom, dom.child_at(id, i)) {
                return true;
            }
        }
        false
    }
    walk(dom, root)
}

/// Does `el` have a child element named `local` (in the W namespace)?
fn has_child(dom: &Dom, el: NodeId, name: &XName) -> bool {
    dom.element(el, name).is_some()
}

/// Path check `el / a / b` exists (a, b are direct-child element names).
fn has_path(dom: &Dom, el: NodeId, a: &XName, b: &XName) -> bool {
    dom.elements(el, Some(a))
        .into_iter()
        .any(|ae| has_child(dom, ae, b))
}

/// Port of `AcceptMoveFromMoveToTransform` — unwrap `w:moveTo`, drop `w:moveFrom`.
pub fn accept_move_from_move_to_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    if !dom.is_element(node) {
        return vec![dom.clone_subtree(node)];
    }
    let name = dom.name(node).unwrap();
    if name == W::name("moveTo") {
        let mut out = Vec::new();
        for c in dom.nodes(node) {
            out.extend(accept_move_from_move_to_transform(dom, c));
        }
        return out;
    }
    if name == W::name("moveFrom") {
        return vec![];
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        for tn in accept_move_from_move_to_transform(dom, c) {
            dom.add(ne, tn);
        }
    }
    vec![ne]
}

/// True iff a transformed element still carries run/inline content (port of
/// `HasRunContent`) — used to decide whether an emptied `w:hyperlink` survives.
fn has_run_content(dom: &Dom, element: NodeId) -> bool {
    let content_names = [
        W::name("r"),
        W::name("smartTag"),
        W::name("ins"),
        W::name("del"),
        W::name("hyperlink"),
        W::name("fldSimple"),
        W::name("sdt"),
    ];
    dom.elements(element, None)
        .into_iter()
        .any(|e| dom.name(e).is_some_and(|n| content_names.contains(&n)))
}

/// Port of `AcceptAllOtherRevisionsTransform` — accept inserts (unwrap `w:ins`),
/// drop deletions and revision-range markers, accept formatting-change markers,
/// handle deleted rows/tables, cell merges, and empty hyperlink shells.
pub fn accept_all_other_revisions_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    if !dom.is_element(node) {
        return vec![dom.clone_subtree(node)];
    }
    let name = dom.name(node).unwrap();

    // Accept inserted content: collapse w:ins.
    if name == W::ins() {
        let mut out = Vec::new();
        for c in dom.nodes(node) {
            out.extend(accept_all_other_revisions_transform(dom, c));
        }
        return out;
    }

    // Drop revision-range markers (handled by the range transforms).
    let drop_markers = [
        "customXmlDelRangeStart",
        "customXmlDelRangeEnd",
        "customXmlInsRangeStart",
        "customXmlInsRangeEnd",
        "customXmlMoveFromRangeStart",
        "customXmlMoveFromRangeEnd",
        "customXmlMoveToRangeStart",
        "customXmlMoveToRangeEnd",
        "moveFromRangeStart",
        "moveFromRangeEnd",
        "moveToRangeStart",
        "moveToRangeEnd",
    ];
    if name.namespace_name() == W::URI && drop_markers.contains(&name.local_name()) {
        return vec![];
    }

    // Accept formatting-change / deleted-content markers by removing them.
    let drop_format = [
        "pPrChange",
        "rPrChange",
        "tblPrChange",
        "tblGridChange",
        "tcPrChange",
        "trPrChange",
        "tblPrExChange",
        "sectPrChange",
        "numberingChange",
        "delInstrText",
        "delText",
        "cellIns",
    ];
    if name.namespace_name() == W::URI && drop_format.contains(&name.local_name()) {
        return vec![];
    }

    // m:f / m:fPr / m:ctrlPr / w:del → remove the math fraction.
    if name == M::name("f") {
        let removed = dom
            .elements(node, Some(&M::name("fPr")))
            .into_iter()
            .any(|fpr| has_path(dom, fpr, &M::name("ctrlPr"), &W::del()));
        if removed {
            return vec![];
        }
    }

    // w:tr / w:trPr / w:del → deleted row.
    if name == W::name("tr") && has_path(dom, node, &W::name("trPr"), &W::del()) {
        return vec![];
    }

    // w:tbl whose rows are ALL deleted → drop the whole table.
    if name == W::name("tbl") {
        let rows = dom.elements(node, Some(&W::name("tr")));
        if !rows.is_empty()
            && rows
                .iter()
                .all(|&tr| has_path(dom, tr, &W::name("trPr"), &W::del()))
        {
            return vec![];
        }
    }

    // Accept deleted text: drop w:del.
    if name == W::del() {
        return vec![];
    }

    // Vertically-merged cell markers.
    if name == W::name("cellMerge") {
        let parent_is_tcpr = dom
            .parent(node)
            .and_then(|p| dom.name(p))
            .is_some_and(|pn| pn == W::name("tcPr"));
        if parent_is_tcpr {
            let vmerge = dom
                .attribute(node, &W::name("vMerge"))
                .map(|s| s.to_string());
            if vmerge.as_deref() == Some("rest") {
                let v = dom.new_element(W::name("vMerge"));
                dom.set_attribute_value(v, &W::val(), Some("restart"));
                return vec![v];
            }
            if vmerge.as_deref() == Some("cont") {
                let v = dom.new_element(W::name("vMerge"));
                dom.set_attribute_value(v, &W::val(), Some("continue"));
                return vec![v];
            }
        }
    }

    // w:hyperlink that collapses to an empty shell after accepting children → drop.
    if name == W::name("hyperlink") {
        let ne = dom.new_element(name);
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for c in dom.nodes(node) {
            for tn in accept_all_other_revisions_transform(dom, c) {
                dom.add(ne, tn);
            }
        }
        return if has_run_content(dom, ne) {
            vec![ne]
        } else {
            vec![]
        };
    }

    // Identity clone with transformed children.
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        for tn in accept_all_other_revisions_transform(dom, c) {
            dom.add(ne, tn);
        }
    }
    vec![ne]
}

/// Port of `AcceptRevisionsForElement` (the documented simple-revision entry
/// point, RevisionProcessor.ts:1004): RemoveRsid → AcceptMoveFromMoveTo →
/// AcceptAllOtherRevisions → strip PT.UniqueId/RunIds → drop empty w:numPr.
///
/// NOTE: this is the element-level accept (handles the common ins/del/format
/// cases). The fuller part-level pipeline (`AcceptRevisionsForPart`) adds
/// deleted-paragraph-mark merging, move-from ranges, field codes, content
/// controls, table merges, and OrderTcPr — see the module status note.
///
/// ACCEPT-SKIP-01: when the subtree has no tracked-revision elements, skip
/// the two identity full-tree rebuilds (move + all-other) after RemoveRsid.
pub fn accept_revisions_for_element(dom: &mut Dom, element: NodeId) -> NodeId {
    let has_rev = element_has_tracked_revisions(dom, element);
    let e = remove_rsid_transform(dom, element).expect("root not dropped by rsid removal");
    let e = if has_rev {
        let v = accept_move_from_move_to_transform(dom, e);
        debug_assert_eq!(v.len(), 1);
        let e = v[0];
        let v = accept_all_other_revisions_transform(dom, e);
        debug_assert_eq!(v.len(), 1);
        v[0]
    } else {
        e
    };

    // Strip PT.UniqueId / PT.RunIds attributes from all descendants.
    let unique_id = PT::name("UniqueId");
    let run_ids = PT::name("RunIds");
    for d in dom.descendants_and_self(e, None) {
        dom.set_attribute_value(d, &unique_id, None);
        dom.set_attribute_value(d, &run_ids, None);
    }

    // Remove empty w:numPr elements.
    let num_pr = W::name("numPr");
    for np in dom.descendants(e, Some(&num_pr)) {
        if !dom.has_elements(np) {
            dom.remove(np);
        }
    }
    e
}

// ─────────────────────────────────────────────────────────────────────────────
// P0 — document-scope accept/reject (mirror of AcceptRevisionsDocument /
// RejectRevisionsDocument, RevisionProcessor.ts:909/:155). Needed by M4.B
// HashBlockLevelContent, which hashes source1's accepted projection and
// source2's rejected projection. pt:Unid is preserved (only PT.UniqueId/RunIds
// are stripped by accept), so the projection's hashes correlate back.
// ─────────────────────────────────────────────────────────────────────────────

/// `AcceptRevisionsDocument` at element scope — accept all tracked revisions in
/// the `root` element's subtree, returning the new root. Runs the FULL
/// part-content pipeline (A.10), matching `AcceptRevisionsForPart` (:1314).
pub fn accept_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId {
    accept_revisions_for_part_content(dom, root)
}

/// The "skipping" parent used by `ReverseRevisionsTransform` — nearest ancestor
/// whose name is not `w:sdtContent`/`w:sdt`/`w:smartTag`.
fn effective_parent(dom: &Dom, node: NodeId) -> Option<NodeId> {
    let skip = [W::name("sdtContent"), W::name("sdt"), W::name("smartTag")];
    dom.ancestors(node, None)
        .into_iter()
        .find(|&a| dom.name(a).is_some_and(|n| !skip.contains(&n)))
}

/// Port of `ReverseRevisionsForPartTransform` — invert the *sense* of each
/// revision (del↔ins, moveFrom↔moveTo, delText→t, custom-xml ranges, …),
/// context-aware via the effective parent. Builds a fresh subtree.
///
/// Faithful quirk preserved: the `InInsert` flag is effectively always false
/// (the TS creates a fresh `ReverseRevisionsInfo{InInsert:true}` for inserted
/// runs but never threads it into the recursion), so the `w:t`→`w:delText`
/// (InInsert) and `instrText`→`delInstrText` (InInsert) branches never fire.
fn reverse_revisions_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    let parent = effective_parent(dom, node);
    let parent_name = parent.and_then(|p| dom.name(p));
    let grandparent_is_ppr = parent
        .and_then(|p| dom.parent(p))
        .and_then(|gp| dom.name(gp))
        .is_some_and(|n| n == W::p_pr());

    let in_p_or_hyperlink =
        matches!(&parent_name, Some(n) if *n == W::p() || *n == W::name("hyperlink"));

    // Deleted run / deleted math char → w:ins (wrapping reversed children).
    if name == W::del() && (in_p_or_hyperlink || parent_name.as_ref() == Some(&M::name("r"))) {
        return rebuild_named(dom, W::ins(), node, false);
    }
    // Deleted paragraph mark (del in rPr/pPr) → empty w:ins.
    if name == W::del() && parent_name.as_ref() == Some(&W::r_pr()) && grandparent_is_ppr {
        return dom.new_element(W::ins());
    }
    // Inserted paragraph mark (ins in rPr/pPr) → empty w:del.
    if name == W::ins() && parent_name.as_ref() == Some(&W::r_pr()) && grandparent_is_ppr {
        return dom.new_element(W::del());
    }
    // Inserted run / inserted math char → w:del.
    if name == W::ins() && (in_p_or_hyperlink || parent_name.as_ref() == Some(&M::name("r"))) {
        return rebuild_named(dom, W::del(), node, false);
    }
    // Deleted / inserted table row (del/ins in trPr) → swap.
    if name == W::del() && parent_name.as_ref() == Some(&W::name("trPr")) {
        return dom.new_element(W::ins());
    }
    if name == W::ins() && parent_name.as_ref() == Some(&W::name("trPr")) {
        return dom.new_element(W::del());
    }

    // moveFrom↔moveTo and their ranges (attributes preserved).
    let swap_pairs: &[(&str, &str)] = &[
        ("moveFrom", "moveTo"),
        ("moveFromRangeStart", "moveToRangeStart"),
        ("moveFromRangeEnd", "moveToRangeEnd"),
        ("moveTo", "moveFrom"),
        ("moveToRangeStart", "moveFromRangeStart"),
        ("moveToRangeEnd", "moveFromRangeEnd"),
        ("customXmlDelRangeStart", "customXmlInsRangeStart"),
        ("customXmlDelRangeEnd", "customXmlInsRangeEnd"),
        ("customXmlInsRangeStart", "customXmlDelRangeStart"),
        ("customXmlInsRangeEnd", "customXmlDelRangeEnd"),
        ("customXmlMoveFromRangeStart", "customXmlMoveToRangeStart"),
        ("customXmlMoveFromRangeEnd", "customXmlMoveToRangeEnd"),
        ("customXmlMoveToRangeStart", "customXmlMoveFromRangeStart"),
        ("customXmlMoveToRangeEnd", "customXmlMoveFromRangeEnd"),
        ("delInstrText", "instrText"),
        ("delText", "t"),
    ];
    if name.namespace_name() == W::URI
        && let Some((_, to)) = swap_pairs
            .iter()
            .find(|(from, _)| *from == name.local_name())
    {
        return rebuild_named(dom, W::name(to), node, true);
    }

    // Identity: rebuild with attributes + reversed children.
    rebuild_named(dom, name, node, true)
}

/// Build `<new_name attrs? children…>` from `src`'s children (reversed).
fn rebuild_named(dom: &mut Dom, new_name: XName, src: NodeId, keep_attrs: bool) -> NodeId {
    let ne = dom.new_element(new_name);
    if keep_attrs {
        for (an, av) in dom.attributes(src) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
    }
    for c in dom.nodes(src) {
        let t = reverse_revisions_transform(dom, c);
        dom.add(ne, t);
    }
    ne
}

/// Port of `RejectRevisionsForPartTransform` — revert the *non-invertible*
/// revisions (property changes) and drop inserted structural markers. Returns
/// `None` to drop the node.
fn reject_revisions_for_part_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(dom.clone_subtree(node));
    }
    let name = dom.name(node).unwrap();

    // Inserted numbering properties: numPr containing w:ins → drop.
    if name == W::name("numPr") && dom.element(node, &W::ins()).is_some() {
        return None;
    }
    // Property-change reverts: replace the prop element by the change's saved copy.
    let change_reverts: &[(&str, &str, &str)] = &[
        ("pPr", "pPrChange", "pPr"),
        ("sectPr", "sectPrChange", "sectPr"),
        ("tblGrid", "tblGridChange", "tblGrid"),
        ("tcPr", "tcPrChange", "tcPr"),
        ("trPr", "trPrChange", "trPr"),
        ("tblPrEx", "tblPrExChange", "tblPrEx"),
        ("tblPr", "tblPrChange", "tblPr"),
    ];
    if name.namespace_name() == W::URI
        && let Some((_, change, saved)) = change_reverts
            .iter()
            .find(|(prop, _, _)| *prop == name.local_name())
        && let Some(chg) = dom.element(node, &W::name(change))
    {
        // the saved <w:pPr>/<w:tcPr>/… inside the *Change element
        let new_prop = match dom.element(chg, &W::name(saved)) {
            Some(sp) => dom.clone_subtree(sp),
            None => dom.new_element(W::name(saved)),
        };
        // pPrChange specially re-adds the live rPr (so run-mark formatting survives)
        if name == W::p_pr()
            && let Some(rpr) = dom.element(node, &W::r_pr())
        {
            let rpr_clone = dom.clone_subtree(rpr);
            dom.add(new_prop, rpr_clone);
        }
        return reject_revisions_for_part_transform(dom, new_prop);
    }
    // rPrChange: replace rPr by the change's saved rPr.
    if name == W::r_pr()
        && let Some(chg) = dom.element(node, &W::name("rPrChange"))
    {
        let saved = dom.element(chg, &W::r_pr());
        let new_rpr = match saved {
            Some(sp) => dom.clone_subtree(sp),
            None => dom.new_element(W::r_pr()),
        };
        return reject_revisions_for_part_transform(dom, new_rpr);
    }
    // numberingChange / cellDel / cellMerge → drop.
    if name == W::name("numberingChange")
        || name == W::name("cellDel")
        || name == W::name("cellMerge")
    {
        return None;
    }
    // tc whose tcPr contains a cellIns → drop the inserted cell.
    if name == W::name("tc") {
        let has_cell_ins = dom
            .elements(node, Some(&W::name("tcPr")))
            .into_iter()
            .any(|tcpr| dom.element(tcpr, &W::name("cellIns")).is_some());
        if has_cell_ins {
            return None;
        }
    }

    // Identity: rebuild with attributes + transformed children.
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        if let Some(t) = reject_revisions_for_part_transform(dom, c) {
            dom.add(ne, t);
        }
    }
    Some(ne)
}

/// `RejectRevisionsDocument` at element scope: revert property changes, invert
/// the sense of every remaining revision, strip rsids, then accept. The net
/// effect is the document's *original* (pre-revision) projection.
pub fn reject_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId {
    let reverted =
        reject_revisions_for_part_transform(dom, root).expect("reject: root must not be dropped");
    let reversed = reverse_revisions_transform(dom, reverted);
    let derssid = remove_rsid_transform(dom, reversed).expect("reject: root not dropped by rsid");
    accept_revisions_for_part_content(dom, derssid)
}

// ───────────────────────────── A.0 — part-pipeline walkers ──────────────────

/// `TagTypeEnum` (RevisionProcessor.cs :2389): how an element appears in the
/// doc-order tag stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    Element,
    EmptyElement,
    EndElement,
}

/// `Tag` (RevisionProcessor.cs :2393): one open/empty/close event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag {
    pub element: NodeId,
    pub tag_type: TagType,
}

/// A.0 — `DescendantAndSelfTags` (:2397): stream the element and its
/// descendants as open/empty/close tags in document order. FAITHFUL: a child
/// with no nodes AT ALL (no elements, no text) is `EmptyElement`; a child
/// with only text still opens and closes; the root element itself always
/// gets an open/close pair, never `EmptyElement`.
pub fn descendant_and_self_tags(dom: &Dom, element: NodeId) -> Vec<Tag> {
    let mut out = vec![Tag {
        element,
        tag_type: TagType::Element,
    }];
    // (children, next index) — mirrors the C# iterator stack.
    let mut stack: Vec<(Vec<NodeId>, usize)> = vec![(dom.elements(element, None), 0)];
    while let Some(top) = stack.last_mut() {
        if top.1 < top.0.len() {
            let current = top.0[top.1];
            top.1 += 1;
            if dom.nodes(current).is_empty() {
                out.push(Tag {
                    element: current,
                    tag_type: TagType::EmptyElement,
                });
                continue;
            }
            out.push(Tag {
                element: current,
                tag_type: TagType::Element,
            });
            stack.push((dom.elements(current, None), 0));
            continue;
        }
        stack.pop();
        if let Some(parent_frame) = stack.last() {
            // C#: EndElement for `iteratorStack.Peek().Current` — the element
            // whose children were just exhausted.
            out.push(Tag {
                element: parent_frame.0[parent_frame.1 - 1],
                tag_type: TagType::EndElement,
            });
        }
    }
    out.push(Tag {
        element,
        tag_type: TagType::EndElement,
    });
    out
}

/// `BlockContentInfo` (RevisionProcessor.cs :52): prev/this/next links for
/// block-level content. `iterate_block_content_elements` fills all three
/// (`this` always `Some`); `get_paragraph_info` fills `previous` (= previous
/// element sibling) and `this` only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockContentInfo {
    pub previous_block_content_element: Option<NodeId>,
    pub this_block_content_element: Option<NodeId>,
    pub next_block_content_element: Option<NodeId>,
}

/// First `w:p`/`w:tbl` among `roots`' descendants-and-self, document order.
fn first_block_content(dom: &Dom, roots: &[NodeId]) -> Option<NodeId> {
    let (p, tbl) = (W::p(), W::name("tbl"));
    for &r in roots {
        if let Some(hit) = dom
            .descendants_and_self(r, None)
            .into_iter()
            .find(|&e| dom.name(e).is_some_and(|n| n == p || n == tbl))
        {
            return Some(hit);
        }
    }
    None
}

/// Element siblings after `id`, document order.
fn elements_after_self(dom: &Dom, id: NodeId) -> Vec<NodeId> {
    dom.nodes_after_self(id)
        .into_iter()
        .filter(|&n| dom.is_element(n))
        .collect()
}

/// A.0 — `IterateBlockContentElements` (:1909) + `AnnotateBlockContentElements`
/// (:1855): the doc-order chain of block content (`w:p`/`w:tbl`) under
/// `element`, linked prev/this/next. FAITHFUL: the next-search starts at the
/// current element's FOLLOWING siblings (climbing ancestors up to `element`),
/// so a table's inner paragraphs never appear once the table itself matched.
pub fn iterate_block_content_elements(dom: &Dom, element: NodeId) -> Vec<BlockContentInfo> {
    if dom.elements(element, None).is_empty() {
        return Vec::new();
    }
    let Some(first) = first_block_content(dom, &dom.elements(element, None)) else {
        return Vec::new();
    };

    let mut chain: Vec<NodeId> = vec![first];
    'outer: loop {
        let mut current = *chain.last().unwrap();
        loop {
            if let Some(next) = first_block_content(dom, &elements_after_self(dom, current)) {
                chain.push(next);
                break;
            }
            let Some(parent) = dom.parent(current) else {
                break 'outer;
            };
            current = parent;
            // When we've backed up the tree to the contentContainer, we're done.
            if current == element {
                break 'outer;
            }
        }
    }

    (0..chain.len())
        .map(|i| BlockContentInfo {
            previous_block_content_element: (i > 0).then(|| chain[i - 1]),
            this_block_content_element: Some(chain[i]),
            next_block_content_element: chain.get(i + 1).copied(),
        })
        .collect()
}

/// `W.BlockLevelContentContainers` (PtOpenXmlUtil.cs :5569).
fn block_level_content_containers() -> [XName; 7] {
    [
        W::body(),
        W::name("tc"),
        W::name("txbxContent"),
        W::name("hdr"),
        W::name("ftr"),
        W::name("endnote"),
        W::footnote(),
    ]
}

/// A.0 — `GetParagraphInfo` (:2917) + `InitializeParagraphInfo` (:2890): for a
/// child of a block-level content container, `this` = the first
/// descendant-or-self in {`w:p`, `w:tc`, `w:txbxContent`} — nulled when that
/// first hit is a `tc`/`txbxContent` (means "no own paragraph") — and
/// `previous` = the previous element sibling (content element of any kind).
///
/// # Panics
/// Like the C# `ArgumentException`, panics when the parent is not a
/// block-level content container.
pub fn get_paragraph_info(dom: &Dom, content_element: NodeId) -> BlockContentInfo {
    let parent = dom
        .parent(content_element)
        .expect("GetParagraphInfo called for element without parent");
    let parent_name = dom.name(parent);
    assert!(
        parent_name.is_some_and(|n| block_level_content_containers().contains(&n)),
        "GetParagraphInfo called for element that is not child of content container"
    );

    let (p, tc, txbx) = (W::p(), W::name("tc"), W::name("txbxContent"));
    let mut paragraph = dom
        .descendants_and_self(content_element, None)
        .into_iter()
        .find(|&e| dom.name(e).is_some_and(|n| n == p || n == tc || n == txbx));
    if let Some(hit) = paragraph
        && dom.name(hit).is_some_and(|n| n == tc || n == txbx)
    {
        paragraph = None;
    }

    let previous = dom
        .nodes_before_self(content_element)
        .into_iter()
        .rev()
        .find(|&n| dom.is_element(n));

    BlockContentInfo {
        previous_block_content_element: previous,
        this_block_content_element: paragraph,
        next_block_content_element: None,
    }
}

/// A.0 — `ContentElementsBeforeSelf` (:2926): previous element siblings,
/// nearest first.
pub fn content_elements_before_self(dom: &Dom, element: NodeId) -> Vec<NodeId> {
    dom.nodes_before_self(element)
        .into_iter()
        .rev()
        .filter(|&n| dom.is_element(n))
        .collect()
}

// ───────────────────────────── A.1 — field-code fixup ───────────────────────

/// A.1 — `TransformInstrTextToDelInstrText` (:1433): rename `w:instrText` to
/// `w:delInstrText` (attrs + nodes carried as-is), rebuilding everything else.
fn transform_instr_text_to_del_instr_text(dom: &mut Dom, node: NodeId) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if name == W::name("instrText") {
        let ne = dom.new_element(W::name("delInstrText"));
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        // C# takes element.Nodes() as-is (no recursion below instrText).
        for c in dom.nodes(node) {
            let cc = dom.clone_subtree(c);
            dom.add(ne, cc);
        }
        return ne;
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        let tc = transform_instr_text_to_del_instr_text(dom, c);
        dom.add(ne, tc);
    }
    ne
}

/// A.1 — `FixUpDeletedOrInsertedFieldCodesTransform` (:1354): inside each
/// `w:p`, group-adjacent the child elements by kind — 2 = `w:del` holding
/// `w:r/w:fldChar`, 3 = `w:ins` holding `w:r/w:fldChar`, 4 = `w:r` with
/// `w:instrText`, 1 = other. A key-4 group strictly BETWEEN two key-2 groups
/// is wrapped in a new `w:del` with `instrText` → `delInstrText`; between two
/// key-3 groups it is wrapped in a new `w:ins` (instrText kept). Boundary or
/// mixed-flank groups pass through. FAITHFUL: the paragraph branch iterates
/// child ELEMENTS only (direct text under `w:p` is dropped, as in C#), and
/// the wrapping del/ins carries no author/date attributes.
pub fn fix_up_deleted_or_inserted_field_codes_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if name == W::p() {
        let key_of = |dom: &Dom, e: NodeId| -> u8 {
            let n = dom.name(e).unwrap();
            let holds_fld_char = |d: &Dom| {
                d.elements(e, Some(&W::r()))
                    .into_iter()
                    .any(|r| d.element(r, &W::name("fldChar")).is_some())
            };
            if n == W::del() && holds_fld_char(dom) {
                2
            } else if n == W::ins() && holds_fld_char(dom) {
                3
            } else if n == W::r() && dom.element(e, &W::name("instrText")).is_some() {
                4
            } else {
                1
            }
        };
        let children = dom.elements(node, None);
        let grouped = crate::util::group_adjacent(children, |&e| key_of(dom, e));
        let g_len = grouped.len();

        let new_paragraph = dom.new_element(W::p());
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(new_paragraph, &an, Some(&av));
        }
        for (i, (key, group)) in grouped.iter().enumerate() {
            match key {
                1..=3 => {
                    for &e in group {
                        let t = fix_up_deleted_or_inserted_field_codes_transform(dom, e);
                        dom.add(new_paragraph, t);
                    }
                }
                4 => {
                    let flanked_by = |k: u8| {
                        i != 0 && i != g_len - 1 && grouped[i - 1].0 == k && grouped[i + 1].0 == k
                    };
                    if flanked_by(2) {
                        let del = dom.new_element(W::del());
                        for &e in group {
                            let t = transform_instr_text_to_del_instr_text(dom, e);
                            dom.add(del, t);
                        }
                        dom.add(new_paragraph, del);
                    } else if flanked_by(3) {
                        let ins = dom.new_element(W::ins());
                        for &e in group {
                            let t = fix_up_deleted_or_inserted_field_codes_transform(dom, e);
                            dom.add(ins, t);
                        }
                        dom.add(new_paragraph, ins);
                    } else {
                        for &e in group {
                            let t = fix_up_deleted_or_inserted_field_codes_transform(dom, e);
                            dom.add(new_paragraph, t);
                        }
                    }
                }
                _ => unreachable!("Internal error"),
            }
        }
        return new_paragraph;
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        let tc = fix_up_deleted_or_inserted_field_codes_transform(dom, c);
        dom.add(ne, tc);
    }
    ne
}

// ───────────────────────────── A.2 — moveFrom ranges ────────────────────────

/// A.2 — `AcceptMoveFromRanges` (:1530): walk the tag stream; while one or
/// more `moveFromRangeStart` ids are open, record every other element's open
/// tag (start-list) and close tag (end-list; empty elements land in both).
/// When a `moveFromRangeEnd` MATCHES an open id, that range's records flush
/// into the global lists; an unmatched start never flushes (inert). Elements
/// present in BOTH global lists — i.e. strictly inside a matched range — are
/// deleted via a rebuild; with nothing to delete the input element is
/// returned unchanged (identity, like C#). The range markers themselves are
/// never collected (AcceptAllOtherRevisions strips them later).
pub fn accept_move_from_ranges(dom: &mut Dom, document: NodeId) -> NodeId {
    use std::collections::{HashMap, HashSet};

    let mfrs = W::name("moveFromRangeStart");
    let mfre = W::name("moveFromRangeEnd");

    let mut start_tags_in_range: Vec<NodeId> = Vec::new();
    let mut end_tags_in_range: Vec<NodeId> = Vec::new();
    // id → (potential start tags, potential end tags)
    let mut potential: HashMap<String, (Vec<NodeId>, Vec<NodeId>)> = HashMap::new();

    for tag in descendant_and_self_tags(dom, document) {
        let name = dom.name(tag.element).unwrap();
        if name == mfrs {
            let id = dom
                .attribute(tag.element, &W::id())
                .unwrap_or("")
                .to_string();
            potential.insert(id, (Vec::new(), Vec::new()));
            continue;
        }
        if name == mfre {
            let id = dom.attribute(tag.element, &W::id()).unwrap_or("");
            if let Some((starts, ends)) = potential.remove(id) {
                start_tags_in_range.extend(starts);
                end_tags_in_range.extend(ends);
            }
            continue;
        }
        if potential.is_empty() {
            continue;
        }
        match tag.tag_type {
            TagType::Element => {
                for (starts, _) in potential.values_mut() {
                    starts.push(tag.element);
                }
            }
            TagType::EmptyElement => {
                for (starts, ends) in potential.values_mut() {
                    starts.push(tag.element);
                    ends.push(tag.element);
                }
            }
            TagType::EndElement => {
                for (_, ends) in potential.values_mut() {
                    ends.push(tag.element);
                }
            }
        }
    }

    let end_set: HashSet<NodeId> = end_tags_in_range.into_iter().collect();
    let to_delete: HashSet<NodeId> = start_tags_in_range
        .into_iter()
        .filter(|e| end_set.contains(e))
        .collect();
    if to_delete.is_empty() {
        return document;
    }
    accept_move_from_ranges_transform(dom, document, &to_delete)
        .expect("the document root is never in a moveFrom range")
}

/// A.2 — `AcceptMoveFromRangesTransform` (:2629): rebuild, dropping the
/// elements marked for deletion (and thereby their subtrees).
fn accept_move_from_ranges_transform(
    dom: &mut Dom,
    node: NodeId,
    to_delete: &std::collections::HashSet<NodeId>,
) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(dom.clone_subtree(node));
    }
    if to_delete.contains(&node) {
        return None;
    }
    let ne = dom.new_element(dom.name(node).unwrap());
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        if let Some(tc) = accept_move_from_ranges_transform(dom, c, to_delete) {
            dom.add(ne, tc);
        }
    }
    Some(ne)
}

// ─────────────────────── A.3 — paragraph end tags in moveFrom ───────────────

/// A.3 — `CollapseParagraphTransform` (:1821): a `w:p` collapses to its
/// element children minus `w:pPr` (clones — C# XLinq re-parents by cloning);
/// other elements rebuild recursively; non-elements pass through.
pub fn collapse_paragraph_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    if !dom.is_element(node) {
        return vec![dom.clone_subtree(node)];
    }
    let name = dom.name(node).unwrap();
    if name == W::p() {
        let keep: Vec<NodeId> = dom
            .elements(node, None)
            .into_iter()
            .filter(|&e| dom.name(e) != Some(W::p_pr()))
            .collect();
        return keep.into_iter().map(|e| dom.clone_subtree(e)).collect();
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        for tc in collapse_paragraph_transform(dom, c) {
            dom.add(ne, tc);
        }
    }
    vec![ne]
}

/// A.3 — `CoalesqueParagraphEndTagsInMoveFromTransform` (:2645): produce the
/// first group member, with its (first descendant) paragraph replaced by one
/// carrying the paragraph's own children plus the COLLAPSED content of the
/// group's subsequent members. NOTE: unreachable from the accept transform in
/// practice — see the FAITHFUL-BUG note on
/// [`accept_paragraph_end_tags_in_move_from_transform`].
pub fn coalesque_paragraph_end_tags_in_move_from_transform(
    dom: &mut Dom,
    node: NodeId,
    group: &[NodeId],
) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if name == W::p() {
        let np = dom.new_element(W::p());
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(np, &an, Some(&av));
        }
        for e in dom.elements(node, None) {
            let ce = dom.clone_subtree(e);
            dom.add(np, ce);
        }
        for &member in group.iter().skip(1) {
            for collapsed in collapse_paragraph_transform(dom, member) {
                dom.add(np, collapsed);
            }
        }
        return np;
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        let tc = coalesque_paragraph_end_tags_in_move_from_transform(dom, c, group);
        dom.add(ne, tc);
    }
    ne
}

/// A.3 — `AcceptParagraphEndTagsInMoveFromTransform` (:1610): group a content
/// container's children by whether their paragraph mark sits in an OPEN
/// moveFrom range (`moveFromRangeStart` child without `moveFromRangeEnd`), or
/// they are a `w:p` directly following such a paragraph.
///
/// FAITHFUL-BUG (preserved; TS port identical, RevisionProcessor.ts:1313 with
/// a "needs rewritten" note at :971): the branch condition is inverted — the
/// coalescing path only executes when there is a single all-`Other` group,
/// where it degenerates to a pass-through; whenever an in-range group exists
/// the code takes the plain recursive rebuild instead. Net effect: a deep
/// identity rebuild. Both the TS goldens and the C# RP baselines were
/// generated with this behavior, so we reproduce it rather than "fix" it.
pub fn accept_paragraph_end_tags_in_move_from_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if block_level_content_containers().contains(&name) {
        let mfrs = W::name("moveFromRangeStart");
        let mfre = W::name("moveFromRangeEnd");
        let mark_in_open_range = |dom: &Dom, p: NodeId| {
            !dom.elements(p, Some(&mfrs)).is_empty() && dom.elements(p, Some(&mfre)).is_empty()
        };
        let key_of = |dom: &Dom, c: NodeId| -> bool {
            // true = ParagraphEndTagInMoveFromRange, false = Other
            let pi = get_paragraph_info(dom, c);
            if let Some(this) = pi.this_block_content_element
                && mark_in_open_range(dom, this)
            {
                return true;
            }
            let previous = content_elements_before_self(dom, c).into_iter().find(|&e| {
                get_paragraph_info(dom, e)
                    .this_block_content_element
                    .is_some()
            });
            if let Some(prev) = previous {
                let pi2 = get_paragraph_info(dom, prev);
                if dom.name(c) == Some(W::p())
                    && mark_in_open_range(dom, pi2.this_block_content_element.unwrap())
                {
                    return true;
                }
            }
            false
        };
        let children = dom.elements(node, None);
        let grouped = crate::util::group_adjacent(children, |&c| key_of(dom, c));

        if grouped.len() == 1 && !grouped[0].0 {
            // "Nothing to do": rebuild the container with its children cloned
            // as-is (C# attaches the original elements to a new parent, which
            // XLinq clones; descendants are NOT re-transformed here).
            let ne = dom.new_element(name);
            for (an, av) in dom.attributes(node) {
                dom.set_attribute_value(ne, &an, Some(&av));
            }
            for (key, group) in grouped {
                if !key {
                    for e in group {
                        let ce = dom.clone_subtree(e);
                        dom.add(ne, ce);
                    }
                } else {
                    // Unreachable given the branch guard; kept for shape
                    // parity with the C# select.
                    let first = group[0];
                    let t = coalesque_paragraph_end_tags_in_move_from_transform(dom, first, &group);
                    dom.add(ne, t);
                }
            }
            return ne;
        }
        // FAITHFUL-BUG: in-range groups exist → plain recursive rebuild, no
        // coalescing.
        let ne = dom.new_element(name);
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for c in dom.nodes(node) {
            let tc = accept_paragraph_end_tags_in_move_from_transform(dom, c);
            dom.add(ne, tc);
        }
        return ne;
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        let tc = accept_paragraph_end_tags_in_move_from_transform(dom, c);
        dom.add(ne, tc);
    }
    ne
}

// ─────────────── A.4 — deleted / moved-from content controls ────────────────

/// A.4 — `AcceptDeletedAndMovedFromContentControls` (:2491): walk the tag
/// stream tracking two range kinds. `customXmlDelRange` collects ONLY `w:sdt`
/// tags; `customXmlMoveFromRange` collects every element (the `w:sdt` block
/// feeds both trackers). An `w:sdt` strictly inside a matched del range is
/// COLLAPSED to its `sdtContent` children; any element strictly inside a
/// matched moveFrom range is DELETED. Unmatched starts never flush; with
/// nothing collected the input element is returned unchanged (identity).
pub fn accept_deleted_and_moved_from_content_controls(dom: &mut Dom, root: NodeId) -> NodeId {
    use std::collections::{HashMap, HashSet};

    let cxdel_s = W::name("customXmlDelRangeStart");
    let cxdel_e = W::name("customXmlDelRangeEnd");
    let cxmf_s = W::name("customXmlMoveFromRangeStart");
    let cxmf_e = W::name("customXmlMoveFromRangeEnd");
    let mfrs = W::name("moveFromRangeStart");
    let mfre = W::name("moveFromRangeEnd");
    let sdt = W::name("sdt");

    let mut del_starts: Vec<NodeId> = Vec::new();
    let mut del_ends: Vec<NodeId> = Vec::new();
    let mut mf_starts: Vec<NodeId> = Vec::new();
    let mut mf_ends: Vec<NodeId> = Vec::new();
    let mut potential_del: HashMap<String, (Vec<NodeId>, Vec<NodeId>)> = HashMap::new();
    let mut potential_mf: HashMap<String, (Vec<NodeId>, Vec<NodeId>)> = HashMap::new();

    for tag in descendant_and_self_tags(dom, root) {
        let name = dom.name(tag.element).unwrap();
        if name == cxdel_s {
            let id = dom
                .attribute(tag.element, &W::id())
                .unwrap_or("")
                .to_string();
            potential_del.insert(id, (Vec::new(), Vec::new()));
            continue;
        }
        if name == cxdel_e {
            let id = dom.attribute(tag.element, &W::id()).unwrap_or("");
            if let Some((starts, ends)) = potential_del.remove(id) {
                del_starts.extend(starts);
                del_ends.extend(ends);
            }
            continue;
        }
        if name == cxmf_s {
            let id = dom
                .attribute(tag.element, &W::id())
                .unwrap_or("")
                .to_string();
            potential_mf.insert(id, (Vec::new(), Vec::new()));
            continue;
        }
        if name == cxmf_e {
            let id = dom.attribute(tag.element, &W::id()).unwrap_or("");
            if let Some((starts, ends)) = potential_mf.remove(id) {
                mf_starts.extend(starts);
                mf_ends.extend(ends);
            }
            continue;
        }
        if name == sdt {
            match tag.tag_type {
                TagType::Element => {
                    for (starts, _) in potential_del.values_mut().chain(potential_mf.values_mut()) {
                        starts.push(tag.element);
                    }
                }
                TagType::EmptyElement => {
                    for (starts, ends) in
                        potential_del.values_mut().chain(potential_mf.values_mut())
                    {
                        starts.push(tag.element);
                        ends.push(tag.element);
                    }
                }
                TagType::EndElement => {
                    for (_, ends) in potential_del.values_mut().chain(potential_mf.values_mut()) {
                        ends.push(tag.element);
                    }
                }
            }
            continue;
        }
        if !potential_mf.is_empty() && name != mfrs && name != mfre {
            match tag.tag_type {
                TagType::Element => {
                    for (starts, _) in potential_mf.values_mut() {
                        starts.push(tag.element);
                    }
                }
                TagType::EmptyElement => {
                    for (starts, ends) in potential_mf.values_mut() {
                        starts.push(tag.element);
                        ends.push(tag.element);
                    }
                }
                TagType::EndElement => {
                    for (_, ends) in potential_mf.values_mut() {
                        ends.push(tag.element);
                    }
                }
            }
        }
    }

    let del_end_set: HashSet<NodeId> = del_ends.into_iter().collect();
    let to_collapse: HashSet<NodeId> = del_starts
        .into_iter()
        .filter(|e| del_end_set.contains(e))
        .collect();
    let mf_end_set: HashSet<NodeId> = mf_ends.into_iter().collect();
    let to_delete: HashSet<NodeId> = mf_starts
        .into_iter()
        .filter(|e| mf_end_set.contains(e))
        .collect();

    if to_collapse.is_empty() && to_delete.is_empty() {
        return root;
    }
    let out: Vec<NodeId> = accept_deleted_and_moved_from_content_controls_transform(
        dom,
        root,
        &to_collapse,
        &to_delete,
    );
    debug_assert_eq!(out.len(), 1, "the root is neither collapsed nor deleted");
    out[0]
}

/// A.4 — `AcceptDeletedAndMovedFromContentControlsTransform` (:2468): splice
/// a collapsed sdt's `sdtContent` child nodes (transformed) in its place,
/// drop deleted elements, rebuild the rest.
fn accept_deleted_and_moved_from_content_controls_transform(
    dom: &mut Dom,
    node: NodeId,
    to_collapse: &std::collections::HashSet<NodeId>,
    to_delete: &std::collections::HashSet<NodeId>,
) -> Vec<NodeId> {
    if !dom.is_element(node) {
        return vec![dom.clone_subtree(node)];
    }
    let name = dom.name(node).unwrap();
    if name == W::name("sdt") && to_collapse.contains(&node) {
        let content = dom
            .element(node, &W::name("sdtContent"))
            .expect("collapsed w:sdt must carry sdtContent (C# NREs otherwise)");
        let mut out = Vec::new();
        for c in dom.nodes(content) {
            out.extend(accept_deleted_and_moved_from_content_controls_transform(
                dom,
                c,
                to_collapse,
                to_delete,
            ));
        }
        return out;
    }
    if to_delete.contains(&node) {
        return vec![];
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        for tc in
            accept_deleted_and_moved_from_content_controls_transform(dom, c, to_collapse, to_delete)
        {
            dom.add(ne, tc);
        }
    }
    vec![ne]
}

// ─────────────── A.5a — deleted / moved-from paragraph marks ────────────────

/// A.5a — `IsRunContent` (:2356): `Some(true)` = run-level content,
/// `Some(false)` = marker/non-content, `None` = unknown (C# throws).
fn is_run_content(name: &XName) -> Option<bool> {
    if name.namespace_name() == crate::namespaces::M::URI {
        return Some(true);
    }
    if name.namespace_name() != W::URI {
        return None;
    }
    match name.local_name() {
        "r" | "fldSimple" | "hyperlink" | "subDoc" | "smartTag" | "smartTagPr" => Some(true),
        "bookmarkStart"
        | "bookmarkEnd"
        | "commentRangeStart"
        | "commentRangeEnd"
        | "customXmlDelRangeStart"
        | "customXmlDelRangeEnd"
        | "customXmlInsRangeStart"
        | "customXmlInsRangeEnd"
        | "customXmlMoveFromRangeStart"
        | "customXmlMoveFromRangeEnd"
        | "customXmlMoveToRangeStart"
        | "customXmlMoveToRangeEnd"
        | "del"
        | "moveFrom"
        | "moveFromRangeStart"
        | "moveFromRangeEnd"
        | "moveToRangeStart"
        | "moveToRangeEnd"
        | "permStart"
        | "permEnd"
        | "proofErr" => Some(false),
        _ => None,
    }
}

/// A.5a — `CollapseTransform` (:2331): splice `w:dir`/`w:bdr`/`w:ins`/
/// `w:moveTo`/`w:smartTag` to their element children and `w:sdt` to its
/// `sdtContent` element children — ONE level, un-recursed, exactly like the
/// C# (`return element.Elements()`); drop `w:pPr`; rebuild everything else
/// recursively. (Yes, `w:bdr` — the C# comment says `bdo` but the code tests
/// `W.bdr`; reproduced as written.)
fn collapse_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    if !dom.is_element(node) {
        return vec![dom.clone_subtree(node)];
    }
    let name = dom.name(node).unwrap();
    if name.namespace_name() == W::URI
        && matches!(
            name.local_name(),
            "dir" | "bdr" | "ins" | "moveTo" | "smartTag"
        )
    {
        let kids = dom.elements(node, None);
        return kids.into_iter().map(|e| dom.clone_subtree(e)).collect();
    }
    if name == W::name("sdt") {
        let mut out = Vec::new();
        for sc in dom.elements(node, Some(&W::name("sdtContent"))) {
            for e in dom.elements(sc, None) {
                out.push(dom.clone_subtree(e));
            }
        }
        return out;
    }
    if name == W::p_pr() {
        return vec![];
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        for tc in collapse_transform(dom, c) {
            dom.add(ne, tc);
        }
    }
    vec![ne]
}

/// A.5a — `AllParaContentIsDeleted` (:2310): after collapsing wrappers, does
/// the paragraph hold NO run-level content?
///
/// # Panics
/// Like the C# ("Internal error 20"), on a child element `IsRunContent`
/// cannot classify.
fn all_para_content_is_deleted(dom: &mut Dom, p: NodeId) -> bool {
    let collapsed = collapse_transform(dom, p);
    debug_assert_eq!(collapsed.len(), 1, "w:p rebuilds to a single element");
    let test_p = collapsed[0];
    !dom.elements(test_p, None).into_iter().any(|ce| {
        let n = dom.name(ce).unwrap();
        is_run_content(&n)
            .unwrap_or_else(|| panic!("Internal error 20, found element {}", n.clark()))
    })
}

/// `p / pPr / rPr / (del | moveFrom)` — is the paragraph mark deleted or
/// moved from?
fn paragraph_mark_is_deleted_or_moved_from(dom: &Dom, p: NodeId) -> bool {
    // At most one w:pPr / w:rPr; use singular element lookups to avoid Vec churn.
    dom.element(p, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| {
            dom.elements(rpr, None).into_iter().any(|e| {
                dom.name(e)
                    .is_some_and(|n| n == W::del() || n == W::name("moveFrom"))
            })
        })
}

/// A.5a — `AcceptDeletedAndMoveFromParagraphMarksTransform` (:2119): the
/// 3-state grouping machine over a container's block-content chain. A run of
/// deleted-mark paragraphs PLUS the immediately following normal paragraph
/// form one DeletedRange group, which merges into a single paragraph carrying
/// `g.Last()`'s pPr (:2271, the RP052 fix) and every member's collapsed
/// content; the merged paragraph is nuked when its content is entirely
/// deleted, its last member's mark is `w:del`, and it is the container's last
/// block content (or a table follows) (:2276). Tables (and m:* block content)
/// bound groups and reset the state. FAITHFUL: the container rebuild keeps
/// only `w:tcPr` children + the chain elements + the body-level `sectPr`
/// (re-appended last); other non-chain children are dropped, and merged
/// paragraphs lose the original `w:p` attributes — exactly like the C#.
pub fn accept_deleted_and_move_from_paragraph_marks_transform(
    dom: &mut Dom,
    node: NodeId,
) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if !block_level_content_containers().contains(&name) {
        let ne = dom.new_element(name);
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for c in dom.nodes(node) {
            let tc = accept_deleted_and_move_from_paragraph_marks_transform(dom, c);
            dom.add(ne, tc);
        }
        return ne;
    }

    let body_sect_pr = if name == W::body() {
        dom.element(node, &W::name("sectPr"))
    } else {
        None
    };

    let chain = iterate_block_content_elements(dom, node);
    // (is_deleted_range, grouping_key) aligned with the chain.
    let mut infos: Vec<(bool, i32)> = Vec::with_capacity(chain.len());
    let mut current_key = 0i32;
    let mut state = 0u8; // 0 = non-deleted, 1 = in deleted, 2 = paragraph following
    for c in &chain {
        let this = c.this_block_content_element.unwrap();
        let tn = dom.name(this).unwrap();
        if tn == W::p() {
            if paragraph_mark_is_deleted_or_moved_from(dom, this) {
                match state {
                    0 | 2 => {
                        state = 1;
                        current_key += 1;
                        infos.push((true, current_key));
                    }
                    _ => infos.push((true, current_key)),
                }
            } else {
                match state {
                    0 => {
                        current_key += 1;
                        infos.push((false, current_key));
                    }
                    1 => {
                        // the paragraph following a deleted run JOINS its group
                        state = 2;
                        infos.push((true, current_key));
                    }
                    _ => {
                        state = 0;
                        current_key += 1;
                        infos.push((false, current_key));
                    }
                }
            }
        } else if tn == W::name("tbl") || tn.namespace_name() == M::URI {
            current_key += 1;
            infos.push((false, current_key));
            state = 0;
        } else {
            // defensive parity with C#: keep state and key (chain only ever
            // yields w:p / w:tbl, so this arm is unreachable in practice)
            infos.push((false, current_key));
        }
    }

    let zipped: Vec<(BlockContentInfo, (bool, i32))> = chain.into_iter().zip(infos).collect();
    let grouped = crate::util::group_adjacent(zipped, |z| z.1.1);

    let ne = dom.new_element(name.clone());
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for e in dom.elements(node, Some(&W::name("tcPr"))) {
        let ce = dom.clone_subtree(e);
        dom.add(ne, ce);
    }
    for (_key, group) in grouped {
        if group[0].1.0 {
            // DeletedRange: merge into one paragraph.
            let last_this = group.last().unwrap().0.this_block_content_element.unwrap();
            let np = dom.new_element(W::p());
            for ppr in dom.elements(last_this, Some(&W::p_pr())) {
                let c = dom.clone_subtree(ppr);
                dom.add(np, c);
            }
            for z in &group {
                let this = z.0.this_block_content_element.unwrap();
                for collapsed in collapse_paragraph_transform(dom, this) {
                    dom.add(np, collapsed);
                }
            }
            let last_mark_is_del = dom
                .element(last_this, &W::p_pr())
                .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                .is_some_and(|rpr| dom.element(rpr, &W::del()).is_some());
            let next = group.last().unwrap().0.next_block_content_element;
            let next_is_none_or_tbl =
                next.is_none() || next.is_some_and(|n| dom.name(n) == Some(W::name("tbl")));
            if all_para_content_is_deleted(dom, np) && last_mark_is_del && next_is_none_or_tbl {
                continue; // nuke: never attached
            }
            dom.add(ne, np);
        } else {
            for z in &group {
                let this = z.0.this_block_content_element.unwrap();
                let rebuilt_name = dom.name(this).unwrap();
                let re = dom.new_element(rebuilt_name);
                for (an, av) in dom.attributes(this) {
                    dom.set_attribute_value(re, &an, Some(&av));
                }
                for c in dom.nodes(this) {
                    let tc = accept_deleted_and_move_from_paragraph_marks_transform(dom, c);
                    dom.add(re, tc);
                }
                dom.add(ne, re);
            }
        }
    }
    if let Some(sp) = body_sect_pr {
        let c = dom.clone_subtree(sp);
        dom.add(ne, c);
    }
    ne
}

// ─────────────── A.5b — content-control re-wrap after mark merge ────────────

/// A.5b — `AnnotateRunElementsWithId` (:1935): number every descendant `w:r`
/// with `pt:UniqueId` 0.. in document order (in place).
pub fn annotate_run_elements_with_id(dom: &mut Dom, element: NodeId) {
    let unique_id = PT::name("UniqueId");
    for (run_id, r) in (0..).zip(dom.descendants(element, Some(&W::r()))) {
        dom.set_attribute_value(r, &unique_id, Some(&run_id.to_string()));
    }
}

/// Descendants of `element` in document order, not descending INTO elements
/// named `trim` (the `DescendantsTrimmed` helper the annotators use).
fn descendants_trimmed(dom: &Dom, element: NodeId, trim: &XName) -> Vec<NodeId> {
    let mut out = Vec::new();
    let mut stack: Vec<NodeId> = dom.elements(element, None).into_iter().rev().collect();
    while let Some(e) = stack.pop() {
        out.push(e);
        if dom.name(e).as_ref() != Some(trim) {
            for c in dom.elements(e, None).into_iter().rev() {
                stack.push(c);
            }
        }
    }
    out
}

/// A.5b — `AnnotateContentControlsWithRunIds` (:1945): give every descendant
/// `w:sdt` a `pt:RunIds` (comma-joined `pt:UniqueId`s of its runs, trimmed at
/// `w:txbxContent`) and its own `pt:UniqueId` (in place).
pub fn annotate_content_controls_with_run_ids(dom: &mut Dom, element: NodeId) {
    let unique_id = PT::name("UniqueId");
    let run_ids_name = PT::name("RunIds");
    let txbx = W::name("txbxContent");
    for (sdt_id, e) in (0..).zip(dom.descendants(element, Some(&W::name("sdt")))) {
        let ids: Vec<String> = descendants_trimmed(dom, e, &txbx)
            .into_iter()
            .filter(|&d2| dom.name(d2) == Some(W::r()))
            .filter_map(|r| dom.attribute(r, &unique_id).map(str::to_string))
            .collect();
        dom.set_attribute_value(e, &run_ids_name, Some(&ids.join(",")));
        dom.set_attribute_value(e, &unique_id, Some(&sdt_id.to_string()));
    }
}

/// `Order_sdt` (:2090): schema order for rebuilt `w:sdt` children.
fn order_sdt(name: &XName) -> i32 {
    if name.namespace_name() != W::URI {
        return 999;
    }
    match name.local_name() {
        "sdtPr" => 10,
        "sdtEndPr" => 20,
        "sdtContent" => 30,
        "bookmarkStart" => 40,
        "bookmarkEnd" => 50,
        _ => 999,
    }
}

/// A.5b — `AddBlockLevelContentControls` (:1964): re-create the `w:sdt`
/// wrappers the paragraph-mark transform stripped. For every original sdt
/// (by `pt:UniqueId`) missing from `new_document`, locate its runs (by
/// `pt:RunIds`) in the new document, find their deepest common ancestor, and
/// either wrap the whole paragraph or wrap the child range in a rebuilt sdt
/// (children in `Order_sdt` order). Mutates `new_document` in place.
///
/// FAITHFUL-BUG (TS identical): in the whole-paragraph branch the C# orders
/// `contentControl.Elements()` — the ORIGINAL sdt's elements, including its
/// original `sdtContent` — instead of the freshly-built control, so the
/// replacement is a clone of the original sdt (pre-transform content).
///
/// # Panics
/// Like the C# (`.First()`), when an annotated run of a missing sdt no longer
/// exists in `new_document`.
pub fn add_block_level_content_controls(
    dom: &mut Dom,
    new_document: NodeId,
    original: NodeId,
) -> NodeId {
    use std::collections::HashSet;

    let sdt = W::name("sdt");
    let unique_id = PT::name("UniqueId");
    let run_ids_name = PT::name("RunIds");

    let original_ccs = dom.descendants(original, Some(&sdt));
    let existing_ids: HashSet<String> = dom
        .descendants(new_document, Some(&sdt))
        .into_iter()
        .filter_map(|e| dom.attribute(e, &unique_id).map(str::to_string))
        .collect();

    // Index new-document runs once (O(1) lookup). The *run* nodes and their
    // pt:UniqueId attrs are stable for the duration of re-wrap; new w:sdt
    // wrappers may be attached under new_document later in the loop, but that
    // does not invalidate this run index. First occurrence wins for duplicate
    // UniqueIds — matches the old `.find()` document-order semantics.
    let mut run_by_id: HashMap<String, NodeId> = HashMap::new();
    for r in dom.descendants(new_document, Some(&W::r())) {
        if let Some(id) = dom.attribute(r, &unique_id) {
            run_by_id.entry(id.to_string()).or_insert(r);
        }
    }

    for cc in original_ccs {
        let cc_id = dom.attribute(cc, &unique_id).unwrap_or("").to_string();
        if existing_ids.contains(&cc_id) {
            continue;
        }
        let run_ids: Vec<String> = dom
            .attribute(cc, &run_ids_name)
            .unwrap_or("")
            .split(',')
            .map(str::to_string)
            .collect();
        let runs: Vec<String> = dom
            .descendants(cc, Some(&W::r()))
            .into_iter()
            .filter_map(|r| dom.attribute(r, &unique_id).map(str::to_string))
            .filter(|id| run_ids.contains(id))
            .collect();
        // O(1) index via run_by_id. Runs whose content was entirely a deleted
        // revision no longer exist after acceptance — filter_map skips them
        // and empty controls fall through to `continue` below (upstream C#
        // used .First() and crashed on fully-deleted sdt).
        let runs_in_new_document: Vec<NodeId> = runs
            .iter()
            .filter_map(|id| run_by_id.get(id).copied())
            .collect();

        // deepest common ancestor of all the runs (nearest-first intersection)
        let Some(first_run) = runs_in_new_document.first().copied() else {
            continue;
        };
        let mut intersection: Vec<NodeId> = dom.ancestors(first_run, None);
        for &run in &runs_in_new_document[1..] {
            let anc: HashSet<NodeId> = dom.ancestors(run, None).into_iter().collect();
            intersection.retain(|a| anc.contains(a));
        }
        let Some(&common_ancestor) = intersection.first() else {
            continue;
        };

        let child_containing = |dom: &Dom, run: NodeId| -> NodeId {
            dom.elements(common_ancestor, None)
                .into_iter()
                .find(|&c| {
                    dom.descendants_and_self(c, Some(&W::r()))
                        .into_iter()
                        .any(|z| z == run)
                })
                .expect("common ancestor child containing the run")
        };
        let first_run_child = child_containing(dom, *runs_in_new_document.first().unwrap());
        let last_run_child = child_containing(dom, *runs_in_new_document.last().unwrap());

        // Children that "count" for the whole-paragraph test.
        let significant: Vec<NodeId> = dom
            .elements(common_ancestor, None)
            .into_iter()
            .filter(|&e| {
                let n = dom.name(e).unwrap();
                n != W::p_pr()
                    && n != W::name("commentRangeStart")
                    && n != W::name("commentRangeEnd")
            })
            .collect();
        if dom.name(common_ancestor) == Some(W::p())
            && significant.first() == Some(&first_run_child)
            && significant.last() == Some(&last_run_child)
        {
            // Whole-paragraph wrap. FAITHFUL-BUG: the replacement is built
            // from the ORIGINAL content control's elements (clone-on-attach),
            // ordered by Order_sdt — not from a control holding the merged
            // paragraph.
            let new_cc = dom.new_element(dom.name(cc).unwrap());
            for (an, av) in dom.attributes(cc) {
                dom.set_attribute_value(new_cc, &an, Some(&av));
            }
            let mut cc_children = dom.elements(cc, None);
            cc_children.sort_by_key(|&e| order_sdt(&dom.name(e).unwrap()));
            for e in cc_children {
                let clone = dom.clone_subtree(e);
                dom.add(new_cc, clone);
            }
            dom.add_before_self(common_ancestor, new_cc);
            dom.remove(common_ancestor);
            continue;
        }

        // Range wrap: children before / in / after the run-child range.
        let children = dom.elements(common_ancestor, None);
        let first_idx = children.iter().position(|&c| c == first_run_child).unwrap();
        let last_idx = children.iter().position(|&c| c == last_run_child).unwrap();
        let before: Vec<NodeId> = children[..first_idx].to_vec();
        let in_range: Vec<NodeId> = children[first_idx..=last_idx].to_vec();
        let after: Vec<NodeId> = children[last_idx + 1..].to_vec();

        for &c in &children {
            dom.remove(c);
        }
        let new_cc = dom.new_element(dom.name(cc).unwrap());
        for (an, av) in dom.attributes(cc) {
            dom.set_attribute_value(new_cc, &an, Some(&av));
        }
        let sdt_content = dom.new_element(W::name("sdtContent"));
        for &e in &in_range {
            dom.add(sdt_content, e); // detached → moved, like the C#
        }
        let cc_props: Vec<NodeId> = dom
            .elements(cc, None)
            .into_iter()
            .filter(|&e| dom.name(e) != Some(W::name("sdtContent")))
            .collect();
        let mut cc_kids: Vec<NodeId> = cc_props.into_iter().map(|e| dom.clone_subtree(e)).collect();
        cc_kids.push(sdt_content);
        cc_kids.sort_by_key(|&e| order_sdt(&dom.name(e).unwrap()));
        for e in cc_kids {
            dom.add(new_cc, e);
        }
        for &e in &before {
            dom.add(common_ancestor, e);
        }
        dom.add(common_ancestor, new_cc);
        for &e in &after {
            dom.add(common_ancestor, e);
        }
    }
    new_document
}

/// A.5b — `AcceptDeletedAndMoveFromParagraphMarks` (:2098): annotate runs and
/// content controls (on the ORIGINAL, in place), run the A.5a transform, then
/// re-wrap the content controls the transform stripped.
pub fn accept_deleted_and_move_from_paragraph_marks(dom: &mut Dom, element: NodeId) -> NodeId {
    annotate_run_elements_with_id(dom, element);
    annotate_content_controls_with_run_ids(dom, element);
    let new_element = accept_deleted_and_move_from_paragraph_marks_transform(dom, element);
    add_block_level_content_controls(dom, new_element, element)
}

// ─────────────── A.6 — rows left empty by moveFrom ──────────────────────────

/// `BlockLevelElements` (:2766) — the direct-cell-child names that make a
/// cell non-empty for [`remove_rows_left_empty_by_move_from`].
fn a6_block_level_elements() -> [XName; 8] {
    [
        W::p(),
        W::name("tbl"),
        W::name("sdt"),
        W::del(),
        W::ins(),
        M::name("oMath"),
        M::name("oMathPara"),
        W::name("moveTo"),
    ]
}

/// A.6 — `RemoveRowsLeftEmptyByMoveFrom` (:2777): drop every `w:tr` whose
/// cells all lost their block-level content to an accepted moveFrom; rebuild
/// everything else. The pipeline gates this on the input having carried
/// `w:moveFrom` (captured BEFORE AcceptMoveFromMoveTo consumes the markers).
pub fn remove_rows_left_empty_by_move_from(dom: &mut Dom, node: NodeId) -> NodeId {
    remove_rows_left_empty_by_move_from_inner(dom, node).expect("the root element is not a w:tr")
}

fn remove_rows_left_empty_by_move_from_inner(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(dom.clone_subtree(node));
    }
    let name = dom.name(node).unwrap();
    if name == W::name("tr") {
        let block = a6_block_level_elements();
        let non_empty_cells = dom
            .elements(node, Some(&W::name("tc")))
            .into_iter()
            .any(|tc| {
                dom.elements(tc, None)
                    .into_iter()
                    .any(|tcc| dom.name(tcc).is_some_and(|n| block.contains(&n)))
            });
        if !non_empty_cells {
            return None;
        }
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        if let Some(tc) = remove_rows_left_empty_by_move_from_inner(dom, c) {
            dom.add(ne, tc);
        }
    }
    Some(ne)
}

// ─────────────── A.7 — deleted cells + tcPr order ───────────────────────────

/// `Order_tcPr` (:1466): schema order for rebuilt `w:tcPr` children.
fn order_tc_pr(name: &XName) -> i32 {
    if name.namespace_name() != W::URI {
        return 999;
    }
    match name.local_name() {
        "cnfStyle" => 10,
        "tcW" => 20,
        "gridSpan" => 30,
        "hMerge" => 40,
        "vMerge" => 50,
        "tcBorders" => 60,
        "shd" => 70,
        "noWrap" => 80,
        "tcMar" => 90,
        "textDirection" => 100,
        "tcFitText" => 110,
        "vAlign" => 120,
        "hideMark" => 130,
        "headers" => 140,
        _ => 999,
    }
}

/// A.7 — `AcceptDeletedCellsTransform` (:2674): inside each `w:tr`, group
/// adjacent children by (deleted-cell?, anchor), where a cell is "deleted"
/// when it carries `w:cellDel` OR the next `w:tc` after it does, and the
/// anchor is the nearest preceding (or self) `w:tc` WITHOUT `cellDel`. A
/// group led by its anchor collapses to one cell whose `gridSpan` widens by
/// the number of absorbed cells, `tcPr` children re-ordered per Order_tcPr;
/// a group starting with a deleted cell (no anchor) is dropped. FAITHFUL:
/// the rebuilt cell loses the original `w:tc` attributes, and an anchor cell
/// without `w:tcPr` panics (C# NREs on `currentTcPr.Elements()`).
pub fn accept_deleted_cells_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if name != W::name("tr") {
        let ne = dom.new_element(name);
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for c in dom.nodes(node) {
            let tc = accept_deleted_cells_transform(dom, c);
            dom.add(ne, tc);
        }
        return ne;
    }

    let tc_name = W::name("tc");
    let cell_del = W::name("cellDel");
    let has_cell_del = |dom: &Dom, e: NodeId| !dom.descendants(e, Some(&cell_del)).is_empty();

    let children = dom.elements(node, None);
    // key: (is_deleted_cell_group, disambiguator)
    let key_of = |dom: &Dom, e: NodeId| -> (bool, Option<NodeId>) {
        let cell_after = dom
            .nodes_after_self(e)
            .into_iter()
            .find(|&s| dom.name(s) == Some(tc_name.clone()));
        let cell_after_is_deleted = cell_after.is_some_and(|ca| has_cell_del(dom, ca));
        if dom.name(e) == Some(tc_name.clone()) && (cell_after_is_deleted || has_cell_del(dom, e)) {
            let anchor = std::iter::once(e)
                .chain(content_elements_before_self(dom, e))
                .find(|&z| dom.name(z) == Some(tc_name.clone()) && !has_cell_del(dom, z));
            return (true, anchor);
        }
        (false, Some(e))
    };
    let grouped = crate::util::group_adjacent(children, |&e| key_of(dom, e));

    let tr = dom.new_element(W::name("tr"));
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(tr, &an, Some(&av));
    }
    for ((is_deleted, _anchor), group) in grouped {
        if !is_deleted {
            for e in group {
                let c = dom.clone_subtree(e);
                dom.add(tr, c);
            }
            continue;
        }
        let first = group[0];
        if has_cell_del(dom, first) {
            continue; // no anchor precedes: the whole group is dropped
        }
        let tcpr_name = W::name("tcPr");
        let grid_span_name = W::name("gridSpan");
        // Anchor cells always carry tcPr in the C# path (NRE otherwise); unwrap once.
        let current_tc_pr = dom
            .element(first, &tcpr_name)
            .expect("anchor cell must carry w:tcPr (C# NREs on currentTcPr.Elements())");
        let grid_span: i32 = dom
            .element(current_tc_pr, &grid_span_name)
            .and_then(|g| dom.attribute(g, &W::val()))
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let new_grid_span = grid_span + group.len() as i32 - 1;

        let gs = dom.new_element(grid_span_name.clone());
        dom.set_attribute_value(gs, &W::val(), Some(&new_grid_span.to_string()));
        let mut tcpr_kids: Vec<NodeId> = vec![gs];
        let rest: Vec<NodeId> = dom
            .elements(current_tc_pr, None)
            .into_iter()
            .filter(|&e| dom.name(e) != Some(grid_span_name.clone()))
            .collect();
        for e in rest {
            tcpr_kids.push(dom.clone_subtree(e));
        }
        tcpr_kids.sort_by_key(|&e| order_tc_pr(&dom.name(e).unwrap()));

        let ordered_tc_pr = dom.new_element(tcpr_name.clone());
        for e in tcpr_kids {
            dom.add(ordered_tc_pr, e);
        }
        let new_tc = dom.new_element(tc_name.clone());
        dom.add(new_tc, ordered_tc_pr);
        let body_kids: Vec<NodeId> = dom
            .elements(first, None)
            .into_iter()
            .filter(|&e| dom.name(e) != Some(tcpr_name.clone()))
            .collect();
        for e in body_kids {
            let c = dom.clone_subtree(e);
            dom.add(new_tc, c);
        }
        dom.add(tr, new_tc);
    }
    tr
}

// ─────────────── A.8 — merge adjacent tables ────────────────────────────────

/// A.8 — `FixWidths` (:1484): clone the table and rewrite each `w:tcW`'s
/// `w:w` to the sum of the grid columns its cell spans (per the ORIGINAL
/// table's `tblGrid`). FAITHFUL: cells without a `tcW` do not advance the
/// grid cursor, exactly like the C#.
fn fix_widths(dom: &mut Dom, tbl: NodeId) -> NodeId {
    let grid_lines: Vec<i64> = dom
        .elements(tbl, Some(&W::name("tblGrid")))
        .into_iter()
        .flat_map(|g| dom.elements(g, Some(&W::name("gridCol"))))
        .map(|gc| {
            dom.attribute(gc, &W::name("w"))
                .and_then(|v| v.parse().ok())
                .expect("gridCol w:w must be an integer (C# casts)")
        })
        .collect();
    let new_tbl = dom.clone_subtree(tbl);
    for tr in dom.elements(new_tbl, Some(&W::name("tr"))) {
        let mut last_used: i64 = -1;
        for tc in dom.elements(tr, Some(&W::name("tc"))) {
            // Singular tcPr / tcW / gridSpan — avoid multi-Vec flat_map scans.
            let tc_w = dom
                .element(tc, &W::name("tcPr"))
                .and_then(|p| dom.element(p, &W::name("tcW")))
                .filter(|&w| dom.attribute(w, &W::name("w")).is_some());
            let Some(tc_w) = tc_w else { continue };
            let grid_span: i64 = dom
                .element(tc, &W::name("tcPr"))
                .and_then(|p| dom.element(p, &W::name("gridSpan")))
                .and_then(|g| dom.attribute(g, &W::val()))
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            let z = std::cmp::min(grid_lines.len() as i64 - 1, last_used + grid_span);
            let w: i64 = grid_lines
                .iter()
                .enumerate()
                .filter(|(i, _)| (*i as i64) > last_used && (*i as i64) <= z)
                .map(|(_, g)| g)
                .sum();
            dom.set_attribute_value(tc_w, &W::name("w"), Some(&w.to_string()));
            last_used += grid_span;
        }
    }
    new_tbl
}

/// True when a table subtree carries row/cell/run revision marks. Used to
/// gate adjacent-table merge so clean content tables stay separate.
fn table_has_revision_marks(dom: &Dom, tbl: NodeId) -> bool {
    for tag in ["ins", "del", "moveFrom", "moveTo", "cellIns", "cellDel"] {
        if !dom.descendants(tbl, Some(&W::name(tag))).is_empty() {
            return true;
        }
    }
    false
}

/// A.8 — `MergeAdjacentTablesTransform` (:464): where an element has direct
/// `w:tbl` children, merge each run of ≥2 adjacent tables sharing the same
/// bidiVisual-kind into one table: `tblPr` from the first, `tblGrid` = the
/// diffs of the union of every member's cumulative grid widths, and each
/// member's rows re-fit (`FixWidths`) with cells re-spanned over the finer
/// grid (`gridSpan`, `Order_tcPr` order).
///
/// M112 (file_130 Word parity): C#/PowerTools fires on ANY adjacent tables.
/// Word Compare does **not** merge clean (no-revision) adjacent tables —
/// file_131's metadata tables stay as two tables (1-col + 2-col). Merging
/// them into one 2-col 12-row table shifts LO page geometry and costs ~1–2
/// score points on large-doc near-90 pairs. Gate: only merge a group when
/// at least one member carries revision marks (ins/del/move/cellIns/cellDel).
pub fn merge_adjacent_tables_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    let tbl_name = W::name("tbl");
    if dom.element(node, &tbl_name).is_none() {
        let ne = dom.new_element(name);
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for c in dom.nodes(node) {
            let tc = merge_adjacent_tables_transform(dom, c);
            dom.add(ne, tc);
        }
        return ne;
    }

    let children = dom.elements(node, None);
    let grouped = crate::util::group_adjacent(children, |&e| {
        if dom.name(e) != Some(tbl_name.clone()) {
            return String::new();
        }
        let bidi = dom
            .elements(e, Some(&W::name("tblPr")))
            .into_iter()
            .any(|p| dom.element(p, &W::name("bidiVisual")).is_some());
        if bidi {
            "tbl|bidiVisual".to_string()
        } else {
            "tbl".to_string()
        }
    });

    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for (key, group) in grouped {
        if key.is_empty() || group.len() == 1 {
            for e in group {
                let c = dom.clone_subtree(e);
                dom.add(ne, c);
            }
            continue;
        }
        // M112: leave clean adjacent tables unmerged (Word Compare shape).
        if !group.iter().any(|&t| table_has_revision_marks(dom, t)) {
            for e in group {
                let c = dom.clone_subtree(e);
                dom.add(ne, c);
            }
            continue;
        }
        // union of cumulative grid widths across the group, ascending
        let mut rolled: Vec<i64> = Vec::new();
        for &tbl in &group {
            let mut sum = 0i64;
            for g in dom.elements(tbl, Some(&W::name("tblGrid"))) {
                for gc in dom.elements(g, Some(&W::name("gridCol"))) {
                    let v: i64 = dom
                        .attribute(gc, &W::name("w"))
                        .and_then(|v| v.parse().ok())
                        .expect("gridCol w:w must be an integer (C# casts)");
                    sum += v;
                    rolled.push(sum);
                }
            }
        }
        rolled.sort_unstable();
        rolled.dedup();

        let new_table = dom.new_element(tbl_name.clone());
        for pr in dom.elements(group[0], Some(&W::name("tblPr"))) {
            let c = dom.clone_subtree(pr);
            dom.add(new_table, c);
        }
        let new_grid = dom.new_element(W::name("tblGrid"));
        for (i, &r) in rolled.iter().enumerate() {
            let v = if i == 0 { r } else { r - rolled[i - 1] };
            let gc = dom.new_element(W::name("gridCol"));
            dom.set_attribute_value(gc, &W::name("w"), Some(&v.to_string()));
            dom.add(new_grid, gc);
        }
        dom.add(new_table, new_grid);

        for &tbl in &group {
            let fixed = fix_widths(dom, tbl);
            for tr in dom.elements(fixed, Some(&W::name("tr"))) {
                let new_row = dom.new_element(W::name("tr"));
                for (an, av) in dom.attributes(tr) {
                    dom.set_attribute_value(new_row, &an, Some(&av));
                }
                let non_cells: Vec<NodeId> = dom
                    .elements(tr, None)
                    .into_iter()
                    .filter(|&e| dom.name(e) != Some(W::name("tc")))
                    .collect();
                for e in non_cells {
                    let c = dom.clone_subtree(e);
                    dom.add(new_row, c);
                }
                for tc in dom.elements(tr, Some(&W::name("tc"))) {
                    let w: Option<i64> = dom
                        .element(tc, &W::name("tcPr"))
                        .and_then(|p| dom.element(p, &W::name("tcW")))
                        .and_then(|t| dom.attribute(t, &W::name("w")))
                        .and_then(|v| v.parse().ok());
                    let Some(w) = w else {
                        let c = dom.clone_subtree(tc);
                        dom.add(new_row, c);
                        continue;
                    };
                    let mut width_to_left = 0i64;
                    for btc in dom.elements(tr, Some(&W::name("tc"))) {
                        if btc == tc {
                            break;
                        }
                        width_to_left += dom
                            .element(btc, &W::name("tcPr"))
                            .and_then(|p| dom.element(p, &W::name("tcW")))
                            .and_then(|t| dom.attribute(t, &W::name("w")))
                            .and_then(|v| v.parse::<i64>().ok())
                            .unwrap_or(0);
                    }
                    // rolled_pairs = [0] ++ rolled; start = first >= width_to_left
                    let rolled_pairs: Vec<i64> =
                        std::iter::once(0).chain(rolled.iter().copied()).collect();
                    let start = rolled_pairs.iter().position(|&gv| gv >= width_to_left);
                    let Some(start_idx) = start else {
                        let c = dom.clone_subtree(tc);
                        dom.add(new_row, c);
                        continue;
                    };
                    let start_value = rolled_pairs[start_idx];
                    let grids_required = rolled_pairs[start_idx..]
                        .iter()
                        .take_while(|&&gv| gv - start_value < w)
                        .count() as i64;

                    let mut tcpr_kids: Vec<NodeId> = Vec::new();
                    let props: Vec<NodeId> = dom
                        .elements(tc, Some(&W::name("tcPr")))
                        .into_iter()
                        .flat_map(|p| dom.elements(p, None))
                        .filter(|&e| dom.name(e) != Some(W::name("gridSpan")))
                        .collect();
                    for e in props {
                        tcpr_kids.push(dom.clone_subtree(e));
                    }
                    if grids_required != 1 {
                        let gs = dom.new_element(W::name("gridSpan"));
                        dom.set_attribute_value(gs, &W::val(), Some(&grids_required.to_string()));
                        tcpr_kids.push(gs);
                    }
                    tcpr_kids.sort_by_key(|&e| order_tc_pr(&dom.name(e).unwrap()));
                    let ordered_tc_pr = dom.new_element(W::name("tcPr"));
                    for e in tcpr_kids {
                        dom.add(ordered_tc_pr, e);
                    }
                    let new_cell = dom.new_element(W::name("tc"));
                    dom.add(new_cell, ordered_tc_pr);
                    let body_kids: Vec<NodeId> = dom
                        .elements(tc, None)
                        .into_iter()
                        .filter(|&e| dom.name(e) != Some(W::name("tcPr")))
                        .collect();
                    for e in body_kids {
                        let c = dom.clone_subtree(e);
                        dom.add(new_cell, c);
                    }
                    dom.add(new_row, new_cell);
                }
                dom.add(new_table, new_row);
            }
        }
        dom.add(ne, new_table);
    }
    ne
}

// ─────────────── A.9 — empty paragraph in empty cells ───────────────────────

/// A.9 — `AddEmptyParagraphToAnyEmptyCells` (:1448): a `w:tc` with no element
/// children other than `w:tcPr` gains an empty `w:p`; everything else
/// rebuilds recursively.
pub fn add_empty_paragraph_to_any_empty_cells(dom: &mut Dom, node: NodeId) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if name == W::name("tc")
        && !dom
            .elements(node, None)
            .into_iter()
            .any(|e| dom.name(e) != Some(W::name("tcPr")))
    {
        let ne = dom.new_element(W::name("tc"));
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for e in dom.elements(node, None) {
            let c = dom.clone_subtree(e);
            dom.add(ne, c);
        }
        let p = dom.new_element(W::p());
        dom.add(ne, p);
        return ne;
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        let tc = add_empty_paragraph_to_any_empty_cells(dom, c);
        dom.add(ne, tc);
    }
    ne
}

// ─────────────── A.10 — the full AcceptRevisionsForPart pipeline ────────────

/// A.10 — `AcceptRevisionsForPart` (:1314) at content scope: the exact
/// 15-step transform order. `contains_move_from` is captured AFTER the
/// field-code fixup but BEFORE AcceptMoveFromMoveTo consumes the `w:moveFrom`
/// wrappers, gating RemoveRowsLeftEmptyByMoveFrom exactly like the C#.
///
/// ACCEPT-SKIP-01: when the subtree has no tracked-revision elements, skip
/// every revision-semantic full-tree rebuild (field fixup, move*, all-other,
/// deleted-cells, merge-adjacent). Still runs RemoveRsid, A.9 empty-cell
/// fill (not revision-gated in C#), and UniqueId/numPr cleanup.
pub fn accept_revisions_for_part_content(dom: &mut Dom, root: NodeId) -> NodeId {
    let has_rev = element_has_tracked_revisions(dom, root);
    let e = remove_rsid_transform(dom, root).expect("root not dropped by rsid removal");
    let e = if has_rev {
        let e = fix_up_deleted_or_inserted_field_codes_transform(dom, e);
        let contains_move_from = !dom.descendants(e, Some(&W::name("moveFrom"))).is_empty();
        let e = {
            let v = accept_move_from_move_to_transform(dom, e);
            debug_assert_eq!(v.len(), 1);
            v[0]
        };
        let e = accept_move_from_ranges(dom, e);
        let e = accept_paragraph_end_tags_in_move_from_transform(dom, e);
        let e = accept_deleted_and_moved_from_content_controls(dom, e);
        let e = accept_deleted_and_move_from_paragraph_marks(dom, e);
        let e = if contains_move_from {
            remove_rows_left_empty_by_move_from(dom, e)
        } else {
            e
        };
        let e = {
            let v = accept_all_other_revisions_transform(dom, e);
            debug_assert_eq!(v.len(), 1);
            v[0]
        };
        let e = accept_deleted_cells_transform(dom, e);
        merge_adjacent_tables_transform(dom, e)
    } else {
        e
    };
    let e = add_empty_paragraph_to_any_empty_cells(dom, e);

    // Strip PT.UniqueId / PT.RunIds attributes from all descendants.
    let unique_id = PT::name("UniqueId");
    let run_ids = PT::name("RunIds");
    for d in dom.descendants_and_self(e, None) {
        dom.set_attribute_value(d, &unique_id, None);
        dom.set_attribute_value(d, &run_ids, None);
    }
    // Remove empty w:numPr elements.
    let num_pr = W::name("numPr");
    for np in dom.descendants(e, Some(&num_pr)) {
        if !dom.has_elements(np) {
            dom.remove(np);
        }
    }
    e
}

// ─────────────── A.11 — package-scope accept / reject ───────────────────────

/// A.11 — `AcceptRevisionsForStylesTransform` (:1300): drop `pPrChange`/
/// `rPrChange` from the styles part, rebuild the rest.
fn accept_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(dom.clone_subtree(node));
    }
    let name = dom.name(node).unwrap();
    if name == W::name("pPrChange") || name == W::name("rPrChange") {
        return None;
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        if let Some(tc) = accept_revisions_for_styles_transform(dom, c) {
            dom.add(ne, tc);
        }
    }
    Some(ne)
}

/// A.11 — `RejectRevisionsForStylesTransform` (:391): a `pPr` holding a
/// `pPrChange` reverts to the change's inner `pPr` (same for `rPr`); a change
/// element without its inner properties drops the whole node (C# recurses
/// null).
fn reject_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(dom.clone_subtree(node));
    }
    let name = dom.name(node).unwrap();
    if name == W::p_pr()
        && let Some(chg) = dom.element(node, &W::name("pPrChange"))
    {
        let inner = dom.element(chg, &W::p_pr());
        return inner.and_then(|i| reject_revisions_for_styles_transform(dom, i));
    }
    if name == W::r_pr()
        && let Some(chg) = dom.element(node, &W::name("rPrChange"))
    {
        let inner = dom.element(chg, &W::r_pr());
        return inner.and_then(|i| reject_revisions_for_styles_transform(dom, i));
    }
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        if let Some(tc) = reject_revisions_for_styles_transform(dom, c) {
            dom.add(ne, tc);
        }
    }
    Some(ne)
}

/// The parts `AcceptRevisions`/`RejectRevisions` (:1277/:31) walk, in the C#
/// order: main, headers, footers, endnotes, footnotes, then styles (flagged).
fn revision_bearing_parts(pkg: &crate::opc::PartFs) -> Vec<(String, bool)> {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let mut headers = Vec::new();
    let mut footers = Vec::new();
    let mut endnotes = Vec::new();
    let mut footnotes = Vec::new();
    let mut styles = Vec::new();
    if let Some(rels) = pkg.read_rels_for(&main) {
        for r in &rels.items {
            if r.target_mode.as_deref() == Some("External") {
                continue;
            }
            let bucket = match r.rel_type.rsplit('/').next().unwrap_or("") {
                "header" => &mut headers,
                "footer" => &mut footers,
                "endnotes" => &mut endnotes,
                "footnotes" => &mut footnotes,
                "styles" => &mut styles,
                _ => continue,
            };
            bucket.push(pkg.resolve_rel_target(&main, &r.target));
        }
    }
    let mut out = vec![(main, false)];
    for p in headers
        .into_iter()
        .chain(footers)
        .chain(endnotes)
        .chain(footnotes)
    {
        out.push((p, false));
    }
    for p in styles {
        out.push((p, true));
    }
    out
}

fn process_part<F>(pkg: &mut crate::opc::PartFs, part: &str, f: F)
where
    F: FnOnce(&mut Dom, NodeId) -> Option<NodeId>,
{
    let Some(xml) = pkg.part_string(part) else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    let Some(new_root) = f(&mut dom, root) else {
        return;
    };
    dom.replace_with(root, &[new_root]);
    pkg.set_part(part, dom.serialize_document(doc).into_bytes());
}

/// A.11 — `AcceptRevisions` (:1277) at package scope: run the full part
/// pipeline over main + headers + footers + endnotes + footnotes, and the
/// styles transform over the styles part.
pub fn accept_revisions_package(pkg: &mut crate::opc::PartFs) {
    for (part, is_styles) in revision_bearing_parts(pkg) {
        if is_styles {
            process_part(pkg, &part, |dom, root| {
                accept_revisions_for_styles_transform(dom, root)
            });
        } else {
            process_part(pkg, &part, |dom, root| {
                Some(accept_revisions_for_part_content(dom, root))
            });
        }
    }
}

/// A.11 — `RejectRevisions` (:31) at package scope: per content part, the
/// revert → reverse → rsid-strip → full-accept composition (the C# phases the
/// same steps across all parts; parts are independent, so per-part composition
/// is equivalent); the styles part reverts its property changes then accepts
/// the leftovers.
pub fn reject_revisions_package(pkg: &mut crate::opc::PartFs) {
    for (part, is_styles) in revision_bearing_parts(pkg) {
        if is_styles {
            process_part(pkg, &part, |dom, root| {
                let rejected = reject_revisions_for_styles_transform(dom, root)?;
                accept_revisions_for_styles_transform(dom, rejected)
            });
        } else {
            process_part(pkg, &part, |dom, root| {
                Some(reject_revisions_document(dom, root))
            });
        }
    }
}
