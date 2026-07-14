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
        let del_text = W::name("delText");
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
        let is_ppr = dom.name(t.atom.content_element) == Some(W::p_pr());
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
    if dom.name(ppr_atom.atom.content_element) == Some(W::p_pr())
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
                n == Some(W::t()) || n == Some(W::name("delText"))
            })
            .map(|t| dom.value(t.atom.content_element))
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
    let t = dom.new_element(if deleted { W::name("delText") } else { W::t() });
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

/// M4.E.1 — `FlattenToComparisonUnitAtomList` (:4141): nested correlated tree →
/// flat status-tagged atom list. Equal carries content/ancestors from the AFTER
/// atom and a link to the BEFORE atom; zip truncates to the shorter side.
pub fn flatten_to_comparison_unit_atom_list(
    seqs: &[CorrelatedSequence],
) -> Vec<ComparisonUnitAtom> {
    let mut out = Vec::new();
    for cs in seqs {
        match cs.correlation_status {
            CorrelationStatus::Equal => {
                let before = flatten_atoms(cs.com_units_1.as_deref().unwrap_or(&[]));
                let after = flatten_atoms(cs.com_units_2.as_deref().unwrap_or(&[]));
                for (b, a) in before.iter().zip(after.iter()) {
                    let mut atom = a.clone();
                    atom.correlation_status = CorrelationStatus::Equal;
                    atom.content_element_before = Some(b.content_element);
                    atom.comparison_unit_atom_before = Some(Box::new(b.clone()));
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

fn is_ppr_atom(dom: &Dom, atom: &ComparisonUnitAtom) -> bool {
    dom.name(atom.content_element) == Some(W::p_pr())
}
fn atom_in_textbox(dom: &Dom, atom: &ComparisonUnitAtom) -> bool {
    let txbx = W::name("txbxContent");
    atom.ancestor_elements
        .iter()
        .any(|&a| dom.name(a).as_ref() == Some(&txbx))
}

/// M4.E.2 — `AssembleAncestorUnidsInOrderToRebuildXmlTreeProperly` (:3974).
/// Three phases (see WmlComparer.ts): A copy before→after pPr ancestor Unids;
/// B seed ancestor_unids from the paragraph mark (reverse walk, minting missing);
/// C fix text boxes in a second reverse pass.
pub fn assemble_ancestor_unids(dom: &mut Dom, atoms: &mut [ComparisonUnitAtom]) {
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
    let mut current: Option<Vec<String>> = None;
    let mut current_elems: Option<Vec<NodeId>> = None;
    for atom in atoms.iter_mut().rev() {
        if is_ppr_atom(dom, atom) && !atom_in_textbox(dom, atom) {
            let mut cur: Vec<String> = atom
                .ancestor_elements
                .clone()
                .into_iter()
                .map(|ae| unid_or_mint(dom, ae))
                .collect();
            if let Some(d) = &deepest_unid
                && let Some(first) = cur.first_mut()
            {
                *first = d.clone();
            }
            atom.ancestor_unids = Some(cur.clone());
            current = Some(cur);
            current_elems = Some(atom.ancestor_elements.clone());
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
            atom.ancestor_unids = Some(full);
        }
    }

    // ── Phase C (reverse, text-box fix) ─────────────────────────────────────────
    let mut current: Option<Vec<String>> = None;
    let mut skip_until_ppr = false;
    for atom in atoms.iter_mut().rev() {
        if let Some(cur) = &current
            && atom.ancestor_elements.len() < cur.len()
        {
            skip_until_ppr = true;
            current = None;
            continue;
        }
        if is_ppr_atom(dom, atom) {
            if !atom_in_textbox(dom, atom) {
                skip_until_ppr = true;
                current = None;
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
            atom.ancestor_unids = Some(cur.clone());
            current = Some(cur);
            skip_until_ppr = false;
            continue;
        }
        if skip_until_ppr {
            continue;
        }
        if let Some(cur) = &current {
            let extra: Vec<NodeId> = atom
                .ancestor_elements
                .iter()
                .skip(cur.len())
                .copied()
                .collect();
            let mut full = cur.clone();
            for ae in extra {
                full.push(unid_or_mint(dom, ae));
            }
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
        let k = key(it);
        if !map.contains_key(&k) {
            order.push(k.clone());
        }
        map.entry(k).or_default().push(*it);
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
        CorrelationStatus::Deleted => dom.set_attribute_value(node, &PT::status(), Some("Deleted")),
        CorrelationStatus::Inserted => {
            dom.set_attribute_value(node, &PT::status(), Some("Inserted"))
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
    let txbx = W::name("txbxContent");
    atom.ancestor_elements
        .iter()
        .skip(level)
        .any(|&a| dom.name(a).as_ref() == Some(&txbx))
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
    let grouped = group_by_key_stable(atoms, |ca| {
        if level >= ca.ancestor_elements.len() {
            return String::new();
        }
        let u = ca
            .ancestor_unids
            .as_ref()
            .and_then(|u| u.get(level).cloned())
            .unwrap_or_default();
        if u.is_empty() {
            return String::new();
        }
        let nm = dref
            .name(ca.ancestor_elements[level])
            .map(|n| n.local_name().to_string())
            .unwrap_or_default();
        format!("{u}|{nm}")
    });
    let grouped: Vec<_> = grouped.into_iter().filter(|(k, _)| !k.is_empty()).collect();
    if grouped.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    for (gkey, g) in grouped {
        let ancestor = g[0].ancestor_elements[level];
        let aname = dom.name(ancestor).unwrap();

        // Step 3 — group children by (next-level unid | status), txbx → Equal.
        let groupedchildren = group_adjacent(g.iter().cloned(), |gc| {
            let mut key = if level < gc.ancestor_elements.len() - 1 {
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
            key.push('|');
            key.push_str(st);
            key
        });

        // w:p
        if aname == W::p() {
            let p = dom.new_element(W::p());
            for (an, av) in dom.attributes(ancestor) {
                if an.namespace_name() != PT::URI {
                    dom.set_attribute_value(p, &an, Some(&av));
                }
            }
            // gkey is now "unid|elementName"; store only the Unid (scratch).
            let unid = gkey.split('|').next().unwrap_or(&gkey);
            dom.set_attribute_value(p, &PT::unid(), Some(unid));
            for (key, gc) in &groupedchildren {
                let spl: Vec<&str> = key.splitn(2, '|').collect();
                if spl[0].is_empty() {
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
                let spl: Vec<&str> = key.splitn(2, '|').collect();
                if spl[0].is_empty() {
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
                let text: String = gc.iter().map(|a| dom.value(a.content_element)).collect();
                let first = &gc[0];
                let elem_name = match first.correlation_status {
                    CorrelationStatus::Deleted => W::name("delText"),
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
        if aname == W::name("drawing") {
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

        // w:pict (VML image) — clone full subtree + status. Must not fall
        // through to reconstruct_element / empty Allowable shell: attribute-
        // only v:imagedata children emit no atoms when recursed (M74).
        if aname == W::name("pict") {
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
        if aname == crate::namespaces::M::name("oMath")
            || aname == crate::namespaces::M::name("oMathPara")
        {
            for (_key, gc) in &groupedchildren {
                for gcc in gc {
                    let rev = match gcc.correlation_status {
                        CorrelationStatus::Deleted => Some(W::del()),
                        CorrelationStatus::MovedSource => Some(W::name("moveFrom")),
                        CorrelationStatus::Inserted => Some(W::ins()),
                        CorrelationStatus::MovedDestination => Some(W::name("moveTo")),
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
                    | CorrelationStatus::MovedDestination => {
                        for gcc in gc {
                            let dup = dom.new_element(aname.clone());
                            for (an, av) in dom.attributes(ancestor) {
                                if an.namespace_name() != PT::URI {
                                    dom.set_attribute_value(dup, &an, Some(&av));
                                }
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
        let props: &[&str] = if aname == W::name("tbl") {
            &["tblPr", "tblGrid"]
        } else if aname == W::name("tr") {
            &["trPr"]
        } else if aname == W::name("tc") {
            &["tcPr"]
        } else if aname == W::name("sdt") {
            &["sdtPr", "sdtEndPr"]
        } else if aname == W::name("ruby") {
            &["rubyPr"]
        } else {
            &[]
        };
        let pict_props = aname == W::name("pict");
        let recon = reconstruct_element(
            dom, &g, ancestor, props, pict_props, level, settings, id_gen,
        );
        out.push(recon);
    }
    out
}

/// M4.E.6 — `ReconstructElement` (:6984): rebuild a container element, hoisting
/// the named property children first, then the recursively-coalesced children.
#[allow(clippy::too_many_arguments)]
fn reconstruct_element(
    dom: &mut Dom,
    g: &[&ComparisonUnitAtom],
    ancestor: NodeId,
    props: &[&str],
    pict_props: bool,
    level: usize,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> NodeId {
    let aname = dom.name(ancestor).unwrap();
    let new_children = coalesce_recurse(dom, g, level + 1, settings, id_gen);
    let ne = dom.new_element(aname.clone());
    for (an, av) in dom.attributes(ancestor) {
        dom.set_attribute_value(ne, &an, Some(&av));
    }
    // hoist property children (in declared order)
    if pict_props {
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
    if settings.merge_replaced_paragraphs && aname == W::name("tbl") {
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
                dom.element(ne, &W::name("tblPr")),
                dom.element(old_tbl, &W::name("tblPr")),
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
    // Word-alignment (M-TBL rule 4, parity/_scratch/table_class_forensics.md):
    // a DEGENERATE grid — fewer w:gridCol entries than the real column count
    // implied by the rows' gridSpan/tc structure — is rebuilt Word's way:
    // per-column gridCols (equal split of the page content width) plus
    // `tblW 0 auto`. GT table-vmerge-colspan_text-box: 1×4985 → 4675+4675;
    // GT nested-table-rowspan_numbered-list: 1×9970 → 4887+4905.
    if settings.merge_replaced_paragraphs && aname == W::name("tbl") {
        rebuild_degenerate_grid(dom, ne, ancestor);
    }
    ne
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
        .elements(tbl, Some(&W::name("tr")))
        .into_iter()
        .map(|tr| {
            dom.elements(tr, Some(&W::name("tc")))
                .into_iter()
                .map(|tc| {
                    dom.element(tc, &W::name("tcPr"))
                        .and_then(|pr| dom.element(pr, &W::name("gridSpan")))
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
        .map(|&root| dom.descendants(root, Some(&W::name("sectPr"))))
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
    if let Some(tbl_pr) = dom.element(tbl, &W::name("tblPr")) {
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
mod opaque_text_tests {
    //! Direct coverage for `delete_text_in_opaque` (private). Pins the
    //! status→text-kind contract — in particular that `MovedSource` renames `w:t`
    //! to `w:delText` (ISO/IEC 29500-1 §17.3.3.7: `delText` replaces `t` within a
    //! `del` *or `moveFrom`*), which is otherwise unexercised because move
    //! detection is off by default.
    use super::*;

    /// `<w:drawing><w:r><w:t>txt</w:t></w:r></w:drawing>` — an opaque subtree.
    fn opaque_with_text(d: &mut Dom, txt: &str) -> NodeId {
        let drawing = d.new_element(W::name("drawing"));
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
        let drawing = d.new_element(W::name("drawing"));
        let r = d.new_element(W::r());
        let instr = d.new_element(W::name("instrText"));
        d.add_text(instr, "FIELD");
        d.add(r, instr);
        d.add(drawing, r);
        delete_text_in_opaque(&mut d, drawing, CorrelationStatus::Deleted);
        assert_eq!(
            d.descendants(drawing, Some(&W::name("instrText"))).len(),
            1,
            "instrText untouched"
        );
        assert!(
            d.descendants(drawing, Some(&W::name("delText"))).is_empty(),
            "no delText fabricated from instrText"
        );
    }
}

#[cfg(test)]
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
