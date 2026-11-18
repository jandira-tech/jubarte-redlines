// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Markup pre-processing, story budgets and whole-package accept/reject.

use super::*;

/// Merge every direct `w:body` child of `root` into one body and return it.
/// Normalize Strict/ISO OOXML namespace URIs (`http://purl.oclc.org/ooxml/<cat>/`)
/// to the Transitional URIs (`http://schemas.openxmlformats.org/<cat>/2006/`) the
/// comparer's XName tables use. Word writes either variant; we only model
/// Transitional, so a Strict document.xml otherwise has "no body" (and all markup
/// is unrecognized). No-op for Transitional docs (the common case).
pub(super) fn normalize_strict_namespaces(xml: &str) -> std::borrow::Cow<'_, str> {
    if !xml.contains("purl.oclc.org/ooxml/") {
        return std::borrow::Cow::Borrowed(xml);
    }
    let s = xml
        .replace(
            "http://purl.oclc.org/ooxml/wordprocessingml/",
            "http://schemas.openxmlformats.org/wordprocessingml/2006/",
        )
        .replace(
            "http://purl.oclc.org/ooxml/officeDocument/",
            "http://schemas.openxmlformats.org/officeDocument/2006/",
        )
        .replace(
            "http://purl.oclc.org/ooxml/drawingml/",
            "http://schemas.openxmlformats.org/drawingml/2006/",
        );
    std::borrow::Cow::Owned(s)
}

/// Some producers emit multiple `w:body` elements (invalid, but real — e.g.
/// Apache POI MultipleBodyBug); Word concatenates them. Single-body docs are
/// returned unchanged (early return), so existing behavior is untouched.
pub(super) fn merged_body(dom: &mut Dom, root: NodeId) -> Option<NodeId> {
    let bodies = dom.elements(root, Some(&W::body()));
    if bodies.len() <= 1 {
        return bodies.first().copied();
    }
    let sectpr = W::sect_pr();
    let target = bodies[0];
    let mut content: Vec<NodeId> = Vec::new();
    let mut last_sectpr: Option<NodeId> = None;
    for &b in &bodies {
        for c in dom.nodes(b) {
            dom.remove(c);
            if dom.is_element(c) && dom.name(c).as_ref() == Some(&sectpr) {
                last_sectpr = Some(c);
            } else {
                content.push(c);
            }
        }
    }
    for c in content {
        dom.add(target, c);
    }
    if let Some(sp) = last_sectpr {
        dom.add(target, sp);
    }
    for &b in &bodies[1..] {
        dom.remove(b);
    }
    Some(target)
}

/// Wrap each `w:br` a producer left directly under a paragraph (the schema
/// forbids it) in a run of its own with no properties, `<w:r><w:br/></w:r>`,
/// the way Word reads it. Left bare, the break sat outside the insertion or
/// deletion around it, so Word's Reject All kept every inserted break
/// (bc0135eaa1: 89 of them, 5 pages instead of 2).
fn wrap_bare_breaks(dom: &mut Dom, root: NodeId) {
    let bare: Vec<NodeId> = dom
        .descendants(root, Some(&W::name("br")))
        .into_iter()
        .filter(|&br| {
            dom.parent(br)
                .and_then(|p| dom.name(p))
                .is_some_and(|n| n == W::p())
        })
        .collect();
    for br in bare {
        let run = dom.new_element(W::r());
        dom.add_before_self(br, run);
        dom.remove(br);
        dom.add(run, br);
    }
}

/// The styleIds the package's stylesheet defines, or `None` without one.
fn defined_style_ids(pkg: &PartFs, main: &str) -> Option<std::collections::HashSet<String>> {
    let part = pkg.read_rels_for(main).and_then(|rels| {
        rels.items
            .iter()
            .find(|r| r.target_mode.is_none() && r.rel_type.ends_with("/styles"))
            .map(|r| pkg.resolve_rel_target(main, &r.target))
    })?;
    let xml = pkg.part_string(&part)?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    Some(crate::comparer::footnotes::defined_style_ids(&dom, root))
}

/// [`pre_process_markup`]. `keep_unresolved_styles` leaves a reference to a
/// style the stylesheet lacks in place: an edit's two sides share one
/// stylesheet, so the reference cannot take the other side's style (#1868).
pub(super) fn pre_process(
    pkg: &mut PartFs,
    starting_id_for_footnotes_endnotes: i32,
    keep_unresolved_styles: bool,
) -> Result<Vec<String>, OpcError> {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    // Resolve the notes parts via the document rels — the package-level
    // equivalent of wDoc.MainDocumentPart.FootnotesPart/EndnotesPart.
    let mut fn_part: Option<String> = None;
    let mut en_part: Option<String> = None;
    if let Some(rels) = pkg.read_rels_for(&main) {
        for r in &rels.items {
            if r.target_mode.as_deref() == Some("External") {
                continue;
            }
            match r.rel_type.rsplit('/').next().unwrap_or("") {
                "footnotes" => fn_part = Some(pkg.resolve_rel_target(&main, &r.target)),
                "endnotes" => en_part = Some(pkg.resolve_rel_target(&main, &r.target)),
                _ => {}
            }
        }
    }

    let Some(main_xml) = pkg.part_string(&main) else {
        return Ok(Vec::new());
    };
    let fn_xml = fn_part.as_deref().and_then(|p| pkg.part_string(p));
    let en_xml = en_part.as_deref().and_then(|p| pkg.part_string(p));

    let mut dom = Dom::new();
    let main_doc = dom.parse_xdocument(&main_xml);
    let Some(main_root) = dom.root(main_doc) else {
        return Ok(Vec::new());
    };
    let fn_doc = fn_xml.as_deref().map(|x| dom.parse_xdocument(x));
    let fn_root = fn_doc.and_then(|d| dom.root(d));
    let en_doc = en_xml.as_deref().map(|x| dom.parse_xdocument(x));
    let en_root = en_doc.and_then(|d| dom.root(d));

    // Renumber only when there is something to renumber or rewrite; a doc
    // WITH references but no notes part reaches the orphan panic inside the
    // unique-range step, before any part creation.
    let fn_ref = W::name("footnoteReference");
    let en_ref = W::name("endnoteReference");
    let has_refs = dom
        .descendants(main_root, None)
        .into_iter()
        .any(|d| dom.name(d).is_some_and(|n| n == fn_ref || n == en_ref));
    let mut changed = Vec::new();
    // C.1 — unique-range renumbering (only meaningful when notes-relevant; a
    // doc WITH references but no notes part errs inside, where C# throws).
    if has_refs || fn_root.is_some() || en_root.is_some() {
        crate::comparer::footnotes::change_footnote_endnote_references_to_unique_range(
            &mut dom,
            main_root,
            fn_root,
            en_root,
            starting_id_for_footnotes_endnotes,
            false,
        )
        .map_err(invalid_content)?;
    }

    // Before the unids, so the runs the repair adds get theirs.
    let style_ids = defined_style_ids(pkg, &main);
    for r in [Some(main_root), fn_root, en_root].into_iter().flatten() {
        wrap_bare_breaks(&mut dom, r);
        // Word reads a paragraph or run naming a style its own stylesheet
        // lacks as unstyled. Kept until the stylesheets merge, the reference
        // took the other document's style of that id (221577c35b: the
        // original's Normal text turned into the revision's headings).
        if let Some(ids) = &style_ids
            && !keep_unresolved_styles
        {
            crate::comparer::footnotes::strip_unresolved_style_refs(&mut dom, r, ids);
        }
    }

    // C.5 — `AddUnidsToMarkupInContentParts` (:600): stamp `pt:Unid` on every
    // element of main + notes parts and declare pt14 mc:Ignorable on each
    // root. Runs BEFORE FillInEmpty like C#, so the stock fill paragraphs are
    // deliberately unid-less after preprocessing.
    crate::unid::assign_to_all_elements(&mut dom, main_root);
    crate::comparer::finalize::ignore_pt14_namespace(&mut dom, main_root);
    for r in [fn_root, en_root].into_iter().flatten() {
        crate::unid::assign_to_all_elements(&mut dom, r);
        crate::comparer::finalize::ignore_pt14_namespace(&mut dom, r);
    }

    // C.3 — `FillInEmptyFootnotesEndnotes` (:513): childless note definitions
    // gain the stock reference paragraph before diffing. (C# runs it after
    // AddFootnotesEndnotesParts, but freshly created parts hold no
    // definitions, so applying it here is identical.)
    if let Some(r) = fn_root {
        crate::comparer::footnotes::fill_in_empty_footnotes_endnotes(&mut dom, r, true);
    }
    if let Some(r) = en_root {
        crate::comparer::footnotes::fill_in_empty_footnotes_endnotes(&mut dom, r, false);
    }

    // Write-back: main always (it now carries unids), notes parts when present.
    pkg.set_part(&main, dom.serialize_document(main_doc).into_bytes());
    changed.push(main.clone());
    if let (Some(p), Some(d)) = (fn_part.as_deref(), fn_doc) {
        pkg.set_part(p, dom.serialize_document(d).into_bytes());
        changed.push(p.to_string());
    }
    if let (Some(p), Some(d)) = (en_part.as_deref(), en_doc) {
        pkg.set_part(p, dom.serialize_document(d).into_bytes());
        changed.push(p.to_string());
    }

    // C.2 — `AddFootnotesEndnotesParts` (:1604): UNCONDITIONALLY add an EMPTY
    // namespace-decorated notes part (rels + content type) when one is
    // missing. No separator notes here — C# adds those only when Rectify
    // rebuilds the output part. Runs AFTER the renumbering, like C# (a doc
    // with references but no part errs above, never reaches creation).
    let dir = main.rsplit_once('/').map(|(d, _)| d).unwrap_or("word");
    for (present, local) in [
        (fn_part.is_some(), "footnotes"),
        (en_part.is_some(), "endnotes"),
    ] {
        if present {
            continue;
        }
        let part = format!("{dir}/{local}.xml");
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <w:{local} {NOTES_ROOT_NAMESPACE_ATTRS}></w:{local}>"
        );
        pkg.set_part(&part, xml.into_bytes());
        pkg.add_document_relationship(
            &main,
            &format!("http://schemas.openxmlformats.org/officeDocument/2006/relationships/{local}"),
            &format!("{local}.xml"),
        );
        pkg.add_content_type_override(
            &format!("/{part}"),
            &format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{local}+xml"),
        );
        changed.push(part);
    }

    // C.4 — `DetachExternalData` (:497): strip `c:externalData` from every
    // chart part related to the main document. External-link relationships
    // are not propagated to the destination document, so the references would
    // dangle; the chart's own rels are left untouched. (C# rewrites every
    // chart part; we only rewrite ones that actually held externalData —
    // a serialization-only difference.)
    let chart_parts: Vec<String> = pkg
        .read_rels_for(&main)
        .map(|rels| {
            rels.items
                .iter()
                .filter(|r| {
                    r.target_mode.as_deref() != Some("External") && r.rel_type.ends_with("/chart")
                })
                .map(|r| pkg.resolve_rel_target(&main, &r.target))
                .collect()
        })
        .unwrap_or_default();
    for part in chart_parts {
        let Some(xml) = pkg.part_string(&part) else {
            continue;
        };
        let mut cdom = Dom::new();
        let cdoc = cdom.parse_xdocument(&xml);
        let Some(croot) = cdom.root(cdoc) else {
            continue;
        };
        let ext: Vec<NodeId> =
            cdom.descendants(croot, Some(&crate::namespaces::C::name("externalData")));
        if ext.is_empty() {
            continue;
        }
        for e in ext {
            cdom.remove(e);
        }
        pkg.set_part(&part, cdom.serialize_document(cdoc).into_bytes());
        changed.push(part);
    }
    Ok(changed)
}

/// Markup the engine refuses (C# throws DocxodusException): an `Err` the
/// caller can handle, never a panic, which would abort a WASM instance.
/// `InvalidData` is std's kind for well-formed-but-unacceptable input.
fn invalid_content(message: String) -> OpcError {
    crate::opc::refused(UnsupportedContentError { message })
}

/// A story's weight under [`crate::admission::InputLimits::max_story_units`]:
/// its elements plus the characters of its text, about the atoms the
/// comparer makes of it.
pub(super) fn story_units(dom: &Dom, root: NodeId) -> u64 {
    let text = [W::t(), W::del_text(), W::instr_text(), W::del_instr_text()];
    let mut units = 1u64;
    dom.for_each_descendant_element(root, None, |e| {
        units = units.saturating_add(1);
        if text.iter().any(|name| dom.name_is(e, name)) {
            let chars = u64::try_from(dom.value(e).chars().count()).unwrap_or(u64::MAX);
            units = units.saturating_add(chars);
        }
    });
    units
}

/// Refuse `story` when `units` pass the budget, before any atom is made:
/// the comparer's memory follows the atoms (word_exp B1, 2026-10-31).
pub(super) fn refuse_past_story_budget(
    units: u64,
    story: &str,
    limits: crate::admission::InputLimits,
) -> Result<(), OpcError> {
    if units <= limits.max_story_units {
        return Ok(());
    }
    Err(crate::opc::refused(crate::admission::AdmissionError {
        kind: crate::admission::AdmissionErrorKind::InputLimit,
        message: format!(
            "{story} holds {units} story units (characters and elements); the limit is {} story units",
            limits.max_story_units
        ),
    }))
}

/// Refuse a story the atomizer cannot flatten, in one walk: a subdocument,
/// an altChunk, a content part, or a customXml range marker (word_exp N4
/// panicked the atomizer, 2026-10-31), or an element with two direct
/// `w:sectPr` (the atomizer's last-sectPr move expects one). A `w:customXml`
/// element is not a refusal: the atomizer unwraps it and keeps its content
/// (#1674). `story` names the part or side in the error.
pub(super) fn refuse_unatomizable(dom: &Dom, root: NodeId, story: &str) -> Result<(), OpcError> {
    let content_part = W::content_part();
    let sect_pr = W::sect_pr();
    let custom_xml = W::custom_xml();
    // The atomizer unwraps `w:customXml` first, so a sectPr's parent is
    // its nearest ancestor that is not one: two sectPr meet there.
    let unwrapped_parent = |mut node: NodeId| loop {
        let parent = dom.parent(node)?;
        if !dom.name_is(parent, &custom_xml) {
            return Some(parent);
        }
        node = parent;
    };
    let mut sect_parents = std::collections::HashSet::new();
    let mut bad = None;
    dom.find_descendant_element(root, None, |e| {
        if dom.name_is(e, &sect_pr) {
            if let Some(parent) = unwrapped_parent(e)
                && !sect_parents.insert(parent)
            {
                bad = Some("Document contains two sectPr in one story element".to_string());
            }
        } else if dom.name_is(e, &content_part)
            || dom.name(e).is_some_and(|name| {
                crate::comparer::tables::INVALID_ELEMENTS.contains(&name) && name != custom_xml
            })
        {
            let name = dom
                .name(e)
                .map(|n| n.local_name().to_string())
                .unwrap_or_default();
            bad = Some(format!("Document contains {name}"));
        }
        bad.is_some()
    });
    bad.map_or(Ok(()), |e| Err(invalid_content(format!("{story}: {e}"))))
}

/// The namespace declarations C# attaches to a freshly-created
/// `w:footnotes`/`w:endnotes` root (`NamespaceAttributes`/
/// `FreshNamespaceAttributes` :1580–:1602), verbatim.
const NOTES_ROOT_NAMESPACE_ATTRS: &str = concat!(
    "xmlns:wpc=\"http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas\" ",
    "xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\" ",
    "xmlns:o=\"urn:schemas-microsoft-com:office:office\" ",
    "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" ",
    "xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" ",
    "xmlns:v=\"urn:schemas-microsoft-com:vml\" ",
    "xmlns:wp14=\"http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing\" ",
    "xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" ",
    "xmlns:w10=\"urn:schemas-microsoft-com:office:word\" ",
    "xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" ",
    "xmlns:w14=\"http://schemas.microsoft.com/office/word/2010/wordml\" ",
    "xmlns:wpg=\"http://schemas.microsoft.com/office/word/2010/wordprocessingGroup\" ",
    "xmlns:wpi=\"http://schemas.microsoft.com/office/word/2010/wordprocessingInk\" ",
    "xmlns:wne=\"http://schemas.microsoft.com/office/word/2006/wordml\" ",
    "xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\" ",
    "mc:Ignorable=\"w14 wp14\""
);

/// [`accept_revisions`] under the caller's admission budget, which the
/// compare flow already checked (#645). Bookmark and comment ids stay, so a
/// redline names its comments by the ids the documents gave them, as
/// `edit` reports them.
pub(super) fn accept_source_revisions(
    docx: &[u8],
    limits: crate::admission::InputLimits,
) -> Result<Vec<u8>, OpcError> {
    let mut pkg = PartFs::open_within(docx, limits)?;
    crate::revision_processor::accept_revisions_package(&mut pkg);
    pkg.to_zip()
}

/// Collision-proof target name for copying `want` (with `bytes`) into `out`:
/// free name or byte-identical existing part → `want` unchanged; an existing
/// part with DIFFERENT content → the first free or identical candidate: a
/// header or footer (`header1.xml`) takes the next number of its kind
/// (`header3.xml`), a name the short ids spell (`header3.p1`); any other
/// part `{dir}/redlineB[_{n}]_{base}`. Byte-based on purpose — `part_string`
/// returns None for binary parts and would treat existing images as absent.
pub(super) fn unique_part_name(out: &PartFs, want: &str, bytes: &[u8]) -> String {
    unique_part_name_where(out, want, bytes, &|_| true)
}

/// [`unique_part_name`], reusing an identical part only when `same` also
/// holds for it (a header's relationships, so identical XML that shows
/// another image is another part).
pub(super) fn unique_part_name_where(
    out: &PartFs,
    want: &str,
    bytes: &[u8],
    same: &dyn Fn(&str) -> bool,
) -> String {
    match out.part_bytes(want) {
        None => want.to_string(),
        Some(existing) if existing == bytes && same(want) => want.to_string(),
        Some(_) => {
            let (dir, base) = want.rsplit_once('/').unwrap_or(("word", want));
            let story = ["header", "footer"].into_iter().find(|kind| {
                base.strip_prefix(kind)
                    .and_then(|rest| rest.strip_suffix(".xml"))
                    .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            });
            if let Some(kind) = story {
                for n in 1usize.. {
                    let candidate = format!("{dir}/{kind}{n}.xml");
                    match out.part_bytes(&candidate) {
                        None => return candidate,
                        Some(existing) if existing == bytes && same(&candidate) => {
                            return candidate;
                        }
                        Some(_) => {}
                    }
                }
            }
            let mut n = 0usize;
            loop {
                let candidate = if n == 0 {
                    format!("{dir}/redlineB_{base}")
                } else {
                    format!("{dir}/redlineB_{n}_{base}")
                };
                match out.part_bytes(&candidate) {
                    None => return candidate,
                    Some(existing) if existing == bytes && same(&candidate) => return candidate,
                    Some(_) => n += 1,
                }
            }
        }
    }
}
