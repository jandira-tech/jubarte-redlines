// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Produce tracked-revision markup (M4.4). Core of
//! `ProduceDocumentWithTrackedRevisions` for the paragraph-text case.
//!
//! Consumes the LCS-tagged atom stream and rebuilds a `<w:document>` where
//! inserted content is wrapped in `<w:ins>` and deleted content in `<w:del>`
//! (with `<w:t>` → `<w:delText>`), each carrying `w:id`/`w:author`/`w:date`.
//!
//! NOTE: the full TS producer additionally handles inserted/deleted paragraph
//! marks (paragraph merge/split), tables, footnotes, moves, and format changes
//! (WmlComparer.ts:2222+, plus fixups). This core covers runs of text within
//! paragraphs — the common case — and is the base those refinements extend.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::namespaces::{PT, W};
use crate::util::group_adjacent;
use crate::xmllinq::{Dom, NodeId, XNamespace};

use super::lcs::TaggedAtom;
use super::{CorrelationStatus, WmlComparerSettings};

static REV_ID: AtomicU64 = AtomicU64::new(1);

/// Deleted opaque subtrees (drawings, text boxes, `mc:AlternateContent`) are
/// cloned verbatim, so their nested text still reads `w:t`. `w:t` inside `w:del`
/// is non-conformant — Word writes `w:delText`. Rename every descendant `w:t`
/// in the cloned subtree to `w:delText` for pure deletions only.
///
/// **MovedSource:** Word Compare keeps `w:t` inside `w:moveFrom` (broken_ones_two
/// oracle). Renaming to `delText` invited nested `w:del` (wrap_bare) and Word's
/// "unreadable content" dialog. Inserted / moved-destination keep `w:t`.
/// `w:instrText` is left untouched — the `W::t()` filter excludes it.
fn delete_text_in_opaque(dom: &mut Dom, node: NodeId, status: CorrelationStatus) {
    if matches!(status, CorrelationStatus::Deleted) {
        // Hoist the name out of the loop — `W::name` allocates a fresh `XName`
        // on each call; `XName` is `Arc`-cheap to clone.
        let del_text = W::del_text();
        for t in dom.descendants(node, Some(&W::t())) {
            dom.set_name(t, del_text.clone());
        }
    }
}

fn next_rev_id() -> String {
    REV_ID.fetch_add(1, Ordering::Relaxed).to_string()
}

/// Build the redline `<w:document>` node from the tagged atom stream.
pub fn produce_document(
    dom: &mut Dom,
    tagged: &[TaggedAtom],
    settings: &WmlComparerSettings,
) -> NodeId {
    let doc = dom.new_document();
    let document = dom.new_element(W::document());
    dom.set_attribute_value(document, &XNamespace::xmlns().name("w"), Some(W::URI));
    dom.set_attribute_value(document, &XNamespace::xmlns().name("pt14"), Some(PT::URI));
    let body = dom.new_element(W::body());

    // Split the stream into paragraphs: a pPr atom ends a paragraph.
    let mut para: Vec<TaggedAtom> = Vec::new();
    for t in tagged {
        let is_ppr = dom.name_is(t.atom.content_element, &W::p_pr());
        if is_ppr {
            let p = build_paragraph(dom, &para, t, settings);
            dom.add(body, p);
            para.clear();
        } else {
            para.push(t.clone());
        }
    }
    // trailing content with no paragraph mark
    if !para.is_empty() {
        let synthetic = TaggedAtom {
            atom: para[0].atom.clone(),
            status: CorrelationStatus::Equal,
        };
        let p = build_paragraph(dom, &para, &synthetic, settings);
        dom.add(body, p);
    }

    dom.add(document, body);
    dom.add(doc, document);
    doc
}

/// Build one `<w:p>` from its run atoms + the paragraph-mark atom.
fn build_paragraph(
    dom: &mut Dom,
    run_atoms: &[TaggedAtom],
    ppr_atom: &TaggedAtom,
    settings: &WmlComparerSettings,
) -> NodeId {
    let p = dom.new_element(W::p());

    // Carry the paragraph's pPr (clone the content element if it's a real pPr).
    if dom.name_is(ppr_atom.atom.content_element, &W::p_pr())
        && dom.has_elements(ppr_atom.atom.content_element)
    {
        let ppr = dom.clone_subtree(ppr_atom.atom.content_element);
        dom.add(p, ppr);
    }

    // Group consecutive atoms by status, emit runs wrapped per status.
    let groups = group_adjacent(run_atoms.iter().cloned(), |t| t.status);
    for (status, group) in groups {
        // Concatenate the text of this status-run (text atoms only).
        let text: String = group
            .iter()
            .filter(|t| {
                let n = dom.name(t.atom.content_element);
                n == Some(W::t()) || n == Some(W::del_text())
            })
            .map(|t| dom.value_str(t.atom.content_element).into_owned())
            .collect();
        if text.is_empty() {
            continue;
        }
        match status {
            CorrelationStatus::Inserted => {
                let ins = wrap_run(dom, &text, false, settings, CorrelationStatus::Inserted);
                dom.add(p, ins);
            }
            CorrelationStatus::Deleted => {
                let del = wrap_run(dom, &text, true, settings, CorrelationStatus::Deleted);
                dom.add(p, del);
            }
            _ => {
                // Equal: a plain run.
                let r = build_text_run(dom, &text, false);
                dom.add(p, r);
            }
        }
    }
    p
}

/// Build `<w:r><w:t>text</w:t></w:r>` (or delText when `deleted`).
fn build_text_run(dom: &mut Dom, text: &str, deleted: bool) -> NodeId {
    let r = dom.new_element(W::r());
    let t = dom.new_element(if deleted { W::del_text() } else { W::t() });
    if text.starts_with(' ') || text.ends_with(' ') {
        dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
    }
    dom.add_text(t, text);
    dom.add(r, t);
    r
}

/// Wrap a run in `<w:ins>`/`<w:del>` with id/author/date.
fn wrap_run(
    dom: &mut Dom,
    text: &str,
    deleted: bool,
    settings: &WmlComparerSettings,
    status: CorrelationStatus,
) -> NodeId {
    let wrapper_name = if matches!(status, CorrelationStatus::Deleted) {
        W::del()
    } else {
        W::ins()
    };
    let wrapper = dom.new_element(wrapper_name);
    dom.set_attribute_value(wrapper, &W::id(), Some(&next_rev_id()));
    dom.set_attribute_value(wrapper, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(wrapper, &W::date(), Some(&settings.date_time_for_revisions));
    let r = build_text_run(dom, text, deleted);
    dom.add(wrapper, r);
    wrapper
}

// ─────────────────────────────────────────────────────────────────────────────
// M4.E — faithful reassembly: Flatten → AssembleAncestorUnids → CoalesceRecurse.
// (Added alongside the M4.4 shortcut producer above, which stays until M4.I.)
// ─────────────────────────────────────────────────────────────────────────────

use super::atoms::{ComparisonUnit, ComparisonUnitAtom, CorrelatedSequence};
use crate::unid::generate_unid;

fn flatten_atoms(units: &[ComparisonUnit]) -> Vec<ComparisonUnitAtom> {
    units
        .iter()
        .flat_map(|u| u.descendant_atoms().into_iter().cloned())
        .collect()
}

/// True when a before-side atom carries `pt:PreDelete="orig"` on itself or an
/// ancestor (word-mode flattened A-only pre-existing deletion). Equal emit
/// would drop the stamp (content comes from AFTER); force del+ins instead.
fn atom_has_predelete_orig(dom: &Dom, atom: &ComparisonUnitAtom) -> bool {
    let pre = PT::name("PreDelete");
    if dom.attribute(atom.content_element, &pre) == Some(crate::comparer::PREDELETE_STAMP_ORIG) {
        return true;
    }
    atom.ancestor_elements
        .iter()
        .any(|&a| dom.attribute(a, &pre) == Some(crate::comparer::PREDELETE_STAMP_ORIG))
}

/// M4.E.1 — `FlattenToComparisonUnitAtomList` (:4141): nested correlated tree →
/// flat status-tagged atom list. Equal carries content/ancestors from the AFTER
/// atom and a link to the BEFORE atom; zip truncates to the shorter side.
///
/// M-MOVE S1 exception: when the BEFORE atom is a PreDelete-orig span, emit
/// Deleted(before)+Inserted(after) instead of Equal so history survives even
/// if an upstream correlation path lost the salt (m36 / fresh-p4).
pub fn flatten_to_comparison_unit_atom_list(
    dom: &Dom,
    seqs: &[CorrelatedSequence],
) -> Vec<ComparisonUnitAtom> {
    let mut out = Vec::new();
    for cs in seqs {
        match cs.correlation_status {
            CorrelationStatus::Equal => {
                let before = flatten_atoms(cs.com_units_1.as_deref().unwrap_or(&[]));
                let after = flatten_atoms(cs.com_units_2.as_deref().unwrap_or(&[]));
                // M-MOVE S1: if any BEFORE atom is a PreDelete-orig span, emit
                // the whole before run as Deleted and the whole after run as
                // Inserted (paragraph/word granularity). Per-atom del+ins
                // confetti fails convert_stamped coalescing and m36.
                if before.iter().any(|b| atom_has_predelete_orig(dom, b)) {
                    for b in &before {
                        let mut del = b.clone();
                        del.correlation_status = CorrelationStatus::Deleted;
                        out.push(del);
                    }
                    for a in &after {
                        let mut ins = a.clone();
                        ins.correlation_status = CorrelationStatus::Inserted;
                        out.push(ins);
                    }
                    continue;
                }
                for (b, a) in before.iter().zip(after.iter()) {
                    let mut atom = a.clone();
                    atom.correlation_status = CorrelationStatus::Equal;
                    atom.content_element_before = Some(b.content_element);
                    atom.comparison_unit_atom_before = Some(std::sync::Arc::new(b.clone()));
                    out.push(atom);
                }
            }
            CorrelationStatus::Deleted => {
                for a in flatten_atoms(cs.com_units_1.as_deref().unwrap_or(&[])) {
                    let mut x = a;
                    x.correlation_status = CorrelationStatus::Deleted;
                    out.push(x);
                }
            }
            CorrelationStatus::Inserted => {
                for a in flatten_atoms(cs.com_units_2.as_deref().unwrap_or(&[])) {
                    let mut x = a;
                    x.correlation_status = CorrelationStatus::Inserted;
                    out.push(x);
                }
            }
            other => panic!("Internal error: unexpected status in flatten: {other:?}"),
        }
    }
    out
}

/// Authored control identifiers distinguish an existing inner control from a
/// newly introduced outer wrapper. Prefer a stable tag, then id, then alias;
/// Word can regenerate ids without changing the authored tag. QName alone
/// pairs the old inner control with the new outer one and discards its metadata.
fn same_authored_control(dom: &Dom, a: NodeId, b: NodeId) -> bool {
    let properties = W::name("sdtPr");
    let (Some(a), Some(b)) = (dom.element(a, &properties), dom.element(b, &properties)) else {
        return false;
    };
    for name in [W::name("tag"), W::id(), W::name("alias")] {
        let value = |node| {
            dom.element(node, &name)
                .and_then(|child| dom.attribute(child, &W::val()))
        };
        if let (Some(a), Some(b)) = (value(a), value(b))
            && !a.is_empty()
            && !b.is_empty()
        {
            return a == b;
        }
    }
    false
}

/// Resolve repeated authored tags by preferring the retained control's exact
/// id within the same transparent ancestry segment. A tag-only match remains
/// valid when Word regenerated the id and no stronger candidate exists.
fn authored_control_match_strength(dom: &Dom, a: NodeId, b: NodeId) -> u8 {
    if !same_authored_control(dom, a, b) {
        return 0;
    }
    let control_id = |node| {
        dom.element(node, &W::name("sdtPr"))
            .and_then(|pr| dom.element(pr, &W::id()))
            .and_then(|id| dom.attribute(id, &W::val()))
            .filter(|id| !id.is_empty())
    };
    if let (Some(a), Some(b)) = (control_id(a), control_id(b))
        && a == b
    {
        2
    } else {
        1
    }
}

/// Flatten only unmatched content-control ancestry in correlated paragraphs.
/// Equal atoms otherwise use B's control bucket while A's deletion islands use
/// a separate paragraph bucket, which moves every deleted word to the end.
/// Keep aligned controls intact, and require every non-control ancestor to
/// match so table, cell and text-box boundaries cannot be crossed.
fn align_content_control_ancestry(dom: &Dom, atoms: &mut [ComparisonUnitAtom]) {
    let sdt = W::sdt();
    let content = W::sdt_content();
    let paragraph = W::p();
    let is_control = |node| dom.name_is(node, &sdt) || dom.name_is(node, &content);
    let mut unmatched = std::collections::HashSet::new();
    for atom in atoms.iter() {
        if atom.correlation_status != CorrelationStatus::Equal {
            continue;
        }
        let Some(before) = &atom.comparison_unit_atom_before else {
            continue;
        };
        let after_path = &atom.ancestor_elements;
        let before_path = &before.ancestor_elements;
        let (mut a, mut b) = (0, 0);
        let mut pending = Vec::new();
        let mut matched_paragraph = false;
        while a < after_path.len() && b < before_path.len() {
            if dom.name_is(after_path[a], &sdt) && dom.name_is(before_path[b], &sdt) {
                let current_match =
                    authored_control_match_strength(dom, after_path[a], before_path[b]);
                // Look only within this transparent-control segment. Crossing
                // a paragraph, cell or text-box ancestor would erase structure.
                let after_match = after_path[a + 1..]
                    .iter()
                    .copied()
                    .take_while(|&node| is_control(node))
                    .filter(|&node| dom.name_is(node, &sdt))
                    .map(|node| authored_control_match_strength(dom, node, before_path[b]))
                    .max()
                    .unwrap_or(0);
                let before_match = before_path[b + 1..]
                    .iter()
                    .copied()
                    .take_while(|&node| is_control(node))
                    .filter(|&node| dom.name_is(node, &sdt))
                    .map(|node| authored_control_match_strength(dom, node, after_path[a]))
                    .max()
                    .unwrap_or(0);
                // Equally strong matches in both directions can be the same
                // retained controls in a different nesting order. Neither is
                // surplus. A stronger unilateral match still beats a weaker
                // regenerated-id/tag-only candidate on the opposite side.
                if after_match > current_match && after_match > before_match {
                    pending.push(after_path[a]);
                    a += 1;
                    // Skip the wrapper and its own content as one unit;
                    // otherwise the opposite side's matched sdt is mistaken
                    // for the unmatched sdtContent on the next iteration.
                    if after_path
                        .get(a)
                        .is_some_and(|&node| dom.name_is(node, &content))
                    {
                        pending.push(after_path[a]);
                        a += 1;
                    }
                    continue;
                }
                if before_match > current_match && before_match > after_match {
                    pending.push(before_path[b]);
                    b += 1;
                    // Skip the wrapper and its own content as one unit;
                    // otherwise the opposite side's matched sdt is mistaken
                    // for the unmatched sdtContent on the next iteration.
                    if before_path
                        .get(b)
                        .is_some_and(|&node| dom.name_is(node, &content))
                    {
                        pending.push(before_path[b]);
                        b += 1;
                    }
                    continue;
                }
            }
            if dom.name(after_path[a]) == dom.name(before_path[b]) {
                matched_paragraph |= dom.name_is(after_path[a], &paragraph);
                a += 1;
                b += 1;
            } else if is_control(after_path[a]) {
                pending.push(after_path[a]);
                a += 1;
            } else if is_control(before_path[b]) {
                pending.push(before_path[b]);
                b += 1;
            } else {
                break;
            }
        }
        if a == after_path.len() && b == before_path.len() && matched_paragraph {
            unmatched.extend(pending);
        }
    }
    if unmatched.is_empty() {
        return;
    }
    let strip = |atom: &mut ComparisonUnitAtom| {
        if atom
            .ancestor_elements
            .iter()
            .any(|node| unmatched.contains(node))
        {
            atom.ancestor_elements = atom
                .ancestor_elements
                .iter()
                .copied()
                .filter(|node| !unmatched.contains(node))
                .collect::<Vec<_>>()
                .into();
            atom.ancestor_unids = None;
        }
    };
    for atom in atoms {
        strip(atom);
        if let Some(before) = &mut atom.comparison_unit_atom_before {
            strip(std::sync::Arc::make_mut(before));
        }
    }
}

fn is_ppr_atom(dom: &Dom, atom: &ComparisonUnitAtom) -> bool {
    dom.name_is(atom.content_element, &W::p_pr())
}
fn atom_in_textbox(dom: &Dom, atom: &ComparisonUnitAtom) -> bool {
    let txbx = W::txbx_content();
    atom.ancestor_elements
        .iter()
        .any(|&a| dom.name(a).as_ref() == Some(&txbx))
}

/// M4.E.2 — `AssembleAncestorUnidsInOrderToRebuildXmlTreeProperly` (:3974).
/// Three phases (see WmlComparer.ts): A copy before→after pPr ancestor Unids;
/// PRODUCE-UNID-01: (ancestor-elements chain, its minted unid chain) memo.
type UnidChainMemo = Option<(std::sync::Arc<[NodeId]>, std::sync::Arc<[String]>)>;

/// B seed ancestor_unids from the paragraph mark (reverse walk, minting missing);
/// C fix text boxes in a second reverse pass.
pub fn assemble_ancestor_unids(dom: &mut Dom, atoms: &mut [ComparisonUnitAtom]) {
    align_content_control_ancestry(dom, atoms);
    let unid = PT::unid();
    let footnote = W::footnote();
    let endnote = W::endnote();

    // ── Phase A ───────────────────────────────────────────────────────────────
    for atom in atoms.iter() {
        let mut do_set = false;
        if is_ppr_atom(dom, atom) {
            if atom_in_textbox(dom, atom) {
                do_set = true;
            }
            if atom.correlation_status == CorrelationStatus::Equal {
                do_set = true;
            }
        }
        if do_set && let Some(before) = &atom.comparison_unit_atom_before {
            let after_anc = &atom.ancestor_elements;
            let before_anc = &before.ancestor_elements;
            if after_anc.len() == before_anc.len() {
                let pairs: Vec<(NodeId, Option<String>)> = after_anc
                    .iter()
                    .zip(before_anc.iter())
                    .filter_map(|(&aft, &bef)| {
                        match (dom.attribute(aft, &unid), dom.attribute(bef, &unid)) {
                            (Some(_), Some(bv)) => Some((aft, Some(bv.to_string()))),
                            _ => None,
                        }
                    })
                    .collect();
                for (aft, bv) in pairs {
                    dom.set_attribute_value(aft, &unid, bv.as_deref());
                }
            }
        }
    }

    // deepest-ancestor (footnote/endnote root) override for index 0.
    let deepest_unid: Option<String> = atoms.last().and_then(|last| {
        last.ancestor_elements.first().and_then(|&outer| {
            let nm = dom.name(outer);
            if nm.as_ref() == Some(&footnote) || nm.as_ref() == Some(&endnote) {
                dom.attribute(outer, &unid).map(|s| s.to_string())
            } else {
                None
            }
        })
    });

    // helper: unid of an ancestor element, minting if absent.
    let unid_or_mint = |dom: &mut Dom, ae: NodeId| -> String {
        match dom.attribute(ae, &unid) {
            Some(u) => u.to_string(),
            None => {
                let g = generate_unid();
                dom.set_attribute_value(ae, &unid, Some(&g));
                g
            }
        }
    };

    // ── Phase B (reverse) ──────────────────────────────────────────────────────
    let mut current: Option<std::sync::Arc<[String]>> = None;
    // PATH-01: track the shared Arc chain (not a cloned Vec).
    let mut current_elems: Option<std::sync::Arc<[NodeId]>> = None;
    // PRODUCE-UNID-01: atoms of one run share an ancestor_elements Arc — reuse
    // the chain built for the previous atom instead of rebuilding per atom.
    let mut memo: UnidChainMemo = None;
    for atom in atoms.iter_mut().rev() {
        if is_ppr_atom(dom, atom) && !atom_in_textbox(dom, atom) {
            let mut cur: Vec<String> = atom
                .ancestor_elements
                .iter()
                .map(|&ae| unid_or_mint(dom, ae))
                .collect();
            if let Some(d) = &deepest_unid
                && let Some(first) = cur.first_mut()
            {
                *first = d.clone();
            }
            let cur: std::sync::Arc<[String]> = cur.into();
            atom.ancestor_unids = Some(std::sync::Arc::clone(&cur));
            current = Some(cur);
            current_elems = Some(std::sync::Arc::clone(&atom.ancestor_elements));
            memo = None;
        } else {
            let prefix = current.clone().unwrap_or_default();
            // Borrow the following paragraph's Unid prefix to bridge MATCHED A/B
            // paragraphs (different NodeIds, parallel structure) so their content
            // shares one paragraph Unid. Stop borrowing where the ancestor ELEMENT
            // TYPES diverge: blindly borrowing the whole `prefix.len()` desyncs
            // ancestor_unids from ancestor_elements when this atom's ancestor shape
            // differs from the next paragraph's (e.g. text in an outer table cell
            // that precedes a nested table) — CoalesceRecurse then nests block
            // content in a run (`w:p` inside `w:r`, Word "unreadable",
            // sd-2672-nested-table_sd-2672-sdt-table). Name-based (not NodeId
            // identity) so cross-tree matched paragraphs still share a Unid (m21).
            if let Some((elems, unids)) = &memo
                && std::sync::Arc::ptr_eq(elems, &atom.ancestor_elements)
            {
                atom.ancestor_unids = Some(std::sync::Arc::clone(unids));
                continue;
            }
            let prev_elems = current_elems.clone().unwrap_or_default();
            let mut share = 0usize;
            while share < prefix.len()
                && share < atom.ancestor_elements.len()
                && share < prev_elems.len()
                && dom.name(atom.ancestor_elements[share]) == dom.name(prev_elems[share])
            {
                share += 1;
            }
            let mut full: Vec<String> = prefix[..share].to_vec();
            for &ae in atom.ancestor_elements.iter().skip(share) {
                full.push(unid_or_mint(dom, ae));
            }
            if let Some(d) = &deepest_unid
                && let Some(first) = full.first_mut()
            {
                *first = d.clone();
            }
            let full: std::sync::Arc<[String]> = full.into();
            memo = Some((
                std::sync::Arc::clone(&atom.ancestor_elements),
                std::sync::Arc::clone(&full),
            ));
            atom.ancestor_unids = Some(full);
        }
    }

    // ── Phase C (reverse, text-box fix) ─────────────────────────────────────────
    let mut current: Option<std::sync::Arc<[String]>> = None;
    let mut skip_until_ppr = false;
    let mut memo: UnidChainMemo = None;
    for atom in atoms.iter_mut().rev() {
        if let Some(cur) = &current
            && atom.ancestor_elements.len() < cur.len()
        {
            skip_until_ppr = true;
            current = None;
            memo = None;
            continue;
        }
        if is_ppr_atom(dom, atom) {
            if !atom_in_textbox(dom, atom) {
                skip_until_ppr = true;
                current = None;
                memo = None;
                continue;
            }
            // text-box pPr: rebuild prefix (must already have Unids — Phase B minted them)
            let cur: Vec<String> = atom
                .ancestor_elements
                .iter()
                .map(|&ae| {
                    dom.attribute(ae, &unid)
                        .map(|s| s.to_string())
                        .expect("text-box pPr ancestor must have a Unid (Phase B)")
                })
                .collect();
            let cur: std::sync::Arc<[String]> = cur.into();
            atom.ancestor_unids = Some(std::sync::Arc::clone(&cur));
            current = Some(cur);
            skip_until_ppr = false;
            memo = None;
            continue;
        }
        if skip_until_ppr {
            continue;
        }
        if let Some(cur) = &current {
            if let Some((elems, unids)) = &memo
                && std::sync::Arc::ptr_eq(elems, &atom.ancestor_elements)
            {
                atom.ancestor_unids = Some(std::sync::Arc::clone(unids));
                continue;
            }
            let extra: Vec<NodeId> = atom
                .ancestor_elements
                .iter()
                .skip(cur.len())
                .copied()
                .collect();
            let mut full: Vec<String> = cur.as_ref().to_vec();
            for ae in extra {
                full.push(unid_or_mint(dom, ae));
            }
            let full: std::sync::Arc<[String]> = full.into();
            memo = Some((
                std::sync::Arc::clone(&atom.ancestor_elements),
                std::sync::Arc::clone(&full),
            ));
            atom.ancestor_unids = Some(full);
        }
    }
}

// ── M4.E.3-E.7 — CoalesceRecurse + ReconstructElement ────────────────────────

/// `GetXmlSpaceAttribute` — `Some("preserve")` when leading/trailing whitespace.
fn xml_space_attr(text: &str) -> Option<&'static str> {
    match (text.chars().next(), text.chars().last()) {
        (Some(f), _) if f.is_whitespace() => Some("preserve"),
        (_, Some(l)) if l.is_whitespace() => Some("preserve"),
        _ => None,
    }
}

fn status_str(s: CorrelationStatus) -> &'static str {
    match s {
        CorrelationStatus::Deleted => "Deleted",
        CorrelationStatus::Inserted => "Inserted",
        CorrelationStatus::MovedSource => "MovedSource",
        CorrelationStatus::MovedDestination => "MovedDestination",
        CorrelationStatus::FormatChanged => "FormatChanged",
        CorrelationStatus::Equal => "Equal",
        _ => "Nil",
    }
}

/// Stable first-key-seen bucket grouping (port of `groupByKey`).
fn group_by_key_stable<'a, K: Eq + std::hash::Hash + Clone>(
    items: &[&'a ComparisonUnitAtom],
    key: impl Fn(&ComparisonUnitAtom) -> K,
) -> Vec<(K, Vec<&'a ComparisonUnitAtom>)> {
    // Groups hold references, not owned atoms: coalesce_recurse re-groups every
    // atom at every nesting level, and ComparisonUnitAtom is fat (sha1_hash +
    // ancestor_unids: Vec<String> + a recursive Box<before-atom>), so cloning
    // per level was the dominant produce-phase allocation (samply). Grouping
    // semantics are unchanged — only ownership.
    let mut order: Vec<K> = Vec::new();
    let mut map: std::collections::HashMap<K, Vec<&'a ComparisonUnitAtom>> =
        std::collections::HashMap::new();
    for it in items {
        // One hash of the key per atom: the vacant arm records first-seen order.
        match map.entry(key(it)) {
            std::collections::hash_map::Entry::Vacant(e) => {
                order.push(e.key().clone());
                e.insert(vec![*it]);
            }
            std::collections::hash_map::Entry::Occupied(mut e) => e.get_mut().push(*it),
        }
    }
    order
        .into_iter()
        .map(|k| {
            let v = map.remove(&k).unwrap();
            (k, v)
        })
        .collect()
}

/// Add the `pt:Status` (+ move/format) attributes to a constructed node.
fn tag_status(dom: &mut Dom, node: NodeId, status: CorrelationStatus, atom: &ComparisonUnitAtom) {
    match status {
        CorrelationStatus::Deleted => {
            dom.set_attribute_value(node, &PT::status(), Some("Deleted"));
        }
        CorrelationStatus::Inserted => {
            dom.set_attribute_value(node, &PT::status(), Some("Inserted"));
        }
        CorrelationStatus::MovedSource | CorrelationStatus::MovedDestination => {
            dom.set_attribute_value(node, &PT::status(), Some(status_str(status)));
            if let Some(id) = atom.move_group_id {
                dom.set_attribute_value(node, &PT::name("MoveGroupId"), Some(&id.to_string()));
                dom.set_attribute_value(
                    node,
                    &PT::name("MoveName"),
                    Some(atom.move_name.as_deref().unwrap_or("")),
                );
            }
        }
        CorrelationStatus::FormatChanged => {
            dom.set_attribute_value(node, &PT::status(), Some("FormatChanged"));
            if let Some(fc) = &atom.format_change {
                if let Some(old) = fc.old_run_properties {
                    let s = dom.serialize_element(old);
                    dom.set_attribute_value(node, &PT::name("OldRPr"), Some(&s));
                }
                // M81: body pilcrow format change carries projected old pPr.
                if let Some(old) = fc.old_para_properties {
                    let s = dom.serialize_element(old);
                    dom.set_attribute_value(node, &PT::name("OldPPr"), Some(&s));
                }
            }
        }
        _ => {}
    }
}

fn is_txbx_from_level(dom: &Dom, atom: &ComparisonUnitAtom, level: usize) -> bool {
    let txbx = W::txbx_content();
    atom.ancestor_elements
        .iter()
        .skip(level)
        .any(|&a| dom.name(a).as_ref() == Some(&txbx))
}

/// M463 — final-serialization pass: every `w:ins`/`w:del` whose element
/// content is exactly OMML math (`m:oMath`/`m:oMathPara`) is unwrapped, and
/// the revision state is rewritten INSIDE the math the way Word Compare
/// writes it. Runs AFTER all mesh/finalize passes, which reason about the
/// outer-wrapped shape.
///
/// LibreOffice renders Word's internal-marked math as placeholder boxes; the
/// outer wrap rendered the live formula instead — every formula's ink
/// diverged from the oracle (math family n=28, mean ≈58; math_func ×
/// math_groupchr 54.6).
pub fn convert_outer_math_wraps_to_internal(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
) {
    let math_names = [
        crate::namespaces::M::name("oMath"),
        crate::namespaces::M::name("oMathPara"),
    ];
    // Next free revision id — internal marks need fresh ones.
    let mut max_id: u32 = 0;
    for e in dom.descendants(root, None) {
        if let Some(v) = dom.attribute(e, &W::id())
            && let Ok(n) = v.parse::<u32>()
        {
            max_id = max_id.max(n);
        }
    }
    let mut id_gen = max_id + 1;
    for rev_name in [W::ins(), W::del()] {
        let wrappers: Vec<NodeId> = dom
            .descendants(root, Some(&rev_name))
            .into_iter()
            .filter(|&w| {
                let kids = dom.elements(w, None);
                !kids.is_empty()
                    && kids
                        .iter()
                        .all(|&k| dom.name(k).is_some_and(|n| math_names.contains(&n)))
            })
            .collect();
        for w in wrappers {
            let maths: Vec<NodeId> = dom.elements(w, None);
            for m in &maths {
                mark_math_revisions_internally(dom, *m, &rev_name, settings, &mut id_gen);
            }
            for m in maths {
                dom.remove(m);
                dom.add_before_self(w, m);
            }
            dom.remove(w);
        }
    }
}

/// Rewrite a `m:oMath(Para)` subtree so its revision state lives INSIDE the
/// math, the way Word Compare writes it:
/// - every `m:r` moves its children (m:rPr?, w:rPr?, m:t…) into a
///   `w:ins`/`w:del` child; a `w:rPr` with Cambria Math rFonts is
///   materialized when the run stores none (Word always writes it);
/// - every `m:ctrlPr` moves its `w:rPr` (materialized likewise) into the
///   same mark.
///
/// `m:t` stays `m:t` under `w:del` — math content never becomes delText.
fn mark_math_revisions_internally(
    dom: &mut Dom,
    math_root: NodeId,
    rev_name: &crate::xmllinq::XName,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let m_r = crate::namespaces::M::name("r");
    let m_ctrl_pr = crate::namespaces::M::name("ctrlPr");
    let w_rpr = W::r_pr();
    let cambria = |dom: &mut Dom| -> NodeId {
        let rpr = dom.new_element(W::r_pr());
        let fonts = dom.new_element(W::name("rFonts"));
        dom.set_attribute_value(fonts, &W::name("ascii"), Some("Cambria Math"));
        dom.set_attribute_value(fonts, &W::name("hAnsi"), Some("Cambria Math"));
        dom.add(rpr, fonts);
        rpr
    };
    let mut new_mark = |dom: &mut Dom| -> NodeId {
        let w = dom.new_element(rev_name.clone());
        dom.set_attribute_value(w, &W::id(), Some(&id_gen.to_string()));
        *id_gen += 1;
        dom.set_attribute_value(w, &W::author(), Some(&settings.author_for_revisions));
        dom.set_attribute_value(w, &W::date(), Some(&settings.date_time_for_revisions));
        w
    };
    let targets: Vec<(NodeId, bool)> = dom
        .descendants(math_root, Some(&m_r))
        .into_iter()
        .map(|n| (n, false))
        .chain(
            dom.descendants(math_root, Some(&m_ctrl_pr))
                .into_iter()
                .map(|n| (n, true)),
        )
        .collect();
    for (node, is_ctrl_pr) in targets {
        // Skip runs that already carry a revision mark (defensive).
        if dom
            .elements(node, None)
            .iter()
            .any(|&c| dom.name(c).is_some_and(|n| n == W::ins() || n == W::del()))
        {
            continue;
        }
        let children: Vec<NodeId> = dom.elements(node, None);
        let mark = new_mark(dom);
        let mut saw_wrpr = false;
        for c in children {
            let is_m_rpr = dom.name(c) == Some(crate::namespaces::M::name("rPr"));
            saw_wrpr |= dom.name(c).as_ref() == Some(&w_rpr);
            dom.remove(c);
            dom.add(mark, c);
            // Materialize the math w:rPr right after m:rPr, before m:t.
            let _ = is_m_rpr;
        }
        if !saw_wrpr {
            let rpr = cambria(dom);
            // After m:rPr when present, else first.
            let anchor = dom
                .elements(mark, None)
                .into_iter()
                .find(|&c| dom.name(c) == Some(crate::namespaces::M::name("rPr")));
            match anchor {
                Some(a) => dom.add_after_self(a, rpr),
                None => match dom.elements(mark, None).first().copied() {
                    Some(first) => dom.add_before_self(first, rpr),
                    None => dom.add(mark, rpr),
                },
            }
        }
        let _ = is_ctrl_pr;
        dom.add(node, mark);
    }
}

/// M4.E.3-E.7 — `CoalesceRecurse` (:6024). Returns the constructed nodes for this
/// level. `id_gen` is the `s_MaxId` analog (oMath revision ids).
pub fn coalesce_recurse(
    dom: &mut Dom,
    atoms: &[&ComparisonUnitAtom],
    level: usize,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> Vec<NodeId> {
    // Step 1 — group by (ancestor Unid, element type) at this level (stable), drop
    // empty keys. A Unid must map to ONE element type; when the correlation assigns
    // the SAME Unid to different types (A's <w:tbl> nested in a cell ↔ B's <w:sdt>),
    // grouping by Unid alone merges them and spills a stray child (e.g. a <w:tc>
    // directly under <w:sdtContent>). Keying on element-name too keeps the divergent
    // structures separate. (sd-2672-nested-table_sd-2672-sdt-table.)
    let dref: &Dom = dom;
    // Tuple key, not format!("{u}|{nm}") — the concat + re-split was a
    // measurable slice of produce-phase hashing/allocation.
    let grouped = group_by_key_stable(atoms, |ca| {
        if level >= ca.ancestor_elements.len() {
            return (String::new(), String::new());
        }
        let u = ca
            .ancestor_unids
            .as_ref()
            .and_then(|u| u.get(level).cloned())
            .unwrap_or_default();
        if u.is_empty() {
            return (String::new(), String::new());
        }
        let nm = dref
            .name(ca.ancestor_elements[level])
            .map(|n| n.local_name().to_string())
            .unwrap_or_default();
        (u, nm)
    });
    let grouped: Vec<_> = grouped
        .into_iter()
        .filter(|(k, _)| !k.0.is_empty())
        .collect();
    if grouped.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    for (gkey, g) in grouped {
        let ancestor = g[0].ancestor_elements[level];
        let aname = dom.name(ancestor).unwrap();

        // Step 3 — group children by (next-level unid | status), txbx → Equal.
        let groupedchildren = group_adjacent(g.iter().cloned(), |gc| {
            let key = if level < gc.ancestor_elements.len() - 1 {
                gc.ancestor_unids
                    .as_ref()
                    .and_then(|u| u.get(level + 1).cloned())
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let st = if is_txbx_from_level(dom, gc, level) {
                "Equal"
            } else {
                status_str(gc.correlation_status)
            };
            // One revised run can correlate with several differently
            // formatted original runs. Preserve each old rPr boundary so
            // finalize records the right history instead of the first atom's.
            let old_run_properties = if aname == W::p()
                && gc.correlation_status == CorrelationStatus::FormatChanged
                && gc
                    .ancestor_elements
                    .get(level + 1)
                    .is_some_and(|&node| dom.name_is(node, &W::r()))
            {
                // Split direct runs only. A formatted run below an inline
                // container is split inside that container's reconstruction;
                // splitting it here would duplicate the entire SDT/link.
                gc.format_change
                    .as_ref()
                    .and_then(|change| change.old_run_properties)
            } else {
                None
            };
            (key, st, old_run_properties)
        });

        // w:p
        if aname == W::p() {
            let p = dom.new_element(W::p());
            for (an, av) in dom.attributes(ancestor) {
                if an.namespace_name() != PT::URI {
                    dom.set_attribute_value(p, &an, Some(&av));
                }
            }
            // Tuple key: .0 is the Unid (scratch).
            dom.set_attribute_value(p, &PT::unid(), Some(&gkey.0));
            for (key, gc) in &groupedchildren {
                if key.0.is_empty() {
                    for gcc in gc {
                        let dup = dom.clone_subtree(gcc.content_element);
                        tag_status(dom, dup, gcc.correlation_status, gcc);
                        dom.add(p, dup);
                    }
                } else {
                    for child in coalesce_recurse(dom, gc, level + 1, settings, id_gen) {
                        dom.add(p, child);
                    }
                }
            }
            out.push(p);
            continue;
        }

        // w:r
        if aname == W::r() {
            let r = dom.new_element(W::r());
            for (an, av) in dom.attributes(ancestor) {
                // the pt:PreDelete / pt:PreIns stamp trios are the ONLY
                // scratch attr families produce must carry:
                // finalize::convert_stamped_predeletes / _preins turn the
                // stamped runs back into pending w:del / w:ins. Explicit
                // allowlist — no other pt:* attr may leak into the redline.
                if an.namespace_name() != PT::URI
                    || matches!(
                        an.local_name(),
                        "PreDelete"
                            | "PreDelAuthor"
                            | "PreDelDate"
                            | "PreIns"
                            | "PreInsAuthor"
                            | "PreInsDate"
                    )
                {
                    dom.set_attribute_value(r, &an, Some(&av));
                }
            }
            if let Some(rpr) = dom.element(ancestor, &W::r_pr()) {
                let rpr_clone = dom.clone_subtree(rpr);
                dom.add(r, rpr_clone);
            }
            for (key, gc) in &groupedchildren {
                if key.0.is_empty() {
                    for gcc in gc {
                        let dup = dom.clone_subtree(gcc.content_element);
                        tag_status(dom, dup, gcc.correlation_status, gcc);
                        dom.add(r, dup);
                    }
                } else {
                    for child in coalesce_recurse(dom, gc, level + 1, settings, id_gen) {
                        dom.add(r, child);
                    }
                }
            }
            out.push(r);
            continue;
        }

        // w:t — emit text elements (w:t / w:delText) with status; no wrapper.
        // Pure del → delText. MovedSource → w:t (Word Compare; see delete_text_in_opaque).
        if aname == W::t() {
            for (_key, gc) in &groupedchildren {
                let text: String = gc
                    .iter()
                    .map(|a| dom.value_str(a.content_element).into_owned())
                    .collect();
                let first = &gc[0];
                let elem_name = match first.correlation_status {
                    CorrelationStatus::Deleted => W::del_text(),
                    _ => W::t(),
                };
                let te = dom.new_element(elem_name);
                tag_status(dom, te, first.correlation_status, first);
                if let Some(sp) = xml_space_attr(&text) {
                    dom.set_attribute_value(te, &XNamespace::xml().name("space"), Some(sp));
                }
                dom.add_text(te, &text);
                out.push(te);
            }
            continue;
        }

        // w:drawing — clone + status (part relocation deferred to M4.H).
        if aname == W::drawing() {
            for (_key, gc) in &groupedchildren {
                for gcc in gc {
                    let d = dom.clone_subtree(gcc.content_element);
                    tag_status(dom, d, gcc.correlation_status, gcc);
                    delete_text_in_opaque(dom, d, gcc.correlation_status);
                    out.push(d);
                }
            }
            continue;
        }

        // w:pict (VML image) and w:object (embedded OLE picture) — clone
        // full subtree + status. Must not fall through to
        // reconstruct_element / empty Allowable shell: attribute-only
        // v:imagedata children emit no atoms when recursed (M74), and an
        // object rebuilt that way lost its picture and its revision mark.
        if aname == W::pict() || aname == W::object() {
            for (_key, gc) in &groupedchildren {
                for gcc in gc {
                    let d = dom.clone_subtree(gcc.content_element);
                    tag_status(dom, d, gcc.correlation_status, gcc);
                    delete_text_in_opaque(dom, d, gcc.correlation_status);
                    out.push(d);
                }
            }
            continue;
        }

        // mc:AlternateContent — verbatim clone + status.
        if aname == crate::namespaces::MC::name("AlternateContent") {
            for (_key, gc) in &groupedchildren {
                for gcc in gc {
                    let d = dom.clone_subtree(gcc.content_element);
                    tag_status(dom, d, gcc.correlation_status, gcc);
                    delete_text_in_opaque(dom, d, gcc.correlation_status);
                    out.push(d);
                }
            }
            continue;
        }

        // m:oMath / m:oMathPara — wrap in real w:del/w:ins/w:moveFrom/w:moveTo.
        // The outer wrap is the shape every mesh/finalize pass reasons about;
        // the final Word serialization (revision marks INSIDE the math, M463)
        // is produced by `convert_outer_math_wraps_to_internal` at the very
        // end of the pipeline.
        if aname == crate::namespaces::M::name("oMath")
            || aname == crate::namespaces::M::name("oMathPara")
        {
            for (_key, gc) in &groupedchildren {
                for gcc in gc {
                    let rev = match gcc.correlation_status {
                        CorrelationStatus::Deleted => Some(W::del()),
                        CorrelationStatus::MovedSource => Some(W::move_from()),
                        CorrelationStatus::Inserted => Some(W::ins()),
                        CorrelationStatus::MovedDestination => Some(W::move_to()),
                        _ => None,
                    };
                    let content = dom.clone_subtree(gcc.content_element);
                    match rev {
                        Some(rname) => {
                            let w = dom.new_element(rname);
                            dom.set_attribute_value(
                                w,
                                &W::author(),
                                Some(&settings.author_for_revisions),
                            );
                            dom.set_attribute_value(w, &W::id(), Some(&id_gen.to_string()));
                            *id_gen += 1;
                            dom.set_attribute_value(
                                w,
                                &W::date(),
                                Some(&settings.date_time_for_revisions),
                            );
                            dom.add(w, content);
                            out.push(w);
                        }
                        None => out.push(content),
                    }
                }
            }
            continue;
        }

        // AllowableRunChildren — fresh element (attrs minus pt:) + status.
        if super::tables::ALLOWABLE_RUN_CHILDREN.contains(&aname) {
            for (_key, gc) in &groupedchildren {
                let first = &gc[0];
                match first.correlation_status {
                    CorrelationStatus::Deleted
                    | CorrelationStatus::Inserted
                    | CorrelationStatus::MovedSource
                    | CorrelationStatus::MovedDestination
                    | CorrelationStatus::FormatChanged => {
                        for gcc in gc {
                            let dup = dom.new_element(aname.clone());
                            for (an, av) in dom.attributes(ancestor) {
                                if an.namespace_name() != PT::URI {
                                    dom.set_attribute_value(dup, &an, Some(&av));
                                }
                            }
                            // The leaf's content rides along: an inserted or
                            // deleted w:instrText without its text is a field
                            // with no code (blank page numbers, citations).
                            for n in dom.nodes(gcc.content_element) {
                                let c = dom.clone_subtree(n);
                                dom.add(dup, c);
                            }
                            tag_status(dom, dup, gcc.correlation_status, gcc);
                            out.push(dup);
                        }
                    }
                    _ => {
                        for gcc in gc {
                            out.push(dom.clone_subtree(gcc.content_element));
                        }
                    }
                }
            }
            continue;
        }

        // Container elements → ReconstructElement (props hoisted first).
        let props: &[&str] = if aname == W::tbl() {
            &["tblPr", "tblGrid"]
        } else if aname == W::tr() {
            &["trPr"]
        } else if aname == W::tc() {
            &["tcPr"]
        } else if aname == W::sdt() {
            &["sdtPr", "sdtEndPr"]
        } else if aname == W::name("ruby") {
            &["rubyPr"]
        } else {
            &[]
        };
        let recon = reconstruct_element(dom, &g, ancestor, props, level, settings, id_gen);
        out.push(recon);
    }
    out
}

/// M4.E.6 — `ReconstructElement` (:6984): rebuild a container element, hoisting
/// the named property children first, then the recursively-coalesced children.
fn reconstruct_element(
    dom: &mut Dom,
    g: &[&ComparisonUnitAtom],
    ancestor: NodeId,
    props: &[&str],
    level: usize,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> NodeId {
    let aname = dom.name(ancestor).unwrap();
    // Generic inline containers (hyperlink, sdtContent, smartTag, ...) also
    // need separate reconstructed runs for each original formatting boundary.
    // Restrict segmentation to direct run children: splitting block-container
    // groups here would duplicate paragraphs or table structure.
    let split_run_formats = g
        .iter()
        .any(|atom| atom.correlation_status == CorrelationStatus::FormatChanged)
        && g.iter().all(|atom| {
            atom.ancestor_elements
                .get(level + 1)
                .is_some_and(|&node| dom.name_is(node, &W::r()))
        });
    let new_children = if split_run_formats {
        let groups = group_adjacent(g.iter().copied(), |atom| {
            let run = atom
                .ancestor_unids
                .as_ref()
                .and_then(|unids| unids.get(level + 1))
                .cloned();
            let old = if atom.correlation_status == CorrelationStatus::FormatChanged {
                atom.format_change
                    .as_ref()
                    .and_then(|change| change.old_run_properties)
            } else {
                None
            };
            (run, atom.correlation_status, old)
        });
        let mut children = Vec::new();
        for (_, atoms) in groups {
            children.extend(coalesce_recurse(dom, &atoms, level + 1, settings, id_gen));
        }
        children
    } else {
        coalesce_recurse(dom, g, level + 1, settings, id_gen)
    };
    let ne = dom.new_element(aname.clone());
    for (an, av) in dom.attributes(ancestor) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    // hoist property children (in declared order)
    if aname == W::pict() {
        for p in dom.elements(ancestor, Some(&crate::namespaces::VML::name("shapetype"))) {
            let c = dom.clone_subtree(p);
            dom.add(ne, c);
        }
    }
    for pname in props {
        for p in dom.elements(ancestor, Some(&W::name(pname))) {
            let c = dom.clone_subtree(p);
            dom.add(ne, c);
        }
    }
    // Word-alignment (M-TBL, parity/_scratch/table_class_forensics.md): a
    // MERGED table takes the NEW table's effective tblPr/tblGrid (hoisted
    // above from `ancestor`, the modified side), and Word records the OLD
    // table's properties in w:tblPrChange (last child of tblPr) and
    // w:tblGridChange (last child of tblGrid) — GT table-bookmark-end_
    // table-vmerge-colspan: effective tblW 6000/grid 3502·3509·3285, old
    // tblW 9360/union grid preserved in the change records. Ours dropped
    // the history entirely, so the old width kept rendering (2 vs 3 pages).
    if settings.merge_replaced_paragraphs && aname == W::tbl() {
        // Bind the table element name once; the find_map below runs per atom.
        let is_tbl = |anc: NodeId| dom.name(anc).is_some_and(|nm| *nm.local_name() == *"tbl");
        // the OLD table node: Deleted atoms carry doc A's ancestors directly;
        // Equal atoms carry them on `comparison_unit_atom_before`.
        let old_tbl = g.iter().find_map(|a| {
            let direct = a
                .ancestor_elements
                .get(level)
                .copied()
                .filter(|&anc| anc != ancestor && is_tbl(anc));
            direct.or_else(|| {
                let before = a.comparison_unit_atom_before.as_ref()?;
                before
                    .ancestor_elements
                    .get(level)
                    .copied()
                    .filter(|&anc| anc != ancestor && is_tbl(anc))
            })
        });
        // M-TBL rule 2b — orientation: `ancestor` (the hoist source) may
        // resolve to the OLD (doc A) table when the merged group leads with
        // A-side atoms. Word keeps the NEW table's props effective in either
        // orientation (GT table-bookmark-end_table-vmerge-colspan: effective
        // 0/auto from B, A's 6000 in tblPrChange; ours kept A's 6000 with the
        // NEW props in the change record). Detect: a Deleted atom (or a
        // `comparison_unit_atom_before`) owns `ancestor` → the "other" table
        // found above is really the NEW one; re-hoist from it and record
        // `ancestor` as the old side.
        let ancestor_is_old = g.iter().any(|a| {
            (a.correlation_status == CorrelationStatus::Deleted
                && a.ancestor_elements.get(level) == Some(&ancestor))
                || a.comparison_unit_atom_before
                    .as_ref()
                    .is_some_and(|b| b.ancestor_elements.get(level) == Some(&ancestor))
        });
        let old_tbl = match (old_tbl, ancestor_is_old) {
            (Some(new_tbl), true) => {
                for pname in ["tblPr", "tblGrid"] {
                    for hoisted in dom.elements(ne, Some(&W::name(pname))) {
                        dom.remove(hoisted);
                    }
                    for p in dom.elements(new_tbl, Some(&W::name(pname))) {
                        let c = dom.clone_subtree(p);
                        dom.add(ne, c);
                    }
                }
                Some(ancestor)
            }
            (found, _) => found,
        };
        if let Some(old_tbl) = old_tbl {
            let strip_change = |dom: &mut Dom, el: NodeId, change: &str| {
                for c in dom.elements(el, Some(&W::name(change))) {
                    dom.remove(c);
                }
            };
            // tblPr → tblPrChange
            if let (Some(new_pr), Some(old_pr)) = (
                dom.element(ne, &W::tbl_pr()),
                dom.element(old_tbl, &W::tbl_pr()),
            ) {
                let old_clone = dom.clone_subtree(old_pr);
                strip_change(dom, old_clone, "tblPrChange");
                if dom.serialize_element(old_clone) != dom.serialize_element(new_pr) {
                    let change = dom.new_element(W::name("tblPrChange"));
                    dom.set_attribute_value(change, &W::id(), Some(&id_gen.to_string()));
                    *id_gen += 1;
                    dom.set_attribute_value(
                        change,
                        &W::author(),
                        Some(&settings.author_for_revisions),
                    );
                    dom.set_attribute_value(
                        change,
                        &W::date(),
                        Some(&settings.date_time_for_revisions),
                    );
                    dom.add(change, old_clone);
                    dom.add(new_pr, change);
                } else {
                    dom.remove(old_clone);
                }
            }
            // tblGrid → tblGridChange
            if let (Some(new_grid), Some(old_grid)) = (
                dom.element(ne, &W::name("tblGrid")),
                dom.element(old_tbl, &W::name("tblGrid")),
            ) {
                let old_clone = dom.clone_subtree(old_grid);
                strip_change(dom, old_clone, "tblGridChange");
                if dom.serialize_element(old_clone) != dom.serialize_element(new_grid) {
                    let change = dom.new_element(W::name("tblGridChange"));
                    // w:id ONLY. CT_TblGridChange is the one revision-history
                    // element that does not extend CT_TrackChange, so it
                    // declares neither w:author nor w:date — unlike the
                    // tblPrChange directly above, which does. Copying that
                    // block wholesale made the validator report
                    // Sch_UndeclaredAttribute on both.
                    dom.set_attribute_value(change, &W::id(), Some(&id_gen.to_string()));
                    *id_gen += 1;
                    dom.add(change, old_clone);
                    dom.add(new_grid, change);
                } else {
                    dom.remove(old_clone);
                }
            }
        }
    }
    for c in new_children {
        dom.add(ne, c);
    }
    if settings.detect_format_changes && (aname == W::tr() || aname == W::tc()) {
        record_revised_row_cell_props(dom, ne, g, ancestor, level, settings, id_gen);
    }
    // Word-alignment (M-TBL rule 4, parity/_scratch/table_class_forensics.md):
    // a DEGENERATE grid — fewer w:gridCol entries than the real column count
    // implied by the rows' gridSpan/tc structure — is rebuilt Word's way:
    // per-column gridCols (equal split of the page content width) plus
    // `tblW 0 auto`. GT table-vmerge-colspan_text-box: 1×4985 → 4675+4675;
    // GT nested-table-rowspan_numbered-list: 1×9970 → 4887+4905.
    if settings.merge_replaced_paragraphs && aname == W::tbl() {
        rebuild_degenerate_grid(dom, ne, ancestor);
    }
    ne
}

/// A merged row or cell shows the revision's `trPr` /
/// `tcPr` and records the original's in `trPrChange` / `tcPrChange`
/// (ff42b4a7a3's rewritten header row; 117 of the 300 table-bearing Word
/// redlines of 500_extra carry a `tcPrChange`). The container is rebuilt
/// from its first atom's ancestor — the original's when that atom is
/// deleted — so accepting kept the original's shading and widths.
fn record_revised_row_cell_props(
    dom: &mut Dom,
    ne: NodeId,
    g: &[&ComparisonUnitAtom],
    ancestor: NodeId,
    level: usize,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let Some(aname) = dom.name(ancestor) else {
        return;
    };
    let (pr_local, change_local) = if aname == W::tr() {
        ("trPr", "trPrChange")
    } else {
        ("tcPr", "tcPrChange")
    };
    let same_kind = |anc: &NodeId| dom.name(*anc).as_ref() == Some(&aname);
    // The original's container: a deleted atom's own ancestor, or an equal
    // atom's before-side one. The revision's: any other atom's ancestor.
    let old = g.iter().find_map(|a| match a.correlation_status {
        CorrelationStatus::Deleted | CorrelationStatus::MovedSource => {
            a.ancestor_elements.get(level).copied().filter(same_kind)
        }
        _ => a
            .comparison_unit_atom_before
            .as_ref()?
            .ancestor_elements
            .get(level)
            .copied()
            .filter(same_kind),
    });
    let new = g.iter().find_map(|a| match a.correlation_status {
        CorrelationStatus::Deleted | CorrelationStatus::MovedSource => None,
        _ => a.ancestor_elements.get(level).copied().filter(same_kind),
    });
    let (Some(old), Some(new)) = (old, new) else {
        return;
    };
    if old == new {
        return;
    }
    let pr_name = W::name(pr_local);
    if ancestor != new {
        for hoisted in dom.elements(ne, Some(&pr_name)) {
            dom.remove(hoisted);
        }
        if let Some(p) = dom.element(new, &pr_name) {
            let c = dom.clone_subtree(p);
            place_row_cell_props(dom, ne, c);
        }
    }
    // The record's inner block may carry no revision of its own
    // (CT_TrPrBase has no ins/del; CT_TcPrInner no tcPrChange).
    let old_pr = match dom.element(old, &pr_name) {
        Some(p) => dom.clone_subtree(p),
        None => dom.new_element(pr_name.clone()),
    };
    for c in dom.elements(old_pr, None) {
        if dom
            .name(c)
            .is_some_and(|n| matches!(n.local_name(), "ins" | "del" | "trPrChange" | "tcPrChange"))
        {
            dom.remove(c);
        }
    }
    let live = match dom.element(ne, &pr_name) {
        Some(p) => p,
        None => {
            let p = dom.new_element(pr_name.clone());
            place_row_cell_props(dom, ne, p);
            p
        }
    };
    let live_sig: String = dom
        .elements(live, None)
        .into_iter()
        .filter(|&c| {
            dom.name(c).is_some_and(|n| {
                !matches!(n.local_name(), "ins" | "del" | "trPrChange" | "tcPrChange")
            })
        })
        .map(|c| props_signature(dom, c))
        .collect();
    let old_sig: String = dom
        .elements(old_pr, None)
        .into_iter()
        .map(|c| props_signature(dom, c))
        .collect();
    if live_sig == old_sig {
        if dom.elements(live, None).is_empty() {
            dom.remove(live);
        }
        return;
    }
    let change = dom.new_element(W::name(change_local));
    dom.set_attribute_value(change, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;
    dom.set_attribute_value(change, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(change, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add(change, old_pr);
    dom.add(live, change);
}

/// A property element's name, sorted attributes and children, recursively:
/// the comparer's scratch `pt14:*` ids and rsids are no properties, so two
/// clones of the same `w:tcW` compare equal whatever Unid each carries.
fn props_signature(dom: &Dom, n: NodeId) -> String {
    let Some(name) = dom.name(n) else {
        return String::new();
    };
    let mut attrs: Vec<String> = dom
        .attributes(n)
        .into_iter()
        .filter(|(k, _)| k.namespace_name() != PT::URI && !k.local_name().starts_with("rsid"))
        .map(|(k, v)| format!("{}={v}", k.local_name()))
        .collect();
    attrs.sort();
    let kids: String = dom
        .elements(n, None)
        .into_iter()
        .map(|c| props_signature(dom, c))
        .collect();
    format!("<{}{}>{kids}</>", name.local_name(), attrs.join(" "))
}
/// Put a row's `trPr` after its `tblPrEx`, a cell's `tcPr` first.
fn place_row_cell_props(dom: &mut Dom, container: NodeId, pr: NodeId) {
    match dom.element(container, &W::name("tblPrEx")) {
        Some(ex) => dom.add_after_self(ex, pr),
        None => dom.add_first(container, pr),
    }
}

/// CT_TblPrBase child order (wml.xsd). A synthesized child must be inserted
/// immediately after the last existing predecessor so `tblPr` stays
/// schema-valid — Word repairs an out-of-order `CT_TblPrBase`.
const TBLPR_CHILD_ORDER: &[&str] = &[
    "tblStyle",
    "tblpPr",
    "tblOverlap",
    "bidiVisual",
    "tblStyleRowBandSize",
    "tblStyleColBandSize",
    "tblW",
    "jc",
    "tblCellSpacing",
    "tblInd",
    "tblBorders",
    "shd",
    "tblLayout",
    "tblCellMar",
    "tblLook",
    "tblCaption",
    "tblDescription",
];

/// Insert `child` (a new tblPr child named `local`) under `tbl_pr` in
/// CT_TblPrBase order: after the last present predecessor, else first.
fn add_tblpr_child_in_order(dom: &mut Dom, tbl_pr: NodeId, child: NodeId, local: &str) {
    let new_rank = TBLPR_CHILD_ORDER
        .iter()
        .position(|&n| n == local)
        .unwrap_or(usize::MAX);
    let anchor = dom.elements(tbl_pr, None).into_iter().rev().find(|&e| {
        dom.name(e).is_some_and(|nm| {
            TBLPR_CHILD_ORDER
                .iter()
                .position(|&n| n == nm.local_name())
                .is_some_and(|rank| rank < new_rank)
        })
    });
    match anchor {
        Some(a) => dom.add_after_self(a, child),
        None => dom.add_first(tbl_pr, child),
    }
}

/// M-TBL rule 4 — see call site. `src_tbl` is the source-document table node
/// used to locate the section geometry (page width minus margins).
fn rebuild_degenerate_grid(dom: &mut Dom, tbl: NodeId, src_tbl: NodeId) {
    let Some(grid) = dom.element(tbl, &W::name("tblGrid")) else {
        return;
    };
    let grid_cols = dom.elements(grid, Some(&W::name("gridCol")));
    // real column count: max over rows of Σ gridSpan (default 1) per cell
    let real_cols = dom
        .elements(tbl, Some(&W::tr()))
        .into_iter()
        .map(|tr| {
            dom.elements(tr, Some(&W::tc()))
                .into_iter()
                .map(|tc| {
                    dom.element(tc, &W::tc_pr())
                        .and_then(|pr| dom.element(pr, &W::grid_span()))
                        .and_then(|gs| dom.attribute(gs, &W::val()))
                        .and_then(|v| v.parse::<usize>().ok())
                        .unwrap_or(1)
                })
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0);
    if real_cols < 2 || grid_cols.len() >= real_cols {
        return;
    }
    // effective width = page content width from the source doc's sectPr,
    // falling back to the declared grid total
    let content_width = dom
        .ancestors(src_tbl, None)
        .last()
        .map(|&root| dom.descendants(root, Some(&W::sect_pr())))
        .and_then(|s| s.first().copied())
        .and_then(|sect| {
            let w: i64 = dom
                .element(sect, &W::name("pgSz"))
                .and_then(|e| dom.attribute(e, &W::name("w")))
                .and_then(|v| v.parse().ok())?;
            let mar = dom.element(sect, &W::name("pgMar"))?;
            let l: i64 = dom
                .attribute(mar, &W::name("left"))
                .and_then(|v| v.parse().ok())?;
            let r: i64 = dom
                .attribute(mar, &W::name("right"))
                .and_then(|v| v.parse().ok())?;
            Some(w - l - r)
        })
        .filter(|&w| w > 0)
        .unwrap_or_else(|| {
            grid_cols
                .iter()
                .filter_map(|&c| dom.attribute(c, &W::name("w")))
                .filter_map(|v| v.parse::<i64>().ok())
                .sum()
        });
    if content_width <= 0 {
        return;
    }
    for c in &grid_cols {
        dom.remove(*c);
    }
    let each = content_width / real_cols as i64;
    let mut new_cols = Vec::with_capacity(real_cols);
    for i in 0..real_cols {
        let w = if i + 1 == real_cols {
            content_width - each * (real_cols as i64 - 1)
        } else {
            each
        };
        let col = dom.new_element(W::name("gridCol"));
        dom.set_attribute_value(col, &W::name("w"), Some(&w.to_string()));
        new_cols.push(col);
    }
    // gridCols must precede any tblGridChange history in the grid
    for col in new_cols.into_iter().rev() {
        dom.add_first(grid, col);
    }
    // tblW → 0 auto (CT_TblPrBase schema order: tblW follows tblStyle,
    // tblpPr, tblOverlap, bidiVisual, tblStyleRowBandSize,
    // tblStyleColBandSize). Insert after the last present predecessor so a
    // floating (tblpPr) or banded table stays schema-valid; mutate in place
    // when tblW already exists.
    if let Some(tbl_pr) = dom.element(tbl, &W::tbl_pr()) {
        let tblw = match dom.element(tbl_pr, &W::name("tblW")) {
            Some(e) => e,
            None => {
                let e = dom.new_element(W::name("tblW"));
                add_tblpr_child_in_order(dom, tbl_pr, e, "tblW");
                e
            }
        };
        dom.set_attribute_value(tblw, &W::name("w"), Some("0"));
        dom.set_attribute_value(tblw, &W::name("type"), Some("auto"));
    }
}

/// M4.E.8 — `ProduceNewWmlMarkupFromCorrelatedSequence` (:5926): reset the id
/// counter and coalesce at level 0.
pub fn produce_new_wml_markup_from_correlated_sequence(
    dom: &mut Dom,
    atoms: &[ComparisonUnitAtom],
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> Vec<NodeId> {
    // Borrow each atom once; coalesce_recurse threads &-slices (no atom clones).
    let refs: Vec<&ComparisonUnitAtom> = atoms.iter().collect();
    coalesce_recurse(dom, &refs, 0, settings, id_gen)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod opaque_text_tests {
    //! Direct coverage for `delete_text_in_opaque` (private). Pins the
    //! status→text-kind contract — in particular that `MovedSource` renames `w:t`
    //! to `w:delText` (ISO/IEC 29500-1 §17.3.3.7: `delText` replaces `t` within a
    //! `del` *or `moveFrom`*), which is otherwise unexercised because move
    //! detection is off by default.
    use super::*;

    /// `<w:drawing><w:r><w:t>txt</w:t></w:r></w:drawing>` — an opaque subtree.
    fn opaque_with_text(d: &mut Dom, txt: &str) -> NodeId {
        let drawing = d.new_element(W::drawing());
        let r = d.new_element(W::r());
        let t = d.new_element(W::t());
        d.add_text(t, txt);
        d.add(r, t);
        d.add(drawing, r);
        drawing
    }

    /// Local name of the first `w:t`/`w:delText` leaf under `node`.
    fn text_kind(d: &Dom, node: NodeId) -> String {
        let leaf = d
            .descendants(node, None)
            .into_iter()
            .find(|&c| {
                matches!(
                    d.name(c).as_ref().map(|n| n.local_name()),
                    Some("t") | Some("delText")
                )
            })
            .expect("a text leaf");
        d.name(leaf).unwrap().local_name().to_string()
    }

    #[test]
    fn deleted_opaque_text_becomes_deltext() {
        let mut d = Dom::new();
        let n = opaque_with_text(&mut d, "x");
        delete_text_in_opaque(&mut d, n, CorrelationStatus::Deleted);
        assert_eq!(text_kind(&d, n), "delText");
    }

    #[test]
    fn moved_source_opaque_text_stays_t_like_word() {
        let mut d = Dom::new();
        let n = opaque_with_text(&mut d, "x");
        delete_text_in_opaque(&mut d, n, CorrelationStatus::MovedSource);
        assert_eq!(
            text_kind(&d, n),
            "t",
            "Word Compare keeps w:t inside moveFrom (not delText)"
        );
    }

    #[test]
    fn non_deleted_opaque_text_stays_t() {
        for status in [
            CorrelationStatus::Inserted,
            CorrelationStatus::MovedDestination,
            CorrelationStatus::Equal,
        ] {
            let mut d = Dom::new();
            let n = opaque_with_text(&mut d, "x");
            delete_text_in_opaque(&mut d, n, status);
            assert_eq!(
                text_kind(&d, n),
                "t",
                "non-deleted opaque text stays w:t for {status:?}"
            );
        }
    }

    #[test]
    fn instr_text_is_untouched() {
        // `w:instrText` is not `w:t`, so the `W::t()` filter must leave it alone
        // even under a deletion (renaming it would corrupt the field code).
        let mut d = Dom::new();
        let drawing = d.new_element(W::drawing());
        let r = d.new_element(W::r());
        let instr = d.new_element(W::instr_text());
        d.add_text(instr, "FIELD");
        d.add(r, instr);
        d.add(drawing, r);
        delete_text_in_opaque(&mut d, drawing, CorrelationStatus::Deleted);
        assert_eq!(
            d.descendants(drawing, Some(&W::instr_text())).len(),
            1,
            "instrText untouched"
        );
        assert!(
            d.descendants(drawing, Some(&W::del_text())).is_empty(),
            "no delText fabricated from instrText"
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tblpr_order_tests {
    //! Word-validity regression: a synthesized `w:tblW` must land in its
    //! `CT_TblPrBase` schema slot (after tblpPr/bidiVisual/...), not pinned
    //! after tblStyle — Word repairs an out-of-order `tblPr` child sequence.
    use super::*;

    #[test]
    fn tblw_inserted_after_tblppr_and_bidivisual() {
        let mut dom = Dom::new();
        let xml = concat!(
            "<w:tblPr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
            "<w:tblStyle w:val=\"TableGrid\"/>",
            "<w:tblpPr w:leftFromText=\"0\"/>",
            "<w:bidiVisual/>",
            "</w:tblPr>"
        );
        let doc = dom.parse_xdocument(xml);
        let tblpr = dom.root(doc).expect("root");
        let tblw = dom.new_element(W::name("tblW"));
        add_tblpr_child_in_order(&mut dom, tblpr, tblw, "tblW");
        let order: Vec<String> = dom
            .elements(tblpr, None)
            .into_iter()
            .map(|e| dom.name(e).unwrap().local_name().to_string())
            .collect();
        let pos = |n: &str| order.iter().position(|x| x == n).unwrap();
        assert!(
            pos("tblW") > pos("tblpPr") && pos("tblW") > pos("bidiVisual"),
            "tblW must follow tblpPr and bidiVisual (CT_TblPrBase), got: {order:?}"
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod content_control_source_order_regressions {
    use crate::comparer::{WmlComparerSettings, compare_bodies_faithful};
    use crate::namespaces::W;
    use crate::revision_processor::{accept_revisions_document, reject_revisions_document};
    use crate::xmllinq::{Dom, NodeId};

    fn document(dom: &mut Dom, body: &str) -> (NodeId, NodeId) {
        let document = dom.parse_xdocument(&format!(
            "<w:document xmlns:w=\"{}\"><w:body>{body}<w:sectPr/></w:body></w:document>",
            W::URI
        ));
        let root = dom.root(document).unwrap();
        (root, dom.element(root, &W::body()).unwrap())
    }

    fn paragraph(text: &str) -> String {
        format!("<w:p><w:r><w:t>{text}</w:t></w:r></w:p>")
    }

    fn control(text: &str, block: bool) -> String {
        let content = if block {
            paragraph(text)
        } else {
            format!("<w:r><w:t>{text}</w:t></w:r>")
        };
        let control = format!(
            "<w:sdt><w:sdtPr><w:alias w:val=\"Clause\"/><w:id w:val=\"11\"/></w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>"
        );
        if block {
            control
        } else {
            format!("<w:p>{control}</w:p>")
        }
    }

    fn text(dom: &Dom, root: NodeId) -> String {
        dom.descendants(root, Some(&W::t()))
            .into_iter()
            .map(|node| dom.value(node))
            .collect()
    }

    #[test]
    fn introducing_or_removing_a_control_keeps_replacement_words_in_both_source_positions() {
        let before = "The second party shall deliver the updated report within sixty days after receiving the signed request from the first party.";
        let after = "The first party shall deliver the complete report within thirty days after receiving the written request from the other party.";
        for block in [false, true] {
            for reverse in [false, true] {
                let (left, right, old_text, new_text) = if reverse {
                    (control(after, block), paragraph(before), after, before)
                } else {
                    (paragraph(before), control(after, block), before, after)
                };
                let mut dom = Dom::new();
                let (a_root, a_body) = document(&mut dom, &left);
                let (b_root, b_body) = document(&mut dom, &right);
                let compared = compare_bodies_faithful(
                    &mut dom,
                    a_root,
                    b_root,
                    a_body,
                    b_body,
                    &WmlComparerSettings::default(),
                );
                let accept_root = dom.clone_subtree(compared);
                let accepted = accept_revisions_document(&mut dom, accept_root);
                let rejected = reject_revisions_document(&mut dom, compared);
                assert_eq!(
                    text(&dom, accepted),
                    new_text,
                    "accept, block={block}, reverse={reverse}"
                );
                assert_eq!(
                    text(&dom, rejected),
                    old_text,
                    "reject, block={block}, reverse={reverse}"
                );
            }
        }
    }

    #[test]
    fn equal_control_ancestry_keeps_its_wrapper_and_authored_properties() {
        for block in [false, true] {
            let body = control("Unchanged controlled clause", block);
            let mut dom = Dom::new();
            let (a_root, a_body) = document(&mut dom, &body);
            let (b_root, b_body) = document(&mut dom, &body);
            let compared = compare_bodies_faithful(
                &mut dom,
                a_root,
                b_root,
                a_body,
                b_body,
                &WmlComparerSettings::default(),
            );
            assert_eq!(text(&dom, compared), "Unchanged controlled clause");
            let controls = dom.descendants(compared, Some(&W::sdt()));
            assert_eq!(
                controls.len(),
                1,
                "aligned equal controls must not be flattened"
            );
            let properties = dom.element(controls[0], &W::name("sdtPr")).unwrap();
            assert_eq!(
                dom.attribute(
                    dom.element(properties, &W::name("alias")).unwrap(),
                    &W::val()
                ),
                Some("Clause")
            );
            assert_eq!(
                dom.attribute(dom.element(properties, &W::id()).unwrap(), &W::val()),
                Some("11")
            );
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod run_format_history_regressions {
    use super::*;
    use crate::comparer::atoms::FormatChangeInfo;
    use crate::comparer::finalize::mark_content_transform;
    use crate::revision_processor::{accept_revisions_document, reject_revisions_document};
    use std::sync::Arc;

    fn run_snapshot(dom: &Dom, p: NodeId) -> Vec<(String, bool, bool, Option<String>)> {
        dom.descendants(p, Some(&W::r()))
            .into_iter()
            .map(|r| {
                let props = dom.element(r, &W::r_pr());
                let has = |name| props.is_some_and(|pr| dom.element(pr, &W::name(name)).is_some());
                let highlight = props
                    .and_then(|pr| dom.element(pr, &W::name("highlight")))
                    .and_then(|h| dom.attribute(h, &W::val()))
                    .map(str::to_string);
                let text = dom
                    .descendants(r, Some(&W::t()))
                    .into_iter()
                    .map(|t| dom.value(t))
                    .collect();
                (text, has("b"), has("i"), highlight)
            })
            .collect()
    }

    #[test]
    fn one_new_run_preserves_each_original_format_boundary_when_rejected() {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(r#"<w:p xmlns:w="{}"><w:r><w:rPr><w:i/><w:highlight w:val="yellow"/></w:rPr><w:t>left party right</w:t></w:r></w:p>"#, W::URI));
        let new_p = dom.root(doc).unwrap();
        let new_r = dom.element(new_p, &W::r()).unwrap();
        let new_t = dom.element(new_r, &W::t()).unwrap();
        let new_props = dom.element(new_r, &W::r_pr()).unwrap();
        let old_props = dom.new_element(W::r_pr());
        let bold = dom.new_element(W::name("b"));
        dom.add(old_props, bold);
        let ancestry: Arc<[NodeId]> = vec![new_p, new_r, new_t].into();
        let unids: Arc<[String]> = vec![
            "paragraph".into(),
            "one-new-run".into(),
            "one-new-text".into(),
        ]
        .into();
        let mut atoms = Vec::new();
        for (text, original) in [
            ("left ", None),
            ("party", Some(old_props)),
            (" right", None),
        ] {
            for character in text.chars() {
                let leaf = dom.new_element(W::t());
                dom.add_text(leaf, &character.to_string());
                let mut atom =
                    ComparisonUnitAtom::new(leaf, Arc::clone(&ancestry), "format-regression");
                atom.ancestor_unids = Some(Arc::clone(&unids));
                atom.correlation_status = CorrelationStatus::FormatChanged;
                atom.format_change = Some(FormatChangeInfo {
                    old_run_properties: original,
                    new_run_properties: Some(new_props),
                    old_para_properties: None,
                    changed_properties: vec!["italic".into(), "highlight".into()],
                });
                atoms.push(atom);
            }
        }
        let settings = WmlComparerSettings::default();
        let refs: Vec<_> = atoms.iter().collect();
        let mut revision_id = 1;
        let assembled = coalesce_recurse(&mut dom, &refs, 0, &settings, &mut revision_id);
        assert_eq!(assembled.len(), 1);
        let marked = mark_content_transform(&mut dom, assembled[0], &settings, &mut revision_id);
        assert_eq!(marked.len(), 1);
        let redline = marked[0];
        assert_eq!(dom.descendants(redline, Some(&W::r_pr_change())).len(), 3);
        let accept_root = dom.clone_subtree(redline);
        let accepted = accept_revisions_document(&mut dom, accept_root);
        assert_eq!(
            run_snapshot(&dom, accepted),
            [
                ("left ".into(), false, true, Some("yellow".into())),
                ("party".into(), false, true, Some("yellow".into())),
                (" right".into(), false, true, Some("yellow".into())),
            ]
        );
        let reject_root = dom.clone_subtree(redline);
        let rejected = reject_revisions_document(&mut dom, reject_root);
        assert_eq!(
            run_snapshot(&dom, rejected),
            [
                ("left ".into(), false, false, None),
                ("party".into(), true, false, None),
                (" right".into(), false, false, None),
            ]
        );
        assert!(
            dom.descendants(accepted, Some(&W::r_pr_change()))
                .is_empty()
        );
        assert!(
            dom.descendants(rejected, Some(&W::r_pr_change()))
                .is_empty()
        );
        assert_eq!(dom.value(old_props), "");
        assert!(dom.element(old_props, &W::name("b")).is_some());
    }

    #[test]
    fn inline_containers_preserve_each_original_format_boundary_when_rejected() {
        for wrapper in ["sdt", "hyperlink"] {
            let mut dom = Dom::new();
            let run = "<w:r><w:rPr><w:i/><w:highlight w:val=\"yellow\"/></w:rPr><w:t>left party right</w:t></w:r>";
            let content = if wrapper == "sdt" {
                format!(
                    "<w:sdt><w:sdtPr><w:id w:val=\"11\"/><w:alias w:val=\"Clause\"/><w:tag w:val=\"authored-clause\"/></w:sdtPr><w:sdtContent>{run}</w:sdtContent></w:sdt>"
                )
            } else {
                format!("<w:hyperlink w:anchor=\"Clause\">{run}</w:hyperlink>")
            };
            let doc = dom.parse_xdocument(&format!("<w:p xmlns:w=\"{}\">{content}</w:p>", W::URI));
            let new_p = dom.root(doc).unwrap();
            let new_r = dom.descendants(new_p, Some(&W::r()))[0];
            let new_t = dom.element(new_r, &W::t()).unwrap();
            let new_props = dom.element(new_r, &W::r_pr()).unwrap();
            let old_props = dom.new_element(W::r_pr());
            let bold = dom.new_element(W::name("b"));
            dom.add(old_props, bold);
            let container = dom.descendants(new_p, Some(&W::name(wrapper)))[0];
            let mut ancestry = vec![new_p, container];
            if wrapper == "sdt" {
                ancestry.push(dom.element(container, &W::sdt_content()).unwrap());
            }
            ancestry.extend([new_r, new_t]);
            let unids: Arc<[String]> = (0..ancestry.len())
                .map(|i| format!("ancestor-{i}"))
                .collect::<Vec<_>>()
                .into();
            let ancestry: Arc<[NodeId]> = ancestry.into();
            let mut atoms = Vec::new();
            for (text, original) in [
                ("left ", None),
                ("party", Some(old_props)),
                (" right", None),
            ] {
                for character in text.chars() {
                    let leaf = dom.new_element(W::t());
                    dom.add_text(leaf, &character.to_string());
                    let mut atom =
                        ComparisonUnitAtom::new(leaf, Arc::clone(&ancestry), "format-regression");
                    atom.ancestor_unids = Some(Arc::clone(&unids));
                    atom.correlation_status = CorrelationStatus::FormatChanged;
                    atom.format_change = Some(FormatChangeInfo {
                        old_run_properties: original,
                        new_run_properties: Some(new_props),
                        old_para_properties: None,
                        changed_properties: vec!["italic".into(), "highlight".into()],
                    });
                    atoms.push(atom);
                }
            }
            let settings = WmlComparerSettings::default();
            let refs: Vec<_> = atoms.iter().collect();
            let mut revision_id = 1;
            let assembled = coalesce_recurse(&mut dom, &refs, 0, &settings, &mut revision_id);
            assert_eq!(assembled.len(), 1);
            let marked =
                mark_content_transform(&mut dom, assembled[0], &settings, &mut revision_id);
            assert_eq!(marked.len(), 1);
            let redline = marked[0];
            assert_eq!(dom.descendants(redline, Some(&W::r_pr_change())).len(), 3);
            let accept_root = dom.clone_subtree(redline);
            let accepted = accept_revisions_document(&mut dom, accept_root);
            assert_eq!(
                run_snapshot(&dom, accepted),
                [
                    ("left ".into(), false, true, Some("yellow".into())),
                    ("party".into(), false, true, Some("yellow".into())),
                    (" right".into(), false, true, Some("yellow".into())),
                ]
            );
            let reject_root = dom.clone_subtree(redline);
            let rejected = reject_revisions_document(&mut dom, reject_root);
            assert_eq!(
                run_snapshot(&dom, rejected),
                [
                    ("left ".into(), false, false, None),
                    ("party".into(), true, false, None),
                    (" right".into(), false, false, None),
                ]
            );
            assert!(
                dom.descendants(accepted, Some(&W::r_pr_change()))
                    .is_empty()
            );
            assert!(
                dom.descendants(rejected, Some(&W::r_pr_change()))
                    .is_empty()
            );
            assert_eq!(dom.value(old_props), "");
            assert!(dom.element(old_props, &W::name("b")).is_some());
            for output in [redline, accepted, rejected] {
                assert_eq!(
                    dom.descendants(output, Some(&W::p())).len(),
                    0,
                    "root is the only paragraph, wrapper={wrapper}"
                );
                let containers = dom.descendants(output, Some(&W::name(wrapper)));
                assert_eq!(containers.len(), 1, "wrapper={wrapper}");
                if wrapper == "hyperlink" {
                    assert_eq!(
                        dom.attribute(containers[0], &W::name("anchor")),
                        Some("Clause")
                    );
                } else {
                    let properties = dom.element(containers[0], &W::name("sdtPr")).unwrap();
                    for (name, value) in [
                        ("id", "11"),
                        ("alias", "Clause"),
                        ("tag", "authored-clause"),
                    ] {
                        assert_eq!(
                            dom.attribute(
                                dom.element(properties, &W::name(name)).unwrap(),
                                &W::val()
                            ),
                            Some(value)
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod authored_control_identity_regressions {
    use super::same_authored_control;
    use crate::namespaces::W;
    use crate::xmllinq::Dom;

    #[test]
    fn nonempty_tags_override_regenerated_ids_and_empty_tags_allow_fallbacks() {
        for (left, right, expected) in [
            (
                "<w:tag w:val=\"Clause\"/><w:id w:val=\"11\"/>",
                "<w:tag w:val=\"Clause\"/><w:id w:val=\"13\"/>",
                true,
            ),
            (
                "<w:tag w:val=\"\"/><w:id w:val=\"11\"/>",
                "<w:tag w:val=\"\"/><w:id w:val=\"11\"/>",
                true,
            ),
            (
                "<w:tag w:val=\"\"/><w:alias w:val=\"Clause\"/>",
                "<w:tag w:val=\"\"/><w:alias w:val=\"Clause\"/>",
                true,
            ),
            (
                "<w:tag w:val=\"Clause\"/><w:id w:val=\"11\"/>",
                "<w:tag w:val=\"Wrapper\"/><w:id w:val=\"11\"/>",
                false,
            ),
        ] {
            let mut dom = Dom::new();
            let document = dom.parse_xdocument(&format!(
                "<w:document xmlns:w=\"{}\"><w:sdt><w:sdtPr>{left}</w:sdtPr></w:sdt><w:sdt><w:sdtPr>{right}</w:sdtPr></w:sdt></w:document>", W::URI
            ));
            let root = dom.root(document).unwrap();
            let controls = dom.descendants(root, Some(&W::sdt()));
            assert_eq!(
                same_authored_control(&dom, controls[0], controls[1]),
                expected,
                "left={left}, right={right}"
            );
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod repeated_control_tag_regressions {
    use crate::comparer::{WmlComparerSettings, compare_bodies_faithful};
    use crate::namespaces::W;
    use crate::revision_processor::{accept_revisions_document, reject_revisions_document};
    use crate::xmllinq::{Dom, NodeId};

    fn document(dom: &mut Dom, body: &str) -> (NodeId, NodeId) {
        let doc = dom.parse_xdocument(&format!(
            "<w:document xmlns:w='{}'><w:body>{body}<w:sectPr/></w:body></w:document>",
            W::URI
        ));
        let root = dom.root(doc).unwrap();
        (root, dom.element(root, &W::body()).unwrap())
    }

    fn controlled(content: &str, id: &str, alias: &str) -> String {
        format!(
            "<w:sdt><w:sdtPr><w:alias w:val='{alias}'/><w:tag w:val='Clause'/><w:id w:val='{id}'/></w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>"
        )
    }

    fn text(dom: &Dom, root: NodeId) -> String {
        dom.descendants(root, Some(&W::t()))
            .into_iter()
            .map(|n| dom.value(n))
            .collect()
    }

    #[test]
    fn repeated_tag_outer_wrapper_never_replaces_retained_inner_control_metadata() {
        for block in [false, true] {
            for reverse in [false, true] {
                for settings in [
                    WmlComparerSettings::default(),
                    WmlComparerSettings::powertools_faithful(),
                ] {
                    let words = "The retained clause shall remain in its authored control.";
                    let run = format!("<w:r><w:t>{words}</w:t></w:r>");
                    let content = if block {
                        format!("<w:p>{run}</w:p>")
                    } else {
                        run
                    };
                    let inner = controlled(&content, "11", "Inner");
                    let outer = controlled(&inner, "22", "Outer");
                    let wrap = |body: &str| {
                        if block {
                            body.to_string()
                        } else {
                            format!("<w:p>{body}</w:p>")
                        }
                    };
                    let (left, right) = if reverse {
                        (wrap(&outer), wrap(&inner))
                    } else {
                        (wrap(&inner), wrap(&outer))
                    };
                    let mut dom = Dom::new();
                    let (ar, ab) = document(&mut dom, &left);
                    let (br, bb) = document(&mut dom, &right);
                    let compared = compare_bodies_faithful(&mut dom, ar, br, ab, bb, &settings);
                    let ac = dom.clone_subtree(compared);
                    let accepted = accept_revisions_document(&mut dom, ac);
                    let rc = dom.clone_subtree(compared);
                    let rejected = reject_revisions_document(&mut dom, rc);
                    for projection in [compared, accepted, rejected] {
                        assert_eq!(
                            text(&dom, projection),
                            words,
                            "block={block}, reverse={reverse}"
                        );
                        let controls = dom.descendants(projection, Some(&W::sdt()));
                        assert_eq!(
                            controls.len(),
                            1,
                            "only the retained control is correlated; block={block}, reverse={reverse}"
                        );
                        let props = dom.element(controls[0], &W::name("sdtPr")).unwrap();
                        for (name, expected) in
                            [("id", "11"), ("tag", "Clause"), ("alias", "Inner")]
                        {
                            let property = dom.element(props, &W::name(name)).unwrap();
                            assert_eq!(
                                dom.attribute(property, &W::val()),
                                Some(expected),
                                "block={block}, reverse={reverse}, property={name}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod reordered_retained_control_identity_regression {
    use super::*;
    use std::sync::Arc;

    fn source(dom: &mut Dom, outer: &str, inner: &str) -> ComparisonUnitAtom {
        let doc = dom.parse_xdocument(&format!(
            "<w:p xmlns:w='{}'><w:sdt><w:sdtPr><w:tag w:val='Clause'/><w:id w:val='{outer}'/><w:alias w:val='Control-{outer}'/></w:sdtPr><w:sdtContent><w:sdt><w:sdtPr><w:tag w:val='Clause'/><w:id w:val='{inner}'/><w:alias w:val='Control-{inner}'/></w:sdtPr><w:sdtContent><w:r><w:t>shared</w:t></w:r></w:sdtContent></w:sdt></w:sdtContent></w:sdt></w:p>", W::URI
        ));
        let root = dom.root(doc).unwrap();
        let leaf = dom.descendants(root, Some(&W::t()))[0];
        let mut path = dom.ancestors(leaf, None);
        path.reverse();
        path.push(leaf);
        let mut atom = ComparisonUnitAtom::new(leaf, path, "shared");
        atom.correlation_status = CorrelationStatus::Equal;
        atom
    }

    fn ids(dom: &Dom, atom: &ComparisonUnitAtom) -> Vec<String> {
        atom.ancestor_elements
            .iter()
            .copied()
            .filter(|&node| dom.name_is(node, &W::sdt()))
            .map(|sdt| {
                let pr = dom.element(sdt, &W::name("sdtPr")).unwrap();
                let id = dom.element(pr, &W::id()).unwrap();
                dom.attribute(id, &W::val()).unwrap().to_string()
            })
            .collect()
    }

    #[test]
    fn both_retained_controls_cannot_be_stripped_when_their_nesting_order_changes() {
        let mut dom = Dom::new();
        let before = source(&mut dom, "11", "22");
        let mut after = source(&mut dom, "22", "11");
        after.comparison_unit_atom_before = Some(Arc::new(before));
        let mut atoms = [after];
        align_content_control_ancestry(&dom, &mut atoms);
        assert_eq!(
            ids(&dom, &atoms[0]),
            ["22", "11"],
            "both after-side IDs also exist before; neither wrapper is surplus"
        );
        assert_eq!(
            ids(&dom, atoms[0].comparison_unit_atom_before.as_ref().unwrap()),
            ["11", "22"],
            "both before-side IDs also exist after; neither wrapper is surplus"
        );
    }

    #[test]
    fn stronger_unilateral_id_match_beats_opposite_regenerated_id_tag_match() {
        let mut dom = Dom::new();
        let before = source(&mut dom, "11", "99");
        let mut after = source(&mut dom, "22", "11");
        for (atom, id) in [(&before, "99"), (&after, "22")] {
            let controls: Vec<_> = atom
                .ancestor_elements
                .iter()
                .copied()
                .filter(|&node| dom.name_is(node, &W::sdt()))
                .collect();
            for control in controls {
                let pr = dom.element(control, &W::name("sdtPr")).unwrap();
                let identifier = dom.element(pr, &W::id()).unwrap();
                if dom.attribute(identifier, &W::val()) == Some(id) {
                    let tag = dom.element(pr, &W::name("tag")).unwrap();
                    dom.set_attribute_value(tag, &W::val(), Some("Wrapper"));
                }
            }
        }
        after.comparison_unit_atom_before = Some(Arc::new(before));
        let mut atoms = [after];
        align_content_control_ancestry(&dom, &mut atoms);
        assert_eq!(ids(&dom, &atoms[0]), ["11"]);
        assert_eq!(
            ids(&dom, atoms[0].comparison_unit_atom_before.as_ref().unwrap()),
            ["11"]
        );
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_ruby_math_ownership {
    use super::*;
    use crate::namespaces::M;

    fn semantic(dom: &Dom, node: NodeId) -> String {
        let Some(name) = dom.name(node) else {
            return format!("text:{:?}", dom.text_value(node));
        };
        let mut attrs = dom
            .attributes(node)
            .into_iter()
            .filter(|(name, _)| {
                name.namespace_name() != PT::URI
                    && name.namespace_name() != "http://www.w3.org/2000/xmlns/"
            })
            .map(|(name, value)| {
                (
                    format!("{{{}}}{}", name.namespace_name(), name.local_name()),
                    value.to_owned(),
                )
            })
            .collect::<Vec<_>>();
        attrs.sort();
        let mut result = format!(
            "{{{}}}{} {:?}",
            name.namespace_name(),
            name.local_name(),
            attrs
        );
        for child in dom.nodes(node) {
            let child = semantic(dom, child);
            result.push_str(&format!("{}:{child}", child.len()));
        }
        result
    }

    #[test]
    fn ruby_reassembly_retains_reading_base_and_property_owners() {
        for explicit_format in [false, true] {
            for text in ["A", "base words", "甲乙"] {
                let props = if explicit_format {
                    "<w:rPr><w:i/><w:color w:val=\"123456\"/></w:rPr>"
                } else {
                    ""
                };
                let xml = format!(
                    "<w:body xmlns:w=\"{}\"><w:p><w:pPr><w:spacing w:after=\"120\"/></w:pPr><w:r>{props}<w:ruby><w:rubyPr><w:rubyAlign w:val=\"center\"/><w:hps w:val=\"12\"/><w:hpsRaise w:val=\"10\"/><w:hpsBaseText w:val=\"20\"/><w:lid w:val=\"ja-JP\"/></w:rubyPr><w:rt><w:r><w:rPr><w:b/></w:rPr><w:t>reading</w:t></w:r></w:rt><w:rubyBase><w:r>{props}<w:t>{text}</w:t></w:r></w:rubyBase></w:ruby></w:r></w:p></w:body>",
                    W::URI
                );
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&xml);
                let body = dom.root(doc).unwrap();
                let original = dom.elements(body, Some(&W::p()))[0];
                let expected = semantic(&dom, original);
                let settings = WmlComparerSettings::default();
                let mut atoms = super::super::atomize::create_comparison_unit_atom_list(
                    &mut dom, body, &settings,
                );
                for atom in &mut atoms {
                    atom.correlation_status = CorrelationStatus::Equal;
                }
                assemble_ancestor_unids(&mut dom, &mut atoms);
                let mut id = 200;
                let result = produce_new_wml_markup_from_correlated_sequence(
                    &mut dom, &atoms, &settings, &mut id,
                );
                assert_eq!(result.len(), 1);
                // Raw coalescing emits the paragraph-mark properties last;
                // the public producer orders them before serializing the tree.
                super::super::finalize::move_paragraph_properties_first(&mut dom, result[0]);
                assert_eq!(dom.value(result[0]), format!("reading{text}"));
                let reading = dom.descendants(result[0], Some(&W::name("rt")))[0];
                let base = dom.descendants(result[0], Some(&W::name("rubyBase")))[0];
                assert_eq!(dom.value(reading), "reading");
                assert_eq!(dom.value(base), text);
                assert_eq!(
                    semantic(&dom, result[0]),
                    expected,
                    "format={explicit_format} text={text}"
                );
                assert_eq!(id, 200, "equal content must not mint revision IDs");
            }
        }
    }

    #[test]
    fn existing_math_revisions_keep_attribution_and_new_siblings_are_marked_once() {
        for existing in ["ins", "del"] {
            for added in ["ins", "del"] {
                for explicit_format in [false, true] {
                    let format = if explicit_format {
                        "<w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\"/><w:b/></w:rPr>"
                    } else {
                        ""
                    };
                    let xml = format!(
                        "<m:oMath xmlns:m=\"{}\" xmlns:w=\"{}\"><m:r><w:{existing} w:id=\"7\" w:author=\"Prior\" w:date=\"2025-01-01T00:00:00Z\"><w:rPr><w:i/></w:rPr><m:t>old</m:t></w:{existing}></m:r><m:r><m:rPr><m:sty m:val=\"p\"/></m:rPr>{format}<m:t>new</m:t></m:r><m:f><m:fPr><m:ctrlPr><w:rPr><w:color w:val=\"445566\"/></w:rPr></m:ctrlPr></m:fPr><m:num/><m:den/></m:f></m:oMath>",
                        M::URI,
                        W::URI
                    );
                    let mut dom = Dom::new();
                    let doc = dom.parse_xdocument(&xml);
                    let math = dom.root(doc).unwrap();
                    let prior = dom.descendants(math, Some(&W::name(existing)))[0];
                    let frozen = semantic(&dom, prior);
                    let mut id = 30;
                    let settings = WmlComparerSettings::default();
                    mark_math_revisions_internally(
                        &mut dom,
                        math,
                        &W::name(added),
                        &settings,
                        &mut id,
                    );
                    assert_eq!(semantic(&dom, prior), frozen);
                    assert_eq!(id, 32, "only live run and control properties get new marks");
                    let runs = dom.descendants(math, Some(&M::name("r")));
                    assert_eq!(dom.value(runs[0]), "old");
                    assert_eq!(dom.value(runs[1]), "new");
                    let mark = dom.elements(runs[1], Some(&W::name(added)))[0];
                    assert_eq!(dom.attribute(mark, &W::id()), Some("30"));
                    assert_eq!(
                        dom.attribute(mark, &W::author()),
                        Some(settings.author_for_revisions.as_str())
                    );
                    assert_eq!(
                        dom.attribute(mark, &W::date()),
                        Some(settings.date_time_for_revisions.as_str())
                    );
                    let rpr = dom.elements(mark, Some(&W::r_pr()))[0];
                    assert_eq!(
                        dom.descendants(rpr, Some(&W::name("b"))).len(),
                        usize::from(explicit_format)
                    );
                    let before_second = semantic(&dom, math);
                    mark_math_revisions_internally(
                        &mut dom,
                        math,
                        &W::name(added),
                        &settings,
                        &mut id,
                    );
                    assert_eq!(semantic(&dom, math), before_second);
                    assert_eq!(id, 32);
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod nontext_run_format_history_tests {
    use super::*;
    use crate::comparer::atoms::FormatChangeInfo;
    use crate::comparer::finalize::mark_content_transform;
    use crate::revision_processor::{accept_revisions_document, reject_revisions_document};
    use std::sync::Arc;

    fn signature(dom: &Dom, node: NodeId) -> String {
        let mut attrs = dom
            .attributes(node)
            .into_iter()
            .filter(|(name, _)| !dom.is_namespace_declaration(name))
            .map(|(name, value)| (format!("{name:?}"), value))
            .collect::<Vec<_>>();
        attrs.sort();
        let children = dom
            .nodes(node)
            .into_iter()
            .map(|child| {
                let value = signature(dom, child);
                format!("{}:{value}", value.len())
            })
            .collect::<String>();
        format!(
            "{:?}:{attrs:?}:{:?}:{children}",
            dom.name(node),
            dom.text_value(node)
        )
    }

    #[test]
    fn changed_nontext_run_payload_keeps_complete_original_and_revised_properties() {
        for leaf in [
            "<w:footnoteReference w:id='1023'/>",
            "<w:endnoteReference w:id='2023'/>",
            "<w:tab/>",
            "<w:br w:type='textWrapping' w:clear='all'/>",
            "<w:instrText xml:space='preserve'> DATE \\@ yyyy </w:instrText>",
            "<w:fldChar w:fldCharType='begin' w:fldLock='1'/>",
        ] {
            for old_empty in [false, true] {
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&format!(
                    "<w:p xmlns:w='{}'><w:r><w:rPr><w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:b/><w:color w:val='123456'/><w:sz w:val='22'/><w:lang w:val='en-US'/></w:rPr>{leaf}</w:r></w:p>", W::URI));
                let paragraph = dom.root(doc).unwrap();
                let run = dom.element(paragraph, &W::r()).unwrap();
                let new_props = dom.element(run, &W::r_pr()).unwrap();
                let payload = dom
                    .elements(run, None)
                    .into_iter()
                    .find(|&child| !dom.name_is(child, &W::r_pr()))
                    .unwrap();
                let old_doc = dom.parse_xdocument(&format!(
                    "<w:rPr xmlns:w='{}'>{}</w:rPr>", W::URI,
                    if old_empty { "" } else { "<w:rFonts w:ascii='Cambria' w:hAnsi='Cambria'/><w:i/><w:color w:val='654321'/><w:sz w:val='24'/><w:lang w:val='fr-FR'/>" }));
                let old_props = dom.root(old_doc).unwrap();
                let before = signature(&dom, old_props);
                let after = signature(&dom, new_props);
                let payload_before = signature(&dom, payload);
                let mut atom = ComparisonUnitAtom::new(
                    payload,
                    Arc::from([paragraph, run, payload]),
                    "owned-leaf",
                );
                atom.ancestor_unids = Some(Arc::from(["p".into(), "r".into(), "leaf".into()]));
                atom.correlation_status = CorrelationStatus::FormatChanged;
                atom.format_change = Some(FormatChangeInfo {
                    old_run_properties: (!old_empty).then_some(old_props),
                    new_run_properties: Some(new_props),
                    old_para_properties: None,
                    changed_properties: vec!["runFormatting".into()],
                });
                let settings = WmlComparerSettings::default();
                let mut id = 61;
                let rebuilt = coalesce_recurse(&mut dom, &[&atom], 0, &settings, &mut id);
                assert_eq!(rebuilt.len(), 1);
                let marked = mark_content_transform(&mut dom, rebuilt[0], &settings, &mut id);
                assert_eq!(marked.len(), 1);
                let redline = marked[0];
                let changes = dom.descendants(redline, Some(&W::r_pr_change()));
                assert_eq!(changes.len(), 1, "leaf={leaf}, old_empty={old_empty}");
                let history = changes[0];
                assert_eq!(
                    dom.attribute(history, &W::author()),
                    Some(settings.author_for_revisions.as_str())
                );
                assert_eq!(
                    dom.attribute(history, &W::date()),
                    Some(settings.date_time_for_revisions.as_str())
                );
                assert_eq!(
                    signature(&dom, dom.element(history, &W::r_pr()).unwrap()),
                    before
                );
                // Match the complete producer pipeline: scratch carriers are
                // internal to finalization, never authored payload attributes.
                crate::comparer::finalize::remove_powertools_scratch_markup(&mut dom, redline);
                let accepted_input = dom.clone_subtree(redline);
                let accepted = accept_revisions_document(&mut dom, accepted_input);
                let rejected = reject_revisions_document(&mut dom, redline);
                for (projection, props) in [(accepted, &after), (rejected, &before)] {
                    let runs = dom.descendants(projection, Some(&W::r()));
                    assert_eq!(runs.len(), 1);
                    let rpr = dom.element(runs[0], &W::r_pr()).unwrap();
                    assert_eq!(
                        signature(&dom, rpr),
                        *props,
                        "leaf={leaf}, old_empty={old_empty}"
                    );
                    let children = dom
                        .elements(runs[0], None)
                        .into_iter()
                        .filter(|&child| !dom.name_is(child, &W::r_pr()))
                        .collect::<Vec<_>>();
                    assert_eq!(children.len(), 1);
                    assert_eq!(signature(&dom, children[0]), payload_before);
                    assert!(
                        dom.descendants(projection, Some(&W::r_pr_change()))
                            .is_empty()
                    );
                }
                assert_eq!(signature(&dom, old_props), before);
                assert_eq!(signature(&dom, new_props), after);
                assert_eq!(signature(&dom, payload), payload_before);
            }
        }
    }
}
