//! Atomization + coalesce (M4.1). Port of `CreateComparisonUnitAtomList`,
//! `CreateComparisonUnitAtomListRecurse`, `Coalesce`, `CoalesceRecurseSimple`.
//!
//! `CreateComparisonUnitAtomList` flattens a content tree into a stream of
//! per-character / per-leaf atoms, each remembering its ancestor chain (outermost
//! → leaf, excluding `w:body`). `Coalesce` rebuilds the tree by regrouping atoms
//! on each ancestor's `pt14:Unid` at successive depths. The round-trip
//! `coalesce(atomize(body))` reconstructs a structurally-equal body — the
//! invariant the whole comparer relies on.

use std::sync::Arc;

use crate::namespaces::{MC, PT, W};
use crate::unid::assign_to_all_elements;
use crate::util::group_adjacent;
use crate::util::sha1::sha1_hex;
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

use super::atoms::ComparisonUnitAtom;
use super::tables::{
    ALLOWABLE_RUN_CHILDREN, ELEMENTS_TO_THROW_AWAY, INVALID_ELEMENTS, recursion_info,
};
use super::{CorrelationStatus, WmlComparerSettings};

/// `VerifyNoInvalidContent` (:8678) — error if any descendant is an
/// `InvalidElements` member. Returns the offending local name on failure.
pub fn verify_no_invalid_content(dom: &Dom, content_parent: NodeId) -> Result<(), String> {
    for d in dom.descendants(content_parent, None) {
        if let Some(name) = dom.name(d)
            && INVALID_ELEMENTS.contains(&name)
        {
            return Err(format!("Document contains {}", name.local_name()));
        }
    }
    Ok(())
}

/// `MoveLastSectPrIntoLastParagraph` (:8819) — move a trailing body-level
/// `w:sectPr` into the last paragraph's `w:pPr`. Errors on >1 direct sectPr.
pub fn move_last_sectpr_into_last_paragraph(
    dom: &mut Dom,
    content_parent: NodeId,
) -> Result<(), String> {
    let sectprs = dom.elements(content_parent, Some(&W::sect_pr()));
    if sectprs.len() > 1 {
        return Err("Invalid document: multiple body-level sectPr".to_string());
    }
    let Some(last_sectpr) = sectprs.first().copied() else {
        return Ok(());
    };
    // last direct-child paragraph, else last descendant paragraph
    let last_para = dom
        .elements(content_parent, Some(&W::p()))
        .last()
        .copied()
        .or_else(|| {
            dom.descendants(content_parent, Some(&W::p()))
                .last()
                .copied()
        });
    let Some(last_para) = last_para else {
        // degenerate: no paragraph — leave the body-level sectPr in place
        return Ok(());
    };
    let ppr = match dom.element(last_para, &W::p_pr()) {
        Some(pp) => pp,
        None => {
            let pp = dom.new_element(W::p_pr());
            dom.add_first(last_para, pp);
            pp
        }
    };
    let moved = dom.clone_subtree(last_sectpr);
    dom.add(ppr, moved);
    for sp in dom.elements(content_parent, Some(&W::sect_pr())) {
        dom.remove(sp);
    }
    Ok(())
}

/// D.1 — `GetRevisionTrackingElementFromAncestors` (:8945): the `w:del`/
/// `w:ins`/`w:moveFrom`/`w:moveTo` element that gives an atom its status.
/// pPr special case: the rev-track lives in `pPr/rPr/{del,ins}` (first
/// match), not in the ancestors.
fn revision_tracking_element_from_ancestors(
    dom: &Dom,
    content: NodeId,
    ancestors: &[NodeId],
) -> Option<NodeId> {
    if dom.name(content) == Some(W::p_pr()) {
        for rpr in dom.elements(content, Some(&W::r_pr())) {
            for e in dom.elements(rpr, None) {
                let n = dom.name(e).unwrap();
                if n == W::del() || n == W::ins() {
                    return Some(e);
                }
            }
        }
        return None;
    }
    ancestors.iter().copied().find(|&a| {
        let n = dom.name(a).unwrap();
        n == W::del() || n == W::ins() || n == W::move_from() || n == W::move_to()
    })
}

/// D.1 — the ComparisonUnitAtom ctor's status mapping (:8909): derive the
/// correlation status FROM the revision tracking element's name.
fn status_from_rev_track_element(dom: &Dom, rte: Option<NodeId>) -> CorrelationStatus {
    let Some(rte) = rte else {
        return CorrelationStatus::Equal;
    };
    let n = dom.name(rte).unwrap();
    if n == W::del() {
        CorrelationStatus::Deleted
    } else if n == W::ins() {
        CorrelationStatus::Inserted
    } else if n == W::move_from() {
        CorrelationStatus::MovedSource
    } else if n == W::move_to() {
        CorrelationStatus::MovedDestination
    } else {
        // C# leaves the ctor-default status when the name matches nothing —
        // unreachable here because the finder only returns those four names.
        CorrelationStatus::Equal
    }
}

/// `GetSha1HashStringForElement` + the atom hash (localName + normalized text).
fn atom_hash(dom: &Dom, content: NodeId, settings: &WmlComparerSettings) -> String {
    let mut text = dom.value(content);
    if settings.case_insensitive {
        text = text.to_uppercase();
    }
    if settings.conflate_breaking_and_nonbreaking_spaces {
        // Faithful: GetSha1HashStringForElement does `split(" ").join(" ")`
        // (verified by hexdump :9312) — regular space U+0020 → NBSP U+00A0.
        text = text.replace(' ', "\u{00A0}");
    }
    let local = dom
        .name(content)
        .map(|n| n.local_name().to_string())
        .unwrap_or_default();
    // If a precomputed SHA1Hash attribute is present, prefer it (PreProcess path).
    if let Some(h) = dom.attribute(content, &PT::sha1_hash()) {
        return h.to_string();
    }
    sha1_hex(&format!("{local}{text}"))
}

/// `CreateComparisonUnitAtomList(contentParent)` — assign unids, then flatten.
pub fn create_comparison_unit_atom_list(
    dom: &mut Dom,
    content_parent: NodeId,
    settings: &WmlComparerSettings,
) -> Vec<ComparisonUnitAtom> {
    verify_no_invalid_content(dom, content_parent).expect("invalid content in comparer input");
    assign_to_all_elements(dom, content_parent);
    move_last_sectpr_into_last_paragraph(dom, content_parent)
        .expect("invalid document: multiple body sectPr");
    let mut list = Vec::new();
    // ATOM-STACK-01: maintain the ancestor path while recursing instead of
    // re-walking `ancestors_and_self` for every character atom.
    let mut path = Vec::new();
    recurse(dom, content_parent, &mut list, settings, &mut path);
    list
}

/// `AnnotateElementWithProps` (:8971) — recurse into child elements, skipping the
/// declared property children (which Coalesce re-attaches structurally).
fn annotate_element_with_props(
    dom: &mut Dom,
    element: NodeId,
    list: &mut Vec<ComparisonUnitAtom>,
    child_property_names: Option<&[XName]>,
    settings: &WmlComparerSettings,
    path: &mut Vec<NodeId>,
) {
    for item in dom.elements(element, None) {
        let skip = match (child_property_names, dom.name(item)) {
            (Some(props), Some(n)) => props.contains(&n),
            _ => false,
        };
        if !skip {
            recurse(dom, item, list, settings, path);
        }
    }
}

fn push_atom(
    dom: &Dom,
    content: NodeId,
    ancestors: Arc<[NodeId]>,
    list: &mut Vec<ComparisonUnitAtom>,
    settings: &WmlComparerSettings,
) {
    let mut hash = atom_hash(dom, content, settings);
    // M-MOVE S1: salt the atom hash of pt:PreDelete-stamped content (word-mode
    // flattened pre-existing deletions) so it can never correlate Equal with
    // identical live/unstamped content at word/atom level — otherwise doc A's
    // deletion history AND doc B's real insertions both vanish when B kept the
    // text (fresh-p4). Only PreDelete: pt:PreIns carries REQUIRE Equal
    // correlation with B's live copy (D1 / m32 w18). Unstamped content keeps
    // today's hash byte-identical.
    let predel = PT::name("PreDelete");
    if dom.attribute(content, &predel) == Some(super::PREDELETE_STAMP_ORIG)
        || ancestors
            .iter()
            .any(|&a| dom.attribute(a, &predel) == Some(super::PREDELETE_STAMP_ORIG))
    {
        hash = sha1_hex(&format!("PREDEL|{hash}"));
    }
    // PATH-01: store the shared Arc chain (no per-atom Vec clone).
    let mut atom = ComparisonUnitAtom::new(content, Arc::clone(&ancestors), hash);
    atom.rev_track_element =
        revision_tracking_element_from_ancestors(dom, content, ancestors.as_ref());
    atom.correlation_status = status_from_rev_track_element(dom, atom.rev_track_element);
    list.push(atom);
}

/// Chain for an atom at `element`: `path` (ancestors excluding body) + `element`.
/// PATH-01: returns `Arc` so multi-char `w:t` siblings share one allocation.
fn chain_with(path: &[NodeId], element: NodeId) -> Arc<[NodeId]> {
    let mut c = Vec::with_capacity(path.len() + 1);
    c.extend_from_slice(path);
    c.push(element);
    Arc::from(c)
}

fn recurse(
    dom: &mut Dom,
    element: NodeId,
    list: &mut Vec<ComparisonUnitAtom>,
    settings: &WmlComparerSettings,
    path: &mut Vec<NodeId>,
) {
    let name = match dom.name(element) {
        Some(n) => n,
        None => return,
    };

    // Content-root containers: walk children only (do not emit the container
    // itself as an atom). hdr/ftr are the body equivalent for header/footer
    // part compares (PR #81 writeback path).
    if name == W::body()
        || name == W::footnote()
        || name == W::endnote()
        || name == W::name("hdr")
        || name == W::name("ftr")
    {
        // Non-allocating child walk (see Dom::child_at): recurse does not
        // add/remove children of `element`, so its content is stable and the
        // index sequence equals `elements(element, None)` — without the Vec.
        // path stays empty under content roots (stop containers).
        let mut i = 0;
        while i < dom.child_count(element) {
            let item = dom.child_at(element, i);
            i += 1;
            if dom.name(item).is_some() {
                recurse(dom, item, list, settings, path);
            }
        }
        return;
    }

    if name == W::p() {
        // children except pPr (non-allocating; see the body branch above)
        path.push(element);
        let mut i = 0;
        while i < dom.child_count(element) {
            let item = dom.child_at(element, i);
            i += 1;
            match dom.name(item) {
                Some(n) if n != W::p_pr() => recurse(dom, item, list, settings, path),
                _ => {}
            }
        }
        path.pop();
        // the paragraph mark atom (pPr, or a fresh empty pPr). Faithful to
        // WmlComparer.ts: the atom's ancestor chain is the PARAGRAPH's
        // (`element.AncestorsAndSelf()`), i.e. `[…, w:p]` — NOT `[…, w:p, w:pPr]`.
        // This makes the pPr hit the leaf case in CoalesceRecurse (so its full
        // content/children are preserved as the paragraph mark).
        let para_props = dom.element(element, &W::p_pr());
        let content = match para_props {
            Some(pp) => pp,
            None => dom.new_element(W::p_pr()),
        };
        let chain = chain_with(path, element);
        push_atom(dom, content, chain, list, settings);
        return;
    }

    if name == W::r() {
        // children except rPr (non-allocating; see the body branch above)
        path.push(element);
        let mut i = 0;
        while i < dom.child_count(element) {
            let item = dom.child_at(element, i);
            i += 1;
            match dom.name(item) {
                Some(n) if n != W::r_pr() => recurse(dom, item, list, settings, path),
                _ => {}
            }
        }
        path.pop();
        return;
    }

    if name == W::t() || name == W::del_text() {
        // Own the text: we mutate the Dom while splitting into char atoms.
        let val = dom.value(element);
        // PATH-01: one shared Arc chain for every character in this text node.
        let chain = chain_with(path, element);
        for ch in val.chars() {
            // content = fresh <w:t>ch</w:t> (or delText)
            let content = dom.new_element(name.clone());
            dom.add_text(content, &ch.to_string());
            push_atom(dom, content, Arc::clone(&chain), list, settings);
        }
        return;
    }

    // mc:AlternateContent → a single opaque atom (Choice+Fallback kept verbatim).
    if name == MC::name("AlternateContent") {
        let chain = chain_with(path, element);
        push_atom(dom, element, chain, list, settings);
        return;
    }

    // w:pict → opaque leaf (like drawing / AC). Recursing into VML
    // shapetype/shape/imagedata produces zero atoms for attribute-only
    // leaves, so the reconstructed pict was an empty shell and media was
    // never referenced (file_11×file_12: Word keeps v:imagedata under
    // w:ins; ours dropped the whole image). Hash still covers nested
    // rIds via S_ELEMENTS_WITH_RELATIONSHIP_IDS on imagedata when needed.
    if name == W::pict() {
        let chain = chain_with(path, element);
        push_atom(dom, element, chain, list, settings);
        return;
    }

    // AllowableRunChildren (or w:object) → a single verbatim leaf atom.
    if ALLOWABLE_RUN_CHILDREN.contains(&name) || name == W::object() {
        let chain = chain_with(path, element);
        push_atom(dom, element, chain, list, settings);
        return;
    }

    // Empty w:fldSimple (`<w:fldSimple w:instr="PAGE"/>`, no cached result
    // run) → a single opaque atom. Recursing into it yields ZERO atoms, so
    // the field silently vanished from the redline (page-numbering footer:
    // fldSimple 3 → 0 while GT keeps every field; every rendered page showed
    // "Pg  Left aligned" with empty numbers). Non-empty fldSimple still
    // recurses (its result runs diff normally).
    if name == W::name("fldSimple") && dom.elements(element, None).is_empty() {
        let chain = chain_with(path, element);
        push_atom(dom, element, chain, list, settings);
        return;
    }

    // RecursionElements → recurse, skipping the declared property children.
    if let Some(ri) = recursion_info(&name) {
        path.push(element);
        annotate_element_with_props(
            dom,
            element,
            list,
            ri.child_property_names.as_deref(),
            settings,
            path,
        );
        path.pop();
        return;
    }

    // ElementsToThrowAway → produce no atoms.
    if ELEMENTS_TO_THROW_AWAY.contains(&name) {
        return;
    }

    // Fallthrough: recurse into all child elements.
    path.push(element);
    annotate_element_with_props(dom, element, list, None, settings, path);
    path.pop();
}

/// `Coalesce(atomList)` — rebuild a `<w:document><w:body>…` from the atom stream.
/// Returns the new document node.
pub fn coalesce(dom: &mut Dom, atoms: &[ComparisonUnitAtom]) -> NodeId {
    let doc = dom.new_document();
    let document = dom.new_element(W::document());
    // xmlns:w / xmlns:pt14 declarations (as in the TS).
    dom.set_attribute_value(document, &XNamespace::xmlns().name("w"), Some(W::URI));
    dom.set_attribute_value(document, &XNamespace::xmlns().name("pt14"), Some(PT::URI));
    let body = dom.new_element(W::body());
    let children = coalesce_recurse(dom, atoms, 0);
    for c in children {
        dom.add(body, c);
    }
    dom.add(document, body);
    dom.add(doc, document);
    doc
}

/// Port of `CoalesceRecurseSimple` — regroup atoms by ancestor Unid at `level`,
/// rebuild each ancestor element, recursing deeper.
fn coalesce_recurse(dom: &mut Dom, atoms: &[ComparisonUnitAtom], level: usize) -> Vec<NodeId> {
    // group by AncestorElements[level]'s Unid, preserving order
    let groups = group_by_ancestor_unid(dom, atoms, level);
    let mut out = Vec::new();
    for group in groups {
        let ancestor = group[0].ancestor_elements[level];
        let aname = dom.name(ancestor).unwrap();

        if aname == W::p() {
            // group adjacent by content element name
            let by_name = group_adjacent(group.iter().cloned(), |a| {
                dom.name(a.content_element).unwrap()
            });
            let p = dom.new_element(W::p());
            for (an, av) in dom.attributes(ancestor) {
                dom.set_attribute_value(p, &an, Some(&av));
            }
            // pPr group(s) first (the paragraph mark), then child runs.
            for (cname, gc) in &by_name {
                if *cname == W::p_pr() {
                    for atom in gc {
                        let cloned = dom.clone_subtree(atom.content_element);
                        dom.add(p, cloned);
                    }
                }
            }
            for (cname, gc) in &by_name {
                if *cname != W::p_pr() {
                    let children = coalesce_recurse(dom, gc, level + 1);
                    for c in children {
                        dom.add(p, c);
                    }
                }
            }
            out.push(p);
            continue;
        }

        if aname == W::r() {
            let by_name = group_adjacent(group.iter().cloned(), |a| {
                dom.name(a.content_element).unwrap()
            });
            let r = dom.new_element(W::r());
            // rPr from ancestor run
            for rpr in dom.elements(ancestor, Some(&W::r_pr())) {
                let cloned = dom.clone_subtree(rpr);
                dom.add(r, cloned);
            }
            for (cname, gc) in &by_name {
                if *cname == W::t() || *cname == W::del_text() {
                    let text: String = gc.iter().map(|a| dom.value_str(a.content_element)).collect();
                    let t = dom.new_element(cname.clone());
                    if let Some(sp) = xml_space_attr(&text) {
                        dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some(sp));
                    }
                    dom.add_text(t, &text);
                    dom.add(r, t);
                } else {
                    for atom in gc {
                        let cloned = dom.clone_subtree(atom.content_element);
                        dom.add(r, cloned);
                    }
                }
            }
            out.push(r);
            continue;
        }

        // generic ancestor: rebuild with attributes + recurse deeper
        let ne = dom.new_element(aname);
        for (an, av) in dom.attributes(ancestor) {
            dom.set_attribute_value(ne, &an, Some(&av));
        }
        let children = coalesce_recurse(dom, &group, level + 1);
        for c in children {
            dom.add(ne, c);
        }
        out.push(ne);
    }
    out
}

/// `GetXmlSpaceAttribute` — returns Some("preserve") when leading/trailing space.
fn xml_space_attr(text: &str) -> Option<&'static str> {
    if text.starts_with(' ') || text.ends_with(' ') {
        Some("preserve")
    } else {
        None
    }
}

/// Group atoms by the `pt14:Unid` of their ancestor at `level`, preserving the
/// order of first appearance (port of `groupByKey`).
fn group_by_ancestor_unid(
    dom: &Dom,
    atoms: &[ComparisonUnitAtom],
    level: usize,
) -> Vec<Vec<ComparisonUnitAtom>> {
    let unid_name = PT::unid();
    let mut order: Vec<String> = Vec::new();
    let mut map: std::collections::HashMap<String, Vec<ComparisonUnitAtom>> =
        std::collections::HashMap::new();
    for atom in atoms {
        let ancestor = atom.ancestor_elements[level];
        let key = dom
            .attribute(ancestor, &unid_name)
            .unwrap_or("")
            .to_string();
        if !map.contains_key(&key) {
            order.push(key.clone());
        }
        map.entry(key).or_default().push(atom.clone());
    }
    order.into_iter().map(|k| map.remove(&k).unwrap()).collect()
}
