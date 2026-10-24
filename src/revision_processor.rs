// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Revision accept / reject pipeline (M3).
//!
//! Port of `RevisionProcessor.ts` — the byte-level accept / reject facade
//! (`accept_revisions_package`, `reject_revisions_package`) and the
//! A.0–A.11 transforms that compose
//! them: paragraph / block-content iteration, paragraph-mark and
//! content-control re-wrap, moveFrom / moveTo, deleted-cell merging,
//! adjacent-table merging, and the reject path (revert → reverse →
//! remove rsid → accept).
//!
//! The element-scope helpers (`accept_revisions_document`,
//! `reject_revisions_document`, `accept_revisions_for_element`) are
//! the per-subtree entry the M4 compare pipeline calls to hash the
//! accepted / rejected projections before correlating them.

mod bookmarks;
pub(crate) mod comments;
mod notes;
mod sections;
pub(crate) mod style_records;
mod word_save;

pub(crate) use comments::{prune_orphan_comments, renumber_rescued_comments};
pub(crate) use sections::Resolution;
pub(crate) use word_save::drop_unreferenced_headers_footers;

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

/// The element names whose presence indicates the document carries
/// tracked revisions (one per local name in the `w:` namespace). The
/// `has_tracked_revisions` check below is the constant-time local-name
/// lookup the byte-level pipeline uses.
pub fn tracked_revisions_elements() -> Vec<XName> {
    TRACKED_REVISION_LOCALS.iter().map(|l| W::name(l)).collect()
}

/// True if `root` (or any descendant) is a tracked-revision element. The
/// element-scope equivalent of `RevisionProcessor.PartHasTrackedRevisions`.
///
/// The check is a non-allocating DFS against the static local-name set
/// (ACCEPT-SCAN-01): no `Vec<XName>` / `descendants_and_self` allocation,
/// so the byte-level pipeline can call it on every part without GC churn.
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

/// Port of `AcceptMoveFromMoveToTransform` — unwrap `w:moveTo`, drop
/// `w:moveFrom`. A moved-from paragraph mark (`pPr/rPr/moveFrom`) is kept
/// (A.5a joins that paragraph with the next, so the join the witness
/// expects needs to be visible). Otherwise the whole `w:moveFrom` is hoisted
/// into range markers.
///
/// Subtrees with no moveFrom / moveTo (and no other tracked revision
/// markers) are detached and returned as-is — `Dom::add` reuses them
/// without `clone_subtree`.
pub fn accept_move_from_move_to_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    // No move/revision markers in this subtree → transfer ownership.
    if !element_has_tracked_revisions(dom, node) {
        return take_subtree(dom, node);
    }
    if !dom.is_element(node) {
        return take_subtree(dom, node);
    }
    let name = dom.name(node).unwrap();
    if name == W::move_to() {
        let mut out = Vec::new();
        for c in dom.nodes(node) {
            out.extend(accept_move_from_move_to_transform(dom, c));
        }
        return out;
    }
    if name == W::move_from() {
        // A moved-from paragraph mark (`pPr/rPr/moveFrom`) stays for A.5a,
        // which joins that paragraph with the next as Word does; dropping it
        // here left an empty paragraph behind.
        let is_mark = dom.parent(node).is_some_and(|rpr| {
            dom.name(rpr) == Some(W::r_pr())
                && dom.parent(rpr).and_then(|ppr| dom.name(ppr)) == Some(W::p_pr())
        });
        return if is_mark {
            take_subtree(dom, node)
        } else {
            // The bookmark range pass already removed wholly moved-away
            // spans. Preserve empty or partially surviving anchors exactly
            // as the ordinary deletion transform does.
            hoist_range_markers_from(dom, node)
        };
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

/// Detach `node` from its parent (if any) and return it for reparenting.
/// Unparented nodes are reused by [`Dom::add`] without cloning.
fn take_subtree(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    if dom.parent(node).is_some() {
        dom.remove(node);
    }
    vec![node]
}

/// Non-allocating presence of `name` on `root` or any descendant element.
fn element_or_desc_has_name(dom: &Dom, root: NodeId, name: &XName) -> bool {
    fn walk(dom: &Dom, id: NodeId, name: &XName) -> bool {
        if dom.name(id).as_ref() == Some(name) {
            return true;
        }
        let n = dom.child_count(id);
        for i in 0..n {
            if walk(dom, dom.child_at(id, i), name) {
                return true;
            }
        }
        false
    }
    walk(dom, root, name)
}

/// True iff a transformed element still carries run/inline content (port of
/// `HasRunContent`) — used to decide whether an emptied `w:hyperlink` survives.
fn has_run_content(dom: &Dom, element: NodeId) -> bool {
    let content_names = [
        W::r(),
        W::smart_tag(),
        W::ins(),
        W::del(),
        W::hyperlink(),
        W::fld_simple(),
        W::sdt(),
    ];
    dom.elements(element, None)
        .into_iter()
        .any(|e| dom.name(e).is_some_and(|n| content_names.contains(&n)))
}

fn try_accept_wholly_revised_equation(
    dom: &mut Dom,
    node: NodeId,
    name: &XName,
) -> Option<Vec<NodeId>> {
    if name == &M::name("oMath") || name == &M::name("oMathPara") {
        let owners: Vec<_> = dom
            .descendants(node, None)
            .into_iter()
            .filter(|&n| dom.name_is(n, &M::name("r")) || dom.name_is(n, &M::name("ctrlPr")))
            .collect();
        let has_authored_empty_equation = dom
            .descendants(node, Some(&M::name("oMath")))
            .into_iter()
            .any(|equation| {
                !dom.descendants(equation, None)
                    .into_iter()
                    .any(|n| dom.name_is(n, &M::name("r")) || dom.name_is(n, &M::name("ctrlPr")))
            });
        let has_unowned_content = dom.descendants(node, None).into_iter().any(|child| {
            let ancestors = dom.ancestors(child, None);
            if dom.name_is(child, &W::del())
                || ancestors
                    .iter()
                    .any(|&ancestor| dom.name_is(ancestor, &W::del()))
            {
                return false;
            }
            let Some(child_name) = dom.name(child) else {
                return !dom
                    .text_value(child)
                    .is_some_and(|text| text.trim().is_empty());
            };
            // oMath admits ordinary WML runs, anchors and transparent controls.
            // They are independent owners even when all m:r owners are deleted.
            if child_name.namespace_name() != M::URI {
                return true;
            }
            if child_name.local_name() == "r" || child_name.local_name() == "ctrlPr" {
                return false;
            }
            // Math property leaves describe the removed equation; an empty
            // expression/container outside properties is an authored sibling.
            let in_math_properties = child_name.local_name().ends_with("Pr")
                || ancestors.iter().any(|&ancestor| {
                    dom.name(ancestor).is_some_and(|name| {
                        name.namespace_name() == M::URI && name.local_name().ends_with("Pr")
                    })
                });
            !in_math_properties && dom.elements(child, None).is_empty()
        });
        if !has_authored_empty_equation
            && !has_unowned_content
            && !owners.is_empty()
            && owners.iter().all(|&owner| {
                let children = dom.elements(owner, None);
                !children.is_empty()
                    && children.iter().all(|&child| dom.name_is(child, &W::del()))
                    && dom.nodes(owner).into_iter().all(|child| {
                        dom.is_element(child)
                            || dom
                                .text_value(child)
                                .is_some_and(|text| text.trim().is_empty())
                    })
            })
        {
            return Some(hoist_range_markers_from(dom, node));
        }
    }
    None
}

fn try_accept_math_fraction(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &M::name("f") {
        let removed = dom
            .elements(node, Some(&M::name("fPr")))
            .into_iter()
            .any(|fpr| has_path(dom, fpr, &M::name("ctrlPr"), &W::del()));
        if removed {
            return Some(vec![]);
        }
    }
    None
}

fn try_accept_deleted_row(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::tr() && has_path(dom, node, &W::tr_pr(), &W::del()) {
        return Some(vec![]);
    }
    None
}

fn try_accept_deleted_table(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::tbl() && table_rows_all_deleted(dom, node) {
        return Some(vec![]);
    }
    None
}

fn try_accept_deleted_text(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::del() {
        return Some(hoist_range_markers_from(dom, node));
    }
    None
}

fn try_accept_cell_merge(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::cell_merge() {
        let parent_is_tcpr = dom
            .parent(node)
            .and_then(|p| dom.name(p))
            .is_some_and(|pn| pn == W::tc_pr());
        if parent_is_tcpr {
            let vmerge = dom.attribute(node, &W::v_merge()).map(|s| s.to_string());
            if vmerge.as_deref() == Some("rest") {
                let v = dom.new_element(W::v_merge());
                dom.set_attribute_value(v, &W::val(), Some("restart"));
                return Some(vec![v]);
            }
            if vmerge.as_deref() == Some("cont") {
                let v = dom.new_element(W::v_merge());
                dom.set_attribute_value(v, &W::val(), Some("continue"));
                return Some(vec![v]);
            }
        }
    }
    None
}

fn try_accept_hyperlink(dom: &mut Dom, node: NodeId, name: &XName) -> Option<Vec<NodeId>> {
    if name == &W::hyperlink() {
        let ne = dom.new_element(name.clone());
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for c in dom.nodes(node) {
            for tn in accept_all_other_revisions_transform(dom, c) {
                dom.add(ne, tn);
            }
        }
        return Some(if has_run_content(dom, ne) {
            vec![ne]
        } else {
            hoist_range_markers_from(dom, ne)
        });
    }
    None
}

/// Port of `AcceptAllOtherRevisionsTransform` — accept inserts (unwrap
/// `w:ins`), drop deletions and revision-range markers, accept formatting-
/// change markers, handle deleted rows/tables, cell merges, and empty
/// hyperlink shells.
///
/// A wholly-revised equation (every `m:r` / `m:ctrlPr` deleted) is
/// removed entirely; a wholly-revised oMath inside a surviving paragraph
/// goes with it (so the paragraph never carries an empty math shell).
///
/// Subtrees with no tracked-revision elements are detached and returned
/// as-is (ACCEPT-REUSE-CLEAN); callers reparent via `Dom::add`, which
/// reuses unparented nodes.
pub fn accept_all_other_revisions_transform(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    // Clean subtree (incl. plain text leaves): transfer, do not rebuild.
    if !element_has_tracked_revisions(dom, node) {
        return take_subtree(dom, node);
    }
    if !dom.is_element(node) {
        return take_subtree(dom, node);
    }
    let name = dom.name(node).unwrap();

    // Accept inserted content: collapse w:ins.
    // #1155: resolve even the final paragraph mark, unlike Word's Accept/Reject All.
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

    if let Some(res) = try_accept_wholly_revised_equation(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_math_fraction(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_deleted_row(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_deleted_table(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_deleted_text(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_cell_merge(dom, node, &name) {
        return res;
    }
    if let Some(res) = try_accept_hyperlink(dom, node, &name) {
        return res;
    }

    // Identity clone with transformed children.
    let ne = dom.new_element(name.clone());
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
    let unique_id = PT::unique_id();
    let run_ids = PT::run_ids();
    for d in dom.descendants_and_self(e, None) {
        dom.set_attribute_value(d, &unique_id, None);
        dom.set_attribute_value(d, &run_ids, None);
    }

    // Remove empty w:numPr elements.
    let num_pr = W::num_pr();
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
    let skip = [W::sdt_content(), W::sdt(), W::smart_tag()];
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

    // Deleted paragraph mark (del in rPr/pPr) → empty w:ins.
    if name == W::del() && parent_name.as_ref() == Some(&W::r_pr()) && grandparent_is_ppr {
        return dom.new_element(W::ins());
    }
    // Inserted paragraph mark (ins in rPr/pPr) → empty w:del.
    if name == W::ins() && parent_name.as_ref() == Some(&W::r_pr()) && grandparent_is_ppr {
        return dom.new_element(W::del());
    }
    // Deleted / inserted table row (del/ins in trPr) → empty swap.
    if name == W::del() && parent_name.as_ref() == Some(&W::tr_pr()) {
        return dom.new_element(W::ins());
    }
    if name == W::ins() && parent_name.as_ref() == Some(&W::tr_pr()) {
        return dom.new_element(W::del());
    }
    // Any OTHER deleted content → w:ins (reversed children); any other inserted
    // content → w:del. REJECT-LOSSLESS-01: the C# reference only flips del/ins
    // whose effective parent is w:p / w:hyperlink / m:r, leaving a content del/ins
    // nested in a transparent run container (`w:fldSimple`, `mc:Choice`/`mc:Fallback`,
    // …) as identity — which the trailing accept then DROPS (del) or KEEPS (ins),
    // losing the base/revised text (field results, AlternateContent). Since on reject
    // a content deletion must always be RESTORED and a content insertion always
    // REMOVED, flipping every remaining del↔ins is both correct and lossless. Reached
    // only AFTER the paragraph-mark / table-row markers above (which carry no run
    // children and must stay empty); `parent`/`parent_name` are still consulted there.
    if name == W::del() {
        return rebuild_named(dom, W::ins(), node, false);
    }
    if name == W::ins() {
        return rebuild_named(dom, W::del(), node, false);
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
///
/// A property element reverting to its saved copy keeps the revisions a
/// selective resolution left tracked in it ([`carry_kept_revisions`]).
fn reject_revisions_for_part_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(dom.clone_subtree(node));
    }
    let name = dom.name(node).unwrap();

    // Inserted numbering properties: numPr containing w:ins → drop.
    if name == W::num_pr() && dom.element(node, &W::ins()).is_some() {
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
        // R9: a sectPrChange records the section's properties, never its
        // header and footer references (CT_SectPrBase has none): the live
        // ones stay, first as the schema orders them (205503ead9).
        if name == W::sect_pr() {
            let refs: Vec<NodeId> = dom
                .elements(node, None)
                .into_iter()
                .filter(|&e| {
                    dom.name(e).is_some_and(|n| {
                        n == W::name("headerReference") || n == W::name("footerReference")
                    })
                })
                .collect();
            for r in refs.into_iter().rev() {
                let c = dom.clone_subtree(r);
                dom.add_first(new_prop, c);
            }
        }
        carry_kept_revisions(dom, node, new_prop, false);
        return reject_revisions_for_part_transform(dom, new_prop);
    }
    // rPrChange: replace rPr by the change's saved rPr.
    if name == W::r_pr()
        && let Some(chg) = dom.element(node, &W::r_pr_change())
    {
        let saved = dom.element(chg, &W::r_pr());
        let new_rpr = match saved {
            Some(sp) => dom.clone_subtree(sp),
            None => dom.new_element(W::r_pr()),
        };
        carry_kept_revisions(dom, node, new_rpr, true);
        return reject_revisions_for_part_transform(dom, new_rpr);
    }
    // numberingChange / cellDel / cellMerge → drop.
    if name == W::numbering_change() || name == W::cell_del() || name == W::cell_merge() {
        return None;
    }
    // tc whose tcPr contains a cellIns → drop the inserted cell.
    if name == W::tc() {
        let has_cell_ins = dom
            .elements(node, Some(&W::tc_pr()))
            .into_iter()
            .any(|tcpr| dom.element(tcpr, &W::cell_ins()).is_some());
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
///
/// `RejectRevisionsDocument` at element scope — reject every tracked
/// revision in `root`'s subtree (revert → reverse → remove rsid → accept),
/// returning the new root.
///
/// When the subtree has no tracked-revision elements the full reject
/// path is identity; only `RemoveRsid` is needed (in-place). The return
/// value is the same `root` `NodeId`.
pub fn reject_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId {
    reject_content(dom, root, false, &mut Vec::new())
}

fn reject_content(
    dom: &mut Dom,
    root: NodeId,
    keep_comments: bool,
    kept: &mut Vec<String>,
) -> NodeId {
    if !element_has_tracked_revisions(dom, root) {
        return remove_rsid_transform(dom, root).expect("reject clean: root not dropped by rsid");
    }
    let reverted =
        reject_revisions_for_part_transform(dom, root).expect("reject: root must not be dropped");
    // O.0 (#1243): before the reversal, drop only the orphan moveTo range
    // markers. The reversal would turn them into moveFrom ranges whose
    // unwrapped content A.2 deletes on the accept pass below, but Word keeps
    // an orphan moveTo range's content on Reject (#1235). Orphan moveFrom
    // markers survive the reversal as moveTo markers — inert for A.2/A.4,
    // stripped by A.10 — so their content is kept too (#1243).
    let reverted = normalize_orphan_moves(dom, reverted, OrphanMovePass::RejectMoveToMarkers);
    let reversed = reverse_revisions_transform(dom, reverted);
    let derssid = remove_rsid_transform(dom, reversed).expect("reject: root not dropped by rsid");
    accept_content(dom, derssid, keep_comments, kept)
}

// ───────────────────────────── A.0 — part-pipeline walkers ──────────────────

/// `TagTypeEnum` (RevisionProcessor.cs :2389): how an element appears in the
/// doc-order tag stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagType {
    /// An element open tag (one or more children follow).
    Element,
    /// An element that carries no children of any kind (no elements and no
    /// text); serializes as a self-closing tag in XML.
    EmptyElement,
    /// An element close tag (its children are exhausted).
    EndElement,
}

/// `Tag` (RevisionProcessor.cs :2393): one open/empty/close event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag {
    /// The element this tag event names.
    pub element: NodeId,
    /// Whether this is an open / empty / close event.
    pub tag_type: TagType,
}

/// A.0 — `DescendantAndSelfTags` (RevisionProcessor.cs :2397): stream
/// `element` and its descendants as open / empty / close tags in document
/// order.
///
/// Faithful to C#: a child with no nodes AT ALL (no elements and no
/// text) is `EmptyElement`; a child with only text still opens and closes;
/// the root element itself always gets an open/close pair, never
/// `EmptyElement`.
pub fn descendant_and_self_tags(dom: &Dom, element: NodeId) -> Vec<Tag> {
    let mut out = vec![Tag {
        element,
        tag_type: TagType::Element,
    }];
    // DOM-ITER-04: stack of (element children, next index, last advanced element).
    // Element-child lists still materialize (EndElement needs the finished id),
    // but empty checks use `child_count` (no `nodes()` clone).
    let mut stack: Vec<(Vec<NodeId>, usize)> = vec![(element_children_vec(dom, element), 0)];
    while let Some(top) = stack.last_mut() {
        if top.1 < top.0.len() {
            let current = top.0[top.1];
            top.1 += 1;
            if dom.child_count(current) == 0 {
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
            stack.push((element_children_vec(dom, current), 0));
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

/// Direct element children (document order) without the `elements()` filter path.
fn element_children_vec(dom: &Dom, id: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    let n = dom.child_count(id);
    for i in 0..n {
        let c = dom.child_at(id, i);
        if dom.is_element(c) {
            out.push(c);
        }
    }
    out
}

/// `BlockContentInfo` (RevisionProcessor.cs :52): prev / this / next links
/// for block-level content. [`iterate_block_content_elements`] fills all
/// three (`this_block_content_element` always `Some`);
/// [`get_paragraph_info`] fills only the previous sibling and `this`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockContentInfo {
    /// Previous block-content element in document order (`Some` after the
    /// first chain element).
    pub previous_block_content_element: Option<NodeId>,
    /// The chain element itself (always `Some` from
    /// [`iterate_block_content_elements`]).
    pub this_block_content_element: Option<NodeId>,
    /// Next block-content element in document order (`None` at the end).
    pub next_block_content_element: Option<NodeId>,
}

/// First paragraph, table, or opaque equation block among `roots`' descendants-and-self, document order.
/// DOM-ITER-04: early-exit iterative walk (no full `descendants_and_self` Vec).
fn first_block_content(dom: &Dom, roots: &[NodeId]) -> Option<NodeId> {
    let (p, tbl) = (W::p(), W::tbl());
    let is_block = |e: NodeId| {
        dom.name(e).is_some_and(|n| {
            n == p || n == tbl || n == M::name("oMath") || n == M::name("oMathPara")
        })
    };
    for &r in roots {
        if is_block(r) {
            return Some(r);
        }
        let mut stack: Vec<(NodeId, usize)> = vec![(r, 0)];
        while let Some((node, i)) = stack.last_mut() {
            let n = dom.child_count(*node);
            if *i >= n {
                stack.pop();
                continue;
            }
            let c = dom.child_at(*node, *i);
            *i += 1;
            if !dom.is_element(c) {
                continue;
            }
            if is_block(c) {
                return Some(c);
            }
            stack.push((c, 0));
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
/// (:1855): the doc-order chain of paragraphs, tables, and equation blocks under
/// `element`, linked prev/this/next. FAITHFUL: the next-search starts at the
/// current element's FOLLOWING siblings (climbing ancestors up to `element`),
/// so a table's inner paragraphs never appear once the table itself matched.
pub fn iterate_block_content_elements(dom: &Dom, element: NodeId) -> Vec<BlockContentInfo> {
    // DOM-ITER-04: one element-children collect (was two `elements()` calls).
    let kids = element_children_vec(dom, element);
    if kids.is_empty() {
        return Vec::new();
    }
    let Some(first) = first_block_content(dom, &kids) else {
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

/// `W.BlockLevelContentContainers` (PtOpenXmlUtil.cs :5569), plus
/// `w:comment`: the C# pipeline never visits `comments.xml`, but Word's
/// Accept/Reject All does, and a deleted or inserted paragraph mark inside
/// a comment merges the comment's own paragraphs — never across the
/// `w:comment` boundary.
fn block_level_content_containers() -> [XName; 8] {
    [
        W::body(),
        W::tc(),
        W::txbx_content(),
        W::hdr(),
        W::ftr(),
        W::endnote(),
        W::footnote(),
        W::name("comment"),
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

    let (p, tc, txbx) = (W::p(), W::tc(), W::txbx_content());
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
    if name == W::instr_text() {
        let ne = dom.new_element(W::del_instr_text());
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
    // ACCEPT-SKIP-A1: no field marks → identity (keep parent links; do not detach).
    let fld = W::fld_char();
    let instr = W::instr_text();
    if !element_or_desc_has_name(dom, node, &fld) && !element_or_desc_has_name(dom, node, &instr) {
        return node;
    }
    if !dom.is_element(node) {
        return node;
    }
    let name = dom.name(node).unwrap();
    if name == W::p() {
        let key_of = |dom: &Dom, e: NodeId| -> u8 {
            let n = dom.name(e).unwrap();
            let holds_fld_char = |d: &Dom| {
                d.elements(e, Some(&W::r()))
                    .into_iter()
                    .any(|r| d.element(r, &W::fld_char()).is_some())
            };
            if n == W::del() && holds_fld_char(dom) {
                2
            } else if n == W::ins() && holds_fld_char(dom) {
                3
            } else if n == W::r() && dom.element(e, &W::instr_text()).is_some() {
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

// ────────── O.0 — orphan move normalization (#1235, #1243) ───────────

/// Which orphan-move cleanup a resolution path runs (#1243).
#[derive(Clone, Copy, PartialEq, Eq)]
enum OrphanMovePass {
    /// Accept (and Reject, on its reversed tree): orphan `moveTo` elements
    /// become `w:ins` and orphan `moveFrom` elements become `w:del`; range
    /// markers stay. Word deletes an orphan `moveFromRange`'s unwrapped
    /// content on Accept (#1243), which is A.2's own reading of the
    /// markers, and A.10 strips every leftover marker afterwards.
    AcceptElements,
    /// Reject, before the reversal: drop only orphan `moveToRange…`
    /// markers (no element conversion). The reversal would otherwise turn
    /// them into `moveFromRange` markers whose unwrapped content A.2
    /// deletes on the accept pass below, while Word keeps an orphan
    /// `moveToRange`'s content on Reject (#1235).
    RejectMoveToMarkers,
}

/// #1235, #1243: a `w:moveTo` / `w:moveFrom` is an ORPHAN when no range of
/// the other kind shares its `w:name` in the same part — a bare `moveTo`
/// has no `w:name` at all, so it is an orphan unless it sits inside a
/// paired `moveToRange`. Word keeps an orphan `moveTo` like an insertion
/// and an orphan `moveFrom` like a deletion; only the elements themselves
/// carry moved content.
///
/// [`OrphanMovePass::AcceptElements`] rewrites those elements to
/// `w:ins`/`w:del` and leaves every range marker in place: Word deletes the
/// unwrapped content of a `moveFromRange` on Accept whether or not a
/// `moveToRange` shares its `w:name` (#1243), so A.2 (A.4 for the
/// `customXml…` markers) must still see an orphan `moveFromRange`.
/// [`OrphanMovePass::RejectMoveToMarkers`] drops only orphan
/// `moveToRange`/`customXmlMoveToRange` markers, before the reversal turns
/// them into `moveFromRange` markers whose content A.2 would wrongly
/// delete — Word keeps it (#1235). A well-formed move —
/// `moveFromRange`/`moveToRange` sharing a `w:name` — keeps its markers
/// and its `moveFrom`/`moveTo` elements and is untouched.
fn normalize_orphan_moves(dom: &mut Dom, root: NodeId, pass: OrphanMovePass) -> NodeId {
    let ns = W::URI;
    let mut from_names: HashSet<String> = HashSet::new();
    let mut to_names: HashSet<String> = HashSet::new();
    for e in dom.descendants_and_self(root, None) {
        let Some(n) = dom.name(e) else { continue };
        if n.namespace_name() != ns {
            continue;
        }
        match n.local_name() {
            "moveFromRangeStart" | "customXmlMoveFromRangeStart" => {
                if let Some(v) = dom.attribute(e, &W::name("name")) {
                    from_names.insert(v.to_string());
                }
            }
            "moveToRangeStart" | "customXmlMoveToRangeStart" => {
                if let Some(v) = dom.attribute(e, &W::name("name")) {
                    to_names.insert(v.to_string());
                }
            }
            _ => {}
        }
    }
    match pass {
        // No moveTo range markers exist → nothing to drop before the
        // reversal.
        OrphanMovePass::RejectMoveToMarkers if to_names.is_empty() => return root,
        OrphanMovePass::AcceptElements if from_names.is_empty() && to_names.is_empty() => {
            // Ranges cannot tell orphan from paired without names, but a bare
            // `w:moveTo` / `w:moveFrom` is still orphaned: it has no `w:name`
            // to pair with anything.
            let has_move = dom.descendants_and_self(root, None).into_iter().any(|e| {
                dom.name(e)
                    .is_some_and(|n| n == W::move_to() || n == W::move_from())
            });
            if !has_move {
                return root;
            }
        }
        _ => {}
    }

    let mut to_ins: HashSet<NodeId> = HashSet::new();
    let mut to_del: HashSet<NodeId> = HashSet::new();
    let mut drop_markers: HashSet<NodeId> = HashSet::new();
    // id → w:name of every currently open range, by flavor.
    let mut open_from: HashMap<String, String> = HashMap::new();
    let mut open_to: HashMap<String, String> = HashMap::new();
    for tag in descendant_and_self_tags(dom, root) {
        let e = tag.element;
        let Some(n) = dom.name(e) else { continue };
        if n.namespace_name() != ns {
            continue;
        }
        match n.local_name() {
            "moveFromRangeStart" | "customXmlMoveFromRangeStart" => {
                let id = dom.attribute(e, &W::id()).unwrap_or("").to_string();
                let name = dom.attribute(e, &W::name("name")).unwrap_or("").to_string();
                open_from.insert(id, name);
            }
            "moveFromRangeEnd" | "customXmlMoveFromRangeEnd" => {
                let id = dom.attribute(e, &W::id()).unwrap_or("");
                open_from.remove(id);
            }
            "moveToRangeStart" | "customXmlMoveToRangeStart" => {
                let id = dom.attribute(e, &W::id()).unwrap_or("").to_string();
                let name = dom.attribute(e, &W::name("name")).unwrap_or("").to_string();
                if pass == OrphanMovePass::RejectMoveToMarkers && !from_names.contains(&name) {
                    drop_markers.insert(e);
                }
                open_to.insert(id, name);
            }
            "moveToRangeEnd" | "customXmlMoveToRangeEnd" => {
                let id = dom.attribute(e, &W::id()).unwrap_or("");
                if let Some(name) = open_to.remove(id)
                    && pass == OrphanMovePass::RejectMoveToMarkers
                    && !from_names.contains(&name)
                {
                    drop_markers.insert(e);
                }
            }
            "moveTo"
                if tag.tag_type != TagType::EndElement
                    && pass == OrphanMovePass::AcceptElements =>
            {
                let inside_paired_range = open_to.values().any(|name| from_names.contains(name));
                if !inside_paired_range {
                    to_ins.insert(e);
                }
            }
            "moveFrom"
                if tag.tag_type != TagType::EndElement
                    && pass == OrphanMovePass::AcceptElements =>
            {
                let inside_paired_range = open_from.values().any(|name| to_names.contains(name));
                if !inside_paired_range {
                    to_del.insert(e);
                }
            }
            _ => {}
        }
    }
    if to_ins.is_empty() && to_del.is_empty() && drop_markers.is_empty() {
        return root;
    }
    normalize_orphan_moves_transform(dom, root, &to_ins, &to_del, &drop_markers)
        .expect("the part root is never a move element or range marker")
}

/// Rebuild for [`normalize_orphan_moves`]: orphan `moveTo`/`moveFrom`
/// re-emerge as `w:ins`/`w:del` (attributes preserved), orphan range markers
/// vanish, everything else rebuilds recursively.
fn normalize_orphan_moves_transform(
    dom: &mut Dom,
    node: NodeId,
    to_ins: &HashSet<NodeId>,
    to_del: &HashSet<NodeId>,
    drop_markers: &HashSet<NodeId>,
) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(dom.clone_subtree(node));
    }
    if drop_markers.contains(&node) {
        return None;
    }
    let new_name = if to_ins.contains(&node) {
        W::ins()
    } else if to_del.contains(&node) {
        W::del()
    } else {
        dom.name(node).unwrap()
    };
    let ne = dom.new_element(new_name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        if let Some(tc) = normalize_orphan_moves_transform(dom, c, to_ins, to_del, drop_markers) {
            dom.add(ne, tc);
        }
    }
    Some(ne)
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
    let to_delete = move_from_range_deleted_elements(dom, document);
    if to_delete.is_empty() {
        return document;
    }
    accept_move_from_ranges_transform(dom, document, &to_delete)
        .expect("the document root is never in a moveFrom range")
}

/// Elements wholly removed by completed move-from ranges. The bookmark
/// prepass uses the same deletion set before wrappers/anchors are hoisted,
/// so range-only moved characters count as deleted just like wrapped ones.
fn move_from_range_deleted_elements(dom: &Dom, document: NodeId) -> HashSet<NodeId> {
    use std::collections::{HashMap, HashSet};

    let mfrs = W::move_from_range_start();
    let mfre = W::move_from_range_end();

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
    // A range is a pair of markers, not a container: Word ends one inside the
    // first cell of the table after a moved heading (30f20e787b). The
    // properties of a container the range only enters stay with it (a table
    // keeps its tblPr and tblGrid); one wholly inside goes with its container.
    // A paragraph's pPr inside the range is its mark, not moved text: its
    // deleted or moved-from state is what A.5a joins paragraphs by (Word),
    // and a paragraph that survives keeps its formatting.
    start_tags_in_range
        .into_iter()
        .filter(|&e| {
            end_set.contains(&e)
                // Empty/partially surviving range anchors were hoisted from
                // moveFrom after wholly deleted bookmarks were removed.
                // The move range must not discard those anchors a second time.
                && !dom.name(e).is_some_and(|name| is_inline_range_marker(&name))
                && !is_container_property(dom, e)
                && !dom
                    .ancestors(e, None)
                    .into_iter()
                    .any(|a| is_container_property(dom, a))
        })
        .collect()
}

/// Is `e` the property element of its container (a paragraph's pPr, a
/// table's tblPr or tblGrid, a row's trPr or tblPrEx, a cell's tcPr, a
/// content control's sdtPr or sdtEndPr)?
fn is_container_property(dom: &Dom, e: NodeId) -> bool {
    dom.name(e).is_some_and(|n| {
        n.namespace_name() == W::URI
            && matches!(
                n.local_name(),
                "pPr" | "tblPr" | "tblGrid" | "tblPrEx" | "trPr" | "tcPr" | "sdtPr" | "sdtEndPr"
            )
    })
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
        } else {
            // A whole inline/block container can fall inside the move range;
            // preserve its surviving anchors without keeping the container.
            for marker in hoist_range_markers_from(dom, c) {
                dom.add(ne, marker);
            }
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
///
/// ACCEPT-SKIP-A3: when the subtree has no `w:moveFromRangeStart`, every
/// grouping key is "Other" and the transform is a pure identity rebuild —
/// transfer `node` without cloning (common path for ins/del-only redlines).
pub fn accept_paragraph_end_tags_in_move_from_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    // ACCEPT-SKIP-A3: no moveFromRangeStart → identity (keep parent links).
    if !element_or_desc_has_name(dom, node, &W::move_from_range_start()) {
        return node;
    }
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if block_level_content_containers().contains(&name) {
        let mfrs = W::move_from_range_start();
        let mfre = W::move_from_range_end();
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
    let mfrs = W::move_from_range_start();
    let mfre = W::move_from_range_end();
    let sdt = W::sdt();

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
    if name == W::sdt() && to_collapse.contains(&node) {
        let content = dom
            .element(node, &W::sdt_content())
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
    // An authored compatibility wrapper carries its complete presentation
    // alternatives, like math or a field wrapper. Capability selection belongs
    // to consumers; paragraph-mark resolution must preserve both alternatives.
    if *name == crate::namespaces::MC::name("AlternateContent") {
        return Some(true);
    }
    if name.namespace_name() == crate::namespaces::M::URI {
        return Some(true);
    }
    // A revision kept tracked holds content the paragraph keeps; its move
    // range markers hold none.
    if name.namespace_name() == FROZEN_NS {
        return Some(!name.local_name().contains("Range"));
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
    if name == W::sdt() {
        let mut out = Vec::new();
        for sc in dom.elements(node, Some(&W::sdt_content())) {
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
        if n == M::name("oMath") || n == M::name("oMathPara") {
            // Word stores math revisions inside the equation. A.5 runs before
            // A.7 removes those internal revisions; probe that same transform
            // on this already-cloned CollapseTransform tree so a fully removed
            // equation cannot keep an otherwise deleted terminal pilcrow alive.
            // Authored empty equations, mixed owners and independent anchors
            // retain the ownership decisions of the ordinary math transform.
            return accept_all_other_revisions_transform(dom, ce)
                .into_iter()
                .any(|node| {
                    dom.name(node).is_some_and(|name| {
                        is_run_content(&name).unwrap_or_else(|| {
                            panic!("Internal error 20, found element {}", name.clark())
                        })
                    })
                });
        }
        is_run_content(&n)
            .unwrap_or_else(|| panic!("Internal error 20, found element {}", n.clark()))
    })
}

/// Does every row of `tbl` carry `trPr/w:del` (and is there a row)?
fn table_rows_all_deleted(dom: &Dom, tbl: NodeId) -> bool {
    let rows = wrapped_children(dom, tbl, &W::tr());
    !rows.is_empty()
        && rows.into_iter().all(|tr| {
            dom.element(tr, &W::tr_pr())
                .is_some_and(|pr| dom.element(pr, &W::del()).is_some())
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
                    .is_some_and(|n| n == W::del() || n == W::move_from())
            })
        })
}

/// ACCEPT-SKIP-A5: non-allocating presence of any paragraph whose mark is
/// deleted or moved-from under `root`.
fn has_deleted_or_moved_from_paragraph_mark(dom: &Dom, root: NodeId) -> bool {
    fn walk(dom: &Dom, id: NodeId) -> bool {
        if dom.name(id).as_ref() == Some(&W::p())
            && paragraph_mark_is_deleted_or_moved_from(dom, id)
        {
            return true;
        }
        let n = dom.child_count(id);
        for i in 0..n {
            if walk(dom, dom.child_at(id, i)) {
                return true;
            }
        }
        false
    }
    walk(dom, root)
}

/// A.5a — `AcceptDeletedAndMoveFromParagraphMarksTransform` (:2119): the
/// 3-state grouping machine over a container's block-content chain. A run of
/// deleted-mark paragraphs PLUS the immediately following normal paragraph
/// form one DeletedRange group, which merges into a single paragraph carrying
/// `g.Last()`'s pPr (:2271, the RP052 fix) and every member's collapsed
/// content; the merged paragraph is nuked when its content is entirely
/// deleted, its last member's mark is deleted or moved away (C#: deleted
/// only), and it is the container's last block content (or a table follows)
/// (:2276). Tables (and m:* block content)
/// bound groups and reset the state. FAITHful: the container rebuild keeps
/// only `w:tcPr` children + the chain elements + the body-level `sectPr`
/// (re-appended last); other non-chain children are dropped, and merged
/// paragraphs lose the original `w:p` attributes — exactly like the C#.
pub fn accept_deleted_and_move_from_paragraph_marks_transform(
    dom: &mut Dom,
    node: NodeId,
) -> NodeId {
    accept_paragraph_marks_with_owners(dom, node, &mut HashMap::new())
}

// Original paragraph owners are private reconstruction evidence. Keeping them
// in Rust avoids adding scratch attributes to the public transform's output.
fn accept_paragraph_marks_with_owners(
    dom: &mut Dom,
    node: NodeId,
    paragraph_owners: &mut HashMap<NodeId, Vec<NodeId>>,
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
            let tc = accept_paragraph_marks_with_owners(dom, c, paragraph_owners);
            dom.add(ne, tc);
        }
        if dom.name_is(node, &W::p()) {
            paragraph_owners.insert(ne, vec![node]);
        }
        return ne;
    }

    let body_sect_pr = if name == W::body() {
        dom.element(node, &W::sect_pr())
    } else {
        None
    };

    let chain = iterate_block_content_elements(dom, node);
    // (is_deleted_range, grouping_key) aligned with the chain.
    let mut infos: Vec<(bool, i32)> = Vec::with_capacity(chain.len());
    let mut current_key = 0i32;
    let mut state = 0u8; // 0 = non-deleted, 1 = in deleted, 2 = paragraph following
    // Does the deleted-mark run so far keep anything once its marks go?
    let mut run_brings = false;
    // Deleted marks Word keeps, each with the paragraph after it, whose
    // properties the kept paragraph takes.
    let mut held_marks: HashMap<NodeId, NodeId> = HashMap::new();
    for c in &chain {
        let this = c.this_block_content_element.unwrap();
        let tn = dom.name(this).unwrap();
        if tn == W::p() {
            let brings = !brings_nothing(dom, this);
            let mark_goes = paragraph_mark_is_deleted_or_moved_from(dom, this);
            let held = mark_goes
                && c.next_block_content_element.is_some_and(|next| {
                    dom.name_is(next, &W::p())
                        && holds_mark(dom, this, next, brings || (state == 1 && run_brings))
                });
            if held {
                held_marks.insert(this, c.next_block_content_element.unwrap());
            }
            if mark_goes && !held {
                match state {
                    0 | 2 => {
                        state = 1;
                        current_key += 1;
                        run_brings = brings;
                        infos.push((true, current_key));
                    }
                    _ => {
                        run_brings |= brings;
                        infos.push((true, current_key));
                    }
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
        } else if tn == W::tbl()
            && state == 1
            && c.next_block_content_element
                .is_some_and(|n| dom.name(n) == Some(W::p()))
            && table_rows_all_deleted(dom, this)
        {
            // Word: a table whose rows are all deleted vanishes on accept
            // and does not end the deleted-mark run, which goes on to join
            // the paragraph after it.
            infos.push((true, current_key));
        } else if tn == W::tbl() || tn.namespace_name() == M::URI {
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

    // Prebuild rebuilt block nodes, keyed by the original block element(s)
    // they replace. Deleted-range merges map many originals → one paragraph.
    // None = nuked empty deleted trailing para; markers_from_nuke are comment
    // anchors hoisted out of that discarded para so they still emit.
    let mut rebuilt: Vec<(Vec<NodeId>, Option<NodeId>, Vec<NodeId>)> = Vec::new();
    for (_key, group) in &grouped {
        if group[0].1.0 {
            // DeletedRange: merge into one paragraph.
            let last_this = group.last().unwrap().0.this_block_content_element.unwrap();
            let np = dom.new_element(W::p());
            if let Some(&next) = held_marks.get(&last_this) {
                if let Some(ppr) = held_mark_properties(dom, last_this, next) {
                    dom.add(np, ppr);
                }
            } else {
                for ppr in dom.elements(last_this, Some(&W::p_pr())) {
                    let c = dom.clone_subtree(ppr);
                    dom.add(np, c);
                }
            }
            let mut orig_ids = Vec::new();
            for z in group {
                let this = z.0.this_block_content_element.unwrap();
                orig_ids.push(this);
                if dom.name(this) == Some(W::tbl()) {
                    // A wholly deleted table absorbed into the run.
                    continue;
                }
                for collapsed in collapse_paragraph_transform(dom, this) {
                    dom.add(np, collapsed);
                }
            }
            paragraph_owners.insert(
                np,
                orig_ids
                    .iter()
                    .copied()
                    .filter(|&source| dom.name_is(source, &W::p()))
                    .collect(),
            );
            // Word also drops an emptied moved-away paragraph before a table
            // (its Reject All of 30f20e787b's moved heading).
            let last_mark_goes = paragraph_mark_is_deleted_or_moved_from(dom, last_this);
            let next = group.last().unwrap().0.next_block_content_element;
            // An opaque equation also ends the paragraph-mark merge chain.
            // A fully deleted carrier before that barrier has no surviving
            // mark or content to own an empty paragraph in the projection.
            let next_is_end_or_block_barrier = next.is_none()
                || next.is_some_and(|n| {
                    dom.name(n).is_some_and(|name| {
                        name == W::tbl() || name == M::name("oMath") || name == M::name("oMathPara")
                    })
                });
            if all_para_content_is_deleted(dom, np)
                && last_mark_goes
                && next_is_end_or_block_barrier
            {
                // Nuke empty deleted para, but keep comment anchors that lived
                // and bookmarks that lived inside its w:del runs (starts 9/10
                // between delText).
                let markers = hoist_range_markers_from(dom, np);
                rebuilt.push((orig_ids, None, markers));
            } else {
                rebuilt.push((orig_ids, Some(np), Vec::new()));
            }
        } else {
            for z in group {
                let this = z.0.this_block_content_element.unwrap();
                let rebuilt_name = dom.name(this).unwrap();
                let re = dom.new_element(rebuilt_name);
                for (an, av) in dom.attributes(this) {
                    dom.set_attribute_value(re, &an, Some(&av));
                }
                let held = held_marks.get(&this).copied();
                if let Some(next) = held
                    && let Some(ppr) = held_mark_properties(dom, this, next)
                {
                    dom.add(re, ppr);
                }
                for c in dom.nodes(this) {
                    if held.is_some() && dom.name_is(c, &W::p_pr()) {
                        continue;
                    }
                    let tc = accept_paragraph_marks_with_owners(dom, c, paragraph_owners);
                    dom.add(re, tc);
                }
                if dom.name_is(this, &W::p()) {
                    paragraph_owners.insert(re, vec![this]);
                }
                rebuilt.push((vec![this], Some(re), Vec::new()));
            }
        }
    }

    let ne = dom.new_element(name.clone());
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for e in dom.elements(node, Some(&W::tc_pr())) {
        let ce = dom.clone_subtree(e);
        dom.add(ne, ce);
    }
    // Emit in original element-child order so body-level commentRange*/bookmark*
    // between tables and paragraphs are preserved. The prior rebuild only kept
    // p/tbl chain members and dropped other body children — that deleted outer
    // nested commentRangeEnd after tables (ids 2/66) and broke comment carry.
    let mut emitted: HashSet<usize> = HashSet::new();
    for c in dom.elements(node, None) {
        let Some(cn) = dom.name(c) else {
            continue;
        };
        if cn == W::tc_pr() || cn == W::sect_pr() {
            continue;
        }
        if is_body_level_range_marker(&cn) {
            let clone = dom.clone_subtree(c);
            dom.add(ne, clone);
            continue;
        }
        // Direct block content (p/tbl child of body/cell).
        if let Some(ri) = rebuilt.iter().position(|(ids, _, _)| ids.contains(&c))
            && emitted.insert(ri)
        {
            let (_ids, rebuilt_node, markers) = &rebuilt[ri];
            if let Some(rebuilt_node) = rebuilt_node {
                dom.add(ne, *rebuilt_node);
            }
            for &m in markers {
                dom.add(ne, m);
            }
            continue;
        }
        // Nested block content under wrappers (w:sdt → sdtContent → p).
        // The chain walks into sdt-wrapped paragraphs, but emit used to only
        // match *direct* body children — so a surviving sdt's paragraph was
        // rebuilt then never attached (m28 a5b_mixed: left=0 SDTs). Emit any
        // rebuilt entry whose original id is under this child; nuked entries
        // (None) drop the fully-deleted sdt while A.5b re-wraps survivors.
        for (ri, (ids, rebuilt_node, markers)) in rebuilt.iter().enumerate() {
            if emitted.contains(&ri) {
                continue;
            }
            let nested = ids
                .iter()
                .any(|&id| id == c || dom.ancestors(id, None).into_iter().any(|a| a == c));
            if !nested {
                continue;
            }
            emitted.insert(ri);
            if let Some(rebuilt_node) = rebuilt_node {
                dom.add(ne, *rebuilt_node);
            }
            for &m in markers {
                dom.add(ne, m);
            }
        }
    }
    if let Some(sp) = body_sect_pr {
        let c = dom.clone_subtree(sp);
        dom.add(ne, c);
    }
    ne
}

/// Body/cell-level markers that must survive block-content rebuild (not p/tbl).
fn is_body_level_range_marker(name: &XName) -> bool {
    name.namespace_name() == W::URI
        && matches!(
            name.local_name(),
            "commentRangeStart"
                | "commentRangeEnd"
                | "bookmarkStart"
                | "bookmarkEnd"
                | "permStart"
                | "permEnd"
        )
}

fn is_inline_range_marker(name: &XName) -> bool {
    name.namespace_name() == W::URI
        && matches!(
            name.local_name(),
            "commentRangeStart"
                | "commentRangeEnd"
                | "bookmarkStart"
                | "bookmarkEnd"
                | "permStart"
                | "permEnd"
        )
}

/// Permission ranges follow Word's character-range rule, including paragraph
/// marks: discard both endpoints only when their nonempty span disappears.
/// Originally empty ranges have nothing to lose and remain in place.
fn drop_wholly_deleted_permissions(dom: &mut Dom, root: NodeId) {
    let (start, end) = (W::name("permStart"), W::name("permEnd"));
    let dropped = bookmarks::wholly_deleted_endpoints(dom, root, &start, &end);
    for endpoint in dropped {
        dom.remove(endpoint);
    }
}

/// Pull comment, bookmark and permission markers out of a subtree being discarded
/// (e.g. accepted `w:del`) so anchors between delText runs are not lost.
fn hoist_range_markers_from(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    for e in dom.descendants(node, None) {
        let Some(n) = dom.name(e) else {
            continue;
        };
        if is_inline_range_marker(&n) {
            out.push(dom.clone_subtree(e));
        }
    }
    out
}

// ─────────────── A.5b — content-control re-wrap after mark merge ────────────

/// A.5b — `AnnotateRunElementsWithId` (:1935): number every descendant `w:r`
/// with `pt:UniqueId` 0.. in document order (in place).
pub fn annotate_run_elements_with_id(dom: &mut Dom, element: NodeId) {
    let unique_id = PT::unique_id();
    for (run_id, r) in (0..).zip(dom.descendants(element, Some(&W::r()))) {
        dom.set_attribute_value(r, &unique_id, Some(&run_id.to_string()));
    }
}

/// Number every descendant `w:p` (`pt:UniqueId` `p0`, `p1`, …) so A.5b can
/// find a control's paragraphs when it has no run to anchor on (an empty
/// control keeps its paragraph mark, and so its place). Not in the C#, which
/// anchors on runs only and drops such a control whenever another paragraph
/// mark of the part is deleted.
fn annotate_paragraph_elements_with_id(dom: &mut Dom, element: NodeId) {
    let unique_id = PT::unique_id();
    for (paragraph_id, p) in (0..).zip(dom.descendants(element, Some(&W::p()))) {
        dom.set_attribute_value(p, &unique_id, Some(&format!("p{paragraph_id}")));
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
    let unique_id = PT::unique_id();
    let run_ids_name = PT::run_ids();
    let txbx = W::txbx_content();
    for (sdt_id, e) in (0..).zip(dom.descendants(element, Some(&W::sdt()))) {
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
    add_content_controls_with_owners(dom, new_document, original, None)
}

fn add_content_controls_with_owners(
    dom: &mut Dom,
    new_document: NodeId,
    original: NodeId,
    paragraph_owners: Option<&HashMap<NodeId, Vec<NodeId>>>,
) -> NodeId {
    use std::collections::HashSet;

    let sdt = W::sdt();
    let unique_id = PT::unique_id();
    let run_ids_name = PT::run_ids();

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

    let mut paragraph_by_id: HashMap<String, NodeId> = HashMap::new();
    for p in dom.descendants(new_document, Some(&W::p())) {
        if let Some(id) = dom.attribute(p, &unique_id) {
            paragraph_by_id.entry(id.to_string()).or_insert(p);
        }
    }
    let mut original_run_by_id: HashMap<String, NodeId> = HashMap::new();
    for r in dom.descendants(original, Some(&W::r())) {
        if let Some(id) = dom.attribute(r, &unique_id) {
            original_run_by_id.entry(id.to_string()).or_insert(r);
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
        // A run an earlier whole-paragraph wrap took out of the document (a
        // control nested in one already restored from its clone) is gone.
        // When every paragraph of the control goes, its deleted or moved-away
        // text does not anchor it either: a control holding nothing else goes
        // with its paragraphs, else its clone would replace the paragraph
        // their marks merged into. A control whose text alone is deleted
        // keeps its mark and so stays, emptied.
        let paragraphs = dom.descendants(cc, Some(&W::p()));
        let owned_paragraphs = paragraphs.iter().copied().collect::<HashSet<_>>();
        let block_removed = !paragraphs.is_empty()
            && paragraphs.iter().all(|&p| {
                dom.element(p, &W::p_pr())
                    .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                    .is_some_and(|rpr| {
                        dom.element(rpr, &W::del()).is_some()
                            || dom.element(rpr, &W::move_from()).is_some()
                    })
            });
        let mut runs_in_new_document: Vec<NodeId> = runs
            .iter()
            .filter_map(|id| run_by_id.get(id).copied())
            .filter(|&r| {
                let ancestors = dom.ancestors(r, None);
                ancestors.contains(&new_document)
                    && !(block_removed
                        && ancestors.iter().any(|&a| {
                            dom.name(a)
                                .is_some_and(|n| n == W::del() || n == W::move_from())
                        }))
            })
            .collect();
        // A paragraph the merge joined to this control's brings something to
        // the join when it keeps words or sits in a control of its own; one
        // that brings nothing goes, and leaves the control a block (Word 16,
        // the #1694 A/B).
        let cc_ancestors = dom.ancestors(cc, None);
        let brings = |dom: &Dom, source: NodeId| {
            !brings_nothing(dom, source)
                || dom
                    .ancestors(source, None)
                    .into_iter()
                    .any(|a| dom.name_is(a, &sdt) && a != cc && !cc_ancestors.contains(&a))
        };
        let foreign = |dom: &Dom, owners: &[NodeId]| {
            owners
                .iter()
                .any(|&source| !owned_paragraphs.contains(&source) && brings(dom, source))
        };
        // A control whose paragraphs all go keeps its place, emptied: inline
        // in the paragraph they join when that paragraph holds something
        // else, else a block around it.
        if block_removed
            && runs_in_new_document.is_empty()
            && let Some(paragraph_owners) = paragraph_owners
            && let Some(&last_own) = paragraphs.last()
            && let Some((&joined, owners)) = paragraph_owners.iter().find(|(joined, owners)| {
                owners.contains(&last_own)
                    && owners
                        .last()
                        .is_some_and(|last| !owned_paragraphs.contains(last))
                    && dom.ancestors(**joined, None).contains(&new_document)
            })
        {
            let after: Vec<NodeId> = owners
                .iter()
                .rposition(|source| owned_paragraphs.contains(source))
                .map(|last| owners[last + 1..].to_vec())
                .unwrap_or_default();
            let later = dom
                .descendants(joined, Some(&W::r()))
                .into_iter()
                .find(|&r| {
                    dom.attribute(r, &unique_id)
                        .and_then(|id| original_run_by_id.get(id))
                        .is_some_and(|&source| {
                            dom.ancestors(source, None)
                                .iter()
                                .any(|a| after.contains(a))
                        })
                })
                .and_then(|r| {
                    dom.ancestors_and_self(r, None)
                        .into_iter()
                        .find(|&c| dom.parent(c) == Some(joined))
                });
            let emptied = dom.new_element(sdt.clone());
            for (an, av) in dom.attributes(cc) {
                dom.set_attribute_value(emptied, &an, Some(&av));
            }
            let props: Vec<NodeId> = dom
                .elements(cc, None)
                .into_iter()
                .filter(|&e| dom.name(e) != Some(W::sdt_content()))
                .collect();
            let mut kids: Vec<NodeId> = props.into_iter().map(|e| dom.clone_subtree(e)).collect();
            let content = dom.new_element(W::sdt_content());
            kids.push(content);
            kids.sort_by_key(|&e| order_sdt(&dom.name(e).unwrap()));
            for e in kids {
                dom.add(emptied, e);
            }
            if !foreign(dom, owners) {
                dom.add_before_self(joined, emptied);
                dom.remove(joined);
                dom.add(content, joined);
                continue;
            }
            match later {
                Some(child) => dom.add_before_self(child, emptied),
                None => dom.add(joined, emptied),
            }
            continue;
        }
        // No run left to anchor it: the control stands on the paragraphs
        // that kept their marks, emptied. A paragraph nested in the
        // control's own content (a cell of the table it wraps) anchors it
        // through that content, else the control would be rebuilt inside
        // the table.
        if runs_in_new_document.is_empty() && !block_removed {
            let wrapper = |n: XName| n == sdt || n == W::sdt_content();
            let content = dom.element(cc, &W::sdt_content());
            let mut anchors: Vec<NodeId> = Vec::new();
            for &p in &paragraphs {
                let levels = dom
                    .ancestors(p, None)
                    .into_iter()
                    .take_while(|&a| Some(a) != content)
                    .filter(|&a| dom.name(a).is_some_and(|n| !wrapper(n)))
                    .count();
                let Some(mut anchor) = dom
                    .attribute(p, &unique_id)
                    .and_then(|id| paragraph_by_id.get(id).copied())
                    .filter(|&p| dom.ancestors(p, None).contains(&new_document))
                else {
                    continue;
                };
                let mut climbed = 0;
                while climbed < levels {
                    let Some(parent) = dom.parent(anchor).filter(|&a| a != new_document) else {
                        break;
                    };
                    if dom.name(parent).is_some_and(|n| !wrapper(n)) {
                        climbed += 1;
                    }
                    anchor = parent;
                }
                if !anchors.contains(&anchor) {
                    anchors.push(anchor);
                }
            }
            runs_in_new_document = anchors;
        }

        // A block control owns paragraphs/tables, not merely the runs that
        // happen to survive inside them. Restore that source ownership before
        // choosing the insertion level; a cell paragraph is not the owner of
        // a control whose sdtContent originally held the whole table.
        if let Some(content) = dom.element(cc, &W::sdt_content()) {
            let original_runs: HashMap<String, NodeId> = dom
                .descendants(cc, Some(&W::r()))
                .into_iter()
                .filter_map(|run| Some((dom.attribute(run, &unique_id)?.to_string(), run)))
                .collect();
            let mut anchors = Vec::new();
            for &run in &runs_in_new_document {
                let owned = dom
                    .attribute(run, &unique_id)
                    .and_then(|id| original_runs.get(id).copied())
                    .and_then(|source| {
                        let mut path = dom
                            .ancestors_and_self(source, None)
                            .into_iter()
                            .take_while(|&n| n != content)
                            .collect::<Vec<_>>();
                        path.reverse();
                        // Nested block SDTs are transparent on the source
                        // ownership path. Select their first actual block,
                        // then promote to its transformed counterpart. The
                        // outer control is restored first; inner controls
                        // subsequently wrap that same transformed payload.
                        let mut top = None;
                        for node in path {
                            let name = dom.name(node)?;
                            if name == W::sdt() || name == W::sdt_content() {
                                continue;
                            }
                            if name == W::p() || name == W::tbl() {
                                top = Some(node);
                            }
                            break;
                        }
                        let top = top?;
                        let name = dom.name(top)?;
                        let nested = dom
                            .ancestors(source, None)
                            .into_iter()
                            .take_while(|&n| n != top)
                            .filter(|&n| dom.name(n).as_ref() == Some(&name))
                            .count();
                        let owner = dom
                            .ancestors(run, None)
                            .into_iter()
                            .filter(|&n| dom.name(n).as_ref() == Some(&name))
                            .nth(nested)?;
                        if name == W::p() {
                            if let Some(paragraph_owners) = paragraph_owners {
                                // Every contributing source pilcrow must belong
                                // to this control, including runless successors.
                                let owners = paragraph_owners.get(&owner)?;
                                if foreign(dom, owners) {
                                    return None;
                                }
                            } else {
                                // The standalone public rewrap helper has no
                                // merge map. Only an unchanged paragraph ID can
                                // prove complete ownership; otherwise keep its
                                // established run-range reconstruction.
                                let owner_id = dom.attribute(owner, &unique_id)?;
                                if !paragraphs.iter().any(|&source| {
                                    dom.attribute(source, &unique_id) == Some(owner_id)
                                }) {
                                    return None;
                                }
                            }
                        }
                        Some(owner)
                    })
                    .unwrap_or(run);
                if !anchors.contains(&owned) {
                    anchors.push(owned);
                }
            }
            runs_in_new_document = anchors;
        }

        // deepest common ancestor of all the runs (nearest-first intersection)
        let Some(first_run) = runs_in_new_document.first().copied() else {
            continue;
        };
        let mut intersection: Vec<NodeId> = dom.ancestors(first_run, None);
        for &run in &runs_in_new_document[1..] {
            let anc: HashSet<NodeId> = dom.ancestors(run, None).into_iter().collect();
            intersection.retain(|a| anc.contains(a));
        }
        // A revision wrapper is no place for a block control: rebuilt inside
        // the `w:del` holding all its text, the control would go with it.
        let Some(&common_ancestor) = intersection.iter().find(|&&a| {
            dom.name(a).is_none_or(|n| {
                n != W::del() && n != W::ins() && n != W::move_from() && n != W::move_to()
            })
        }) else {
            continue;
        };

        let child_containing = |dom: &Dom, run: NodeId| -> NodeId {
            dom.ancestors_and_self(run, None)
                .into_iter()
                .find(|&c| dom.parent(c) == Some(common_ancestor))
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
        let foreign_pilcrow_owner = paragraph_owners
            .and_then(|owners| owners.get(&common_ancestor))
            .is_some_and(|owners| foreign(dom, owners));
        if dom.name(common_ancestor) == Some(W::p())
            && !foreign_pilcrow_owner
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
        let sdt_content = dom.new_element(W::sdt_content());
        for &e in &in_range {
            dom.add(sdt_content, e); // detached → moved, like the C#
        }
        let cc_props: Vec<NodeId> = dom
            .elements(cc, None)
            .into_iter()
            .filter(|&e| dom.name(e) != Some(W::sdt_content()))
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

/// Does `paragraph` (a source paragraph, before the mark merge) leave nothing
/// once its marks go: no run with content outside a deletion or a move away,
/// nothing else but its properties and markers? The markers are bookmarks,
/// proofing marks, a comment's range and its reference run, and permission
/// ranges. Anything not known to be empty brings something (#1694: only
/// these shapes were checked against Word, rounds 1 to 9).
fn brings_nothing(dom: &Dom, paragraph: NodeId) -> bool {
    dom.elements(paragraph, None).into_iter().all(|child| {
        let Some(name) = dom.name(child) else {
            return true;
        };
        name == W::p_pr()
            || name == W::del()
            || name == W::move_from()
            || name == W::bookmark_start()
            || name == W::bookmark_end()
            || name == W::name("proofErr")
            || name == W::name("commentRangeStart")
            || name == W::name("commentRangeEnd")
            || name == W::name("permStart")
            || name == W::name("permEnd")
            || (name == W::r()
                && dom.elements(child, None).into_iter().all(|e| {
                    dom.name_is(e, &W::r_pr()) || dom.name_is(e, &W::name("commentReference"))
                }))
    })
}

/// Does Word keep the deleted mark of paragraph `a` before `b`? It does
/// where the join would cross the edge of a block control holding more than
/// one paragraph, and the paragraph outside that control keeps something:
/// words (`a_keeps` for `a`, with the deleted-mark run before it) or a
/// control the other paragraph is not in. Such a control cannot become
/// inline, and Word does not take a paragraph out of it to join one outside
/// (the #1694 A/B: hn13, hn20, hn32–hn35, hn39, hn40). An outside paragraph
/// that keeps nothing goes, as at any join (hn23, hn24, hn31).
///
/// A control ending at `a` still holds more than one paragraph only while
/// one of its marks stays: once they all go, its paragraphs join the one
/// after it, and it can become inline there (round 9: k01, k06, k08).
fn holds_mark(dom: &Dom, a: NodeId, b: NodeId, a_keeps: bool) -> bool {
    let sdt = W::sdt();
    let around_a = dom.ancestors(a, Some(&sdt));
    let around_b = dom.ancestors(b, Some(&sdt));
    let own = |around: &[NodeId], other: &[NodeId]| around.iter().any(|s| !other.contains(s));
    let a_keeps = a_keeps || own(&around_a, &around_b);
    let b_keeps = !brings_nothing(dom, b) || own(&around_b, &around_a);
    let paragraphs = |s: NodeId| dom.descendants(s, Some(&W::p()));
    let multi = |s: NodeId| paragraphs(s).len() > 1;
    let keeps_a_mark = |s: NodeId| {
        paragraphs(s)
            .into_iter()
            .any(|p| !paragraph_mark_is_deleted_or_moved_from(dom, p))
    };
    (a_keeps && around_b.iter().any(|&s| !around_a.contains(&s) && multi(s)))
        || (b_keeps
            && around_a
                .iter()
                .any(|&s| !around_b.contains(&s) && multi(s) && keeps_a_mark(s)))
}

/// The properties of a paragraph whose deleted mark Word kept: those of the
/// paragraph after it, as if the two had joined (the #1694 A/B, hn41; style
/// and numbering included, round 9: c02). No mark of the next paragraph's
/// goes with them, nor its section break (c03); the kept paragraph's own
/// section break goes with the deleted mark that carried it (c04, e05).
fn held_mark_properties(dom: &mut Dom, held: NodeId, next: NodeId) -> Option<NodeId> {
    debug_assert!(dom.name_is(held, &W::p()));
    let ppr = dom.new_element(W::p_pr());
    if let Some(source) = dom.element(next, &W::p_pr()) {
        for child in dom.elements(source, None) {
            if dom.name_is(child, &W::sect_pr()) {
                continue;
            }
            let copy = dom.clone_subtree(child);
            if dom.name_is(copy, &W::r_pr()) {
                for mark in dom.elements(copy, None) {
                    if dom.name_is(mark, &W::del()) || dom.name_is(mark, &W::move_from()) {
                        dom.remove(mark);
                    }
                }
            }
            dom.add(ppr, copy);
        }
    }
    (!dom.elements(ppr, None).is_empty()).then_some(ppr)
}

/// A.5b — `AcceptDeletedAndMoveFromParagraphMarks` (:2098): annotate runs and
/// content controls (on the ORIGINAL, in place), run the A.5a transform, then
/// re-wrap the content controls the transform stripped.
///
/// ACCEPT-SKIP-A5: when no paragraph mark is `pPr/rPr/(del|moveFrom)`, the
/// A.5a grouping machine is a pure identity rebuild and annotate/rewrap do
/// nothing useful — transfer `element` without cloning.
pub fn accept_deleted_and_move_from_paragraph_marks(dom: &mut Dom, element: NodeId) -> NodeId {
    // ACCEPT-SKIP-A5: no deleted/moved-from paragraph marks → identity.
    if !has_deleted_or_moved_from_paragraph_mark(dom, element) {
        return element;
    }
    annotate_run_elements_with_id(dom, element);
    annotate_paragraph_elements_with_id(dom, element);
    annotate_content_controls_with_run_ids(dom, element);
    let mut paragraph_owners = HashMap::new();
    let new_element = accept_paragraph_marks_with_owners(dom, element, &mut paragraph_owners);
    add_content_controls_with_owners(dom, new_element, element, Some(&paragraph_owners))
}

// ─────────────── A.6 — rows left empty by moveFrom ──────────────────────────

/// `BlockLevelElements` (:2766) — the direct-cell-child names that make a
/// cell non-empty for [`remove_rows_left_empty_by_move_from`].
fn a6_block_level_elements() -> [XName; 8] {
    [
        W::p(),
        W::tbl(),
        W::sdt(),
        W::del(),
        W::ins(),
        M::name("oMath"),
        M::name("oMathPara"),
        W::move_to(),
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
    if name == W::tr() {
        let block = a6_block_level_elements();
        let non_empty_cells = dom.elements(node, Some(&W::tc())).into_iter().any(|tc| {
            dom.elements(tc, None)
                .into_iter()
                .any(|tcc| dom.name(tcc).is_some_and(|n| block.contains(&n)))
        });
        if !non_empty_cells {
            return None;
        }
    }
    let had_rows = name == W::tbl() && dom.element(node, &W::tr()).is_some();
    let ne = dom.new_element(name);
    for (an, av) in dom.attributes(node) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    for c in dom.nodes(node) {
        if let Some(tc) = remove_rows_left_empty_by_move_from_inner(dom, c) {
            dom.add(ne, tc);
        }
    }
    // A table this pass emptied of rows goes with them: the earlier pass
    // emptied the cells of a table whose rows were all deleted, and a
    // rowless `w:tbl` is no table Word writes.
    if had_rows && dom.element(ne, &W::tr()).is_none() {
        return None;
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
///
/// ACCEPT-SKIP-A7: when the subtree has no `w:cellDel`, transfer `node`
/// without a full-tree identity rebuild (common path for redlines that only
/// carry ins/del).
pub fn accept_deleted_cells_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    let cell_del = W::cell_del();
    // ACCEPT-SKIP-A7: no cellDel → identity (keep parent links).
    if !element_or_desc_has_name(dom, node, &cell_del) {
        return node;
    }
    if !dom.is_element(node) {
        return node;
    }
    let name = dom.name(node).unwrap();
    if name != W::tr() {
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

    let tc_name = W::tc();
    let cell_del = W::cell_del();
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

    let tr = dom.new_element(W::tr());
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
        let tcpr_name = W::tc_pr();
        let grid_span_name = W::grid_span();
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

/// R8: the `tblPrEx`-able properties of `member` that `first` (whose
/// `tblPr` the merged table keeps) does not share.
fn own_tbl_pr_ex(dom: &Dom, first: NodeId, member: NodeId) -> Vec<NodeId> {
    let ex_of = |t: NodeId| -> Vec<NodeId> {
        dom.element(t, &W::tbl_pr())
            .map(|pr| {
                dom.elements(pr, None)
                    .into_iter()
                    .filter(|&c| tbl_pr_ex_rank(dom, c).is_some())
                    .collect()
            })
            .unwrap_or_default()
    };
    let shared: HashSet<String> = ex_of(first)
        .into_iter()
        .map(|c| dom.serialize_element(c))
        .collect();
    ex_of(member)
        .into_iter()
        .filter(|&c| !shared.contains(&dom.serialize_element(c)))
        .collect()
}

/// R8: row `tr`'s `tblPrEx` with its table's `own` properties under the
/// row's own, in schema order.
fn row_tbl_pr_ex(dom: &mut Dom, tr: NodeId, own: &[NodeId]) -> NodeId {
    let row_ex: Vec<NodeId> = dom
        .element(tr, &W::name("tblPrEx"))
        .map(|ex| dom.elements(ex, None))
        .unwrap_or_default();
    let mut kids: Vec<NodeId> = row_ex.clone();
    for &c in own {
        if !row_ex.iter().any(|&r| dom.name(r) == dom.name(c)) {
            kids.push(c);
        }
    }
    kids.sort_by_key(|&c| tbl_pr_ex_rank(dom, c).unwrap_or(TBL_PR_EX.len()));
    let ex = dom.new_element(W::name("tblPrEx"));
    for c in kids {
        let cc = dom.clone_subtree(c);
        dom.add(ex, cc);
    }
    ex
}

/// Matching children in order, through wrappers but not into nested tables.
fn wrapped_children(dom: &Dom, parent: NodeId, name: &XName) -> Vec<NodeId> {
    let mut children = Vec::new();
    for child in dom.elements(parent, None) {
        if dom.name_is(child, name) {
            children.push(child);
        } else if dom.name(child) == Some(W::sdt())
            || dom.name(child) == Some(W::sdt_content())
            || dom.name(child) == Some(W::custom_xml())
        {
            children.extend(wrapped_children(dom, child, name));
        }
    }
    children
}

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
    for tr in wrapped_children(dom, new_tbl, &W::tr()) {
        let mut last_used: i64 = -1;
        for tc in wrapped_children(dom, tr, &W::tc()) {
            // Singular tcPr / tcW / gridSpan — avoid multi-Vec flat_map scans.
            let tc_w = dom
                .element(tc, &W::tc_pr())
                .and_then(|p| dom.element(p, &W::name("tcW")))
                .filter(|&w| dom.attribute(w, &W::name("w")).is_some());
            let Some(tc_w) = tc_w else { continue };
            let grid_span: i64 = dom
                .element(tc, &W::tc_pr())
                .and_then(|p| dom.element(p, &W::grid_span()))
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
        if element_or_desc_has_name(dom, tbl, &W::name(tag)) {
            return true;
        }
    }
    false
}

/// Which runs of adjacent tables A.8 merges.
#[derive(Clone, Copy, PartialEq, Eq)]
enum TableMerge {
    /// M112 (compare): a run with a revision-marked member.
    Revised,
    /// Word's save after Accept / Reject All: a run of tables alike in every
    /// property only a whole table carries (R8).
    WordSave,
}

/// The `tblPr` children a row can override through `w:tblPrEx`
/// (`CT_TblPrExBase`), in schema order.
const TBL_PR_EX: [&str; 9] = [
    "tblW",
    "jc",
    "tblCellSpacing",
    "tblInd",
    "tblBorders",
    "shd",
    "tblLayout",
    "tblCellMar",
    "tblLook",
];

fn tbl_pr_ex_rank(dom: &Dom, e: NodeId) -> Option<usize> {
    let n = dom.name(e)?;
    (n.namespace_name() == W::URI)
        .then(|| TBL_PR_EX.iter().position(|&l| l == n.local_name()))
        .flatten()
}

/// A table's whole-table properties (its `tblPr` minus what `tblPrEx` can
/// carry), serialized: Word joins adjacent tables whose keys agree. A grid
/// change still tracked (a selective resolution kept it) is the table's own,
/// so it keys the table apart.
fn whole_table_key(dom: &Dom, tbl: NodeId) -> String {
    let mut key = String::from("tbl");
    if let Some(pr) = dom.element(tbl, &W::tbl_pr()) {
        for c in dom.elements(pr, None) {
            if tbl_pr_ex_rank(dom, c).is_none() {
                key.push_str(&dom.serialize_element(c));
            }
        }
    }
    for grid in dom.elements(tbl, Some(&W::name("tblGrid"))) {
        for change in dom.elements(grid, Some(&W::name("tblGridChange"))) {
            key.push_str(&dom.serialize_element(change));
        }
    }
    key
}

/// The grouping key of A.8 for `e` (empty: not a table).
fn table_merge_key(dom: &Dom, e: NodeId, mode: TableMerge) -> String {
    if dom.name(e) != Some(W::tbl()) {
        return String::new();
    }
    if mode == TableMerge::WordSave {
        return whole_table_key(dom, e);
    }
    let bidi = dom
        .elements(e, Some(&W::tbl_pr()))
        .into_iter()
        .any(|p| dom.element(p, &W::name("bidiVisual")).is_some());
    if bidi {
        "tbl|bidiVisual".to_string()
    } else {
        "tbl".to_string()
    }
}

/// True if any element under `root` has ≥2 adjacent direct `w:tbl` children
/// that A.8 merges in `mode`.
fn subtree_needs_adjacent_table_merge(dom: &Dom, root: NodeId, mode: TableMerge) -> bool {
    fn walk(dom: &Dom, id: NodeId, mode: TableMerge) -> bool {
        if !dom.is_element(id) {
            return false;
        }
        // Scan direct element children for adjacent tbl runs.
        let kids = dom.elements(id, None);
        let mut prev_key = String::new();
        let mut run_has_rev = false;
        for &k in &kids {
            let key = table_merge_key(dom, k, mode);
            if key.is_empty() {
                prev_key = key;
                run_has_rev = false;
                continue;
            }
            let joins = key == prev_key;
            match mode {
                TableMerge::WordSave if joins => return true,
                TableMerge::WordSave => {}
                TableMerge::Revised => {
                    run_has_rev = (joins && run_has_rev) || table_has_revision_marks(dom, k);
                    if joins && run_has_rev {
                        return true;
                    }
                }
            }
            prev_key = key;
        }
        kids.into_iter().any(|k| walk(dom, k, mode))
    }
    walk(dom, root, mode)
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
///
/// ACCEPT-SKIP-A8: when no mergeable adjacent revision-bearing table group
/// exists anywhere under `node`, transfer without a full-tree rebuild.
pub fn merge_adjacent_tables_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    merge_adjacent_tables(dom, node, TableMerge::Revised)
}

/// R8 — Word's save after Accept / Reject All: adjacent tables alike in every
/// whole-table property are one table in Word's model (72cc9f4ac6,
/// 3d4318d7e9); a later member's own `tblPrEx`-able properties ride on its
/// rows. Identity when nothing merges.
fn merge_adjacent_tables_like_word(dom: &mut Dom, node: NodeId) -> NodeId {
    merge_adjacent_tables(dom, node, TableMerge::WordSave)
}

fn merge_adjacent_tables(dom: &mut Dom, node: NodeId, mode: TableMerge) -> NodeId {
    // ACCEPT-SKIP-A8: nothing to merge → identity (keep parent links).
    if !subtree_needs_adjacent_table_merge(dom, node, mode) {
        return node;
    }
    if !dom.is_element(node) {
        return node;
    }
    let name = dom.name(node).unwrap();
    let tbl_name = W::tbl();
    if dom.element(node, &tbl_name).is_none() {
        let ne = dom.new_element(name);
        for (an, av) in dom.attributes(node) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        for c in dom.nodes(node) {
            let tc = merge_adjacent_tables(dom, c, mode);
            dom.add(ne, tc);
        }
        return ne;
    }

    let children = dom.elements(node, None);
    let grouped = crate::util::group_adjacent(children, |&e| table_merge_key(dom, e, mode));

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
        if mode == TableMerge::Revised && !group.iter().any(|&t| table_has_revision_marks(dom, t)) {
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
        for pr in dom.elements(group[0], Some(&W::tbl_pr())) {
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
            let own_ex = if mode == TableMerge::WordSave && tbl != group[0] {
                own_tbl_pr_ex(dom, group[0], tbl)
            } else {
                Vec::new()
            };
            let fixed = fix_widths(dom, tbl);
            for tr in wrapped_children(dom, fixed, &W::tr()) {
                let new_row = dom.new_element(W::tr());
                for (an, av) in dom.attributes(tr) {
                    dom.set_attribute_value(new_row, &an, Some(&av));
                }
                if !own_ex.is_empty() {
                    let ex = row_tbl_pr_ex(dom, tr, &own_ex);
                    dom.add(new_row, ex);
                }
                for property in [W::name("tblPrEx"), W::tr_pr()] {
                    if !own_ex.is_empty() && property == W::name("tblPrEx") {
                        continue;
                    }
                    for e in dom.elements(tr, Some(&property)) {
                        let c = dom.clone_subtree(e);
                        dom.add(new_row, c);
                    }
                }
                for child in dom.elements(tr, None) {
                    if dom.name(child) == Some(W::name("tblPrEx"))
                        || dom.name(child) == Some(W::tr_pr())
                    {
                        continue;
                    }
                    let c = dom.clone_subtree(child);
                    dom.add(new_row, c);
                }
                let mut width_to_left = 0i64;
                for tc in wrapped_children(dom, new_row, &W::tc()) {
                    let w: Option<i64> = dom
                        .element(tc, &W::tc_pr())
                        .and_then(|p| dom.element(p, &W::name("tcW")))
                        .and_then(|t| dom.attribute(t, &W::name("w")))
                        .and_then(|v| v.parse().ok());
                    let Some(w) = w else { continue };
                    let cell_start = width_to_left;
                    width_to_left += w;
                    // rolled_pairs = [0] ++ rolled; start = first >= cell_start
                    let rolled_pairs: Vec<i64> =
                        std::iter::once(0).chain(rolled.iter().copied()).collect();
                    let start = rolled_pairs.iter().position(|&gv| gv >= cell_start);
                    let Some(start_idx) = start else { continue };
                    let start_value = rolled_pairs[start_idx];
                    let grids_required = rolled_pairs[start_idx..]
                        .iter()
                        .take_while(|&&gv| gv - start_value < w)
                        .count() as i64;

                    let mut tcpr_kids: Vec<NodeId> = Vec::new();
                    let props: Vec<NodeId> = dom
                        .elements(tc, Some(&W::tc_pr()))
                        .into_iter()
                        .flat_map(|p| dom.elements(p, None))
                        .filter(|&e| dom.name(e) != Some(W::grid_span()))
                        .collect();
                    for e in props {
                        tcpr_kids.push(dom.clone_subtree(e));
                    }
                    if grids_required != 1 {
                        let gs = dom.new_element(W::grid_span());
                        dom.set_attribute_value(gs, &W::val(), Some(&grids_required.to_string()));
                        tcpr_kids.push(gs);
                    }
                    tcpr_kids.sort_by_key(|&e| order_tc_pr(&dom.name(e).unwrap()));
                    let ordered_tc_pr = dom.new_element(W::tc_pr());
                    for e in tcpr_kids {
                        dom.add(ordered_tc_pr, e);
                    }
                    let new_cell = dom.new_element(W::tc());
                    dom.add(new_cell, ordered_tc_pr);
                    let body_kids: Vec<NodeId> = dom
                        .elements(tc, None)
                        .into_iter()
                        .filter(|&e| dom.name(e) != Some(W::tc_pr()))
                        .collect();
                    for e in body_kids {
                        let c = dom.clone_subtree(e);
                        dom.add(new_cell, c);
                    }
                    dom.replace_with(tc, &[new_cell]);
                }
                dom.replace_with(tr, &[new_row]);
            }
            for child in dom.elements(fixed, None) {
                if dom.name(child) == Some(W::tbl_pr())
                    || dom.name(child) == Some(W::name("tblGrid"))
                {
                    continue;
                }
                let c = dom.clone_subtree(child);
                dom.add(new_table, c);
            }
        }
        dom.add(ne, new_table);
    }
    ne
}

// ─────────────── A.9 — empty paragraph in empty cells ───────────────────────

/// True if any `w:tc` under `root` has no element children other than `w:tcPr`
/// (the A.9 empty-cell predicate). Non-allocating DFS for ACCEPT-SKIP-02.
fn has_empty_table_cell(dom: &Dom, root: NodeId) -> bool {
    let tc = W::tc();
    let tcpr = W::tc_pr();
    fn walk(dom: &Dom, id: NodeId, tc: &XName, tcpr: &XName) -> bool {
        if let Some(name) = dom.name(id)
            && name == *tc
        {
            // empty = no element child other than w:tcPr (incl. zero children)
            let mut only_tcpr = true;
            for i in 0..dom.child_count(id) {
                let c = dom.child_at(id, i);
                if dom.is_element(c) && dom.name(c).is_some_and(|n| n != *tcpr) {
                    only_tcpr = false;
                    break;
                }
            }
            if only_tcpr {
                return true;
            }
        }
        for i in 0..dom.child_count(id) {
            if walk(dom, dom.child_at(id, i), tc, tcpr) {
                return true;
            }
        }
        false
    }
    walk(dom, root, &tc, &tcpr)
}

/// A.9 — `AddEmptyParagraphToAnyEmptyCells` (:1448): a `w:tc` with no element
/// children other than `w:tcPr` gains an empty `w:p`.
///
/// ACCEPT-INPLACE-A9: mutate empty cells in place (append `w:p`) instead of
/// rebuilding the entire subtree. Returns the same `node` root (callers that
/// rebind `e = add_empty...(dom, e)` stay correct).
pub fn add_empty_paragraph_to_any_empty_cells(dom: &mut Dom, node: NodeId) -> NodeId {
    if !dom.is_element(node) {
        return node;
    }
    let tc = W::tc();
    let tcpr = W::tc_pr();
    // Collect empty cells first — cannot mutate while walking.
    let mut empty: Vec<NodeId> = Vec::new();
    collect_empty_table_cells(dom, node, &tc, &tcpr, &mut empty);
    for cell in empty {
        let p = dom.new_element(W::p());
        dom.add(cell, p);
    }
    node
}

fn collect_empty_table_cells(
    dom: &Dom,
    id: NodeId,
    tc: &XName,
    tcpr: &XName,
    out: &mut Vec<NodeId>,
) {
    if let Some(name) = dom.name(id)
        && name == *tc
    {
        let mut only_tcpr = true;
        for i in 0..dom.child_count(id) {
            let c = dom.child_at(id, i);
            if dom.is_element(c) && dom.name(c).is_some_and(|n| n != *tcpr) {
                only_tcpr = false;
                break;
            }
        }
        if only_tcpr {
            out.push(id);
        }
    }
    for i in 0..dom.child_count(id) {
        let c = dom.child_at(id, i);
        if dom.is_element(c) {
            collect_empty_table_cells(dom, c, tc, tcpr, out);
        }
    }
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
    accept_content(dom, root, false, &mut Vec::new())
}

fn accept_content(
    dom: &mut Dom,
    root: NodeId,
    keep_comments: bool,
    kept: &mut Vec<String>,
) -> NodeId {
    let has_rev = element_has_tracked_revisions(dom, root);
    let e = remove_rsid_transform(dom, root).expect("root not dropped by rsid removal");
    let e = if has_rev {
        // O.0: normalize orphan move elements first — the comment/bookmark
        // prepasses and A.2/A.4 all treat matched moveFrom-range content as
        // deleted, so orphan `moveTo`/`moveFrom` elements must be `w:ins`/
        // `w:del` before any of them runs (#1235). No range marker is
        // dropped here: Word deletes an orphan moveFrom range's unwrapped
        // content on Accept (#1243), which is A.2's own doing, and A.10
        // strips every leftover marker afterwards.
        let e = normalize_orphan_moves(dom, e, OrphanMovePass::AcceptElements);
        if keep_comments {
            kept.extend(comments::keep_wholly_deleted_comments(dom, e));
        }
        comments::drop_wholly_deleted_comments(dom, e);
        bookmarks::drop_wholly_deleted_bookmarks(dom, e);
        drop_wholly_deleted_permissions(dom, e);
        let e = fix_up_deleted_or_inserted_field_codes_transform(dom, e);
        let contains_move_from = !dom.descendants(e, Some(&W::move_from())).is_empty();
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
        let e = merge_adjacent_tables_transform(dom, e);
        bookmarks::drop_unpaired_bookmarks(dom, e);
        e
    } else {
        e
    };
    // ACCEPT-SKIP-02: A.9 is a full-tree rebuild; skip when no empty cells.
    let e = if has_empty_table_cell(dom, e) {
        add_empty_paragraph_to_any_empty_cells(dom, e)
    } else {
        e
    };
    // #1223: a `w:comment` must keep at least one `w:p`. Rejecting a
    // comment's only insertion or accepting a wholly deleted one can empty
    // it; Word writes an empty `w:p` there.
    if dom.name(e) == Some(W::name("comments")) {
        ensure_comment_paragraphs(dom, e);
    }

    // Strip PT.UniqueId / PT.RunIds attributes from all descendants.
    let unique_id = PT::unique_id();
    let run_ids = PT::run_ids();
    for d in dom.descendants_and_self(e, None) {
        dom.set_attribute_value(d, &unique_id, None);
        dom.set_attribute_value(d, &run_ids, None);
    }
    // Remove empty w:numPr elements.
    let num_pr = W::num_pr();
    for np in dom.descendants(e, Some(&num_pr)) {
        if !dom.has_elements(np) {
            dom.remove(np);
        }
    }
    e
}

// ─────────────── A.11 — package-scope accept / reject ───────────────────────

/// A property element `live` reverting to its saved copy `reverted` keeps
/// the revisions left tracked in it (frozen by a selective resolution): a
/// paragraph mark's insertion or deletion ahead of the run properties
/// (`first`), a row's or a cell's marks after the rest.
fn carry_kept_revisions(dom: &mut Dom, live: NodeId, reverted: NodeId, first: bool) {
    let kept: Vec<NodeId> = dom
        .elements(live, None)
        .into_iter()
        .filter(|&e| dom.name(e).is_some_and(|n| n.namespace_name() == FROZEN_NS))
        .collect();
    let kept: Vec<NodeId> = kept.into_iter().map(|e| dom.clone_subtree(e)).collect();
    if first {
        for c in kept.into_iter().rev() {
            dom.add_first(reverted, c);
        }
    } else {
        for c in kept {
            dom.add(reverted, c);
        }
    }
}

/// Give every `w:comment` under a `w:comments` root without a block child
/// (`w:p`, `w:tbl`, `w:sdt`, `w:customXml`) an empty paragraph (#1223).
fn ensure_comment_paragraphs(dom: &mut Dom, root: NodeId) {
    let block = [W::p(), W::tbl(), W::sdt(), W::custom_xml()];
    for comment in dom.elements(root, Some(&W::name("comment"))) {
        if !dom
            .elements(comment, None)
            .into_iter()
            .any(|c| dom.name(c).is_some_and(|n| block.contains(&n)))
        {
            let p = dom.new_element(W::p());
            dom.add(comment, p);
        }
    }
}

/// A.11 — `AcceptRevisionsForStylesTransform` (:1300): drop `pPrChange`/
/// `rPrChange` from the styles part, rebuild the rest.
fn accept_revisions_for_styles_transform(dom: &mut Dom, node: NodeId) -> Option<NodeId> {
    if !dom.is_element(node) {
        return Some(dom.clone_subtree(node));
    }
    let name = dom.name(node).unwrap();
    if name == W::p_pr_change() || name == W::r_pr_change() {
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
        && let Some(chg) = dom.element(node, &W::p_pr_change())
    {
        let inner = dom.element(chg, &W::p_pr());
        return inner.and_then(|i| reject_revisions_for_styles_transform(dom, i));
    }
    if name == W::r_pr()
        && let Some(chg) = dom.element(node, &W::r_pr_change())
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

/// The part `rel` of the main part targets (`numbering`, `comments`),
/// resolved against the main part; `None` when the package has none.
fn related_part(pkg: &crate::opc::PartFs, rel: &str) -> Option<String> {
    let main = pkg.main_document_part()?;
    let rels = pkg.read_rels_for(&main)?;
    rels.items
        .iter()
        .find(|r| {
            r.target_mode.as_deref() != Some("External")
                && r.rel_type.rsplit('/').next() == Some(rel)
        })
        .map(|r| pkg.resolve_rel_target(&main, &r.target))
}

/// The main document's numbering part, when it has one.
fn numbering_part(pkg: &crate::opc::PartFs) -> Option<String> {
    related_part(pkg, "numbering")
}

/// The main document's comments part (`word/comments.xml`), when it has
/// one. Its revisions resolve like every other part's, but they are not
/// listed, so `changes` skips this part.
pub(crate) fn comments_part(pkg: &crate::opc::PartFs) -> Option<String> {
    related_part(pkg, "comments")
}

/// The parts `AcceptRevisions`/`RejectRevisions` (:1277/:31) walk, in the C#
/// order — main, headers, footers, endnotes, footnotes, then styles
/// (flagged) — with the comments part Word's own Accept/Reject All also
/// settles (#1223) between the notes and the styles.
pub(crate) fn revision_bearing_parts(pkg: &crate::opc::PartFs) -> Vec<(String, bool)> {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let mut headers = Vec::new();
    let mut footers = Vec::new();
    let mut endnotes = Vec::new();
    let mut footnotes = Vec::new();
    let mut comments = Vec::new();
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
                "comments" => &mut comments,
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
        .chain(comments)
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
    resolve_package(pkg, Resolution::Accept, None);
}

/// A.11 — `RejectRevisions` (:31) at package scope: per content part, the
/// revert → reverse → rsid-strip → full-accept composition (the C# phases the
/// same steps across all parts; parts are independent, so per-part composition
/// is equivalent); the styles part reverts its property changes then accepts
/// the leftovers.
pub fn reject_revisions_package(pkg: &mut crate::opc::PartFs) {
    resolve_package(pkg, Resolution::Reject, None);
}

/// Revision elements renamed into this namespace sit a resolution out and
/// come back unchanged ([`crate::changes`]).
pub(crate) const FROZEN_NS: &str = "urn:jubarte:frozen-revision";

/// Package-wide ids reserved before any revision is frozen or removed.
/// Annotation ids and XML parts outside the revision pipeline have other scopes.
pub(crate) struct RevisionIds {
    next: u64,
    used: HashSet<u64>,
}

impl RevisionIds {
    pub(crate) fn new(pkg: &crate::opc::PartFs) -> Self {
        let mut used = HashSet::new();
        for (part, _) in revision_bearing_parts(pkg) {
            let Some(xml) = pkg.part_string(&part) else {
                continue;
            };
            let mut dom = Dom::new();
            let doc = dom.parse_xdocument(&xml);
            let Some(root) = dom.root(doc) else { continue };
            for node in dom.descendants_and_self(root, None) {
                if dom
                    .name(node)
                    .is_some_and(|name| crate::validate::has_unique_revision_id(&name))
                    && let Some(id) = dom
                        .attribute(node, &W::id())
                        .and_then(|id| id.parse::<u64>().ok())
                {
                    used.insert(id);
                }
            }
        }
        Self::from_used(used)
    }

    pub(crate) fn from_used(used: HashSet<u64>) -> Self {
        let next = used.iter().max().map_or(0, |id| id.saturating_add(1));
        Self { next, used }
    }

    /// Keep max + 1 until Word's signed 32-bit range is exhausted, then
    /// allocate the lowest unused id. Reserve copies immediately, too.
    pub(crate) fn allocate(&mut self) -> u64 {
        const MAX: u64 = i32::MAX as u64;
        if self.next > MAX {
            self.next = 0;
        }
        while self.used.contains(&self.next) {
            self.next += 1;
        }
        assert!(self.next <= MAX, "revision id space exhausted");
        let id = self.next;
        self.used.insert(id);
        self.next += 1;
        id
    }
}

/// A copied property history is a distinct record. Allocate at the copy
/// site, leaving the original's attribution, payload and id untouched.
pub(crate) fn clone_with_fresh_property_ids(
    dom: &mut Dom,
    source: NodeId,
    next_id: &mut RevisionIds,
) -> NodeId {
    let copy = dom.clone_subtree(source);
    refresh_property_ids(dom, copy, |_| next_id.allocate());
    copy
}

/// Give every property history under `copy`, `copy` included, an id from
/// `allocate`, which is passed the id the record had, leaving its
/// attribution and payload untouched: a copy of a record is a distinct
/// record.
pub(crate) fn refresh_property_ids(
    dom: &mut Dom,
    copy: NodeId,
    mut allocate: impl FnMut(Option<u64>) -> u64,
) {
    for node in std::iter::once(copy).chain(dom.descendants(copy, None)) {
        if dom.name(node).is_some_and(|name| {
            matches!(name.namespace_name(), W::URI | FROZEN_NS)
                && matches!(name.local_name(), "rPrChange" | "pPrChange")
        }) {
            let old = dom
                .attribute(node, &W::id())
                .and_then(|id| id.parse::<u64>().ok());
            dom.set_attribute_value(node, &W::id(), Some(&allocate(old).to_string()));
        }
    }
}

/// Called with each revision-bearing part's name and parsed root before it
/// is resolved; renames the revisions to keep into [`FROZEN_NS`].
pub(crate) type Freeze<'a> = &'a dyn Fn(&str, &mut Dom, NodeId);

/// Accept or reject every revision `freeze` leaves in place. Bookmark and
/// comment ids are identifiers: the ones kept keep their ids, whatever is
/// left (#1264). Return the removed comments' original ids.
pub(crate) fn resolve_package(
    pkg: &mut crate::opc::PartFs,
    resolution: Resolution,
    freeze: Option<Freeze<'_>>,
) -> Vec<String> {
    resolve_with_comments(pkg, resolution, freeze, false, &mut Vec::new())
}

/// [`resolve_package`] for an edit plan, the only entry point that can
/// rescue removed anchors.
pub(crate) fn resolve_package_for_edit(
    pkg: &mut crate::opc::PartFs,
    resolution: Resolution,
    freeze: Option<Freeze<'_>>,
    keep_comments: bool,
) -> (Vec<String>, Vec<String>) {
    let mut kept = Vec::new();
    let removed = resolve_with_comments(pkg, resolution, freeze, keep_comments, &mut kept);
    (removed, kept)
}

fn resolve_with_comments(
    pkg: &mut crate::opc::PartFs,
    resolution: Resolution,
    freeze: Option<Freeze<'_>>,
    keep_comments: bool,
    kept_comments: &mut Vec<String>,
) -> Vec<String> {
    let parts = revision_bearing_parts(pkg);
    let mut next_id = RevisionIds::new(pkg);
    let levels = numbering_part(pkg)
        .and_then(|p| pkg.part_string(&p))
        .map(|xml| style_records::Levels::parse(&xml))
        .unwrap_or_default();
    // #1223 settles revisions inside comment bodies. A comments part with
    // none is neither settled nor tidied, as before #1223, so a redline
    // carries it as Word's does (#1467); a comment whose anchors are gone
    // is still pruned from it.
    let comments = comments_part(pkg);
    let mut untouched_comments = None;
    for (part, is_styles) in parts.clone() {
        let is_comments = comments.as_ref() == Some(&part);
        process_part(pkg, &part, |dom, root| {
            if is_comments && !element_has_tracked_revisions(dom, root) {
                untouched_comments = Some(part.clone());
                return None;
            }
            if let Some(freeze) = freeze {
                freeze(&part, dom, root);
            }
            let resolved = match (is_styles, resolution) {
                (true, Resolution::Accept) => accept_revisions_for_styles_transform(dom, root),
                (true, Resolution::Reject) => {
                    let recorded = style_records::recorded_blocks(dom, root);
                    let rejected = reject_revisions_for_styles_transform(dom, root)?;
                    style_records::restore_against_built_ins(
                        dom,
                        rejected,
                        &recorded,
                        &levels,
                        &mut next_id,
                    );
                    accept_revisions_for_styles_transform(dom, rejected)
                }
                (false, _) => {
                    sections::carry_vanishing_section_references(dom, root, resolution);
                    Some(match resolution {
                        Resolution::Accept => {
                            accept_content(dom, root, keep_comments, kept_comments)
                        }
                        Resolution::Reject => {
                            reject_content(dom, root, keep_comments, kept_comments)
                        }
                    })
                }
            }?;
            thaw(dom, resolved);
            Some(resolved)
        });
    }
    let stories = story_parts(&parts);
    let removed_comments = comments::prune_orphan_comments(pkg, &stories);
    notes::prune_orphan_notes(pkg, &stories);
    let tidied: Vec<String> = stories
        .into_iter()
        .filter(|part| untouched_comments.as_ref() != Some(part))
        .collect();
    word_save::tidy(pkg, &tidied);
    removed_comments
}

/// Rename every frozen revision element under `root` back into `w:`.
fn thaw(dom: &mut Dom, root: NodeId) {
    let frozen: Vec<(NodeId, String)> = dom
        .descendants_and_self(root, None)
        .into_iter()
        .filter_map(|e| {
            let name = dom.name(e)?;
            (name.namespace_name() == FROZEN_NS).then(|| (e, name.local_name().to_string()))
        })
        .collect();
    for (e, local) in &frozen {
        dom.set_name(*e, W::name(local));
    }
}

/// The content parts of [`revision_bearing_parts`] (styles left out).
fn story_parts(parts: &[(String, bool)]) -> Vec<String> {
    parts
        .iter()
        .filter(|(_, is_styles)| !is_styles)
        .map(|(p, _)| p.clone())
        .collect()
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod revision_boundary_coverage_tests {
    use super::*;

    fn parse(body: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(
            "<w:body xmlns:w='{}' xmlns:x='urn:fixture' x:owner='story'>{body}</w:body>",
            W::URI
        ));
        let root = dom.root(doc).unwrap();
        (dom, root)
    }

    fn paragraph(text: &str) -> String {
        format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
    }

    fn cell(text: &str, properties: &str) -> String {
        format!(
            "<w:tc><w:tcPr>{properties}</w:tcPr>{}</w:tc>",
            paragraph(text)
        )
    }

    fn table(properties: &str, widths: &[i64], row_properties: &str, cells: &str) -> String {
        let grid: String = widths
            .iter()
            .map(|w| format!("<w:gridCol w:w='{w}'/>"))
            .collect();
        format!(
            "<w:tbl><w:tblPr>{properties}</w:tblPr><w:tblGrid>{grid}</w:tblGrid><w:tr x:owner='row'>{row_properties}{cells}</w:tr></w:tbl>"
        )
    }

    fn local_children(dom: &Dom, node: NodeId) -> Vec<String> {
        dom.elements(node, None)
            .into_iter()
            .map(|n| dom.name(n).unwrap().local_name().to_string())
            .collect()
    }

    fn widths(dom: &Dom, table: NodeId) -> Vec<String> {
        let grid = dom.element(table, &W::name("tblGrid")).unwrap();
        dom.elements(grid, Some(&W::name("gridCol")))
            .into_iter()
            .map(|n| dom.attribute(n, &W::name("w")).unwrap().to_string())
            .collect()
    }

    fn span(dom: &Dom, cell: NodeId) -> Option<String> {
        dom.element(cell, &W::tc_pr())
            .and_then(|p| dom.element(p, &W::grid_span()))
            .and_then(|s| dom.attribute(s, &W::val()))
            .map(str::to_string)
    }

    #[test]
    fn revised_table_union_preserves_rows_payload_and_unrelated_clean_groups() {
        let a = table(
            "<w:tblStyle w:val='A'/>",
            &[1000, 2000],
            "<w:trPr><w:ins w:id='7'/></w:trPr>",
            &(cell("A", "<w:tcW w:w='9' w:type='dxa'/>")
                + &cell("B", "<w:shd w:fill='123456'/><w:tcW w:w='8' w:type='dxa'/>")),
        );
        let b = table(
            "<w:tblStyle w:val='B'/>",
            &[1500, 1500],
            "<w:trPr><w:cantSplit/></w:trPr>",
            &(cell("C", "<w:tcW w:w='7'/>")
                + &cell("D", "<w:tcW w:w='6'/><w:gridSpan w:val='1'/>")),
        );
        let clean = table("", &[3000], "", &cell("clean", "<w:tcW w:w='3000'/>"));
        let (mut dom, root) = parse(&(a + &b + &paragraph("separator") + &clean + &clean));
        let original = dom.serialize_element(root);
        let result = merge_adjacent_tables_transform(&mut dom, root);
        assert_ne!(result, root);
        assert_eq!(dom.serialize_element(root), original);
        assert_eq!(
            dom.attribute(result, &XName::get("owner", "urn:fixture")),
            Some("story")
        );
        assert_eq!(local_children(&dom, result), ["tbl", "p", "tbl", "tbl"]);
        let merged = dom.elements(result, Some(&W::tbl()))[0];
        assert_eq!(widths(&dom, merged), ["1000", "500", "1500"]);
        let style = dom
            .element(
                dom.element(merged, &W::tbl_pr()).unwrap(),
                &W::name("tblStyle"),
            )
            .unwrap();
        assert_eq!(dom.attribute(style, &W::val()), Some("A"));
        let rows = dom.elements(merged, Some(&W::tr()));
        assert_eq!(rows.len(), 2);
        assert_eq!(
            dom.attribute(rows[1], &XName::get("owner", "urn:fixture")),
            Some("row")
        );
        let cells: Vec<NodeId> = rows
            .iter()
            .flat_map(|&r| dom.elements(r, Some(&W::tc())))
            .collect();
        assert_eq!(
            cells.iter().map(|&c| dom.value(c)).collect::<Vec<_>>(),
            ["A", "B", "C", "D"]
        );
        assert_eq!(
            cells.iter().map(|&c| span(&dom, c)).collect::<Vec<_>>(),
            [None, Some("2".into()), Some("2".into()), None]
        );
        assert_eq!(
            local_children(&dom, dom.element(cells[1], &W::tc_pr()).unwrap()),
            ["tcW", "gridSpan", "shd"]
        );
        assert_eq!(dom.descendants(merged, Some(&W::ins())).len(), 1);
        assert_eq!(
            dom.descendants(merged, Some(&W::name("cantSplit"))).len(),
            1
        );
        assert_eq!(dom.value(result), "ABCDseparatorcleanclean");
    }

    #[test]
    fn fix_widths_clamps_spans_and_leaves_widthless_cells_out_of_cursor() {
        let cells = cell("no-width", "<w:shd w:fill='ABABAB'/>")
            + &cell(
                "wide",
                "<w:tcW w:w='17' w:type='dxa'/><w:gridSpan w:val='2'/>",
            )
            + &cell("last", "<w:tcW w:w='18'/><w:gridSpan w:val='bad'/>")
            + &cell("past-grid", "<w:tcW w:w='19'/><w:gridSpan w:val='5'/>");
        let (mut dom, root) = parse(&table("", &[1000, 2000, 3000], "", &cells));
        let source = dom.element(root, &W::tbl()).unwrap();
        let original = dom.serialize_element(source);
        let fixed = fix_widths(&mut dom, source);
        let row = dom.element(fixed, &W::tr()).unwrap();
        let cells = dom.elements(row, Some(&W::tc()));
        let actual: Vec<Option<String>> = cells
            .iter()
            .map(|&c| {
                dom.element(c, &W::tc_pr())
                    .and_then(|p| dom.element(p, &W::name("tcW")))
                    .and_then(|w| dom.attribute(w, &W::name("w")))
                    .map(str::to_string)
            })
            .collect();
        assert_eq!(
            actual,
            [
                None,
                Some("3000".into()),
                Some("3000".into()),
                Some("0".into())
            ]
        );
        let width = dom
            .element(dom.element(cells[1], &W::tc_pr()).unwrap(), &W::name("tcW"))
            .unwrap();
        assert_eq!(dom.attribute(width, &W::name("type")), Some("dxa"));
        assert_eq!(dom.value(fixed), "no-widthwidelastpast-grid");
        assert_eq!(dom.serialize_element(source), original);
    }

    #[test]
    fn nested_merge_clones_widthless_cell_and_keeps_control_identity() {
        let cells = cell("opaque", "<w:shd w:fill='101010'/><w:cellIns w:id='8'/>")
            + &cell("sized", "<w:tcW w:w='100'/>");
        let a = table("", &[100], "", &cells);
        let b = table("", &[100], "", &cell("second", "<w:tcW w:w='100'/>"));
        let (mut dom, root) = parse(&format!(
            "<w:sdt><w:sdtPr><w:tag w:val='owner'/></w:sdtPr><w:sdtContent>{a}{b}</w:sdtContent></w:sdt>{}",
            paragraph("tail")
        ));
        let opaque = dom.descendants(root, Some(&W::tc()))[0];
        let expected = dom.serialize_element(opaque);
        let result = merge_adjacent_tables_transform(&mut dom, root);
        assert_eq!(dom.descendants(result, Some(&W::tbl())).len(), 1);
        assert_eq!(dom.descendants(result, Some(&W::sdt())).len(), 1);
        let tag = dom.descendants(result, Some(&W::name("tag")))[0];
        assert_eq!(dom.attribute(tag, &W::val()), Some("owner"));
        assert_eq!(
            dom.serialize_element(dom.descendants(result, Some(&W::tc()))[0]),
            expected
        );
        assert_eq!(dom.value(result), "opaquesizedsecondtail");
    }

    #[test]
    fn word_save_merges_row_exceptions_with_row_values_taking_precedence() {
        let a = table(
            "<w:tblStyle w:val='same'/><w:jc w:val='center'/><w:tblW w:w='100'/><w:shd w:fill='AAAAAA'/>",
            &[100],
            "",
            &cell("first", "<w:tcW w:w='100'/>"),
        );
        let b = table(
            "<w:tblStyle w:val='same'/><w:jc w:val='center'/><w:tblW w:w='200'/><w:shd w:fill='BBBBBB'/><w:tblLayout w:type='fixed'/>",
            &[100],
            "<w:tblPrEx><w:shd w:fill='CCCCCC'/><x:payload x:owner='exception'/></w:tblPrEx><w:trPr><w:cantSplit/></w:trPr>",
            &cell("second", "<w:tcW w:w='100'/>"),
        );
        let (mut dom, root) = parse(&(a + &b));
        let result = merge_adjacent_tables_like_word(&mut dom, root);
        let tables = dom.elements(result, Some(&W::tbl()));
        assert_eq!(tables.len(), 1);
        let rows = dom.elements(tables[0], Some(&W::tr()));
        assert!(dom.element(rows[0], &W::name("tblPrEx")).is_none());
        let ex = dom.element(rows[1], &W::name("tblPrEx")).unwrap();
        assert_eq!(
            local_children(&dom, ex),
            ["tblW", "shd", "tblLayout", "payload"]
        );
        assert_eq!(
            dom.attribute(dom.element(ex, &W::name("tblW")).unwrap(), &W::name("w")),
            Some("200")
        );
        assert_eq!(
            dom.attribute(dom.element(ex, &W::name("shd")).unwrap(), &W::name("fill")),
            Some("CCCCCC")
        );
        assert_eq!(dom.elements(rows[1], Some(&W::name("tblPrEx"))).len(), 1);
        assert_eq!(
            dom.descendants(rows[1], Some(&W::name("cantSplit"))).len(),
            1
        );
        assert_eq!(dom.value(result), "firstsecond");
    }

    #[test]
    fn table_merge_identity_respects_whole_properties_grid_history_and_bidi() {
        let plain = table("", &[100], "", &cell("plain", "<w:tcW w:w='100'/>"));
        let marked = table(
            "<w:bidiVisual/>",
            &[100],
            "<w:trPr><w:del/></w:trPr>",
            &cell("marked", "<w:tcW w:w='100'/>"),
        );
        for body in [
            plain.clone() + &plain,
            plain.clone() + &marked,
            marked.clone() + &paragraph("break") + &marked,
        ] {
            let (mut dom, root) = parse(&body);
            let before = dom.serialize_element(root);
            assert_eq!(merge_adjacent_tables_transform(&mut dom, root), root);
            assert_eq!(dom.serialize_element(root), before);
        }
        let other_style = table(
            "<w:tblStyle w:val='different'/>",
            &[100],
            "",
            &cell("style", "<w:tcW w:w='100'/>"),
        );
        let changed_grid = plain.replacen("</w:tblGrid>", "<w:tblGridChange w:id='2'><w:tblGrid><w:gridCol w:w='50'/></w:tblGrid></w:tblGridChange></w:tblGrid>", 1);
        let foreign = table(
            "<x:payload x:owner='whole'/>",
            &[100],
            "",
            &cell("foreign", "<w:tcW w:w='100'/>"),
        );
        for body in [
            plain.clone() + &other_style,
            plain.clone() + &changed_grid,
            plain + &foreign,
        ] {
            let (mut dom, root) = parse(&body);
            assert_eq!(merge_adjacent_tables_like_word(&mut dom, root), root);
        }
    }

    #[test]
    fn accepting_deleted_cells_keeps_anchor_payload_and_expands_existing_span() {
        let cells = cell("leading-deletion", "<w:cellDel/>")
            + &cell(
                "anchor",
                "<w:shd w:fill='EEEEEE'/><w:gridSpan w:val='2'/><w:tcW w:w='700'/><x:payload x:owner='cell'/>",
            )
            + &cell("deleted-one", "<w:cellDel/>")
            + &cell("deleted-two", "<w:cellDel/>")
            + &cell("tail", "<w:tcW w:w='300'/>");
        let (mut dom, root) = parse(&table(
            "",
            &[100, 100, 100, 100, 100, 100],
            "<w:trPr><w:cantSplit/></w:trPr>",
            &cells,
        ));
        let original = dom.serialize_element(root);
        let result = accept_deleted_cells_transform(&mut dom, root);
        let row = dom.descendants(result, Some(&W::tr()))[0];
        assert_eq!(
            dom.attribute(row, &XName::get("owner", "urn:fixture")),
            Some("row")
        );
        let cells = dom.elements(row, Some(&W::tc()));
        assert_eq!(cells.len(), 2);
        assert_eq!(dom.value(result), "anchortail");
        assert_eq!(span(&dom, cells[0]), Some("4".into()));
        assert_eq!(span(&dom, cells[1]), None);
        assert_eq!(
            local_children(&dom, dom.element(cells[0], &W::tc_pr()).unwrap()),
            ["tcW", "gridSpan", "shd", "payload"]
        );
        assert!(dom.descendants(result, Some(&W::cell_del())).is_empty());
        assert_eq!(dom.serialize_element(root), original);
    }

    #[test]
    fn deleted_cell_anchor_defaults_bad_span_and_orders_all_schema_properties() {
        let property_names = [
            "headers",
            "hideMark",
            "vAlign",
            "tcFitText",
            "textDirection",
            "tcMar",
            "noWrap",
            "shd",
            "tcBorders",
            "vMerge",
            "hMerge",
            "tcW",
            "cnfStyle",
        ];
        let properties: String = property_names.iter().map(|p| format!("<w:{p}/>")).collect();
        let cells = cell(
            "anchor",
            &(properties + "<w:gridSpan w:val='not-an-integer'/><x:gridSpan x:owner='foreign'/>"),
        ) + &cell("deleted", "<w:cellDel/>");
        let (mut dom, root) = parse(&table("", &[100, 100], "", &cells));
        let result = accept_deleted_cells_transform(&mut dom, root);
        let cell = dom.descendants(result, Some(&W::tc()))[0];
        let pr = dom.element(cell, &W::tc_pr()).unwrap();
        assert_eq!(span(&dom, cell), Some("2".into()));
        assert_eq!(
            local_children(&dom, pr),
            [
                "cnfStyle",
                "tcW",
                "gridSpan",
                "hMerge",
                "vMerge",
                "tcBorders",
                "shd",
                "noWrap",
                "tcMar",
                "textDirection",
                "tcFitText",
                "vAlign",
                "hideMark",
                "headers",
                "gridSpan"
            ]
        );
        assert_eq!(
            dom.descendants(result, Some(&XName::get("gridSpan", "urn:fixture")))
                .len(),
            1
        );
        assert_eq!(dom.value(result), "anchor");
    }

    #[test]
    fn custom_xml_deleted_control_unwraps_nested_controls_but_retains_order_and_anchors() {
        let (mut dom, root) = parse(
            "<w:customXmlDelRangeStart w:id='4'/><w:sdt><w:sdtPr><w:tag w:val='outer'/></w:sdtPr><w:sdtContent><w:bookmarkStart w:id='3' w:name='kept'/><w:p><w:r><w:t>before</w:t></w:r><w:sdt><w:sdtPr><w:tag w:val='inner'/></w:sdtPr><w:sdtContent><w:r><w:t>inside</w:t></w:r><x:payload x:owner='run'/></w:sdtContent></w:sdt><w:r><w:t>after</w:t></w:r></w:p><w:bookmarkEnd w:id='3'/></w:sdtContent></w:sdt><w:customXmlDelRangeEnd w:id='4'/><w:p><w:r><w:t>tail</w:t></w:r></w:p>",
        );
        let result = accept_deleted_and_moved_from_content_controls(&mut dom, root);
        assert!(dom.descendants(result, Some(&W::sdt())).is_empty());
        assert_eq!(dom.value(result), "beforeinsideaftertail");
        assert_eq!(
            local_children(&dom, result),
            [
                "customXmlDelRangeStart",
                "bookmarkStart",
                "p",
                "bookmarkEnd",
                "customXmlDelRangeEnd",
                "p"
            ]
        );
        assert_eq!(
            dom.descendants(result, Some(&W::name("bookmarkStart")))
                .len(),
            1
        );
        assert_eq!(
            dom.descendants(result, Some(&XName::get("payload", "urn:fixture")))
                .len(),
            1
        );
    }

    #[test]
    fn custom_xml_move_deletes_whole_control_and_plain_content_but_keeps_outside_control() {
        let (mut dom, root) = parse(
            "<w:customXmlMoveFromRangeStart w:id='9'/><w:sdt><w:sdtPr><w:tag w:val='deleted'/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>gone</w:t></w:r></w:p></w:sdtContent></w:sdt><w:p><w:r><w:t>also-gone</w:t></w:r></w:p><w:customXmlMoveFromRangeEnd w:id='9'/><w:sdt><w:sdtPr><w:tag w:val='kept'/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>retained</w:t></w:r></w:p></w:sdtContent></w:sdt>",
        );
        let result = accept_deleted_and_moved_from_content_controls(&mut dom, root);
        assert_eq!(dom.value(result), "retained");
        assert_eq!(
            local_children(&dom, result),
            [
                "customXmlMoveFromRangeStart",
                "customXmlMoveFromRangeEnd",
                "sdt"
            ]
        );
        let controls = dom.descendants(result, Some(&W::sdt()));
        assert_eq!(controls.len(), 1);
        let tag = dom.descendants(controls[0], Some(&W::name("tag")))[0];
        assert_eq!(dom.attribute(tag, &W::val()), Some("kept"));
    }

    #[test]
    fn unmatched_custom_ranges_leave_control_payload_and_parent_links_untouched() {
        for kind in ["customXmlDelRange", "customXmlMoveFromRange"] {
            for end in [String::new(), format!("<w:{kind}End w:id='other'/>")] {
                let (mut dom, root) = parse(&format!(
                    "<w:{kind}Start w:id='unmatched'/><w:sdt><w:sdtPr/><w:sdtContent>{}</w:sdtContent></w:sdt>{end}",
                    paragraph("retained")
                ));
                let before = dom.serialize_element(root);
                let control = dom.element(root, &W::sdt()).unwrap();
                assert_eq!(
                    accept_deleted_and_moved_from_content_controls(&mut dom, root),
                    root
                );
                assert_eq!(dom.parent(control), Some(root));
                assert_eq!(dom.serialize_element(root), before);
            }
        }
    }

    #[test]
    fn move_range_that_enters_table_keeps_container_properties_and_surviving_content() {
        let (mut dom, root) = parse(
            "<w:moveFromRangeStart w:id='7'/><w:p><w:r><w:t>heading-gone</w:t></w:r></w:p><w:tbl><w:tblPr><w:tblStyle w:val='retained'/></w:tblPr><w:tblGrid><w:gridCol w:w='2400'/></w:tblGrid><w:tr><w:trPr><w:cantSplit/></w:trPr><w:tc><w:tcPr><w:tcW w:w='2400'/></w:tcPr><w:p><w:pPr><w:keepNext/></w:pPr><w:r><w:t>prefix-gone</w:t></w:r><w:moveFromRangeEnd w:id='7'/><w:r><w:t>remaining</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
        );
        let result = accept_move_from_ranges(&mut dom, root);
        assert_eq!(dom.value(result), "remaining");
        assert_eq!(dom.descendants(result, Some(&W::tbl())).len(), 1);
        assert_eq!(dom.descendants(result, Some(&W::name("tblStyle"))).len(), 1);
        assert_eq!(dom.descendants(result, Some(&W::name("gridCol"))).len(), 1);
        assert_eq!(
            dom.descendants(result, Some(&W::name("cantSplit"))).len(),
            1
        );
        assert_eq!(dom.descendants(result, Some(&W::name("tcW"))).len(), 1);
        assert_eq!(dom.descendants(result, Some(&W::name("keepNext"))).len(), 1);
        assert_eq!(
            dom.descendants(result, Some(&W::move_from_range_end()))
                .len(),
            1
        );
    }

    #[test]
    fn accept_cell_merge_converts_restart_and_continue_only_inside_cell_properties() {
        for (value, expected) in [
            ("rest", Some("restart")),
            ("cont", Some("continue")),
            ("unknown", None),
        ] {
            let (mut dom, root) = parse(&format!(
                "<w:tc><w:tcPr><w:cellMerge w:id='4' w:vMerge='{value}' w:vMergeOrig='cont'/></w:tcPr>{}</w:tc>",
                paragraph("cell")
            ));
            let output = accept_all_other_revisions_transform(&mut dom, root);
            assert_eq!(output.len(), 1);
            let result = output[0];
            let merges = dom.descendants(result, Some(&W::v_merge()));
            match expected {
                Some(expected) => {
                    assert_eq!(merges.len(), 1);
                    assert_eq!(dom.attribute(merges[0], &W::val()), Some(expected));
                    assert!(dom.descendants(result, Some(&W::cell_merge())).is_empty());
                }
                None => {
                    assert!(merges.is_empty());
                    let marker = dom.descendants(result, Some(&W::cell_merge()))[0];
                    assert_eq!(dom.attribute(marker, &W::v_merge()), Some("unknown"));
                }
            }
            assert_eq!(dom.value(result), "cell");
        }
        let (mut dom, root) =
            parse("<x:payload><w:cellMerge w:id='5' w:vMerge='rest'/></x:payload>");
        let output = accept_all_other_revisions_transform(&mut dom, root);
        assert_eq!(dom.descendants(output[0], Some(&W::cell_merge())).len(), 1);
        assert!(dom.descendants(output[0], Some(&W::v_merge())).is_empty());
    }

    #[test]
    fn internally_deleted_equations_leave_no_math_shell_but_keep_live_and_empty_math() {
        for inserted in [false, true] {
            for para in [false, true] {
                let marker = if inserted { "ins" } else { "del" };
                let equation = format!(
                    "<m:oMath><m:f><m:fPr><m:ctrlPr><w:{marker} w:id='3'><w:rPr><w:b/></w:rPr></w:{marker}></m:ctrlPr></m:fPr><m:num><m:r><w:{marker} w:id='4'><w:rPr><w:rFonts w:ascii='Cambria Math'/></w:rPr><m:t>x</m:t></w:{marker}></m:r></m:num><m:den><m:r><w:{marker} w:id='5'><m:t>y</m:t></w:{marker}></m:r></m:den></m:f></m:oMath>"
                );
                let equation = if para {
                    format!(
                        "<m:oMathPara><m:oMathParaPr><m:jc m:val='center'/></m:oMathParaPr>{equation}</m:oMathPara>"
                    )
                } else {
                    equation
                };
                let live = "<m:oMath><m:r><m:t>live</m:t></m:r></m:oMath>";
                let empty = "<m:oMath><m:r/></m:oMath>";
                let body = format!(
                    "<w:p xmlns:m='{}'><w:pPr><w:spacing w:after='240'/></w:pPr>{equation}{live}{empty}<w:r><w:t>tail</w:t></w:r></w:p>",
                    M::URI
                );
                let (mut dom, root) = parse(&body);
                let accepted = accept_revisions_for_part_content(&mut dom, root);
                let (mut rejected_dom, rejected_root) = parse(&body);
                let rejected = reject_revisions_document(&mut rejected_dom, rejected_root);
                for (projection_dom, projection, keep_changed) in [
                    (&dom, accepted, inserted),
                    (&rejected_dom, rejected, !inserted),
                ] {
                    assert_eq!(
                        projection_dom
                            .descendants(projection, Some(&M::name("oMath")))
                            .len(),
                        if keep_changed { 3 } else { 2 }
                    );
                    assert_eq!(
                        projection_dom
                            .descendants(projection, Some(&M::name("oMathPara")))
                            .len(),
                        usize::from(keep_changed && para)
                    );
                    assert_eq!(
                        projection_dom.value(projection),
                        if keep_changed {
                            "xylivetail"
                        } else {
                            "livetail"
                        }
                    );
                    let spacing =
                        projection_dom.descendants(projection, Some(&W::name("spacing")))[0];
                    assert_eq!(
                        projection_dom.attribute(spacing, &W::name("after")),
                        Some("240")
                    );
                    assert!(
                        projection_dom
                            .descendants(projection, Some(&W::del()))
                            .is_empty()
                    );
                    assert!(
                        projection_dom
                            .descendants(projection, Some(&W::ins()))
                            .is_empty()
                    );
                }
            }
        }
        // A live empty run in an otherwise deleted equation is an authored
        // owner, so deleting another run cannot delete its enclosing equation.
        let (mut dom, root) = parse(&format!(
            "<w:p><m:oMath xmlns:m='{}'><m:r><w:del w:id='8'><m:t>gone</m:t></w:del></m:r><m:r/></m:oMath></w:p>",
            M::URI
        ));
        let result = accept_revisions_for_part_content(&mut dom, root);
        assert_eq!(dom.descendants(result, Some(&M::name("oMath"))).len(), 1);
        assert_eq!(dom.descendants(result, Some(&M::name("r"))).len(), 2);
        assert_eq!(dom.value(result), "");
    }

    #[test]
    fn display_math_preserves_independent_empty_equation_and_mixed_run_payload() {
        let (mut dom, root) = parse(&format!(
            "<w:p><m:oMathPara xmlns:m='{}'><m:oMath><m:r><w:del w:id='8'><m:t>gone</m:t></w:del></m:r></m:oMath><m:oMath/></m:oMathPara><m:oMath xmlns:m='{}'><m:r><w:del w:id='9'><m:t>old</m:t></w:del><m:t>live</m:t></m:r></m:oMath></w:p>",
            M::URI,
            M::URI
        ));
        let result = accept_revisions_for_part_content(&mut dom, root);
        assert_eq!(dom.value(result), "live");
        assert_eq!(
            dom.descendants(result, Some(&M::name("oMathPara"))).len(),
            1
        );
        assert_eq!(dom.descendants(result, Some(&M::name("oMath"))).len(), 2);
        let display = dom.descendants(result, Some(&M::name("oMathPara")))[0];
        let authored_empty = dom.elements(display, Some(&M::name("oMath")));
        assert_eq!(authored_empty.len(), 1);
        assert!(dom.nodes(authored_empty[0]).is_empty());
        assert!(dom.descendants(result, Some(&W::del())).is_empty());
    }

    #[test]
    fn internally_deleted_math_does_not_consume_independent_anchors_controls_or_empty_branches() {
        fn semantic(dom: &Dom, node: NodeId) -> String {
            let mut attributes = dom
                .attributes_iter(node)
                .filter(|(name, _)| !dom.is_namespace_declaration(name))
                .collect::<Vec<_>>();
            attributes.sort_by(|a, b| {
                (a.0.namespace_name(), a.0.local_name())
                    .cmp(&(b.0.namespace_name(), b.0.local_name()))
            });
            let children = dom
                .nodes(node)
                .into_iter()
                .map(|child| semantic(dom, child))
                .collect::<Vec<_>>();
            format!(
                "{:?}:{attributes:?}:{:?}:{children:?}",
                dom.name(node),
                dom.text_value(node)
            )
        }
        let deleted_run = "<m:r><w:del w:id='8'><m:t>gone</m:t></w:del></m:r>";
        for live in [
            "<w:bookmarkStart w:id='31' w:name='Clause'/><w:bookmarkEnd w:id='31'/>",
            "<w:commentRangeStart w:id='41'/><w:commentRangeEnd w:id='41'/>",
            "<w:permStart w:id='51' w:edGrp='everyone'/><w:permEnd w:id='51'/>",
            "<w:r><w:rPr><w:b/></w:rPr><w:t>ordinary-live</w:t></w:r>",
            "<w:customXml w:uri='urn:source' w:element='Clause'><w:customXmlPr><w:attr w:name='owner' w:val='authored'/></w:customXmlPr><w:r><w:t>custom-live</w:t></w:r></w:customXml>",
            "<w:sdt><w:sdtPr><w:id w:val='61'/><w:tag w:val='Clause'/></w:sdtPr><w:sdtContent><w:r><w:rPr><w:i/></w:rPr><w:t>control-live</w:t></w:r></w:sdtContent></w:sdt>",
            "<m:rad><m:radPr><m:degHide m:val='1'/></m:radPr><m:deg/><m:e/></m:rad>",
        ] {
            let body = format!(
                "<w:p><m:oMath xmlns:m='{}'>{deleted_run}{live}</m:oMath></w:p>",
                M::URI
            );
            let (mut dom, root) = parse(&body);
            let expected_body = format!(
                "<w:p><m:oMath xmlns:m='{}'><m:r/>{live}</m:oMath></w:p>",
                M::URI
            );
            let (expected_dom, expected_root) = parse(&expected_body);
            let output = accept_all_other_revisions_transform(&mut dom, root);
            assert_eq!(output.len(), 1);
            assert_eq!(
                semantic(&dom, output[0]),
                semantic(&expected_dom, expected_root),
                "live={live}"
            );
        }
        // An enclosing insertion is an independent lifetime owner. A deleted
        // nested run does not by itself prove its equation was wholly deleted.
        let (mut dom, root) = parse(&format!(
            "<w:p><m:oMath xmlns:m='{}'><w:ins w:id='9'>{deleted_run}</w:ins></m:oMath></w:p>",
            M::URI
        ));
        let output = accept_all_other_revisions_transform(&mut dom, root);
        assert_eq!(dom.descendants(output[0], Some(&M::name("oMath"))).len(), 1);
        assert_eq!(dom.descendants(output[0], Some(&M::name("r"))).len(), 1);
        assert_eq!(dom.value(output[0]), "");
    }

    #[test]
    fn wholly_deleted_math_hoists_owned_range_metadata_instead_of_discarding_it() {
        let (mut dom, root) = parse(&format!(
            "<w:p><m:oMath xmlns:m='{}'><m:r><w:del w:id='8'><w:bookmarkStart w:id='31' w:name='Clause'/><w:commentRangeStart w:id='41'/><m:t>gone</m:t><w:commentRangeEnd w:id='41'/><w:bookmarkEnd w:id='31'/></w:del></m:r></m:oMath><w:r><w:t>tail</w:t></w:r></w:p>",
            M::URI
        ));
        let output = accept_all_other_revisions_transform(&mut dom, root);
        assert!(
            dom.descendants(output[0], Some(&M::name("oMath")))
                .is_empty()
        );
        assert_eq!(dom.value(output[0]), "tail");
        let paragraph = dom.descendants(output[0], Some(&W::p()))[0];
        let children = dom.elements(paragraph, None);
        assert_eq!(children.len(), 5);
        for (child, name, id) in [
            (children[0], "bookmarkStart", "31"),
            (children[1], "commentRangeStart", "41"),
            (children[2], "commentRangeEnd", "41"),
            (children[3], "bookmarkEnd", "31"),
        ] {
            assert!(dom.name_is(child, &W::name(name)));
            assert_eq!(dom.attribute(child, &W::id()), Some(id));
        }
        assert_eq!(dom.attribute(children[0], &W::name("name")), Some("Clause"));
        assert!(dom.name_is(children[4], &W::r()));
    }

    #[test]
    fn accept_deleted_fraction_drops_its_math_payload_and_preserves_live_fraction() {
        let (mut dom, root) = parse(&format!(
            "<m:oMath xmlns:m='{}'><m:f><m:fPr><m:ctrlPr><w:del w:id='1'/></m:ctrlPr></m:fPr><m:num><m:r><m:t>deleted-numerator</m:t></m:r></m:num><m:den><m:r><m:t>deleted-denominator</m:t></m:r></m:den></m:f><m:f><m:fPr><m:ctrlPr><w:ins w:id='2'/></m:ctrlPr></m:fPr><m:num><m:r><m:t>live-numerator</m:t></m:r></m:num><m:den><m:r><m:t>live-denominator</m:t></m:r></m:den></m:f></m:oMath>",
            M::URI
        ));
        let output = accept_all_other_revisions_transform(&mut dom, root);
        assert_eq!(output.len(), 1);
        assert_eq!(dom.descendants(output[0], Some(&M::name("f"))).len(), 1);
        assert_eq!(dom.value(output[0]), "live-numeratorlive-denominator");
        assert!(dom.descendants(output[0], Some(&W::del())).is_empty());
        assert!(dom.descendants(output[0], Some(&W::ins())).is_empty());
    }

    #[test]
    fn rejecting_structural_markers_drops_inserted_numbering_and_cell_without_losing_neighbors() {
        let (mut dom, root) = parse(
            "<w:p><w:pPr><w:keepNext/><w:numPr><w:ilvl w:val='1'/><w:numId w:val='8'/><w:ins w:id='3'/></w:numPr></w:pPr><w:r><w:t>paragraph</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:tcPr><w:cellIns w:id='4'/></w:tcPr><w:p><w:r><w:t>inserted-cell</w:t></w:r></w:p></w:tc><w:tc><w:tcPr><w:cellDel w:id='5'/><w:cellMerge w:id='6'/></w:tcPr><w:p><w:pPr><w:numPr><w:numId w:val='9'/><w:numberingChange w:id='7'/></w:numPr></w:pPr><w:r><w:t>original-cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
        );
        let result = reject_revisions_for_part_transform(&mut dom, root).unwrap();
        assert_eq!(dom.value(result), "paragraphoriginal-cell");
        assert_eq!(dom.descendants(result, Some(&W::tc())).len(), 1);
        assert_eq!(dom.descendants(result, Some(&W::num_pr())).len(), 1);
        assert_eq!(dom.descendants(result, Some(&W::name("keepNext"))).len(), 1);
        let num_id = dom.descendants(result, Some(&W::name("numId")))[0];
        assert_eq!(dom.attribute(num_id, &W::val()), Some("9"));
        for name in [
            W::ins(),
            W::cell_ins(),
            W::cell_del(),
            W::cell_merge(),
            W::numbering_change(),
        ] {
            assert!(dom.descendants(result, Some(&name)).is_empty());
        }
    }

    #[test]
    fn missing_saved_style_properties_drop_changed_block_while_accept_keeps_live_properties() {
        let (mut dom, root) = parse(
            "<w:style w:styleId='owner'><w:name w:val='style-name'/><w:pPr><w:keepNext/><w:pPrChange w:id='1'/></w:pPr><w:rPr><w:b/><w:rPrChange w:id='2'/></w:rPr><x:payload x:owner='retained'>style-payload</x:payload></w:style>",
        );
        let accepted = accept_revisions_for_styles_transform(&mut dom, root).unwrap();
        let rejected = reject_revisions_for_styles_transform(&mut dom, root).unwrap();
        assert_eq!(
            dom.descendants(accepted, Some(&W::name("keepNext"))).len(),
            1
        );
        assert_eq!(dom.descendants(accepted, Some(&W::name("b"))).len(), 1);
        assert!(
            dom.descendants(accepted, Some(&W::p_pr_change()))
                .is_empty()
        );
        assert!(
            dom.descendants(accepted, Some(&W::r_pr_change()))
                .is_empty()
        );
        assert!(dom.descendants(rejected, Some(&W::p_pr())).is_empty());
        assert!(dom.descendants(rejected, Some(&W::r_pr())).is_empty());
        for result in [accepted, rejected] {
            let style = dom.element(result, &W::name("style")).unwrap();
            assert_eq!(dom.attribute(style, &W::name("styleId")), Some("owner"));
            assert_eq!(dom.value(result), "style-payload");
            assert_eq!(
                dom.descendants(result, Some(&XName::get("payload", "urn:fixture")))
                    .len(),
                1
            );
        }
    }

    #[test]
    fn acceptance_removes_empty_numbering_but_keeps_live_numbering_and_clears_atom_annotations() {
        let (mut dom, root) = parse(
            "<w:p><w:pPr><w:numPr/></w:pPr><w:r><w:t>plain</w:t></w:r></w:p><w:p><w:pPr><w:numPr><w:numId w:val='4'/></w:numPr></w:pPr><w:r><w:t>numbered</w:t></w:r></w:p>",
        );
        let run = dom.descendants(root, Some(&W::r()))[0];
        dom.set_attribute_value(root, &PT::unique_id(), Some("body-id"));
        dom.set_attribute_value(run, &PT::run_ids(), Some("run-id"));
        let result = accept_revisions_for_element(&mut dom, root);
        assert_eq!(dom.value(result), "plainnumbered");
        assert_eq!(dom.descendants(result, Some(&W::num_pr())).len(), 1);
        assert_eq!(
            dom.attribute(
                dom.descendants(result, Some(&W::name("numId")))[0],
                &W::val()
            ),
            Some("4")
        );
        for node in dom.descendants_and_self(result, None) {
            assert!(dom.attribute(node, &PT::unique_id()).is_none());
            assert!(dom.attribute(node, &PT::run_ids()).is_none());
        }
    }

    #[test]
    fn unmatched_or_contentless_move_ranges_preserve_identity_and_properties() {
        for body in [
            "<w:moveFromRangeEnd w:id='missing'/><w:p><w:r><w:t>retained</w:t></w:r></w:p>",
            "<w:moveFromRangeStart w:id='open'/><w:p><w:r><w:t>retained</w:t></w:r></w:p>",
            "<w:moveFromRangeStart w:id='empty'/><w:p><w:pPr><w:keepNext/></w:pPr><w:moveFromRangeEnd w:id='empty'/><w:r><w:t>retained</w:t></w:r></w:p>",
        ] {
            let (mut dom, root) = parse(body);
            let before = dom.serialize_element(root);
            let paragraph = dom.element(root, &W::p()).unwrap();
            assert_eq!(accept_move_from_ranges(&mut dom, root), root);
            assert_eq!(dom.parent(paragraph), Some(root));
            assert_eq!(dom.serialize_element(root), before);
            assert_eq!(dom.value(root), "retained");
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod public_block_control_ownership_tests {
    use super::*;
    use crate::opc::PartFs;

    const MAIN: &str = "word/document.xml";
    const DATE: &str = "2001-02-03T04:05:06Z";
    const PROPS: &str = "<w:spacing w:after='120'/><w:rPr><w:b/>";
    const META: &str = "<w:sdtPr><w:alias w:val='Contract schedule'/><w:tag w:val='source-owner'/><w:id w:val='42'/><w:lock w:val='sdtLocked'/></w:sdtPr><w:sdtEndPr><w:rPr><w:color w:val='234567'/></w:rPr></w:sdtEndPr>";

    fn package(content: &str) -> Vec<u8> {
        let mut pkg = PartFs::open(include_bytes!(
            "../tests/fixtures/word_probes/tokens/cell_a.docx"
        ))
        .unwrap();
        for part in pkg.parts() {
            pkg.remove_part(&part);
        }
        let empty=b"<Relationships xmlns='http://schemas.openxmlformats.org/package/2006/relationships'/>";
        pkg.set_part("_rels/.rels", empty.to_vec());
        pkg.set_part("word/_rels/document.xml.rels", empty.to_vec());
        pkg.add_package_relationship(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
            MAIN,
        );
        pkg.add_content_type_override(
            "/word/document.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        );
        pkg.set_part(MAIN,format!("<w:document xmlns:w='{}' xmlns:m='{}'><w:body>{content}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI,M::URI).into_bytes());
        pkg.to_zip().unwrap()
    }
    fn resolved(bytes: &[u8], reject: bool) -> (Dom, NodeId) {
        let out = if reject {
            crate::document_comparer::reject_revisions(bytes).unwrap()
        } else {
            crate::document_comparer::accept_revisions(bytes).unwrap()
        };
        let pkg = PartFs::open(&out).unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&pkg.part_string(MAIN).unwrap());
        let root = dom.root(doc).unwrap();
        (dom, root)
    }
    fn assert_owner(dom: &Dom, root: NodeId) -> NodeId {
        let controls = dom.descendants(root, Some(&W::sdt()));
        assert_eq!(controls.len(), 1);
        let control = controls[0];
        assert_eq!(dom.parent(control), dom.element(root, &W::body()));
        let props = dom.element(control, &W::sdt_pr()).unwrap();
        for (local, value) in [
            ("alias", "Contract schedule"),
            ("tag", "source-owner"),
            ("id", "42"),
            ("lock", "sdtLocked"),
        ] {
            let child = dom.element(props, &W::name(local)).unwrap();
            assert_eq!(dom.attribute(child, &W::val()), Some(value));
        }
        let end = dom.element(control, &W::name("sdtEndPr")).unwrap();
        let color = dom.descendants(end, Some(&W::name("color")))[0];
        assert_eq!(dom.attribute(color, &W::val()), Some("234567"));
        dom.element(control, &W::sdt_content()).unwrap()
    }
    #[test]
    fn public_mark_resolution_rewraps_transformed_paragraphs_with_original_control_metadata() {
        for kind in ["del", "ins"] {
            for reject in [false, true] {
                let source = format!(
                    "<w:sdt>{META}<w:sdtContent><w:p><w:pPr>{PROPS}<w:{kind} w:id='17' w:author='Source editor' w:date='{DATE}'/></w:rPr></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>First</w:t></w:r></w:p><w:p><w:pPr>{PROPS}</w:rPr></w:pPr><w:r><w:rPr><w:u w:val='single'/></w:rPr><w:t>Second</w:t></w:r></w:p></w:sdtContent></w:sdt>"
                );
                let (dom, root) = resolved(&package(&source), reject);
                let content = assert_owner(&dom, root);
                let ps = dom.elements(content, Some(&W::p()));
                let joined = (kind == "del") != reject;
                assert_eq!(
                    ps.len(),
                    if joined { 1 } else { 2 },
                    "{kind} reject={reject}"
                );
                assert_eq!(dom.value(content), "FirstSecond");
                let runs = dom.descendants(content, Some(&W::r()));
                assert_eq!(runs.len(), 2);
                let first = dom.element(runs[0], &W::r_pr()).unwrap();
                let second = dom.element(runs[1], &W::r_pr()).unwrap();
                assert!(dom.element(first, &W::name("i")).is_some());
                assert!(dom.element(second, &W::name("u")).is_some());
                for p in ps {
                    let ppr = dom.element(p, &W::p_pr()).unwrap();
                    let spacing = dom.element(ppr, &W::spacing_el()).unwrap();
                    assert_eq!(dom.attribute(spacing, &W::name("after")), Some("120"));
                    let mark = dom.element(ppr, &W::r_pr()).unwrap();
                    assert!(dom.element(mark, &W::name("b")).is_some());
                }
            }
        }
    }
    #[test]
    fn public_mark_resolution_preserves_whole_table_payload_at_the_controls_block_level() {
        for kind in ["del", "ins"] {
            let table = "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/><w:shd w:fill='123456'/></w:tcPr><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Owned cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
            let source = format!(
                "<w:sdt>{META}<w:sdtContent><w:p><w:pPr><w:rPr><w:{kind} w:id='17' w:author='Source editor' w:date='{DATE}'/></w:rPr></w:pPr></w:p>{table}</w:sdtContent></w:sdt>"
            );
            let (dom, root) = resolved(&package(&source), kind == "ins");
            let content = assert_owner(&dom, root);
            let tables = dom.descendants(root, Some(&W::tbl()));
            assert_eq!(tables.len(), 1);
            assert_eq!(dom.parent(tables[0]), Some(content));
            assert_eq!(dom.value(tables[0]), "Owned cell");
            let cell = dom.descendants(tables[0], Some(&W::tc()))[0];
            let properties = dom.element(cell, &W::tc_pr()).unwrap();
            let shade = dom.element(properties, &W::name("shd")).unwrap();
            assert_eq!(dom.attribute(shade, &W::name("fill")), Some("123456"));
            let grid = dom.element(tables[0], &W::name("tblGrid")).unwrap();
            let column = dom.elements(grid, None)[0];
            assert_eq!(dom.attribute(column, &W::name("w")), Some("1800"));
        }
    }
    #[test]
    fn nested_block_controls_keep_each_owner_and_transformed_paragraph_payload() {
        for depth in [2, 3] {
            for kind in ["del", "ins"] {
                for reject in [false, true] {
                    let mut source = format!(
                        "<w:p><w:pPr>{PROPS}<w:{kind} w:id='17'/></w:rPr></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t>First</w:t></w:r></w:p><w:p><w:pPr>{PROPS}</w:rPr></w:pPr><w:r><w:rPr><w:u w:val='single'/></w:rPr><w:t>Second</w:t></w:r></w:p>"
                    );
                    for id in (1..=depth).rev() {
                        source = format!(
                            "<w:sdt><w:sdtPr><w:id w:val='{id}'/><w:tag w:val='owner-{id}'/><w:alias w:val='Owner {id}'/><w:lock w:val='sdtLocked'/></w:sdtPr><w:sdtEndPr><w:rPr><w:color w:val='234567'/></w:rPr></w:sdtEndPr><w:sdtContent>{source}</w:sdtContent></w:sdt>"
                        );
                    }
                    let (dom, root) = resolved(&package(&source), reject);
                    let controls = dom.descendants(root, Some(&W::sdt()));
                    assert_eq!(controls.len(), depth);
                    let mut parent = dom.element(root, &W::body()).unwrap();
                    for (index, &control) in controls.iter().enumerate() {
                        assert_eq!(dom.parent(control), Some(parent));
                        let props = dom.element(control, &W::sdt_pr()).unwrap();
                        for (name, expected) in [
                            ("id", (index + 1).to_string()),
                            ("tag", format!("owner-{}", index + 1)),
                            ("alias", format!("Owner {}", index + 1)),
                            ("lock", "sdtLocked".to_string()),
                        ] {
                            assert_eq!(
                                dom.attribute(
                                    dom.element(props, &W::name(name)).unwrap(),
                                    &W::val()
                                ),
                                Some(expected.as_str())
                            );
                        }
                        let end = dom.element(control, &W::name("sdtEndPr")).unwrap();
                        let color = dom.descendants(end, Some(&W::name("color")))[0];
                        assert_eq!(dom.attribute(color, &W::val()), Some("234567"));
                        parent = dom.element(control, &W::sdt_content()).unwrap();
                    }
                    let ps = dom.elements(parent, Some(&W::p()));
                    assert_eq!(ps.len(), if (kind == "del") != reject { 1 } else { 2 });
                    assert_eq!(dom.value(parent), "FirstSecond");
                    assert!(!element_has_tracked_revisions(&dom, root));
                    let runs = dom.descendants(parent, Some(&W::r()));
                    assert_eq!(runs.len(), 2);
                    assert!(
                        dom.element(dom.element(runs[0], &W::r_pr()).unwrap(), &W::name("i"))
                            .is_some()
                    );
                    assert!(
                        dom.element(dom.element(runs[1], &W::r_pr()).unwrap(), &W::name("u"))
                            .is_some()
                    );
                    for p in ps {
                        let properties = dom.element(p, &W::p_pr()).unwrap();
                        assert_eq!(
                            dom.attribute(
                                dom.element(properties, &W::spacing_el()).unwrap(),
                                &W::name("after")
                            ),
                            Some("120")
                        );
                        assert!(
                            dom.element(
                                dom.element(properties, &W::r_pr()).unwrap(),
                                &W::name("b")
                            )
                            .is_some()
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn nested_block_controls_keep_outer_table_owner_and_nested_cell_geometry() {
        let inner_table = "<w:tbl><w:tblPr><w:tblW w:w='900' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='900'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='900' w:type='dxa'/><w:shd w:fill='ABCDEF'/></w:tcPr><w:p><w:r><w:rPr><w:b/></w:rPr><w:t>Nested payload</w:t></w:r></w:p></w:tc></w:tr></w:tbl>";
        let outer_table = format!(
            "<w:tbl><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr>{inner_table}<w:p/></w:tc></w:tr></w:tbl>"
        );
        for kind in ["del", "ins"] {
            let source = format!(
                "<w:sdt>{META}<w:sdtContent><w:sdt><w:sdtPr><w:id w:val='43'/><w:tag w:val='inner-table-owner'/></w:sdtPr><w:sdtContent><w:p><w:pPr><w:rPr><w:{kind} w:id='17'/></w:rPr></w:pPr></w:p>{outer_table}</w:sdtContent></w:sdt></w:sdtContent></w:sdt>"
            );
            let (dom, root) = resolved(&package(&source), kind == "ins");
            let controls = dom.descendants(root, Some(&W::sdt()));
            assert_eq!(controls.len(), 2);
            assert_eq!(dom.parent(controls[0]), dom.element(root, &W::body()));
            let outer_content = dom.element(controls[0], &W::sdt_content()).unwrap();
            assert_eq!(dom.parent(controls[1]), Some(outer_content));
            let inner_content = dom.element(controls[1], &W::sdt_content()).unwrap();
            let tables = dom.descendants(root, Some(&W::tbl()));
            assert_eq!(tables.len(), 2);
            assert_eq!(dom.parent(tables[0]), Some(inner_content));
            let cells = dom.descendants(root, Some(&W::tc()));
            assert_eq!(cells.len(), 2);
            assert_eq!(dom.parent(tables[1]), Some(cells[0]));
            assert_eq!(dom.value(root), "Nested payload");
            assert!(!element_has_tracked_revisions(&dom, root));
            for (cell, width) in [(cells[0], "1800"), (cells[1], "900")] {
                let props = dom.element(cell, &W::tc_pr()).unwrap();
                assert_eq!(
                    dom.attribute(dom.element(props, &W::name("tcW")).unwrap(), &W::name("w")),
                    Some(width)
                );
            }
            let nested_props = dom.element(cells[1], &W::tc_pr()).unwrap();
            assert_eq!(
                dom.attribute(
                    dom.element(nested_props, &W::name("shd")).unwrap(),
                    &W::name("fill")
                ),
                Some("ABCDEF")
            );
            for (control, id) in [(controls[0], "42"), (controls[1], "43")] {
                let props = dom.element(control, &W::sdt_pr()).unwrap();
                assert_eq!(
                    dom.attribute(dom.element(props, &W::name("id")).unwrap(), &W::val()),
                    Some(id)
                );
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod body_equation_paragraph_mark_ownership_tests {
    use super::*;
    use crate::opc::PartFs;

    fn equation(display: bool) -> String {
        let inner = "<m:oMath><m:f><m:fPr><m:type m:val='lin'/></m:fPr><m:num><m:r><m:rPr><m:sty m:val='p'/></m:rPr><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/><w:color w:val='234567'/></w:rPr><m:t>x</m:t></m:r></m:num><m:den><m:r><m:t>y</m:t></m:r></m:den></m:f></m:oMath>";
        if display {
            format!(
                "<m:oMathPara><m:oMathParaPr><m:jc m:val='center'/></m:oMathParaPr>{inner}</m:oMathPara>"
            )
        } else {
            inner.into()
        }
    }
    fn semantic(dom: &Dom, node: NodeId) -> String {
        if !dom.is_element(node) {
            return dom.text_value(node).unwrap_or_default().into();
        }
        let name = dom.name(node).unwrap();
        let mut attrs = dom
            .attributes_iter(node)
            .filter(|(n, _)| {
                n.namespace_name() != "http://www.w3.org/2000/xmlns/" && n.local_name() != "xmlns"
            })
            .collect::<Vec<_>>();
        attrs.sort_by_key(|(n, v)| {
            (
                n.namespace_name().to_string(),
                n.local_name().to_string(),
                *v,
            )
        });
        format!(
            "{:?}{attrs:?}[{}]",
            name,
            dom.nodes(node)
                .into_iter()
                .map(|n| semantic(dom, n))
                .collect::<Vec<_>>()
                .join("|")
        )
    }
    fn parse(content: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}' xmlns:m='{}'><w:body>{content}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>", W::URI, M::URI));
        let root = dom.root(doc).unwrap();
        (dom, root)
    }
    fn assert_math(dom: &Dom, root: NodeId, expected: &str, display: bool) {
        let body = dom.element(root, &W::body()).unwrap();
        let math = dom.elements(
            body,
            Some(&M::name(if display { "oMathPara" } else { "oMath" })),
        );
        assert_eq!(math.len(), 1);
        assert_eq!(semantic(dom, math[0]), expected);
        let kids = dom.elements(body, None);
        assert_eq!(
            kids.iter()
                .map(|&n| dom.name(n).unwrap())
                .collect::<Vec<_>>(),
            vec![
                W::p(),
                M::name(if display { "oMathPara" } else { "oMath" }),
                W::p(),
                W::sect_pr()
            ]
        );
        assert_eq!(dom.value(kids[0]), "before");
        assert_eq!(dom.value(kids[2]), "after");
    }
    #[test]
    fn equation_is_an_opaque_block_barrier_for_deleted_paragraph_marks() {
        for display in [false, true] {
            let math = equation(display);
            let (mut dom, root) = parse(&format!(
                "<w:p><w:pPr><w:rPr><w:del w:id='7' w:author='Comparer'/></w:rPr></w:pPr><w:r><w:t>before</w:t></w:r></w:p>{math}<w:p><w:r><w:t>after</w:t></w:r></w:p>"
            ));
            let body = dom.element(root, &W::body()).unwrap();
            let source_math = dom.elements(body, None)[1];
            let expected = semantic(&dom, source_math);
            let chain = iterate_block_content_elements(&dom, body);
            assert_eq!(chain.len(), 3);
            assert_eq!(chain[1].this_block_content_element, Some(source_math));
            let result = accept_deleted_and_move_from_paragraph_marks_transform(&mut dom, root);
            assert_math(&dom, result, &expected, display);
        }
    }
    #[test]
    fn public_accept_and_reject_preserve_unchanged_body_math_among_revised_marks() {
        for display in [false, true] {
            for reject in [false, true] {
                let mark = if reject { "ins" } else { "del" };
                let math = equation(display);
                let content = format!(
                    "<w:p><w:pPr><w:rPr><w:{mark} w:id='7' w:author='Comparer' w:date='2001-02-03T04:05:06Z'/></w:rPr></w:pPr><w:r><w:t>before</w:t></w:r></w:p>{math}<w:p><w:r><w:t>after</w:t></w:r></w:p>"
                );
                let (dom, root) = parse(&content);
                let body = dom.element(root, &W::body()).unwrap();
                let expected = semantic(&dom, dom.elements(body, None)[1]);
                let xml = dom.serialize_element(root);
                let mut pkg = PartFs::open(include_bytes!(
                    "../tests/fixtures/word_probes/tokens/cell_a.docx"
                ))
                .unwrap();
                pkg.set_part("word/document.xml", xml.into_bytes());
                let bytes = pkg.to_zip().unwrap();
                let result = if reject {
                    crate::document_comparer::reject_revisions(&bytes)
                } else {
                    crate::document_comparer::accept_revisions(&bytes)
                }
                .unwrap();
                let pkg = PartFs::open(&result).unwrap();
                let mut result_dom = Dom::new();
                let doc =
                    result_dom.parse_xdocument(&pkg.part_string("word/document.xml").unwrap());
                let result_root = result_dom.root(doc).unwrap();
                assert_math(&result_dom, result_root, &expected, display);
                assert!(!has_deleted_or_moved_from_paragraph_mark(
                    &result_dom,
                    result_root
                ));
            }
        }
    }

    #[test]
    fn terminal_internal_equation_revisions_preserve_exact_surviving_paragraph_owners() {
        fn normalize_empty_properties(dom: &mut Dom, root: NodeId) {
            for node in dom.descendants(root, None).into_iter().rev() {
                if [W::p_pr(), W::r_pr()]
                    .iter()
                    .any(|name| dom.name_is(node, name))
                    && dom.nodes(node).is_empty()
                    && dom
                        .attributes(node)
                        .iter()
                        .all(|(name, _)| dom.is_namespace_declaration(name))
                {
                    dom.remove(node);
                }
            }
        }
        let prefix = "<w:p><w:pPr><w:spacing w:after='360'/></w:pPr><w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:t>Authored prefix</w:t></w:r></w:p>";
        let live_equation = "<m:oMath><m:r><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/><w:i/></w:rPr><m:t>live</m:t></m:r></m:oMath>";
        for reject in [false, true] {
            let mark = if reject { "ins" } else { "del" };
            let changed = format!(
                "<m:r><w:{mark} w:id='71' w:author='Equation editor' w:date='2026-01-02T03:04:05Z'><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/><w:color w:val='234567'/></w:rPr><m:t>changed</m:t></w:{mark}></m:r>"
            );
            for (label, equation, surviving) in [
                (
                    "only revised equation",
                    format!("<m:oMath>{changed}</m:oMath>"),
                    String::new(),
                ),
                (
                    "independent live equation",
                    format!("<m:oMath>{changed}</m:oMath>{live_equation}"),
                    live_equation.to_string(),
                ),
                (
                    "independent empty equation",
                    format!("<m:oMath>{changed}</m:oMath><m:oMath/>"),
                    "<m:oMath/>".to_string(),
                ),
                (
                    "authored empty run",
                    format!("<m:oMath>{changed}<m:r/></m:oMath>"),
                    "<m:oMath><m:r/><m:r/></m:oMath>".to_string(),
                ),
            ] {
                let input = format!(
                    "{prefix}<w:p><w:pPr><w:rPr><w:{mark} w:id='72' w:author='Equation editor' w:date='2026-01-02T03:04:05Z'/></w:rPr></w:pPr>{equation}</w:p>"
                );
                let expected = if surviving.is_empty() {
                    prefix.to_string()
                } else {
                    format!("{prefix}<w:p>{surviving}</w:p>")
                };
                let (mut dom, root) = parse(&input);
                let independent = dom.clone_subtree(root);
                let source = dom.serialize_element(independent);
                let result = if reject {
                    reject_revisions_document(&mut dom, root)
                } else {
                    accept_revisions_document(&mut dom, root)
                };
                normalize_empty_properties(&mut dom, result);
                let (mut expected_dom, expected_root) = parse(&expected);
                normalize_empty_properties(&mut expected_dom, expected_root);
                assert_eq!(
                    semantic(&dom, result),
                    semantic(&expected_dom, expected_root),
                    "{label}/reject={reject}: full surviving paragraph, run properties and equation owners"
                );
                assert_eq!(
                    dom.serialize_element(independent),
                    source,
                    "independent authored source remains unchanged"
                );
                let repeated = if reject {
                    reject_revisions_document(&mut dom, result)
                } else {
                    accept_revisions_document(&mut dom, result)
                };
                normalize_empty_properties(&mut dom, repeated);
                assert_eq!(
                    semantic(&dom, repeated),
                    semantic(&expected_dom, expected_root),
                    "idempotent {label}/reject={reject}"
                );
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod deleted_title_equation_barrier_tests {
    use super::*;
    use crate::opc::PartFs;
    fn equation(text: &str, marker: Option<&str>, display: bool) -> String {
        let content = format!(
            "<m:rPr><m:sty m:val='p'/></m:rPr><w:rPr><w:rFonts w:ascii='Cambria Math' w:hAnsi='Cambria Math'/><w:color w:val='234567'/></w:rPr><m:t>{text}</m:t>"
        );
        let content = marker.map_or_else(|| content.clone(), |m| format!("<w:{m} w:id='8' w:author='Comparer' w:date='2001-02-03T04:05:06Z'>{content}</w:{m}>"));
        // The producer tracks the complete mathematical run payload,
        // including m:rPr. Leaving those properties outside the marker
        // authors an independent empty run that resolution must preserve.
        let math = format!("<m:oMath><m:r>{content}</m:r></m:oMath>");
        if display {
            format!(
                "<m:oMathPara><m:oMathParaPr><m:jc m:val='center'/></m:oMathParaPr>{math}</m:oMathPara>"
            )
        } else {
            math
        }
    }
    fn title(text: &str, marker: Option<&str>, revised: bool) -> String {
        let mark = marker.map_or(String::new(), |m| {
            format!(
                "<w:rPr><w:{m} w:id='7' w:author='Comparer' w:date='2001-02-03T04:05:06Z'/></w:rPr>"
            )
        });
        let text_tag = if marker == Some("del") {
            "delText"
        } else {
            "t"
        };
        let run = format!(
            "<w:r><w:rPr><w:b/><w:color w:val='123456'/></w:rPr><w:{text_tag}>{text}</w:{text_tag}></w:r>"
        );
        let run = marker.map_or_else(|| run.clone(), |m| format!("<w:{m} w:id='9' w:author='Comparer' w:date='2001-02-03T04:05:06Z'>{run}</w:{m}>"));
        format!(
            "<w:p><w:pPr><w:spacing w:after='{}'/><w:ind w:left='{}'/>{mark}</w:pPr>{run}</w:p>",
            if revised { 240 } else { 120 },
            if revised { 360 } else { 180 }
        )
    }
    fn parse(content: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}' xmlns:m='{}'><w:body>{content}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>", W::URI, M::URI));
        let root = dom.root(doc).unwrap();
        (dom, root)
    }
    fn semantic(dom: &Dom, node: NodeId) -> String {
        if !dom.is_element(node) {
            return dom.text_value(node).unwrap_or_default().into();
        }
        let name = dom.name(node).unwrap();
        let mut attrs = dom
            .attributes_iter(node)
            .filter(|(n, _)| {
                n.namespace_name() != "http://www.w3.org/2000/xmlns/" && n.local_name() != "xmlns"
            })
            .collect::<Vec<_>>();
        attrs.sort_by_key(|(n, v)| {
            (
                n.namespace_name().to_string(),
                n.local_name().to_string(),
                *v,
            )
        });
        format!(
            "{:?}{attrs:?}[{}]",
            name,
            dom.nodes(node)
                .into_iter()
                .map(|n| semantic(dom, n))
                .collect::<Vec<_>>()
                .join("|")
        )
    }
    #[test]
    fn deleted_title_before_equation_is_removed_without_consuming_the_equation() {
        for display in [false, true] {
            let math = equation("old-equation", Some("del"), display);
            let (mut dom, root) =
                parse(&format!("{}{math}", title("old-title", Some("del"), false)));
            let body = dom.element(root, &W::body()).unwrap();
            let original_math = dom.elements(body, None)[1];
            let expected_math = semantic(&dom, original_math);
            let result = accept_deleted_and_move_from_paragraph_marks_transform(&mut dom, root);
            let body = dom.element(result, &W::body()).unwrap();
            let children = dom.elements(body, None);
            assert_eq!(children.len(), 2);
            assert_eq!(semantic(&dom, children[0]), expected_math);
            assert!(dom.name_is(children[1], &W::sect_pr()));
        }
    }
    #[test]
    fn public_replaced_titles_and_equations_recover_complete_owned_sources() {
        for display in [false, true] {
            let content = format!(
                "{}{}{}{}",
                title("old-title", Some("del"), false),
                equation("old-equation", Some("del"), display),
                title("new-title", Some("ins"), true),
                equation("new-equation", Some("ins"), display)
            );
            let (dom, root) = parse(&content);
            let mut pkg = PartFs::open(include_bytes!(
                "../tests/fixtures/word_probes/tokens/cell_a.docx"
            ))
            .unwrap();
            pkg.set_part(
                "word/document.xml",
                dom.serialize_element(root).into_bytes(),
            );
            let bytes = pkg.to_zip().unwrap();
            for reject in [false, true] {
                let output = if reject {
                    crate::document_comparer::reject_revisions(&bytes)
                } else {
                    crate::document_comparer::accept_revisions(&bytes)
                }
                .unwrap();
                let output = PartFs::open(&output).unwrap();
                let mut actual = Dom::new();
                let doc = actual.parse_xdocument(&output.part_string("word/document.xml").unwrap());
                let root = actual.root(doc).unwrap();
                let body = actual.element(root, &W::body()).unwrap();
                let source = if reject {
                    format!(
                        "{}{}",
                        title("old-title", None, false),
                        equation("old-equation", None, display)
                    )
                } else {
                    format!(
                        "{}{}",
                        title("new-title", None, true),
                        equation("new-equation", None, display)
                    )
                };
                let (expected, root) = parse(&source);
                let expected_body = expected.element(root, &W::body()).unwrap();
                assert_eq!(
                    semantic(&actual, body),
                    semantic(&expected, expected_body),
                    "display={display} reject={reject}"
                );
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod complete_field_instruction_metadata_boundary_tests {
    use super::*;
    #[test]
    fn repaired_complete_field_codes_preserve_source_comments_processing_instructions_and_formats()
    {
        for marker in ["ins", "del"] {
            for code_pieces in [1usize, 2] {
                let mut dom = Dom::new();
                let codes = if code_pieces == 1 {
                    vec![" REF Clause "]
                } else {
                    vec![" REF ", "Clause "]
                };
                let codes=codes.iter().map(|code|format!("<w:r><w:rPr><!--authored run format--><?owner keep?><w:b/><w:color w:val='123456'/></w:rPr><w:instrText xml:space='preserve'>{code}</w:instrText></w:r>")).collect::<String>();
                let doc=dom.parse_xdocument(&format!("<w:p xmlns:w='{}'><w:pPr><w:spacing w:after='120'/></w:pPr><w:{marker} w:id='11' w:author='Field owner' w:date='2001-02-03T04:05:06Z'><w:r><w:fldChar w:fldCharType='begin'/></w:r></w:{marker}>{codes}<w:{marker} w:id='12' w:author='Field owner' w:date='2001-02-03T04:05:06Z'><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:{}>cached clause</w:{}></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r></w:{marker}><w:bookmarkStart w:id='31' w:name='Clause'/><w:bookmarkEnd w:id='31'/></w:p>",W::URI,if marker=="del" {"delText"} else {"t"},if marker=="del" {"delText"} else {"t"}));
                let root = dom.root(doc).unwrap();
                let source = dom.serialize_element(root);
                let source_codes = dom
                    .elements(root, Some(&W::r()))
                    .iter()
                    .map(|&run| dom.serialize_element(run))
                    .collect::<Vec<_>>();
                let children = dom.elements(root, None);
                let first = dom.serialize_element(children[1]);
                let last = dom.serialize_element(children[children.len() - 3]);
                // Transform helpers may transfer clean child nodes. As the
                // package pipeline does, repair an independent working copy.
                let working = dom.clone_subtree(root);
                let result = fix_up_deleted_or_inserted_field_codes_transform(&mut dom, working);
                let wrappers = dom.elements(result, Some(&W::name(marker)));
                assert_eq!(wrappers.len(), 3);
                assert_eq!(dom.serialize_element(wrappers[0]), first);
                assert_eq!(dom.serialize_element(wrappers[2]), last);
                assert!(
                    !dom.has_attributes(wrappers[1]),
                    "A.1 generated field-code wrapper has no invented attribution"
                );
                let actual_codes = dom
                    .elements(wrappers[1], Some(&W::r()))
                    .iter()
                    .map(|&run| dom.serialize_element(run))
                    .collect::<Vec<_>>();
                let expected_codes = if marker == "del" {
                    source_codes
                        .iter()
                        .map(|s| s.replace("instrText", "delInstrText"))
                        .collect::<Vec<_>>()
                } else {
                    source_codes
                };
                assert_eq!(
                    actual_codes, expected_codes,
                    "only the field instruction tag changes; all mixed source nodes and formats survive"
                );
                assert_eq!(
                    dom.descendants(result, Some(&W::fld_char()))
                        .iter()
                        .map(|&node| dom.attribute(node, &W::name("fldCharType")).unwrap())
                        .collect::<Vec<_>>(),
                    vec!["begin", "separate", "end"]
                );
                assert_eq!(
                    dom.attribute(
                        dom.element(result, &W::bookmark_start()).unwrap(),
                        &W::name("name")
                    ),
                    Some("Clause")
                );
                assert_eq!(
                    dom.serialize_element(root),
                    source,
                    "repair uses independent nodes, leaving source owners intact"
                );
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod alternate_content_paragraph_owner_tests {
    use super::*;
    use crate::namespaces::{A, MC, W14};

    fn root(dom: &mut Dom, body: &str) -> NodeId {
        let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}' xmlns:mc='{}' xmlns:w14='{}' xmlns:a='{}' mc:Ignorable='w14'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>", W::URI, MC::URI, W14::URI, A::URI));
        dom.root(doc).unwrap()
    }

    fn semantic(dom: &Dom, node: NodeId, out: &mut Vec<String>) {
        if dom.is_element(node) {
            let name = dom.name(node).unwrap();
            let mut attrs = dom
                .attributes_iter(node)
                .filter(|(n, _)| !dom.is_namespace_declaration(n))
                .map(|(n, v)| (n.clark(), v))
                .collect::<Vec<_>>();
            attrs.sort();
            if [W::p_pr(), W::r_pr()].contains(&name)
                && attrs.is_empty()
                && dom.nodes(node).is_empty()
            {
                return;
            }
            out.push(format!("begin:{}:{attrs:?}", name.clark()));
            for child in dom.nodes(node) {
                semantic(dom, child, out);
            }
            out.push(format!("end:{}", name.clark()));
        } else if dom.is_text(node) {
            out.push(format!("text:{:?}", dom.text_value(node)));
        } else {
            out.push(dom.serialize_element(node));
        }
    }

    #[test]
    fn authored_property_only_alternatives_survive_but_wholly_deleted_wrappers_do_not() {
        let payload = "<mc:AlternateContent><mc:Choice Requires='w14'><w:r><w:rPr><w:b/><w14:textFill><a:solidFill><a:srgbClr val='123456'/></a:solidFill></w14:textFill></w:rPr></w:r></mc:Choice><mc:Fallback><w:r><w:rPr><w:lang w:val='fr-FR'/></w:rPr></w:r></mc:Fallback></mc:AlternateContent>";
        let first_props = "<w:pPr><w:spacing w:before='120'/></w:pPr>";
        let closing_props = "<w:pPr><w:spacing w:after='240'/><w:jc w:val='right'/></w:pPr>";
        let tail = "<w:r><w:rPr><w:i/></w:rPr><w:t>Independent tail</w:t></w:r>";
        for wholly_deleted in [false, true] {
            for reject in [false, true] {
                let content = if wholly_deleted {
                    format!(
                        "<w:del w:id='42' w:author='Original owner' w:date='2000-01-01T00:00:00Z'>{payload}</w:del>"
                    )
                } else {
                    payload.to_owned()
                };
                let input = format!(
                    "<w:p><w:pPr><w:spacing w:before='120'/><w:rPr><w:del w:id='41' w:author='Original owner' w:date='2000-01-01T00:00:00Z'/></w:rPr></w:pPr>{content}</w:p><w:p>{closing_props}{tail}</w:p>"
                );
                let expected = if reject {
                    format!("<w:p>{first_props}{payload}</w:p><w:p>{closing_props}{tail}</w:p>")
                } else {
                    format!(
                        "<w:p>{closing_props}{}{tail}</w:p>",
                        if wholly_deleted { "" } else { payload }
                    )
                };
                let mut dom = Dom::new();
                let authored = root(&mut dom, &input);
                let actual = if reject {
                    reject_revisions_document(&mut dom, authored)
                } else {
                    accept_revisions_document(&mut dom, authored)
                };
                let expected = root(&mut dom, &expected);
                let mut actual_events = Vec::new();
                let mut expected_events = Vec::new();
                semantic(&dom, actual, &mut actual_events);
                semantic(&dom, expected, &mut expected_events);
                assert_eq!(
                    actual_events, expected_events,
                    "wholly_deleted={wholly_deleted} reject={reject}: authored alternatives preserve exact effects/language even without text; outer revision remains authoritative"
                );
            }
        }
    }

    #[test]
    fn surviving_alternate_content_keeps_both_presentations_across_deleted_paragraph_marks() {
        let payload = "<mc:AlternateContent><mc:Choice Requires='w14'><w:r><w:rPr><w:b/><w14:textFill><a:solidFill><a:srgbClr val='123456'/></a:solidFill></w14:textFill></w:rPr><w:t>Native presentation</w:t><w:tab/></w:r></mc:Choice><mc:Fallback><w:r><w:rPr><w:color w:val='654321'/></w:rPr><w:t>Fallback presentation</w:t><w:br/></w:r></mc:Fallback></mc:AlternateContent>";
        let closing_props = "<w:pPr><w:spacing w:after='240'/><w:jc w:val='right'/></w:pPr>";
        let tail = "<w:r><w:rPr><w:i/></w:rPr><w:t>Independent tail</w:t></w:r>";
        for moved in [false, true] {
            for reject in [false, true] {
                let marker = if moved { "moveFrom" } else { "del" };
                let first_props = "<w:pPr><w:spacing w:before='120'/></w:pPr>";
                let marked_props = format!(
                    "<w:pPr><w:spacing w:before='120'/><w:rPr><w:{marker} w:id='41' w:author='Original owner' w:date='2000-01-01T00:00:00Z'/></w:rPr></w:pPr>"
                );
                let input =
                    format!("<w:p>{marked_props}{payload}</w:p><w:p>{closing_props}{tail}</w:p>");
                let expected = if reject {
                    format!("<w:p>{first_props}{payload}</w:p><w:p>{closing_props}{tail}</w:p>")
                } else {
                    format!("<w:p>{closing_props}{payload}{tail}</w:p>")
                };
                let mut dom = Dom::new();
                let authored = root(&mut dom, &input);
                let independent = dom.clone_subtree(authored);
                let snapshot = dom.serialize_element(independent);
                let actual = if reject {
                    reject_revisions_document(&mut dom, authored)
                } else {
                    accept_revisions_document(&mut dom, authored)
                };
                let expected = root(&mut dom, &expected);
                let mut actual_events = Vec::new();
                let mut expected_events = Vec::new();
                semantic(&dom, actual, &mut actual_events);
                semantic(&dom, expected, &mut expected_events);
                assert_eq!(
                    actual_events, expected_events,
                    "moved={moved} reject={reject}: exact source alternatives, authored effects and final paragraph owner"
                );
                assert_eq!(dom.serialize_element(independent), snapshot);
            }
        }
    }
}
