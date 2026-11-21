// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Header, footer and note stories: section pairing, adoption and marking.

use super::*;

pub(super) fn story_diff_error(part: &str, reason: &'static str) -> OpcError {
    crate::opc::refused(StoryDiffError {
        part: part.to_string(),
        reason,
    })
}

pub(super) fn ensure_story_text_tracked(
    part: &str,
    original: &str,
    revised: &str,
    redlined: &str,
    has_revisions: bool,
) -> Result<(), OpcError> {
    if !has_revisions && (original != revised || redlined != original) {
        return Err(story_diff_error(
            part,
            "story text changed without revision marks",
        ));
    }
    Ok(())
}

/// An unresolved reference has no meaning in its source story. Neutralize
/// each side against its OWN rels before comparing, even when the other side
/// happens to resolve the same spelling. Keep valid references for the carry
/// step, which must fail explicitly if their type cannot be carried.
pub(super) fn neutralize_story_relationships(
    pkg: &PartFs,
    part: &str,
    xml: &str,
) -> Result<String, OpcError> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom
        .root(doc)
        .ok_or_else(|| story_diff_error(part, "story has no root element"))?;
    let mut changed = false;
    for el in dom.descendants_and_self(root, None) {
        for (name, rid) in dom.attributes(el) {
            if !crate::comparer::tables::S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(&name) {
                continue;
            }
            let resolves = pkg.read_rels_for(part).is_some_and(|rels| {
                rels.items.iter().any(|r| {
                    r.id == rid
                        && (r.target_mode.as_deref() == Some("External")
                            || crate::comparer::parts::is_external_relationship(
                                &r.rel_type,
                                &r.target,
                            )
                            || pkg
                                .part_bytes(&pkg.resolve_rel_target(part, &r.target))
                                .is_some())
                })
            });
            if !resolves {
                dom.set_attribute_value(el, &name, None);
                changed = true;
            }
        }
    }
    Ok(if changed {
        dom.serialize_document(doc)
    } else {
        xml.to_string()
    })
}

/// `xml` (part `part_b` of `pkg2`) with each relationship it references
/// carried onto `part_a` of `out` under a fresh id (an image with A's bytes
/// reuses A's relationship), so B's content means what it meant in B once
/// it is diffed into A's part. None when a relationship cannot be carried.
pub(super) fn carry_revised_part_relationships(
    out: &mut PartFs,
    part_a: &str,
    (pkg2, part_b): (&PartFs, &str),
    xml: &str,
) -> Option<String> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc)?;
    let mut minted: std::collections::HashMap<(String, Option<&str>), String> =
        std::collections::HashMap::new();
    for el in dom.descendants_and_self(root, None) {
        for (name, rid) in dom.attributes(el) {
            if !crate::comparer::tables::S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(&name) {
                continue;
            }
            let suffix = dom
                .name(el)
                .and_then(|n| crate::comparer::parts::required_rel_type_suffix(n.local_name()));
            let key = (rid.clone(), suffix);
            let id = match minted.get(&key) {
                Some(id) => id.clone(),
                None => {
                    let id = crate::comparer::parts::carry_relationship(
                        out,
                        part_a,
                        pkg2,
                        part_b,
                        &rid,
                        |ty| suffix.is_none_or(|s| ty.ends_with(s)),
                    )?;
                    minted.insert(key, id.clone());
                    id
                }
            };
            dom.set_attribute_value(el, &name, Some(&id));
        }
    }
    Some(dom.serialize_document(doc))
}

/// Every relationship `xml` (part `part_b` of `pkg2`) references resolves in
/// `part_a`'s rels to the same type, mode and target, and an internal target
/// to the same bytes. A redlined part written over `part_a` keeps A's rels,
/// so B's references then still mean what they meant in B.
pub(super) fn part_rels_agree(
    (pkg1, part_a): (&PartFs, &str),
    (pkg2, part_b): (&PartFs, &str),
    xml: &str,
) -> bool {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(xml);
    let Some(root) = dom.root(document) else {
        return false;
    };
    let ids: std::collections::BTreeSet<String> = dom
        .descendants_and_self(root, None)
        .into_iter()
        .flat_map(|e| dom.attributes(e))
        .filter(|(name, _)| crate::comparer::tables::S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(name))
        .map(|(_, value)| value)
        .collect();
    if ids.is_empty() {
        return true;
    }
    let (Some(rels_a), Some(rels_b)) = (pkg1.read_rels_for(part_a), pkg2.read_rels_for(part_b))
    else {
        return false;
    };
    ids.iter().all(|id| {
        let find =
            |rels: &crate::opc::Relationships| rels.items.iter().find(|r| &r.id == id).cloned();
        let (Some(a), Some(b)) = (find(rels_a), find(rels_b)) else {
            return false;
        };
        if a.rel_type != b.rel_type || a.target_mode != b.target_mode {
            return false;
        }
        if a.target_mode.as_deref() == Some("External") {
            return a.target == b.target;
        }
        let bytes_a = pkg1.part_bytes(&pkg1.resolve_rel_target(part_a, &a.target));
        let bytes_b = pkg2.part_bytes(&pkg2.resolve_rel_target(part_b, &b.target));
        bytes_a.is_some() && bytes_a == bytes_b
    })
}

/// The header/footer parts a document references, as (kind, type, part-name):
/// kind ∈ {"header","footer"}, type ∈ {"default","even","first"}. Read from the
/// `headerReference`/`footerReference` elements in the main document, resolved to
/// part names via the document rels.
pub(super) fn header_footer_refs(pkg: &PartFs) -> Vec<(String, String, String)> {
    header_footer_refs_by_section(pkg)
        .1
        .into_iter()
        .map(|(_, kind, ty, part)| (kind, ty, part))
        .collect()
}

/// A header/footer reference with the index of the section holding it.
type SectionRef = (usize, String, String, String);

/// A document's section count and [`header_footer_refs`], each tagged
/// with the index of the section that references it.
pub(super) fn header_footer_refs_by_section(pkg: &PartFs) -> (usize, Vec<SectionRef>) {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(xml) = pkg.part_string(&main) else {
        return (0, Vec::new());
    };
    let Some(rels) = pkg.read_rels_for(&main) else {
        return (0, Vec::new());
    };
    let id_to_target: std::collections::HashMap<&str, &str> = rels
        .items
        .iter()
        .map(|r| (r.id.as_str(), r.target.as_str()))
        .collect();
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let Some(root) = d.root(doc) else {
        return (0, Vec::new());
    };
    let sections = section_props(&d, root);
    let mut out = Vec::new();
    for (ref_name, kind) in [
        (W::header_reference(), "header"),
        (W::footer_reference(), "footer"),
    ] {
        for r in d.descendants(root, Some(&ref_name)) {
            let ty = d.attribute(r, &W::type_()).unwrap_or("default").to_string();
            let section = d
                .parent(r)
                .and_then(|s| sections.iter().position(|&x| x == s))
                .unwrap_or(0);
            if let Some(rid) = d.attribute(r, &R::id())
                && let Some(&tgt) = id_to_target.get(rid)
            {
                out.push((
                    section,
                    kind.to_string(),
                    ty,
                    referenced_part(pkg, &main, tgt),
                ));
            }
        }
    }
    (sections.len(), out)
}

/// The part a relationship target of `main` names. Prefer the actual OPC
/// target. Historical generators also wrote package-root Word paths as
/// relative targets; keep that fallback only when the resolved part is
/// absent.
pub(super) fn referenced_part(pkg: &PartFs, main: &str, target: &str) -> String {
    let resolved = pkg.resolve_rel_target(main, target);
    if pkg.part_bytes(&resolved).is_some() {
        resolved
    } else if target.starts_with("word/") && pkg.part_bytes(target).is_some() {
        target.to_string()
    } else {
        resolved
    }
}

/// The header/footer parts that only deleted section breaks of the redline
/// `root` reference: no live section references them (see
/// [`section_story_parts`]). Nothing shows such a part once the breaks are
/// accepted.
pub(super) fn parts_only_deleted_breaks_reference(
    dom: &Dom,
    root: NodeId,
    pkg: &PartFs,
    main: &str,
) -> std::collections::HashSet<String> {
    let sections = section_story_parts(dom, root, pkg, main);
    let live: std::collections::HashSet<&String> = sections
        .iter()
        .filter(|(deleted, _)| !deleted)
        .flat_map(|(_, parts)| parts)
        .collect();
    sections
        .iter()
        .filter(|(deleted, _)| *deleted)
        .flat_map(|(_, parts)| parts)
        .filter(|part| !live.contains(part))
        .cloned()
        .collect()
}

/// Unmerged comparisons retain A's section stories. This cleanup deliberately
/// follows the flag, not a preset: faithful mode, `--no-paragraph-merge`, and
/// custom settings all need the same Accept All semantics and Word-crash fix.
pub(super) fn needs_unmerged_section_story_cleanup(settings: &WmlComparerSettings) -> bool {
    !settings.merge_replaced_paragraphs
}

/// A story slot includes both its kind and its default/first/even type.
fn has_header_footer_slot(refs: &[(String, String, String)], kind: &str, ty: &str) -> bool {
    refs.iter().any(|(k, t, _)| k == kind && t == ty)
}

/// Whole-story deletion must not touch a part shared with any slot B keeps.
pub(super) fn a_only_header_footer_parts(
    original: &PartFs,
    revised: &PartFs,
) -> std::collections::HashSet<String> {
    let a_refs = header_footer_refs(original);
    let b_refs = header_footer_refs(revised);
    let shared: std::collections::HashSet<&String> = a_refs
        .iter()
        .filter(|(kind, ty, _)| has_header_footer_slot(&b_refs, kind, ty))
        .map(|(_, _, part)| part)
        .collect();
    a_refs
        .iter()
        .filter(|(_, _, part)| !shared.contains(part))
        .map(|(_, _, part)| part.clone())
        .collect()
}

/// PowerTools drops the original's section breaks altogether. We preserve
/// them for Reject All, but must not promote their A-only stories onto the
/// live final section (#1837). Leave shared slots and live earlier sections
/// alone; their treatment remains the faithful preset's existing behavior.
pub(super) fn strip_final_a_only_deleted_break_stories(
    dom: &mut Dom,
    root: NodeId,
    out: &PartFs,
    revised: &PartFs,
    main: &str,
) {
    let Some(final_sect) = dom
        .element(root, &W::body())
        .and_then(|body| dom.element(body, &W::sect_pr()))
    else {
        return;
    };
    let mut sections = section_story_parts(dom, root, out, main);
    sections.pop(); // The direct body child is the final section.
    let live: std::collections::HashSet<&String> = sections
        .iter()
        .filter(|(deleted, _)| !deleted)
        .flat_map(|(_, parts)| parts)
        .collect();
    let dead: std::collections::HashSet<&String> = sections
        .iter()
        .filter(|(deleted, _)| *deleted)
        .flat_map(|(_, parts)| parts)
        .filter(|part| !live.contains(part))
        .collect();
    let b_refs = header_footer_refs(revised);
    let Some(rels) = out.read_rels_for(main) else {
        return;
    };
    for reference in dom.elements(final_sect, None) {
        let kind = if dom.name_is(reference, &W::header_reference()) {
            "header"
        } else if dom.name_is(reference, &W::footer_reference()) {
            "footer"
        } else {
            continue;
        };
        let ty = dom.attribute(reference, &W::type_()).unwrap_or("default");
        if has_header_footer_slot(&b_refs, kind, ty) {
            continue;
        }
        let target = dom
            .attribute(reference, &R::id())
            .and_then(|id| rels.items.iter().find(|r| r.id == id).map(|r| &r.target));
        if target.is_some_and(|t| dead.contains(&referenced_part(out, main, t))) {
            // Reject cannot restore this reference; an earlier live section may supply a different story.
            dom.remove(reference);
        }
    }
}

/// A header or footer that holds a page number leaves the
/// deleted section breaks of the redline `root` for its final section.
/// For each (kind, `w:type`) slot the body's own `w:sectPr` does not set,
/// the story it inherits from the nearest earlier section moves when that
/// section is a deleted break, no live section references the story, and
/// the story's closing paragraph holds a PAGE or NUMPAGES field. The final
/// section then references it, and no deleted break does. Two more
/// conditions: the redline deletes a table row, and no section before the
/// story's first break sets the slot.
///
/// A deleted break that references such a footer crashed Word 16.115
/// whatever the story's marks (s176, s472, #1833), Word's own redline
/// included, and Word opened every one once the reference moved. Word logs
/// the crash in its table cache, at the end of a deleted row. Accepted,
/// nothing shows the story, as in Word's Accept All; rejected, the breaks
/// that lost the reference show no footer, since a `sectPrChange` cannot
/// record references. Where an earlier section sets the slot, they would
/// show that section's footer instead, so the reference stays. Stories
/// that stay keep their breaks, and `jubarte validate` reports them
/// (`DELETED_SECTION_PAGE_FIELD`).
pub(super) fn move_page_story_to_final_section(
    dom: &mut Dom,
    root: NodeId,
    pkg: &PartFs,
    main: &str,
    eligible: &std::collections::HashSet<String>,
) {
    let Some(rels) = pkg.read_rels_for(main) else {
        return;
    };
    let targets: std::collections::HashMap<&str, &str> = rels
        .items
        .iter()
        .map(|r| (r.id.as_str(), r.target.as_str()))
        .collect();
    let sections = section_props(dom, root);
    let Some((&last, earlier)) = sections.split_last() else {
        return;
    };
    if !dom.parent(last).is_some_and(|p| dom.name_is(p, &W::body())) {
        return;
    }
    // Word's crash is in its table cache, at the end of a deleted row: no
    // deleted row, no move (0 of the 10 such corpus redlines crashed).
    let deletes_a_row = dom
        .descendants(root, Some(&W::tr_pr()))
        .into_iter()
        .any(|pr| dom.element(pr, &W::del()).is_some());
    if !deletes_a_row {
        return;
    }
    let deleted = |dom: &Dom, sect: NodeId| {
        dom.parent(sect)
            .filter(|&p| dom.name_is(p, &W::p_pr()))
            .and_then(|p| dom.element(p, &W::r_pr()))
            .is_some_and(|r| dom.element(r, &W::del()).is_some())
    };
    let (header_reference, footer_reference) = (W::header_reference(), W::footer_reference());
    let (ty_attr, rid) = (W::type_(), R::id());
    // Each header/footer reference of a section: (its slot, its part).
    let references = |dom: &Dom, sect: NodeId| -> Vec<(NodeId, (bool, String), String)> {
        dom.elements(sect, None)
            .into_iter()
            .filter_map(|r| {
                let header = dom.name_is(r, &header_reference);
                if !header && !dom.name_is(r, &footer_reference) {
                    return None;
                }
                let ty = dom.attribute(r, &ty_attr).unwrap_or("default");
                let target = targets.get(dom.attribute(r, &rid)?)?;
                Some((
                    r,
                    (header, ty.to_string()),
                    referenced_part(pkg, main, target),
                ))
            })
            .collect()
    };
    let live: std::collections::HashSet<String> = sections
        .iter()
        .filter(|&&s| !deleted(dom, s))
        .flat_map(|&s| references(dom, s))
        .map(|(_, _, part)| part)
        .collect();
    let set: std::collections::HashSet<(bool, String)> = references(dom, last)
        .into_iter()
        .map(|(_, slot, _)| slot)
        .collect();
    let mut inherited = std::collections::BTreeMap::new();
    for &sect in earlier {
        for (r, slot, part) in references(dom, sect) {
            inherited.insert(slot, (sect, r, part));
        }
    }
    let earlier_references: Vec<_> = earlier.iter().map(|&s| references(dom, s)).collect();
    let title_page = |dom: &Dom, sect: NodeId| {
        dom.element(sect, &W::title_pg())
            .is_some_and(|t| !on_off_is_off(dom, t))
    };
    let moving: Vec<_> = inherited
        .into_iter()
        .filter(|(slot, (sect, _, part))| {
            // The section that sets the slot last; every earlier section
            // from it on shows its story.
            let at = earlier.iter().position(|s| s == sect).unwrap_or(0);
            // Rejected, the breaks that lose the reference show the slot of
            // the section before them: another section's story when one
            // sets it (the trusts' exhibit labels), even between two that
            // set this one. Only an unset slot, a blank, moves.
            let shadowed = earlier_references[..at]
                .iter()
                .flatten()
                .any(|(_, s, p)| s == slot && p != part);
            // A first-page story shows only in a section with `w:titlePg`:
            // moved to a final section without it, the first pages of the
            // sections that show it would lose it and no page would show it.
            let first_unshown = slot.1 == "first"
                && earlier[at..].iter().any(|&s| title_page(dom, s))
                && !title_page(dom, last);
            !(!eligible.contains(part)
                || set.contains(slot)
                || shadowed
                || first_unshown
                || !deleted(dom, *sect)
                || live.contains(part)
                || !story_closes_with_page_field(pkg, part))
        })
        .collect();
    // A part leaves the deleted breaks whole or not at all: a reference to
    // it in a slot that stays (shared by two slots, or handed down in a
    // slot a later section overrides) keeps the part where it is.
    let approved: std::collections::HashSet<(&(bool, String), &str)> = moving
        .iter()
        .map(|(slot, (_, _, part))| (slot, part.as_str()))
        .collect();
    let mut cleared = std::collections::HashSet::new();
    for (_, (_, reference, part)) in &moving {
        let whole = earlier_references
            .iter()
            .flatten()
            .filter(|(_, _, p)| p == part)
            .all(|(_, s, p)| approved.contains(&(s, p.as_str())));
        if !whole {
            continue;
        }
        // Once per part: a second slot of the part must not take back the
        // reference the first one moved.
        if cleared.insert(part.as_str()) {
            for (r, _, other) in earlier_references.iter().flatten() {
                if other == part {
                    dom.remove(*r);
                }
            }
        }
        // `EG_HdrFtrReferences` is a choice: header and footer references
        // come in any order, ahead of the section's other properties.
        dom.add_first(last, *reference);
    }
}

/// Whether the header or footer `part` of `pkg` ends in a paragraph that
/// holds a PAGE or NUMPAGES field (see `validate::holds_page_field`).
fn story_closes_with_page_field(pkg: &PartFs, part: &str) -> bool {
    let Some(xml) = pkg.part_string(part) else {
        return false;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    dom.root(doc)
        .and_then(|story| crate::validate_dom::closing_paragraph(&dom, story))
        .is_some_and(|p| crate::validate_dom::holds_page_field(&dom, p))
}

/// A document's sections in order: each paragraph mark's `w:sectPr`, then
/// the body's own. The `w:sectPr` in a `w:sectPrChange` is no section.
pub(super) fn section_props(dom: &Dom, root: NodeId) -> Vec<NodeId> {
    dom.descendants(root, Some(&W::sect_pr()))
        .into_iter()
        .filter(|&s| {
            dom.parent(s)
                .is_some_and(|p| dom.name_is(p, &W::p_pr()) || dom.name_is(p, &W::body()))
        })
        .collect()
}

/// The revised section each original section became, read off the
/// redline's section breaks: the section of a deleted break has none (Word
/// leaves its headers as they were, probes hs1/hs3), an inserted break's is
/// the revision's alone. None when the breaks do not account for both
/// documents' sections.
pub(super) fn section_pairing(
    redline_xml: &str,
    sections_a: usize,
    sections_b: usize,
) -> Option<Vec<Option<usize>>> {
    let mut d = Dom::new();
    let doc = d.parse_xdocument(redline_xml);
    let root = d.root(doc)?;
    let marked = |s: NodeId, mark: &XName| {
        d.parent(s)
            .filter(|&p| d.name_is(p, &W::p_pr()))
            .and_then(|p| d.element(p, &W::r_pr()))
            .is_some_and(|r| d.element(r, mark).is_some())
    };
    let mut pairing = Vec::new();
    let mut at_b = 0;
    for s in section_props(&d, root) {
        let deleted = marked(s, &W::del());
        if !marked(s, &W::ins()) {
            pairing.push((!deleted).then_some(at_b));
        }
        if !deleted {
            at_b += 1;
        }
    }
    (pairing.len() == sections_a && at_b == sections_b).then_some(pairing)
}

/// The revised part a header/footer part is diffed against, or none when
/// A's part is unchanged in B. With the redline's `section` pairing, it is
/// the one the paired revised section shows for the same kind and type
/// (its own or the one it inherits); a part no surviving section shows has
/// none. Without it, the `at`-th such reference in each
/// document. Keyed by kind and type alone, every section's default footer
/// met the last section's ("Page 1 of 4" against "Page 4 of 4"), and an
/// unchanged footer came out deleted and inserted again.
pub(super) fn pair_header_footer(
    pkg1: &PartFs,
    pkg2: &PartFs,
    refs_b: &[SectionRef],
    (kind, ty, part_a): (&str, &str, &str),
    at: usize,
    section: Option<Option<usize>>,
) -> Option<String> {
    let same_slot: Vec<(usize, &str)> = refs_b
        .iter()
        .filter(|(_, k, t, _)| k == kind && t == ty)
        .map(|(s, _, _, p)| (*s, p.as_str()))
        .collect();
    let bytes_a = pkg1.part_bytes(part_a);
    if same_slot
        .iter()
        .any(|&(_, p)| pkg2.part_bytes(p) == bytes_a)
    {
        return None;
    }
    let paired = match section {
        Some(Some(b)) => same_slot.iter().rev().find(|&&(s, _)| s <= b),
        Some(None) => None,
        None => same_slot.get(at).or(same_slot.last()),
    };
    paired.map(|&(_, p)| p.to_string())
}

/// Word Compare leaves the body-level final `sectPr` without
/// headerReference/footerReference when an earlier mid-body section break
/// already defines the same (kind, type) slot — later sections inherit.
/// Evidence (docx_lots_of_comments_*, verdana×strict01, word_clean_strict01×…):
/// Word's redline has HF only on mid `pPr/sectPr`; the body final is empty.
/// Our pipeline sometimes leaves A's (or adopted) refs on the final as well,
/// which dual-binds chrome and diverges from Word. Strip only the **body
/// direct-child** final; mid multi-section even/default/first copies stay.
/// A slot the revision's own final `sectPr` sets (`revised_final`) stays:
/// Word's redline sections are the revision's (f8c1ce3e92 keeps the final
/// section's first-page header and footer after a title-page section).
pub(super) fn strip_final_sectpr_inherited_header_footer(
    dom: &mut Dom,
    result_root: NodeId,
    revised_final: &std::collections::HashSet<(bool, String)>,
) {
    let href = W::header_reference();
    let fref = W::footer_reference();
    let type_name = W::type_();
    let Some(body) = dom.element(result_root, &W::body()) else {
        return;
    };
    // Body-level final sectPr is a direct child of w:body (not pPr/sectPr).
    let Some(final_sect) = dom.element(body, &W::sect_pr()) else {
        return;
    };
    let mut earlier_slots: std::collections::HashSet<(bool, String)> =
        std::collections::HashSet::new();
    for sect in dom.descendants(body, Some(&W::sect_pr())) {
        if sect == final_sect {
            continue;
        }
        for e in dom.elements(sect, None) {
            let Some(n) = dom.name(e) else { continue };
            let is_header = if n == href {
                true
            } else if n == fref {
                false
            } else {
                continue;
            };
            let ty = dom
                .attribute(e, &type_name)
                .unwrap_or("default")
                .to_string();
            earlier_slots.insert((is_header, ty));
        }
    }
    if earlier_slots.is_empty() {
        return;
    }
    let mut to_remove: Vec<NodeId> = Vec::new();
    for e in dom.elements(final_sect, None) {
        let Some(n) = dom.name(e) else { continue };
        let is_header = if n == href {
            true
        } else if n == fref {
            false
        } else {
            continue;
        };
        let ty = dom
            .attribute(e, &type_name)
            .unwrap_or("default")
            .to_string();
        let slot = (is_header, ty);
        if earlier_slots.contains(&slot) && !revised_final.contains(&slot) {
            to_remove.push(e);
        }
    }
    for n in to_remove {
        dom.remove(n);
    }
}

/// The (is header, `w:type`) header/footer slots the body-level final
/// `sectPr` of `pkg`'s main document sets explicitly.
pub(super) fn final_header_footer_slots(pkg: &PartFs) -> std::collections::HashSet<(bool, String)> {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(xml) = pkg.part_string(&main) else {
        return Default::default();
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(sect) = dom
        .root(doc)
        .and_then(|r| dom.element(r, &W::body()))
        .and_then(|b| dom.element(b, &W::sect_pr()))
    else {
        return Default::default();
    };
    dom.elements(sect, None)
        .into_iter()
        .filter_map(|e| {
            let n = dom.name(e)?;
            let is_header = if n == W::header_reference() {
                true
            } else if n == W::footer_reference() {
                false
            } else {
                return None;
            };
            let ty = dom.attribute(e, &W::type_()).unwrap_or("default");
            Some((is_header, ty.to_string()))
        })
        .collect()
}

/// Word-alignment mode (settings-gated): Word's Compare presents the REVISED
/// document's headers/footers as a UNION per (kind, `w:type`) slot. A's
/// existing refs/parts stay untouched; for each (header|footer,
/// even|default|first) reference in doc B's effective final sectPr that is
/// ABSENT from the output's final sectPr, copy doc B's part (+ its rels and
/// internal targets) into the output and reference it (evidence:
/// comments_complex-style-attr — the header exists only in doc B yet renders
/// in Word's redline; page-numbering-examples vs potpourritest — Word's
/// redline carries all six slots, A's footer diffed + B's five other parts).
/// "Absent" is judged with OOXML inheritance: a slot is only filled by B when
/// no sectPr in the output body (walking every sectPr in document order, not
/// just the final one) carries that (kind, type) ref — modeling the nearest
/// preceding section's inherited refs (sd-2517: B's blank footer otherwise
/// shadowed A's 19 inherited 3-line footers).
pub(super) fn adopt_revised_header_footer(
    dom: &mut Dom,
    result_root: NodeId,
    pkg2: &PartFs,
    out: &mut PartFs,
    out_main: &str,
    settings: &WmlComparerSettings,
) {
    let href = W::header_reference();
    let fref = W::footer_reference();
    let Some(body) = dom.element(result_root, &W::body()) else {
        return;
    };
    let Some(out_sect) = dom.element(body, &W::sect_pr()) else {
        return;
    };
    // Collect (kind, w:type) refs present on ANY body sectPr (mid-breaks + final).
    let body_slots = |dom: &Dom, body: NodeId| -> std::collections::HashSet<(bool, String)> {
        dom.descendants(body, Some(&W::sect_pr()))
            .into_iter()
            .flat_map(|sect| dom.elements(sect, None))
            .filter_map(|e| {
                let n = dom.name(e)?;
                let is_header = if n == href {
                    true
                } else if n == fref {
                    false
                } else {
                    return None;
                };
                let ty = dom
                    .attribute(e, &W::type_())
                    .unwrap_or("default")
                    .to_string();
                Some((is_header, ty))
            })
            .collect()
    };
    // Slots the final sectPr already carries explicitly.
    let final_slots: std::collections::HashSet<(bool, String)> = dom
        .elements(out_sect, None)
        .into_iter()
        .filter_map(|e| {
            let n = dom.name(e)?;
            let is_header = if n == href {
                true
            } else if n == fref {
                false
            } else {
                return None;
            };
            let ty = dom
                .attribute(e, &W::type_())
                .unwrap_or("default")
                .to_string();
            Some((is_header, ty))
        })
        .collect();
    // Whole-body occupancy (A-sourced mid-section footers, etc.).
    let body_occupied = body_slots(dom, body);
    // M66: when the FINAL sectPr lacks a (kind,type) that B's final carries,
    // still adopt B's part onto the final sect — but only if that slot never
    // came from A anywhere in the body. Mid-body footers from *inserted B
    // sections* (file_21: 19 mid footers, empty final) must not block B's
    // last-section footer20; A's genuine mid footers (sd-2517) still block
    // B blank from blanking the final slot.
    //
    // `a_ever` ≈ body slots that are not solely B-insert artifacts is hard to
    // recover after merge; practical rule used by Word evidence on file_21:
    // adopt B final ref when final_slots lacks it AND (body has no such slot
    // OR the package still lacks the B part under any name). The second
    // disjunct is applied below per-ref after we resolve B's target.

    let main2 = pkg2
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(x2) = pkg2.part_string(&main2) else {
        return;
    };
    let mut d2 = Dom::new();
    let doc2 = d2.parse_xdocument(&x2);
    let Some(r2) = d2.root(doc2) else {
        return;
    };
    let Some(b2) = d2.element(r2, &W::body()) else {
        return;
    };
    let sect2 = d2
        .element(b2, &W::sect_pr())
        .or_else(|| d2.descendants(b2, Some(&W::sect_pr())).last().copied());
    let Some(sect2) = sect2 else {
        return;
    };
    let id_to_target: std::collections::HashMap<String, String> = pkg2
        .read_rels_for(&main2)
        .map(|rels| {
            rels.items
                .iter()
                .map(|r| (r.id.clone(), r.target.clone()))
                .collect()
        })
        .unwrap_or_default();

    let refs: Vec<NodeId> = d2
        .elements(sect2, None)
        .into_iter()
        .filter(|&e| d2.name(e).is_some_and(|n| n == href || n == fref))
        .collect();
    // Copies this pass adopted under a new name (`header3.xml` for B's
    // `header1.xml`): the copy stands for its source's basename, never for
    // its own, which may be another B part's.
    let mut renamed: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for r in refs {
        let is_header = d2.name(r).as_ref() == Some(&href);
        let ty = d2
            .attribute(r, &W::type_())
            .unwrap_or("default")
            .to_string();
        let slot = (is_header, ty.clone());
        // Final sectPr already has this slot → leave it.
        if final_slots.contains(&slot) {
            continue;
        }
        let Some(rid) = d2.attribute(r, &R::id()) else {
            continue;
        };
        let Some(target) = id_to_target.get(rid) else {
            continue;
        };
        let src_part = pkg2.resolve_rel_target(&main2, target);
        let Some(bytes) = pkg2.part_bytes(&src_part).map(<[u8]>::to_vec) else {
            continue;
        };
        // If body already has this slot AND an identical (or any) part with this
        // basename is already packaged, skip — mid-section A footers stay.
        // file_21: mid-body has footer/default from B inserts but final is empty
        // and footer20.xml is missing → still adopt onto final.
        let basename = src_part.rsplit('/').next().unwrap_or(&src_part);
        let part_already = renamed.values().any(|source| source == basename)
            || out
                .parts()
                .iter()
                .any(|p| p.ends_with(basename) && !renamed.contains_key(p));
        if body_occupied.contains(&slot) && part_already {
            continue;
        }
        // name collision with an existing (different) part: copy B's content
        // under a fresh name instead of dropping or clobbering it — byte-safe
        // existence check (part_string is None for binary parts)
        // The part's relationships by type, target and mode (ids aside).
        let rels_of = |pkg: &PartFs, part: &str| {
            pkg.read_rels_for(part).map(|rels| {
                rels.items
                    .iter()
                    .map(|r| (r.rel_type.clone(), r.target.clone(), r.target_mode.clone()))
                    .collect::<Vec<_>>()
            })
        };
        let src_rels = rels_of(pkg2, &src_part);
        let part = unique_part_name_where(out, &src_part, &bytes, &|candidate| {
            rels_of(out, candidate) == src_rels
        });
        if part != src_part {
            renamed.insert(part.clone(), basename.to_string());
        }
        let newly_adopted = out.part_bytes(&part).is_none();
        if newly_adopted {
            out.set_part(&part, bytes);
            // M378 (tiff×h_f −3.3): B-only header/footer content is pure-I in
            // Word (base had no HF). Copying B live left page numbers + labels
            // as Equal and thrash LO pagefair. Mark body content as inserted.
            mark_adopted_hf_content_as_inserted(out, &part, settings);
        }
        let ct = if is_header {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"
        } else {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"
        };
        out.add_content_type_override(&format!("/{part}"), ct);

        // carry the part's own rels + internal targets (header images etc.);
        // internal targets get the same collision-proof treatment — an
        // existing same-named part with DIFFERENT bytes (e.g. doc A's own
        // media/image1.png) must never be overwritten, so B's payload lands
        // under a fresh name and the rel target follows it
        if let Some(hrels) = pkg2.read_rels_for(&src_part) {
            let mut rels_xml = String::from(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
            );
            for hr in &hrels.items {
                let mode = hr
                    .target_mode
                    .as_deref()
                    .map(|m| format!(" TargetMode=\"{m}\""))
                    .unwrap_or_default();
                let mut rel_target_out = hr.target.clone();
                if hr.target_mode.as_deref() != Some("External") {
                    let t = pkg2.resolve_rel_target(&src_part, &hr.target);
                    if let Some(tb) = pkg2.part_bytes(&t).map(<[u8]>::to_vec) {
                        let t_out = unique_part_name(out, &t, &tb);
                        if out.part_bytes(&t_out).is_none() {
                            out.set_part(&t_out, tb);
                        }
                        // rel target is part-dir-relative; a renamed copy
                        // stays in the same directory
                        if t_out != t {
                            rel_target_out = crate::opc::relative_rel_target(&part, &t_out);
                        }
                        if let Some(ext) = t_out.rsplit('.').next() {
                            // case-insensitive: real packages carry .PNG/.Jpg
                            let ext_lc = ext.to_ascii_lowercase();
                            let mime = match ext_lc.as_str() {
                                "png" => Some("image/png"),
                                "jpeg" | "jpg" => Some("image/jpeg"),
                                "gif" => Some("image/gif"),
                                "tiff" | "tif" => Some("image/tiff"),
                                "bmp" => Some("image/bmp"),
                                "svg" => Some("image/svg+xml"),
                                "ico" => Some("image/x-icon"),
                                "emf" => Some("image/x-emf"),
                                "wmf" => Some("image/x-wmf"),
                                _ => None,
                            };
                            if let Some(m) = mime {
                                out.add_content_type_default(ext, m);
                            }
                        }
                    }
                }
                // XML-escape attribute values — external hyperlink targets
                // legitimately carry '&' (URLs with query strings)
                let xe = |s: &str| {
                    s.replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('"', "&quot;")
                };
                rels_xml.push_str(&format!(
                    "<Relationship Id=\"{}\" Type=\"{}\" Target=\"{}\"{}/>",
                    xe(&hr.id),
                    xe(&hr.rel_type),
                    xe(&rel_target_out),
                    mode
                ));
            }
            rels_xml.push_str("</Relationships>");
            let rels_part = match part.rsplit_once('/') {
                Some((dir, base)) => format!("{dir}/_rels/{base}.rels"),
                None => format!("_rels/{part}.rels"),
            };
            out.set_part(&rels_part, rels_xml.into_bytes());
        }

        // rel from the output main + reference element in the final sectPr
        let rel_type = if is_header {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header"
        } else {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"
        };
        // word/-parts get the dir-relative form, anything else the absolute
        // OPC form ("/customXml/…") so the target still resolves.
        let rel_target = crate::opc::relative_rel_target(out_main, &part);
        let new_rid = out.add_document_relationship(out_main, rel_type, &rel_target);
        let refel = dom.new_element(if is_header {
            href.clone()
        } else {
            fref.clone()
        });
        dom.set_attribute_value(refel, &W::type_(), Some(&ty));
        dom.set_attribute_value(refel, &R::id(), Some(&new_rid));
        dom.add_first(out_sect, refel);
    }
}

/// M378 — wrap body content of a newly adopted B-only header/footer as pure-I.
/// Word Compare marks PAGE fields + labels as inserts when the original had no
/// HF; a live copy of B's part renders without revision chrome (−3 pagefair on
/// tiff×h_f_normal).
pub(super) fn mark_adopted_hf_content_as_inserted(
    out: &mut PartFs,
    part: &str,
    settings: &WmlComparerSettings,
) {
    let Some(xml) = out.part_string(part) else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    // Root is w:hdr or w:ftr. New marks number above the story's own.
    let mut next_id = next_free_revision_id(&dom, root);
    let author = settings.author_for_revisions.as_str();
    let date = settings.date_time_for_revisions.as_str();
    let paras: Vec<NodeId> = dom.descendants(root, Some(&W::p()));
    for p in paras {
        // Skip if already has revision markup.
        if !dom.descendants(p, Some(&W::ins())).is_empty()
            || !dom.descendants(p, Some(&W::del())).is_empty()
        {
            continue;
        }
        let kids: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| dom.name(c) != Some(W::p_pr()))
            .collect();
        if kids.is_empty() {
            continue;
        }
        // Any contentful child? (run / hyperlink / sdt / drawing container)
        let has_content = kids.iter().any(|&c| {
            let Some(n) = dom.name(c) else {
                return false;
            };
            n == W::r()
                || n == W::hyperlink()
                || n == W::sdt()
                || n == W::drawing()
                || n == W::fld_simple()
                || n.local_name() == "AlternateContent"
        });
        if !has_content {
            continue;
        }
        // Single wrapper around all body kids (Word PAGE field shape).
        let ins = dom.new_element(W::ins());
        dom.set_attribute_value(ins, &W::author(), Some(author));
        dom.set_attribute_value(ins, &W::date(), Some(date));
        let id = next_id.to_string();
        next_id += 1;
        dom.set_attribute_value(ins, &W::id(), Some(&id));
        // Insert wrapper before first body child, then move kids into it.
        if let Some(&first) = kids.first() {
            if dom.parent(first).is_some() {
                dom.add_before_self(first, ins);
            } else {
                dom.add(p, ins);
            }
        } else {
            dom.add(p, ins);
        }
        for c in kids {
            if dom.parent(c).is_none() {
                continue;
            }
            dom.remove(c);
            dom.add(ins, c);
        }
        // Word also stamps mark ins on pPr/rPr for PAGE-number paras.
        if let Some(ppr) = dom.element(p, &W::p_pr()) {
            let rpr = match dom.element(ppr, &W::r_pr()) {
                Some(r) => r,
                None => {
                    let r = dom.new_element(W::r_pr());
                    // rPr last-ish before pPrChange if any; otherwise append.
                    if let Some(ppc) = dom.element(ppr, &W::p_pr_change()) {
                        dom.add_before_self(ppc, r);
                    } else {
                        dom.add(ppr, r);
                    }
                    r
                }
            };
            if dom.element(rpr, &W::ins()).is_none() && dom.element(rpr, &W::del()).is_none() {
                let mark = dom.new_element(W::ins());
                dom.set_attribute_value(mark, &W::author(), Some(author));
                dom.set_attribute_value(mark, &W::date(), Some(date));
                let mid = next_id.to_string();
                next_id += 1;
                dom.set_attribute_value(mark, &W::id(), Some(&mid));
                // Mark first under rPr (Word: ins then rStyle).
                if let Some(first) = dom.elements(rpr, None).first().copied() {
                    dom.add_before_self(first, mark);
                } else {
                    dom.add(rpr, mark);
                }
            }
        }
    }
    mark_story_rows(&mut dom, root, |d| {
        let mark = d.new_element(W::ins());
        d.set_attribute_value(mark, &W::author(), Some(author));
        d.set_attribute_value(mark, &W::date(), Some(date));
        d.set_attribute_value(mark, &W::id(), Some(&next_id.to_string()));
        next_id += 1;
        mark
    });
    out.set_part(part, dom.serialize_element(root).into_bytes());
}

/// M383 — when the revised document has **no** headers/footers but the original
/// does, Word marks all original HF content pure-D. Eng left A HF live (−45 on
/// h_f_normal_odd_even_firstpg×basic_footnotes). Inverse of M378. The rule
/// holds per (kind, type) slot: a first-page footer does not preserve A's
/// default footer (#1837). A part in `dead_only` (only
/// deleted section breaks reference it) goes as a whole story (#1833).
pub(super) fn mark_a_only_hf_content_as_deleted(
    out: &mut PartFs,
    pkg1: &PartFs,
    pkg2: &PartFs,
    dead_only: &std::collections::HashSet<String>,
    settings: &WmlComparerSettings,
) {
    let a_only = a_only_header_footer_parts(pkg1, pkg2);
    let mut seen = std::collections::HashSet::new();
    let (section_count, a_refs) = header_footer_refs_by_section(pkg1);
    for (_, _, _, part) in &a_refs {
        // A shared slot's content diff owns the markup for its entire part.
        if !a_only.contains(part) || !seen.insert(part.clone()) {
            continue;
        }
        // Part may be stored under the same name in out (A-based package).
        let whole_story = dead_only.contains(part);
        let final_only = a_refs
            .iter()
            .filter(|(_, _, _, p)| p == part)
            .all(|(section, _, _, _)| section + 1 == section_count);
        if needs_unmerged_section_story_cleanup(settings) && !whole_story && !final_only {
            continue;
        }
        mark_hf_part_content_as_deleted(out, part, settings, whole_story);
    }
}

/// Word's redline of a header/footer whose revised story ends with a table:
/// the original's closing paragraph, emptied by the diff, loses its mark,
/// since the revised story needs none, and keeps its own properties
/// unrecorded (bc0135eaa1). Accepted, the story ends with the table.
pub(super) fn delete_story_closing_mark(
    dom: &mut Dom,
    story: NodeId,
    settings: &WmlComparerSettings,
) {
    let Some(&last) = dom.elements(story, None).last() else {
        return;
    };
    let live_content = [W::t(), W::drawing(), W::pict(), W::object()];
    if !dom.name_is(last, &W::p())
        || live_content
            .iter()
            .any(|n| !dom.descendants(last, Some(n)).is_empty())
    {
        return;
    }
    let ppr = match dom.element(last, &W::p_pr()) {
        Some(p) => p,
        None => {
            let p = dom.new_element(W::p_pr());
            dom.add_first(last, p);
            p
        }
    };
    let mark = dom.element(ppr, &W::r_pr());
    if mark
        .is_some_and(|r| dom.element(r, &W::ins()).is_some() || dom.element(r, &W::del()).is_some())
    {
        return;
    }
    // The diff recorded B's blank properties over A's; Word keeps A's.
    if let Some(change) = dom.element(ppr, &W::p_pr_change()) {
        let old = dom.element(change, &W::p_pr());
        dom.remove(change);
        for c in dom.elements(ppr, None) {
            if Some(c) != mark {
                dom.remove(c);
            }
        }
        for c in old.map(|o| dom.elements(o, None)).unwrap_or_default() {
            let clone = dom.clone_subtree(c);
            match mark {
                Some(r) => dom.add_before_self(r, clone),
                None => dom.add(ppr, clone),
            }
        }
    }
    let mark = match mark {
        Some(r) => r,
        None => {
            let r = dom.new_element(W::r_pr());
            dom.add(ppr, r);
            r
        }
    };
    let del = dom.new_element(W::del());
    let id = next_free_revision_id(dom, story).to_string();
    dom.set_attribute_value(del, &W::id(), Some(&id));
    dom.set_attribute_value(del, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(del, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add_first(mark, del);
}

/// Word's redline of a header/footer the revised document drops: the
/// content goes pure-D and every paragraph mark but the story's last is
/// deleted (a paragraph without `pPr` included). The last mark stays: it is
/// the blank paragraph the revised side implies, so its properties reset to
/// that paragraph's (the kind's own `Header`/`Footer` style at most) with
/// the old ones in a `pPrChange` and `rPrChange`. The last paragraph of a
/// content control keeps its mark as well. (Word also restyles that blank
/// paragraph with a style it picks by index from the other document, e.g.
/// `ListParagraph` for a footer; that is not copied.)
///
/// A `whole_story` (one only deleted section breaks reference) loses its
/// last mark too, the paragraph keeping its own properties: no revised
/// paragraph stands in for it, as nothing shows the story once the breaks
/// are accepted. This is Docxodus' shape, and Word's Accept All of it
/// empties the story. Word 16.115 crashed opening Word's own shape of 9
/// redlines whose footer holds a PAGE or NUMPAGES field. With every mark
/// deleted it opened 7 of them; s176 and s472 still crash, because a
/// deleted break references the footer at all (#1833).
pub(super) fn mark_hf_part_content_as_deleted(
    out: &mut PartFs,
    part: &str,
    settings: &WmlComparerSettings,
    whole_story: bool,
) {
    let Some(xml) = out.part_string(part) else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    let mut next_id = next_free_revision_id(&dom, root);
    let author = settings.author_for_revisions.as_str();
    let date = settings.date_time_for_revisions.as_str();
    let mut revision = |dom: &mut Dom, name: XName| -> NodeId {
        let mark = dom.new_element(name);
        dom.set_attribute_value(mark, &W::author(), Some(author));
        dom.set_attribute_value(mark, &W::date(), Some(date));
        dom.set_attribute_value(mark, &W::id(), Some(&next_id.to_string()));
        next_id += 1;
        mark
    };
    let own_style = if dom.name(root) == Some(W::ftr()) {
        "Footer"
    } else {
        "Header"
    };
    let story_last = dom.elements(root, Some(&W::p())).last().copied();
    let paras: Vec<NodeId> = dom.descendants(root, Some(&W::p()));
    for p in paras {
        if !dom.descendants(p, Some(&W::ins())).is_empty()
            || !dom.descendants(p, Some(&W::del())).is_empty()
        {
            continue;
        }
        let kids: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| dom.name(c) != Some(W::p_pr()))
            .collect();
        let has_content = kids.iter().any(|&c| {
            let Some(n) = dom.name(c) else {
                return false;
            };
            n == W::r()
                || n == W::hyperlink()
                || n == W::sdt()
                || n == W::drawing()
                || n == W::fld_simple()
                || n.local_name() == "AlternateContent"
        });
        if has_content {
            let del = revision(&mut dom, W::del());
            dom.add_before_self(kids[0], del);
            for c in kids {
                dom.remove(c);
                // Rename w:t → w:delText under this run tree for pure-D.
                for t in dom.descendants(c, Some(&W::t())) {
                    dom.set_name(t, W::del_text());
                }
                dom.add(del, c);
            }
        }
        let ends_content_control = dom.parent(p).is_some_and(|c| {
            dom.name(c) == Some(W::sdt_content())
                && dom.elements(c, Some(&W::p())).last() == Some(&p)
        });
        if Some(p) == story_last && !whole_story {
            reset_to_blank_paragraph(&mut dom, p, own_style, &mut revision);
        } else if !ends_content_control {
            // Word's PAGE + label shape: the mark goes with the content.
            let ppr = child_or_first(&mut dom, p, W::p_pr());
            let rpr = child_before(&mut dom, ppr, W::r_pr(), &["sectPr", "pPrChange"]);
            if dom.element(rpr, &W::ins()).is_none() && dom.element(rpr, &W::del()).is_none() {
                let mark = revision(&mut dom, W::del());
                match dom.elements(rpr, None).first().copied() {
                    Some(first) => dom.add_before_self(first, mark),
                    None => dom.add(rpr, mark),
                }
            }
        }
    }
    mark_story_rows(&mut dom, root, |d| revision(d, W::del()));
    out.set_part(part, dom.serialize_element(root).into_bytes());
}

/// Mark every table row of a one-sided header/footer story with the
/// story's revision (`w:del` dropped, `w:ins` adopted), as Word's redline
/// does. Word 16.115 hangs on a story table whose cells are all deleted
/// while its rows are unmarked (b09, 0.12.0 release sample). Rows already
/// carrying a mark keep it.
fn mark_story_rows(dom: &mut Dom, story: NodeId, mut mark: impl FnMut(&mut Dom) -> NodeId) {
    for tr in dom.descendants(story, Some(&W::tr())) {
        let trpr = crate::comparer::lcs_table::row_properties(dom, tr);
        if dom.element(trpr, &W::ins()).is_some() || dom.element(trpr, &W::del()).is_some() {
            continue;
        }
        let rev = mark(dom);
        crate::comparer::lcs_table::add_row_mark(dom, trpr, rev);
    }
}

/// Reset the story's last paragraph to the blank one Word's redline keeps:
/// no properties but the kind's own style, the old paragraph properties in
/// a `pPrChange` and the old mark formatting in an `rPrChange`.
fn reset_to_blank_paragraph(
    dom: &mut Dom,
    p: NodeId,
    own_style: &str,
    revision: &mut impl FnMut(&mut Dom, XName) -> NodeId,
) {
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return;
    };
    let keep = |dom: &Dom, c: NodeId| {
        let Some(n) = dom.name(c) else {
            return true;
        };
        matches!(n.local_name(), "rPr" | "sectPr" | "pPrChange")
            || (n == W::p_style() && dom.attribute(c, &W::val()) == Some(own_style))
    };
    let old: Vec<NodeId> = dom
        .elements(ppr, None)
        .into_iter()
        .filter(|&c| !keep(dom, c))
        .collect();
    if !old.is_empty() && dom.element(ppr, &W::p_pr_change()).is_none() {
        let change = revision(dom, W::p_pr_change());
        let recorded = dom.new_element(W::p_pr());
        // The recorded properties are the whole old set, the kept style
        // included.
        for c in dom.elements(ppr, None) {
            if !matches!(
                dom.name(c).map(|n| n.local_name().to_string()).as_deref(),
                Some("rPr" | "sectPr" | "pPrChange")
            ) {
                let copy = dom.clone_subtree(c);
                dom.add(recorded, copy);
            }
        }
        for c in old {
            dom.remove(c);
        }
        dom.add(change, recorded);
        dom.add(ppr, change);
    }
    if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
        let props: Vec<NodeId> = dom
            .elements(rpr, None)
            .into_iter()
            .filter(|&c| {
                !dom.name(c).is_some_and(|n| {
                    matches!(
                        n.local_name(),
                        "ins" | "del" | "moveFrom" | "moveTo" | "rPrChange"
                    )
                })
            })
            .collect();
        if !props.is_empty() && dom.element(rpr, &W::r_pr_change()).is_none() {
            let change = revision(dom, W::r_pr_change());
            let recorded = dom.new_element(W::r_pr());
            for c in props {
                dom.remove(c);
                dom.add(recorded, c);
            }
            dom.add(change, recorded);
            dom.add(rpr, change);
        }
    }
}

/// `parent`'s `name` child, created as its first child when absent.
pub(super) fn child_or_first(dom: &mut Dom, parent: NodeId, name: XName) -> NodeId {
    if let Some(c) = dom.element(parent, &name) {
        return c;
    }
    let c = dom.new_element(name);
    match dom.elements(parent, None).first().copied() {
        Some(first) => dom.add_before_self(first, c),
        None => dom.add(parent, c),
    }
    c
}

/// `parent`'s `name` child, created before the first of `later` (schema
/// order) or last when none is present.
pub(super) fn child_before(dom: &mut Dom, parent: NodeId, name: XName, later: &[&str]) -> NodeId {
    if let Some(c) = dom.element(parent, &name) {
        return c;
    }
    let c = dom.new_element(name);
    let anchor = dom.elements(parent, None).into_iter().find(|&n| {
        dom.name(n)
            .is_some_and(|nm| later.contains(&nm.local_name()))
    });
    match anchor {
        Some(a) => dom.add_before_self(a, c),
        None => dom.add(parent, c),
    }
    c
}

/// M379 — when the original has no real footnotes/endnotes separators but the
/// revised document does, copy B's notes part into the package. Word carries
/// separator + continuationSeparator (often with drawings) even when body has
/// zero footnote refs (tiff×h_f_normal). Empty shells thrash LO page geometry.
pub(super) fn adopt_b_notes_when_a_lacks_separators(
    out: &mut PartFs,
    pkg1: &PartFs,
    pkg2: &PartFs,
    out_main: &str,
) {
    for (part, rel_suffix, ct) in [
        (
            "word/footnotes.xml",
            "footnotes",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        ),
        (
            "word/endnotes.xml",
            "endnotes",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml",
        ),
    ] {
        let a_has_sep = pkg1.part_string(part).is_some_and(|x| {
            x.contains("w:type=\"separator\"") || x.contains("w:type='separator'")
        });
        if a_has_sep {
            continue;
        }
        let Some(b_xml) = pkg2.part_string(part) else {
            continue;
        };
        if !(b_xml.contains("w:type=\"separator\"") || b_xml.contains("w:type='separator'")) {
            continue;
        }
        // Also skip if out already has a real separator part (paired compare).
        let out_has_sep = out.part_string(part).is_some_and(|x| {
            x.contains("w:type=\"separator\"") || x.contains("w:type='separator'")
        });
        if out_has_sep {
            continue;
        }
        // When B.3/B.4 already produced the part (rectified content defs,
        // renumbered 1..n), a wholesale copy of B's part would clobber those
        // defs with B's PRE-rectify ids (c2: ref 1 vs def 2001 dangling).
        // Merge only B's STRUCTURAL notes (separator/continuationSeparator/
        // continuationNotice) in front of the rectified defs instead.
        if let Some(out_xml) = out.part_string(part) {
            let mut sd = Dom::new();
            let od = sd.parse_xdocument(&out_xml);
            let bd = sd.parse_xdocument(&b_xml);
            if let (Some(or), Some(br)) = (sd.root(od), sd.root(bd)) {
                let def = if rel_suffix == "footnotes" {
                    crate::namespaces::W::footnote()
                } else {
                    crate::namespaces::W::endnote()
                };
                let structural: Vec<_> = sd
                    .elements(br, Some(&def))
                    .into_iter()
                    .filter(|&n| crate::comparer::footnotes::is_structural_note(&sd, n))
                    .collect();
                let first_out = sd.elements(or, Some(&def)).first().copied();
                for s in structural {
                    let cloned = sd.clone_subtree(s);
                    match first_out {
                        Some(f) => sd.add_before_self(f, cloned),
                        None => sd.add(or, cloned),
                    }
                }
                out.set_part(part, sd.serialize_element(or).into_bytes());
            }
            continue;
        }
        out.set_part(part, b_xml.into_bytes());
        out.add_content_type_override(&format!("/{part}"), ct);
        // Ensure main-document relationship.
        let has_rel = out.read_rels_for(out_main).is_some_and(|r| {
            r.items
                .iter()
                .any(|i| i.rel_type.ends_with(&format!("/{rel_suffix}")))
        });
        if !has_rel {
            let target = crate::opc::relative_rel_target(out_main, part);
            out.add_document_relationship(
                out_main,
                &format!(
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{rel_suffix}"
                ),
                &target,
            );
        }
        // Carry notes-part rels + media (separator drawings).
        if let Some(nrels) = pkg2.read_rels_for(part) {
            let mut rels_xml = String::from(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
            );
            for nr in &nrels.items {
                let mode = nr
                    .target_mode
                    .as_deref()
                    .map(|m| format!(" TargetMode=\"{m}\""))
                    .unwrap_or_default();
                let mut rel_target_out = nr.target.clone();
                if nr.target_mode.as_deref() != Some("External") {
                    let t = pkg2.resolve_rel_target(part, &nr.target);
                    if let Some(tb) = pkg2.part_bytes(&t).map(<[u8]>::to_vec) {
                        let t_out = unique_part_name(out, &t, &tb);
                        if out.part_bytes(&t_out).is_none() {
                            out.set_part(&t_out, tb);
                        }
                        if t_out != t {
                            rel_target_out = crate::opc::relative_rel_target(part, &t_out);
                        }
                    }
                }
                let xe = |s: &str| {
                    s.replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('"', "&quot;")
                };
                rels_xml.push_str(&format!(
                    "<Relationship Id=\"{}\" Type=\"{}\" Target=\"{}\"{}/>",
                    xe(&nr.id),
                    xe(&nr.rel_type),
                    xe(&rel_target_out),
                    mode
                ));
            }
            rels_xml.push_str("</Relationships>");
            let base = part.rsplit('/').next().unwrap_or(part);
            out.set_part(&format!("word/_rels/{base}.rels"), rels_xml.into_bytes());
        }
    }
}
