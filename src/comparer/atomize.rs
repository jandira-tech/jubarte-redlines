// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

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
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

use super::atoms::{AtomHash, ComparisonUnitAtom};
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
    // last direct-child paragraph, else last non-cell descendant paragraph
    let last_para = dom
        .elements(content_parent, Some(&W::p()))
        .last()
        .copied()
        .or_else(|| {
            dom.descendants(content_parent, Some(&W::p()))
                .into_iter()
                .rev()
                // Final body geometry is not a section break inside a table.
                // A block-control story paragraph is eligible; cell paragraphs
                // have independent structural ownership on both sources.
                .find(|&paragraph| dom.ancestors(paragraph, Some(&W::tc())).is_empty())
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
    if dom.name_is(content, &W::p_pr()) {
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
///
/// Returns the inline [`AtomHash`] digest. The precomputed-`pt:SHA1Hash` path
/// DECODES the stamped 40-char hex (`from_hex`) so it lands on the exact digest a
/// fresh `of_bytes(localName+text)` would produce for identical content — the two
/// must correlate Equal (same content; PreProcess just precomputed the hash).
/// Hashing the hex string instead would break that correlation.
fn atom_hash(dom: &Dom, content: NodeId, settings: &WmlComparerSettings) -> AtomHash {
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
        return AtomHash::from_hex(h);
    }
    // A field's begin, separate and end carry no text; without the type they
    // hash alike and one document's separate can pair with the other's begin.
    if local == "fldChar"
        && let Some(ty) = dom.attribute(content, &W::name("fldCharType"))
    {
        text.push_str(ty);
    }
    // Break kind and clearance carry visible/layout semantics, unlike run
    // formatting handled separately. Equal-correlating all empty br leaves
    // copied B's page break into A's rejected text-wrapping break.
    if dom.name_is(content, &W::name("br")) {
        for (attribute, default) in [("type", "textWrapping"), ("clear", "none")] {
            if let Some(value) = dom.attribute(content, &W::name(attribute))
                && value != default
            {
                text.push('|');
                text.push_str(attribute);
                text.push('=');
                text.push_str(value);
            }
        }
    }
    AtomHash::of_bytes(format!("{local}{text}").as_bytes())
}

/// Salts every atom of a complex field with the codes of the fields around it.
///
/// Word replaces a field whose code changed as a whole, nested fields included.
/// Matching the result text of two different fields (" Act " in `STYLEREF
/// "Name Of Act/Reg"` and in `STYLEREF "PrincipalAct_Reg`) leaves an inserted
/// and a deleted `begin` side by side, two `separate`s and crossed `end`s,
/// which crashes Word once the instructions are present. Matching the begin of
/// an `IF` whose code lost `\*MERGEFORMAT`, or its unchanged inner fields,
/// leaves fields half deleted (98bf5f3d×a3701d36). So begin, code, separate,
/// end and result all carry the whole chain of codes, outermost first: a
/// field's shell matches only when every enclosing code is the same.
fn salt_field_results(dom: &Dom, list: &mut [ComparisonUnitAtom]) {
    let fld_char = W::name("fldChar");
    let fld_char_type = W::name("fldCharType");
    let instr = W::name("instrText");
    let del_instr = W::name("delInstrText");
    let kind = |el: NodeId| -> Option<&str> {
        if dom.name(el)? == fld_char {
            dom.attribute(el, &fld_char_type)
        } else {
            None
        }
    };
    // Pass 1: each begin's code — every instruction up to its own separate
    // (or end), nested fields' codes and results included.
    let mut codes: Vec<String> = Vec::new();
    let mut code_of: Vec<Option<usize>> = vec![None; list.len()];
    // Open fields, innermost last: (index into codes, inside the result).
    let mut open: Vec<(usize, bool)> = Vec::new();
    for (i, atom) in list.iter().enumerate() {
        let el = atom.content_element;
        match kind(el) {
            Some("begin") => {
                code_of[i] = Some(codes.len());
                open.push((codes.len(), false));
                codes.push(String::new());
            }
            Some("separate") => {
                if let Some(field) = open.last_mut() {
                    field.1 = true;
                }
            }
            Some("end") => {
                open.pop();
            }
            _ => {
                if dom.name(el).is_some_and(|n| n == instr || n == del_instr) {
                    let text = dom.value(el);
                    for &(c, _) in open.iter().filter(|(_, in_result)| !in_result) {
                        codes[c].push_str(&text);
                    }
                }
            }
        }
    }
    if codes.is_empty() {
        return;
    }
    // Pass 2: salt every atom inside a field with the chain of open codes.
    let mut chain: Vec<usize> = Vec::new();
    for (i, atom) in list.iter_mut().enumerate() {
        let ty = kind(atom.content_element);
        if ty == Some("begin")
            && let Some(c) = code_of[i]
        {
            chain.push(c);
        }
        if !chain.is_empty() {
            let mut salted = String::from("FIELD");
            for &c in &chain {
                salted.push('|');
                salted.push_str(codes[c].trim());
            }
            salted.push('|');
            salted.push_str(&atom.sha1_hash.to_hex_string());
            atom.sha1_hash = AtomHash::of_bytes(salted.as_bytes());
        }
        if ty == Some("end") {
            chain.pop();
        }
    }
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
    salt_field_results(dom, &mut list);
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
    ancestors: &Arc<[NodeId]>,
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
        // Salt over the inner digest's 40-char hex — byte-identical to the former
        // `sha1_hex(format!("PREDEL|{hex}"))`, so a salted atom keeps the same
        // (distinct-from-unsalted) value it always had.
        hash = AtomHash::of_bytes(format!("PREDEL|{}", hash.to_hex_string()).as_bytes());
    }
    // PATH-01: store the shared Arc chain (no per-atom Vec clone).
    let mut atom = ComparisonUnitAtom::new(content, Arc::clone(ancestors), hash);
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

/// Recurses into each named child of `element` except one named `exclude`.
/// Non-allocating (see `Dom::child_at`): `recurse` never adds or removes
/// children of `element`, so the index walk equals `elements(element, None)`
/// without the Vec.
fn recurse_children(
    dom: &mut Dom,
    element: NodeId,
    exclude: Option<&XName>,
    list: &mut Vec<ComparisonUnitAtom>,
    settings: &WmlComparerSettings,
    path: &mut Vec<NodeId>,
) {
    for i in 0..dom.child_count(element) {
        let item = dom.child_at(element, i);
        match dom.name(item) {
            Some(n) if exclude != Some(&n) => recurse(dom, item, list, settings, path),
            _ => {}
        }
    }
}

fn recurse(
    dom: &mut Dom,
    element: NodeId,
    list: &mut Vec<ComparisonUnitAtom>,
    settings: &WmlComparerSettings,
    path: &mut Vec<NodeId>,
) {
    let Some(name) = dom.name(element) else {
        return;
    };

    // Content-root containers: walk children only (do not emit the container
    // itself as an atom). hdr/ftr are the body equivalent for header/footer
    // part compares (PR #81 writeback path).
    //
    // Stop-set for the ancestor *path* matches pre-ATOM-STACK `ancestor_chain`:
    // body / footnotes / endnotes / hdr / ftr are excluded. Individual
    // `w:footnote` / `w:endnote` definitions are NOT stop nodes — they must
    // remain on the path so ProcessFootnoteEndnote → produce can rebuild the
    // note wrapper (parity CRASH regression when path stayed empty here).
    if name == W::body() || name == W::name("hdr") || name == W::name("ftr") {
        // True path-stop containers: path stays empty underneath.
        recurse_children(dom, element, None, list, settings, path);
        return;
    }
    if name == W::footnote() || name == W::endnote() {
        // Walk children only (no atom for the note itself), but push onto path.
        path.push(element);
        recurse_children(dom, element, None, list, settings, path);
        path.pop();
        return;
    }
    // w:footnotes / w:endnotes parts: if ever used as content_parent, mirror
    // the old stop set (exclude the part from descendant paths).
    if name == W::name("footnotes") || name == W::name("endnotes") {
        recurse_children(dom, element, None, list, settings, path);
        return;
    }

    if name == W::p() {
        // children except pPr
        path.push(element);
        recurse_children(dom, element, Some(&W::p_pr()), list, settings, path);
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
        push_atom(dom, content, &chain, list, settings);
        return;
    }

    if name == W::r() {
        // children except rPr
        path.push(element);
        recurse_children(dom, element, Some(&W::r_pr()), list, settings, path);
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
            push_atom(dom, content, &chain, list, settings);
        }
        return;
    }

    // mc:AlternateContent → a single opaque atom (Choice+Fallback kept verbatim).
    if name == MC::name("AlternateContent") {
        let chain = chain_with(path, element);
        push_atom(dom, element, &chain, list, settings);
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
        push_atom(dom, element, &chain, list, settings);
        return;
    }

    // AllowableRunChildren (or w:object) → a single verbatim leaf atom.
    if ALLOWABLE_RUN_CHILDREN.contains(&name) || name == W::object() {
        let chain = chain_with(path, element);
        push_atom(dom, element, &chain, list, settings);
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
        push_atom(dom, element, &chain, list, settings);
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
                    let text: String = gc
                        .iter()
                        .map(|a| dom.value_str(a.content_element))
                        .collect();
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

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod break_payload_identity_regressions {
    use super::atom_hash;
    use crate::comparer::WmlComparerSettings;
    use crate::namespaces::W;
    use crate::xmllinq::Dom;

    #[test]
    fn break_defaults_match_but_authored_kind_and_clearance_do_not() {
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&format!(
            "<w:r xmlns:w=\"{}\"><w:br/><w:br w:type=\"textWrapping\" w:clear=\"none\"/><w:br w:type=\"page\"/><w:br w:type=\"column\"/><w:br w:clear=\"left\"/><w:br w:clear=\"right\"/></w:r>", W::URI
        ));
        let root = dom.root(document).unwrap();
        let breaks = dom.elements(root, Some(&W::name("br")));
        for settings in [
            WmlComparerSettings::default(),
            WmlComparerSettings::powertools_faithful(),
        ] {
            let hashes = breaks
                .iter()
                .map(|&node| atom_hash(&dom, node, &settings))
                .collect::<Vec<_>>();
            assert_eq!(hashes[0], hashes[1]);
            assert_eq!(
                hashes[0],
                super::AtomHash::of_bytes(b"br"),
                "bare/default break identity stays compatible"
            );
            for i in 2..hashes.len() {
                for j in 0..i {
                    assert_ne!(hashes[i], hashes[j]);
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod final_section_table_ownership_tests {
    use super::*;

    fn parse(body: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let document =
            dom.parse_xdocument(&format!("<w:body xmlns:w='{}'>{body}</w:body>", W::URI));
        let root = dom.root(document).unwrap();
        (dom, root)
    }

    #[test]
    fn a_table_only_story_keeps_its_final_section_outside_all_cell_paragraphs() {
        for nested_control in [false, true] {
            let table = "<w:tbl><w:tblPr><w:tblW w:w='3600' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='3600'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='3600'/></w:tcPr><w:p><w:pPr><w:spacing w:after='240'/></w:pPr><w:r><w:t>cell</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl><w:p/></w:tc></w:tr></w:tbl>";
            let blocks = if nested_control {
                format!(
                    "<w:sdt><w:sdtPr><w:id w:val='42'/><w:tag w:val='Clause'/></w:sdtPr><w:sdtContent>{table}</w:sdtContent></w:sdt>"
                )
            } else {
                table.to_string()
            };
            let (mut dom, body) = parse(&format!(
                "{blocks}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/><w:pgMar w:left='1440' w:right='1440'/></w:sectPr>"
            ));
            let before = dom.serialize_element(body);
            move_last_sectpr_into_last_paragraph(&mut dom, body).unwrap();
            assert_eq!(dom.serialize_element(body), before);
            assert_eq!(dom.elements(body, Some(&W::sect_pr())).len(), 1);
            for paragraph in dom.descendants(body, Some(&W::p())) {
                assert!(dom.descendants(paragraph, Some(&W::sect_pr())).is_empty());
            }
        }
    }

    #[test]
    fn block_control_paragraph_still_owns_the_closing_section_instead_of_later_table_cells() {
        let (mut dom, body) = parse(
            "<w:sdt><w:sdtPr><w:id w:val='42'/><w:tag w:val='Clause'/></w:sdtPr><w:sdtContent><w:p><w:pPr><w:spacing w:after='240'/></w:pPr><w:r><w:t>story</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:sdtContent></w:sdt><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>",
        );
        let control = dom.descendants(body, Some(&W::sdt()))[0];
        let props_before = dom.serialize_element(dom.element(control, &W::sdt_pr()).unwrap());
        let paragraphs = dom.descendants(body, Some(&W::p()));
        move_last_sectpr_into_last_paragraph(&mut dom, body).unwrap();
        assert!(dom.elements(body, Some(&W::sect_pr())).is_empty());
        assert_eq!(dom.descendants(paragraphs[0], Some(&W::sect_pr())).len(), 1);
        assert!(
            dom.descendants(paragraphs[1], Some(&W::sect_pr()))
                .is_empty()
        );
        assert_eq!(
            dom.serialize_element(dom.element(control, &W::sdt_pr()).unwrap()),
            props_before
        );
        let spacing = dom.descendants(paragraphs[0], Some(&W::spacing_el()))[0];
        assert_eq!(dom.attribute(spacing, &W::name("after")), Some("240"));
        assert_eq!(dom.value(body), "storycell");
    }
    #[test]
    fn existing_cell_section_properties_and_history_are_never_replaced_by_body_geometry() {
        let (mut dom, body) = parse(
            "<w:tbl><w:tr><w:tc><w:p><w:pPr><w:spacing w:after='240'/><w:sectPr><w:pgSz w:w='10000' w:h='15000'/><w:sectPrChange w:id='7' w:author='Prior editor'><w:sectPr><w:pgSz w:w='9000' w:h='14000'/></w:sectPr></w:sectPrChange></w:sectPr></w:pPr></w:p></w:tc></w:tr></w:tbl><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>",
        );
        let before = dom.serialize_element(body);
        move_last_sectpr_into_last_paragraph(&mut dom, body).unwrap();
        assert_eq!(dom.serialize_element(body), before);
        assert_eq!(dom.elements(body, Some(&W::sect_pr())).len(), 1);
        let cell = dom.descendants(body, Some(&W::tc()))[0];
        assert_eq!(dom.descendants(cell, Some(&W::sect_pr())).len(), 2);
        let history = dom.descendants(cell, Some(&W::name("sectPrChange")))[0];
        assert_eq!(dom.attribute(history, &W::id()), Some("7"));
        assert_eq!(dom.attribute(history, &W::author()), Some("Prior editor"));
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_atomizer_owned_boundaries {
    use super::*;

    fn parse(root: &str, content: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let document = dom.parse_xdocument(&format!(
            "<w:{root} xmlns:w='{}' xmlns:pt='{}'>{content}</w:{root}>",
            W::URI,
            PT::URI
        ));
        let node = dom.root(document).unwrap();
        (dom, node)
    }

    #[test]
    fn note_definition_owns_every_character_and_mark_but_the_part_is_a_path_stop() {
        for (part, note) in [("footnotes", "footnote"), ("endnotes", "endnote")] {
            for individual in [false, true] {
                let paragraph = "<w:p><w:pPr><w:spacing w:after='240'/></w:pPr><w:r><w:rPr><w:i/></w:rPr><w:t> X </w:t><w:tab/></w:r></w:p>";
                let content = if individual {
                    paragraph.to_string()
                } else {
                    format!("<w:{note} w:id='42'>{paragraph}</w:{note}>")
                };
                let (mut dom, root) = parse(if individual { note } else { part }, &content);
                if individual {
                    dom.set_attribute_value(root, &W::id(), Some("42"));
                }
                let owner = if individual {
                    root
                } else {
                    dom.element(root, &W::name(note)).unwrap()
                };
                let p = dom.element(owner, &W::p()).unwrap();
                let r = dom.element(p, &W::r()).unwrap();
                let t = dom.element(r, &W::t()).unwrap();
                let tab = dom.element(r, &W::name("tab")).unwrap();
                let ppr = dom.element(p, &W::p_pr()).unwrap();
                let atoms = create_comparison_unit_atom_list(
                    &mut dom,
                    root,
                    &WmlComparerSettings::default(),
                );
                assert_eq!(atoms.len(), 5);
                assert_eq!(
                    atoms
                        .iter()
                        .map(|a| dom.value(a.content_element))
                        .collect::<Vec<_>>(),
                    [" ", "X", " ", "", ""]
                );
                for atom in &atoms[..3] {
                    assert_eq!(atom.ancestor_elements.as_ref(), [owner, p, r, t]);
                }
                assert_eq!(atoms[3].ancestor_elements.as_ref(), [owner, p, r, tab]);
                assert_eq!(atoms[4].ancestor_elements.as_ref(), [owner, p]);
                assert_eq!(atoms[4].content_element, ppr);
                let rebuilt = coalesce(&mut dom, &atoms);
                let rebuilt_root = dom.root(rebuilt).unwrap();
                let rebuilt_body = dom.element(rebuilt_root, &W::body()).unwrap();
                let rebuilt_note = dom.element(rebuilt_body, &W::name(note)).unwrap();
                assert_eq!(dom.attribute(rebuilt_note, &W::id()), Some("42"));
                let rebuilt_p = dom.element(rebuilt_note, &W::p()).unwrap();
                assert_eq!(
                    dom.serialize_element(dom.element(rebuilt_p, &W::p_pr()).unwrap()),
                    dom.serialize_element(ppr)
                );
                let runs = dom.elements(rebuilt_p, Some(&W::r()));
                assert_eq!(
                    runs.len(),
                    2,
                    "text and tab have separate coalesced run groups"
                );
                let source_run_properties =
                    dom.serialize_element(dom.element(r, &W::r_pr()).unwrap());
                for &rebuilt_run in &runs {
                    assert_eq!(
                        dom.serialize_element(dom.element(rebuilt_run, &W::r_pr()).unwrap()),
                        source_run_properties
                    );
                }
                let rebuilt_r = runs[0];
                let rebuilt_t = dom.element(rebuilt_r, &W::t()).unwrap();
                assert_eq!(dom.value(rebuilt_t), " X ");
                assert_eq!(
                    dom.attribute(rebuilt_t, &XNamespace::xml().name("space")),
                    Some("preserve")
                );
                let payloads = runs
                    .iter()
                    .flat_map(|&run| dom.elements(run, None))
                    .filter(|&node| !dom.name_is(node, &W::r_pr()))
                    .map(|node| (dom.name(node).unwrap(), dom.value(node)))
                    .collect::<Vec<_>>();
                assert_eq!(
                    payloads,
                    [(W::t(), " X ".to_string()), (W::name("tab"), String::new())]
                );
                assert_eq!(dom.elements(runs[1], Some(&W::name("tab"))).len(), 1);
                assert!(dom.elements(runs[0], Some(&W::name("tab"))).is_empty());
                assert!(dom.elements(runs[1], Some(&W::t())).is_empty());
            }
        }
    }

    #[test]
    fn original_deletion_salt_follows_its_owner_and_never_salts_revised_or_inserted_content() {
        let raw = AtomHash::of_bytes(b"tX");
        let salted = AtomHash::of_bytes(format!("PREDEL|{}", raw.to_hex_string()).as_bytes());
        assert_ne!(raw, salted);
        for location in ["content", "run", "paragraph"] {
            for (marker, value, expected) in [
                ("PreDelete", super::super::PREDELETE_STAMP_ORIG, salted),
                ("PreDelete", super::super::PREDELETE_STAMP_REV, raw),
                ("PreIns", super::super::PREDELETE_STAMP_ORIG, raw),
                ("PreDelete", "", raw),
            ] {
                let (mut dom, body) = parse(
                    "body",
                    "<w:p><w:pPr><w:spacing w:after='240'/></w:pPr><w:r><w:t>X</w:t></w:r></w:p>",
                );
                let p = dom.element(body, &W::p()).unwrap();
                let r = dom.element(p, &W::r()).unwrap();
                let t = dom.element(r, &W::t()).unwrap();
                // Production text splitting creates new content leaves; stamped
                // text ownership is carried by its original ancestor leaf.
                let owner = match location {
                    "content" => t,
                    "run" => r,
                    _ => p,
                };
                dom.set_attribute_value(owner, &PT::name(marker), Some(value));
                let atoms = create_comparison_unit_atom_list(
                    &mut dom,
                    body,
                    &WmlComparerSettings::default(),
                );
                assert_eq!(atoms.len(), 2);
                assert_eq!(atoms[0].sha1_hash, expected, "{location} {marker}={value}");
                assert_eq!(atoms[0].ancestor_elements.as_ref(), [p, r, t]);
                assert_eq!(atoms[0].correlation_status, CorrelationStatus::Equal);
                assert_eq!(dom.value(atoms[0].content_element), "X");
                let plain_mark = atom_hash(
                    &dom,
                    atoms[1].content_element,
                    &WmlComparerSettings::default(),
                );
                let expected_mark = if location == "paragraph"
                    && marker == "PreDelete"
                    && value == super::super::PREDELETE_STAMP_ORIG
                {
                    AtomHash::of_bytes(format!("PREDEL|{}", plain_mark.to_hex_string()).as_bytes())
                } else {
                    plain_mark
                };
                assert_eq!(atoms[1].sha1_hash, expected_mark);
            }
        }
    }

    #[test]
    fn tracked_payload_status_and_paragraph_mark_status_have_independent_owners() {
        for (tracking, status) in [
            ("ins", CorrelationStatus::Inserted),
            ("del", CorrelationStatus::Deleted),
            ("moveFrom", CorrelationStatus::MovedSource),
            ("moveTo", CorrelationStatus::MovedDestination),
        ] {
            for pilcrow in [None, Some("ins"), Some("del")] {
                let text_tag = if tracking == "del" || tracking == "moveFrom" {
                    "delText"
                } else {
                    "t"
                };
                let props = pilcrow.map_or_else(String::new, |mark| {
                    format!("<w:rPr><w:{mark} w:id='8' w:author='Mark editor'/></w:rPr>")
                });
                let (mut dom, body) = parse(
                    "body",
                    &format!(
                        "<w:p><w:pPr><w:spacing w:after='240'/>{props}</w:pPr><w:{tracking} w:id='7' w:author='Payload editor'><w:r><w:{text_tag}>X</w:{text_tag}><w:tab/></w:r></w:{tracking}></w:p>"
                    ),
                );
                let p = dom.element(body, &W::p()).unwrap();
                let revision = dom.element(p, &W::name(tracking)).unwrap();
                let ppr = dom.element(p, &W::p_pr()).unwrap();
                assign_to_all_elements(&mut dom, body);
                let snapshot = dom.serialize_element(ppr);
                let atoms = create_comparison_unit_atom_list(
                    &mut dom,
                    body,
                    &WmlComparerSettings::default(),
                );
                assert_eq!(atoms.len(), 3);
                for atom in &atoms[..2] {
                    assert_eq!(atom.rev_track_element, Some(revision));
                    assert_eq!(atom.correlation_status, status);
                    assert_eq!(atom.ancestor_elements[0], p);
                    assert_eq!(atom.ancestor_elements[1], revision);
                }
                let expected = match pilcrow {
                    Some("ins") => CorrelationStatus::Inserted,
                    Some("del") => CorrelationStatus::Deleted,
                    _ => CorrelationStatus::Equal,
                };
                assert_eq!(atoms[2].correlation_status, expected);
                assert_eq!(atoms[2].content_element, ppr);
                assert_eq!(atoms[2].ancestor_elements.as_ref(), [p]);
                assert_eq!(
                    atoms[2].rev_track_element,
                    pilcrow.map(|mark| dom.descendants(ppr, Some(&W::name(mark)))[0])
                );
                assert_eq!(dom.serialize_element(ppr), snapshot);
            }
        }
    }

    #[test]
    fn stamped_hash_is_authoritative_while_fresh_text_normalization_is_option_owned() {
        let (mut dom, root) = parse("r", "<w:t>é x</w:t>");
        let t = dom.element(root, &W::t()).unwrap();
        for case in [false, true] {
            for space in [false, true] {
                let settings = WmlComparerSettings {
                    case_insensitive: case,
                    conflate_breaking_and_nonbreaking_spaces: space,
                    ..WmlComparerSettings::default()
                };
                let text = match (case, space) {
                    (false, false) => "té x",
                    (true, false) => "tÉ X",
                    (false, true) => "té\u{a0}x",
                    (true, true) => "tÉ\u{a0}X",
                };
                assert_eq!(
                    atom_hash(&dom, t, &settings),
                    AtomHash::of_bytes(text.as_bytes())
                );
                let stamped = AtomHash::of_bytes(b"preprocessed relationship identity");
                dom.set_attribute_value(t, &PT::sha1_hash(), Some(&stamped.to_hex_string()));
                assert_eq!(atom_hash(&dom, t, &settings), stamped);
                dom.set_attribute_value(t, &PT::sha1_hash(), None);
                assert_eq!(dom.value(t), "é x");
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_atomizer_field_chain_boundaries {
    use super::*;

    fn field_atoms(code: &str, nested: bool) -> (Dom, Vec<ComparisonUnitAtom>) {
        let mut dom = Dom::new();
        let inner = if nested {
            "<w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText> REF Clause </w:instrText></w:r><w:r><w:fldChar w:fldCharType='separate'/></w:r><w:r><w:t>X</w:t></w:r><w:r><w:fldChar w:fldCharType='end'/></w:r>"
        } else {
            ""
        };
        let doc = dom.parse_xdocument(&format!("<w:body xmlns:w='{}'><w:p><w:r><w:t>A</w:t></w:r><w:r><w:fldChar w:fldCharType='begin'/></w:r><w:r><w:instrText> {code} </w:instrText></w:r>{inner}<w:r><w:fldChar w:fldCharType='end'/></w:r><w:r><w:t>Z</w:t></w:r></w:p></w:body>", W::URI));
        let body = dom.root(doc).unwrap();
        let atoms =
            create_comparison_unit_atom_list(&mut dom, body, &WmlComparerSettings::default());
        (dom, atoms)
    }

    #[test]
    fn fields_without_cached_results_still_own_their_complete_nested_instruction_chain() {
        for nested in [false, true] {
            let (dom, atoms) = field_atoms("IF", nested);
            let (other, changed) = field_atoms("QUOTE", nested);
            assert_eq!(atoms.len(), if nested { 11 } else { 6 });
            assert_eq!(atoms.len(), changed.len());
            let end = atoms.len() - 3;
            let base_settings = WmlComparerSettings::default();
            for (i, atom) in atoms.iter().enumerate() {
                let base = atom_hash(&dom, atom.content_element, &base_settings);
                let expected = if i >= 1 && i <= end {
                    let codes = if nested && (3..=7).contains(&i) {
                        "IF  REF Clause|REF Clause"
                    } else if nested {
                        "IF  REF Clause"
                    } else {
                        "IF"
                    };
                    AtomHash::of_bytes(format!("FIELD|{codes}|{}", base.to_hex_string()).as_bytes())
                } else {
                    base
                };
                assert_eq!(atom.sha1_hash, expected, "nested={nested}, atom={i}");
                assert_eq!(
                    dom.name(atom.content_element),
                    other.name(changed[i].content_element)
                );
                if (1..=end).contains(&i) {
                    assert_ne!(atom.sha1_hash, changed[i].sha1_hash);
                } else {
                    assert_eq!(atom.sha1_hash, changed[i].sha1_hash);
                }
                assert_eq!(atom.correlation_status, CorrelationStatus::Equal);
            }
            assert_eq!(dom.value(atoms[0].content_element), "A");
            assert_eq!(dom.value(atoms[atoms.len() - 2].content_element), "Z");
            if nested {
                assert_eq!(dom.value(atoms[6].content_element), "X");
            }
        }
    }

    #[test]
    fn header_and_footer_roots_stop_paths_and_preserve_each_whitespace_boundary() {
        for root in ["hdr", "ftr"] {
            for text in ["X", " X", "X ", " X ", "X Y", "\u{a0}X\u{a0}"] {
                for deleted in [false, true] {
                    let tag = if deleted { "delText" } else { "t" };
                    let revision_start = if deleted {
                        "<w:del w:id='7' w:author='Prior editor'>"
                    } else {
                        ""
                    };
                    let revision_end = if deleted { "</w:del>" } else { "" };
                    let mut dom = Dom::new();
                    let doc = dom.parse_xdocument(&format!("<w:{root} xmlns:w='{}'><w:p><w:pPr><w:spacing w:after='240'/></w:pPr>{revision_start}<w:r><w:rPr><w:b/></w:rPr><w:{tag} xml:space='preserve'>{text}</w:{tag}><w:br w:type='page'/></w:r>{revision_end}</w:p></w:{root}>",W::URI));
                    let story = dom.root(doc).unwrap();
                    let atoms = create_comparison_unit_atom_list(
                        &mut dom,
                        story,
                        &WmlComparerSettings::default(),
                    );
                    assert_eq!(atoms.len(), text.chars().count() + 2);
                    assert!(
                        atoms
                            .iter()
                            .all(|a| dom.name_is(a.ancestor_elements[0], &W::p()))
                    );
                    assert!(atoms.iter().all(|a| !a.ancestor_elements.contains(&story)));
                    let rebuilt = coalesce(&mut dom, &atoms);
                    let rebuilt_root = dom.root(rebuilt).unwrap();
                    let body = dom.element(rebuilt_root, &W::body()).unwrap();
                    assert_eq!(dom.elements(body, Some(&W::p())).len(), 1);
                    let emitted = dom.descendants(body, Some(&W::name(tag)));
                    assert_eq!(emitted.len(), 1);
                    assert_eq!(dom.value(emitted[0]), text);
                    assert_eq!(
                        dom.attribute(emitted[0], &XNamespace::xml().name("space")),
                        if text.starts_with(' ') || text.ends_with(' ') {
                            Some("preserve")
                        } else {
                            None
                        }
                    );
                    let breaks = dom.descendants(body, Some(&W::name("br")));
                    assert_eq!(breaks.len(), 1);
                    assert_eq!(dom.attribute(breaks[0], &W::name("type")), Some("page"));
                    let runs = dom.descendants(body, Some(&W::r()));
                    assert_eq!(runs.len(), 2);
                    for &run in &runs {
                        let rpr = dom.element(run, &W::r_pr()).unwrap();
                        assert_eq!(dom.elements(rpr, None).len(), 1);
                        assert!(dom.element(rpr, &W::name("b")).is_some());
                    }
                    let payloads = runs
                        .iter()
                        .flat_map(|&run| dom.elements(run, None))
                        .filter(|&node| !dom.name_is(node, &W::r_pr()))
                        .map(|node| (dom.name(node).unwrap(), dom.value(node)))
                        .collect::<Vec<_>>();
                    assert_eq!(
                        payloads,
                        [
                            (W::name(tag), text.to_string()),
                            (W::name("br"), String::new())
                        ]
                    );
                    let marks = dom.descendants(body, Some(&W::del()));
                    assert_eq!(marks.len(), 2 * usize::from(deleted));
                    for &run in &runs {
                        let owners = dom.ancestors(run, Some(&W::del()));
                        assert_eq!(owners.len(), usize::from(deleted));
                        if deleted {
                            assert_eq!(dom.attribute(owners[0], &W::id()), Some("7"));
                            assert_eq!(
                                dom.attribute(owners[0], &W::author()),
                                Some("Prior editor")
                            );
                            assert_eq!(dom.elements(owners[0], Some(&W::r())), [run]);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod coverage_atomizer_leaf_stamp_boundaries {
    use super::*;

    #[test]
    fn opaque_leaf_stamps_salt_original_deletions_once_and_preserve_the_authored_payload() {
        for (leaf, payload) in [
            ("tab", "<w:tab/>"),
            ("br", "<w:br w:type='page' w:clear='all'/>"),
            ("fldSimple", "<w:fldSimple w:instr='DATE'/>"),
        ] {
            for stamp in [
                None,
                Some(super::super::PREDELETE_STAMP_ORIG),
                Some(super::super::PREDELETE_STAMP_REV),
            ] {
                let mut dom = Dom::new();
                let doc = dom.parse_xdocument(&format!(
                    "<w:body xmlns:w='{}'><w:p><w:r>{payload}</w:r></w:p></w:body>",
                    W::URI
                ));
                let body = dom.root(doc).unwrap();
                let node = dom.descendants(body, Some(&W::name(leaf)))[0];
                if let Some(stamp) = stamp {
                    dom.set_attribute_value(node, &PT::name("PreDelete"), Some(stamp));
                }
                let settings = WmlComparerSettings::default();
                let raw = atom_hash(&dom, node, &settings);
                let expected = if stamp == Some(super::super::PREDELETE_STAMP_ORIG) {
                    AtomHash::of_bytes(format!("PREDEL|{}", raw.to_hex_string()).as_bytes())
                } else {
                    raw
                };
                let atoms = create_comparison_unit_atom_list(&mut dom, body, &settings);
                let snapshot = dom.serialize_element(node);
                assert_eq!(atoms.len(), 2);
                assert_eq!(atoms[0].content_element, node);
                assert_eq!(atoms[0].sha1_hash, expected);
                assert_eq!(atoms[0].ancestor_elements.last(), Some(&node));
                assert_eq!(atoms[0].correlation_status, CorrelationStatus::Equal);
                assert_eq!(dom.serialize_element(node), snapshot);
                assert_eq!(atoms[1].sha1_hash, AtomHash::of_bytes(b"pPr"));
            }
        }
    }
}
