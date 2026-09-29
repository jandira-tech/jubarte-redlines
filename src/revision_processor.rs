// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Port of `RevisionProcessor.ts` — accept-revisions pipeline (M3).
//!
//! Scope (per the implementation plan): the ACCEPT path only. Reject,
//! consolidate, and the HTML/markdown surfaces are out of scope.

mod annotation_ids;
mod bookmarks;
mod comments;
mod notes;
mod sections;
pub(crate) mod style_records;
mod word_save;

pub(crate) use sections::Resolution;

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
///
/// ACCEPT-REUSE-CLEAN: subtrees with no moveFrom/moveTo (and no other tracked
/// revision markers) are detached and returned as-is — `Dom::add` then reuses
/// them without `clone_subtree`.
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
            vec![]
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

/// Port of `AcceptAllOtherRevisionsTransform` — accept inserts (unwrap `w:ins`),
/// drop deletions and revision-range markers, accept formatting-change markers,
/// handle deleted rows/tables, cell merges, and empty hyperlink shells.
///
/// ACCEPT-REUSE-CLEAN: when a subtree has no tracked-revision elements, detach
/// and return it instead of `clone_subtree` + identity rebuild. Callers reparent
/// via `Dom::add`, which reuses unparented nodes.
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
    if name == W::tr() && has_path(dom, node, &W::tr_pr(), &W::del()) {
        return vec![];
    }

    // w:tbl whose rows are ALL deleted → drop the whole table.
    if name == W::tbl() {
        let rows = dom.elements(node, Some(&W::tr()));
        if !rows.is_empty()
            && rows
                .iter()
                .all(|&tr| has_path(dom, tr, &W::tr_pr(), &W::del()))
        {
            return vec![];
        }
    }

    // Accept deleted text: drop w:del. Hoist comment range markers that lived
    // inside the deletion so nested/table comment anchors survive accept
    // (docx_lots_of_comments redline: starts 9/10 sit between delText runs;
    // dropping the whole w:del orphaned those comments → carry 2/6), and the
    // bookmarks left there: `bookmarks::drop_wholly_deleted_bookmarks` has
    // already removed those whose every character went.
    if name == W::del() {
        return hoist_range_markers_from(dom, node);
    }

    // Vertically-merged cell markers.
    if name == W::cell_merge() {
        let parent_is_tcpr = dom
            .parent(node)
            .and_then(|p| dom.name(p))
            .is_some_and(|pn| pn == W::tc_pr());
        if parent_is_tcpr {
            let vmerge = dom.attribute(node, &W::v_merge()).map(|s| s.to_string());
            if vmerge.as_deref() == Some("rest") {
                let v = dom.new_element(W::v_merge());
                dom.set_attribute_value(v, &W::val(), Some("restart"));
                return vec![v];
            }
            if vmerge.as_deref() == Some("cont") {
                let v = dom.new_element(W::v_merge());
                dom.set_attribute_value(v, &W::val(), Some("continue"));
                return vec![v];
            }
        }
    }

    // w:hyperlink that collapses to an empty shell after accepting children → drop.
    if name == W::hyperlink() {
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
/// REJECT-SKIP-01: when the subtree has no tracked-revision elements, the
/// reject/reverse/accept rebuilds are identity — only RemoveRsid is needed
/// (in-place). Dirty trees take the full faithful path.
pub fn reject_revisions_document(dom: &mut Dom, root: NodeId) -> NodeId {
    if !element_has_tracked_revisions(dom, root) {
        return remove_rsid_transform(dom, root).expect("reject clean: root not dropped by rsid");
    }
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
    /// Public API item.
    Element,
    /// Public API item.
    EmptyElement,
    /// Public API item.
    EndElement,
}

/// `Tag` (RevisionProcessor.cs :2393): one open/empty/close event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag {
    /// `element`.
    pub element: NodeId,
    /// `tag_type`.
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

/// `BlockContentInfo` (RevisionProcessor.cs :52): prev/this/next links for
/// block-level content. `iterate_block_content_elements` fills all three
/// (`this` always `Some`); `get_paragraph_info` fills `previous` (= previous
/// element sibling) and `this` only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockContentInfo {
    /// `previous_block_content_element`.
    pub previous_block_content_element: Option<NodeId>,
    /// `this_block_content_element`.
    pub this_block_content_element: Option<NodeId>,
    /// `next_block_content_element`.
    pub next_block_content_element: Option<NodeId>,
}

/// First `w:p`/`w:tbl` among `roots`' descendants-and-self, document order.
/// DOM-ITER-04: early-exit iterative walk (no full `descendants_and_self` Vec).
fn first_block_content(dom: &Dom, roots: &[NodeId]) -> Option<NodeId> {
    let (p, tbl) = (W::p(), W::tbl());
    let is_block = |e: NodeId| dom.name(e).is_some_and(|n| n == p || n == tbl);
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
/// (:1855): the doc-order chain of block content (`w:p`/`w:tbl`) under
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

/// `W.BlockLevelContentContainers` (PtOpenXmlUtil.cs :5569).
fn block_level_content_containers() -> [XName; 7] {
    [
        W::body(),
        W::tc(),
        W::txbx_content(),
        W::hdr(),
        W::ftr(),
        W::endnote(),
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
    let to_delete: HashSet<NodeId> = start_tags_in_range
        .into_iter()
        .filter(|&e| {
            end_set.contains(&e)
                && !is_container_property(dom, e)
                && !dom
                    .ancestors(e, None)
                    .into_iter()
                    .any(|a| is_container_property(dom, a))
        })
        .collect();
    if to_delete.is_empty() {
        return document;
    }
    accept_move_from_ranges_transform(dom, document, &to_delete)
        .expect("the document root is never in a moveFrom range")
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
        is_run_content(&n)
            .unwrap_or_else(|| panic!("Internal error 20, found element {}", n.clark()))
    })
}

/// Does every row of `tbl` carry `trPr/w:del` (and is there a row)?
fn table_rows_all_deleted(dom: &Dom, tbl: NodeId) -> bool {
    let rows = dom.elements(tbl, Some(&W::tr()));
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
        dom.element(node, &W::sect_pr())
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
            for ppr in dom.elements(last_this, Some(&W::p_pr())) {
                let c = dom.clone_subtree(ppr);
                dom.add(np, c);
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
            // Word also drops an emptied moved-away paragraph before a table
            // (its Reject All of 30f20e787b's moved heading).
            let last_mark_goes = paragraph_mark_is_deleted_or_moved_from(dom, last_this);
            let next = group.last().unwrap().0.next_block_content_element;
            let next_is_none_or_tbl =
                next.is_none() || next.is_some_and(|n| dom.name(n) == Some(W::tbl()));
            if all_para_content_is_deleted(dom, np) && last_mark_goes && next_is_none_or_tbl {
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
                for c in dom.nodes(this) {
                    let tc = accept_deleted_and_move_from_paragraph_marks_transform(dom, c);
                    dom.add(re, tc);
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

/// Pull comment range and bookmark markers out of a subtree being discarded
/// (e.g. accepted `w:del`) so anchors between delText runs are not lost.
fn hoist_range_markers_from(dom: &mut Dom, node: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    for e in dom.descendants(node, None) {
        let Some(n) = dom.name(e) else {
            continue;
        };
        if n.namespace_name() == W::URI
            && matches!(
                n.local_name(),
                "commentRangeStart" | "commentRangeEnd" | "bookmarkStart" | "bookmarkEnd"
            )
        {
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
        // No run left to anchor it: the control stands on the paragraphs
        // that kept their marks, emptied.
        if runs_in_new_document.is_empty() && !block_removed {
            runs_in_new_document = paragraphs
                .iter()
                .filter_map(|&p| dom.attribute(p, &unique_id))
                .filter_map(|id| paragraph_by_id.get(id).copied())
                .filter(|&p| dom.ancestors(p, None).contains(&new_document))
                .collect();
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
    let new_element = accept_deleted_and_move_from_paragraph_marks_transform(dom, element);
    add_block_level_content_controls(dom, new_element, element)
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
    for tr in dom.elements(new_tbl, Some(&W::tr())) {
        let mut last_used: i64 = -1;
        for tc in dom.elements(tr, Some(&W::tc())) {
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
            for tr in dom.elements(fixed, Some(&W::tr())) {
                let new_row = dom.new_element(W::tr());
                for (an, av) in dom.attributes(tr) {
                    dom.set_attribute_value(new_row, &an, Some(&av));
                }
                if !own_ex.is_empty() {
                    let ex = row_tbl_pr_ex(dom, tr, &own_ex);
                    dom.add(new_row, ex);
                }
                let non_cells: Vec<NodeId> = dom
                    .elements(tr, None)
                    .into_iter()
                    .filter(|&e| {
                        dom.name(e) != Some(W::tc())
                            && (own_ex.is_empty() || dom.name(e) != Some(W::name("tblPrEx")))
                    })
                    .collect();
                for e in non_cells {
                    let c = dom.clone_subtree(e);
                    dom.add(new_row, c);
                }
                for tc in dom.elements(tr, Some(&W::tc())) {
                    let w: Option<i64> = dom
                        .element(tc, &W::tc_pr())
                        .and_then(|p| dom.element(p, &W::name("tcW")))
                        .and_then(|t| dom.attribute(t, &W::name("w")))
                        .and_then(|v| v.parse().ok());
                    let Some(w) = w else {
                        let c = dom.clone_subtree(tc);
                        dom.add(new_row, c);
                        continue;
                    };
                    let mut width_to_left = 0i64;
                    for btc in dom.elements(tr, Some(&W::tc())) {
                        if btc == tc {
                            break;
                        }
                        width_to_left += dom
                            .element(btc, &W::tc_pr())
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
    let has_rev = element_has_tracked_revisions(dom, root);
    let e = remove_rsid_transform(dom, root).expect("root not dropped by rsid removal");
    let e = if has_rev {
        bookmarks::drop_wholly_deleted_bookmarks(dom, e);
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

/// The parts `AcceptRevisions`/`RejectRevisions` (:1277/:31) walk, in the C#
/// order: main, headers, footers, endnotes, footnotes, then styles (flagged).
/// The main document's numbering part, when it has one.
fn numbering_part(pkg: &crate::opc::PartFs) -> Option<String> {
    let main = pkg.main_document_part()?;
    let rels = pkg.read_rels_for(&main)?;
    rels.items
        .iter()
        .find(|r| {
            r.target_mode.as_deref() != Some("External")
                && r.rel_type.rsplit('/').next() == Some("numbering")
        })
        .map(|r| pkg.resolve_rel_target(&main, &r.target))
}

pub(crate) fn revision_bearing_parts(pkg: &crate::opc::PartFs) -> Vec<(String, bool)> {
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

/// Called with each revision-bearing part's name and parsed root before it
/// is resolved; renames the revisions to keep into [`FROZEN_NS`].
pub(crate) type Freeze<'a> = &'a dyn Fn(&str, &mut Dom, NodeId);

/// Accept or reject every revision `freeze` leaves in place. Annotation ids
/// are renumbered only when no revision is left, so the ids of the ones kept
/// still name them.
pub(crate) fn resolve_package(
    pkg: &mut crate::opc::PartFs,
    resolution: Resolution,
    freeze: Option<Freeze<'_>>,
) {
    let parts = revision_bearing_parts(pkg);
    let levels = numbering_part(pkg)
        .and_then(|p| pkg.part_string(&p))
        .map(|xml| style_records::Levels::parse(&xml))
        .unwrap_or_default();
    let mut kept = false;
    for (part, is_styles) in parts.clone() {
        process_part(pkg, &part, |dom, root| {
            if let Some(freeze) = freeze {
                freeze(&part, dom, root);
            }
            let resolved = match (is_styles, resolution) {
                (true, Resolution::Accept) => accept_revisions_for_styles_transform(dom, root),
                (true, Resolution::Reject) => {
                    let recorded = style_records::recorded_blocks(dom, root);
                    let rejected = reject_revisions_for_styles_transform(dom, root)?;
                    style_records::restore_against_built_ins(dom, rejected, &recorded, &levels);
                    accept_revisions_for_styles_transform(dom, rejected)
                }
                (false, _) => {
                    sections::carry_vanishing_section_references(dom, root, resolution);
                    Some(match resolution {
                        Resolution::Accept => accept_revisions_for_part_content(dom, root),
                        Resolution::Reject => reject_revisions_document(dom, root),
                    })
                }
            }?;
            kept |= thaw(dom, resolved);
            Some(resolved)
        });
    }
    let stories = story_parts(&parts);
    comments::prune_orphan_comments(pkg, &stories);
    notes::prune_orphan_notes(pkg, &stories);
    if !kept {
        annotation_ids::renumber(pkg, &stories);
    }
    word_save::tidy(pkg, &stories);
}

/// Rename every frozen revision element under `root` back into `w:`; true
/// when there was one.
fn thaw(dom: &mut Dom, root: NodeId) -> bool {
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
    !frozen.is_empty()
}

/// The content parts of [`revision_bearing_parts`] (styles left out).
fn story_parts(parts: &[(String, bool)]) -> Vec<String> {
    parts
        .iter()
        .filter(|(_, is_styles)| !is_styles)
        .map(|(p, _)| p.clone())
        .collect()
}
