// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.H.4 — footnote/endnote id-range, mandatory parts, empty-fill, revision
//! predicate. Port of ChangeFootnoteEndnoteReferencesToUniqueRange (:2135),
//! MandatorySeparatorNotes (:2077), FillInEmptyFootnotesEndnotes (:1053),
//! ContentContainsFootnoteEndnoteReferencesThatHaveRevisions (:1101).
//! (Full cross-part ProcessFootnoteEndnote diff + package plumbing land next.)

use super::WmlComparerSettings;
use crate::namespaces::W;
use crate::xmllinq::{Dom, NodeId, XName};

fn set_id(dom: &mut Dom, e: NodeId, v: &str) {
    dom.set_attribute_value(e, &W::id(), Some(v));
}

/// M4.H.4 — `ChangeFootnoteEndnoteReferencesToUniqueRange` (:2135): renumber each
/// `w:footnoteReference`/`w:endnoteReference` (document order) and its definition
/// to a unique monotonic range from `start`. Orphan ref → warning+remove (if
/// `has_log`) else `Err`. Returns the warnings.
pub fn change_footnote_endnote_references_to_unique_range(
    dom: &mut Dom,
    main_root: NodeId,
    fn_root: Option<NodeId>,
    en_root: Option<NodeId>,
    start: i32,
    has_log: bool,
) -> Result<Vec<String>, String> {
    let fn_ref = W::name("footnoteReference");
    let en_ref = W::name("endnoteReference");
    let refs: Vec<NodeId> = dom
        .descendants(main_root, None)
        .into_iter()
        .filter(|&d| dom.name(d).is_some_and(|n| n == fn_ref || n == en_ref))
        .collect();

    let mut warnings = Vec::new();
    let mut orphans = Vec::new();
    for (id, r) in (start..).zip(refs) {
        let old = dom.attribute(r, &W::id()).unwrap_or("").to_string();
        let new = id.to_string();
        let is_fn = dom.name(r).unwrap() == fn_ref;
        let (notes_root, def_name) = if is_fn {
            (fn_root, W::footnote())
        } else {
            (en_root, W::endnote())
        };
        let def = notes_root.and_then(|nr| {
            dom.elements(nr, Some(&def_name))
                .into_iter()
                .find(|&e| dom.attribute(e, &W::id()) == Some(old.as_str()))
        });
        match def {
            Some(def) => {
                set_id(dom, r, &new);
                set_id(dom, def, &new);
            }
            None => {
                if has_log {
                    warnings.push(format!(
                        "orphaned {} reference id '{old}'",
                        def_name.local_name()
                    ));
                    orphans.push(r);
                } else {
                    return Err(format!(
                        "Invalid document: {} reference id '{old}' has no definition",
                        def_name.local_name()
                    ));
                }
            }
        }
    }
    for o in orphans {
        dom.remove(o);
    }
    Ok(warnings)
}

/// M4.H.4 — `MandatorySeparatorNotes` (:2077): the `separator` (id -1) and
/// `continuationSeparator` (id 0) notes a footnotes/endnotes part must contain.
pub fn mandatory_separator_notes(dom: &mut Dom, is_footnote: bool) -> Vec<NodeId> {
    let note_name = if is_footnote {
        W::footnote()
    } else {
        W::endnote()
    };
    let sep_inner = "separator"; // both footnote and endnote parts use w:separator
    let mk = |dom: &mut Dom, ty: &str, id: &str, inner: &str| -> NodeId {
        let note = dom.new_element(note_name.clone());
        dom.set_attribute_value(note, &W::name("type"), Some(ty));
        dom.set_attribute_value(note, &W::id(), Some(id));
        let p = dom.new_element(W::p());
        let ppr = dom.new_element(W::p_pr());
        let spacing = dom.new_element(W::name("spacing"));
        dom.set_attribute_value(spacing, &W::name("after"), Some("0"));
        dom.set_attribute_value(spacing, &W::name("line"), Some("240"));
        dom.set_attribute_value(spacing, &W::name("lineRule"), Some("auto"));
        dom.add(ppr, spacing);
        dom.add(p, ppr);
        let r = dom.new_element(W::r());
        let s = dom.new_element(W::name(inner));
        dom.add(r, s);
        dom.add(p, r);
        dom.add(note, p);
        note
    };
    vec![
        mk(dom, "separator", "-1", sep_inner),
        mk(dom, "continuationSeparator", "0", "continuationSeparator"),
    ]
}

/// M4.H.4 — `FillInEmptyFootnotesEndnotes` (:1053): give every element-less
/// `w:footnote`/`w:endnote` a stock reference paragraph.
pub fn fill_in_empty_footnotes_endnotes(dom: &mut Dom, notes_root: NodeId, is_footnote: bool) {
    let (note_name, p_style, r_style, ref_el) = if is_footnote {
        (
            W::footnote(),
            "FootnoteText",
            "FootnoteReference",
            "footnoteRef",
        )
    } else {
        (
            W::endnote(),
            "EndnoteText",
            "EndnoteReference",
            "endnoteRef",
        )
    };
    let notes: Vec<NodeId> = dom.elements(notes_root, Some(&note_name));
    for note in notes {
        if dom.has_elements(note) {
            continue;
        }
        let p = dom.new_element(W::p());
        let ppr = dom.new_element(W::p_pr());
        let pstyle = dom.new_element(W::name("pStyle"));
        dom.set_attribute_value(pstyle, &W::val(), Some(p_style));
        dom.add(ppr, pstyle);
        dom.add(p, ppr);
        let r = dom.new_element(W::r());
        let rpr = dom.new_element(W::r_pr());
        let rstyle = dom.new_element(W::name("rStyle"));
        dom.set_attribute_value(rstyle, &W::val(), Some(r_style));
        dom.add(rpr, rstyle);
        dom.add(r, rpr);
        let refe = dom.new_element(W::name(ref_el));
        dom.add(r, refe);
        dom.add(p, r);
        dom.add(note, p);
    }
}

/// M4.H.4 — `ContentContainsFootnoteEndnoteReferencesThatHaveRevisions` (:1101):
/// true if any footnote/endnote referenced under `element` has a definition (in
/// the delta parts) containing `w:ins`/`w:del`.
pub fn content_contains_footnote_endnote_references_that_have_revisions(
    dom: &Dom,
    element: NodeId,
    fn_root: Option<NodeId>,
    en_root: Option<NodeId>,
) -> bool {
    let fn_ref = W::name("footnoteReference");
    let en_ref = W::name("endnoteReference");
    let ins = W::ins();
    let del = W::del();
    for d in dom.descendants(element, None) {
        let Some(n) = dom.name(d) else { continue };
        let (notes_root, def_name) = if n == fn_ref {
            (fn_root, W::footnote())
        } else if n == en_ref {
            (en_root, W::endnote())
        } else {
            continue;
        };
        let id = dom.attribute(d, &W::id());
        if let (Some(nr), Some(id)) = (notes_root, id)
            && let Some(def) = dom
                .elements(nr, Some(&def_name))
                .into_iter()
                .find(|&e| dom.attribute(e, &W::id()) == Some(id))
        {
            let has_rev = dom
                .descendants(def, None)
                .into_iter()
                .any(|x| dom.name(x).is_some_and(|xn| xn == ins || xn == del));
            if has_rev {
                return true;
            }
        }
    }
    false
}

/// Collect `w:styleId` values defined under a styles root (`w:styles`).
pub fn defined_style_ids(dom: &Dom, styles_root: NodeId) -> std::collections::HashSet<String> {
    let style = W::name("style");
    let style_id = W::name("styleId");
    dom.elements(styles_root, Some(&style))
        .into_iter()
        .filter_map(|st| dom.attribute(st, &style_id).map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Drop `w:pStyle` / `w:rStyle` whose `@w:val` is not defined in the stylesheet.
///
/// Word's compare output omits dangling style refs; LibreOffice still maps
/// built-in names (`Heading1`, `Title`, …) to factory heading look when the
/// attribute is present, even if `styles.xml` has no matching `w:style`. That
/// alone flips pixel scores on style-demo pairs (heading_1_bold × heading_1_style:
/// Word/jubarte ~100 with no `pStyle`; ours kept `pStyle=Heading1` → ~54).
pub fn strip_unresolved_style_refs(
    dom: &mut Dom,
    root: NodeId,
    defined: &std::collections::HashSet<String>,
) -> usize {
    let mut removed = 0;
    for local in ["pStyle", "rStyle"] {
        let nm = W::name(local);
        let victims: Vec<NodeId> = dom
            .descendants(root, Some(&nm))
            .into_iter()
            .filter(|&e| {
                matches!(dom.attribute(e, &W::val()),
                    Some(v) if !v.is_empty() && !defined.contains(v))
            })
            .collect();
        for e in victims {
            dom.remove(e);
            removed += 1;
        }
    }
    removed
}

/// M4.H.8 — `CopyMissingStylesFromOneDocToAnother` (:2547): copy each `w:style`
/// from `from_root` not already present in `to_root` (keyed by type+styleId),
/// dropping its `w:default` attribute (avoid two defaults). Ensures inserted
/// content's styles exist in the output.
pub fn copy_missing_styles(dom: &mut Dom, to_root: NodeId, from_root: NodeId) {
    let style = W::name("style");
    let type_a = W::name("type");
    let style_id = W::name("styleId");
    let mut existing: std::collections::HashSet<(String, String)> = dom
        .elements(to_root, Some(&style))
        .into_iter()
        .map(|st| {
            (
                dom.attribute(st, &type_a).unwrap_or("").to_string(),
                dom.attribute(st, &style_id).unwrap_or("").to_string(),
            )
        })
        .collect();
    let from_styles = dom.elements(from_root, Some(&style));
    for s in from_styles {
        let key = (
            dom.attribute(s, &type_a).unwrap_or("").to_string(),
            dom.attribute(s, &style_id).unwrap_or("").to_string(),
        );
        // Mirror the TS, which re-queries the growing destination each iteration:
        // a (type, styleId) duplicated within `from_root` is copied at most once.
        if !existing.insert(key) {
            continue;
        }
        let cloned = dom.clone_subtree(s);
        dom.set_attribute_value(cloned, &W::name("default"), None);
        dom.add(to_root, cloned);
    }
}

/// E.1 — `AddNumberingChildInSchemaOrder` (:2295): insert into `w:numbering`
/// respecting `numPicBullet* abstractNum* num* numIdMacAtCleanup?` — place
/// BEFORE the first existing child of a higher rank.
fn add_numbering_child_in_schema_order(dom: &mut Dom, numbering: NodeId, child: NodeId) {
    fn rank(dom: &Dom, e: NodeId) -> i32 {
        match dom
            .name(e)
            .map(|n| n.local_name().to_string())
            .unwrap_or_default()
            .as_str()
        {
            "numPicBullet" => 0,
            "abstractNum" => 1,
            "num" => 2,
            _ => 3, // numIdMacAtCleanup (and any trailing element) sorts last
        }
    }
    let r = rank(dom, child);
    let first_later = dom
        .elements(numbering, None)
        .into_iter()
        .find(|&e| rank(dom, e) > r);
    match first_later {
        Some(fl) => dom.add_before_self(fl, child),
        None => dom.add(numbering, child),
    }
}

/// E.1 — `GetIntAttribute` (:2315): parse an integer attribute, None when
/// missing or unparseable.
fn get_int_attribute(dom: &Dom, e: NodeId, name: &XName) -> Option<i32> {
    dom.attribute(e, name).and_then(|v| v.parse().ok())
}

/// E.1 — `NormalizeAbstractNumForComparison` (:2331) + `XNode.DeepEquals`:
/// clone minus the abstractNumId attribute and the (first) nsid/tmpl
/// children, compared by serialization.
fn normalized_abstract_num_signature(dom: &mut Dom, abstract_num: NodeId) -> String {
    let c = dom.clone_subtree(abstract_num);
    dom.set_attribute_value(c, &W::name("abstractNumId"), None);
    for name in ["nsid", "tmpl"] {
        if let Some(e) = dom.element(c, &W::name(name)) {
            dom.remove(e);
        }
    }
    dom.serialize_element(c)
}

/// E.1 — `CopyMissingNumberingFromOneDocToAnother` (:2143): content-dedup
/// (an abstractNum whose normalized content already exists is REUSED),
/// id-remap (colliding abstractNumId/numId get fresh ids past the destination
/// maximum, with num references rewired), and schema-order insertion.
/// Malformed elements (missing/unparseable ids) are skipped like C#.
pub fn copy_missing_numbering(dom: &mut Dom, to_root: NodeId, from_root: NodeId) {
    let abstract_num = W::name("abstractNum");
    let num = W::name("num");
    let abstract_num_id = W::name("abstractNumId");
    let num_id_attr = W::name("numId");

    let mut max_abstract_num_id = dom
        .elements(to_root, Some(&abstract_num))
        .into_iter()
        .map(|e| get_int_attribute(dom, e, &abstract_num_id).unwrap_or(0))
        .max()
        .unwrap_or(0);
    let mut max_num_id = dom
        .elements(to_root, Some(&num))
        .into_iter()
        .map(|e| get_int_attribute(dom, e, &num_id_attr).unwrap_or(0))
        .max()
        .unwrap_or(0);

    // source abstractNumId → destination abstractNumId
    let mut abstract_map: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    for an in dom.elements(from_root, Some(&abstract_num)) {
        let Some(from_id) = get_int_attribute(dom, an, &abstract_num_id) else {
            continue;
        };
        let normalized_from = normalized_abstract_num_signature(dom, an);
        // content match against the CURRENT destination (grows as we copy)
        let to_ans: Vec<NodeId> = dom.elements(to_root, Some(&abstract_num));
        let mut matching: Option<NodeId> = None;
        for e in to_ans {
            if normalized_abstract_num_signature(dom, e) == normalized_from {
                matching = Some(e);
                break;
            }
        }
        if let Some(m) = matching
            && let Some(existing_id) = get_int_attribute(dom, m, &abstract_num_id)
        {
            abstract_map.insert(from_id, existing_id);
            continue;
        }
        let same_id_taken = dom
            .elements(to_root, Some(&abstract_num))
            .into_iter()
            .any(|e| get_int_attribute(dom, e, &abstract_num_id) == Some(from_id));
        let target_id = if same_id_taken {
            max_abstract_num_id += 1;
            max_abstract_num_id
        } else {
            // retained ids must advance the watermark, or a later collision
            // can allocate max+1 == an already-retained id (PR #53 review)
            max_abstract_num_id = max_abstract_num_id.max(from_id);
            from_id
        };
        let cloned = dom.clone_subtree(an);
        dom.set_attribute_value(cloned, &abstract_num_id, Some(&target_id.to_string()));
        abstract_map.insert(from_id, target_id);
        add_numbering_child_in_schema_order(dom, to_root, cloned);
    }

    for n in dom.elements(from_root, Some(&num)) {
        let Some(from_num_id) = get_int_attribute(dom, n, &num_id_attr) else {
            continue;
        };
        let Some(from_ref) = dom
            .element(n, &abstract_num_id)
            .and_then(|e| get_int_attribute(dom, e, &W::val()))
        else {
            continue;
        };
        let mapped = *abstract_map.get(&from_ref).unwrap_or(&from_ref);
        let existing = dom
            .elements(to_root, Some(&num))
            .into_iter()
            .find(|&e| get_int_attribute(dom, e, &num_id_attr) == Some(from_num_id));
        if let Some(existing) = existing {
            let existing_ref = dom
                .element(existing, &abstract_num_id)
                .and_then(|e| get_int_attribute(dom, e, &W::val()));
            if existing_ref == Some(mapped) {
                continue; // same num with the same (mapped) reference
            }
            max_num_id += 1;
            let cloned = dom.clone_subtree(n);
            dom.set_attribute_value(cloned, &num_id_attr, Some(&max_num_id.to_string()));
            if let Some(e) = dom.element(cloned, &abstract_num_id) {
                dom.set_attribute_value(e, &W::val(), Some(&mapped.to_string()));
            }
            add_numbering_child_in_schema_order(dom, to_root, cloned);
        } else {
            // retained numId — advance the watermark so a later collision
            // can't allocate a duplicate (PR #53 review)
            max_num_id = max_num_id.max(from_num_id);
            let cloned = dom.clone_subtree(n);
            if mapped != from_ref
                && let Some(e) = dom.element(cloned, &abstract_num_id)
            {
                dom.set_attribute_value(e, &W::val(), Some(&mapped.to_string()));
            }
            add_numbering_child_in_schema_order(dom, to_root, cloned);
        }
    }
}

/// Word-mode repair (beyond PowerTools): Word synthesizes a default decimal
/// multilevel numbering definition when a document references a `w:numId`
/// that no `w:num` defines — its document-open repair does exactly this
/// (evidence: nested-table-rowspan_numbered-list — the revised fixture's
/// dangling numId=2 gets a synthesized numbering part; carried verbatim, the
/// list renders as plain paragraphs). Appends ONE abstractNum (fresh id) and
/// a `w:num` per dangling id, in schema order.
pub fn synthesize_dangling_numbering(dom: &mut Dom, numbering_root: NodeId, dangling: &[String]) {
    if dangling.is_empty() {
        return;
    }
    let fresh_aid = dom
        .elements(numbering_root, Some(&W::name("abstractNum")))
        .into_iter()
        .filter_map(|e| get_int_attribute(dom, e, &W::name("abstractNumId")))
        .max()
        .map_or(0, |m| m + 1);
    let mut lvls = String::new();
    for ilvl in 0..9 {
        lvls.push_str(&format!(
            "<w:lvl w:ilvl=\"{ilvl}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
             <w:lvlText w:val=\"%{n}.\"/><w:lvlJc w:val=\"left\"/>\
             <w:pPr><w:ind w:left=\"{left}\" w:hanging=\"360\"/></w:pPr></w:lvl>",
            n = ilvl + 1,
            left = 720 * (ilvl + 1),
        ));
    }
    let xml = format!(
        "<w:abstractNum xmlns:w=\"{w}\" w:abstractNumId=\"{fresh_aid}\">\
         <w:multiLevelType w:val=\"multilevel\"/>{lvls}</w:abstractNum>",
        w = W::URI
    );
    let d = dom.parse_xdocument(&xml);
    if let Some(an) = dom.root(d) {
        add_numbering_child_in_schema_order(dom, numbering_root, an);
    }
    for id in dangling {
        let nxml = format!(
            "<w:num xmlns:w=\"{w}\" w:numId=\"{id}\">\
             <w:abstractNumId w:val=\"{fresh_aid}\"/></w:num>",
            w = W::URI
        );
        let nd = dom.parse_xdocument(&nxml);
        if let Some(ne) = dom.root(nd) {
            add_numbering_child_in_schema_order(dom, numbering_root, ne);
        }
    }
}

/// Errors raised by [`rectify_footnote_endnote_ids`]. Mirrors the failure
/// paths around `WlComparer` `RectifyFootnoteEndnoteIds` (:3816); typed so
/// callers can choose to fail-loud vs skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RectifyError {
    /// A note definition id was not found in before ∪ after parts.
    MissingNoteDef {
        /// The missing note definition id.
        id: String,
    },
    /// The withRevisions footnotes/endnotes part is absent.
    MissingTargetPart {
        /// `"footnotes"` or `"endnotes"`.
        kind: &'static str,
    },
}

impl std::fmt::Display for RectifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RectifyError::MissingNoteDef { id } => {
                write!(
                    f,
                    "Internal error: note definition id '{id}' not found in before ∪ after parts"
                )
            }
            RectifyError::MissingTargetPart { kind } => {
                write!(
                    f,
                    "Internal error: withRevisions {kind} part is absent, cannot write renumbered note definitions"
                )
            }
        }
    }
}

impl std::error::Error for RectifyError {}

/// The three roots the rectifier needs for one note kind (footnote or endnote):
/// the original `before`, the comparison `after`, and the freshly-built
/// `with_revisions` (where we keep separators and re-add the renumbered defs).
#[derive(Debug, Clone, Copy, Default)]
pub struct NotesSet {
    /// `before`.
    pub before: Option<NodeId>,
    /// `after`.
    pub after: Option<NodeId>,
    /// `with_revisions`.
    pub with_revisions: Option<NodeId>,
}

/// M4.H.6/B.3 — `RectifyFootnoteEndnoteIds` (:3309): in the withRevisions
/// notes part keep only the separators (-1/0), then re-add the *referenced*
/// definitions renumbered 1-based by reference order (after-part lookup wins
/// over before), and finalize each withRevisions part (:3433–:3450):
/// MarkContentAsDeletedOrInserted + CoalesceAdjacentRuns + order-per-standard
/// + IgnorePt14.
pub fn rectify_footnote_endnote_ids(
    dom: &mut Dom,
    main_root: NodeId,
    footnotes: NotesSet,
    endnotes: NotesSet,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> Result<(), RectifyError> {
    // Plan both note kinds first. Only after every lookup succeeds do we
    // touch the DOM, so a missing def or missing target part leaves the
    // document in its pre-call state.
    let fn_plan = plan_rectify(
        dom,
        main_root,
        &W::name("footnoteReference"),
        &W::footnote(),
        &footnotes,
    )?;
    let en_plan = plan_rectify(
        dom,
        main_root,
        &W::name("endnoteReference"),
        &W::endnote(),
        &endnotes,
    )?;
    apply_rectify(dom, &W::footnote(), &footnotes, &fn_plan);
    apply_rectify(dom, &W::endnote(), &endnotes, &en_plan);
    // B.3 — notes-part finalization (:3433–:3450), per withRevisions part.
    for wr in [footnotes.with_revisions, endnotes.with_revisions]
        .into_iter()
        .flatten()
    {
        finalize_notes_part(dom, wr, settings, id_gen);
    }
    Ok(())
}

/// The withRevisions notes-part finalization (C# :3433–:3450):
/// MarkContentAsDeletedOrInserted (pt:Status → real `w:ins`/`w:del`),
/// CoalesceAdjacentRunsWithIdenticalFormatting, WmlOrderElementsPerStandard
/// (our produce-path equivalents: pPr-first + paragraph-mark revision order),
/// IgnorePt14Namespace. MarkContent rebuilds the tree, so the result is
/// grafted back into the SAME root node — callers' `NodeId`s stay valid.
fn finalize_notes_part(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let rebuilt = super::finalize::mark_content_as_deleted_or_inserted(dom, root, settings, id_gen);
    if rebuilt != root {
        for c in dom.nodes(root) {
            dom.remove(c);
        }
        for c in dom.nodes(rebuilt) {
            dom.add(root, c);
        }
    }
    super::finalize::coalesce_all_paragraphs(dom, root);
    super::finalize::move_paragraph_properties_first(dom, root);
    super::finalize::fix_paragraph_mark_revision_order(dom, root);
    // C# finalization leaves pt:Status; we still strip Unid/other scratch so
    // notes parts don't ship powertools markup Word must ignore.
    super::finalize::remove_powertools_scratch_markup(dom, root);
    super::finalize::ignore_pt14_namespace(dom, root);
}

struct RectifyPlan {
    refs: Vec<NodeId>,
    defs: Vec<NodeId>,
}

fn plan_rectify(
    dom: &Dom,
    main_root: NodeId,
    ref_name: &XName,
    def_name: &XName,
    notes: &NotesSet,
) -> Result<RectifyPlan, RectifyError> {
    let refs = dom.descendants(main_root, Some(ref_name));
    let mut defs = Vec::with_capacity(refs.len());
    for r in &refs {
        let old = dom.attribute(*r, &W::id()).unwrap_or("").to_string();
        let def = lookup_def(dom, notes.after, def_name, &old)
            .or_else(|| lookup_def(dom, notes.before, def_name, &old));
        let def = def.ok_or(RectifyError::MissingNoteDef { id: old })?;
        defs.push(def);
    }
    // If references exist and the caller needs new defs written, the target
    // part must be present — otherwise the rewrite would leave dangling ids.
    if !refs.is_empty() && notes.with_revisions.is_none() {
        let kind = if def_name == &W::footnote() {
            "footnotes"
        } else {
            "endnotes"
        };
        return Err(RectifyError::MissingTargetPart { kind });
    }
    Ok(RectifyPlan { refs, defs })
}

fn lookup_def(dom: &Dom, root: Option<NodeId>, def_name: &XName, old: &str) -> Option<NodeId> {
    root.and_then(|rt| {
        dom.elements(rt, Some(def_name))
            .into_iter()
            .find(|&e| dom.attribute(e, &W::id()) == Some(old))
    })
}

/// Structural (non-content) note definitions that must survive rectify.
/// Separators use fixed ids -1/0; `continuationNotice` is commonly id=1 and is
/// listed from `w:settings`/`w:footnotePr` — dropping it while leaving the
/// settings reference triggers Word "unreadable content" (OpenXmlValidator
/// Semantic: settings references missing footnote/endnote id).
pub(crate) fn is_structural_note(dom: &Dom, note: NodeId) -> bool {
    match dom.attribute(note, &W::id()) {
        Some("-1") | Some("0") => return true,
        _ => {}
    }
    matches!(
        dom.attribute(note, &W::name("type")),
        Some("separator") | Some("continuationSeparator") | Some("continuationNotice")
    )
}

fn apply_rectify(dom: &mut Dom, def_name: &XName, notes: &NotesSet, plan: &RectifyPlan) {
    // strip non-structural notes from the withRevisions part (keep separators
    // and continuationNotice; drop stale content defs)
    if let Some(wr) = notes.with_revisions {
        for note in dom.elements(wr, Some(def_name)) {
            if !is_structural_note(dom, note) {
                dom.remove(note);
            }
        }
    }
    // Renumber content notes avoiding ids still held by structural notes
    // (e.g. continuationNotice at id=1). Fall back to the planned 1..n
    // sequence when nothing is reserved.
    let reserved: std::collections::HashSet<String> = notes
        .with_revisions
        .map(|wr| {
            dom.elements(wr, Some(def_name))
                .into_iter()
                .filter_map(|n| dom.attribute(n, &W::id()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let mut next = 1u32;
    let mut assigned: Vec<String> = Vec::with_capacity(plan.refs.len());
    for _ in 0..plan.refs.len() {
        while reserved.contains(&next.to_string()) {
            next += 1;
        }
        assigned.push(next.to_string());
        next += 1;
    }
    for (idx, r) in plan.refs.iter().enumerate() {
        dom.set_attribute_value(*r, &W::id(), Some(&assigned[idx]));
    }
    if let Some(wr) = notes.with_revisions {
        for (idx, def) in plan.defs.iter().enumerate() {
            let cloned = dom.clone_subtree(*def);
            dom.set_attribute_value(cloned, &W::id(), Some(&assigned[idx]));
            dom.add(wr, cloned);
        }
    }
}

/// Drop `w:settings`/`w:footnotePr`/`w:endnotePr` children that reference note
/// ids not present in the corresponding notes part. Rectify may remove
/// special notes (or never re-emit them); a dangling settings reference is a
/// Word repair-dialog trigger (validated by OpenXmlValidator Semantic).
pub fn sync_settings_special_note_ids(
    settings_dom: &mut Dom,
    settings_root: NodeId,
    footnote_ids: &std::collections::HashSet<String>,
    endnote_ids: &std::collections::HashSet<String>,
) {
    for (pr_name, child_name, existing) in [
        ("footnotePr", "footnote", footnote_ids),
        ("endnotePr", "endnote", endnote_ids),
    ] {
        let pr = W::name(pr_name);
        let child = W::name(child_name);
        for pr_el in settings_dom.descendants(settings_root, Some(&pr)) {
            let doomed: Vec<_> = settings_dom
                .elements(pr_el, Some(&child))
                .into_iter()
                .filter(|&e| {
                    let id = settings_dom.attribute(e, &W::id()).unwrap_or("");
                    !id.is_empty() && !existing.contains(id)
                })
                .collect();
            for e in doomed {
                settings_dom.remove(e);
            }
        }
    }
}

/// M4.H.7 — `FixUpFootnotesEndnotesWithCustomMarkers` (:2474/:2484): a custom-marker
/// footnote/endnote reference's delText/t must sit in the SAME run as the
/// reference. For each `footnoteReference`/`endnoteReference[@customMarkFollows]`
/// inside a `w:r` whose grandparent is `w:del`/`w:ins`, pull the delText/t of the
/// run in the next sibling element into the reference's run. In-place.
pub fn fix_up_footnotes_endnotes_with_custom_markers(dom: &mut Dom, root: NodeId) {
    let cmf = W::name("customMarkFollows");
    let refs: Vec<NodeId> = [W::name("footnoteReference"), W::name("endnoteReference")]
        .iter()
        .flat_map(|n| dom.descendants(root, Some(n)))
        .filter(|&r| dom.attribute(r, &cmf).is_some())
        .collect();
    for fnenr in refs {
        let Some(par) = dom.parent(fnenr) else {
            continue;
        };
        let Some(gp) = dom.parent(par) else { continue };
        if dom.name(par) != Some(W::r()) {
            continue;
        }
        let gpn = dom.name(gp);
        let (is_del, leaf) = if gpn == Some(W::del()) {
            (true, W::name("delText"))
        } else if gpn == Some(W::ins()) {
            (false, W::t())
        } else {
            continue;
        };
        // already has the text leaf in the same run → nothing to do
        if dom.element(par, &leaf).is_some() {
            continue;
        }
        let Some(after_gp) = dom.next_element(gp) else {
            continue;
        };
        // Only the matching revision wrapper is the marker source; an unrelated
        // sibling must not have its content stripped/reattached to the
        // reference's run.
        let expected_gp = if is_del { W::del() } else { W::ins() };
        if dom.name(after_gp) != Some(expected_gp) {
            continue;
        }
        // Move a single marker-bearing leaf (the first one) so unrelated tracked
        // runs in the same `w:del`/`w:ins` are left intact.
        let Some(lf) = dom
            .elements(after_gp, Some(&W::r()))
            .into_iter()
            .find_map(|r| dom.element(r, &leaf))
        else {
            continue;
        };
        let cloned = dom.clone_subtree(lf);
        dom.add(par, cloned);
        dom.remove(lf);
    }
}

/// Look up a note definition by id — C# `.Elements().FirstOrDefault(e =>
/// (string)e.Attribute(W.id) == id)`: ANY child element name, and a missing
/// attribute matches a missing id (null == null).
fn note_def_by_id(dom: &Dom, root: NodeId, id: Option<&str>) -> Option<NodeId> {
    dom.elements(root, None)
        .into_iter()
        .find(|&e| dom.attribute(e, &W::id()) == id)
}

/// C# `ReplaceNodes`: clear `target`'s children, then append clones of
/// `source`'s children.
fn replace_nodes(dom: &mut Dom, target: NodeId, source: NodeId) {
    for c in dom.nodes(target) {
        dom.remove(c);
    }
    for c in dom.nodes(source) {
        dom.add(target, c);
    }
}

/// The reference-marker guarantee (C# :3071–:3104, identical in all three
/// branches): if the produced note content has no marker run, prepend a
/// kind-correct one before the first run of the first paragraph.
/// FAITHFUL-BUG: the *presence check* probes only the FOOTNOTE marker
/// (rStyle "FootnoteReference" or a `w:footnoteRef` descendant) even when
/// processing an endnote; only the inserted run is kind-correct.
/// `direct_child_run` mirrors the Equal branch's `firstPara.Element(W.r)` vs
/// the Inserted/Deleted branches' `Descendants(W.r).FirstOrDefault()`.
fn ensure_reference_marker(dom: &mut Dom, temp: NodeId, is_footnote: bool, direct_child_run: bool) {
    let r_name = W::r();
    let has_marker = dom.descendants(temp, Some(&r_name)).into_iter().any(|run| {
        let style = dom
            .elements(run, Some(&W::r_pr()))
            .into_iter()
            .flat_map(|rpr| dom.elements(rpr, Some(&W::name("rStyle"))))
            .find_map(|st| dom.attribute(st, &W::val()));
        style == Some("FootnoteReference")
            || !dom
                .descendants(run, Some(&W::name("footnoteRef")))
                .is_empty()
    });
    if has_marker {
        return;
    }
    let Some(first_para) = dom.descendants(temp, Some(&W::p())).first().copied() else {
        return;
    };
    let first_run = if direct_child_run {
        dom.element(first_para, &r_name)
    } else {
        dom.descendants(first_para, Some(&r_name)).first().copied()
    };
    let Some(first_run) = first_run else {
        return;
    };
    let (style_val, ref_el) = if is_footnote {
        ("FootnoteReference", "footnoteRef")
    } else {
        ("EndnoteReference", "endnoteRef")
    };
    let marker = dom.new_element(W::r());
    let rpr = dom.new_element(W::r_pr());
    let rstyle = dom.new_element(W::name("rStyle"));
    dom.set_attribute_value(rstyle, &W::val(), Some(style_val));
    dom.add(rpr, rstyle);
    dom.add(marker, rpr);
    let re = dom.new_element(W::name(ref_el));
    dom.add(marker, re);
    dom.add_before_self(first_run, marker);
}

/// Shared tail of the three ProcessFootnoteEndnote branches: assemble unids,
/// produce the redlined markup into a temp `w:body`, guarantee the reference
/// marker, order per standard (our port's produce-path equivalents:
/// pPr-first + paragraph-mark revision order — `WmlOrderElementsPerStandard`
/// has no direct port), and return the rebuilt `w:footnote`/`w:endnote`.
fn produce_note_redline(
    dom: &mut Dom,
    flat: &mut [super::atoms::ComparisonUnitAtom],
    is_footnote: bool,
    direct_child_run: bool,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> Option<NodeId> {
    super::produce::assemble_ancestor_unids(dom, flat);
    let children = super::produce::produce_new_wml_markup_from_correlated_sequence(
        dom, flat, settings, id_gen,
    );
    let temp = dom.new_element(W::body());
    for c in children {
        dom.add(temp, c);
    }
    ensure_reference_marker(dom, temp, is_footnote, direct_child_run);
    super::finalize::move_paragraph_properties_first(dom, temp);
    super::finalize::fix_paragraph_mark_revision_order(dom, temp);
    let fn_name = W::footnote();
    let en_name = W::endnote();
    dom.descendants(temp, None)
        .into_iter()
        .find(|&d| dom.name(d).is_some_and(|n| n == fn_name || n == en_name))
}

/// B.2 — `ProcessFootnoteEndnote` (:2944): for every footnote/endnote
/// reference atom in the correlated body, process its definition keyed by the
/// REFERENCE's correlation status — Equal → nested mini-compare of the two
/// definitions written into the AFTER definition; Inserted → after-definition
/// content re-emitted all-Inserted; Deleted → before-definition content
/// re-emitted all-Deleted (into the BEFORE definition). Definitions carry
/// pt:Status-marked content after this; real `w:ins`/`w:del` arrive with the
/// notes-part finalization (B.3). Any other status panics — C# throws
/// "Internal error" (a real crash path for moved/format-changed references).
pub fn process_footnote_endnote(
    dom: &mut Dom,
    atoms: &[super::atoms::ComparisonUnitAtom],
    notes: &mut super::NotesContext,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    use super::CorrelationStatus;
    use super::atoms::CorrelatedSequence;
    use super::{atomize, lcs, lcs_table, preprocess, produce, units};

    let fn_ref = W::name("footnoteReference");
    let en_ref = W::name("endnoteReference");
    let candidates: Vec<(NodeId, Option<NodeId>, CorrelationStatus)> = atoms
        .iter()
        .filter(|a| {
            dom.name(a.content_element)
                .is_some_and(|n| n == fn_ref || n == en_ref)
        })
        .map(|a| {
            (
                a.content_element,
                a.content_element_before,
                a.correlation_status,
            )
        })
        .collect();

    for (content, content_before, status) in candidates {
        let before_id = content_before
            .and_then(|e| dom.attribute(e, &W::id()))
            .map(str::to_string);
        let after_id = dom.attribute(content, &W::id()).map(str::to_string);
        let is_footnote = dom.name(content).is_some_and(|n| n == fn_ref);

        match status {
            CorrelationStatus::Equal => {
                let (before_root, after_root) = if is_footnote {
                    (
                        notes
                            .fn_before
                            .expect("footnotes part missing in before document (C# NRE)"),
                        notes
                            .fn_after
                            .expect("footnotes part missing in after document (C# NRE)"),
                    )
                } else {
                    (
                        notes
                            .en_before
                            .expect("endnotes part missing in before document (C# NRE)"),
                        notes
                            .en_after
                            .expect("endnotes part missing in after document (C# NRE)"),
                    )
                };
                let def_before = note_def_by_id(dom, before_root, before_id.as_deref())
                    .expect("before note definition not found (C# NRE in AddSha1Hash)");
                let def_after = note_def_by_id(dom, after_root, after_id.as_deref())
                    .expect("after note definition not found (C# NRE in AddSha1Hash)");
                preprocess::add_sha1_hash_to_block_level_content(
                    dom,
                    def_before,
                    settings,
                    &preprocess::null_rel_resolver,
                );
                preprocess::add_sha1_hash_to_block_level_content(
                    dom,
                    def_after,
                    settings,
                    &preprocess::null_rel_resolver,
                );
                let fncal1 = atomize::create_comparison_unit_atom_list(dom, def_before, settings);
                let fncus1 = units::get_comparison_unit_list(dom, &fncal1, settings);
                let fncal2 = atomize::create_comparison_unit_atom_list(dom, def_after, settings);
                let fncus2 = units::get_comparison_unit_list(dom, &fncal2, settings);
                if !(fncus1.is_empty() && fncus2.is_empty()) {
                    // C# calls Lcs directly — no DetectUnrelatedSources
                    // pre-check in the nested path (:3060), and no move /
                    // format-change detection either.
                    let seqs = lcs::lcs(dom, fncus1, fncus2, settings);
                    lcs_table::mark_rows_as_deleted_or_inserted(dom, settings, &seqs, id_gen);
                    let mut flat = produce::flatten_to_comparison_unit_atom_list(dom, &seqs);
                    let new_content =
                        produce_note_redline(dom, &mut flat, is_footnote, true, settings, id_gen)
                            .expect("Internal error");
                    replace_nodes(dom, def_after, new_content);
                }
            }
            CorrelationStatus::Inserted => {
                let after_root = if is_footnote {
                    notes
                        .fn_after
                        .expect("footnotes part missing in after document (C# NRE)")
                } else {
                    notes
                        .en_after
                        .expect("endnotes part missing in after document (C# NRE)")
                };
                let def_after = note_def_by_id(dom, after_root, after_id.as_deref())
                    .expect("after note definition not found (C# NRE in AddSha1Hash)");
                preprocess::add_sha1_hash_to_block_level_content(
                    dom,
                    def_after,
                    settings,
                    &preprocess::null_rel_resolver,
                );
                let fncal2 = atomize::create_comparison_unit_atom_list(dom, def_after, settings);
                let fncus2 = units::get_comparison_unit_list(dom, &fncal2, settings);
                let seqs = vec![CorrelatedSequence::inserted(fncus2)];
                lcs_table::mark_rows_as_deleted_or_inserted(dom, settings, &seqs, id_gen);
                let mut flat = produce::flatten_to_comparison_unit_atom_list(dom, &seqs);
                // C# tolerates a missing rebuilt definition here (the throw is
                // commented out :3202) — unlike the Equal/Deleted branches.
                if let Some(new_content) =
                    produce_note_redline(dom, &mut flat, is_footnote, false, settings, id_gen)
                {
                    replace_nodes(dom, def_after, new_content);
                }
            }
            CorrelationStatus::Deleted => {
                // C# (:3210) — the before-part lookup is keyed by the local
                // var misnamed `afterId` (correct for Deleted: ContentElement
                // IS the before-document element). FAITHFUL-BUG: the footnote
                // sub-branch sets partToUseAfter and leaves partToUseBefore
                // null — the part is only consulted for related-part (image)
                // resolution, which our port services with the null resolver,
                // so the null part is inert here.
                let before_root = if is_footnote {
                    notes
                        .fn_before
                        .expect("footnotes part missing in before document (C# NRE)")
                } else {
                    notes
                        .en_before
                        .expect("endnotes part missing in before document (C# NRE)")
                };
                let def_before = note_def_by_id(dom, before_root, after_id.as_deref())
                    .expect("before note definition not found (C# NRE in AddSha1Hash)");
                preprocess::add_sha1_hash_to_block_level_content(
                    dom,
                    def_before,
                    settings,
                    &preprocess::null_rel_resolver,
                );
                let fncal2 = atomize::create_comparison_unit_atom_list(dom, def_before, settings);
                let fncus2 = units::get_comparison_unit_list(dom, &fncal2, settings);
                let seqs = vec![CorrelatedSequence::deleted(fncus2)];
                lcs_table::mark_rows_as_deleted_or_inserted(dom, settings, &seqs, id_gen);
                let mut flat = produce::flatten_to_comparison_unit_atom_list(dom, &seqs);
                // Tolerate missing rebuild (Inserted branch already does) — after
                // ATOM-STACK path fix the note wrapper is present; keep soft fail
                // so a future path bug does not panic the whole compare.
                if !flat.is_empty()
                    && let Some(new_content) =
                        produce_note_redline(dom, &mut flat, is_footnote, false, settings, id_gen)
                {
                    replace_nodes(dom, def_before, new_content);
                }
            }
            _ => panic!("Internal error"),
        }
    }
}
