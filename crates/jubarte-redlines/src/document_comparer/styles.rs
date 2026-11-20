// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Style-sheet reconciliation: Normal spacing, style chains and records.

use super::*;

/// The `w:style[@w:type='paragraph' @w:styleId='Normal']` element under a
/// styles root, falling back to the `w:default="1"` paragraph style.
pub(super) fn find_normal_style(dom: &Dom, styles_root: NodeId) -> Option<NodeId> {
    let styles: Vec<NodeId> = dom
        .elements(styles_root, Some(&W::name("style")))
        .into_iter()
        .filter(|&s| dom.attribute(s, &W::type_()) == Some("paragraph"))
        .collect();
    styles
        .iter()
        .copied()
        .find(|&s| dom.attribute(s, &W::style_id()) == Some("Normal"))
        .or_else(|| {
            styles
                .iter()
                .copied()
                .find(|&s| dom.attribute(s, &W::name("default")) == Some("1"))
        })
        // LibreOffice writes `style0` named "Normal" and no default flag;
        // Word still treats it as Normal.
        .or_else(|| {
            styles.into_iter().find(|&s| {
                dom.element(s, &W::name("name"))
                    .and_then(|n| dom.attribute(n, &W::val()))
                    .is_some_and(|v| v.eq_ignore_ascii_case("Normal"))
            })
        })
}

/// A style's `pPr/spacing` as (after, line, lineRule), each None when absent.
fn normal_spacing(dom: &Dom, style: NodeId) -> Option<(String, String, String)> {
    let ppr = dom.element(style, &W::p_pr())?;
    let sp = dom.element(ppr, &W::spacing_el())?;
    let get = |n: &str| dom.attribute(sp, &W::name(n)).unwrap_or("").to_string();
    Some((get("after"), get("line"), get("lineRule")))
}

/// `before` companion to [`normal_spacing`]/[`docdefaults_ppr_spacing`] —
/// M479: the Normal-merge truth table was derived over {after, line} only;
/// an A-docDefaults `before` (paragraph_spacing_missing: before=240) leaked
/// through unneutralized, adding 12pt above every Normal paragraph. Word
/// writes the explicit neutralizer (oracle Normal: before="0").
fn spacing_before_attr(dom: &Dom, sp_holder: Option<NodeId>) -> Option<String> {
    let sp = sp_holder?;
    dom.attribute(sp, &W::name("before")).map(str::to_string)
}

/// Word's own paragraph spacing, which it applies to a document with no
/// `w:docDefaults` at all (a LibreOffice export): after 160, line 278
/// (multi_section_nested_table_rowspan, table_bookmark_end ×
/// table_vmerge_colspan — both Word redlines write it into Normal).
const WORD_FACTORY_SPACING: (&str, &str, &str) = ("160", "278", "auto");

/// B's `docdefaults_ppr_spacing`, with Word's factory spacing standing in
/// when B has no `w:docDefaults` at all.
fn b_docdefaults_ppr_spacing(dom: &Dom, b_root: NodeId) -> Option<(String, String, String)> {
    if dom.element(b_root, &W::name("docDefaults")).is_none() {
        let (a, l, r) = WORD_FACTORY_SPACING;
        return Some((a.to_string(), l.to_string(), r.to_string()));
    }
    docdefaults_ppr_spacing(dom, b_root)
}

/// The stylesheet's `docDefaults/pPrDefault/pPr/spacing` as
/// (after, line, lineRule), each "" when absent. Used only when B's Normal
/// style has no stored spacing *and* A had stored spacing to rewrite (Word
/// promotes B's docDefaults into Normal in that case — file_197_file_198).
fn docdefaults_ppr_spacing(dom: &Dom, styles_root: NodeId) -> Option<(String, String, String)> {
    let dd = dom.element(styles_root, &W::name("docDefaults"))?;
    let pd = dom.element(dd, &W::name("pPrDefault"))?;
    let ppr = dom.element(pd, &W::p_pr())?;
    let sp = dom.element(ppr, &W::spacing_el())?;
    let get = |n: &str| dom.attribute(sp, &W::name(n)).unwrap_or("").to_string();
    Some((get("after"), get("line"), get("lineRule")))
}

/// M487 — effective paragraph spacing for `style_id` under one stylesheet:
/// (after, before, line, lineRule), each resolved through the basedOn chain
/// then docDefaults, with OOXML implicit defaults materialized ("0", "0",
/// "240", "auto") so two stylesheets always compare attr-by-attr.
///
/// In a table, `table_style`'s pPr sits between the two, as Word applies
/// it: over the default paragraph style's chain, under any other style's
/// chain (b4cd671041: a Table Grid cell's Normal paragraph shows the
/// table's `after=0 line=240` over Normal's 200/276; a List Paragraph
/// based on that Normal shows 200/276).
///
/// Also returns, per attr, whether B declares it in the style chain or its
/// docDefaults, and whether the table style supplied it.
pub(super) fn effective_para_spacing(
    dom: &Dom,
    styles_root: NodeId,
    by_id: &std::collections::HashMap<String, NodeId>,
    style_id: &str,
    table_style: Option<&str>,
) -> ([String; 4], [bool; 4], [bool; 4]) {
    let attr_names = ["after", "before", "line", "lineRule"];
    let chain_vals = |start: Option<NodeId>| -> [Option<String>; 4] {
        let mut vals: [Option<String>; 4] = [None, None, None, None];
        let mut cur = start;
        for _ in 0..12 {
            let Some(s) = cur else { break };
            if let Some(ppr) = dom.element(s, &W::p_pr())
                && let Some(sp) = dom.element(ppr, &W::spacing_el())
            {
                for (i, n) in attr_names.iter().enumerate() {
                    if vals[i].is_none()
                        && let Some(v) = dom.attribute(sp, &W::name(n))
                    {
                        vals[i] = Some(v.to_string());
                    }
                }
            }
            cur = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))
                .and_then(|v| by_id.get(v).copied());
        }
        vals
    };
    let style = by_id.get(style_id).copied();
    let para = chain_vals(style);
    let table = chain_vals(table_style.and_then(|t| by_id.get(t).copied()));
    let mut dd: [Option<String>; 4] = [None, None, None, None];
    if let Some(d) = dom.element(styles_root, &W::name("docDefaults"))
        && let Some(pd) = dom.element(d, &W::name("pPrDefault"))
        && let Some(ppr) = dom.element(pd, &W::p_pr())
        && let Some(sp) = dom.element(ppr, &W::spacing_el())
    {
        for (i, n) in attr_names.iter().enumerate() {
            dd[i] = dom.attribute(sp, &W::name(n)).map(str::to_string);
        }
    }
    let is_default = style.is_some_and(|s| {
        dom.attribute(s, &W::type_()) == Some("paragraph")
            && matches!(dom.attribute(s, &W::name("default")), Some("1" | "true"))
    });
    // provenance: declared = SOMEWHERE in B's style chain or dd.
    // Word bakes an attr onto inserted paragraphs ONLY when B is entirely
    // silent on it — the paragraph's look is the OOXML implicit default and
    // the output's dd would override it (rstyle_combos: implicit 0/240 IS
    // baked) — or when B's table style gave it (b4cd671041). Values B
    // declares — even in its dd — are otherwise never baked (m370:
    // dd-declared after=200/line=276 stays off the pure-I title).
    let defaults = ["0", "0", "240", "auto"];
    let mut vals: [String; 4] = std::array::from_fn(|i| defaults[i].to_string());
    let mut declared = [false; 4];
    let mut from_table = [false; 4];
    for i in 0..4 {
        let first = if is_default {
            table[i]
                .as_ref()
                .map(|v| (v, true))
                .or(para[i].as_ref().map(|v| (v, false)))
        } else {
            para[i]
                .as_ref()
                .map(|v| (v, false))
                .or(table[i].as_ref().map(|v| (v, true)))
        };
        if let Some((v, t)) = first.or(dd[i].as_ref().map(|v| (v, false))) {
            vals[i] = v.clone();
            from_table[i] = t;
            declared[i] = !t;
        }
    }
    (vals, declared, from_table)
}

/// Revision record element local names that carry a `w:id` identifying the
/// change, including list, move-range and table-cell records. Word treats a colliding id on any of these as the same revision
/// record and drops the later one, so a newly synthesized `w:*Change` must not
/// reuse an id already present in the stylesheet.
const REVISION_CHANGE_ELEMENTS: &[&str] = &[
    "pPrChange",
    "rPrChange",
    "sectPrChange",
    "tblPrChange",
    "tblGridChange",
    "trPrChange",
    "tcPrChange",
    "ins",
    "del",
    "moveFrom",
    "moveTo",
    "moveFromRangeStart",
    "moveFromRangeEnd",
    "moveToRangeStart",
    "moveToRangeEnd",
    "numberingChange",
    "cellIns",
    "cellDel",
    "cellMerge",
];

/// The next free revision id under `styles_root`: one greater than the maximum
/// numeric `w:id` on any revision record present (0 when none).
/// `merge_normal_style_spacing`/`merge_normal_style_rpr` synthesize at most one
/// `w:pPrChange` and one `w:rPrChange` per compare, so a single starting id is
/// reserved here; the rPr pass bumps by one when it fires after the pPr pass.
pub(super) fn next_free_revision_id(dom: &Dom, styles_root: NodeId) -> u32 {
    let mut max: u32 = 0;
    for change in REVISION_CHANGE_ELEMENTS {
        for c in dom.descendants(styles_root, Some(&W::name(change))) {
            if let Some(v) = dom.attribute(c, &W::id())
                && let Ok(n) = v.parse::<u32>()
            {
                max = max.max(n);
            }
        }
    }
    max.saturating_add(1)
}

/// The `w:id` of a new `w:pPrChange`/`w:rPrChange` under `block`. A style
/// whose own `w:pPr`/`w:rPr` already carries the other record from this
/// comparison lends it its id: Word gives a style's two records one id (199
/// of 199 corpus styles carrying both), so the pair is one revision.
/// Otherwise the stylesheet's next free id. "From this comparison" is read
/// as the comparison's author and date, so a source record stamped exactly
/// so (a redline compared again under the same pinned stamp) lends its id
/// too.
fn style_record_id(
    dom: &Dom,
    styles_root: NodeId,
    block: NodeId,
    settings: &WmlComparerSettings,
) -> String {
    let ours = |c: NodeId| {
        dom.attribute(c, &W::author()) == Some(settings.author_for_revisions.as_str())
            && dom.attribute(c, &W::date()) == Some(settings.date_time_for_revisions.as_str())
    };
    dom.parent(block)
        .filter(|&style| dom.name_is(style, &W::name("style")))
        .and_then(|style| {
            [("pPr", "pPrChange"), ("rPr", "rPrChange")]
                .into_iter()
                .find_map(|(own, record)| {
                    let own = dom.element(style, &W::name(own))?;
                    let c = dom.element(own, &W::name(record)).filter(|&c| ours(c))?;
                    dom.attribute(c, &W::id()).map(str::to_string)
                })
        })
        .unwrap_or_else(|| next_free_revision_id(dom, styles_root).to_string())
}

/// `w:pPr` on/off children whose absence reads as off, so "0" neutralizes them.
const PPR_ON_OFF: &[&str] = &[
    "keepNext",
    "keepLines",
    "pageBreakBefore",
    "widowControl",
    "suppressLineNumbers",
    "suppressAutoHyphens",
    "kinsoku",
    "wordWrap",
    "overflowPunct",
    "topLinePunct",
    "autoSpaceDE",
    "autoSpaceDN",
    "bidi",
    "adjustRightInd",
    "snapToGrid",
    "contextualSpacing",
    "mirrorIndents",
    "suppressOverlap",
];

/// `w:pPr` on/off children a document that never declares them reads as on
/// (Word's defaults): an original that turns one off in its docDefaults,
/// against a revision silent on it, gets it written on in Normal (38 of the
/// 470 Word redlines whose original alone declares one, 440c36d875).
const PPR_DEFAULT_ON: &[&str] = &["widowControl", "autoSpaceDE", "autoSpaceDN"];

/// An on/off element set to off.
pub(super) fn on_off_is_off(dom: &Dom, el: NodeId) -> bool {
    matches!(dom.attribute(el, &W::val()), Some("0" | "false" | "off"))
}

/// What [`doc_default_ppr_delta`] writes into the live Normal pPr.
pub(super) struct DocDefaultPprDelta {
    /// Whole pPr children, by local name.
    pub(super) elements: Vec<(String, NodeId)>,
    /// Extra `w:spacing` attributes (`beforeLines`/`afterLines`).
    pub(super) spacing: Vec<(String, String)>,
}

/// The docDefaults paragraph properties the after/before/line rule of
/// [`merge_normal_style_spacing`] does not cover, as Word writes them into the
/// live Normal. The redline keeps A's docDefaults, so B's own paragraph
/// defaults would be lost; Word stores B's effective value (B Normal, else
/// B docDefaults) wherever it differs from A's docDefaults, and neutralizes
/// what only A declares (file_103 × file_104 and the reverse file_104 ×
/// file_105: `ind`, `jc`, `beforeLines`/`afterLines`; sd_1494 ×
/// sdpr_titleonly: a `nil` edge per A border).
///
/// A property with no known neutral value is left alone.
pub(super) fn doc_default_ppr_delta(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
    b_style: Option<NodeId>,
) -> DocDefaultPprDelta {
    let dd_ppr = |dom: &Dom, root: NodeId| -> Option<NodeId> {
        let dd = dom.element(root, &W::name("docDefaults"))?;
        let pd = dom.element(dd, &W::name("pPrDefault"))?;
        dom.element(pd, &W::p_pr())
    };
    let a_dd = dd_ppr(dom, out_root);
    let b_dd = dd_ppr(dom, b_root);
    let b_own = b_style.and_then(|s| dom.element(s, &W::p_pr()));
    let child = |dom: &Dom, ppr: Option<NodeId>, local: &str| {
        ppr.and_then(|p| dom.element(p, &W::name(local)))
    };
    let mut names: Vec<String> = Vec::new();
    for ppr in [a_dd, b_dd].into_iter().flatten() {
        for c in dom.elements(ppr, None) {
            let Some(n) = dom.name(c) else { continue };
            let local = n.local_name().to_string();
            if n.namespace_name() == W::URI
                && !matches!(local.as_str(), "spacing" | "rPr" | "pPrChange" | "sectPr")
                && !names.contains(&local)
            {
                names.push(local);
            }
        }
    }
    let mut elements = Vec::new();
    for local in names {
        let a = child(dom, a_dd, &local);
        let b = child(dom, b_own, &local).or_else(|| child(dom, b_dd, &local));
        if a.map(|n| style_prop_signature(dom, n)) == b.map(|n| style_prop_signature(dom, n)) {
            continue;
        }
        let el = match (b, a) {
            (Some(b), _) => dom.clone_subtree(b),
            (None, Some(a)) => {
                let el = dom.new_element(W::name(&local));
                match local.as_str() {
                    "ind" => {
                        for (n, _) in dom.attributes(a) {
                            if n.namespace_name() == W::URI {
                                dom.set_attribute_value(el, &n, Some("0"));
                            }
                        }
                    }
                    "jc" => {
                        dom.set_attribute_value(el, &W::val(), Some("left"));
                    }
                    // Word's default font alignment (440c36d875: baseline
                    // in the original's docDefaults, auto in Word's Normal).
                    "textAlignment" => {
                        dom.set_attribute_value(el, &W::val(), Some("auto"));
                    }
                    l if PPR_DEFAULT_ON.contains(&l) => {
                        if !on_off_is_off(dom, a) {
                            continue; // already the default
                        }
                    }
                    "pBdr" => {
                        for edge in dom.elements(a, None) {
                            let Some(n) = dom.name(edge) else { continue };
                            let e = dom.new_element(n);
                            dom.set_attribute_value(e, &W::val(), Some("nil"));
                            dom.add(el, e);
                        }
                    }
                    l if PPR_ON_OFF.contains(&l) => {
                        if on_off_is_off(dom, a) {
                            continue; // already the default
                        }
                        dom.set_attribute_value(el, &W::val(), Some("0"));
                    }
                    _ => continue,
                }
                el
            }
            (None, None) => continue,
        };
        elements.push((local, el));
    }
    let sp = |dom: &Dom, ppr: Option<NodeId>| child(dom, ppr, "spacing");
    let (a_sp, b_sp, b_own_sp) = (sp(dom, a_dd), sp(dom, b_dd), sp(dom, b_own));
    let mut attrs: Vec<(String, String)> = Vec::new();
    for s in [a_sp, b_sp].into_iter().flatten() {
        for (n, _) in dom.attributes_iter(s) {
            let local = n.local_name();
            if n.namespace_name() == W::URI
                && matches!(local, "beforeLines" | "afterLines")
                && !attrs.iter().any(|(k, _)| k == local)
            {
                attrs.push((local.to_string(), String::new()));
            }
        }
    }
    let get = |dom: &Dom, s: Option<NodeId>, n: &str| {
        s.and_then(|s| dom.attribute(s, &W::name(n)))
            .map(str::to_string)
    };
    attrs.retain_mut(|(name, v)| {
        let b = get(dom, b_own_sp, name).or_else(|| get(dom, b_sp, name));
        if b == get(dom, a_sp, name) {
            return false;
        }
        *v = b.unwrap_or_else(|| "0".to_string());
        true
    });
    DocDefaultPprDelta {
        elements,
        spacing: attrs,
    }
}

/// M64/M70/M72: after the spacing merge, Normal's other declared pPr
/// children follow the revision — B's `w:ind` replaces A's, A-only children
/// drop into the pPrChange record, B's own children are copied in schema
/// order.
pub(super) fn sync_normal_ppr_with_revised(dom: &mut Dom, ppr: NodeId, b_style: Option<NodeId>) {
    // M64/M70: Normal `w:ind` follows B.
    // - B has ind (file_196 firstLine=432) → copy B's ind onto merged Normal.
    // - B has no ind (file_197 bare Normal) → drop A's leftover firstLine so
    //   Word's after/line-only Normal is not polluted with A ind.
    let b_ind = b_style
        .and_then(|bs| dom.element(bs, &W::p_pr()))
        .and_then(|bppr| dom.element(bppr, &W::name("ind")));
    if let Some(old_ind) = dom.element(ppr, &W::name("ind")) {
        dom.remove(old_ind);
    }
    if let Some(bind) = b_ind {
        let clone = dom.clone_subtree(bind);
        // ind follows spacing in CT_PPr; place before pPrChange (added below).
        if let Some(sp) = dom.element(ppr, &W::spacing_el()) {
            let after_sp = {
                let kids = dom.nodes(ppr);
                kids.iter()
                    .position(|&n| n == sp)
                    .and_then(|i| kids.get(i + 1).copied())
            };
            match after_sp {
                Some(next) => dom.add_before_self(next, clone),
                None => dom.add(ppr, clone),
            }
        } else {
            dom.add_first(ppr, clone);
        }
    }
    // M72 refined by file_198: Word's live Normal after the merge is B's
    // EFFECTIVE block — spacing (computed above) plus B's OWN declared
    // pPr children. A-origin non-spacing props (widowControl/tabs/
    // suppressAutoHyphens on file_77, where B declares none) drop into
    // pPrChange old; B-origin ones (the same names on file_198's
    // LO-flavored B Normal) SURVIVE live — dropping them loses widow
    // control and drifts pagination.
    let keep: &[&str] = if b_ind.is_some() {
        &["spacing", "ind", "pPrChange"]
    } else {
        &["spacing", "pPrChange"]
    };
    let drop: Vec<NodeId> = dom
        .elements(ppr, None)
        .into_iter()
        .filter(|&c| {
            let Some(n) = dom.name(c) else {
                return false;
            };
            !keep.iter().any(|k| n == W::name(k))
        })
        .collect();
    for c in drop {
        dom.remove(c);
    }
    // Copy B Normal's remaining declared pPr children (widowControl,
    // tabs, suppressAutoHyphens, …) in schema order.
    if let Some(bppr) = b_style.and_then(|bs| dom.element(bs, &W::p_pr())) {
        let skip = ["spacing", "ind", "pPrChange", "rPr"];
        let b_kids: Vec<NodeId> = dom.elements(bppr, None);
        for bc in b_kids {
            let Some(n) = dom.name(bc) else { continue };
            if skip.iter().any(|s| n == W::name(s)) {
                continue;
            }
            let local = n.local_name().to_string();
            if dom.element(ppr, &W::name(&local)).is_some() {
                continue;
            }
            let clone = dom.clone_subtree(bc);
            insert_child_by_rank(dom, ppr, clone, &local, &ppr_child_rank);
        }
    }
}

/// M-PAG mechanism 2: rewrite the output stylesheet's Normal to B's target
/// spacing with a `w:pPrChange` holding A's old pPr. Returns true when the
/// stylesheet was modified.
///
/// Word Compare rules (broken_ones_two evidence), when neither side stores
/// Normal **spacing** (M60 refined — bare vs structured Normal):
/// - **Same docDefaults both sides** → leave Normal empty (file_8).
/// - **Differing docDefaults** → write B's dd **only if** B's Normal has pPr
///   or rPr (file_46 / 76 / 198). Both bare → leave empty (file_19 / 18 / 169).
/// - **A has dd, B has none** → single-line after=0 line=240 **only if** either
///   Normal is structured (A/B pPr or rPr: file_77 / 33 / 103). Both bare →
///   leave empty (file_145 / 68 / 13) — promoting 0/240 page-bloats LO.
/// - **A bare Normal (no pPr/rPr), B has dd** → leave empty (file_69 / 100).
/// - **A Normal has rPr or non-spacing pPr, B has dd** → write B's dd
///   (file_34 / 104).
/// - When A stores Normal spacing and B does not → B cascade (dd or factory 160/278).
/// - When B stores Normal spacing → B's stored values (file_21).
pub(super) fn merge_normal_style_spacing(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
    settings: &WmlComparerSettings,
) -> bool {
    let Some(a_style) = find_normal_style(dom, out_root) else {
        return false;
    };
    let stored = |dom: &Dom, style: Option<NodeId>| -> Option<(String, String, String)> {
        let (a, l, r) = style.and_then(|s| normal_spacing(dom, s))?;
        if a.is_empty() && l.is_empty() {
            None
        } else {
            Some((a, l, r))
        }
    };
    let dd_val = |dom: &Dom, styles_root: NodeId| -> Option<(String, String, String)> {
        let (a, l, r) = docdefaults_ppr_spacing(dom, styles_root)?;
        if a.is_empty() && l.is_empty() {
            None
        } else {
            Some((a, l, r))
        }
    };
    let a_stored = stored(dom, Some(a_style));
    let b_style = find_normal_style(dom, b_root);
    let b_stored = stored(dom, b_style);
    let a_dd = dd_val(dom, out_root);
    let b_dd =
        b_docdefaults_ppr_spacing(dom, b_root).filter(|(a, l, _)| !a.is_empty() || !l.is_empty());
    let a_normal_has_rpr = dom.element(a_style, &W::r_pr()).is_some();
    let a_normal_has_ppr = dom.element(a_style, &W::p_pr()).is_some();
    let b_normal_has_rpr = b_style
        .map(|s| dom.element(s, &W::r_pr()).is_some())
        .unwrap_or(false);
    let b_normal_has_ppr = b_style
        .map(|s| dom.element(s, &W::p_pr()).is_some())
        .unwrap_or(false);
    // Word only materializes dd into Normal when a side's Normal already
    // carries structure (pPr/rPr). Bare Normal + bare Normal → leave empty
    // even when docDefaults differ (file_19) or A alone has dd (file_145).
    let a_structured = a_normal_has_ppr || a_normal_has_rpr;
    let b_structured = b_normal_has_ppr || b_normal_has_rpr;
    // `None` target = clear explicit Normal spacing (Word leaves empty pPr and
    // lets docDefaults drive layout — file_22 when B cascade == shared dd).
    // M106 (file_7/5/130): identical dd, A bare Normal, B structured (rPr
    // Aptos) → Word still emits empty live Normal pPr + pPrChange(old = dd
    // spacing 200/276) alongside rPrChange. file_8 both bare / no rPr on B
    // → leave empty (return false below).
    let m106_same_dd_clear = matches!(
        (&a_stored, &b_stored, &a_dd, &b_dd),
        (None, None, Some(a), Some(b)) if a == b
    ) && b_structured
        && !a_structured;

    // ── Word's Normal-spacing target (derived 2026-08-04) ────────────────────
    // The previous case cascade here (stored/dd/factory arms) was fit pair-by-
    // pair and both over- and under-fired: 44 of 760 corpus pairs carried the
    // wrong LIVE Normal spacing (e.g. basic_table_shading×basic_tracked_change:
    // Word live = none, ours = factory 160/278 → global vertical drift).
    // Rebuilding the truth table from every corpus Word oracle (zero ambiguous
    // input groups) gives one closed form, verified 751/752:
    //
    //   Word rewrites Normal so B's EFFECTIVE spacing survives under the
    //   OUTPUT stylesheet's (= A's) docDefaults:
    //     ctx(attr)   = A.docDefaults spacing attr, else app default
    //                   (after=0, line=240, lineRule=auto)
    //     b_eff(attr) = B Normal stored attr, else B.docDefaults attr,
    //                   else app default   — cascade is PER ATTRIBUTE
    //     target      = { attr where b_eff(attr) != ctx(attr) } over
    //                   {after, line}; when line is written, lineRule =
    //                   b_eff(lineRule) rides along; empty target ⇒ clear.
    //
    //   The bare+bare gate stays: Word never materializes dd into an
    //   untouched Normal even when docDefaults differ (file_19/145).
    //
    // The per-attribute cascade is what the old arms could not express: Word
    // mixes sources within one spacing element (B stored line=259 + B dd
    // after=160 → live after=160 line=259) and OMITS attrs already provided
    // by A's docDefaults (A dd line=276 == B line → live a0/l- only).
    // The old FACTORY_NORMAL_SPACING / EMPTY_B_SINGLE_LINE_NORMAL constants
    // fall out: 0/240 is just b_eff = app defaults under a non-default A dd,
    // and 160/278 was B's own dd all along on the pairs that motivated it.
    let raw_a_dd = docdefaults_ppr_spacing(dom, out_root);
    let raw_b_dd = b_docdefaults_ppr_spacing(dom, b_root);
    let raw_b_sp = b_style.and_then(|s| normal_spacing(dom, s));
    // M479 — `before` rides the same per-attribute cascade (the truth table
    // was derived over {after, line}; A-dd before=240 leaked live).
    let sp_node = |dom: &Dom, root: NodeId| -> Option<NodeId> {
        let dd = dom.element(root, &W::name("docDefaults"))?;
        let pd = dom.element(dd, &W::name("pPrDefault"))?;
        let pp = dom.element(pd, &W::p_pr())?;
        dom.element(pp, &W::spacing_el())
    };
    let stored_sp_node = |dom: &Dom, style: Option<NodeId>| -> Option<NodeId> {
        let ppr = dom.element(style?, &W::p_pr())?;
        dom.element(ppr, &W::spacing_el())
    };
    let ctx_before = spacing_before_attr(dom, sp_node(dom, out_root)).unwrap_or_else(|| "0".into());
    let b_eff_before = spacing_before_attr(dom, stored_sp_node(dom, b_style))
        .or_else(|| spacing_before_attr(dom, sp_node(dom, b_root)))
        .unwrap_or_else(|| "0".into());
    let target_before: String = if b_eff_before != ctx_before {
        b_eff_before
    } else {
        String::new()
    };
    let pick = |raw: &Option<(String, String, String)>, i: usize| -> Option<String> {
        raw.as_ref().and_then(|t| {
            let v = match i {
                0 => &t.0,
                1 => &t.1,
                _ => &t.2,
            };
            (!v.is_empty()).then(|| v.clone())
        })
    };
    const APP_DEFAULT: [&str; 3] = ["0", "240", "auto"];
    let ctx = |i: usize| pick(&raw_a_dd, i).unwrap_or_else(|| APP_DEFAULT[i].to_string());
    let b_eff = |i: usize| {
        pick(&raw_b_sp, i)
            .or_else(|| pick(&raw_b_dd, i))
            .unwrap_or_else(|| APP_DEFAULT[i].to_string())
    };
    let b_target: Option<(String, String, String)> =
        if !a_structured && !b_structured && a_stored.is_none() && b_stored.is_none() {
            // both Normals bare: Word leaves the style untouched (m106 requires a
            // structured B, so it cannot land here)
            return false;
        } else {
            let after = if b_eff(0) != ctx(0) {
                b_eff(0)
            } else {
                String::new()
            };
            let line = if b_eff(1) != ctx(1) {
                b_eff(1)
            } else {
                String::new()
            };
            let rule = if line.is_empty() {
                String::new()
            } else {
                b_eff(2)
            };
            if after.is_empty() && line.is_empty() {
                None
            } else {
                Some((after, line, rule))
            }
        };
    let DocDefaultPprDelta {
        elements: dd_elements,
        spacing: dd_spacing,
    } = doc_default_ppr_delta(dom, out_root, b_root, b_style);
    let dd_delta = !dd_elements.is_empty() || !dd_spacing.is_empty();
    // Normal's other declared paragraph properties follow B as well (M72):
    // b42b3ae070's revision keeps the spacing but drops `jc=both`, and
    // Word's redline records the justification in a pPrChange.
    let rest_signature = |dom: &Dom, style: Option<NodeId>| -> Vec<String> {
        let skip = ["spacing", "pPrChange", "rPr"];
        let mut sig: Vec<String> = style
            .and_then(|s| dom.element(s, &W::p_pr()))
            .map(|p| dom.elements(p, None))
            .unwrap_or_default()
            .into_iter()
            .filter(|&c| {
                dom.name(c)
                    .is_some_and(|n| !skip.iter().any(|k| n == W::name(k)))
            })
            .map(|c| style_prop_signature(dom, c))
            .collect();
        sig.sort();
        sig
    };
    let rest_differs = rest_signature(dom, Some(a_style)) != rest_signature(dom, b_style);
    // Identity: A already has the same explicit spacing we would write.
    if target_before.is_empty()
        && !dd_delta
        && !rest_differs
        && let (Some(a), Some(b)) = (&a_stored, &b_target)
        && a == b
    {
        return false;
    }
    // Clearing when A already has no stored spacing is a no-op — except M106,
    // where Word still tracks dd spacing in pPrChange next to rPrChange —
    // and except a live `before` delta (M479), which must be written even
    // when after/line need nothing.
    if b_target.is_none()
        && a_stored.is_none()
        && !m106_same_dd_clear
        && target_before.is_empty()
        && !dd_delta
        && !rest_differs
    {
        return false;
    }
    // Old value = A's stored pPr (empty w:pPr when absent), captured before
    // the rewrite. pPrChange's inner pPr must not itself carry a pPrChange —
    // a CT_PPrBase violation Word repairs/drops — so strip any nested
    // change history from the cloned subtree (PR #81 review: real-world
    // stylesheets with pending redline on Normal).
    // M106: when A never stored pPr, Word's pPrChange old still holds the
    // shared docDefaults spacing (after=200 line=276).
    let old_ppr = match dom.element(a_style, &W::p_pr()) {
        Some(p) => {
            let clone = dom.clone_subtree(p);
            for c in dom.descendants(clone, Some(&W::p_pr_change())) {
                dom.remove(c);
            }
            clone
        }
        None => {
            let p = dom.new_element(W::p_pr());
            if m106_same_dd_clear && let Some((after, line, rule)) = &a_dd {
                let spacing = dom.new_element(W::spacing_el());
                if !after.is_empty() {
                    dom.set_attribute_value(spacing, &W::name("after"), Some(after));
                }
                if !line.is_empty() {
                    dom.set_attribute_value(spacing, &W::name("line"), Some(line));
                }
                if !rule.is_empty() {
                    dom.set_attribute_value(spacing, &W::name("lineRule"), Some(rule));
                }
                dom.add(p, spacing);
            }
            p
        }
    };
    let ppr = match dom.element(a_style, &W::p_pr()) {
        Some(p) => p,
        None => {
            let p = dom.new_element(W::p_pr());
            // pPr precedes rPr in CT_Style; insert before rPr when present.
            match dom.element(a_style, &W::r_pr()) {
                Some(rpr) => dom.add_before_self(rpr, p),
                None => dom.add(a_style, p),
            }
            p
        }
    };
    // Apply or clear Normal spacing.
    if let Some((after, line, rule)) = &b_target {
        let spacing = match dom.element(ppr, &W::spacing_el()) {
            Some(s) => s,
            None => {
                let s = dom.new_element(W::spacing_el());
                dom.add_first(ppr, s);
                s
            }
        };
        // No after-materialization here: the delta rule already writes an
        // explicit "0" when B's effective after must override A's docDefaults
        // (file_196), and Word OMITS after when the context already supplies
        // it (12 corpus oracles carry line=... with no after attribute).
        let after = after.as_str();
        let set = |dom: &mut Dom, name: &str, v: &str| {
            dom.set_attribute_value(
                spacing,
                &W::name(name),
                if v.is_empty() { None } else { Some(v) },
            );
        };
        set(dom, "before", &target_before);
        set(dom, "after", after);
        set(dom, "line", line);
        set(dom, "lineRule", rule);
        sync_normal_ppr_with_revised(dom, ppr, b_style);
    } else if !target_before.is_empty() {
        // M479 — before-only delta: write the lone neutralizer, clearing any
        // stale after/line (paragraph_spacing_missing × pci_table oracle:
        // Normal live spacing = before="0" only).
        let spacing = match dom.element(ppr, &W::spacing_el()) {
            Some(s) => s,
            None => {
                let s = dom.new_element(W::spacing_el());
                dom.add_first(ppr, s);
                s
            }
        };
        for a in ["after", "line", "lineRule"] {
            dom.set_attribute_value(spacing, &W::name(a), None);
        }
        dom.set_attribute_value(spacing, &W::name("before"), Some(&target_before));
    } else if let Some(sp) = dom.element(ppr, &W::spacing_el()) {
        // Clear explicit spacing — Word leaves empty pPr (file_22).
        dom.remove(sp);
    }
    if b_target.is_none() && rest_differs {
        sync_normal_ppr_with_revised(dom, ppr, b_style);
    }
    if !dd_spacing.is_empty() {
        let spacing = match dom.element(ppr, &W::spacing_el()) {
            Some(s) => s,
            None => {
                let s = dom.new_element(W::spacing_el());
                insert_child_by_rank(dom, ppr, s, "spacing", &ppr_child_rank);
                s
            }
        };
        for (name, v) in &dd_spacing {
            dom.set_attribute_value(spacing, &W::name(name), Some(v));
        }
    }
    for (local, el) in dd_elements {
        if let Some(old) = dom.element(ppr, &W::name(&local)) {
            dom.remove(old);
        }
        insert_child_by_rank(dom, ppr, el, &local, &ppr_child_rank);
    }
    let chg = dom.new_element(W::p_pr_change());
    // Word treats a colliding w:id on two styles' records as the same
    // revision and discards the later one (PR #81 review); only the style's
    // own rPrChange shares this id.
    let id = style_record_id(dom, out_root, ppr, settings);
    dom.set_attribute_value(chg, &W::id(), Some(&id));
    dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add(chg, old_ppr);
    dom.add(ppr, chg); // pPrChange is last in CT_PPr
    true
}

/// M111 — cascade Normal's pPrChange/rPrChange onto basedOn=Normal styles.
///
/// Word Compare (file_130 oracle) stamps `w:pPrChange` + `w:rPrChange` on ~30
/// paragraph styles based on Normal (ListParagraph, BodyText, Header, Footer,
/// List*, Quote, …) whenever Normal itself records a format change. We only
/// rewrote Normal (M71/M106), so LO still inherits docDefaults metrics on those
/// styles while Word tracked the cascade — large-doc near-90 residual gap.
///
/// For each paragraph style with `basedOn=Normal` lacking change markup:
/// - pPrChange old = live pPr children, injecting Normal's old spacing when
///   live has no `line` (ListParagraph: add after=200 line=276; BodyText: add
///   line onto existing after).
/// - rPrChange old = Normal's rPrChange old rPr (Aptos/Calibri metrics).
pub(super) fn cascade_normal_change_to_based_styles(
    dom: &mut Dom,
    styles_root: NodeId,
    settings: &WmlComparerSettings,
) -> bool {
    let Some(normal) = find_normal_style(dom, styles_root) else {
        return false;
    };
    let normal_ppc = dom
        .element(normal, &W::p_pr())
        .and_then(|p| dom.element(p, &W::p_pr_change()));
    let normal_rpc = dom
        .element(normal, &W::r_pr())
        .and_then(|r| dom.element(r, &W::r_pr_change()));
    if normal_ppc.is_none() && normal_rpc.is_none() {
        return false;
    }
    let old_spacing = normal_ppc.and_then(|ppc| {
        let old_ppr = dom.element(ppc, &W::p_pr())?;
        dom.element(old_ppr, &W::spacing_el())
    });
    let old_rpr = normal_rpc.and_then(|rpc| dom.element(rpc, &W::r_pr()));

    let mut changed = false;
    let styles: Vec<NodeId> = dom.elements(styles_root, Some(&W::name("style")));
    for style in styles {
        if dom.attribute(style, &W::type_()) != Some("paragraph") {
            continue;
        }
        if dom.attribute(style, &W::style_id()) == Some("Normal") {
            continue;
        }
        let based = dom
            .element(style, &W::name("basedOn"))
            .and_then(|b| dom.attribute(b, &W::val()));
        if based != Some("Normal") {
            continue;
        }

        // --- pPrChange ---
        if let Some(old_sp) = old_spacing {
            let ppr = match dom.element(style, &W::p_pr()) {
                Some(p) => p,
                None => {
                    let p = dom.new_element(W::p_pr());
                    match dom.element(style, &W::r_pr()) {
                        Some(r) => dom.add_before_self(r, p),
                        None => dom.add(style, p),
                    }
                    p
                }
            };
            if dom.element(ppr, &W::p_pr_change()).is_none() {
                let old_ppr = dom.new_element(W::p_pr());
                let mut has_spacing = false;
                for c in dom.elements(ppr, None) {
                    if dom.name(c) == Some(W::p_pr_change()) {
                        continue;
                    }
                    if dom.name(c) == Some(W::spacing_el()) {
                        has_spacing = true;
                    }
                    let clone = dom.clone_subtree(c);
                    dom.add(old_ppr, clone);
                }
                if !has_spacing {
                    let clone = dom.clone_subtree(old_sp);
                    dom.add(old_ppr, clone);
                } else if let Some(live_sp) = dom.element(old_ppr, &W::spacing_el()) {
                    // BodyText: live after-only → old adds line from Normal.
                    if dom.attribute(live_sp, &W::name("line")).is_none() {
                        let line = dom
                            .attribute(old_sp, &W::name("line"))
                            .map(|s| s.to_string());
                        let lr = dom
                            .attribute(old_sp, &W::name("lineRule"))
                            .map(|s| s.to_string());
                        if let Some(line) = line {
                            dom.set_attribute_value(live_sp, &W::name("line"), Some(&line));
                        }
                        if let Some(lr) = lr {
                            dom.set_attribute_value(live_sp, &W::name("lineRule"), Some(&lr));
                        }
                    }
                }
                let chg = dom.new_element(W::p_pr_change());
                let id = style_record_id(dom, styles_root, ppr, settings);
                dom.set_attribute_value(chg, &W::id(), Some(&id));
                dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
                dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
                dom.add(chg, old_ppr);
                dom.add(ppr, chg);
                changed = true;
            }
        }

        // --- rPrChange ---
        if let Some(old_r) = old_rpr {
            let rpr = match dom.element(style, &W::r_pr()) {
                Some(r) => r,
                None => {
                    let r = dom.new_element(W::r_pr());
                    dom.add(style, r);
                    r
                }
            };
            if dom.element(rpr, &W::r_pr_change()).is_none() {
                let chg = dom.new_element(W::r_pr_change());
                let id = style_record_id(dom, styles_root, rpr, settings);
                dom.set_attribute_value(chg, &W::id(), Some(&id));
                dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
                dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
                let clone = dom.clone_subtree(old_r);
                // strip nested rPrChange if any
                for c in dom.descendants(clone, Some(&W::r_pr_change())) {
                    dom.remove(c);
                }
                // The style's own properties replace Normal's: Word records
                // Balloon Text's Tahoma 8 pt (a67dcf9e05), and Reject All
                // must give it back.
                for own in dom.elements(rpr, None) {
                    let Some(name) = dom.name(own) else { continue };
                    if name == W::r_pr_change() {
                        continue;
                    }
                    let copy = dom.clone_subtree(own);
                    if let Some(stale) = dom.element(clone, &name) {
                        // Per attribute: a font slot or language the style
                        // leaves open keeps Normal's old value
                        // (eastAsiaTheme, 0800162a66).
                        if ATTR_MERGED_PROPS.contains(&name.local_name()) {
                            complete_attributes(dom, copy, stale, name.local_name());
                        }
                        dom.remove(stale);
                    }
                    add_rpr_child_in_order(dom, clone, copy, name.local_name());
                }
                dom.add(chg, clone);
                dom.add(rpr, chg);
                changed = true;
            }
        }
    }
    changed
}

// ---------------------------------------------------------------------------
// Workstream S — style-chain resolution.
// ---------------------------------------------------------------------------

/// A `w:basedOn` chain longer than this is treated as malformed and truncated.
/// Word's own limit on style inheritance depth is well below it; the cap only
/// exists so a pathological stylesheet cannot make resolution quadratic.
const MAX_STYLE_CHAIN_DEPTH: usize = 32;

/// `w:style` child order (wml.xsd `CT_Style` sequence). A `w:pPr` or `w:rPr`
/// materialized on a table style must land before `w:tblPr`/`w:trPr`/`w:tcPr`,
/// not at the end — appending produced `TableGrid` with `tblPr` before `rPr`,
/// which the OOXML validator rejects.
const STYLE_CHILD_ORDER: &[&str] = &[
    "name",
    "aliases",
    "basedOn",
    "next",
    "link",
    "autoRedefine",
    "hidden",
    "uiPriority",
    "semiHidden",
    "unhideWhenUsed",
    "qFormat",
    "locked",
    "personal",
    "personalCompose",
    "personalReply",
    "rsid",
    "pPr",
    "rPr",
    "tblPr",
    "trPr",
    "tcPr",
    "tblStylePr",
];

/// Insert `child` under `parent` at the position `order` gives its `local`
/// name: immediately before the first existing child that sorts after it, else
/// appended. Unknown names sort last, so they never displace a known one.
pub(super) fn insert_child_by_rank(
    dom: &mut Dom,
    parent: NodeId,
    child: NodeId,
    local: &str,
    rank_of: &dyn Fn(&str) -> usize,
) {
    let new_rank = rank_of(local);
    let successor = dom.elements(parent, None).into_iter().find(|&e| {
        dom.name(e)
            .is_some_and(|n| rank_of(n.local_name()) > new_rank)
    });
    match successor {
        Some(s) => dom.add_before_self(s, child),
        None => dom.add(parent, child),
    }
}

/// Rank of a `w:style` child in [`STYLE_CHILD_ORDER`].
fn style_child_rank(local: &str) -> usize {
    STYLE_CHILD_ORDER
        .iter()
        .position(|&n| n == local)
        .unwrap_or(usize::MAX)
}

/// Rank of a `w:pPr` child in [`crate::comparer::order_tables::PPR_ORDER`].
pub(super) fn ppr_child_rank(local: &str) -> usize {
    crate::comparer::order_tables::PPR_ORDER
        .iter()
        .find(|(n, _)| *n == local)
        .map(|(_, r)| *r as usize)
        .unwrap_or(usize::MAX)
}

/// M492: deleted paragraphs keep their A-ORIGINAL direct spacing.
/// Word preserves the source paragraph's own w:spacing on del-marked
/// paragraphs (file_22 × file_23 oracle: 31 deleted paras carry their
/// line=240/atLeast declarations; we stripped all but one — each
/// stripped para renders at the taller docDefaults line and the
/// accumulated height drifts the pagination, the long-standing
/// 115-vs-116-page mystery). Finalize passes can't see provenance, so
/// restore here from A's package: any del-marked paragraph whose
/// A-source para (matched by w14:paraId) declared w:spacing gets the
/// original attributes back when the output paragraph lost them.
///
/// Only `w:` attributes are restored: A's working copy carries the
/// comparer's `pt14:Unid` stamps, which must not come back as `w:Unid`.
/// Returns the rewritten `out_xml`, or `None` when nothing changed.
/// Word's redline font table is the union of both documents' tables. Keeping
/// only the original's leaves fonts that arrive with the revised text without
/// their charset/panose/family entry, and Word substitutes blind (file_46 ×
/// file_47: Times New Roman where Word's own redline used Hiragino Mincho for
/// Liberation Serif / Droid Sans Fallback). B's fonts the output does not name
/// are appended; embedded-font children stay behind, since their
/// relationships and obfuscation keys belong to B's package.
/// Returns the rewritten `out_xml`, or `None` when nothing changed.
pub(super) fn merge_revised_font_table(out_xml: &str, b_xml: &str) -> Option<String> {
    let font = W::name("font");
    let name = W::name("name");
    let mut dom = Dom::new();
    let od = dom.parse_xdocument(out_xml);
    let bd = dom.parse_xdocument(b_xml);
    let (out_root, b_root) = (dom.root(od)?, dom.root(bd)?);
    let known: std::collections::HashSet<String> = dom
        .elements(out_root, Some(&font))
        .into_iter()
        .filter_map(|f| dom.attribute(f, &name).map(str::to_string))
        .collect();
    let mut changed = false;
    for f in dom.elements(b_root, Some(&font)) {
        if dom.attribute(f, &name).is_none_or(|n| known.contains(n)) {
            continue;
        }
        let copy = dom.clone_subtree(f);
        for c in dom.elements(copy, None) {
            if dom.name(c).is_some_and(|n| {
                n.namespace_name() == W::URI && n.local_name().starts_with("embed")
            }) {
                dom.remove(c);
            }
        }
        dom.add(out_root, copy);
        changed = true;
    }
    changed.then(|| dom.serialize_element(out_root))
}

pub(super) fn restore_deleted_paragraph_spacing(a_xml: &str, out_xml: &str) -> Option<String> {
    let w14_pid = crate::namespaces::W14::name("paraId");
    let mut a_spacing: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    let mut ad = Dom::new();
    let d = ad.parse_xdocument(a_xml);
    let r = ad.root(d)?;
    for pnode in ad.descendants(r, Some(&W::p())) {
        let Some(pid) = ad.attribute(pnode, &w14_pid).map(str::to_string) else {
            continue;
        };
        if let Some(ppr) = ad.element(pnode, &W::p_pr())
            && let Some(sp) = ad.element(ppr, &W::spacing_el())
        {
            let attrs: Vec<(String, String)> = ad
                .attributes(sp)
                .into_iter()
                .filter(|(n, _)| n.namespace_name() == W::URI)
                .map(|(n, v)| (n.local_name().to_string(), v))
                .collect();
            if !attrs.is_empty() {
                a_spacing.insert(pid, attrs);
            }
        }
    }
    if a_spacing.is_empty() {
        return None;
    }
    let mut pd = Dom::new();
    let d = pd.parse_xdocument(out_xml);
    let root = pd.root(d)?;
    let mut changed = false;
    for pnode in pd.descendants(root, Some(&W::p())) {
        let Some(pid) = pd.attribute(pnode, &w14_pid).map(str::to_string) else {
            continue;
        };
        let Some(attrs) = a_spacing.get(&pid) else {
            continue;
        };
        let Some(ppr) = pd.element(pnode, &W::p_pr()) else {
            continue;
        };
        let mark_del = pd
            .element(ppr, &W::r_pr())
            .is_some_and(|r| pd.element(r, &W::del()).is_some());
        if !mark_del || pd.element(ppr, &W::spacing_el()).is_some() {
            continue;
        }
        let sp = pd.new_element(W::spacing_el());
        for (n, v) in attrs {
            pd.set_attribute_value(sp, &W::name(n), Some(v));
        }
        insert_child_by_rank(&mut pd, ppr, sp, "spacing", &ppr_child_rank);
        changed = true;
    }
    changed.then(|| pd.serialize_element(root))
}

/// Children of a style's `w:pPr`/`w:rPr` that are not formatting: revision
/// records, section properties, and revision-save ids.
fn is_style_prop_noise(name: &crate::xmllinq::XName) -> bool {
    matches!(
        name.local_name(),
        "pPrChange" | "rPrChange" | "sectPr" | "sectPrChange" | "rsid"
    )
}

/// Order-independent signature of one formatting element: expanded name, its
/// non-`rsid` attributes sorted by name, then its children's signatures in
/// document order (child order carries meaning inside `w:tabs`, `w:pBdr`, …).
pub(super) fn style_prop_signature(dom: &Dom, node: NodeId) -> String {
    let Some(n) = dom.name(node) else {
        return String::new();
    };
    let mut out = n.clark();
    let mut attrs: Vec<(String, &str)> = dom
        .attributes_iter(node)
        .filter(|(a, _)| {
            !a.local_name().to_ascii_lowercase().starts_with("rsid")
                && !dom.is_namespace_declaration(a)
        })
        .map(|(a, v)| (a.clark(), v))
        .collect();
    attrs.sort();
    for (a, v) in attrs {
        out.push('\u{1}');
        out.push_str(&a);
        out.push('=');
        out.push_str(v);
    }
    for i in 0..dom.child_count(node) {
        let c = dom.child_at(node, i);
        if dom.is_element(c) {
            out.push('\u{2}');
            out.push_str(&style_prop_signature(dom, c));
        }
    }
    out
}

/// Signature of a style's DECLARED `w:pPr`/`w:rPr` block: the formatting
/// children it writes itself, ignoring everything it inherits. Empty string
/// when the block is absent or holds only noise.
fn declared_props_signature(dom: &Dom, style: NodeId, local: &str) -> String {
    let Some(block) = dom.element(style, &W::name(local)) else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    for c in dom.elements(block, None) {
        let Some(n) = dom.name(c) else { continue };
        if is_style_prop_noise(&n) {
            continue;
        }
        parts.push(style_prop_signature(dom, c));
    }
    parts.sort();
    parts.join("\u{3}")
}

/// Property elements whose ATTRIBUTES are independent inherited values: a link
/// in the chain that sets some of them leaves the rest inherited instead of
/// replacing the element wholesale.
///
/// `w:rFonts` is the one that decides real documents. Oracle evidence
/// (`Hello_docx_world × multi_image_types`): A's `Heading3Char` writes
/// `asciiTheme="minorHAnsi" hAnsiTheme="minorHAnsi"` and B's omits both, and
/// Word records **no change** on any of `Heading3Char`..`Heading9Char` —
/// because omitting them inherits exactly `minorHAnsi` from `docDefaults`.
/// Comparing the element whole marks all seven as changed. `w:lang` behaves the
/// same way (`val` / `eastAsia` / `bidi` are separate slots).
///
/// The same per-attribute rule is already relied on by
/// [`effective_normal_rpr_metrics`] for the footer-metric cascade.
const ATTR_MERGED_PROPS: &[&str] = &["rFonts", "lang"];

/// `w:rFonts` attribute pairs that are alternatives for one typeface slot:
/// naming either clears the other, so a nearer link that switches a slot to an
/// explicit face must not leave the farther link's theme reference standing.
const RFONTS_ALTERNATIVE_SLOTS: [[&str; 2]; 4] = [
    ["ascii", "asciiTheme"],
    ["hAnsi", "hAnsiTheme"],
    ["eastAsia", "eastAsiaTheme"],
    ["cs", "cstheme"],
];

/// A style's EFFECTIVE `w:pPr`/`w:rPr`, resolved the way Word resolves a style
/// chain: `w:docDefaults` first, then each `w:basedOn` ancestor from the root of
/// the chain downwards, then the style's own declared properties. Keyed by
/// element name so a nearer link overrides a farther one, except for
/// [`ATTR_MERGED_PROPS`], which merge attribute-by-attribute.
pub(super) fn effective_style_props(
    dom: &Dom,
    styles_root: NodeId,
    by_id: &std::collections::HashMap<String, NodeId>,
    style: NodeId,
    local: &str,
    default_local: &str,
) -> std::collections::BTreeMap<String, String> {
    effective_chain_props(dom, styles_root, by_id, Some(style), local, default_local)
}

/// What `style` declares beyond its parent: each effective property of
/// [`effective_style_props`] whose value its `basedOn` parent (the
/// docDefaults for a root style) does not already give. A restatement of the
/// parent drops out.
pub(super) fn own_style_props(
    dom: &Dom,
    styles_root: NodeId,
    by_id: &std::collections::HashMap<String, NodeId>,
    style: NodeId,
    local: &str,
    default_local: &str,
) -> std::collections::BTreeMap<String, String> {
    let parent = dom
        .element(style, &W::name("basedOn"))
        .and_then(|b| dom.attribute(b, &W::val()))
        .and_then(|v| by_id.get(v).copied())
        .filter(|&p| p != style);
    let inherited = effective_chain_props(dom, styles_root, by_id, parent, local, default_local);
    let mut own = effective_style_props(dom, styles_root, by_id, style, local, default_local);
    own.retain(|k, v| inherited.get(k) != Some(v));
    own
}

/// Freeze the style blocks whose declared difference is only inherited.
/// Read the original A tree and revised B tree before output parents change.
/// Inbound records are never eligible for removal. Identical declarations
/// keep the established Normal cascade (including partial language slots).
pub(super) fn inherited_only_style_changes(
    dom: &Dom,
    out_root: NodeId,
    a_root: NodeId,
    b_root: NodeId,
) -> std::collections::HashSet<(NodeId, &'static str)> {
    let by_id = |root| -> std::collections::HashMap<String, NodeId> {
        dom.elements(root, Some(&W::name("style")))
            .into_iter()
            .filter_map(|style| Some((dom.attribute(style, &W::style_id())?.to_string(), style)))
            .collect()
    };
    let by_key = |root| -> std::collections::HashMap<(String, String), NodeId> {
        let mut index = std::collections::HashMap::new();
        for style in dom.elements(root, Some(&W::name("style"))) {
            if let Some(key) = style_match_key(dom, style) {
                index.entry(key).or_insert(style);
            }
        }
        index
    };
    let a_ids = by_id(a_root);
    let b_ids = by_id(b_root);
    let a_keys = by_key(a_root);
    let b_keys = by_key(b_root);
    let normal = find_normal_style(dom, out_root);
    let mut suppressed = std::collections::HashSet::new();
    for style in dom.elements(out_root, Some(&W::name("style"))) {
        if Some(style) == normal {
            continue;
        }
        let Some(key) = style_match_key(dom, style) else {
            continue;
        };
        let (Some(&a_style), Some(&b_style)) = (a_keys.get(&key), b_keys.get(&key)) else {
            continue;
        };
        for (local, defaults, change) in [
            ("pPr", "pPrDefault", "pPrChange"),
            ("rPr", "rPrDefault", "rPrChange"),
        ] {
            let inbound = [style, a_style, b_style].into_iter().any(|source| {
                dom.element(source, &W::name(local))
                    .is_some_and(|block| dom.element(block, &W::name(change)).is_some())
            });
            if inbound
                || declared_props_signature(dom, a_style, local)
                    == declared_props_signature(dom, b_style, local)
                || effective_style_props(dom, a_root, &a_ids, a_style, local, defaults)
                    == effective_style_props(dom, b_root, &b_ids, b_style, local, defaults)
            {
                continue;
            }
            if own_style_props(dom, a_root, &a_ids, a_style, local, defaults)
                == own_style_props(dom, b_root, &b_ids, b_style, local, defaults)
            {
                suppressed.insert((style, local));
            }
        }
    }
    suppressed
}

/// Remove newly synthesized inherited-only history after live metrics have
/// been normalized. The cascade record must remain available until then:
/// metric resolution uses tracked styles to replace obsolete A declarations.
pub(super) fn remove_inherited_only_style_records(
    dom: &mut Dom,
    suppressed: &std::collections::HashSet<(NodeId, &'static str)>,
) -> bool {
    let mut changed = false;
    for &(style, local) in suppressed {
        let change = if local == "pPr" {
            "pPrChange"
        } else {
            "rPrChange"
        };
        if let Some(record) = dom
            .element(style, &W::name(local))
            .and_then(|block| dom.element(block, &W::name(change)))
        {
            dom.remove(record);
            changed = true;
        }
    }
    changed
}

/// [`effective_style_props`] from `start` up its `basedOn` chain; `None`
/// resolves the docDefaults alone.
pub(super) fn effective_chain_props(
    dom: &Dom,
    styles_root: NodeId,
    by_id: &std::collections::HashMap<String, NodeId>,
    start: Option<NodeId>,
    local: &str,
    default_local: &str,
) -> std::collections::BTreeMap<String, String> {
    // Walk basedOn to the root of the chain, guarding against cycles.
    let mut chain: Vec<NodeId> = Vec::new();
    let mut seen: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
    let mut cur = start;
    while let Some(s) = cur {
        if !seen.insert(s) || chain.len() >= MAX_STYLE_CHAIN_DEPTH {
            break;
        }
        chain.push(s);
        cur = dom
            .element(s, &W::name("basedOn"))
            .and_then(|b| dom.attribute(b, &W::val()))
            .and_then(|v| by_id.get(v).copied());
    }
    chain.reverse();

    type Props = std::collections::BTreeMap<String, String>;
    type Slots = std::collections::BTreeMap<String, Props>;
    let mut props: Props = Props::new();
    let mut slots: Slots = Slots::new();

    let absorb = |props: &mut Props, slots: &mut Slots, block: NodeId| {
        for i in 0..dom.child_count(block) {
            let c = dom.child_at(block, i);
            if !dom.is_element(c) {
                continue;
            }
            let Some(n) = dom.name(c) else { continue };
            if is_style_prop_noise(&n) {
                continue;
            }
            if !ATTR_MERGED_PROPS.contains(&n.local_name()) {
                props.insert(n.clark(), style_prop_signature(dom, c));
                continue;
            }
            let slot = slots.entry(n.clark()).or_default();
            for (a, v) in dom.attributes_iter(c) {
                if a.local_name().to_ascii_lowercase().starts_with("rsid")
                    || dom.is_namespace_declaration(a)
                {
                    continue;
                }
                if n.local_name() == "rFonts" {
                    for pair in RFONTS_ALTERNATIVE_SLOTS {
                        if pair.contains(&a.local_name()) {
                            // Slots are keyed by EXPANDED name; the alternative
                            // shares this attribute's namespace.
                            for other in pair {
                                slot.remove(&a.namespace().name(other).clark());
                            }
                        }
                    }
                }
                slot.insert(a.clark(), v.to_string());
            }
        }
    };
    if let Some(dd) = dom
        .element(styles_root, &W::name("docDefaults"))
        .and_then(|d| dom.element(d, &W::name(default_local)))
        .and_then(|d| dom.element(d, &W::name(local)))
    {
        absorb(&mut props, &mut slots, dd);
    }
    for s in chain {
        if let Some(block) = dom.element(s, &W::name(local)) {
            absorb(&mut props, &mut slots, block);
        }
    }
    for (name, attrs) in slots {
        let sig = attrs
            .into_iter()
            .map(|(a, v)| format!("{a}={v}"))
            .collect::<Vec<_>>()
            .join("\u{1}");
        props.insert(name, sig);
    }
    props
}

/// The key a style is matched across the two stylesheets by: `(type, name)`,
/// falling back to the styleId when the style carries no `w:name`.
///
/// Matching on the NAME rather than the id is what makes this survive
/// [`canonicalize_style_ids`], which has already rewritten the output's ids
/// (`SD_StrikeChar` → `SDStrikeChar`) while the revised package still holds the
/// originals. The canonical id is a pure function of the name, so equal names
/// are exactly the styles that canonicalization would have unified.
/// Detached copies of the styles of `out_root` whose type and name `b_root`
/// does not define, each with the key that finds it again.
pub(super) fn styles_the_revision_lacks(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
) -> Vec<((String, String), NodeId)> {
    let b_keys: std::collections::HashSet<(String, String)> = dom
        .elements(b_root, Some(&W::name("style")))
        .into_iter()
        .filter_map(|s| style_match_key(dom, s))
        .collect();
    let only_a: Vec<((String, String), NodeId)> = dom
        .elements(out_root, Some(&W::name("style")))
        .into_iter()
        .filter_map(|s| style_match_key(dom, s).map(|k| (k, s)))
        .filter(|(k, _)| !b_keys.contains(k))
        .collect();
    only_a
        .into_iter()
        .map(|(k, s)| (k, dom.clone_subtree(s)))
        .collect()
}

/// Put each saved style back in place of the one its key finds. Twins of one
/// key (a built-in name in two cases) go back in order, each on its own.
pub(super) fn restore_styles(
    dom: &mut Dom,
    styles_root: NodeId,
    saved: Vec<((String, String), NodeId)>,
) -> bool {
    let mut changed = false;
    let mut restored = std::collections::HashSet::new();
    for (key, copy) in saved {
        let Some(live) = dom
            .elements(styles_root, Some(&W::name("style")))
            .into_iter()
            .find(|&s| !restored.contains(&s) && style_match_key(dom, s).as_ref() == Some(&key))
        else {
            continue;
        };
        if dom.serialize_element(live) != dom.serialize_element(copy) {
            dom.replace_with(live, &[copy]);
            restored.insert(copy);
            changed = true;
        } else {
            restored.insert(live);
        }
    }
    changed
}

/// The key Word pairs two stylesheets' styles by: the type, and the name
/// (the id when it has none). A built-in name matches in any case, a custom
/// one only exactly: Word keeps `Indent(A)` beside `Indent(a)`.
pub(super) fn style_match_key(dom: &Dom, style: NodeId) -> Option<(String, String)> {
    let ty = dom
        .attribute(style, &W::type_())
        .unwrap_or("paragraph")
        .to_string();
    let name = dom
        .element(style, &W::name("name"))
        .and_then(|n| dom.attribute(n, &W::val()))
        .or_else(|| dom.attribute(style, &W::style_id()))?;
    let key = if crate::builtin_styles::is_built_in(name) {
        name.to_ascii_lowercase()
    } else {
        name.to_string()
    };
    Some((ty, key))
}

/// Word writes a root style's recorded old properties out in full against the
/// docDefaults (c719b900f0, f1257ca7ea, 221577c35b, 4eff11f045): the old
/// block holds every property the docDefaults set, at the docDefaults value
/// where the original's style leaves it to them, with `w:rFonts`, `w:lang`
/// and `w:spacing` completed attribute by attribute (`w:lang
/// w:val="es-ES"` gains the default `eastAsia`/`bidi`, `w:spacing
/// w:after="0"` the default `line`). A root style with one record gets the
/// other too. Word's Reject All of a record that leaves a property out
/// writes a value of its own (sz=20 and a Times New Roman complex script
/// over the original's 11pt Arial, the live line pitch over the default),
/// so the complete record is what rejects to the original. Only paragraph
/// styles without `w:basedOn` qualify: below them the fallback is the parent
/// style.
pub(super) fn complete_root_style_change_records(
    dom: &mut Dom,
    styles_root: NodeId,
    settings: &WmlComparerSettings,
) -> bool {
    let docdefaults = dom.element(styles_root, &W::name("docDefaults"));
    let defaults = |dom: &Dom, default_local: &str, block_local: &str| {
        docdefaults
            .and_then(|d| dom.element(d, &W::name(default_local)))
            .and_then(|d| dom.element(d, &W::name(block_local)))
            .filter(|&b| !dom.elements(b, None).is_empty())
    };
    let blocks = [
        ("pPr", "pPrDefault", "pPrChange"),
        ("rPr", "rPrDefault", "rPrChange"),
    ];
    let record = |dom: &Dom, style: NodeId, block_local: &str, change_local: &str| {
        dom.element(style, &W::name(block_local))
            .and_then(|b| dom.element(b, &W::name(change_local)))
    };
    let mut changed = false;
    for style in dom.elements(styles_root, Some(&W::name("style"))) {
        if dom.attribute(style, &W::type_()).unwrap_or("paragraph") != "paragraph"
            || dom.element(style, &W::name("basedOn")).is_some()
            || blocks
                .iter()
                .all(|(b, _, c)| record(dom, style, b, c).is_none())
        {
            continue;
        }
        for (block_local, default_local, change_local) in blocks {
            let Some(defaults) = defaults(dom, default_local, block_local) else {
                continue;
            };
            let old = match record(dom, style, block_local, change_local)
                .and_then(|c| dom.element(c, &W::name(block_local)))
            {
                Some(old) => old,
                None => {
                    let block = match dom.element(style, &W::name(block_local)) {
                        Some(block) => block,
                        None => {
                            let block = dom.new_element(W::name(block_local));
                            add_child_in_rank_order(dom, style, block, style_child_rank);
                            block
                        }
                    };
                    let old = dom.new_element(W::name(block_local));
                    for c in dom.elements(block, None) {
                        let copy = dom.clone_subtree(c);
                        dom.add(old, copy);
                    }
                    append_style_change_record(
                        dom,
                        styles_root,
                        block,
                        old,
                        change_local,
                        settings,
                    );
                    old
                }
            };
            changed |= complete_from_defaults(dom, old, defaults, block_local);
        }
    }
    changed
}

/// Complete the old records of each paragraph style with a `w:basedOn` from
/// what the original's docDefaults and chain gave the same style (matched by
/// type and name), so Reject All gives back the original's look rather than
/// Word's built-ins: see
/// [`crate::revision_processor::style_records::complete_from_original_chain`].
/// Styles the original lacks keep their records as written.
pub(super) fn complete_based_style_change_records(
    dom: &mut Dom,
    styles_root: NodeId,
    a_root: NodeId,
) -> bool {
    let a_by_key: std::collections::HashMap<(String, String), NodeId> = dom
        .elements(a_root, Some(&W::name("style")))
        .into_iter()
        .filter_map(|s| Some((style_match_key(dom, s)?, s)))
        .collect();
    // The original's styles each block of which the redline records.
    let mut recorded: std::collections::HashMap<&str, std::collections::HashSet<NodeId>> =
        std::collections::HashMap::new();
    for style in dom.elements(styles_root, Some(&W::name("style"))) {
        let Some(&a_style) = style_match_key(dom, style).and_then(|k| a_by_key.get(&k)) else {
            continue;
        };
        for (block_local, change_local) in [("pPr", "pPrChange"), ("rPr", "rPrChange")] {
            if dom
                .element(style, &W::name(block_local))
                .and_then(|b| dom.element(b, &W::name(change_local)))
                .is_some()
            {
                recorded.entry(block_local).or_default().insert(a_style);
            }
        }
    }
    let mut changed = false;
    for style in dom.elements(styles_root, Some(&W::name("style"))) {
        if dom.attribute(style, &W::type_()).unwrap_or("paragraph") != "paragraph"
            || dom.element(style, &W::name("basedOn")).is_none()
        {
            continue;
        }
        let Some(&a_style) = style_match_key(dom, style).and_then(|k| a_by_key.get(&k)) else {
            continue;
        };
        for (block_local, change_local) in [("pPr", "pPrChange"), ("rPr", "rPrChange")] {
            let restored = recorded.get(block_local);
            let is_recorded = |s: NodeId| restored.is_some_and(|r| r.contains(&s));
            let Some(old) = dom
                .element(style, &W::name(block_local))
                .and_then(|b| dom.element(b, &W::name(change_local)))
                .and_then(|c| dom.element(c, &W::name(block_local)))
            else {
                continue;
            };
            changed |= crate::revision_processor::style_records::complete_from_original_chain(
                dom,
                old,
                a_root,
                a_style,
                block_local,
                &is_recorded,
            );
        }
    }
    changed
}

/// The properties whose recorded old value Word completes attribute by
/// attribute from the docDefaults.
const RECORD_COMPLETED_PROPS: &[&str] = &["rFonts", "lang", "spacing"];

/// `w:spacing` attributes that are alternatives for one value: a recorded
/// `w:before` must not gain the default `w:beforeAutospacing`, which wins.
const SPACING_ALTERNATIVE_SLOTS: [[&str; 3]; 2] = [
    ["before", "beforeAutospacing", "beforeLines"],
    ["after", "afterAutospacing", "afterLines"],
];

/// Give the recorded `old` block every property of the docDefaults `defaults`
/// block it lacks, and complete its [`RECORD_COMPLETED_PROPS`].
pub(super) fn complete_from_defaults(
    dom: &mut Dom,
    old: NodeId,
    defaults: NodeId,
    block_local: &str,
) -> bool {
    let mut changed = false;
    for default in dom.elements(defaults, None) {
        let Some(name) = dom.name(default) else {
            continue;
        };
        if is_style_prop_noise(&name) {
            continue;
        }
        let local = name.local_name().to_string();
        match dom.element(old, &name) {
            Some(recorded) if RECORD_COMPLETED_PROPS.contains(&local.as_str()) => {
                changed |= complete_attributes(dom, recorded, default, &local);
            }
            Some(_) => {}
            None => {
                let copy = dom.clone_subtree(default);
                if block_local == "rPr" {
                    add_rpr_child_in_order(dom, old, copy, &local);
                } else {
                    add_child_in_rank_order(dom, old, copy, ppr_child_rank);
                }
                changed = true;
            }
        }
    }
    changed
}

/// Give `recorded` each attribute of `default` it lacks, unless it names an
/// alternative for that slot (an explicit `w:ascii` over `w:asciiTheme`).
fn complete_attributes(dom: &mut Dom, recorded: NodeId, default: NodeId, local: &str) -> bool {
    let slots: Vec<&[&str]> = match local {
        "rFonts" => RFONTS_ALTERNATIVE_SLOTS
            .iter()
            .map(|p| p.as_slice())
            .collect(),
        "spacing" => SPACING_ALTERNATIVE_SLOTS
            .iter()
            .map(|p| p.as_slice())
            .collect(),
        _ => Vec::new(),
    };
    let mut changed = false;
    for (attr, value) in dom.attributes(default) {
        if dom.is_namespace_declaration(&attr) || dom.attribute(recorded, &attr).is_some() {
            continue;
        }
        let taken = slots
            .iter()
            .filter(|slot| slot.contains(&attr.local_name()))
            .flat_map(|slot| slot.iter())
            .any(|other| {
                dom.attribute(recorded, &attr.namespace().name(other))
                    .is_some()
            });
        if !taken {
            dom.set_attribute_value(recorded, &attr, Some(&value));
            changed = true;
        }
    }
    changed
}

/// Insert `child` under `parent` after the last existing child that `rank`
/// puts before it, or first.
fn add_child_in_rank_order(dom: &mut Dom, parent: NodeId, child: NodeId, rank: fn(&str) -> usize) {
    let own = dom.name(child).map_or(usize::MAX, |n| rank(n.local_name()));
    let anchor = dom
        .elements(parent, None)
        .into_iter()
        .rfind(|&e| dom.name(e).is_some_and(|n| rank(n.local_name()) < own));
    match anchor {
        Some(a) => dom.add_after_self(a, child),
        None => dom.add_first(parent, child),
    }
}

/// Wrap `old` (a `w:pPr`/`w:rPr` clone) in a `w:pPrChange`/`w:rPrChange` record
/// and append it to `block`, which is where CT_PPr / CT_RPr put it.
pub(super) fn append_style_change_record(
    dom: &mut Dom,
    styles_root: NodeId,
    block: NodeId,
    old: NodeId,
    change_local: &str,
    settings: &WmlComparerSettings,
) {
    let chg = dom.new_element(W::name(change_local));
    let id = style_record_id(dom, styles_root, block, settings);
    dom.set_attribute_value(chg, &W::id(), Some(&id));
    dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add(chg, old);
    dom.add(block, chg);
}

/// Workstream S — adopt the REVISED document's definition of every style both
/// documents define, recording the ORIGINAL definition on the `w:style` itself.
///
/// Word's Compare stylesheet resolves the style chain on both sides and, where
/// a style resolves differently, writes B's declared properties live with A's
/// inside a `w:pPrChange` / `w:rPrChange` on the `w:style` element. Oracle
/// evidence (`two_column_two_page × vrect_node`): 18 styles marked, `Title`
/// live at B's `sz=56` with A's `sz=52 color=17365D` in `w:rPrChange` and A's
/// `pBdr` + `after=300` in `w:pPrChange`. Over the 564 corpus pairs that have a
/// Word oracle, 52.5% of the oracle stylesheets carry style-level change
/// markup.
///
/// [`crate::comparer::footnotes::copy_missing_styles`] is keyed on
/// `(type, styleId)` and skips an id already present whatever its body, so
/// before this pass the output kept **A's** definition and recorded nothing:
/// content on both sides rendered with the original's fonts, sizes and borders
/// and the change was invisible. 136 of 597 corpus pairs carry at least one
/// such live collision; they score a mean 59.7 against 79.4 for the pairs
/// without one.
///
/// Two guards keep this from manufacturing empty change records:
/// - the EFFECTIVE properties must differ (so a stylesheet reaching the same
///   result through `basedOn` is not marked), and
/// - the DECLARED properties must differ (so a difference that actually lives
///   in an ancestor is recorded on that ancestor, once, and not restated on
///   every style below it).
///
/// `Normal` is excluded — it is owned by [`merge_normal_style_spacing`] /
/// [`merge_normal_style_rpr`], whose rules are calibrated against a separate
/// body of oracle evidence. Styles already carrying change markup are left
/// alone, which is also what lets [`cascade_normal_change_to_based_styles`]
/// (M111) keep acting as the fallback for styles this pass does not touch.
pub(super) fn merge_revised_style_definitions(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
    settings: &WmlComparerSettings,
    a_declared_keys: &std::collections::HashSet<(String, String)>,
    inherited_only: &std::collections::HashSet<(NodeId, &'static str)>,
) -> bool {
    let style_nm = W::name("style");
    let style_id = W::style_id();
    let normal_out = find_normal_style(dom, out_root);
    let normal_b = find_normal_style(dom, b_root);

    let index_by_id = |dom: &Dom, root: NodeId| -> std::collections::HashMap<String, NodeId> {
        dom.elements(root, Some(&style_nm))
            .into_iter()
            .filter_map(|s| Some((dom.attribute(s, &style_id)?.to_string(), s)))
            .collect()
    };
    let out_by_id = index_by_id(dom, out_root);
    let b_by_id = index_by_id(dom, b_root);

    let mut b_by_key: std::collections::HashMap<(String, String), NodeId> =
        std::collections::HashMap::new();
    for s in dom.elements(b_root, Some(&style_nm)) {
        if let Some(k) = style_match_key(dom, s) {
            b_by_key.entry(k).or_insert(s);
        }
    }

    let mut changed = false;
    for style in dom.elements(out_root, Some(&style_nm)) {
        if Some(style) == normal_out {
            continue;
        }
        let Some(key) = style_match_key(dom, style) else {
            continue;
        };
        let Some(&b_style) = b_by_key.get(&key) else {
            continue;
        };
        if Some(b_style) == normal_b {
            continue;
        }
        for (local, default_local, change_local) in [
            ("pPr", "pPrDefault", "pPrChange"),
            ("rPr", "rPrDefault", "rPrChange"),
        ] {
            let a_declared = declared_props_signature(dom, style, local);
            let b_declared = declared_props_signature(dom, b_style, local);
            if a_declared == b_declared {
                continue;
            }
            let a_eff =
                effective_style_props(dom, out_root, &out_by_id, style, local, default_local);
            let b_eff = effective_style_props(dom, b_root, &b_by_id, b_style, local, default_local);
            if a_eff == b_eff {
                continue;
            }
            // The difference lives in an ancestor when what each side adds
            // to its parent is the same: a style restating its parent adds
            // nothing (c719b900f0's Body Text repeats Normal's Arial). Word
            // records no change there, and its Reject All of one writes
            // values of its own onto the style.
            if inherited_only.contains(&(style, local)) {
                continue;
            }
            // Already tracked (an inbound stylesheet with pending redline, or an
            // earlier pass) — do not stack a second record on the same block.
            if dom
                .element(style, &W::name(local))
                .is_some_and(|blk| dom.element(blk, &W::name(change_local)).is_some())
            {
                continue;
            }

            // Old = A's declared block, change history stripped: the inner pPr
            // of a pPrChange is CT_PPrBase and may not itself carry one.
            let old = match dom.element(style, &W::name(local)) {
                Some(blk) => {
                    let clone = dom.clone_subtree(blk);
                    for c in dom.descendants(clone, Some(&W::name(change_local))) {
                        dom.remove(c);
                    }
                    clone
                }
                None => dom.new_element(W::name(local)),
            };

            // Live = B's declared block. Materialize A's when absent, keeping
            // CT_Style order (pPr precedes rPr).
            let block = match dom.element(style, &W::name(local)) {
                Some(blk) => {
                    let stale: Vec<NodeId> = dom
                        .elements(blk, None)
                        .into_iter()
                        .filter(|&c| dom.name(c).is_none_or(|n| !is_style_prop_noise(&n)))
                        .collect();
                    for c in stale {
                        dom.remove(c);
                    }
                    blk
                }
                None => {
                    let blk = dom.new_element(W::name(local));
                    insert_child_by_rank(dom, style, blk, local, &style_child_rank);
                    blk
                }
            };
            if let Some(b_block) = dom.element(b_style, &W::name(local)) {
                for c in dom.elements(b_block, None) {
                    let Some(n) = dom.name(c) else { continue };
                    if is_style_prop_noise(&n) {
                        continue;
                    }
                    let clone = dom.clone_subtree(c);
                    dom.add(block, clone);
                }
            }
            append_style_change_record(dom, out_root, block, old, change_local, settings);
            changed = true;
        }
    }

    // Phase 2 (S2) — styles COPIED from B (absent from A's stylesheet) bake
    // B's docDefaults-level run metrics and carry style-level change records.
    //
    // Oracle (image_inline_and_block × rtl_page_numpages): A's stylesheet
    // never declares Footer/FootnoteText/Strong1. The output stylesheet keeps
    // A's docDefaults (theme fonts, sz 24, kern 2, ligatures
    // standardContextual), so a verbatim copy of B's style renders with A's
    // metrics — the R2 cluster's cumulative vertical drift. Word declares the
    // neutralizing delta live on each copied style (rFonts TNR, sz 20, szCs
    // 20, kern 0, w14:ligatures none — implicit defaults materialized: sz 20,
    // kern 0, ligatures none) and marks it with `w:rPrChange` (old = declared
    // + lang) and `w:pPrChange` (old = declared pPr).
    {
        let dd_rpr = |dom: &Dom, root: NodeId| -> Option<NodeId> {
            let dd = dom.element(root, &W::name("docDefaults"))?;
            let rd = dom.element(dd, &W::name("rPrDefault"))?;
            dom.element(rd, &W::r_pr())
        };
        let a_dd = dd_rpr(dom, out_root);
        let b_dd = dd_rpr(dom, b_root);
        let attr_map =
            |dom: &Dom, e: Option<NodeId>| -> std::collections::BTreeMap<String, String> {
                e.map(|e| {
                    dom.attributes(e)
                        .into_iter()
                        .map(|(n, v)| (format!("{n:?}"), v))
                        .collect()
                })
                .unwrap_or_default()
            };
        let val_or = |dom: &Dom, dd: Option<NodeId>, local: &str, default: &str| -> String {
            dd.and_then(|d| dom.element(d, &W::name(local)))
                .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
                .unwrap_or_else(|| default.to_string())
        };
        let lig_name = W14::name("ligatures");
        let lig_val = W14::name("val");
        let lig_or = |dom: &Dom, dd: Option<NodeId>| -> String {
            dd.and_then(|d| dom.element(d, &lig_name))
                .and_then(|e| dom.attribute(e, &lig_val).map(str::to_string))
                .unwrap_or_else(|| "none".to_string())
        };
        let a_fonts = a_dd.and_then(|d| dom.element(d, &W::name("rFonts")));
        let b_fonts = b_dd.and_then(|d| dom.element(d, &W::name("rFonts")));
        let fonts_differ = attr_map(dom, a_fonts) != attr_map(dom, b_fonts);
        // NOTE (M476 falsified, M477 evidence): an ascii-family master gate
        // ("same family ⇒ copy verbatim, no bake") recovered tab_test ×
        // table_autofit (+12.8) but broke NINE same-family pairs whose
        // oracles DO bake (nested_comments × math_matrix −35, doc_with_graphs
        // −26, pagination_blank −18.6, …): Word's rule is per-attribute, not
        // per-family — bake exactly the attrs the style's B-side declaration
        // CHAIN does not itself provide (the b_chain_has gates below).
        // M464 — pPr spacing deltas between the two docDefaults (implicit
        // defaults before/after 0, line 240). A B-only style resolving a
        // spacing attr through B's dd renders wrong under A's dd (file_13 ×
        // file_14 oracle bakes after=0 line=240 on every copied style).
        let dd_ppr_spacing = |dom: &Dom, root: NodeId| -> Option<NodeId> {
            let dd = dom.element(root, &W::name("docDefaults"))?;
            let pd = dom.element(dd, &W::name("pPrDefault"))?;
            let pp = dom.element(pd, &W::p_pr())?;
            dom.element(pp, &W::spacing_el())
        };
        let a_sp = dd_ppr_spacing(dom, out_root);
        let b_sp = dd_ppr_spacing(dom, b_root);
        let sp_val = |dom: &Dom, sp: Option<NodeId>, attr: &str, default: &str| -> String {
            sp.and_then(|s| dom.attribute(s, &W::name(attr)).map(str::to_string))
                .unwrap_or_else(|| default.to_string())
        };
        let mut sp_deltas: Vec<(&str, String)> = Vec::new();
        for (attr, default) in [("before", "0"), ("after", "0"), ("line", "240")] {
            let av = sp_val(dom, a_sp, attr, default);
            let bv = sp_val(dom, b_sp, attr, default);
            if av != bv {
                sp_deltas.push((attr, bv));
            }
        }
        let b_line_rule =
            b_sp.and_then(|s| dom.attribute(s, &W::name("lineRule")).map(str::to_string));
        // Output-stylesheet basedOn chains, for "does the declared chain
        // already provide this attr" checks.
        let out_by_id: std::collections::HashMap<String, NodeId> = dom
            .elements(out_root, Some(&style_nm))
            .into_iter()
            .filter_map(|s| dom.attribute(s, &style_id).map(|i| (i.to_string(), s)))
            .collect();
        let chain_has_sp_attr = |dom: &Dom, start: NodeId, attr: &str| -> bool {
            let mut s = start;
            for _ in 0..12 {
                if dom
                    .element(s, &W::p_pr())
                    .and_then(|p| dom.element(p, &W::spacing_el()))
                    .and_then(|sp| dom.attribute(sp, &W::name(attr)))
                    .is_some()
                {
                    return true;
                }
                let Some(based) = dom
                    .element(s, &W::name("basedOn"))
                    .and_then(|b| dom.attribute(b, &W::val()))
                else {
                    return false;
                };
                match out_by_id.get(based) {
                    Some(&n) => s = n,
                    None => return false,
                }
            }
            false
        };
        // Chain-aware rPr check (file_198 × file_199): a copied style whose
        // look is declared somewhere on its B-SIDE basedOn chain (Liberation
        // Serif on B's Normal, later promoted into the output) must not get
        // dd values baked over it — Word renders the chain's declaration.
        // Walk B's chain, not the output's: at bake time the output Normal
        // is still A's (promotion runs later). Mirrors chain_has_sp_attr.
        let b_by_id_for_chain: std::collections::HashMap<String, NodeId> = dom
            .elements(b_root, Some(&style_nm))
            .into_iter()
            .filter_map(|s| dom.attribute(s, &style_id).map(|i| (i.to_string(), s)))
            .collect();
        // M479 — same B-side rule for SPACING attrs: at bake time the output
        // Normal has not yet been promoted, so an out-chain check misses
        // spacing B's chain provides (pci_table Heading7-9 inherit B-Normal's
        // stored line=259; the bake stamped B-dd line=278 over it).
        let b_chain_has_sp_attr = |dom: &Dom, start: NodeId, attr: &str| -> bool {
            let mut s = start;
            for _ in 0..12 {
                if dom
                    .element(s, &W::p_pr())
                    .and_then(|p| dom.element(p, &W::spacing_el()))
                    .and_then(|sp| dom.attribute(sp, &W::name(attr)))
                    .is_some()
                {
                    return true;
                }
                let Some(based) = dom
                    .element(s, &W::name("basedOn"))
                    .and_then(|b| dom.attribute(b, &W::val()))
                else {
                    return false;
                };
                match b_by_id_for_chain.get(based) {
                    Some(&n) => s = n,
                    None => return false,
                }
            }
            false
        };
        let b_chain_has_rpr_elem =
            |dom: &Dom, start: NodeId, name: &crate::xmllinq::XName| -> bool {
                let mut s = start;
                for _ in 0..12 {
                    if dom
                        .element(s, &W::r_pr())
                        .and_then(|r| dom.element(r, name))
                        .is_some()
                    {
                        return true;
                    }
                    let Some(based) = dom
                        .element(s, &W::name("basedOn"))
                        .and_then(|b| dom.attribute(b, &W::val()))
                    else {
                        return false;
                    };
                    match b_by_id_for_chain.get(based) {
                        Some(&n) => s = n,
                        None => return false,
                    }
                }
                false
            };
        // (local, B-effective value) for whitelist props whose A/B effective
        // values differ; implicit defaults sz/szCs 20, kern 0.
        let mut deltas: Vec<(&str, String)> = Vec::new();
        for (local, default) in [("kern", "0"), ("sz", "20"), ("szCs", "20")] {
            let av = val_or(dom, a_dd, local, default);
            let bv = val_or(dom, b_dd, local, default);
            if av != bv {
                deltas.push((local, bv));
            }
        }
        let ligs = (lig_or(dom, a_dd), lig_or(dom, b_dd));
        let b_lang = b_dd.or(a_dd).and_then(|d| dom.element(d, &W::name("lang")));
        if fonts_differ || !deltas.is_empty() || ligs.0 != ligs.1 || !sp_deltas.is_empty() {
            for style in dom.elements(out_root, Some(&style_nm)) {
                // Styles pair by (type, name), never id: the revision's
                // header may be `Encabezado` where the output says `Header`
                // (cda19d51ed), the original's List Paragraph `a4`.
                let Some(key) = style_match_key(dom, style) else {
                    continue;
                };
                if a_declared_keys.contains(&key) {
                    continue;
                }
                let Some(&b_side) = b_by_key.get(&key) else {
                    continue;
                };
                if Some(style) == normal_out {
                    continue;
                }
                // Paragraph styles only — Word leaves the linked *Char
                // styles (FooterChar, Hyperlink, …) unmarked in the oracle.
                let stype = dom.attribute(style, &W::type_()).unwrap_or("").to_string();
                if stype != "paragraph" {
                    continue;
                }
                // Already tracked — leave alone (same rule as phase 1).
                let already = ["rPrChange", "pPrChange"]
                    .iter()
                    .any(|c| !dom.descendants(style, Some(&W::name(c))).is_empty());
                if already {
                    continue;
                }
                // rPr: old = declared + lang; live = declared + delta.
                let declared = dom.element(style, &W::r_pr());
                let old = match declared {
                    Some(r) => dom.clone_subtree(r),
                    None => dom.new_element(W::r_pr()),
                };
                if dom.element(old, &W::name("lang")).is_none()
                    && let Some(l) = b_lang
                {
                    let lc = dom.clone_subtree(l);
                    dom.add(old, lc);
                }
                let live = match declared {
                    Some(r) => r,
                    None => {
                        let r = dom.new_element(W::r_pr());
                        insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                        r
                    }
                };
                let b_side = Some(b_side);
                let b_chain_has = |dom: &Dom, name: &crate::xmllinq::XName| -> bool {
                    b_side.is_some_and(|bs| b_chain_has_rpr_elem(dom, bs, name))
                };
                if fonts_differ
                    && dom.element(live, &W::name("rFonts")).is_none()
                    && !b_chain_has(dom, &W::name("rFonts"))
                    && let Some(bf) = b_fonts
                {
                    let fc = dom.clone_subtree(bf);
                    match dom.elements(live, None).first().copied() {
                        Some(first) => dom.add_before_self(first, fc),
                        None => dom.add(live, fc),
                    }
                }
                for (local, bv) in &deltas {
                    if dom.element(live, &W::name(local)).is_none()
                        && !b_chain_has(dom, &W::name(local))
                    {
                        let e = dom.new_element(W::name(local));
                        dom.set_attribute_value(e, &W::val(), Some(bv));
                        dom.add(live, e);
                    }
                }
                if ligs.0 != ligs.1
                    && dom.element(live, &lig_name).is_none()
                    && !b_chain_has(dom, &lig_name)
                {
                    let e = dom.new_element(lig_name.clone());
                    dom.set_attribute_value(e, &lig_val, Some(&ligs.1));
                    dom.add(live, e);
                }
                append_style_change_record(dom, out_root, live, old, "rPrChange", settings);
                // pPr (paragraph styles): bake B-effective spacing deltas
                // (M464), then record; old = post-bake clone (Word old==live,
                // tiff BodyText oracle).
                if stype == "paragraph" {
                    let ppr = match dom.element(style, &W::p_pr()) {
                        Some(p) => p,
                        None => {
                            let p = dom.new_element(W::p_pr());
                            insert_child_by_rank(dom, style, p, "pPr", &style_child_rank);
                            p
                        }
                    };
                    for (attr, bv) in &sp_deltas {
                        if chain_has_sp_attr(dom, style, attr)
                            || b_side.is_some_and(|bs| b_chain_has_sp_attr(dom, bs, attr))
                        {
                            continue;
                        }
                        let sp = match dom.element(ppr, &W::spacing_el()) {
                            Some(s) => s,
                            None => {
                                let s = dom.new_element(W::spacing_el());
                                insert_child_by_rank(dom, ppr, s, "spacing", &ppr_child_rank);
                                s
                            }
                        };
                        if dom.attribute(sp, &W::name(attr)).is_none() {
                            dom.set_attribute_value(sp, &W::name(attr), Some(bv));
                            if *attr == "line" && dom.attribute(sp, &W::name("lineRule")).is_none()
                            {
                                let rule = b_line_rule.as_deref().unwrap_or("auto");
                                dom.set_attribute_value(sp, &W::name("lineRule"), Some(rule));
                            }
                        }
                    }
                    let old_p = dom.clone_subtree(ppr);
                    for c in dom.descendants(old_p, Some(&W::p_pr_change())) {
                        dom.remove(c);
                    }
                    append_style_change_record(dom, out_root, ppr, old_p, "pPrChange", settings);
                }
                changed = true;
            }
        }
    }
    changed
}

/// M462 — Word's factory docDefaults (current blank-document scaffold).
/// Used when A has no styles part: the output keeps FACTORY defaults (not
/// B's) and every copied B style bakes its B-effective metrics so it still
/// renders as it did in B (tiff_image × two_column oracle).
const FACTORY_DD_RPR: &str = r#"<w:rPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:rFonts w:asciiTheme="minorHAnsi" w:eastAsiaTheme="minorEastAsia" w:hAnsiTheme="minorHAnsi" w:cstheme="minorBidi"/><w:kern w:val="2"/><w:sz w:val="24"/><w:szCs w:val="24"/><w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/><w14:ligatures w14:val="standardContextual"/></w:rPr>"#;
const FACTORY_DD_PPR: &str = r#"<w:pPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:spacing w:after="160" w:line="278" w:lineRule="auto"/></w:pPr>"#;

/// M462 — A has no styles part: rewrite the adopted B stylesheet so its
/// docDefaults become Word's FACTORY defaults, baking each style's
/// B-effective metrics into the style itself. Word renders A's bare
/// paragraphs at factory metrics (sz=24 after=160 line=278) while B-styled
/// content keeps B's look via the per-style bake; adopting B's docDefaults
/// wholesale instead rendered A's content 15% tighter (17 pages vs the
/// oracle's 20 on tiff_image × two_column).
pub(super) fn factory_scaffold_bake_b_styles(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
) -> bool {
    let style_nm = W::name("style");
    let style_id_nm = W::style_id();
    // --- capture B's docDefaults values ---
    let dd = dom.element(root, &W::name("docDefaults"));
    let b_ppr_spacing = dd
        .and_then(|d| dom.element(d, &W::name("pPrDefault")))
        .and_then(|p| dom.element(p, &W::p_pr()))
        .and_then(|p| dom.element(p, &W::spacing_el()));
    let b_sp = |dom: &Dom, attr: &str| -> Option<String> {
        b_ppr_spacing.and_then(|s| dom.attribute(s, &W::name(attr)).map(str::to_string))
    };
    let b_before = b_sp(dom, "before");
    let b_after = b_sp(dom, "after");
    let b_line = b_sp(dom, "line");
    let b_line_rule = b_sp(dom, "lineRule");
    let b_dd_rpr = dd
        .and_then(|d| dom.element(d, &W::name("rPrDefault")))
        .and_then(|r| dom.element(r, &W::r_pr()));
    let b_rv = |dom: &Dom, local: &str| -> Option<String> {
        b_dd_rpr
            .and_then(|r| dom.element(r, &W::name(local)))
            .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
    };
    let b_kern = b_rv(dom, "kern");
    let b_sz = b_rv(dom, "sz");
    let b_sz_cs = b_rv(dom, "szCs");
    let lig_name = W14::name("ligatures");
    let lig_val = W14::name("val");
    let b_lig = b_dd_rpr
        .and_then(|r| dom.element(r, &lig_name))
        .and_then(|e| dom.attribute(e, &lig_val).map(str::to_string));
    let b_fonts_attrs: std::collections::BTreeMap<String, String> = b_dd_rpr
        .and_then(|r| dom.element(r, &W::name("rFonts")))
        .map(|f| {
            dom.attributes(f)
                .into_iter()
                .map(|(n, v)| (format!("{n:?}"), v))
                .collect()
        })
        .unwrap_or_default();
    let b_fonts_node = b_dd_rpr.and_then(|r| dom.element(r, &W::name("rFonts")));
    let b_fonts_clone = b_fonts_node.map(|f| dom.clone_subtree(f));

    // --- replace docDefaults content with factory ---
    if let Some(dd) = dd {
        let rd = dom.element(dd, &W::name("rPrDefault"));
        let pd = dom.element(dd, &W::name("pPrDefault"));
        for e in [rd, pd].into_iter().flatten() {
            dom.remove(e);
        }
        let rp_doc = dom.parse_xdocument(FACTORY_DD_RPR);
        let pp_doc = dom.parse_xdocument(FACTORY_DD_PPR);
        let rd_new = dom.new_element(W::name("rPrDefault"));
        if let Some(r) = dom.root(rp_doc) {
            dom.add(rd_new, r);
        }
        let pd_new = dom.new_element(W::name("pPrDefault"));
        if let Some(p) = dom.root(pp_doc) {
            dom.add(pd_new, p);
        }
        dom.add_first(dd, pd_new);
        dom.add_first(dd, rd_new);
    }
    let factory_fonts: std::collections::BTreeMap<String, String> = [
        ("asciiTheme", "minorHAnsi"),
        ("eastAsiaTheme", "minorEastAsia"),
        ("hAnsiTheme", "minorHAnsi"),
        ("cstheme", "minorBidi"),
    ]
    .iter()
    .map(|(k, v)| (format!("{:?}", W::name(k)), v.to_string()))
    .collect();

    // --- per-style bake ---
    let styles: Vec<NodeId> = dom.elements(root, Some(&style_nm));
    let by_id: std::collections::HashMap<String, NodeId> = styles
        .iter()
        .filter_map(|&s| dom.attribute(s, &style_id_nm).map(|i| (i.to_string(), s)))
        .collect();
    // Value of a declared spacing attr / rPr child anywhere in the basedOn
    // chain (self first). Cycles guarded by a depth cap.
    let chain_spacing = |dom: &Dom, mut s: NodeId, attr: &str| -> Option<String> {
        for _ in 0..12 {
            if let Some(v) = dom
                .element(s, &W::p_pr())
                .and_then(|p| dom.element(p, &W::spacing_el()))
                .and_then(|sp| dom.attribute(sp, &W::name(attr)))
            {
                return Some(v.to_string());
            }
            let based = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))?;
            s = *by_id.get(based)?;
        }
        None
    };
    let chain_rpr = |dom: &Dom, mut s: NodeId, local: &str| -> Option<String> {
        for _ in 0..12 {
            if let Some(v) = dom
                .element(s, &W::r_pr())
                .and_then(|r| dom.element(r, &W::name(local)))
                .and_then(|e| dom.attribute(e, &W::val()))
            {
                return Some(v.to_string());
            }
            let based = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))?;
            s = *by_id.get(based)?;
        }
        None
    };
    let chain_has_rfonts = |dom: &Dom, mut s: NodeId| -> bool {
        for _ in 0..12 {
            if dom
                .element(s, &W::r_pr())
                .and_then(|r| dom.element(r, &W::name("rFonts")))
                .is_some()
            {
                return true;
            }
            let Some(based) = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))
            else {
                return false;
            };
            match by_id.get(based) {
                Some(&n) => s = n,
                None => return false,
            }
        }
        false
    };
    let chain_lig = |dom: &Dom, mut s: NodeId| -> Option<String> {
        for _ in 0..12 {
            if let Some(v) = dom
                .element(s, &W::r_pr())
                .and_then(|r| dom.element(r, &lig_name))
                .and_then(|e| dom.attribute(e, &lig_val))
            {
                return Some(v.to_string());
            }
            let based = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))?;
            s = *by_id.get(based)?;
        }
        None
    };

    let mut changed = false;
    for style in styles {
        let sid = dom.attribute(style, &style_id_nm).unwrap_or("").to_string();
        let stype = dom.attribute(style, &W::type_()).unwrap_or("");
        let is_para = stype == "paragraph";
        let is_char = stype == "character";
        if (!is_para && !is_char) || sid == "Normal" {
            continue;
        }
        let q_format = dom.element(style, &W::name("qFormat")).is_some();
        let old_ppr = dom.element(style, &W::p_pr()).map(|p| {
            let c = dom.clone_subtree(p);
            for ch in dom.descendants(c, Some(&W::p_pr_change())) {
                dom.remove(ch);
            }
            c
        });
        let old_rpr = dom.element(style, &W::r_pr()).map(|r| {
            let c = dom.clone_subtree(r);
            for ch in dom.descendants(c, Some(&W::r_pr_change())) {
                dom.remove(ch);
            }
            c
        });
        let mut touched_ppr = false;
        let mut touched_rpr = false;

        if is_para {
            // spacing: before/after default "0", line default "240".
            let deltas: Vec<(&str, String)> = [
                ("before", &b_before, "0"),
                ("after", &b_after, "160"),
                ("line", &b_line, "278"),
            ]
            .into_iter()
            .filter_map(|(attr, b_dd_v, factory)| {
                if chain_spacing(dom, style, attr).is_some() {
                    return None; // declared chain wins in both contexts
                }
                let default = if attr == "line" { "240" } else { "0" };
                let eff_b = b_dd_v.clone().unwrap_or_else(|| default.to_string());
                let eff_out = factory.to_string();
                (eff_b != eff_out).then_some((attr, eff_b))
            })
            .collect();
            if !deltas.is_empty() {
                let ppr = match dom.element(style, &W::p_pr()) {
                    Some(p) => p,
                    None => {
                        let p = dom.new_element(W::p_pr());
                        insert_child_by_rank(dom, style, p, "pPr", &style_child_rank);
                        p
                    }
                };
                let sp = match dom.element(ppr, &W::spacing_el()) {
                    Some(s) => s,
                    None => {
                        let s = dom.new_element(W::spacing_el());
                        insert_child_by_rank(dom, ppr, s, "spacing", &ppr_child_rank);
                        s
                    }
                };
                for (attr, v) in &deltas {
                    dom.set_attribute_value(sp, &W::name(attr), Some(v));
                    if *attr == "line" && dom.attribute(sp, &W::name("lineRule")).is_none() {
                        let rule = b_line_rule.clone().unwrap_or_else(|| "auto".to_string());
                        dom.set_attribute_value(sp, &W::name("lineRule"), Some(&rule));
                    }
                }
                touched_ppr = true;
            }
        }

        // rPr metric bake (paragraph AND character styles).
        let mut rpr_deltas: Vec<(&str, String)> = Vec::new();
        for (local, b_dd_v, factory, default) in [
            ("kern", &b_kern, "2", "0"),
            ("sz", &b_sz, "24", "20"),
            ("szCs", &b_sz_cs, "24", "20"),
        ] {
            if chain_rpr(dom, style, local).is_some() {
                continue;
            }
            let eff_b = b_dd_v.clone().unwrap_or_else(|| default.to_string());
            if eff_b != factory {
                rpr_deltas.push((local, eff_b));
            }
        }
        let want_fonts = !chain_has_rfonts(dom, style)
            && !b_fonts_attrs.is_empty()
            && b_fonts_attrs != factory_fonts;
        let want_lig = q_format
            && chain_lig(dom, style).is_none()
            && b_lig.clone().unwrap_or_else(|| "none".to_string()) != "standardContextual";
        if !rpr_deltas.is_empty() || want_fonts || want_lig {
            let rpr = match dom.element(style, &W::r_pr()) {
                Some(r) => r,
                None => {
                    let r = dom.new_element(W::r_pr());
                    insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                    r
                }
            };
            if want_fonts
                && dom.element(rpr, &W::name("rFonts")).is_none()
                && let Some(fc) = b_fonts_clone
            {
                let clone = dom.clone_subtree(fc);
                dom.add_first(rpr, clone);
            }
            for (local, v) in &rpr_deltas {
                let e = dom.new_element(W::name(local));
                dom.set_attribute_value(e, &W::val(), Some(v));
                add_rpr_child_in_order(dom, rpr, e, local);
            }
            if want_lig && dom.element(rpr, &lig_name).is_none() {
                let e = dom.new_element(lig_name.clone());
                let v = b_lig.clone().unwrap_or_else(|| "none".to_string());
                dom.set_attribute_value(e, &lig_val, Some(&v));
                dom.add(rpr, e);
            }
            touched_rpr = true;
        }

        // Word marks touched PARAGRAPH styles with both change records; the
        // linked *Char styles stay unmarked (S2 rule).
        if is_para && (touched_ppr || touched_rpr) {
            let already = ["rPrChange", "pPrChange"]
                .iter()
                .any(|c| !dom.descendants(style, Some(&W::name(c))).is_empty());
            if !already {
                let ppr = match dom.element(style, &W::p_pr()) {
                    Some(p) => p,
                    None => {
                        let p = dom.new_element(W::p_pr());
                        insert_child_by_rank(dom, style, p, "pPr", &style_child_rank);
                        p
                    }
                };
                let old_p = old_ppr.unwrap_or_else(|| dom.new_element(W::p_pr()));
                append_style_change_record(dom, root, ppr, old_p, "pPrChange", settings);
                let rpr = match dom.element(style, &W::r_pr()) {
                    Some(r) => r,
                    None => {
                        let r = dom.new_element(W::r_pr());
                        insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                        r
                    }
                };
                let old_r = old_rpr.unwrap_or_else(|| dom.new_element(W::r_pr()));
                append_style_change_record(dom, root, rpr, old_r, "rPrChange", settings);
            }
        }
        changed |= touched_ppr || touched_rpr;
    }
    changed
}

/// M79 — Word-mode single-line normalization on paragraph styles.
///
/// Word Compare writes `line=240 lineRule=auto` onto Heading/Title/ListParagraph
/// spacing when it has rewritten Normal to single-line 0/240 (file_33 oracle).
/// Without it LO inherits docDefaults line=276 after=200 on ListParagraph
/// (ours had empty ListParagraph pPr) and Heading line spacing drifts —
/// 3 pages vs Word 2 for the same body text.
///
/// Gate (critical for file_8): only run when **Normal's live spacing already
/// carries `line`** (post–Normal-merge single-line). file_8 Word leaves
/// Heading1–9 as before/after only (no line); blanket inject regressed −12.
///
/// Rules (when gated on):
/// 1. Heading1–6 / Title / ListParagraph / HighlightedStyle whose spacing
///    lacks `line` get `line=240 lineRule=auto`.
/// 2. Title and ListParagraph with no spacing element get
///    `after=0 line=240 lineRule=auto`.
pub(super) fn normalize_word_paragraph_style_line(dom: &mut Dom, styles_root: NodeId) -> bool {
    // Gate: Normal must already be single-line after merge (has line) AND
    // not a "block spacing" Normal (before>0). file_33: after=0 line=240.
    // file_8: before=480 after=0 — Word leaves Headings without line; our
    // Normal may still carry a stray line attr from earlier merges.
    let Some(normal_sp) = find_normal_style(dom, styles_root)
        .and_then(|n| dom.element(n, &W::p_pr()))
        .and_then(|p| dom.element(p, &W::spacing_el()))
    else {
        return false;
    };
    // M460: only Word's own single-line normalization (line=240) propagates to
    // headings; a Normal carrying B's non-240 line (e.g. 276, basic_comment ×
    // cli_legacy) leaves Heading/Title/ListParagraph line-less in the oracle.
    if dom.attribute(normal_sp, &W::name("line")) != Some("240") {
        return false;
    }
    if let Some(before) = dom.attribute(normal_sp, &W::name("before"))
        && before != "0"
    {
        return false;
    }

    const TOUCH: &[&str] = &[
        "Heading1",
        "Heading2",
        "Heading3",
        "Heading4",
        "Heading5",
        "Heading6",
        "Title",
        "ListParagraph",
        "HighlightedStyle",
    ];

    let mut changed = false;
    let styles: Vec<NodeId> = dom
        .elements(styles_root, Some(&W::name("style")))
        .into_iter()
        .filter(|&s| {
            dom.attribute(s, &W::type_()) == Some("paragraph")
                || dom.attribute(s, &W::type_()).is_none()
        })
        .collect();
    for style in styles {
        let sid = dom
            .attribute(style, &W::style_id())
            .unwrap_or("")
            .to_string();
        if !TOUCH.contains(&sid.as_str()) {
            continue;
        }
        let ppr = match dom.element(style, &W::p_pr()) {
            Some(p) => p,
            None => {
                // Word materializes pPr on Title/ListParagraph even when B is bare.
                if sid != "Title" && sid != "ListParagraph" {
                    continue;
                }
                let p = dom.new_element(W::p_pr());
                // Insert pPr after name/basedOn/next/… but before rPr if present.
                if let Some(rpr) = dom.element(style, &W::r_pr()) {
                    dom.add_before_self(rpr, p);
                } else {
                    dom.add(style, p);
                }
                changed = true;
                p
            }
        };
        if let Some(sp) = dom.element(ppr, &W::spacing_el()) {
            let has_line = dom.attribute(sp, &W::name("line")).is_some();
            if !has_line {
                dom.set_attribute_value(sp, &W::name("line"), Some("240"));
                dom.set_attribute_value(sp, &W::name("lineRule"), Some("auto"));
                changed = true;
            }
        } else if sid == "Title" || sid == "ListParagraph" {
            let sp = dom.new_element(W::spacing_el());
            dom.set_attribute_value(sp, &W::name("after"), Some("0"));
            dom.set_attribute_value(sp, &W::name("line"), Some("240"));
            dom.set_attribute_value(sp, &W::name("lineRule"), Some("auto"));
            // CT_PPrBase order, not "first" and not "just before pPrChange":
            // once workstream S puts B's declared `ind`/`contextualSpacing` on
            // ListParagraph, both of those land spacing after its successors.
            insert_child_by_rank(dom, ppr, sp, "spacing", &ppr_child_rank);
            changed = true;
        }
    }
    changed
}

/// M80 — Word-mode paragraph style rFonts alignment with Normal.
///
/// Word Compare rewrites body paragraph styles so Latin text uses Normal's
/// font (file_33 oracle LO 2pp vs our 3pp with identical spacing):
/// - Title / ListParagraph / HighlightedStyle get full `rFonts` matching
///   Normal when they only store sz (source B bare) — Word materializes
///   Arial on those styles after Normal becomes Arial.
/// - Heading1–6: drop `ascii`/`hAnsi` when they differ from Normal so Latin
///   inherits Normal (Word keeps only eastAsia/cs Calibri on Heading1 while
///   Normal is Arial; we previously forced full Calibri on Headings).
///
/// Without this LO measures headings/lists with Calibri metrics vs Word's
/// Arial and the demo doc spills a third page.
pub(super) fn align_paragraph_style_fonts_with_normal(dom: &mut Dom, styles_root: NodeId) -> bool {
    let Some(normal) = find_normal_style(dom, styles_root) else {
        return false;
    };
    let Some(normal_rpr) = dom.element(normal, &W::r_pr()) else {
        return false;
    };
    let Some(normal_fonts) = dom.element(normal_rpr, &W::name("rFonts")) else {
        return false;
    };
    let Some(normal_ascii) = dom
        .attribute(normal_fonts, &W::name("ascii"))
        .map(|s| s.to_string())
    else {
        return false;
    };
    let normal_font_attrs: Vec<(&str, String)> = ["ascii", "hAnsi", "eastAsia", "cs"]
        .into_iter()
        .filter_map(|a| {
            dom.attribute(normal_fonts, &W::name(a))
                .map(|v| (a, v.to_string()))
        })
        .collect();
    if normal_font_attrs.is_empty() {
        return false;
    }

    const PROMOTE: &[&str] = &["Title", "ListParagraph", "HighlightedStyle"];
    const HEADINGS: &[&str] = &[
        "Heading1", "Heading2", "Heading3", "Heading4", "Heading5", "Heading6",
    ];

    let mut changed = false;
    let styles: Vec<NodeId> = dom
        .elements(styles_root, Some(&W::name("style")))
        .into_iter()
        .filter(|&s| {
            dom.attribute(s, &W::type_()) == Some("paragraph")
                || dom.attribute(s, &W::type_()).is_none()
        })
        .collect();

    for style in styles {
        let sid = dom
            .attribute(style, &W::style_id())
            .unwrap_or("")
            .to_string();
        if PROMOTE.contains(&sid.as_str()) {
            // Only materialize rFonts when the style has none. Do not overwrite
            // theme fonts (file_8 Title = majorHAnsi) or an existing face —
            // Word leaves those alone. file_33 Title/ListParagraph/Highlighted
            // ship with sz-only rPr and no rFonts element.
            let Some(rpr) = dom.element(style, &W::r_pr()) else {
                continue;
            };
            if dom.element(rpr, &W::name("rFonts")).is_some() {
                continue;
            }
            let rf = dom.new_element(W::name("rFonts"));
            for (a, v) in &normal_font_attrs {
                dom.set_attribute_value(rf, &W::name(a), Some(v));
            }
            add_rpr_child_in_order(dom, rpr, rf, "rFonts");
            changed = true;
        } else if HEADINGS.contains(&sid.as_str()) {
            let Some(rpr) = dom.element(style, &W::r_pr()) else {
                continue;
            };
            let Some(rf) = dom.element(rpr, &W::name("rFonts")) else {
                continue;
            };
            let Some(ascii) = dom.attribute(rf, &W::name("ascii")) else {
                continue;
            };
            if ascii == normal_ascii {
                continue;
            }
            // Word: keep eastAsia/cs theme faces; Latin inherits Normal.
            dom.set_attribute_value(rf, &W::name("ascii"), None);
            dom.set_attribute_value(rf, &W::name("hAnsi"), None);
            changed = true;
        }
    }
    changed
}

/// Run-metric keys the footer merge resolves and compares: each rFonts slot
/// as its (explicit, theme) attribute pair, plus sz/szCs values (the
/// properties that set a footer line's box height). A slot's theme attribute
/// overrides its explicit one, so the two resolve together.
const RPR_METRIC_FONT_SLOTS: [(&str, &str); 4] = [
    ("ascii", "asciiTheme"),
    ("hAnsi", "hAnsiTheme"),
    ("eastAsia", "eastAsiaTheme"),
    ("cs", "cstheme"),
];

/// A style tree's `docDefaults/rPrDefault/rPr` node, if present.
/// The line pitch a paragraph with no spacing of its own resolves to under
/// `styles_xml`: the default paragraph style's chain, then docDefaults, then
/// Word's single line (240).
/// Whether a styles part defines a default table style (`TableNormal` in
/// Word-authored documents; generated documents often ship none).
pub(super) fn has_default_table_style(styles_xml: &str) -> bool {
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(styles_xml);
    let Some(root) = dom.root(d) else {
        return false;
    };
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .any(|s| {
            dom.attribute(s, &W::type_()) == Some("table")
                && matches!(dom.attribute(s, &W::name("default")), Some("1" | "true"))
        })
}

pub(super) fn default_paragraph_line(styles_xml: &str) -> String {
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(styles_xml);
    let Some(root) = dom.root(d) else {
        return "240".to_string();
    };
    let style_nm = W::name("style");
    let styles = dom.elements(root, Some(&style_nm));
    let by_id = |id: &str| {
        styles
            .iter()
            .copied()
            .find(|&s| dom.attribute(s, &W::style_id()) == Some(id))
    };
    let line_of = |ppr: Option<NodeId>| {
        ppr.and_then(|p| dom.element(p, &W::spacing_el()))
            .and_then(|sp| dom.attribute(sp, &W::name("line")))
            .map(str::to_string)
    };
    let mut cur = styles.iter().copied().find(|&s| {
        dom.attribute(s, &W::type_()) == Some("paragraph")
            && matches!(dom.attribute(s, &W::name("default")), Some("1" | "true"))
    });
    for _ in 0..12 {
        let Some(s) = cur else { break };
        if let Some(line) = line_of(dom.element(s, &W::p_pr())) {
            return line;
        }
        cur = dom
            .element(s, &W::name("basedOn"))
            .and_then(|b| dom.attribute(b, &W::val()))
            .and_then(by_id);
    }
    let dd = dom
        .element(root, &W::name("docDefaults"))
        .and_then(|d| dom.element(d, &W::name("pPrDefault")))
        .and_then(|d| dom.element(d, &W::p_pr()));
    line_of(dd).unwrap_or_else(|| "240".to_string())
}

fn rpr_default(dom: &Dom, styles_root: NodeId) -> Option<NodeId> {
    let dd = dom.element(styles_root, &W::name("docDefaults"))?;
    let rd = dom.element(dd, &W::name("rPrDefault"))?;
    dom.element(rd, &W::r_pr())
}

/// Normal's EFFECTIVE run metrics: each value from the style's stored rPr
/// when present, else from docDefaults' rPrDefault (per-attribute, the way
/// Word resolves a style chain). Returns each rFonts slot of
/// [`RPR_METRIC_FONT_SLOTS`] as (explicit, theme), then [sz, szCs], each None
/// when defined nowhere. A slot resolves as a pair from the first source that
/// declares either attribute: a theme font (`w:asciiTheme="minorHAnsi"`, the
/// usual docDefaults form) is as much a declaration as a named one.
type FontSlot = (Option<String>, Option<String>);
fn effective_normal_rpr_metrics(
    dom: &Dom,
    styles_root: NodeId,
    normal: Option<NodeId>,
) -> ([FontSlot; 4], [Option<String>; 2]) {
    let stored = normal.and_then(|s| dom.element(s, &W::r_pr()));
    let default = rpr_default(dom, styles_root);
    let font_slot = |(attr, theme): (&str, &str)| -> FontSlot {
        for src in [stored, default] {
            let Some(f) = src.and_then(|r| dom.element(r, &W::name("rFonts"))) else {
                continue;
            };
            let named = dom.attribute(f, &W::name(attr)).map(str::to_string);
            let themed = dom.attribute(f, &W::name(theme)).map(str::to_string);
            if named.is_some() || themed.is_some() {
                return (named, themed);
            }
        }
        (None, None)
    };
    let sz_val = |name: &str| {
        for src in [stored, default] {
            if let Some(v) = src
                .and_then(|r| dom.element(r, &W::name(name)))
                .and_then(|e| dom.attribute(e, &W::val()))
            {
                return Some(v.to_string());
            }
        }
        None
    };
    (
        RPR_METRIC_FONT_SLOTS.map(font_slot),
        [sz_val("sz"), sz_val("szCs")],
    )
}

/// EG_RPrBase child order (wml.xsd `EG_RPrBase` choice sequence). A new rPr
/// child must be inserted immediately after the last existing predecessor so
/// the element stays schema-valid — Word repairs an out-of-order CT_RPr.
const RPR_CHILD_ORDER: &[&str] = &[
    "rStyle",
    "rFonts",
    "b",
    "bCs",
    "i",
    "iCs",
    "caps",
    "smallCaps",
    "strike",
    "dstrike",
    "outline",
    "shadow",
    "emboss",
    "imprint",
    "noProof",
    "snapToGrid",
    "vanish",
    "webHidden",
    "color",
    "spacing",
    "w",
    "kern",
    "position",
    "sz",
    "szCs",
    "highlight",
    "u",
    "effect",
    "bdr",
    "shd",
    "fitText",
    "vertAlign",
    "rtl",
    "cs",
    "em",
    "lang",
    "eastAsianLayout",
    "specVanish",
    "oMath",
];

/// Insert `child` (a new rPr child element named `local`) under `rpr` in
/// EG_RPrBase order: immediately after the last existing predecessor in the
/// schema sequence, or first when no predecessor is present.
pub(super) fn add_rpr_child_in_order(dom: &mut Dom, rpr: NodeId, child: NodeId, local: &str) {
    let new_rank = RPR_CHILD_ORDER
        .iter()
        .position(|&n| n == local)
        .unwrap_or(usize::MAX);
    let existing: Vec<(NodeId, usize)> = dom
        .elements(rpr, None)
        .into_iter()
        .filter_map(|e| {
            let nm = dom.name(e)?;
            let rank = RPR_CHILD_ORDER
                .iter()
                .position(|&n| n == nm.local_name())
                .unwrap_or(usize::MAX);
            Some((e, rank))
        })
        .collect();
    // Insert after the last predecessor (rank < new_rank); fall back to first.
    let anchor = existing
        .iter()
        .rev()
        .find(|(_, rank)| *rank < new_rank)
        .map(|&(e, _)| e);
    match anchor {
        Some(a) => dom.add_after_self(a, child),
        None => dom.add_first(rpr, child),
    }
}

/// M-PAG mechanism 2b / M71: when the output Normal's effective run metrics
/// differ from the REVISED document's, rewrite Normal's rPr to B's effective
/// values with a `w:rPrChange` holding the old rPr. Originally scoped to
/// Copy B's package chrome when the A-based package lacks it.
///
/// Settings/fontTable/webSettings are present on Word redlines whenever the
/// revised side carries them (C3). The theme is not: Word never takes B's
/// (see [`ensure_factory_package_chrome`]).
/// Full docDefaults swap regressed sales_report×sample_document — leave
/// docDefaults to the Normal merge path; only fill **missing** chrome parts.
pub(super) fn adopt_revised_styles_chrome(out: &mut PartFs, pkg2: &PartFs, out_main: &str) {
    // people.xml: Word redlines always carry author identity when B has comments
    // (C2 residual layout for document_100×lots_of_comments).
    const CHROME: [(&str, &str, &str, &str); 4] = [
        (
            "word/settings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
            "settings.xml",
        ),
        (
            "word/webSettings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings",
            "webSettings.xml",
        ),
        (
            "word/fontTable.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable",
            "fontTable.xml",
        ),
        (
            "word/people.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.people+xml",
            "http://schemas.microsoft.com/office/2011/relationships/people",
            "people.xml",
        ),
    ];
    for (part, ctype, rel_type, target) in CHROME {
        if out.part_bytes(part).is_some() {
            continue;
        }
        let Some(bytes) = pkg2.part_bytes(part).map(<[u8]>::to_vec) else {
            continue;
        };
        out.set_part(part, bytes);
        out.add_content_type_override(&format!("/{part}"), ctype);
        let has_rel = out
            .read_rels_for(out_main)
            .is_some_and(|r| r.items.iter().any(|i| i.rel_type == rel_type));
        if !has_rel {
            out.add_document_relationship(out_main, rel_type, target);
        }
    }
}

/// Minimal Word-like package chrome for thin demo packages (C5).
///
/// Word Compare always saves `settings` / `theme` / `fontTable` even when both
/// inputs were bare (styles-only) demos. Without them LO falls back to factory
/// faces that diverge from Word's redline PDF (blue_bold / quarterly_heading
/// / right_aligned_italic class). Inject only when still missing after
/// [`adopt_revised_styles_chrome`].
pub(super) fn ensure_factory_package_chrome(out: &mut PartFs, out_main: &str) {
    // settings
    if out.part_bytes("word/settings.xml").is_none() {
        const SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:zoom w:percent="100"/>
  <w:defaultTabStop w:val="720"/>
  <w:characterSpacingControl w:val="doNotCompress"/>
  <w:compat/>
</w:settings>"#;
        out.set_part("word/settings.xml", SETTINGS.as_bytes().to_vec());
        out.add_content_type_override(
            "/word/settings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
        );
        let has_rel = out
            .read_rels_for(out_main)
            .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/settings")));
        if !has_rel {
            out.add_document_relationship(
                out_main,
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
                "settings.xml",
            );
        }
    }
    // webSettings (Word always writes this; LO uses it for some wrap/compat)
    if out.part_bytes("word/webSettings.xml").is_none() {
        const WEB: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:webSettings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:optimizeForBrowser/>
  <w:allowPNG/>
</w:webSettings>"#;
        out.set_part("word/webSettings.xml", WEB.as_bytes().to_vec());
        out.add_content_type_override(
            "/word/webSettings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml",
        );
        let has_rel = out
            .read_rels_for(out_main)
            .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/webSettings")));
        if !has_rel {
            out.add_document_relationship(
                out_main,
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings",
                "webSettings.xml",
            );
        }
    }
    // fontTable
    if out.part_bytes("word/fontTable.xml").is_none() {
        const FONTS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:font w:name="Calibri"><w:panose1 w:val="020F0502020204030204"/><w:charset w:val="00"/><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>
  <w:font w:name="Times New Roman"><w:panose1 w:val="02020603050405020304"/><w:charset w:val="00"/><w:family w:val="roman"/><w:pitch w:val="variable"/></w:font>
  <w:font w:name="Arial"><w:panose1 w:val="020B0604020202020204"/><w:charset w:val="00"/><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>
</w:fonts>"#;
        out.set_part("word/fontTable.xml", FONTS.as_bytes().to_vec());
        out.add_content_type_override(
            "/word/fontTable.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml",
        );
        let has_rel = out
            .read_rels_for(out_main)
            .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/fontTable")));
        if !has_rel {
            out.add_document_relationship(
                out_main,
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable",
                "fontTable.xml",
            );
        }
    }
    // theme: Word gives an original without a referenced theme its own
    // default theme, never the revision's (an unreferenced theme part does
    // not count).
    if referenced_theme_part(out, out_main).is_none() {
        const THEME_PART: &str = "word/theme/theme1.xml";
        out.set_part(
            THEME_PART,
            crate::word_default_theme::WORD_DEFAULT_THEME
                .as_bytes()
                .to_vec(),
        );
        out.add_content_type_override(
            &format!("/{THEME_PART}"),
            "application/vnd.openxmlformats-officedocument.theme+xml",
        );
        let stale: Vec<String> = out
            .read_rels_for(out_main)
            .map(|r| {
                r.items
                    .iter()
                    .filter(|i| i.rel_type.ends_with("/relationships/theme"))
                    .map(|i| i.rel_type.clone())
                    .collect()
            })
            .unwrap_or_default();
        for rel_type in stale {
            out.remove_relationships_by_type(out_main, &rel_type);
        }
        let target = crate::opc::relative_rel_target(out_main, THEME_PART);
        out.add_document_relationship(out_main, THEME_REL, &target);
    }
}

pub(super) const THEME_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";

/// The theme part `main` references, when the package holds it.
pub(super) fn referenced_theme_part(out: &PartFs, main: &str) -> Option<String> {
    out.read_rels_for(main)?
        .items
        .iter()
        .filter(|r| r.rel_type.ends_with("/relationships/theme"))
        .map(|r| out.resolve_rel_target(main, &r.target))
        .find(|part| out.part_bytes(part).is_some())
}

/// Word-canonical `w:styleId` for a human-readable `w:name` (C3 / C5).
///
/// Word Compare rewrites numeric/`styleN` ids to ECMA-style ids (`heading 1` →
/// `Heading1`, `Normal Table` → `TableNormal`). LO resolves layout from the
/// id for many built-ins; leaving `w:styleId="2"` with `w:name="heading 1"`
/// keeps the correct face in Word but wrong metrics under LO (tolerated-input
/// demos score ~47 with matching body text).
pub(super) fn word_canonical_style_id(name: &str) -> String {
    let n = name.trim();
    // Case-insensitive built-ins (ECMA-376 + Word Compare observations).
    let lower = n.to_ascii_lowercase();
    match lower.as_str() {
        "normal" => return "Normal".into(),
        "heading 1" => return "Heading1".into(),
        "heading 2" => return "Heading2".into(),
        "heading 3" => return "Heading3".into(),
        "heading 4" => return "Heading4".into(),
        "heading 5" => return "Heading5".into(),
        "heading 6" => return "Heading6".into(),
        "heading 7" => return "Heading7".into(),
        "heading 8" => return "Heading8".into(),
        "heading 9" => return "Heading9".into(),
        "default paragraph font" => return "DefaultParagraphFont".into(),
        "normal table" => return "TableNormal".into(),
        "no list" => return "NoList".into(),
        "list paragraph" => return "ListParagraph".into(),
        "footnote text" => return "FootnoteText".into(),
        "footnote reference" => return "FootnoteReference".into(),
        "endnote text" => return "EndnoteText".into(),
        "endnote reference" => return "EndnoteReference".into(),
        "title" => return "Title".into(),
        "subtitle" => return "Subtitle".into(),
        "hyperlink" => return "Hyperlink".into(),
        "strong" => return "Strong".into(),
        "emphasis" => return "Emphasis".into(),
        "quote" => return "Quote".into(),
        "intense quote" => return "IntenseQuote".into(),
        "caption" => return "Caption".into(),
        // Built-ins whose id is not their PascalCased name: the comment
        // styles above all (B's comments.xml keeps `CommentReference`, and
        // renaming it to `AnnotationReference` stranded every reference).
        "annotation text" => return "CommentText".into(),
        "annotation reference" => return "CommentReference".into(),
        "annotation subject" => return "CommentSubject".into(),
        "macro" => return "MacroText".into(),
        "toa heading" => return "TOAHeading".into(),
        "table of figures" => return "TableofFigures".into(),
        "table of authorities" => return "TableofAuthorities".into(),
        "text body" => return "Textbody".into(),
        "preformatted text" => return "PreformattedText".into(),
        "document title" => return "DocumentTitle".into(),
        "highlighted style" => return "HighlightedStyle".into(),
        "red bold character" => return "RedBoldCharacter".into(),
        "blue italic character" => return "BlueItalicCharacter".into(),
        "heading 1 char" => return "Heading1Char".into(),
        "heading 2 char" => return "Heading2Char".into(),
        "heading 3 char" => return "Heading3Char".into(),
        "heading 4 char" => return "Heading4Char".into(),
        "heading 5 char" => return "Heading5Char".into(),
        "heading 6 char" => return "Heading6Char".into(),
        "title char" => return "TitleChar".into(),
        "footnote text char" => return "FootnoteTextChar".into(),
        "default" => return "Default".into(),
        "heading" => return "Heading".into(),
        "list" => return "List".into(),
        "index" => return "Index".into(),
        // Table-of-contents built-ins: styleId is the ALL-CAPS `TOC1`..`TOC9`
        // (name "toc 1"..), which the generic PascalCase below would mangle to
        // `Toc1`. That renames a live built-in to a custom id, so LibreOffice
        // (and Word) drop the built-in TOC indents/dot-leader tabs and the
        // whole table of contents reflows — tanking the visual redline score.
        "toc 1" => return "TOC1".into(),
        "toc 2" => return "TOC2".into(),
        "toc 3" => return "TOC3".into(),
        "toc 4" => return "TOC4".into(),
        "toc 5" => return "TOC5".into(),
        "toc 6" => return "TOC6".into(),
        "toc 7" => return "TOC7".into(),
        "toc 8" => return "TOC8".into(),
        "toc 9" => return "TOC9".into(),
        "toc heading" => return "TOCHeading".into(),
        _ => {}
    }
    // Generic: split on anything but letters and digits (Word ids hold no
    // spaces or punctuation: "Normal (Web)" is NormalWeb), PascalCase each token.
    n.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| {
            let mut cs = t.chars();
            match cs.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
            }
        })
        .collect()
}

/// Word mode: copy the revised stylesheet's styles that the output lacks,
/// pairing styles by type and name as Word does, not by id. A B style whose
/// name the output already holds (Brazilian `Normal` against Dutch
/// `Standaard`, both named "Normal") is not copied; the returned map sends
/// its B id to the output style's id. Styles the names do not pair fall back
/// to the id rule of [`crate::comparer::footnotes::copy_missing_styles`].
pub(super) fn copy_missing_styles_by_name(
    dom: &mut Dom,
    to_root: NodeId,
    from_root: NodeId,
) -> std::collections::HashMap<String, String> {
    let style_nm = W::name("style");
    let style_id = W::style_id();
    let mut by_name: std::collections::HashMap<(String, String), String> =
        std::collections::HashMap::new();
    for s in dom.elements(to_root, Some(&style_nm)) {
        if let (Some(key), Some(id)) = (style_match_key(dom, s), dom.attribute(s, &style_id)) {
            by_name.entry(key).or_insert_with(|| id.to_string());
        }
    }
    let mut b_to_out = std::collections::HashMap::new();
    let mut unpaired = Vec::new();
    for s in dom.elements(from_root, Some(&style_nm)) {
        let paired = style_match_key(dom, s).and_then(|k| by_name.get(&k));
        match (paired, dom.attribute(s, &style_id)) {
            (Some(out_id), Some(b_id)) => {
                if b_id != out_id {
                    b_to_out.insert(b_id.to_string(), out_id.clone());
                }
            }
            _ => unpaired.push(s),
        }
    }
    // Copy the unpaired styles from a scratch stylesheet so the id rule sees
    // only them.
    let scratch = dom.new_element(W::name("styles"));
    for s in unpaired {
        let clone = dom.clone_subtree(s);
        dom.add(scratch, clone);
    }
    crate::comparer::footnotes::copy_missing_styles(dom, to_root, scratch);
    b_to_out
}

/// Rename `w:styleId` values to Word-canonical ids derived from `w:name`.
///
/// Returns the old→new map (only entries that actually change). Also rewrites
/// `basedOn` / `next` / `link` inside the stylesheet. Skips a rename when the
/// target id is already claimed by a different style that is not itself renaming
/// away (no silent merge).
pub(super) fn canonicalize_style_ids(
    dom: &mut Dom,
    styles_root: NodeId,
) -> std::collections::HashMap<String, String> {
    let style_nm = W::name("style");
    let style_id = W::style_id();
    let name_el = W::name("name");
    let styles: Vec<NodeId> = dom.elements(styles_root, Some(&style_nm));

    // Pass 1: desired renames (ignore collisions).
    let mut desired: Vec<(NodeId, String, String)> = Vec::new();
    for s in &styles {
        let Some(old) = dom.attribute(*s, &style_id).map(|v| v.to_string()) else {
            continue;
        };
        let Some(name) = dom
            .element(*s, &name_el)
            .and_then(|e| dom.attribute(e, &W::val()))
            .map(|v| v.to_string())
        else {
            continue;
        };
        let new_id = word_canonical_style_id(&name);
        if new_id.is_empty() || new_id == old {
            continue;
        }
        desired.push((*s, old, new_id));
    }

    // Pass 2: drop collisions — target held by a non-renaming style, or two
    // styles racing for the same target (first wins; prefer already-matching).
    let leaving: std::collections::HashSet<&str> =
        desired.iter().map(|(_, old, _)| old.as_str()).collect();
    let mut taken_targets: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut renames: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut plan: Vec<(NodeId, String)> = Vec::new();
    for (s, old, new_id) in &desired {
        if taken_targets.contains(new_id.as_str()) {
            continue;
        }
        // Occupied by a style that stays put?
        let occupied_by_stayer = styles.iter().any(|&other| {
            dom.attribute(other, &style_id) == Some(new_id.as_str())
                && !leaving.contains(new_id.as_str())
        });
        if occupied_by_stayer {
            continue;
        }
        taken_targets.insert(new_id.as_str());
        renames.insert(old.clone(), new_id.clone());
        plan.push((*s, new_id.clone()));
    }
    for (s, new_id) in &plan {
        dom.set_attribute_value(*s, &style_id, Some(new_id));
    }
    // Rewrite basedOn / next / link vals that point at renamed ids.
    for local in ["basedOn", "next", "link"] {
        let nm = W::name(local);
        for e in dom.descendants(styles_root, Some(&nm)) {
            if let Some(v) = dom.attribute(e, &W::val())
                && let Some(nv) = renames.get(v)
            {
                dom.set_attribute_value(e, &W::val(), Some(nv));
            }
        }
    }
    renames
}

/// Apply a styleId rename map to `pStyle` / `rStyle` / `tblStyle` under `root`.
pub(super) fn remap_style_refs(
    dom: &mut Dom,
    root: NodeId,
    renames: &std::collections::HashMap<String, String>,
) -> usize {
    if renames.is_empty() {
        return 0;
    }
    let mut n = 0;
    for local in ["pStyle", "rStyle", "tblStyle"] {
        let nm = W::name(local);
        for e in dom.descendants(root, Some(&nm)) {
            if let Some(v) = dom.attribute(e, &W::val())
                && let Some(nv) = renames.get(v)
            {
                dom.set_attribute_value(e, &W::val(), Some(nv));
                n += 1;
            }
        }
    }
    n
}

/// Word reads a docDefaults with no pPrDefault as its built-in paragraph
/// defaults, 160 after and 278 auto lines, while a present empty pPrDefault
/// means single spacing (ceaf5f4). Word's redline writes the original's
/// built-in defaults out and tracks the revision's Normal against them
/// (221577c35b), so every style merge below reads the values Word lays out.
pub(super) fn materialize_builtin_paragraph_defaults(dom: &mut Dom, root: NodeId) -> bool {
    let Some(dd) = dom.element(root, &W::name("docDefaults")) else {
        return false;
    };
    if dom.element(dd, &W::name("pPrDefault")).is_some() {
        return false;
    }
    let spacing = dom.new_element(W::spacing_el());
    for (k, v) in [("after", "160"), ("line", "278"), ("lineRule", "auto")] {
        dom.set_attribute_value(spacing, &W::name(k), Some(v));
    }
    let ppr = dom.new_element(W::p_pr());
    dom.add(ppr, spacing);
    let pd = dom.new_element(W::name("pPrDefault"));
    dom.add(pd, ppr);
    dom.add(dd, pd);
    true
}

/// When the A-based styles part lacks `docDefaults` or `latentStyles`, copy
/// them from B (Word redlines always carry both when B has a full stylesheet).
/// Full styles swap is intentionally avoided — it regressed sales_report pairs.
pub(super) fn adopt_missing_styles_structure(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
) -> bool {
    let mut changed = false;
    for local in ["docDefaults", "latentStyles"] {
        let nm = W::name(local);
        if dom.element(out_root, &nm).is_some() {
            continue;
        }
        let Some(src) = dom.element(b_root, &nm) else {
            continue;
        };
        let cloned = dom.clone_subtree(src);
        // docDefaults / latentStyles sort before w:style children.
        if let Some(first_style) = dom.element(out_root, &W::name("style")) {
            dom.add_before_self(first_style, cloned);
        } else {
            dom.add_first(out_root, cloned);
        }
        changed = true;
    }
    changed
}

/// header/footer→Normal (footer knife-edge line box). M71 always runs it in
/// Word mode so no-HF pairs like file_197 also get B's Calibri dd; M65 still
/// skips both-bare Normal (file_170).
///
/// GT evidence (sample-document × sd-2517-localized-heading-styles): GT Normal
/// rPr = Times New Roman sz/szCs 24 + rPrChange(old = Inter sz 22).
/// M467 — prune the merged Normal's live rPr of metric attrs whose value the
/// output docDefaults already supply (Word writes only the delta):
/// `kern` (dd absent = "0"), `sz`/`szCs`, `w14:ligatures` (dd absent =
/// "none"), and per-slot `rFonts` attrs (both concrete and `*Theme` names).
/// Oracle tab_test × table_autofit: B's Normal stores Arial kern=0 sz=20
/// szCs=22 eastAsiaTheme ligatures-none; A-dd has no kern, szCs=22, the same
/// eastAsiaTheme, no ligatures — Word's output Normal is Arial + sz=20 only.
/// basic_comment keeps kern=0 because A-dd kern=2 differs (m461).
pub(super) fn prune_normal_rpr_context_equal_attrs(dom: &mut Dom, out_root: NodeId) -> bool {
    let Some(normal) = find_normal_style(dom, out_root) else {
        return false;
    };
    let Some(rpr) = dom.element(normal, &W::r_pr()) else {
        return false;
    };
    let dd = rpr_default(dom, out_root);
    let dd_val = |dom: &Dom, local: &str, default: &str| -> String {
        dd.and_then(|d| dom.element(d, &W::name(local)))
            .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
            .unwrap_or_else(|| default.to_string())
    };
    let mut changed = false;
    for (local, default) in [("kern", "0"), ("sz", ""), ("szCs", "")] {
        let Some(e) = dom.element(rpr, &W::name(local)) else {
            continue;
        };
        let v = dom.attribute(e, &W::val()).unwrap_or("").to_string();
        let ddv = dd_val(dom, local, default);
        if !v.is_empty() && v == ddv {
            dom.remove(e);
            changed = true;
        }
    }
    let lig_name = W14::name("ligatures");
    let lig_val_name = W14::name("val");
    if let Some(e) = dom.element(rpr, &lig_name) {
        let v = dom.attribute(e, &lig_val_name).unwrap_or("").to_string();
        let ddv = dd
            .and_then(|d| dom.element(d, &lig_name))
            .and_then(|x| dom.attribute(x, &lig_val_name).map(str::to_string))
            .unwrap_or_else(|| "none".to_string());
        if v == ddv {
            dom.remove(e);
            changed = true;
        }
    }
    if let Some(fonts) = dom.element(rpr, &W::name("rFonts")) {
        let dd_fonts = dd.and_then(|d| dom.element(d, &W::name("rFonts")));
        for attr in [
            "ascii",
            "hAnsi",
            "eastAsia",
            "cs",
            "asciiTheme",
            "hAnsiTheme",
            "eastAsiaTheme",
            "cstheme",
        ] {
            let Some(v) = dom.attribute(fonts, &W::name(attr)).map(str::to_string) else {
                continue;
            };
            let ddv = dd_fonts.and_then(|f| dom.attribute(f, &W::name(attr)).map(str::to_string));
            if ddv.as_deref() == Some(v.as_str()) {
                dom.set_attribute_value(fonts, &W::name(attr), None);
                changed = true;
            }
        }
        if !dom.has_attributes(fonts) {
            dom.remove(fonts);
            changed = true;
        }
    }
    changed
}

/// M480b — docDefaults-delta DISABLING neutralizers on both-sides merged
/// styles.
///
/// Word's both-sides algorithm (mined from 132 oracle pairs, 98.7% of the
/// 2022 heading-axis rows): the output keeps A's docDefaults; every style
/// whose formatting blocks differ takes B's declaration plus a tracked
/// redefinition; and Word writes a per-attribute neutralizer for each
/// docDefaults delta the style's post-merge output chain fails to provide
/// (evals__memorandum × evals__nda: every merged heading carries kern 0 +
/// w14:ligatures none against A-dd kern 2 + ligatures). Character styles
/// count the post-merge Normal as a provider — runs resolve docDefaults →
/// paragraph layer → character layer — so a stamped Normal covers them
/// (tab_test × table_autofit oracle). Table styles never take stamps
/// (0/31 oracle rows).
///
/// Only the DISABLING direction fires here (A-dd kerns/ligates, B-dd
/// doesn't): the oracle neutralized 81/81 such pairs. The ENABLING
/// direction has a skip class Word applies whose gate is still unmined
/// (tab_test, table_widths, superdoc_hyperlink_cases) — writing there
/// re-breaks those pairs (the M480a falsification), so it stays out.
pub(super) fn bake_bothsides_dd_disabling_neutralizers(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
) -> bool {
    let kern_name = W::name("kern");
    let lig_name = W14::name("ligatures");
    let lig_val = W14::name("val");
    let dd_out = rpr_default(dom, out_root);
    let dd_b = rpr_default(dom, b_root);
    let kern_of = |dom: &Dom, holder: Option<NodeId>| -> Option<String> {
        holder
            .and_then(|h| dom.element(h, &kern_name))
            .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
    };
    let lig_of = |dom: &Dom, holder: Option<NodeId>| -> Option<String> {
        holder
            .and_then(|h| dom.element(h, &lig_name))
            .and_then(|e| dom.attribute(e, &lig_val).map(str::to_string))
    };
    let do_kern = kern_of(dom, dd_out).is_some_and(|v| v.parse::<i64>().unwrap_or(0) > 0)
        && kern_of(dom, dd_b).is_none_or(|v| v == "0");
    let do_lig = lig_of(dom, dd_out).is_some_and(|v| v != "none")
        && lig_of(dom, dd_b).is_none_or(|v| v == "none");
    if !do_kern && !do_lig {
        return false;
    }

    let style_nm = W::name("style");
    let style_id = W::style_id();
    let type_nm = W::type_();
    let based_nm = W::name("basedOn");
    let normal = find_normal_style(dom, out_root);
    let by_id: std::collections::HashMap<String, NodeId> = dom
        .elements(out_root, Some(&style_nm))
        .into_iter()
        .filter_map(|s| Some((dom.attribute(s, &style_id)?.to_string(), s)))
        .collect();
    let based_of = |dom: &Dom, s: NodeId| -> Option<NodeId> {
        let v = dom
            .element(s, &based_nm)
            .and_then(|b| dom.attribute(b, &W::val()))?;
        by_id.get(v).copied()
    };
    // Nearest live declaration wins; walking direct rPr children only keeps
    // the probe on the live block (change records nest one level deeper).
    let chain_declares = |dom: &Dom, start: NodeId, name: &crate::xmllinq::XName| -> bool {
        let mut cur = Some(start);
        for _ in 0..12 {
            let Some(s) = cur else { break };
            if dom
                .element(s, &W::r_pr())
                .is_some_and(|r| dom.element(r, name).is_some())
            {
                return true;
            }
            cur = based_of(dom, s);
        }
        false
    };

    // Parents first: a stamped ancestor then provides for its descendants.
    let mut styles: Vec<(usize, NodeId)> = dom
        .elements(out_root, Some(&style_nm))
        .into_iter()
        .map(|s| {
            let mut depth = 0usize;
            let mut cur = based_of(dom, s);
            while let Some(p) = cur {
                depth += 1;
                if depth >= 12 {
                    break;
                }
                cur = based_of(dom, p);
            }
            (depth, s)
        })
        .collect();
    styles.sort_by_key(|&(d, _)| d);

    let mut changed = false;
    for (_, style) in styles {
        if Some(style) == normal {
            continue; // Normal takes the M72/M461/M478 merge path
        }
        match dom.attribute(style, &type_nm) {
            Some("paragraph") | Some("character") => {}
            _ => continue,
        }
        // Only styles the compare actually redefined (tracked records from the
        // both-sides merge) participate — untouched styles stay byte-stable.
        let tracked = dom
            .element(style, &W::p_pr())
            .is_some_and(|p| dom.element(p, &W::p_pr_change()).is_some())
            || dom
                .element(style, &W::r_pr())
                .is_some_and(|r| dom.element(r, &W::r_pr_change()).is_some());
        if !tracked {
            continue;
        }
        let is_char = dom.attribute(style, &type_nm) == Some("character");
        for (on, name, mk) in [
            (
                do_kern,
                &kern_name,
                (|dom: &mut Dom| {
                    let e = dom.new_element(W::name("kern"));
                    dom.set_attribute_value(e, &W::val(), Some("0"));
                    e
                }) as fn(&mut Dom) -> NodeId,
            ),
            (do_lig, &lig_name, |dom: &mut Dom| {
                let e = dom.new_element(W14::name("ligatures"));
                dom.set_attribute_value(e, &W14::name("val"), Some("none"));
                e
            }),
        ] {
            if !on || chain_declares(dom, style, name) {
                continue;
            }
            // Character runs resolve through the paragraph layer before the
            // character chain: a declaring post-merge Normal already covers.
            if is_char && normal.is_some_and(|n| chain_declares(dom, n, name)) {
                continue;
            }
            let rpr = match dom.element(style, &W::r_pr()) {
                Some(r) => r,
                None => {
                    let r = dom.new_element(W::r_pr());
                    insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                    r
                }
            };
            let e = mk(dom);
            add_rpr_child_in_order(dom, rpr, e, name.local_name());
            changed = true;
        }
    }
    changed
}

/// Word's redefined paragraph styles hold B's effective metrics as a delta
/// against the output context. Mined from the 747 pool redlines (4,924 tracked
/// paragraph styles present in B): for each rFonts slot, `sz`, `szCs` and the
/// spacing `before`/`after`/`line`, Word writes B's effective value exactly
/// when it differs from what the output style's parent chain and docDefaults
/// resolve, and writes nothing otherwise (rFonts 4,922, sz/szCs/before/after
/// 4,924, line 4,898 of 4,924). B's value resolves through B's own chain and
/// docDefaults, then the factory defaults (sz 20, spacing 0/0/240 auto). A
/// font slot resolves from the nearest rFonts declaring its concrete or theme
/// name. Parents run first, so a child reads its parent's resolved values.
/// B's counterpart is the style of the same (type, name), as Word pairs
/// them: ids name nothing across documents (1b4d's original `a3` is
/// Normal (Web), its revision's `a3` another style).
pub(super) fn resolve_redefined_style_metrics(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
) -> bool {
    let style_nm = W::name("style");
    let index = |dom: &Dom, root: NodeId| -> std::collections::HashMap<String, NodeId> {
        dom.elements(root, Some(&style_nm))
            .into_iter()
            .filter_map(|s| Some((dom.attribute(s, &W::style_id())?.to_string(), s)))
            .collect()
    };
    let out_idx = index(dom, out_root);
    let b_idx = index(dom, b_root);
    let mut b_by_key: std::collections::HashMap<(String, String), NodeId> =
        std::collections::HashMap::new();
    for s in dom.elements(b_root, Some(&style_nm)) {
        if let Some(k) = style_match_key(dom, s) {
            b_by_key.entry(k).or_insert(s);
        }
    }
    let parent = |dom: &Dom, idx: &std::collections::HashMap<String, NodeId>, s: NodeId| {
        dom.element(s, &W::name("basedOn"))
            .and_then(|b| dom.attribute(b, &W::val()))
            .and_then(|v| idx.get(v).copied())
    };
    // Chain from `start` (inclusive) up to 12 styles, then docDefaults.
    let chain = |dom: &Dom,
                 idx: &std::collections::HashMap<String, NodeId>,
                 start: Option<NodeId>|
     -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = start;
        while let Some(c) = cur {
            if out.len() >= 12 || out.contains(&c) {
                break;
            }
            out.push(c);
            cur = parent(dom, idx, c);
        }
        out
    };
    let dd_ppr = |dom: &Dom, root: NodeId| {
        dom.element(root, &W::name("docDefaults"))
            .and_then(|d| dom.element(d, &W::name("pPrDefault")))
            .and_then(|d| dom.element(d, &W::p_pr()))
    };
    // Each holder is a style (reads its pPr/rPr) or a docDefaults pPr/rPr.
    let rpr_of = |dom: &Dom, n: NodeId, is_style: bool| {
        if is_style {
            dom.element(n, &W::r_pr())
        } else {
            Some(n)
        }
    };
    let ppr_of = |dom: &Dom, n: NodeId, is_style: bool| {
        if is_style {
            dom.element(n, &W::p_pr())
        } else {
            Some(n)
        }
    };
    let holders = |dom: &Dom, styles: Vec<NodeId>, root: NodeId, para: bool| {
        let mut h: Vec<(NodeId, bool)> = styles.into_iter().map(|s| (s, true)).collect();
        let d = if para {
            dd_ppr(dom, root)
        } else {
            rpr_default(dom, root)
        };
        if let Some(d) = d {
            h.push((d, false));
        }
        h
    };
    let font_slot = |dom: &Dom, hs: &[(NodeId, bool)], c: &str, t: &str| -> FontSlot {
        for &(n, st) in hs {
            if let Some(f) = rpr_of(dom, n, st).and_then(|r| dom.element(r, &W::name("rFonts"))) {
                let cv = dom.attribute(f, &W::name(c)).map(str::to_string);
                let tv = dom.attribute(f, &W::name(t)).map(str::to_string);
                if cv.is_some() || tv.is_some() {
                    return (cv, tv);
                }
            }
        }
        (None, None)
    };
    let slot_key = |v: &FontSlot| match v {
        (_, Some(t)) => Some(format!("t:{t}")),
        (Some(c), None) => Some(format!("c:{c}")),
        _ => None,
    };
    let run_val = |dom: &Dom, hs: &[(NodeId, bool)], local: &str| -> String {
        hs.iter()
            .find_map(|&(n, st)| {
                rpr_of(dom, n, st)
                    .and_then(|r| dom.element(r, &W::name(local)))
                    .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
            })
            .unwrap_or_else(|| "20".to_string())
    };
    let spacing_val = |dom: &Dom, hs: &[(NodeId, bool)], attr: &str, default: &str| -> String {
        hs.iter()
            .find_map(|&(n, st)| {
                ppr_of(dom, n, st)
                    .and_then(|p| dom.element(p, &W::spacing_el()))
                    .and_then(|e| dom.attribute(e, &W::name(attr)).map(str::to_string))
            })
            .unwrap_or_else(|| default.to_string())
    };
    // (line, lineRule) resolve together from the nearest declaring `line`.
    let line_val = |dom: &Dom, hs: &[(NodeId, bool)]| -> (String, String) {
        hs.iter()
            .find_map(|&(n, st)| {
                let sp = ppr_of(dom, n, st).and_then(|p| dom.element(p, &W::spacing_el()))?;
                let line = dom.attribute(sp, &W::name("line"))?.to_string();
                let rule = dom
                    .attribute(sp, &W::name("lineRule"))
                    .unwrap_or("auto")
                    .to_string();
                Some((line, rule))
            })
            .unwrap_or_else(|| ("240".to_string(), "auto".to_string()))
    };

    let mut styles: Vec<(usize, NodeId)> = dom
        .elements(out_root, Some(&style_nm))
        .into_iter()
        .map(|s| (chain(dom, &out_idx, Some(s)).len(), s))
        .collect();
    styles.sort_by_key(|&(d, _)| d);
    let mut changed = false;
    for (_, style) in styles {
        if dom.attribute(style, &W::type_()) != Some("paragraph")
            || dom
                .attribute(style, &W::name("default"))
                .is_some_and(|v| v == "1" || v == "true")
        {
            continue;
        }
        let tracked = dom
            .element(style, &W::p_pr())
            .is_some_and(|p| dom.element(p, &W::p_pr_change()).is_some())
            || dom
                .element(style, &W::r_pr())
                .is_some_and(|r| dom.element(r, &W::r_pr_change()).is_some());
        let Some(&b_style) = style_match_key(dom, style).and_then(|k| b_by_key.get(&k)) else {
            continue;
        };
        if !tracked {
            continue;
        }
        let b_chain = chain(dom, &b_idx, Some(b_style));
        let o_chain = chain(dom, &out_idx, parent(dom, &out_idx, style));
        let b_r = holders(dom, b_chain.clone(), b_root, false);
        let o_r = holders(dom, o_chain.clone(), out_root, false);
        let b_p = holders(dom, b_chain, b_root, true);
        let o_p = holders(dom, o_chain, out_root, true);

        // --- run metrics ---
        let fonts: Vec<(FontSlot, bool)> = RPR_METRIC_FONT_SLOTS
            .iter()
            .map(|(c, t)| {
                let bv = font_slot(dom, &b_r, c, t);
                let differs = slot_key(&bv) != slot_key(&font_slot(dom, &o_r, c, t));
                (bv, differs)
            })
            .collect();
        let sizes: Vec<(&str, String, bool)> = ["sz", "szCs"]
            .into_iter()
            .map(|l| {
                let bv = run_val(dom, &b_r, l);
                let differs = bv != run_val(dom, &o_r, l);
                (l, bv, differs)
            })
            .collect();
        let need_rpr =
            fonts.iter().any(|f| f.1 && slot_key(&f.0).is_some()) || sizes.iter().any(|s| s.2);
        let rpr = match dom.element(style, &W::r_pr()) {
            Some(r) => Some(r),
            None if need_rpr => {
                let r = dom.new_element(W::r_pr());
                insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                Some(r)
            }
            None => None,
        };
        if let Some(rpr) = rpr {
            // A missing rFonts is created only when a slot will be written.
            let rf = dom.element(rpr, &W::name("rFonts")).or_else(|| {
                fonts
                    .iter()
                    .any(|(bv, differs)| *differs && (bv.0.is_some() || bv.1.is_some()))
                    .then(|| {
                        let f = dom.new_element(W::name("rFonts"));
                        dom.add_first(rpr, f);
                        f
                    })
            });
            if let Some(rf) = rf {
                for ((c, t), (bv, differs)) in RPR_METRIC_FONT_SLOTS.iter().zip(&fonts) {
                    let before = (
                        dom.attribute(rf, &W::name(c)).map(str::to_string),
                        dom.attribute(rf, &W::name(t)).map(str::to_string),
                    );
                    let want = if *differs { bv.clone() } else { (None, None) };
                    if before != want {
                        dom.set_attribute_value(rf, &W::name(c), want.0.as_deref());
                        dom.set_attribute_value(rf, &W::name(t), want.1.as_deref());
                        changed = true;
                    }
                }
                if !dom.has_attributes(rf) {
                    dom.remove(rf);
                }
            }
            for (local, bv, differs) in sizes {
                let existing = dom.element(rpr, &W::name(local));
                match (existing, differs) {
                    (Some(e), false) => {
                        dom.remove(e);
                        changed = true;
                    }
                    (Some(e), true) => {
                        if dom.attribute(e, &W::val()) != Some(bv.as_str()) {
                            dom.set_attribute_value(e, &W::val(), Some(&bv));
                            changed = true;
                        }
                    }
                    (None, true) => {
                        let e = dom.new_element(W::name(local));
                        dom.set_attribute_value(e, &W::val(), Some(&bv));
                        add_rpr_child_in_order(dom, rpr, e, local);
                        changed = true;
                    }
                    (None, false) => {}
                }
            }
        }

        // --- spacing ---
        let mut want: Vec<(&str, Option<String>)> = Vec::new();
        for (attr, default) in [("before", "0"), ("after", "0")] {
            let bv = spacing_val(dom, &b_p, attr, default);
            let differs = bv != spacing_val(dom, &o_p, attr, default);
            want.push((attr, differs.then_some(bv)));
        }
        let (bl, br) = line_val(dom, &b_p);
        let line_differs = (bl.clone(), br.clone()) != line_val(dom, &o_p);
        want.push(("line", line_differs.then(|| bl.clone())));
        want.push(("lineRule", line_differs.then(|| br.clone())));
        let need_spacing = want.iter().any(|w| w.1.is_some());
        let ppr = match dom.element(style, &W::p_pr()) {
            Some(p) => Some(p),
            None if need_spacing => {
                let p = dom.new_element(W::p_pr());
                insert_child_by_rank(dom, style, p, "pPr", &style_child_rank);
                Some(p)
            }
            None => None,
        };
        let Some(ppr) = ppr else { continue };
        let sp = match dom.element(ppr, &W::spacing_el()) {
            Some(sp) => Some(sp),
            None if need_spacing => {
                let sp = dom.new_element(W::spacing_el());
                insert_child_by_rank(dom, ppr, sp, "spacing", &ppr_child_rank);
                Some(sp)
            }
            None => None,
        };
        let Some(sp) = sp else { continue };
        for (attr, v) in want {
            if dom.attribute(sp, &W::name(attr)) != v.as_deref() {
                dom.set_attribute_value(sp, &W::name(attr), v.as_deref());
                changed = true;
            }
        }
        if !dom.has_attributes(sp) {
            dom.remove(sp);
        }
    }
    changed
}

pub(super) fn merge_normal_style_rpr(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
    settings: &WmlComparerSettings,
) -> bool {
    let Some(a_style) = find_normal_style(dom, out_root) else {
        return false;
    };
    let b_style = find_normal_style(dom, b_root);
    // M65: when both Normals lack stored rPr, Word leaves Normal bare (file_170:
    // A bare + B bare + differing dd fonts → empty Normal, not Calibri+rPrChange).
    // Only materialize B's effective run metrics when a side already stores rPr
    // on Normal (footer knife-edge cases with explicit Normal rPr).
    // file_103 × file_104 refines the gate: a stored pPr counts too (A bare,
    // B pPr only → Word writes B's run defaults). 227 corpus Word redlines
    // agree: with either side structured and differing run defaults, Word
    // always writes Normal rPr.
    let structured =
        |s: NodeId| dom.element(s, &W::r_pr()).is_some() || dom.element(s, &W::p_pr()).is_some();
    if !structured(a_style) && !b_style.is_some_and(structured) {
        return false;
    }
    let b_effective = effective_normal_rpr_metrics(dom, b_root, b_style);
    if effective_normal_rpr_metrics(dom, out_root, Some(a_style)) == b_effective {
        return false;
    }
    // Old value = A's stored rPr when present, else A's docDefaults rPr
    // content (Word records the docDefaults-resolved old value — GT's
    // rPrChange holds Inter sz=22 + lang, A's rPrDefault verbatim). rPrChange's
    // inner rPr must not itself carry an rPrChange (CT_RPr violation Word
    // repairs/drops), so strip nested change history from the clone.
    let old_rpr = match dom
        .element(a_style, &W::r_pr())
        .or_else(|| rpr_default(dom, out_root))
    {
        Some(r) => {
            let clone = dom.clone_subtree(r);
            for c in dom.descendants(clone, Some(&W::r_pr_change())) {
                dom.remove(c);
            }
            clone
        }
        None => dom.new_element(W::r_pr()),
    };
    let rpr = match dom.element(a_style, &W::r_pr()) {
        Some(r) => r,
        None => {
            let r = dom.new_element(W::r_pr());
            // rPr follows pPr in CT_Style; Normal's remaining children
            // (name/qFormat/pPr) all precede it, so append.
            dom.add(a_style, r);
            r
        }
    };
    let (font_slots, [sz, sz_cs]) = b_effective;
    let fonts = match dom.element(rpr, &W::name("rFonts")) {
        Some(f) => f,
        None => {
            let f = dom.new_element(W::name("rFonts"));
            dom.add_first(rpr, f);
            f
        }
    };
    for ((attr, theme), (named, themed)) in RPR_METRIC_FONT_SLOTS.iter().zip(&font_slots) {
        dom.set_attribute_value(fonts, &W::name(attr), named.as_deref());
        dom.set_attribute_value(fonts, &W::name(theme), themed.as_deref());
    }
    // sz/szCs must follow EG_RPrBase order (rFonts < b..webHidden < color <
    // spacing < w < kern < position < sz < szCs). Anchoring them to rFonts —
    // as this code previously did — places them before any of color/spacing/
    // w/kern/position Normal already carries, breaking CT_RPr order and
    // tripping Word's repair. Insert after the last existing predecessor so
    // the new/updated sz/szCs land in their schema slot.
    for (name, v) in [("sz", &sz), ("szCs", &sz_cs)] {
        let existing = dom.element(rpr, &W::name(name));
        // CT_HpsMeasure makes w:val REQUIRED. Creating the element and then
        // passing None stripped the attribute but left `<w:szCs/>` behind —
        // not "no size", but XML Word refuses to open (Sch_MissRequiredAttribute
        // at styles.xml w:style[1]/w:rPr/w:szCs). ECMA-376 spells "no value"
        // here as the element's absence, so that is what we write.
        // B silent while A's docDefaults set a size: removing ours would
        // inherit A's, so Word writes the implicit 20 half-points (17 of 17
        // corpus Word redlines).
        let a_dd_size = rpr_default(dom, out_root)
            .and_then(|r| dom.element(r, &W::name(name)))
            .is_some();
        let Some(val) = v.as_deref().or(a_dd_size.then_some("20")) else {
            if let Some(e) = existing {
                dom.remove(e);
            }
            continue;
        };
        let e = match existing {
            Some(e) => e,
            None => {
                let e = dom.new_element(W::name(name));
                add_rpr_child_in_order(dom, rpr, e, name);
                e
            }
        };
        dom.set_attribute_value(e, &W::val(), Some(val));
    }
    // M461 generalized by file_198 — Word's live Normal is B's FULL
    // effective rPr: every stored B child survives (color, lang — not just
    // kern/ligatures), and metrics B keeps in its docDefaults (kern,
    // w14:ligatures) are materialized when the two documents' docDefaults
    // disagree on them (oracle Normal: Liberation fonts + color 00000A +
    // kern 2 + lang zh-CN/hi-IN + ligatures with A-dd declaring none).
    let lig_name = W14::name("ligatures");
    let b_rpr = b_style.and_then(|s| dom.element(s, &W::r_pr()));
    let metric = |n: &crate::xmllinq::XName| {
        ["rFonts", "sz", "szCs", "lang", "kern", "rPrChange"]
            .iter()
            .any(|s| *n == W::name(s))
    };
    // d8b0c2ae01: a property only A's Normal stores (color 0000FF) is not
    // B's; Word records it in the rPrChange and leaves it off the live rPr,
    // or writes B's docDefaults value when B declares one there.
    for ac in dom.elements(rpr, None) {
        let Some(n) = dom.name(ac) else { continue };
        if metric(&n) || n != W::name(n.local_name()) {
            continue;
        }
        if b_rpr.is_some_and(|b| dom.element(b, &n).is_some()) {
            continue;
        }
        match rpr_default(dom, b_root).and_then(|r| dom.element(r, &n)) {
            Some(bd) => {
                let clone = dom.clone_subtree(bd);
                dom.replace_with(ac, &[clone]);
            }
            None => dom.remove(ac),
        }
    }
    if let Some(b_rpr) = b_rpr {
        let b_kids: Vec<NodeId> = dom.elements(b_rpr, None);
        for bc in b_kids {
            let Some(n) = dom.name(bc) else { continue };
            if metric(&n) && n != W::name("kern") {
                continue; // metric slots written above
            }
            if n == lig_name {
                if dom.element(rpr, &lig_name).is_none() {
                    let clone = dom.clone_subtree(bc);
                    dom.add(rpr, clone); // w14 extension: last, pre-rPrChange
                }
                continue;
            }
            let local = n.local_name().to_string();
            let clone = dom.clone_subtree(bc);
            match dom.element(rpr, &W::name(&local)) {
                // B's value wins over A's (Word's live Normal is B's).
                Some(old) => dom.replace_with(old, &[clone]),
                None => add_rpr_child_in_order(dom, rpr, clone, &local),
            }
        }
    }
    // dd-held metrics: materialize B's kern/ligatures when the docDefaults
    // disagree and neither the merged rPr nor B's stored rPr carries them.
    {
        let dd_elem = |dom: &Dom, root: NodeId, name: &crate::xmllinq::XName| -> Option<NodeId> {
            rpr_default(dom, root).and_then(|r| dom.element(r, name))
        };
        // A B with no docDefaults at all reads with Word's factory run
        // defaults: kern 2 and standard contextual ligatures.
        let b_factory = dom.element(b_root, &W::name("docDefaults")).is_none();
        let kern_name = W::name("kern");
        let a_kern = dd_elem(dom, out_root, &kern_name)
            .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string));
        let b_kern = if b_factory {
            Some("2".to_string())
        } else {
            dd_elem(dom, b_root, &kern_name)
                .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
        };
        // B-dd lacks kern (implicit 0) while A-dd kerns: Word materializes
        // the neutralizer — kern 2 left live wraps every long line
        // differently (list_numbering × list_spacer1 oracle: effective kern 0
        // on all 14 Normal-based styles).
        if a_kern != b_kern && dom.element(rpr, &kern_name).is_none() {
            let e = dom.new_element(kern_name);
            dom.set_attribute_value(e, &W::val(), Some(b_kern.as_deref().unwrap_or("0")));
            add_rpr_child_in_order(dom, rpr, e, "kern");
        }
        let a_lig = dd_elem(dom, out_root, &lig_name)
            .and_then(|e| dom.attribute(e, &W14::name("val")).map(str::to_string));
        let b_lig = if b_factory {
            Some("standardContextual".to_string())
        } else {
            dd_elem(dom, b_root, &lig_name)
                .and_then(|e| dom.attribute(e, &W14::name("val")).map(str::to_string))
        };
        if a_lig != b_lig && dom.element(rpr, &lig_name).is_none() {
            let e = dom.new_element(lig_name.clone());
            dom.set_attribute_value(
                e,
                &W14::name("val"),
                Some(b_lig.as_deref().unwrap_or("none")),
            );
            dom.add(rpr, e);
        }
        // Language: only the attributes B's effective value changes against
        // A's docDefaults, a missing one read as Word's en-US / en-US / ar-SA
        // (159 of 161 corpus Word redlines).
        const IMPLICIT_LANG: [(&str, &str); 3] =
            [("val", "en-US"), ("eastAsia", "en-US"), ("bidi", "ar-SA")];
        let lang_name = W::name("lang");
        let lang_attr = |dom: &Dom, lang: Option<NodeId>, attr: &str| {
            lang.and_then(|l| dom.attribute(l, &W::name(attr)).map(str::to_string))
        };
        let a_lang = dd_elem(dom, out_root, &lang_name);
        let b_lang = b_style
            .and_then(|s| dom.element(s, &W::r_pr()))
            .and_then(|r| dom.element(r, &lang_name));
        let b_dd_lang = dd_elem(dom, b_root, &lang_name);
        let delta: Vec<(&str, String)> = IMPLICIT_LANG
            .iter()
            .filter_map(|&(attr, implicit)| {
                let b = lang_attr(dom, b_lang, attr)
                    .or_else(|| lang_attr(dom, b_dd_lang, attr))
                    .unwrap_or_else(|| implicit.to_string());
                let a = lang_attr(dom, a_lang, attr).unwrap_or_else(|| implicit.to_string());
                (a != b).then_some((attr, b))
            })
            .collect();
        if let Some(old) = dom.element(rpr, &lang_name) {
            dom.remove(old);
        }
        if !delta.is_empty() {
            let e = dom.new_element(lang_name);
            for (attr, v) in &delta {
                dom.set_attribute_value(e, &W::name(attr), Some(v));
            }
            add_rpr_child_in_order(dom, rpr, e, "lang");
        }
    }
    let chg = dom.new_element(W::r_pr_change());
    // The pPr pass, when it fired, gave Normal a pPrChange; this record
    // shares its id, as Word's do (see `style_record_id`).
    let id = style_record_id(dom, out_root, rpr, settings);
    dom.set_attribute_value(chg, &W::id(), Some(&id));
    dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add(chg, old_rpr);
    dom.add(rpr, chg); // rPrChange is last in CT_RPr
    true
}

/// M483 — re-cache `w:color` hex values against the package's theme.
///
/// A `w:color` with `w:themeColor` renders by its cached `w:val` hex, not by
/// live theme resolution (LO and Word both paint the cache). Styles brought
/// over from B were cached under B's theme; the output ships A's theme, so
/// every themed color is visibly wrong until re-cached. Word's oracle
/// re-resolves (tab_test H1Char: B declaration kept but val rewritten to
/// A-theme accent1 shade BF = 365F91). Change-record baselines resolve to
/// the same theme, so rewriting every color element is a no-op for them.
pub(super) fn reresolve_theme_color_hexes(
    dom: &mut Dom,
    styles_root: NodeId,
    theme_xml: &str,
) -> bool {
    let slot_hex = |slot: &str| -> Option<String> {
        let i = theme_xml.find(&format!("<a:{slot}>"))?;
        let seg = &theme_xml[i..theme_xml[i..]
            .find(&format!("</a:{slot}>"))
            .map_or(theme_xml.len(), |e| i + e)];
        let hex = if let Some(j) = seg.find("srgbClr val=\"") {
            &seg[j + 13..j + 19]
        } else {
            let j = seg.find("lastClr=\"")?;
            &seg[j + 9..j + 15]
        };
        Some(hex.to_uppercase())
    };
    let slot_of = |name: &str| -> Option<&'static str> {
        Some(match name {
            "accent1" => "a:accent1",
            "accent2" => "a:accent2",
            "accent3" => "a:accent3",
            "accent4" => "a:accent4",
            "accent5" => "a:accent5",
            "accent6" => "a:accent6",
            "text1" => "a:dk1",
            "text2" => "a:dk2",
            "background1" => "a:lt1",
            "background2" => "a:lt2",
            "hyperlink" => "a:hlink",
            "followedHyperlink" => "a:folHlink",
            _ => return None,
        })
    };
    // Word applies themeShade/themeTint as HSL LUMINANCE scaling, not RGB
    // multiply (linear RGB would give accent1 4F81BD shade BF = 3B618E), and
    // truncates each channel: 156082 tint 3F = B2DEF2, shade BF = 0F4761
    // (139 Word samples: 86 exact, the rest one step off; rounding gets 22).
    let apply = |hex: &str, factor: &str, toward_white: bool| -> Option<String> {
        let f = u32::from_str_radix(factor, 16).ok()? as f64 / 255.0;
        let r = u32::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
        let g = u32::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
        let b = u32::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
        let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
        let l = (mx + mn) / 2.0;
        let (h, s) = if (mx - mn).abs() < 1e-9 {
            (0.0, 0.0)
        } else {
            let d = mx - mn;
            let s = if l > 0.5 {
                d / (2.0 - mx - mn)
            } else {
                d / (mx + mn)
            };
            let h = if (mx - r).abs() < 1e-9 {
                ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
            } else if (mx - g).abs() < 1e-9 {
                ((b - r) / d + 2.0) / 6.0
            } else {
                ((r - g) / d + 4.0) / 6.0
            };
            (h, s)
        };
        let l2 = if toward_white {
            1.0 - (1.0 - l) * f
        } else {
            l * f
        };
        let hue = |p: f64, q: f64, mut t: f64| -> f64 {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };
        let (r2, g2, b2) = if s.abs() < 1e-9 {
            (l2, l2, l2)
        } else {
            let q = if l2 < 0.5 {
                l2 * (1.0 + s)
            } else {
                l2 + s - l2 * s
            };
            let p = 2.0 * l2 - q;
            (
                hue(p, q, h + 1.0 / 3.0),
                hue(p, q, h),
                hue(p, q, h - 1.0 / 3.0),
            )
        };
        Some(
            [r2, g2, b2]
                .iter()
                .map(|c| format!("{:02X}", (c * 255.0).floor().clamp(0.0, 255.0) as u32))
                .collect(),
        )
    };

    // Every themed hex a style carries, not only `w:color`'s: shading
    // colours and fills and border colours are re-cached too (2e3f1e26,
    // 512b24be: Word's redline leaves none stale).
    let color_name = W::name("color");
    let slots = [
        ("themeColor", "themeShade", "themeTint", "color"),
        ("themeFill", "themeFillShade", "themeFillTint", "fill"),
    ]
    .map(|(t, sh, ti, hex)| (W::name(t), W::name(sh), W::name(ti), W::name(hex)));
    let mut changed = false;
    for e in dom.descendants(styles_root, None) {
        for (theme_attr, shade_attr, tint_attr, hex_attr) in &slots {
            let Some(name) = dom.attribute(e, theme_attr).map(str::to_string) else {
                continue;
            };
            let Some(slot) = slot_of(&name) else { continue };
            let Some(base) = slot_hex(&slot[2..]) else {
                continue;
            };
            let hex_attr = if dom.name(e).as_ref() == Some(&color_name) {
                W::val()
            } else {
                hex_attr.clone()
            };
            let Some(cur) = dom.attribute(e, &hex_attr).map(str::to_uppercase) else {
                continue;
            };
            let expect = if let Some(sh) = dom.attribute(e, shade_attr).map(str::to_string) {
                apply(&base, &sh, false)
            } else if let Some(ti) = dom.attribute(e, tint_attr).map(str::to_string) {
                apply(&base, &ti, true)
            } else {
                Some(base)
            };
            let Some(expect) = expect else { continue };
            // rounding tolerance: correctly-cached values may differ by ±2 per
            // channel from our HSL math — leave those (they're already right);
            // only genuinely stale caches (different theme) get rewritten.
            let close = cur.len() == 6
                && (0..3).all(|i| {
                    let a = u32::from_str_radix(&cur[i * 2..i * 2 + 2], 16).unwrap_or(999);
                    let b = u32::from_str_radix(&expect[i * 2..i * 2 + 2], 16).unwrap_or(0);
                    a.abs_diff(b) <= 2
                });
            if !close && cur != "AUTO" {
                dom.set_attribute_value(e, &hex_attr, Some(&expect));
                changed = true;
            }
        }
    }
    changed
}
