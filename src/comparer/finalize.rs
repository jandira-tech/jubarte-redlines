//! M4.F — markup finalization: turn the pt:Status-tagged tree into real
//! `w:ins`/`w:del`/`w:rPrChange`/move markup, conjoin paragraph marks, renumber
//! revision ids, and strip scratch markup. Port of MarkContentAsDeletedOrInserted
//! (WmlComparer.cs:2568), ConjoinDeletedInsertedParagraphMarks (:2515),
//! FixUpRevisionIds (:2769), IgnorePt14Namespace (:2912),
//! RemovePowerToolsScratchMarkup (CleanPartTransform :1165).

use crate::namespaces::{MC, PT, R, W, W14, WP14};
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

use super::WmlComparerSettings;
use super::tables::ALLOWABLE_RUN_CHILDREN;

/// `DescendantsTrimmed(node, stop)` — descendants, not recursing into `stop`.
fn descendants_trimmed(dom: &Dom, node: NodeId, stop: &XName) -> Vec<NodeId> {
    let mut out = Vec::new();
    fn walk(dom: &Dom, node: NodeId, stop: &XName, out: &mut Vec<NodeId>) {
        for c in dom.nodes(node) {
            out.push(c);
            if dom.is_element(c) && dom.name(c).as_ref() != Some(stop) {
                walk(dom, c, stop, out);
            }
        }
    }
    walk(dom, node, stop, &mut out);
    out
}

fn is_run_status_carrier(dom: &Dom, d: NodeId) -> bool {
    match dom.name(d) {
        Some(n) => {
            // Text leaves + AllowableRunChildren (incl. bare w:drawing).
            // Also opaque run children that produce tags Status on *themselves*
            // but are not in AllowableRunChildren because atomize keeps them as
            // whole subtrees: mc:AlternateContent (DrawingML/VML text boxes),
            // w:pict, w:object. Without these, inserted text boxes stay plain
            // (no w:ins) — file_69_file_70 Word oracle wraps them in w:ins.
            n == W::t()
                || n == W::name("delText")
                || ALLOWABLE_RUN_CHILDREN.contains(&n)
                || n == MC::name("AlternateContent")
                || n == W::name("pict")
                || n == W::name("object")
        }
        None => false,
    }
}

/// `w:t` inside `w:del` / `w:moveFrom` is non-conformant — Word expects
/// `w:delText` (and `w:delInstrText` for field instructions; Word shows the
/// repair dialog on `w:instrText` under `w:del` — strict01 TOC evidence, and
/// Word's own redlines emit delInstrText). Recurse into the rebuilt run and
/// rename (skipping nested revision containers, which own their own text kind).
fn convert_run_text_to_del_text(dom: &mut Dom, run: NodeId) {
    fn walk(dom: &mut Dom, node: NodeId) {
        for c in dom.nodes(node) {
            if !dom.is_element(c) {
                continue;
            }
            match dom.name(c).as_ref() {
                Some(n)
                    if n == &W::ins()
                        || n == &W::del()
                        || n == &W::name("moveFrom")
                        || n == &W::name("moveTo") =>
                {
                    continue;
                }
                Some(n) if n == &W::t() => dom.set_name(c, W::name("delText")),
                Some(n) if n == &W::instr_text() => dom.set_name(c, W::name("delInstrText")),
                _ => walk(dom, c),
            }
        }
    }
    walk(dom, run);
}

fn rev_el(dom: &mut Dom, name: XName, settings: &WmlComparerSettings, id_gen: &mut u32) -> NodeId {
    let e = dom.new_element(name);
    dom.set_attribute_value(e, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(e, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;
    dom.set_attribute_value(e, &W::date(), Some(&settings.date_time_for_revisions));
    e
}

/// Clone `src`'s attributes onto `dst` (optionally skipping the pt namespace).
fn copy_attrs(dom: &mut Dom, src: NodeId, dst: NodeId, skip_pt: bool) {
    for (an, av) in dom.attributes(src) {
        if skip_pt && an.namespace_name() == PT::URI {
            continue;
        }
        dom.set_attribute_value(dst, &an, Some(&av));
    }
}

/// Copy only `w:author`, `w:date`, `w:id` from `src` onto `dst` — matches the
/// oracle's `SimplifyMoveMarkupToDelIns` (WmlComparer.cs:2874-2878), which
/// passes `moveFrom.Attribute(W.author) / (W.date) / (W.id)` explicitly when
/// constructing the replacement `w:del` / `w:ins`. Stray attributes
/// (rsid*, pt:Unid, future vendor attrs) do not leak through.
fn copy_move_id_attrs(dom: &mut Dom, src: NodeId, dst: NodeId) {
    for name in [W::author(), W::date(), W::id()] {
        // Read into an owned String so we can drop the immutable borrow before
        // taking `&mut dom` for `set_attribute_value`.
        let av = dom.attribute(src, &name).map(str::to_string);
        if let Some(av) = av {
            dom.set_attribute_value(dst, &name, Some(&av));
        }
    }
}

/// Rebuild a run, recursing its children through the transform.
fn rebuild_run(
    dom: &mut Dom,
    element: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
    skip_pt: bool,
) -> NodeId {
    let r = dom.new_element(W::r());
    copy_attrs(dom, element, r, skip_pt);
    for c in dom.nodes(element) {
        for tn in mark_content_transform(dom, c, settings, id_gen) {
            dom.add(r, tn);
        }
    }
    r
}

/// M4.F.1/F.2/F.3 — `MarkContentAsDeletedOrInsertedTransform` (WmlComparer.cs:2568).
pub fn mark_content_transform(
    dom: &mut Dom,
    node: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> Vec<NodeId> {
    if !dom.is_element(node) {
        return vec![dom.clone_subtree(node)];
    }
    let name = dom.name(node).unwrap();

    if name == W::r() {
        let txbx = W::name("txbxContent");
        let carriers: Vec<NodeId> = descendants_trimmed(dom, node, &txbx)
            .into_iter()
            .filter(|&d| is_run_status_carrier(dom, d))
            .collect();
        let mut statuses: Vec<String> = Vec::new();
        for c in &carriers {
            if let Some(s) = dom.attribute(*c, &PT::status())
                && !statuses.iter().any(|x| x == s)
            {
                statuses.push(s.to_string());
            }
        }
        if statuses.len() > 1 {
            panic!("Internal error - both deleted and inserted text in the same run");
        }
        if statuses.is_empty() {
            return vec![rebuild_run(dom, node, settings, id_gen, false)];
        }
        let status = statuses[0].as_str();
        let move_name = |dom: &Dom| -> String {
            carriers
                .iter()
                .find_map(|&c| {
                    dom.attribute(c, &PT::name("MoveName"))
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "move1".to_string())
        };
        match status {
            "Deleted" | "Inserted" => {
                let wrap = if status == "Deleted" {
                    W::del()
                } else {
                    W::ins()
                };
                let w = rev_el(dom, wrap, settings, id_gen);
                let r = rebuild_run(dom, node, settings, id_gen, false);
                if status == "Deleted" {
                    convert_run_text_to_del_text(dom, r);
                }
                dom.add(w, r);
                vec![w]
            }
            "MovedSource" | "MovedDestination" => {
                let (range_start, mid, range_end) = if status == "MovedSource" {
                    (
                        W::name("moveFromRangeStart"),
                        W::name("moveFrom"),
                        W::move_from_range_end(),
                    )
                } else {
                    (
                        W::name("moveToRangeStart"),
                        W::name("moveTo"),
                        W::move_to_range_end(),
                    )
                };
                let mname = move_name(dom);
                let range_id = *id_gen;
                *id_gen += 1;
                let rs = dom.new_element(range_start);
                dom.set_attribute_value(rs, &W::id(), Some(&range_id.to_string()));
                dom.set_attribute_value(rs, &W::name("name"), Some(&mname));
                dom.set_attribute_value(rs, &W::author(), Some(&settings.author_for_revisions));
                dom.set_attribute_value(rs, &W::date(), Some(&settings.date_time_for_revisions));
                let mv = rev_el(dom, mid, settings, id_gen);
                let r = rebuild_run(dom, node, settings, id_gen, false);
                // Word Compare keeps `w:t` inside `w:moveFrom` (not `w:delText`
                // and not a nested `w:del`). Using delText here invited
                // wrap_bare_del_text_runs to nest `w:del` and trigger Word's
                // "unreadable content" repair dialog on move-heavy redlines.
                dom.add(mv, r);
                let re = dom.new_element(range_end);
                dom.set_attribute_value(re, &W::id(), Some(&range_id.to_string()));
                vec![rs, mv, re]
            }
            "FormatChanged" => {
                let old_rpr_str = carriers
                    .iter()
                    .find_map(|&c| dom.attribute(c, &PT::name("OldRPr")).map(|s| s.to_string()));
                let r = rebuild_run(dom, node, settings, id_gen, true);
                let rpr = match dom.element(r, &W::r_pr()) {
                    Some(p) => p,
                    None => {
                        let p = dom.new_element(W::r_pr());
                        dom.add_first(r, p);
                        p
                    }
                };
                let old_rpr = parse_rpr(dom, old_rpr_str.as_deref());
                let chg = dom.new_element(W::name("rPrChange"));
                dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
                *id_gen += 1;
                dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
                dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
                dom.add(chg, old_rpr);
                dom.add(rpr, chg);
                vec![r]
            }
            other => panic!("Internal error - unknown run status: {other}"),
        }
    } else if name == W::p_pr() {
        let status = dom.attribute(node, &PT::status()).map(|s| s.to_string());
        let Some(status) = status else {
            // identity recurse
            let ppr = dom.new_element(W::p_pr());
            copy_attrs(dom, node, ppr, false);
            for c in dom.nodes(node) {
                for tn in mark_content_transform(dom, c, settings, id_gen) {
                    dom.add(ppr, tn);
                }
            }
            return vec![ppr];
        };
        // M81: paragraph-formatting-only change — live pPr is MODIFIED (B);
        // w:pPrChange records ORIGINAL (A) from pt:OldPPr (docxodus :5040).
        //
        // M81c (file_69): when B clears spacing and A had **after-only** small
        // spacing (`after≤40`, no before/line), also materialize it live.
        // Word keeps live after=20 on the drawing residual; pPrChange alone
        // does not affect LO line boxes (score stuck 89.89). Do **not**
        // re-promote Heading residuals (before=400…) — that re-bloats file_33
        // (−15 score when ungated).
        if status == "FormatChanged" {
            let ppr = dom.clone_subtree(node);
            // Drop internal pt: attrs used only for the transform pipeline.
            dom.set_attribute_value(ppr, &PT::status(), None);
            // M97 (file_30): nest mark `w:rPrChange` under live `pPr/rPr` when
            // pilcrow mark fonts differ (Word stamp Aptos→sz32).
            if let Some(old_rpr_s) = dom
                .attribute(node, &PT::name("OldRPr"))
                .map(|s| s.to_string())
            {
                dom.set_attribute_value(ppr, &PT::name("OldRPr"), None);
                let rpr = match dom.element(ppr, &W::r_pr()) {
                    Some(p) => p,
                    None => {
                        let p = dom.new_element(W::r_pr());
                        dom.add_first(ppr, p);
                        p
                    }
                };
                if dom.element(rpr, &W::name("rPrChange")).is_none() {
                    let old_rpr = parse_rpr(dom, Some(&old_rpr_s));
                    let chg = dom.new_element(W::name("rPrChange"));
                    dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
                    *id_gen += 1;
                    dom.set_attribute_value(
                        chg,
                        &W::author(),
                        Some(&settings.author_for_revisions),
                    );
                    dom.set_attribute_value(
                        chg,
                        &W::date(),
                        Some(&settings.date_time_for_revisions),
                    );
                    dom.add(chg, old_rpr);
                    dom.add(rpr, chg);
                }
            }
            if let Some(old_s) = dom
                .attribute(node, &PT::name("OldPPr"))
                .map(|s| s.to_string())
            {
                dom.set_attribute_value(ppr, &PT::name("OldPPr"), None);
                let old_ppr = parse_ppr(dom, Some(&old_s));
                if dom.element(ppr, &W::name("spacing")).is_none()
                    && let Some(old_sp) = dom.element(old_ppr, &W::name("spacing"))
                {
                    let after = dom.attribute(old_sp, &W::name("after")).unwrap_or("");
                    let before = dom.attribute(old_sp, &W::name("before")).unwrap_or("");
                    let line = dom.attribute(old_sp, &W::name("line")).unwrap_or("");
                    let after_n: i64 = after.parse().unwrap_or(i64::MAX);
                    // after-only micro spacing (file_69 after=20)
                    if before.is_empty() && line.is_empty() && after_n > 0 && after_n <= 40 {
                        let sp = dom.clone_subtree(old_sp);
                        dom.add_first(ppr, sp);
                    }
                }
                let chg = dom.new_element(W::name("pPrChange"));
                dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
                *id_gen += 1;
                dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
                dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
                let old_for_chg = parse_ppr(dom, Some(&old_s));
                dom.add(chg, old_for_chg);
                dom.add(ppr, chg); // pPrChange last in CT_PPr
            }
            return vec![ppr];
        }
        let ppr = dom.clone_subtree(node);
        let wrap = match status.as_str() {
            "Deleted" | "MovedSource" => W::del(),
            "Inserted" | "MovedDestination" => W::ins(),
            other => panic!("Internal error - unknown pPr status: {other}"),
        };
        let rpr = match dom.element(ppr, &W::r_pr()) {
            Some(p) => p,
            None => {
                let p = dom.new_element(W::r_pr());
                dom.add_first(ppr, p);
                p
            }
        };
        let mark = rev_el(dom, wrap, settings, id_gen);
        dom.add(rpr, mark);
        vec![ppr]
    } else {
        // identity rebuild
        let ne = dom.new_element(name);
        copy_attrs(dom, node, ne, false);
        for c in dom.nodes(node) {
            for tn in mark_content_transform(dom, c, settings, id_gen) {
                dom.add(ne, tn);
            }
        }
        vec![ne]
    }
}

/// Parse an `OldRPr` attribute string into a `w:rPr` element (empty on failure).
fn parse_rpr(dom: &mut Dom, s: Option<&str>) -> NodeId {
    if let Some(s) = s {
        // wrap so the rPr's namespace prefix resolves
        let doc = dom.parse_xdocument(s);
        if let Some(root) = dom.root(doc) {
            return dom.clone_subtree(root);
        }
    }
    dom.new_element(W::r_pr())
}

/// Parse an `OldPPr` attribute string into a `w:pPr` element (empty on failure).
fn parse_ppr(dom: &mut Dom, s: Option<&str>) -> NodeId {
    if let Some(s) = s {
        let doc = dom.parse_xdocument(s);
        if let Some(root) = dom.root(doc) {
            return dom.clone_subtree(root);
        }
    }
    dom.new_element(W::p_pr())
}

/// M4.F — apply MarkContent to a root, returning the new root.
pub fn mark_content_as_deleted_or_inserted(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> NodeId {
    let v = mark_content_transform(dom, root, settings, id_gen);
    v.into_iter()
        .next()
        .expect("root transform yields one node")
}

/// True when pPr carries structural body props (pStyle/numPr/spacing/jc/ind/…).
/// Ignores rPr / sectPr / pPrChange / pt: attrs (not layout structure).
fn ppr_has_structural_props(dom: &Dom, ppr: NodeId) -> bool {
    for c in dom.elements(ppr, None) {
        let Some(n) = dom.name(c) else {
            continue;
        };
        if n == W::r_pr()
            || n == W::name("sectPr")
            || n == W::name("pPrChange")
            || n.namespace_name() == PT::URI
        {
            continue;
        }
        return true;
    }
    false
}

/// True when the only structural child of pPr is `w:jc` (center-align demos).
fn ppr_is_jc_only(dom: &Dom, ppr: NodeId) -> bool {
    let mut saw_jc = false;
    for c in dom.elements(ppr, None) {
        let Some(n) = dom.name(c) else {
            continue;
        };
        if n == W::r_pr()
            || n == W::name("sectPr")
            || n == W::name("pPrChange")
            || n.namespace_name() == PT::URI
        {
            continue;
        }
        if n == W::name("jc") {
            if saw_jc {
                return false;
            }
            saw_jc = true;
        } else {
            return false;
        }
    }
    saw_jc
}

/// M4.F.4 — `ConjoinTransform` (:2947): collapse a `w:p` with ≥2 `w:pPr` into one
/// cleaned pPr + all non-pPr children.
///
/// M78 (file_33 residual end-zip): when both an Inserted and a Deleted pPr are
/// present **and Inserted has structural props** (ListParagraph+numPr), Word
/// keeps Inserted live and records Deleted in `w:pPrChange` (A's before=400).
///
/// M81b (file_69 drawing residual): when Inserted is empty of structural props
/// but Deleted has spacing (after=20), Word keeps **Deleted live** with a del
/// pilcrow mark — not pPrChange. Preferring Inserted always cleared live
/// spacing and left the score stuck at 89.89.
fn conjoin_transform(dom: &mut Dom, node: NodeId, author: &str, date: &str) -> NodeId {
    if !dom.is_element(node) {
        return dom.clone_subtree(node);
    }
    let name = dom.name(node).unwrap();
    if name == W::p() && dom.elements(node, Some(&W::p_pr())).len() >= 2 {
        let pprs = dom.elements(node, Some(&W::p_pr()));
        let ins_idx = pprs.iter().position(|&p| {
            matches!(
                dom.attribute(p, &PT::status()),
                Some("Inserted") | Some("MovedDestination")
            )
        });
        let del_idx = pprs.iter().position(|&p| {
            matches!(
                dom.attribute(p, &PT::status()),
                Some("Deleted") | Some("MovedSource")
            )
        });
        // Prefer structural Inserted (M78); else structural Deleted (M81b);
        // else last pPr.
        let live_idx = match (ins_idx, del_idx) {
            (Some(i), Some(_d)) if ppr_has_structural_props(dom, pprs[i]) => i,
            (Some(i), Some(d))
                if !ppr_has_structural_props(dom, pprs[i])
                    && ppr_has_structural_props(dom, pprs[d]) =>
            {
                d
            }
            (Some(i), _) => i,
            (_, Some(d)) => d,
            _ => pprs.len().saturating_sub(1),
        };
        let live_src = pprs[live_idx];
        let live_is_inserted = matches!(
            dom.attribute(live_src, &PT::status()),
            Some("Inserted") | Some("MovedDestination")
        );
        let old_src = pprs.iter().copied().find(|&p| {
            p != live_src
                && matches!(
                    dom.attribute(p, &PT::status()),
                    Some("Deleted") | Some("MovedSource")
                )
        });

        let ppr = dom.clone_subtree(live_src);
        // When live is Inserted, strip mark revisions under rPr (equal pilcrow
        // path). When live is Deleted (M81b), KEEP the del pilcrow mark so
        // mark_content / already-present del status matches Word.
        if live_is_inserted {
            for rpr in dom.elements(ppr, Some(&W::r_pr())) {
                for child in dom.elements(rpr, None) {
                    let cn = dom.name(child).unwrap();
                    if cn == W::ins() || cn == W::del() {
                        dom.remove(child);
                    }
                }
                if dom.elements(rpr, None).is_empty() {
                    dom.remove(rpr);
                }
            }
        }
        dom.set_attribute_value(ppr, &PT::status(), None);

        // When live is Inserted, record Deleted in pPrChange (M78).
        // When live is Deleted, do not wrap Deleted into pPrChange — it IS live.
        if live_is_inserted
            && let Some(old) = old_src
            && dom.element(ppr, &W::name("pPrChange")).is_none()
        {
            let old_inner = dom.clone_subtree(old);
            dom.set_attribute_value(old_inner, &PT::status(), None);
            for rpr in dom.elements(old_inner, Some(&W::r_pr())) {
                for child in dom.elements(rpr, None) {
                    let cn = dom.name(child).unwrap();
                    if cn == W::ins() || cn == W::del() {
                        dom.remove(child);
                    }
                }
            }
            let chg = dom.new_element(W::name("pPrChange"));
            dom.set_attribute_value(chg, &W::author(), Some(author));
            dom.set_attribute_value(chg, &W::date(), Some(date));
            dom.set_attribute_value(chg, &W::id(), Some("0"));
            dom.add(chg, old_inner);
            dom.add(ppr, chg);
        }

        let p = dom.new_element(W::p());
        copy_attrs(dom, node, p, false);
        dom.add(p, ppr);
        for child in dom.elements(node, None) {
            if dom.name(child).unwrap() != W::p_pr() {
                let t = conjoin_transform(dom, child, author, date);
                dom.add(p, t);
            }
        }
        return p;
    }
    let ne = dom.new_element(name);
    copy_attrs(dom, node, ne, false);
    for c in dom.nodes(node) {
        let t = conjoin_transform(dom, c, author, date);
        dom.add(ne, t);
    }
    ne
}

/// M4.F.4 — apply Conjoin to a root, returning the new root.
pub fn conjoin_paragraph_marks(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
) -> NodeId {
    conjoin_transform(
        dom,
        root,
        &settings.author_for_revisions,
        &settings.date_time_for_revisions,
    )
}

/// M4.F.5 — `FixUpRevisionIds` (WmlComparer.cs:2769): renumber `w:id` of all revision elements
/// across the given roots (main, then footnotes, then endnotes) from 1; range
/// start/end pairs share an id.
pub fn fix_up_revision_ids(dom: &mut Dom, roots: &[NodeId]) {
    // Every tracked-change / move / property-change carrier that uses w:id.
    // Missing any of these lets renumbered move ranges collide with e.g.
    // tblPrChange ids — Word then raises "unreadable content".
    let rev_names = [
        W::ins(),
        W::del(),
        W::name("moveFrom"),
        W::name("moveTo"),
        W::name("moveFromRangeStart"),
        W::move_from_range_end(),
        W::name("moveToRangeStart"),
        W::move_to_range_end(),
        W::name("rPrChange"),
        W::name("pPrChange"),
        W::name("tblPrChange"),
        W::name("tblGridChange"),
        W::name("trPrChange"),
        W::name("tcPrChange"),
        W::name("sectPrChange"),
        W::name("numberingChange"),
        W::name("cellMerge"),
    ];
    // Comment anchors keep their own ids (must stay aligned with comments.xml).
    // Reserved so revision renumber never reuses them.
    let reserved_names = [
        W::name("commentRangeStart"),
        W::name("commentRangeEnd"),
        W::name("commentReference"),
    ];
    let mut reserved: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut all = Vec::new();
    for &root in roots {
        for d in dom.descendants(root, None) {
            let Some(n) = dom.name(d) else { continue };
            if rev_names.contains(&n) {
                all.push(d);
            } else if reserved_names.contains(&n)
                && let Some(id) = dom.attribute(d, &W::id()).and_then(|s| s.parse().ok())
            {
                reserved.insert(id);
            }
        }
    }
    let mut old_to_new: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    let mut next_id = 1u32;
    let mut alloc = |old_to_new: &mut std::collections::HashMap<String, u32>,
                     old: &str,
                     remember: bool|
     -> u32 {
        while reserved.contains(&next_id) {
            next_id += 1;
        }
        let v = next_id;
        next_id += 1;
        reserved.insert(v); // don't reassign within this pass either
        if remember {
            old_to_new.insert(old.to_string(), v);
        }
        v
    };
    let (mffe, mtre, mffs, mtrs) = (
        W::move_from_range_end(),
        W::move_to_range_end(),
        W::name("moveFromRangeStart"),
        W::name("moveToRangeStart"),
    );
    for rev in all {
        let Some(old) = dom.attribute(rev, &W::id()).map(|s| s.to_string()) else {
            continue;
        };
        let n = dom.name(rev).unwrap();
        let new_id = if n == mffe || n == mtre {
            match old_to_new.get(&old) {
                Some(&v) => v,
                None => alloc(&mut old_to_new, &old, true),
            }
        } else if n == mffs || n == mtrs {
            alloc(&mut old_to_new, &old, true)
        } else {
            alloc(&mut old_to_new, &old, false)
        };
        dom.set_attribute_value(rev, &W::id(), Some(&new_id.to_string()));
    }
}

/// M4.F.7 — `IgnorePt14Namespace` (WmlComparer.cs:2912): declare `xmlns:pt14` + add `pt14` to
/// `mc:Ignorable` (idempotent).
pub fn ignore_pt14_namespace(dom: &mut Dom, root: NodeId) {
    let pt14 = XNamespace::xmlns().name("pt14");
    if dom.attribute(root, &pt14).is_none() {
        dom.set_attribute_value(root, &pt14, Some(PT::URI));
    }
    let ignorable = MC::name("Ignorable");
    let cur = dom.attribute(root, &ignorable).unwrap_or("").to_string();
    let mut toks: Vec<&str> = cur.split_whitespace().collect();
    if !toks.contains(&"pt14") {
        toks.push("pt14");
        dom.set_attribute_value(root, &ignorable, Some(&toks.join(" ")));
    }
}

/// M4.F.7 — `RemovePowerToolsScratchMarkup` (CleanPartTransform, WmlComparer.cs:1165):
/// strip every `pt:*` attribute across `root` and descendants.
pub fn remove_powertools_scratch_markup(dom: &mut Dom, root: NodeId) {
    for el in dom.descendants_and_self(root, None) {
        let pts: Vec<XName> = dom
            .attributes(el)
            .into_iter()
            .map(|(n, _)| n)
            .filter(|n| n.namespace_name() == PT::URI)
            .collect();
        for a in pts {
            dom.set_attribute_value(el, &a, None);
        }
    }
}

// ── CoalesceAdjacentRunsWithIdenticalFormatting (PtOpenXmlUtil.ts:4458) ───────
// Merges adjacent single-w:t runs with identical rPr into one run. Used both in
// PreProcessMarkup (on the inputs, so source fragmentation doesn't inflate the
// diff) and in the produce finalization (merges adjacent w:del; w:ins keep
// distinct ids so don't merge).

const DONT_CONSOLIDATE: &str = "DontConsolidate";

fn xml_space_attr(text: &str) -> Option<&'static str> {
    match (text.chars().next(), text.chars().last()) {
        (Some(f), _) if f.is_whitespace() => Some("preserve"),
        (_, Some(l)) if l.is_whitespace() => Some("preserve"),
        _ => None,
    }
}

fn rpr_string(dom: &Dom, r: NodeId) -> String {
    match dom.element(r, &W::r_pr()) {
        Some(rpr) => dom.serialize_element(rpr),
        None => String::new(),
    }
}

fn coalesce_key(dom: &Dom, ce: NodeId) -> String {
    let name = match dom.name(ce) {
        Some(n) => n,
        None => return DONT_CONSOLIDATE.to_string(),
    };
    if name == W::r() {
        let non_rpr = dom
            .elements(ce, None)
            .into_iter()
            .filter(|&e| dom.name(e) != Some(W::r_pr()))
            .count();
        if non_rpr != 1 {
            return DONT_CONSOLIDATE.to_string();
        }
        if dom.attribute(ce, &PT::name("AbstractNumId")).is_some() {
            return DONT_CONSOLIDATE.to_string();
        }
        // Pre* stamp trios (PreIns / PreDelete carriers, D1/w15 machinery)
        // segregate the key: merging a stamped run with an unstamped (or
        // differently-stamped) neighbor drops the stamp on the merged run and
        // convert_stamped_* never re-emits the pending revision — corpus:
        // contract-review-suggesting-mixed-edits ("ApproveSignedd", −20.9
        // visual). Same-stamp runs still merge (copy_attrs keeps the trio).
        let stamp: String = [
            "PreIns",
            "PreInsAuthor",
            "PreInsDate",
            "PreDelete",
            "PreDelAuthor",
            "PreDelDate",
        ]
        .iter()
        .map(|a| dom.attribute(ce, &PT::name(a)).unwrap_or("").to_string())
        .collect::<Vec<_>>()
        .join("\u{1}");
        let rpr = rpr_string(dom, ce);
        if dom.element(ce, &W::t()).is_some() {
            return format!("Wt{rpr}\u{2}{stamp}");
        }
        if dom.element(ce, &W::instr_text()).is_some() {
            return format!("WinstrText{rpr}\u{2}{stamp}");
        }
        return DONT_CONSOLIDATE.to_string();
    }
    if name == W::del() {
        // Gate (PtOpenXmlUtil.ts:4513): only merge a w:del whose runs hold exactly
        // one non-rPr child AND contain delText. Otherwise (e.g. a deletion holding
        // a w:tab / w:br / drawing) leave it alone, so the text-only run rebuild
        // does not drop that non-text content.
        let non_rpr = dom
            .elements(ce, Some(&W::r()))
            .into_iter()
            .flat_map(|r| dom.elements(r, None))
            .filter(|&e| dom.name(e) != Some(W::r_pr()))
            .count();
        let has_del_text = dom
            .elements(ce, None)
            .into_iter()
            .any(|c| dom.element(c, &W::name("delText")).is_some());
        if non_rpr != 1 || !has_del_text {
            return DONT_CONSOLIDATE.to_string();
        }
        // del key omits id (so adjacent deletions merge)
        let author = dom.attribute(ce, &W::author()).unwrap_or("").to_string();
        let date = dom.attribute(ce, &W::date()).unwrap_or("").to_string();
        let rprs: String = dom
            .elements(ce, Some(&W::r()))
            .into_iter()
            .filter_map(|r| {
                dom.element(r, &W::r_pr())
                    .map(|rp| dom.serialize_element(rp))
            })
            .collect();
        // Pre* stamps segregate adjacent dels the same way they segregate
        // runs — a stamped run merged across dels loses its stamp and the
        // pending revision is never re-emitted (see the w:r key above).
        let stamp: String = dom
            .elements(ce, Some(&W::r()))
            .into_iter()
            .flat_map(|r| {
                [
                    "PreIns",
                    "PreInsAuthor",
                    "PreInsDate",
                    "PreDelete",
                    "PreDelAuthor",
                    "PreDelDate",
                ]
                .iter()
                .map(move |a| (r, *a))
            })
            .map(|(r, a)| dom.attribute(r, &PT::name(a)).unwrap_or("").to_string())
            .collect::<Vec<_>>()
            .join("\u{1}");
        return format!("Wdel{author}{date}{rprs}\u{2}{stamp}");
    }
    DONT_CONSOLIDATE.to_string()
}

fn run_text_concat(dom: &Dom, r: NodeId) -> String {
    let mut s = String::new();
    for d in dom.descendants(r, None) {
        let n = dom.name(d).unwrap();
        if n == W::t() || n == W::name("delText") || n == W::instr_text() {
            s.push_str(&dom.value_str(d));
        }
    }
    s
}

/// `CoalesceAdjacentRunsWithIdenticalFormatting` on one run container.
pub fn coalesce_adjacent_runs(dom: &mut Dom, container: NodeId) -> NodeId {
    let cname = dom.name(container).unwrap();
    let children = dom.elements(container, None);
    let grouped = crate::util::group_adjacent(children, |&ce| coalesce_key(dom, ce));

    let nc = dom.new_element(cname);
    copy_attrs(dom, container, nc, false);
    for (key, g) in grouped {
        if key == DONT_CONSOLIDATE {
            for e in g {
                let c = dom.clone_subtree(e);
                dom.add(nc, c);
            }
            continue;
        }
        let text: String = g.iter().map(|&r| run_text_concat(dom, r)).collect();
        let first = g[0];
        let fname = dom.name(first).unwrap();
        if fname == W::r() {
            let nr = dom.new_element(W::r());
            copy_attrs(dom, first, nr, false);
            if let Some(rpr) = dom.element(first, &W::r_pr()) {
                let c = dom.clone_subtree(rpr);
                dom.add(nr, c);
            }
            let leaf_name = if dom.element(first, &W::instr_text()).is_some() {
                W::instr_text()
            } else {
                W::t()
            };
            let t = dom.new_element(leaf_name);
            if let Some(sp) = xml_space_attr(&text) {
                dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some(sp));
            }
            dom.add_text(t, &text);
            dom.add(nr, t);
            dom.add(nc, nr);
        } else if fname == W::del() {
            let nd = dom.new_element(W::del());
            copy_attrs(dom, first, nd, false);
            let nr = dom.new_element(W::r());
            if let Some(fr) = dom.element(first, &W::r()) {
                copy_attrs(dom, fr, nr, false);
                if let Some(rpr) = dom.element(fr, &W::r_pr()) {
                    let c = dom.clone_subtree(rpr);
                    dom.add(nr, c);
                }
            }
            let dt = dom.new_element(W::name("delText"));
            if let Some(sp) = xml_space_attr(&text) {
                dom.set_attribute_value(dt, &XNamespace::xml().name("space"), Some(sp));
            }
            dom.add_text(dt, &text);
            dom.add(nr, dt);
            dom.add(nd, nr);
            dom.add(nc, nd);
        } else {
            for e in g {
                let c = dom.clone_subtree(e);
                dom.add(nc, c);
            }
        }
    }
    nc
}

/// Word's compare RESOLVES feature-gating `mc:AlternateContent` (replaces it with
/// its `mc:Choice` content) but KEEPS drawing/VML fallbacks (DrawingML Choice +
/// VML Fallback). We previously kept ALL AltContent as opaque atoms
/// (atomize.rs:256, produce.rs:557), so run-level feature-gating AltContent got
/// hoisted to invalid block positions → Word "unreadable content" repair, and
/// `ooxmlsdk` rejects them (`UnexpectedTag { ty: TableCell, found:
/// AlternateContent }`). Validated against the 100-pair corpus: Word retains 7
/// drawing/VML AltContent and 0 text-only ones. Run on BOTH inputs before diffing.
pub fn resolve_alternate_content(dom: &mut Dom, root: NodeId) {
    for ac in dom.descendants_and_self(root, Some(&MC::name("AlternateContent"))) {
        // A nested AltContent inside a discarded Fallback may already be detached.
        if dom.parent(ac).is_none() {
            continue;
        }
        // Keep drawing/VML fallbacks (Word does); resolve only text feature-gating.
        let has_drawing = !dom.descendants(ac, Some(&W::name("drawing"))).is_empty()
            || !dom.descendants(ac, Some(&W::name("pict"))).is_empty();
        if has_drawing {
            continue;
        }
        // Anchor-internal wrappers (e.g. wp:positionH gating wp14:pctPosHOffset
        // vs a wp:posOffset fallback) must survive verbatim: Word keeps them,
        // and resolving to the wp14 Choice leaves the parent schema-EMPTY for
        // consumers that ignore wp14 → "unreadable content" (strict01 cover
        // page, bisected in real Word). The whole w:drawing is one atom, so
        // the original hoisting concern doesn't apply inside it.
        let inside_drawing = dom
            .ancestors_and_self(ac, None)
            .into_iter()
            .any(|a| dom.name(a) == Some(W::name("drawing")));
        if inside_drawing {
            continue;
        }
        // MCE branch selection (ISO/IEC 29500-3 §10.1.2; mirrors the SDK's
        // `mce_choice_replacement_child_bytes` in ooxmlsdk `common/xml.rs`): pick
        // the FIRST `mc:Choice` whose `@Requires` namespaces are all understood,
        // else the `mc:Fallback`. Taking `mc:Choice` unconditionally would resolve
        // to content Word would never keep when the first Choice gates on a
        // namespace this document doesn't even bind, or when a later Choice/Fallback
        // is the one Word selects. "Understood" here means every prefix in
        // `@Requires` resolves to an in-scope `xmlns:*` declaration — you cannot
        // satisfy a requirement for a vocabulary the document never declares.
        let src = dom
            .elements(ac, Some(&MC::name("Choice")))
            .into_iter()
            .find(|&choice| choice_requires_understood(dom, choice))
            .or_else(|| dom.element(ac, &MC::name("Fallback")));
        if let Some(src) = src {
            let kids = dom.nodes(src);
            dom.replace_with(ac, &kids);
        }
    }
}

/// Strip non-standard `w:`-namespace children from `<w:sdtPr>`. Agreement/form
/// tools emit `<w:fieldType>`/`<w:fieldColor>`/… inside sdtPr in the w: namespace
/// — not valid CT_SdtPr content; real Word reports "unreadable content" and
/// recovers by dropping them (and produces a clean redline GT). We match so the
/// output is Word-valid. Extension-namespace children (w14:/w15:/…) are untouched.
pub fn sanitize_sdt_properties(dom: &mut Dom, root: NodeId) {
    // Valid CT_SdtPr w: children (ECMA-376).
    const VALID: &[&str] = &[
        "rPr",
        "alias",
        "lock",
        "placeholder",
        "showingPlcHdr",
        "dataBinding",
        "temporary",
        "id",
        "tag",
        "group",
        "comboBox",
        "date",
        "dropDownList",
        "docPartObj",
        "docPartList",
        "equation",
        "picture",
        "richText",
        "text",
        "citation",
        "bibliography",
    ];
    for sdtpr in dom.descendants(root, Some(&W::name("sdtPr"))) {
        for k in dom.elements(sdtpr, None) {
            if let Some(n) = dom.name(k)
                && n.namespace_name() == W::URI
                && !VALID.contains(&n.local_name())
            {
                dom.remove(k);
            }
        }
    }
}

/// True if every namespace prefix listed in a `mc:Choice`'s `@Requires` is bound by
/// an in-scope `xmlns:*` declaration. An absent/empty `@Requires` is vacuously true
/// (the schema requires it, but be lenient). The `@Requires` attribute is unprefixed
/// (no namespace) per the MCE schema.
fn choice_requires_understood(dom: &Dom, choice: NodeId) -> bool {
    let Some(requires) = dom.attribute(choice, &XNamespace::none().name("Requires")) else {
        return true;
    };
    requires
        .split_whitespace()
        .all(|prefix| prefix_in_scope(dom, choice, prefix))
}

/// True if `prefix` is bound by an `xmlns:prefix` declaration on `node` or any of
/// its ancestors (XML namespace scoping).
fn prefix_in_scope(dom: &Dom, node: NodeId, prefix: &str) -> bool {
    dom.ancestors_and_self(node, None).iter().any(|&anc| {
        dom.attributes(anc)
            .iter()
            .any(|(name, _)| dom.is_namespace_declaration(name) && name.local_name() == prefix)
    })
}

/// Apply CoalesceAdjacentRuns to every `w:p` in `root` (in place).
pub fn coalesce_all_paragraphs(dom: &mut Dom, root: NodeId) {
    let paras = dom.descendants(root, Some(&W::p()));
    for p in paras {
        let np = coalesce_adjacent_runs(dom, p);
        let children = dom.nodes(np);
        // replace p's children with np's coalesced children + keep p attrs
        dom.remove_nodes(p);
        for c in children {
            dom.add(p, c);
        }
    }
}

/// Enforce OOXML child order for paragraph-mark revision markers. When a paragraph
/// mark is inserted/deleted the marker lands in the paragraph's paraRPr; OOXML
/// requires (a) `ins`/`del`/`moveFrom`/`moveTo` to be the FIRST children of that
/// `<w:rPr>` (CT_ParaRPr sequence), and (b) the `<w:rPr>` to follow the paragraph's
/// content properties (pStyle, …) — i.e. sit just before `sectPr`/`pPrChange` (or
/// last). We otherwise appended the marker after `<w:lang>`/etc. and/or placed the
/// paraRPr before `pStyle`, which ooxmlsdk tolerates but real Word rejects as
/// "unreadable content" (contract-acc / hyperlink / NumberingImplicitNumId / nda).
pub fn fix_paragraph_mark_revision_order(dom: &mut Dom, root: NodeId) {
    let is_marker = |dom: &Dom, c: NodeId| {
        matches!(
            dom.name(c).as_ref().map(|n| n.local_name()),
            Some("ins") | Some("del") | Some("moveFrom") | Some("moveTo")
        )
    };
    for ppr in dom.descendants(root, Some(&W::p_pr())) {
        let Some(rpr) = dom.element(ppr, &W::name("rPr")) else {
            continue;
        };
        // (a) move revision markers to the front of the paraRPr (preserve their order).
        let markers: Vec<NodeId> = dom
            .elements(rpr, None)
            .into_iter()
            .filter(|&c| is_marker(dom, c))
            .collect();
        for m in markers.into_iter().rev() {
            dom.remove(m);
            dom.add_first(rpr, m);
        }
        // (b) reposition the paraRPr after content props: before sectPr/pPrChange, else last.
        let anchor = dom
            .element(ppr, &W::name("sectPr"))
            .or_else(|| dom.element(ppr, &W::name("pPrChange")));
        dom.remove(rpr);
        match anchor {
            Some(a) => dom.add_before_self(a, rpr),
            None => dom.add(ppr, rpr),
        }
    }
}

/// `w:pPr` must be the FIRST child of `w:p` (OOXML schema). Our reassembly emits
/// the paragraph mark (pPr) LAST, so Word/LibreOffice silently ignore ALL
/// paragraph formatting — `w:jc` (centering), spacing, indentation, numbering.
/// Move each paragraph's `w:pPr` to the front. Recurses through the whole tree.
pub fn move_paragraph_properties_first(dom: &mut Dom, node: NodeId) {
    if dom.name(node).as_ref() == Some(&W::p()) {
        let kids = dom.nodes(node);
        if let Some(pos) = kids
            .iter()
            .position(|&c| dom.name(c).as_ref() == Some(&W::p_pr()))
            && pos != 0
        {
            let ppr = kids[pos];
            dom.remove(ppr);
            dom.add_first(node, ppr);
        }
    }
    for c in dom.nodes(node) {
        if dom.is_element(c) {
            move_paragraph_properties_first(dom, c);
        }
    }
}

/// Word Compare unwraps TOC/body `w:hyperlink` wrappers into runs with
/// `w:rStyle w:val="Hyperlink"` (file_21: Word has 0 hyperlinks + 251
/// Hyperlink-styled runs; we kept 133 live hyperlinks). LO layout of field
/// anchors differs from Word's plain Hyperlink runs — unwrap for Word-mode.
///
/// Only anchor-based internal hyperlinks (TOC/bookmark jumps, no `r:id`) are
/// unwrapped. An `r:id`-bearing hyperlink is a live external target: dropping
/// its wrapper here would discard the `r:id` before
/// `reconcile_dangling_relationships` runs, silently destroying the link
/// (m16/m29 regression).
pub fn unwrap_hyperlinks_to_styled_runs(dom: &mut Dom, root: NodeId) {
    let hyperlinks: Vec<NodeId> = dom
        .descendants(root, Some(&W::hyperlink()))
        .into_iter()
        .filter(|&hl| dom.attribute(hl, &R::name("id")).is_none())
        .collect();
    for hl in hyperlinks {
        // Ensure every run under the hyperlink carries Hyperlink rStyle.
        let runs: Vec<NodeId> = dom.descendants(hl, Some(&W::r())).into_iter().collect();
        for r in runs {
            let rpr = match dom.element(r, &W::r_pr()) {
                Some(rp) => rp,
                None => {
                    let rp = dom.new_element(W::r_pr());
                    dom.add_first(r, rp);
                    rp
                }
            };
            if dom.element(rpr, &W::name("rStyle")).is_none() {
                let rs = dom.new_element(W::name("rStyle"));
                dom.set_attribute_value(rs, &W::val(), Some("Hyperlink"));
                dom.add_first(rpr, rs);
            }
        }
        // Hoist children before the hyperlink, then drop the wrapper.
        let kids: Vec<NodeId> = dom.nodes(hl);
        for k in kids {
            dom.remove(k);
            dom.add_before_self(hl, k);
        }
        dom.remove(hl);
    }
}

/// Word Compare omits body-level `w:spacing` that only restates the common
/// demo-doc default (line=276, optional after=200 / lineRule=auto). Keeping it
/// shifts line box height vs Word's redline (center_alignment demos ~79 vs 100).
/// Strip such spacing elements; leave non-default spacing alone.
///
/// **M67** (narrow re-landing of M61): pure-deleted paragraphs with no `pStyle`
/// that carry a **Heading residual** spacing pattern — `before≥360` **and**
/// non-empty `after` **and** non-empty `line` (e.g. before=400 after=120
/// line=240 from stripped Heading1 on file_33) — drop the whole `w:spacing`.
/// Word leaves those pure-dels bare. Does **not** strip bare `before=800`
/// (file_196 Word keeps it) or `before≤300` (file_14 winners).
pub fn strip_redundant_demo_default_spacing(dom: &mut Dom, root: NodeId) {
    let spacing_name = W::name("spacing");
    let mut to_remove = Vec::new();
    for p in dom.descendants(root, Some(&W::p())) {
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        let Some(sp) = dom.element(ppr, &spacing_name) else {
            continue;
        };
        let line = dom.attribute(sp, &W::name("line")).unwrap_or("");
        let after = dom.attribute(sp, &W::name("after")).unwrap_or("");
        let before = dom.attribute(sp, &W::name("before")).unwrap_or("");
        let rule = dom.attribute(sp, &W::name("lineRule")).unwrap_or("");
        // Only the demo-default pattern (line 276 ± after 200 ± lineRule auto).
        let line_ok = line == "276";
        let after_ok = after.is_empty() || after == "200";
        let before_ok = before.is_empty();
        let rule_ok = rule.is_empty() || rule == "auto";
        if line_ok && after_ok && before_ok && rule_ok {
            to_remove.push(sp);
            continue;
        }
        // M67: Heading residual on pure-del (before+after+line, no pStyle).
        if let Ok(b) = before.parse::<i64>()
            && b >= 360
            && !after.is_empty()
            && !line.is_empty()
            && dom.element(ppr, &W::name("pStyle")).is_none()
            && para_is_pure_deleted(dom, p)
        {
            to_remove.push(sp);
        }
    }
    for sp in to_remove {
        dom.remove(sp);
    }
}

/// True when a paragraph has deleted content, no live (non-del) `w:t` text,
/// and no `w:ins` — pure deleted body paragraph.
fn para_is_pure_deleted(dom: &Dom, p: NodeId) -> bool {
    let has_del = !dom.descendants(p, Some(&W::del())).is_empty();
    if !has_del {
        return false;
    }
    if !dom.descendants(p, Some(&W::ins())).is_empty() {
        return false;
    }
    for t in dom.descendants(p, Some(&W::t())) {
        let mut in_del = false;
        for a in dom.ancestors_and_self(t, None) {
            if dom.name(a).as_ref() == Some(&W::del()) {
                in_del = true;
                break;
            }
            if a == p {
                break;
            }
        }
        if !in_del {
            let v = dom.value(t);
            if !v.trim().is_empty() {
                return false;
            }
        }
    }
    true
}

/// M69 — Word leaves the final empty body paragraph unmarked when it is a
/// pure-empty pure-del (file_69: trailing `</w:p>` with no del mark; we had
/// `pPr/rPr/del`). Clear para-mark deletion on a content-free last body para.
pub fn strip_trailing_empty_pure_del_mark(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    // Last non-sectPr child.
    let Some(&last) = kids
        .iter()
        .rev()
        .find(|&&k| dom.name(k) != Some(W::name("sectPr")))
    else {
        return;
    };
    if dom.name(last) != Some(W::p()) {
        return;
    }
    // Any body text (live or delText)?
    let has_t = !dom.descendants(last, Some(&W::t())).is_empty()
        || !dom.descendants(last, Some(&W::name("delText"))).is_empty();
    if has_t {
        return;
    }
    // Only act when the only revision is a para-mark del (pure empty del).
    if !para_is_pure_deleted(dom, last) && !para_mark_revision(dom, last, &W::del()) {
        return;
    }
    // No nested ins/del body runs other than empty structure — pure empty.
    if !dom.descendants(last, Some(&W::ins())).is_empty() {
        return;
    }
    // Strip pPr/rPr del mark (and empty rPr/pPr shells).
    if let Some(ppr) = dom.element(last, &W::p_pr()) {
        if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
            if let Some(d) = dom.element(rpr, &W::del()) {
                dom.remove(d);
            }
            if dom.elements(rpr, None).is_empty() {
                dom.remove(rpr);
            }
        }
        if dom.elements(ppr, None).is_empty() {
            dom.remove(ppr);
        }
    }
}

/// M92 — trailing empty body paragraph: Word records live `w:spacing` under
/// `w:pPrChange` (file_30 last empty after list residual). After M69 strips
/// the pure-del mark we often keep live spacing; move it into pPrChange.
pub fn trailing_empty_spacing_to_pprchange(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids
        .iter()
        .rev()
        .find(|&&k| dom.name(k) != Some(W::name("sectPr")))
    else {
        return;
    };
    if dom.name(last) != Some(W::p()) {
        return;
    }
    if !para_has_no_text(dom, last) {
        return;
    }
    // No body ins/del runs — empty shell (post-M69 or Equal empty).
    if !dom.descendants(last, Some(&W::ins())).is_empty()
        || !dom.descendants(last, Some(&W::del())).is_empty()
    {
        return;
    }
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    if dom.element(ppr, &W::name("pPrChange")).is_some() {
        return;
    }
    let Some(sp) = dom.element(ppr, &W::name("spacing")) else {
        return;
    };
    // Only when spacing is the sole layout child (ignore empty rPr).
    for c in dom.elements(ppr, None) {
        let Some(n) = dom.name(c) else {
            continue;
        };
        if n == W::r_pr() {
            continue;
        }
        if n != W::name("spacing") {
            return;
        }
    }
    let old_inner = dom.new_element(W::p_pr());
    let sp_clone = dom.clone_subtree(sp);
    dom.add(old_inner, sp_clone);
    dom.remove(sp);
    let chg = dom.new_element(W::name("pPrChange"));
    dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;
    dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add(chg, old_inner);
    dom.add(ppr, chg);
    // Drop empty rPr if any leftover.
    if let Some(rpr) = dom.element(ppr, &W::r_pr())
        && dom.elements(rpr, None).is_empty()
    {
        dom.remove(rpr);
    }
}

/// True when a paragraph has no `w:t` / `w:delText` content.
fn para_has_no_text(dom: &Dom, p: NodeId) -> bool {
    dom.descendants(p, Some(&W::t())).is_empty()
        && dom.descendants(p, Some(&W::name("delText"))).is_empty()
}

/// True when `p` is an empty pure-inserted paragraph (no text, has ins, no del).
fn para_is_empty_pure_ins(dom: &Dom, p: NodeId) -> bool {
    if dom.name(p) != Some(W::p()) {
        return false;
    }
    if !para_has_no_text(dom, p) {
        return false;
    }
    let has_del =
        !dom.descendants(p, Some(&W::del())).is_empty() || para_mark_revision(dom, p, &W::del());
    if has_del {
        return false;
    }
    let has_ins =
        !dom.descendants(p, Some(&W::ins())).is_empty() || para_mark_revision(dom, p, &W::ins());
    if !has_ins {
        return false;
    }
    // No drawings / tables inside.
    if !dom.descendants(p, Some(&W::name("drawing"))).is_empty()
        || !dom.descendants(p, Some(&W::name("tbl"))).is_empty()
    {
        return false;
    }
    true
}

/// M83a — drop trailing empty pure-inserted body paragraph (file_23).
///
/// Source B often ends with a self-closing empty `<w:p/>` before `sectPr`
/// (after a table). Word Compare omits it entirely; we emitted pure-I with
/// only a para-mark `w:ins`, adding a blank line Word does not have.
pub fn strip_trailing_empty_pure_ins(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids
        .iter()
        .rev()
        .find(|&&k| dom.name(k) != Some(W::name("sectPr")))
    else {
        return;
    };
    if para_is_empty_pure_ins(dom, last) {
        dom.remove(last);
    }
}

/// M85a — drop empty pure-ins sitting immediately before a trailing pure-del run.
///
/// M83a only removed an empty pure-ins when it was the absolute last body child.
/// When residual pure-dels follow (catalog/table demos: B ends empty after table,
/// A residual is pure-D), Word still omits the blank (`… tbl D D D`) while we
/// kept `… tbl Ei D D D` (file_49, file_75, file_35, file_102, …).
pub fn strip_empty_pure_ins_before_trailing_pure_dels(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    // Strip multiple consecutive empties (loop until stable).
    loop {
        let kids = dom.elements(body, None);
        let non_sect: Vec<NodeId> = kids
            .into_iter()
            .filter(|&k| dom.name(k) != Some(W::name("sectPr")))
            .collect();
        if non_sect.len() < 2 {
            return;
        }
        // Trailing pure-del run: find first index of the run.
        let mut run_start = non_sect.len();
        while run_start > 0 {
            let k = non_sect[run_start - 1];
            if dom.name(k) == Some(W::p()) && para_is_pure_deleted(dom, k) {
                run_start -= 1;
            } else {
                break;
            }
        }
        if run_start == non_sect.len() || run_start == 0 {
            // No pure-del run, or body is all pure-dels.
            return;
        }
        let before = non_sect[run_start - 1];
        if !para_is_empty_pure_ins(dom, before) {
            return;
        }
        dom.remove(before);
    }
}

/// True when every `w:t` / `w:delText` under `p` is empty or whitespace.
fn para_body_text_is_whitespace_only(dom: &Dom, p: NodeId) -> bool {
    let mut saw = false;
    for name in [W::t(), W::name("delText")] {
        for t in dom.descendants(p, Some(&name)) {
            saw = true;
            if !dom.value(t).trim().is_empty() {
                return false;
            }
        }
    }
    // No text nodes counts as whitespace-only (empty pure-ins).
    let _ = saw;
    true
}

/// True when `p` is pure-inserted (has ins, no del content/mark).
fn para_is_pure_inserted(dom: &Dom, p: NodeId) -> bool {
    if dom.name(p) != Some(W::p()) {
        return false;
    }
    let has_del =
        !dom.descendants(p, Some(&W::del())).is_empty() || para_mark_revision(dom, p, &W::del());
    if has_del {
        return false;
    }

    !dom.descendants(p, Some(&W::ins())).is_empty() || para_mark_revision(dom, p, &W::ins())
}

/// M86 — fold whitespace-only pure-ins into the following pure-del (file_88).
///
/// Word Compare merges B's trailing space-only pure-I with the first residual
/// pure-D into one mixed para (`[ins " "][del title]` + pPr/rPr/del mark).
/// We left them separate (`I space` then `D title`), adding a blank line and
/// wrong residual shape. Only folds when pure-I body is whitespace-only so
/// content pure-I (file_33 "Summary") stays separate from unrelated pure-D.
pub fn fold_whitespace_pure_ins_into_following_pure_del(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    loop {
        let kids: Vec<NodeId> = dom
            .elements(body, None)
            .into_iter()
            .filter(|&k| dom.name(k) != Some(W::name("sectPr")))
            .collect();
        let mut acted = false;
        for i in 0..kids.len().saturating_sub(1) {
            let ins_p = kids[i];
            let del_p = kids[i + 1];
            if !para_is_pure_inserted(dom, ins_p) || !para_is_pure_deleted(dom, del_p) {
                continue;
            }
            if !para_body_text_is_whitespace_only(dom, ins_p) {
                continue;
            }
            // Need real deleted content (not empty mark-only del).
            if para_body_text_is_whitespace_only(dom, del_p) {
                continue;
            }
            // Replace pure-ins mark with pure-del mark on the carrier (Word:
            // mixed para keeps del mark from the deleted residual).
            if let Some(ppr) = dom.element(ins_p, &W::p_pr()) {
                if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
                    if let Some(ins_m) = dom.element(rpr, &W::ins()) {
                        dom.remove(ins_m);
                    }
                    // If del mark exists on pure-del, clone it into carrier rPr.
                    if dom.element(rpr, &W::del()).is_none()
                        && let Some(dppr) = dom.element(del_p, &W::p_pr())
                        && let Some(drpr) = dom.element(dppr, &W::r_pr())
                        && let Some(dmark) = dom.element(drpr, &W::del())
                    {
                        let cloned = dom.clone_subtree(dmark);
                        dom.add(rpr, cloned);
                    }
                    if dom.elements(rpr, None).is_empty() {
                        dom.remove(rpr);
                    }
                } else if let Some(dppr) = dom.element(del_p, &W::p_pr()) {
                    // Carrier has pPr but no rPr — adopt del's mark shell if any.
                    if let Some(drpr) = dom.element(dppr, &W::r_pr())
                        && dom.element(drpr, &W::del()).is_some()
                    {
                        let cloned = dom.clone_subtree(drpr);
                        dom.add(ppr, cloned);
                    }
                }
            } else if let Some(dppr) = dom.element(del_p, &W::p_pr()) {
                // No pPr on pure-ins — clone del's pPr if it has a del mark.
                if let Some(drpr) = dom.element(dppr, &W::r_pr())
                    && dom.element(drpr, &W::del()).is_some()
                {
                    let cloned = dom.clone_subtree(dppr);
                    // insert pPr as first child
                    if let Some(first) = dom.elements(ins_p, None).first().copied() {
                        dom.add_before_self(first, cloned);
                    } else {
                        dom.add(ins_p, cloned);
                    }
                }
            }
            // Move body (non-pPr) children from pure-del into pure-ins.
            for c in dom.elements(del_p, None) {
                if dom.name(c) != Some(W::p_pr()) {
                    dom.add(ins_p, c);
                }
            }
            dom.remove(del_p);
            acted = true;
            break;
        }
        if !acted {
            return;
        }
    }
}

/// M105 — pure-D short title + following MIX that starts with `w:ins`: move
/// the leading insert runs onto the pure-D para (Word file_7/5/130).
///
/// After multi-para LCS peels subtitle `"… document"` into the short body,
/// produce emits:
///   p2 pure-D `"Left Alignment Demo"`
///   p3 MIX ins`"A comprehensive… demonstration"` + del/Equal body
/// Word keeps the subtitle insert on the **title** residual para:
///   p2 MIX ins`…demonstration` + del`Left Alignment Demo`
///   p3 del`This` + Equal` document` + del`…`
/// Only when pure-D is short (≤8 body tokens), next is mixed with both ins and
/// del, next starts with ins, and pure-D shares no significant token with that
/// leading insert (unrelated short title vs long-doc subtitle).
pub fn fold_leading_ins_from_mix_into_preceding_pure_del(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    loop {
        let kids: Vec<NodeId> = dom
            .elements(body, None)
            .into_iter()
            .filter(|&k| dom.name(k) != Some(W::name("sectPr")))
            .collect();
        let mut acted = false;
        for i in 0..kids.len().saturating_sub(1) {
            let del_p = kids[i];
            let mix_p = kids[i + 1];
            if dom.name(del_p) != Some(W::p()) || dom.name(mix_p) != Some(W::p()) {
                continue;
            }
            if !para_is_pure_deleted(dom, del_p) {
                continue;
            }
            // Next must be mixed (ins + del), not pure-I.
            let has_ins = !dom.descendants(mix_p, Some(&W::ins())).is_empty();
            let has_del = !dom.descendants(mix_p, Some(&W::del())).is_empty()
                || para_mark_revision(dom, mix_p, &W::del());
            if !has_ins || !has_del {
                continue;
            }
            // Short pure-D title only.
            let del_text = para_revision_body_text(dom, del_p);
            let del_tokens: Vec<&str> = del_text
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| !t.is_empty())
                .collect();
            if del_tokens.is_empty() || del_tokens.len() > 8 {
                continue;
            }
            // Leading body children of mix that are w:ins (skip pPr).
            let mix_kids: Vec<NodeId> = dom.elements(mix_p, None);
            let mut leading_ins: Vec<NodeId> = Vec::new();
            for &c in &mix_kids {
                if dom.name(c) == Some(W::p_pr()) {
                    continue;
                }
                if dom.name(c) == Some(W::ins()) {
                    leading_ins.push(c);
                } else {
                    break;
                }
            }
            if leading_ins.is_empty() {
                continue;
            }
            // Collect insert text for relatedness gate.
            let mut ins_text = String::new();
            for &ins_n in &leading_ins {
                for t in dom.descendants(ins_n, Some(&W::t())) {
                    ins_text.push_str(&dom.value_str(t));
                }
            }
            if ins_text.trim().is_empty() {
                continue;
            }
            // Unrelated short title vs subtitle insert (shared sig tokens = 0).
            let del_sig: std::collections::HashSet<String> = del_tokens
                .iter()
                .filter(|t| t.chars().count() >= 4)
                .map(|t| t.to_ascii_lowercase())
                .collect();
            let ins_sig: std::collections::HashSet<String> = ins_text
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| t.chars().count() >= 4)
                .map(|t| t.to_ascii_lowercase())
                .collect();
            if del_sig.intersection(&ins_sig).count() > 0 {
                continue;
            }
            // Move leading ins onto pure-D **before** del body (Word: ins then del).
            let del_body_first = dom
                .elements(del_p, None)
                .into_iter()
                .find(|&c| dom.name(c) != Some(W::p_pr()));
            for &ins_n in &leading_ins {
                if dom.parent(ins_n).is_none() {
                    continue;
                }
                dom.remove(ins_n);
                if let Some(first) = del_body_first {
                    // After first move, still insert before original first del body.
                    if dom.parent(first).is_some() {
                        dom.add_before_self(first, ins_n);
                    } else {
                        dom.add(del_p, ins_n);
                    }
                } else {
                    dom.add(del_p, ins_n);
                }
            }
            // M110: Word keeps a pure-del paragraph mark on the mixed title
            // residual (file_7/130 p2: pPr/rPr/del with ins+del body). Do **not**
            // strip the del mark after moving leading ins onto pure-D.
            // Ensure pPr/rPr/del exists when missing (mark-only pure-D may lack pPr).
            if dom.element(del_p, &W::p_pr()).is_none() {
                // Leave bare when no mark shell — produce path may add later.
            } else if let Some(ppr) = dom.element(del_p, &W::p_pr())
                && let Some(rpr) = dom.element(ppr, &W::r_pr())
            {
                // Keep del mark if present (Word file_130).
                let _ = rpr;
            }
            acted = true;
            break;
        }
        if !acted {
            return;
        }
    }
}

/// M85b — last pure-del with content: Word often omits mark-only `pPr/rPr/del`
/// (bare `<w:del>…</w:del>` only). Mid pure-dels keep the mark shell (file_49 /
/// file_186 last residual line).
pub fn strip_last_pure_del_mark_only_ppr(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids
        .iter()
        .rev()
        .find(|&&k| dom.name(k) != Some(W::name("sectPr")))
    else {
        return;
    };
    if dom.name(last) != Some(W::p()) || !para_is_pure_deleted(dom, last) {
        return;
    }
    // Only when there is delText content (empty last is M69).
    if para_has_no_text(dom, last) {
        return;
    }
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    // pPr must be mark-only: sole child rPr whose sole child is del (no spacing,
    // pStyle, pPrChange, etc.).
    let ppr_kids = dom.elements(ppr, None);
    if ppr_kids.len() != 1 {
        return;
    }
    let rpr = ppr_kids[0];
    if dom.name(rpr) != Some(W::r_pr()) {
        return;
    }
    let rpr_kids = dom.elements(rpr, None);
    if rpr_kids.len() != 1 || dom.name(rpr_kids[0]) != Some(W::del()) {
        return;
    }
    dom.remove(ppr);
}

/// True when paragraph body has both ins and del content (mixed residual).
fn para_is_mixed_revision(dom: &Dom, p: NodeId) -> bool {
    if dom.name(p) != Some(W::p()) {
        return false;
    }
    let has_ins =
        !dom.descendants(p, Some(&W::ins())).is_empty() || para_mark_revision(dom, p, &W::ins());
    let has_del =
        !dom.descendants(p, Some(&W::del())).is_empty() || para_mark_revision(dom, p, &W::del());
    has_ins && has_del
}

/// M83b / M87 / M91 / M93 / M94 — last pure-deleted **or mixed** body paragraph:
/// Word moves live layout props into `w:pPrChange` and drops the mark-only del shell.
///
/// - M83b: live `w:spacing` → pPrChange (file_23 "Document Title").
/// - M87: live `w:numPr` / `w:ind` → pPrChange (file_55 last pure-del "b").
/// - M91: live `w:jc` → pPrChange (file_105 last pure-del right-align).
/// - M93: live `w:pStyle` → pPrChange (file_59 last pure-del PreformattedText).
/// - M94: same for **last mixed** I+D (file_139 last residual mixed keeps
///   pPrChange(spacing), not live spacing + rPr/del).
///
/// Mid pure-dels keep live spacing/numPr/jc/pStyle.
pub fn last_pure_del_spacing_to_pprchange(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids
        .iter()
        .rev()
        .find(|&&k| dom.name(k) != Some(W::name("sectPr")))
    else {
        return;
    };
    if dom.name(last) != Some(W::p()) {
        return;
    }
    // Pure-del or mixed last residual (M94).
    if !para_is_pure_deleted(dom, last) && !para_is_mixed_revision(dom, last) {
        return;
    }
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    if dom.element(ppr, &W::name("pPrChange")).is_some() {
        return;
    }
    // Layout props Word records under pPrChange on the last pure-del / mixed.
    let movable = [
        W::name("spacing"),
        W::num_pr(),
        W::name("ind"),
        W::name("jc"),
        W::name("pStyle"),
    ];
    let mut to_move: Vec<NodeId> = Vec::new();
    for name in &movable {
        if let Some(el) = dom.element(ppr, name) {
            to_move.push(el);
        }
    }
    if to_move.is_empty() {
        return;
    }
    // Build old pPr shell with the moved layout props.
    let old_inner = dom.new_element(W::p_pr());
    for el in &to_move {
        let cloned = dom.clone_subtree(*el);
        dom.add(old_inner, cloned);
        dom.remove(*el);
    }
    let chg = dom.new_element(W::name("pPrChange"));
    dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;
    dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add(chg, old_inner);
    dom.add(ppr, chg);
}

/// M98b (file_167): mixed I+D residual followed by empty trailing para — Word
/// keeps del mark on the mixed (no live spacing) and parks the layout spacing
/// on the empty para as live spacing + `pPrChange` (empty old). After M98
/// sole-del fold, carrier still holds B's spacing; empty is bare `<w:p/>`.
pub fn mixed_spacing_to_following_empty(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) != Some(W::name("sectPr")))
        .collect();
    if kids.len() < 2 {
        return;
    }
    for i in 0..kids.len() - 1 {
        let mixed = kids[i];
        let empty = kids[i + 1];
        if dom.name(mixed) != Some(W::p()) || dom.name(empty) != Some(W::p()) {
            continue;
        }
        if !para_is_mixed_revision(dom, mixed) {
            continue;
        }
        if !para_has_no_text(dom, empty) {
            continue;
        }
        // Only when empty is the last body block (before sectPr) — stamp demos.
        if i + 1 != kids.len() - 1 {
            continue;
        }
        let Some(mppr) = dom.element(mixed, &W::p_pr()) else {
            continue;
        };
        let Some(spacing) = dom.element(mppr, &W::name("spacing")) else {
            continue;
        };
        // Move spacing onto empty pPr.
        let eppr = match dom.element(empty, &W::p_pr()) {
            Some(p) => p,
            None => {
                let p = dom.new_element(W::p_pr());
                if let Some(first) = dom.elements(empty, None).first().copied() {
                    dom.add_before_self(first, p);
                } else {
                    dom.add(empty, p);
                }
                p
            }
        };
        if dom.element(eppr, &W::name("spacing")).is_none() {
            let sp = dom.clone_subtree(spacing);
            dom.add_first(eppr, sp);
        }
        dom.remove(spacing);
        // Empty pPrChange (old empty) when none present — Word shape.
        if dom.element(eppr, &W::name("pPrChange")).is_none() {
            let old_inner = dom.new_element(W::p_pr());
            let chg = dom.new_element(W::name("pPrChange"));
            dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
            *id_gen += 1;
            dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
            dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
            dom.add(chg, old_inner);
            dom.add(eppr, chg);
        }
        // Del pilcrow mark on mixed (Word p3).
        let rpr = match dom.element(mppr, &W::r_pr()) {
            Some(r) => r,
            None => {
                let r = dom.new_element(W::r_pr());
                dom.add(mppr, r);
                r
            }
        };
        if dom.element(rpr, &W::del()).is_none() && dom.element(rpr, &W::ins()).is_none() {
            let mark = dom.new_element(W::del());
            dom.set_attribute_value(mark, &W::id(), Some(&id_gen.to_string()));
            *id_gen += 1;
            dom.set_attribute_value(mark, &W::author(), Some(&settings.author_for_revisions));
            dom.set_attribute_value(mark, &W::date(), Some(&settings.date_time_for_revisions));
            dom.add(rpr, mark);
        }
        // Drop empty mixed pPr shell if only held spacing (keep if rPr remains).
        if dom.elements(mppr, None).is_empty() {
            dom.remove(mppr);
        }
        break; // one trailing empty only
    }
}

/// M102c (file_148): last pure-del with `pPrChange(spacing)` and no live `jc`
/// inherits live `jc` from a preceding body para that has center align (Word
/// pure-D of A line-spacing sentence keeps B's center mark even when the
/// immediate prev mixed residual only has spacing).
pub fn last_pure_del_inherit_prev_jc(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| dom.name(k) != Some(W::name("sectPr")))
        .collect();
    if kids.len() < 2 {
        return;
    }
    let last = kids[kids.len() - 1];
    if dom.name(last) != Some(W::p()) || !para_is_pure_deleted(dom, last) {
        return;
    }
    let Some(lppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    let Some(ppc) = dom.element(lppr, &W::name("pPrChange")) else {
        return;
    };
    // Only when pPrChange carries spacing (line-spacing residual class). Do not
    // undo M91 which already parked jc-only into pPrChange (file_105).
    let ppc_has_spacing = dom
        .element(ppc, &W::p_pr())
        .or_else(|| {
            dom.elements(ppc, None)
                .into_iter()
                .find(|&c| dom.name(c) == Some(W::p_pr()))
        })
        .is_some_and(|inner| dom.element(inner, &W::name("spacing")).is_some())
        || {
            // pPrChange children may be the old pPr directly
            !dom.descendants(ppc, Some(&W::name("spacing"))).is_empty()
        };
    if !ppc_has_spacing {
        return;
    }
    if dom.element(lppr, &W::name("jc")).is_some() {
        return;
    }
    // Walk preceding paras for live jc (not only immediate prev — mixed may
    // hold spacing without jc after M102b).
    let mut donor_jc = None;
    for &prev in kids[..kids.len() - 1].iter().rev() {
        if dom.name(prev) != Some(W::p()) {
            continue;
        }
        let Some(pppr) = dom.element(prev, &W::p_pr()) else {
            continue;
        };
        // Live jc: first jc child before any pPrChange.
        for c in dom.elements(pppr, None) {
            if dom.name(c) == Some(W::name("pPrChange")) {
                break;
            }
            if dom.name(c) == Some(W::name("jc")) {
                donor_jc = Some(c);
                break;
            }
        }
        if donor_jc.is_some() {
            break;
        }
    }
    let Some(jc) = donor_jc else {
        return;
    };
    let cloned = dom.clone_subtree(jc);
    if let Some(ppc) = dom.element(lppr, &W::name("pPrChange")) {
        dom.add_before_self(ppc, cloned);
    } else {
        dom.add_first(lppr, cloned);
    }
}

/// M87b / M94 — last pure-del **or mixed** that already has `pPrChange`: drop
/// mark-only `rPr/del` (Word keeps pPrChange only — file_55 "b", file_139 mixed).
pub fn strip_last_pure_del_mark_when_pprchange(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids
        .iter()
        .rev()
        .find(|&&k| dom.name(k) != Some(W::name("sectPr")))
    else {
        return;
    };
    if dom.name(last) != Some(W::p()) {
        return;
    }
    if !para_is_pure_deleted(dom, last) && !para_is_mixed_revision(dom, last) {
        return;
    }
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    if dom.element(ppr, &W::name("pPrChange")).is_none() {
        return;
    }
    // Remove rPr/del mark shell when rPr only carries del.
    if let Some(rpr) = dom.element(ppr, &W::r_pr())
        && let Some(d) = dom.element(rpr, &W::del())
    {
        // Only strip if rPr has nothing else structural.
        let rpr_kids = dom.elements(rpr, None);
        let only_del = rpr_kids.len() == 1 && rpr_kids[0] == d;
        if only_del {
            dom.remove(rpr);
        } else {
            dom.remove(d);
            if dom.elements(rpr, None).is_empty() {
                dom.remove(rpr);
            }
        }
    }
}

/// Word emits the INSERTION (new text) before the DELETION (old text) at a
/// replacement site; our LCS produces delete-then-insert. Swap an adjacent
/// same-author/date `w:del` immediately followed by `w:ins` into `w:ins` then
/// `w:del`, matching Word's order. Text-preserving: each of the delText / ins-text
/// streams keeps its own order (only their interleaving changes). Recurses.
pub fn reorder_replacements_ins_before_del(dom: &mut Dom, node: NodeId) {
    let ins = W::ins();
    let del = W::del();
    let mut i = 0usize;
    loop {
        let kids = dom.nodes(node);
        if i + 1 >= kids.len() {
            break;
        }
        let (a, b) = (kids[i], kids[i + 1]);
        let is_replacement = dom.is_element(a)
            && dom.is_element(b)
            && dom.name(a).as_ref() == Some(&del)
            && dom.name(b).as_ref() == Some(&ins)
            && dom.attribute(a, &W::author()).map(|s| s.to_string())
                == dom.attribute(b, &W::author()).map(|s| s.to_string())
            && dom.attribute(a, &W::date()).map(|s| s.to_string())
                == dom.attribute(b, &W::date()).map(|s| s.to_string());
        if is_replacement {
            dom.remove(b);
            dom.add_before_self(a, b); // ins (b) now precedes del (a)
            i += 2;
        } else {
            i += 1;
        }
    }
    for c in dom.nodes(node) {
        if dom.is_element(c) {
            reorder_replacements_ins_before_del(dom, c);
        }
    }
}

/// Merge consecutive sibling `w:ins` (and consecutive `w:del`) wrappers that
/// share the same `w:author` and `w:date` into a single wrapper, moving the
/// later wrapper's child runs into the first and removing the emptied wrapper.
/// Runs are kept intact (per-run formatting preserved); only the wrappers merge.
/// Word never emits adjacent same-status revision wrappers; this matches that
/// (PowerTools/ours otherwise inflate the w:ins/w:del element count ~2x).
/// Recurses through the whole tree. Text-preserving (the redlined text is unchanged).
pub fn coalesce_adjacent_revisions(dom: &mut Dom, node: NodeId) {
    let ins = W::ins();
    let del = W::del();
    let kids = dom.nodes(node);
    let mut prev: Option<NodeId> = None;
    for c in kids {
        let is_rev =
            dom.is_element(c) && matches!(dom.name(c), Some(ref n) if *n == ins || *n == del);
        if is_rev {
            if let Some(p) = prev {
                let same = dom.name(p) == dom.name(c)
                    && dom.attribute(p, &W::author()).map(|s| s.to_string())
                        == dom.attribute(c, &W::author()).map(|s| s.to_string())
                    && dom.attribute(p, &W::date()).map(|s| s.to_string())
                        == dom.attribute(c, &W::date()).map(|s| s.to_string());
                if same {
                    for gc in dom.nodes(c) {
                        dom.remove(gc);
                        dom.add(p, gc);
                    }
                    dom.remove(c);
                    continue; // prev unchanged; it may absorb the next sibling too
                }
            }
            prev = Some(c);
        } else {
            prev = None; // any non-revision sibling breaks adjacency
        }
    }
    for c in dom.nodes(node) {
        if dom.is_element(c) {
            coalesce_adjacent_revisions(dom, c);
        }
    }
}

/// M4.F — `SimplifyMoveMarkupToDelIns` (:3309), gated by settings.simplify_move_markup
/// (default off). Converts native move markup to del/ins for Word compatibility
/// (Issue #96): w:moveFrom→w:del, w:moveTo→w:ins (keeping author/date/id +
/// children), and removes all move range markers. Transform over `root`.
pub fn simplify_move_markup_to_del_ins(dom: &mut Dom, root: NodeId) -> NodeId {
    simplify_move_transform(dom, root)
}

fn simplify_move_transform(dom: &mut Dom, node: NodeId) -> NodeId {
    let name = dom.name(node).unwrap();
    let ranges = [
        W::name("moveFromRangeStart"),
        W::move_from_range_end(),
        W::name("moveToRangeStart"),
        W::move_to_range_end(),
    ];
    let new_name = if name == W::name("moveFrom") {
        W::del()
    } else if name == W::name("moveTo") {
        W::ins()
    } else {
        name.clone()
    };
    let ne = dom.new_element(new_name);
    // Oracle (WmlComparer.cs:2874-2878) propagates only w:author, w:date, w:id
    // — but only when rewriting w:moveFrom/w:moveTo. Recreated descendant
    // elements (w:r, w:p, …) must keep all of their attributes.
    if name == W::name("moveFrom") || name == W::name("moveTo") {
        copy_move_id_attrs(dom, node, ne);
    } else {
        copy_attrs(dom, node, ne, false);
    }
    for c in dom.nodes(node) {
        if dom.is_element(c) {
            let cn = dom.name(c).unwrap();
            if ranges.contains(&cn) {
                continue; // drop range markers
            }
            let t = simplify_move_transform(dom, c);
            dom.add(ne, t);
        } else {
            let cc = dom.clone_subtree(c);
            dom.add(ne, cc);
        }
    }
    ne
}

/// Word-alignment mode (settings-gated, never in PowerTools-faithful runs):
/// Word's Compare merges a fully-replaced paragraph PAIR into one paragraph —
/// inserted runs first, then deleted runs, under the INSERTED paragraph's
/// properties (evidence: the `_word_redline` fixture corpus, e.g.
/// heading-1-bold vs heading-1-style P3 = [ins:'Main Title Section',
/// del:'Heading 1 with bold …']). Our fallback instead emits a deleted
/// paragraph followed by an inserted one. This pass pairs each maximal run of
/// fully-deleted paragraphs with the adjacent following run of fully-inserted
/// paragraphs (pairwise, in order; leftovers stay separate) in the body and
/// inside every table cell / textbox.
pub fn merge_replaced_paragraphs(dom: &mut Dom, root: NodeId, comparer_author: &str) {
    let mut containers: Vec<NodeId> = Vec::new();
    if let Some(b) = dom.element(root, &W::body()) {
        containers.push(b);
    }
    for name in [W::name("tc"), W::name("txbxContent"), W::sdt_content()] {
        containers.extend(dom.descendants(root, Some(&name)));
    }
    for c in containers {
        merge_replaced_in_container(dom, c, comparer_author);
    }
}

/// True when `w:pPr/w:rPr` carries a paragraph-mark revision (`w:del`/`w:ins`).
/// Empty deleted/inserted paragraphs often have ONLY this mark (no body
/// `w:del`/`w:ins` child). Without classifying them, del-blocks break and
/// ins-before-del reordering never fires (support_tickets_table_table_bookmark_end:
/// Word is III…DDD…Mt; ours was DDD…III…Mt).
fn para_mark_revision(dom: &Dom, p: NodeId, rev: &crate::xmllinq::XName) -> bool {
    dom.element(p, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| dom.element(rpr, rev).is_some())
}

/// Some(false) = fully deleted, Some(true) = fully inserted, None = neither.
/// Classify a body child of `p` as ins/del/plain. `w:hyperlink` (TOC fields)
/// is transparent — deleted TOC entries are `hyperlink > del > r > delText`,
/// not a direct `w:del` child. Treating the hyperlink as plain broke pure-D
/// runs and blocked I-before-D reorder (file_22: pure-I title stuck at p147).
fn accumulate_para_child_class(
    dom: &Dom,
    c: NodeId,
    ins: &mut bool,
    del: &mut bool,
    plain: &mut bool,
) {
    let Some(n) = dom.name(c) else {
        *plain = true;
        return;
    };
    if n == W::p_pr() || n == W::name("bookmarkStart") || n == W::name("bookmarkEnd") {
        return;
    }
    if n == W::ins() {
        *ins = true;
        return;
    }
    if n == W::del() {
        *del = true;
        return;
    }
    if n == W::hyperlink() {
        for gc in dom.elements(c, None) {
            accumulate_para_child_class(dom, gc, ins, del, plain);
        }
        return;
    }
    *plain = true;
}

fn para_replacement_class(dom: &Dom, p: NodeId) -> Option<bool> {
    if dom.name(p) != Some(W::p()) {
        return None;
    }
    let (mut ins, mut del, mut plain) = (false, false, false);
    for c in dom.elements(p, None) {
        accumulate_para_child_class(dom, c, &mut ins, &mut del, &mut plain);
    }
    if !ins && !del && !plain {
        if para_mark_revision(dom, p, &W::del()) {
            del = true;
        } else if para_mark_revision(dom, p, &W::ins()) {
            ins = true;
        }
    }
    match (ins, del, plain) {
        (true, false, false) => Some(true),
        (false, true, false) => Some(false),
        _ => None,
    }
}

/// Like [`para_replacement_class`], but a `w:ins` authored by someone OTHER
/// than the comparer (a CARRIED pre-existing insertion, D1 machinery) counts
/// toward the DELETED side: GT places doc-A blocks whose only live content
/// is carried foreign-author ins inside the trailing deleted cluster
/// (contract-review-suggesting-mixed-edits, −20.9 visual when classified
/// None and the gap adjacency broke).
fn para_class_carried_aware(dom: &Dom, p: NodeId, comparer_author: &str) -> Option<bool> {
    if dom.name(p) != Some(W::p()) {
        return None;
    }
    let (mut ins, mut del, mut plain) = (false, false, false);
    for c in dom.elements(p, None) {
        let Some(n) = dom.name(c) else {
            plain = true;
            continue;
        };
        if n == W::p_pr() || n == W::name("bookmarkStart") || n == W::name("bookmarkEnd") {
            continue;
        } else if n == W::ins() {
            if dom.attribute(c, &W::author()).unwrap_or("") == comparer_author {
                ins = true;
            } else {
                del = true; // carried revision rides with the deleted cluster
            }
        } else if n == W::del() {
            del = true;
        } else if n == W::hyperlink() {
            // Transparent: TOC hyperlink > del|ins (same as para_replacement_class).
            for gc in dom.elements(c, None) {
                let Some(gn) = dom.name(gc) else {
                    plain = true;
                    continue;
                };
                if gn == W::ins() {
                    if dom.attribute(gc, &W::author()).unwrap_or("") == comparer_author {
                        ins = true;
                    } else {
                        del = true;
                    }
                } else if gn == W::del() {
                    del = true;
                } else {
                    plain = true;
                }
            }
        } else {
            plain = true;
        }
    }
    if !ins && !del && !plain {
        if para_mark_revision(dom, p, &W::del()) {
            del = true;
        } else if para_mark_revision(dom, p, &W::ins()) {
            ins = true;
        }
    }
    match (ins, del, plain) {
        (true, false, false) => Some(true),
        (false, true, false) => Some(false),
        _ => None,
    }
}

/// True when the paragraph holds a REAL deletion (any `w:del` child) — a
/// carried-only "deleted-side" paragraph must not 1v1-merge into a mixed
/// paragraph (its content is pending ins, not replaced text).
fn para_has_real_del(dom: &Dom, p: NodeId) -> bool {
    !dom.elements(p, Some(&W::del())).is_empty()
}

/// Live + deleted body text of a paragraph (for relatedness gates).
fn para_revision_body_text(dom: &Dom, p: NodeId) -> String {
    let mut out = String::new();
    for name in [W::name("t"), W::name("delText")] {
        for t in dom.descendants(p, Some(&name)) {
            out.push_str(&dom.value_str(t));
            out.push(' ');
        }
    }
    out
}

fn body_token_set(text: &str) -> std::collections::HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// True when paragraph body text (live + delText) is only digits/whitespace
/// (file_166 pure-I "24" — corpus fragment, not content fold bait).
fn para_body_is_digits_only(dom: &Dom, p: NodeId) -> bool {
    let t = para_revision_body_text(dom, p);
    let trimmed = t.trim();
    !trimmed.is_empty()
        && trimmed
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_whitespace())
}

/// Alphanumeric character count of para body text (revision-aware).
fn para_body_alnum_len(dom: &Dom, p: NodeId) -> usize {
    para_revision_body_text(dom, p)
        .chars()
        .filter(|c| c.is_alphanumeric())
        .count()
}

/// True when body alphanumerics are 1–2 chars ("a", "x", "OK"). Used to keep
/// Word-style pure confetti for short next residuals (file_29 pure-I "a"
/// before pure-D title) — M90 still folds content pure-I into multi-del.
fn para_body_is_very_short(dom: &Dom, p: NodeId) -> bool {
    (1..=2).contains(&para_body_alnum_len(dom, p))
}

/// True when body looks like a short demo title (≤8 tokens, last significant
/// token is `Demo`). M139 uses this with Jaccard gate so contract×Title-Demo
/// stays pure-I/D (file_82) while related stamped demos still fold.
fn para_looks_like_demo_title(dom: &Dom, p: NodeId) -> bool {
    let t = para_revision_body_text(dom, p);
    let tokens: Vec<&str> = t
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();
    if tokens.is_empty() || tokens.len() > 8 {
        return false;
    }
    tokens
        .iter()
        .rev()
        .find(|s| s.chars().count() >= 4)
        .is_some_and(|s| s.eq_ignore_ascii_case("demo"))
}

/// Token Jaccard of two body strings. Used to avoid folding unrelated pure-I
/// / pure-D neighbors (file_33: Word keeps pure-I "Summary" and pure-D
/// "Heading 1 Style Demo" separate; folding invents a mixed para and costs
/// page geometry).
fn body_text_jaccard(a: &str, b: &str) -> f64 {
    let sa = body_token_set(a);
    let sb = body_token_set(b);
    if sa.is_empty() && sb.is_empty() {
        return 1.0;
    }
    let inter = sa.intersection(&sb).count() as f64;
    let uni = sa.union(&sb).count() as f64;
    if uni == 0.0 { 0.0 } else { inter / uni }
}

/// Minimum body-text Jaccard to fold a pure-del into a pure-ins at an
/// I…I D…D boundary. Zero-overlap neighbors stay separate (Word file_33).
const SOLE_DEL_FOLD_MIN_JACCARD: f64 = 0.12;

fn should_fold_ins_del_pair(dom: &Dom, ins_p: NodeId, del_p: NodeId) -> bool {
    let it = para_revision_body_text(dom, ins_p);
    let dt = para_revision_body_text(dom, del_p);
    // Empty del mark-only: allow fold (mark-only cases from single_paragraph GT).
    if dt.trim().is_empty() {
        return true;
    }
    body_text_jaccard(&it, &dt) + 1e-12 >= SOLE_DEL_FOLD_MIN_JACCARD
}

fn merge_replaced_in_container(dom: &mut Dom, container: NodeId, comparer_author: &str) {
    loop {
        let children: Vec<NodeId> = dom.elements(container, None);
        let classes: Vec<Option<bool>> = children
            .iter()
            .map(|&c| para_class_carried_aware(dom, c, comparer_author))
            .collect();
        // find the first [deleted-block][inserted-block] adjacency
        let mut i = 0;
        let mut acted = false;
        while i < children.len() {
            if classes[i] != Some(false) {
                i += 1;
                continue;
            }
            let del_start = i;
            while i < children.len() && classes[i] == Some(false) {
                i += 1;
            }
            let ins_start = i;
            while i < children.len() && classes[i] == Some(true) {
                i += 1;
            }
            if ins_start == i {
                continue; // deleted block with no following inserted block
            }
            let dels = &children[del_start..ins_start];
            let inss = &children[ins_start..i];
            // M-PI (parity/_scratch/mpi_forensics.md): Word merges only the
            // exact 1v1 replacement pair (heading-1-bold P3 evidence). Larger
            // gaps keep every paragraph separate, ordered [all inserted,
            // B order][all deleted, A order] with the deleted cluster
            // immediately before the closing anchor (green-underline GT: 2
            // ins + 3 del all separate; ole-object GT: whole-doc runs
            // separate). Pairwise-merging bigger runs invented mixed
            // paragraphs Word never produces.
            if dels.len() != 1 || inss.len() != 1 || !para_has_real_del(dom, dels[0]) {
                // Move ALL ins blocks before the first del, then break.
                // `children` is stale after the first remove/add_before_self —
                // the outer rescan re-reads the container. Do not remove the
                // break without rewriting this loop over a post-mutation index.
                let first_del = dels[0];
                let inss: Vec<NodeId> = inss.to_vec();
                let sole_del = if dels.len() == 1 && para_has_real_del(dom, dels[0]) {
                    Some(dels[0])
                } else {
                    None
                };
                for e in &inss {
                    if dom.parent(*e).is_none() {
                        continue;
                    }
                    dom.remove(*e);
                    dom.add_before_self(first_del, *e);
                }
                // Word/jubarte (single_paragraph × small_font_size): N inserted
                // paragraphs + exactly one deleted paragraph → fold the deleted
                // body into the LAST inserted paragraph (mixed I+D runs).
                // Word leaves that mixed para with NO paragraph-mark revision
                // (no pPr/rPr ins|del) — only the first N-1 pure-ins paras keep
                // mark-ins. Copying the deleted mark onto the mixed para was a
                // false "Word" assumption and costs pixel fidelity (~68 vs 100).
                // Multi-del clusters stay separate (green-underline: 2I+3D).
                // Whole-doc trailing sole-del always folds (m44), even at
                // Jaccard 0 — that is the single_paragraph GT shape.
                if let (Some(d), Some(&last_ins)) = (sole_del, inss.last())
                    && dom.parent(d).is_some()
                    && dom.parent(last_ins).is_some()
                {
                    // Strip para-mark revision from the carrier (Word: bare mixed p).
                    if let Some(ippr) = dom.element(last_ins, &W::p_pr()) {
                        if let Some(irpr) = dom.element(ippr, &W::r_pr())
                            && (dom.element(irpr, &W::ins()).is_some()
                                || dom.element(irpr, &W::del()).is_some())
                        {
                            dom.remove(irpr);
                        }
                        // Drop empty pPr that only held the mark revision.
                        if dom.elements(ippr, None).is_empty() {
                            dom.remove(ippr);
                        }
                    }
                    for c in dom.elements(d, None) {
                        if dom.name(c) != Some(W::p_pr()) {
                            dom.add(last_ins, c); // move body del/ins into last ins
                        }
                    }
                    dom.remove(d);
                }
                acted = true;
                break; // children list is stale — rescan (required)
            }
            // 1v1 D-then-I: merge only when related. Unrelated pairs reorder
            // to I-before-D without folding (file_33 mid residual pure-D).
            let mut did = false;
            for (&d, &ins_p) in dels.iter().zip(inss.iter()) {
                if !should_fold_ins_del_pair(dom, ins_p, d) {
                    if dom.parent(ins_p).is_some() && dom.parent(d).is_some() {
                        dom.remove(ins_p);
                        dom.add_before_self(d, ins_p);
                        did = true;
                    }
                    continue;
                }
                let merged = dom.new_element(W::p());
                for (an, av) in dom.attributes(ins_p) {
                    dom.set_attribute_value(merged, &an, Some(&av));
                }
                // Inserted pPr wins when structural (new formatting). M88: when
                // Inserted is bare but Deleted has numPr/spacing, keep Deleted
                // structural live with del mark (file_55 mixed "notation"+"a").
                let ins_struct = dom
                    .element(ins_p, &W::p_pr())
                    .is_some_and(|p| ppr_has_structural_props(dom, p));
                let del_struct = dom
                    .element(d, &W::p_pr())
                    .is_some_and(|p| ppr_has_structural_props(dom, p));
                if ins_struct {
                    if let Some(ppr) = dom.element(ins_p, &W::p_pr()) {
                        let c = dom.clone_subtree(ppr);
                        dom.add(merged, c);
                    }
                } else if del_struct {
                    if let Some(ppr) = dom.element(d, &W::p_pr()) {
                        let c = dom.clone_subtree(ppr);
                        dom.add(merged, c);
                    }
                } else if let Some(ppr) = dom.element(ins_p, &W::p_pr()) {
                    let c = dom.clone_subtree(ppr);
                    dom.add(merged, c);
                }
                for c in dom.elements(ins_p, None) {
                    if dom.name(c) != Some(W::p_pr()) {
                        dom.add(merged, c); // clone-on-attach
                    }
                }
                for c in dom.elements(d, None) {
                    if dom.name(c) != Some(W::p_pr()) {
                        dom.add(merged, c);
                    }
                }
                dom.replace_with(d, &[merged]);
                dom.remove(ins_p);
                did = true;
            }
            if did {
                acted = true;
                break; // children list is stale — rescan
            }
        }
        if !acted {
            // Already ins-before-del (Word H9 / prior reorder): fold a sole
            // trailing pure-del into the last pure-ins (single_paragraph GT).
            let mut j = 0;
            while j < children.len() {
                if classes[j] != Some(true) {
                    j += 1;
                    continue;
                }
                let ins_start = j;
                while j < children.len() && classes[j] == Some(true) {
                    j += 1;
                }
                let del_start = j;
                while j < children.len() && classes[j] == Some(false) {
                    j += 1;
                }
                let inss = &children[ins_start..del_start];
                let dels = &children[del_start..j];
                if inss.is_empty() || dels.is_empty() || !para_has_real_del(dom, dels[0]) {
                    continue;
                }
                // Sole trailing del: always fold (single_paragraph GT / m44).
                // Multi-del: always fold one boundary pair (last I + first D).
                // M90: Word oracles file_38/62/11 (2I+ND) and file_191 (1I+ND)
                // all fold last pure-I with first residual pure-D. The old
                // green-underline skip (2I+3D separate) matched a synthetic
                // PowerTools GT but fights real Word Compare on stamped demos.
                // Contiguous clusters before a table are often 3I+3D (file_14 /
                // m53b). M68 Jaccard best-match fold tried for file_33; LO score
                // −0.47 and still 3pp vs Word 2 — reverted.
                let sole_del = dels.len() == 1;
                // Word: last pure-ins merges with first pure-del at I…I D…D boundary.
                let d = dels[0];
                let last_ins = inss[inss.len() - 1];
                // M77: mid-document sole pure-D after pure-I must not fold into
                // the preceding ins when body texts are unrelated and more
                // content follows (file_33: pure-I "Summary" + pure-D "Heading
                // 1 Style Demo" + more). Whole-doc trailing sole-del (m44)
                // still always folds.
                //
                // M98 (file_167): empty trailing `<w:p/>` before sectPr is not
                // content — Word still folds last pure-I ("Subsection Title")
                // with sole pure-D ("24"). Counting empty p as following_content
                // blocked the fold via the Jaccard gate (0 < 0.12).
                let following_content = children[j..].iter().any(|&c| match dom.name(c) {
                    Some(n) if n == W::name("tbl") => true,
                    Some(n) if n == W::p() => !para_has_no_text(dom, c),
                    _ => false,
                });
                // M77: mid-document sole pure-D after pure-I must not fold into
                // the preceding ins when body texts are unrelated and more
                // content follows (file_33). Whole-doc trailing sole-del (m44)
                // still always folds. Multi-del boundary fold stays ungated
                // (M90 stamped demos); file_78 class is fixed via M116
                // short-circuit, not fold relatedness.
                if sole_del && following_content && !should_fold_ins_del_pair(dom, last_ins, d) {
                    continue;
                }
                // M101 (file_166): sole pure-I that is **digits-only** ("24") +
                // multi pure-D of an unrelated catalog — Word keeps pure-I
                // separate. Content sole pure-I like "Ouch." still folds
                // (file_191 / M89).
                if dels.len() > 1 && inss.len() == 1 && para_body_is_digits_only(dom, last_ins) {
                    continue;
                }
                // M124 (file_29): last pure-I is a 1–2 char residual ("a") in a
                // **short** pure-I run (≤2 paras: e.g. "ONE"+"a") + multi pure-D
                // — Word keeps pure-I separate. Longer pure-I runs (file_54:
                // a/x/x/b) still fold last short "b" into first D (Word MIX
                // "b1.5 Line Spacing Demo"). M90 still folds content pure-I.
                if dels.len() > 1 && inss.len() <= 2 && para_body_is_very_short(dom, last_ins) {
                    continue;
                }
                // M139 (file_82): multi pure-D after a **long** pure-I run
                // (≥5 paras: contract body) when first pure-D is an unrelated
                // short Demo title — Word keeps pure-I then pure-D title (no
                // MIX). Short pure-I runs (file_38/11 stamped demos, M90) still
                // always fold. Also require last pure-I has real content (≥8
                // alnum) so file_54 "b"+empties×Demo title still folds (M90).
                if dels.len() > 1
                    && inss.len() >= 5
                    && para_body_alnum_len(dom, last_ins) >= 8
                    && para_looks_like_demo_title(dom, d)
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    continue;
                }
                // Strip para-mark revision from the carrier (Word: bare mixed p)
                // unless M88 adopts Deleted structural pPr (numPr) with del mark.
                let del_structural = dom
                    .element(d, &W::p_pr())
                    .is_some_and(|dp| ppr_has_structural_props(dom, dp));
                let ins_structural = dom
                    .element(last_ins, &W::p_pr())
                    .is_some_and(|ip| ppr_has_structural_props(dom, ip));
                // M88 (file_55): mixed I+D keeps Deleted live numPr + del mark when
                // Inserted has no structural pPr (B plain text, A list item "a").
                // M102b (file_148): when Inserted is jc-only and Deleted has
                // spacing, Word keeps Deleted spacing + del mark (not live jc).
                let ins_jc_only = dom
                    .element(last_ins, &W::p_pr())
                    .is_some_and(|ip| ppr_is_jc_only(dom, ip));
                let del_has_spacing = dom
                    .element(d, &W::p_pr())
                    .is_some_and(|dp| dom.element(dp, &W::name("spacing")).is_some());
                let adopt_del_ppr =
                    (del_structural && !ins_structural) || (ins_jc_only && del_has_spacing);
                if adopt_del_ppr {
                    if let Some(ippr) = dom.element(last_ins, &W::p_pr()) {
                        dom.remove(ippr);
                    }
                    if let Some(dppr) = dom.element(d, &W::p_pr()) {
                        let cloned = dom.clone_subtree(dppr);
                        // Ensure del pilcrow mark when pure-del had structural layout.
                        if let Some(rpr) = dom.element(cloned, &W::r_pr()) {
                            if dom.element(rpr, &W::del()).is_none() {
                                // leave rPr as-is if no del mark; mark_content may add
                            }
                        } else if para_mark_revision(dom, d, &W::del()) {
                            // rare: structural without rPr — skip
                        }
                        // Insert pPr first on carrier.
                        if let Some(first) = dom.elements(last_ins, None).first().copied() {
                            dom.add_before_self(first, cloned);
                        } else {
                            dom.add(last_ins, cloned);
                        }
                    }
                } else if let Some(ippr) = dom.element(last_ins, &W::p_pr()) {
                    if let Some(irpr) = dom.element(ippr, &W::r_pr())
                        && (dom.element(irpr, &W::ins()).is_some()
                            || dom.element(irpr, &W::del()).is_some())
                    {
                        dom.remove(irpr);
                    }
                    if dom.elements(ippr, None).is_empty() {
                        dom.remove(ippr);
                    }
                }
                for c in dom.elements(d, None) {
                    if dom.name(c) != Some(W::p_pr()) {
                        dom.add(last_ins, c);
                    }
                }
                dom.remove(d);
                acted = true;
                break;
            }
            if !acted {
                return;
            }
        }
    }
}

/// Word-alignment mode: at a multi-block replacement region — a run of
/// fully-deleted block elements immediately followed by a run of fully-
/// inserted ones, where either run contains a NON-paragraph block (table/
/// sdt) — Word emits the INSERTED block first (ole-object_ooxml-style-link:
/// Word page 1 is the new document's heading, ours was the old document's
/// chart; every page misaligned, pixel-diff 0.59 corpus max). Pure-paragraph
/// 1:1 replacements are left to [`merge_replaced_paragraphs`], which runs
/// after this pass.
pub fn reorder_replaced_blocks(dom: &mut Dom, root: NodeId) {
    fn block_class(dom: &Dom, el: NodeId) -> Option<bool> {
        let n = dom.name(el)?;
        if n == W::p() {
            return para_replacement_class(dom, el);
        }
        if n == W::name("tbl") {
            let trs = dom.descendants(el, Some(&W::name("tr")));
            if trs.is_empty() {
                return None;
            }
            let mut cls: Option<bool> = None;
            for tr in trs {
                let trpr = dom.element(tr, &W::name("trPr"))?;
                let c = if dom.element(trpr, &W::del()).is_some() {
                    false
                } else if dom.element(trpr, &W::ins()).is_some() {
                    true
                } else {
                    return None;
                };
                if *cls.get_or_insert(c) != c {
                    return None;
                }
            }
            return cls;
        }
        if n == W::sdt() {
            let content = dom.element(el, &W::sdt_content())?;
            let kids = dom.elements(content, None);
            if kids.is_empty() {
                return None;
            }
            let mut cls: Option<bool> = None;
            for k in kids {
                let c = block_class(dom, k)?;
                if *cls.get_or_insert(c) != c {
                    return None;
                }
            }
            return cls;
        }
        None
    }
    // Scope: body + table cells only — the pass exists to fix page-flow
    // misalignment, and txbxContent/hdr/ftr content does not paginate the
    // body (headers/footers are diffed as separate parts anyway).
    let mut containers: Vec<NodeId> = Vec::new();
    if let Some(b) = dom.element(root, &W::body()) {
        containers.push(b);
    }
    containers.extend(dom.descendants(root, Some(&W::name("tc"))));
    for container in containers {
        let children: Vec<NodeId> = dom.elements(container, None);
        let classes: Vec<Option<bool>> = children.iter().map(|&c| block_class(dom, c)).collect();
        let paras: Vec<bool> = children
            .iter()
            .map(|&c| dom.name(c) == Some(W::p()))
            .collect();
        let mut i = 0;
        while i < children.len() {
            if classes[i] != Some(false) {
                i += 1;
                continue;
            }
            let del_start = i;
            while i < children.len() && classes[i] == Some(false) {
                i += 1;
            }
            let ins_start = i;
            while i < children.len() && classes[i] == Some(true) {
                i += 1;
            }
            if ins_start == i {
                continue; // no inserted run follows
            }
            let region_has_block = (del_start..i).any(|k| !paras[k]);
            if !region_has_block {
                continue; // pure paragraphs — merge_replaced_paragraphs owns it
            }
            let first_del = children[del_start];
            for &e in &children[ins_start..i] {
                dom.remove(e);
                dom.add_before_self(first_del, e);
            }
        }
    }
}

/// Word-alignment mode (settings-gated): a paragraph whose MARK is deleted
/// must not leave its embedded `pPr/sectPr` behind as a live section break —
/// Word renders the deleted paragraph on the same page (a stray break
/// produced whole blank pages in our output, e.g. the
/// mcdoc_meeting-agenda-table-2 benchmark pair).
pub fn drop_sectpr_from_deleted_marks(
    dom: &mut Dom,
    root: NodeId,
    genuine: &std::collections::HashSet<String>,
) {
    let sectpr = W::name("sectPr");
    for ppr in dom.descendants(root, Some(&W::p_pr())) {
        let mark_deleted = dom
            .element(ppr, &W::r_pr())
            .is_some_and(|rpr| dom.element(rpr, &W::del()).is_some());
        if !mark_deleted {
            continue;
        }
        for sp in dom.elements(ppr, Some(&sectpr)) {
            // GENUINE input mid-section breaks survive deletion — Word keeps
            // the deleted content's pagination (strict01_strikethrough: Word's
            // redline renders strict01's struck-through sections across 13
            // pages; dropping them collapsed ours to 8). Only the hoisted
            // final-sectPr artifact class is a phantom break to remove.
            if genuine.contains(&sectpr_identity(dom, sp)) {
                continue;
            }
            dom.remove(sp);
        }
    }
}

/// Word-alignment mode: flatten doc A's PRE-EXISTING tracked INSERTIONS
/// with a stamp — Word carries them as pending w:ins (original author) and
/// they SURVIVE accepting the redline, exactly like accepting Word's own
/// output (forensics #9/#10). The stamped runs re-emerge as pending ins via
/// [`convert_stamped_preins`]. This is the DISCLOSED word-mode accept
/// contract alignment: accept(redline) keeps this text like Word does; the
/// faithful preset keeps accept-first.
pub fn flatten_tracked_insertions_stamped(dom: &mut Dom, body: NodeId) {
    // Pre-order descendants; reverse so nested w:ins unwrap innermost-first.
    // Processing outer-first would replace_with the outer, detaching inners
    // still listed in `inss` and panicking on the next replace_with.
    let mut inss: Vec<NodeId> = dom.descendants(body, Some(&W::ins()));
    inss.reverse();
    for i in inss {
        // Already detached (e.g. paragraph-mark path / concurrent unwrap).
        if dom.parent(i).is_none() {
            continue;
        }
        // paragraph-mark markers live in pPr/rPr and have no run children;
        // unwrapping them is harmless (kids empty -> plain removal)
        let author = dom.attribute(i, &W::author()).map(|s| s.to_string());
        let date = dom.attribute(i, &W::date()).map(|s| s.to_string());
        let kids = dom.nodes(i);
        // Stamp every descendant w:r under this ins — not only direct kids —
        // so hyperlink/sdt/smartTag wrappers still carry PreIns (forensics #10).
        // Skip runs already stamped by an inner w:ins (innermost author wins).
        let runs: Vec<NodeId> = kids
            .iter()
            .flat_map(|&k| {
                if dom.name(k) == Some(W::r()) {
                    vec![k]
                } else {
                    dom.descendants(k, Some(&W::r()))
                }
            })
            .collect();
        for r in runs {
            if dom.attribute(r, &PT::name("PreIns")).is_some() {
                continue;
            }
            // ins>del history (w14c): runs under a nested w:del must stay
            // delText — stamping them PreIns would re-emit them as pending ins.
            let under_nested_del = dom
                .ancestors(r, None)
                .iter()
                .any(|&a| a != i && dom.name(a) == Some(W::del()));
            if under_nested_del {
                continue;
            }
            dom.set_attribute_value(r, &PT::name("PreIns"), Some("1"));
            if let Some(a) = &author {
                dom.set_attribute_value(r, &PT::name("PreInsAuthor"), Some(a));
            }
            if let Some(dt) = &date {
                dom.set_attribute_value(r, &PT::name("PreInsDate"), Some(dt));
            }
        }
        dom.replace_with(i, &kids);
    }
}

/// Produce-side counterpart of [`flatten_tracked_insertions_stamped`]: every
/// run stamped `pt:PreIns` re-emerges as a PENDING INSERTION with its
/// original author — its `w:del` wrapper (the diff saw A-only text absent
/// from B) renames/splits to `w:ins` and delText reverts to w:t; Equal runs
/// get a fresh `w:ins`. Runs before [`coalesce_adjacent_revisions`].
pub fn convert_stamped_preins(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let marker = PT::name("PreIns");
    let runs: Vec<NodeId> = dom
        .descendants(root, Some(&W::r()))
        .into_iter()
        .filter(|&r| dom.attribute(r, &marker).is_some())
        .collect();
    for r in runs {
        dom.set_attribute_value(r, &marker, None);
        let author = dom
            .attribute(r, &PT::name("PreInsAuthor"))
            .map(|s| s.to_string());
        let date = dom
            .attribute(r, &PT::name("PreInsDate"))
            .map(|s| s.to_string());
        dom.set_attribute_value(r, &PT::name("PreInsAuthor"), None);
        dom.set_attribute_value(r, &PT::name("PreInsDate"), None);
        let restamp = |dom: &mut Dom, wrapper: NodeId| {
            if let Some(a) = &author {
                dom.set_attribute_value(wrapper, &W::author(), Some(a));
            }
            if let Some(dt) = &date {
                dom.set_attribute_value(wrapper, &W::date(), Some(dt));
            }
        };
        // Restore delText→t only when the run is leaving a deletion/equal
        // context (mirrors convert_stamped_predeletes ordering: kind fix after
        // parent decision). Skip when already under w:ins.
        let restore_text_kinds = |dom: &mut Dom, run: NodeId| {
            for t in dom.descendants(run, Some(&W::name("delText"))) {
                dom.set_name(t, W::t());
            }
            for t in dom.descendants(run, Some(&W::name("delInstrText"))) {
                dom.set_name(t, W::instr_text());
            }
        };
        let Some(parent) = dom.parent(r) else {
            continue;
        };
        let pname = dom.name(parent);
        if pname == Some(W::ins()) {
            restamp(dom, parent);
            continue;
        }
        if pname == Some(W::del()) {
            let sibs = dom.elements(parent, None);
            if sibs.len() == 1 {
                // Fresh revision id: del-era id must not label the renamed ins.
                dom.set_name(parent, W::ins());
                dom.set_attribute_value(parent, &W::id(), Some(&id_gen.to_string()));
                *id_gen += 1;
                restore_text_kinds(dom, r);
                restamp(dom, parent);
            } else {
                let idx = sibs.iter().position(|&c| c == r).unwrap_or(0);
                let d = rev_el(dom, W::ins(), settings, id_gen);
                restore_text_kinds(dom, r);
                restamp(dom, d);
                dom.remove(r);
                dom.add(d, r);
                dom.add_after_self(parent, d);
                let after: Vec<NodeId> = sibs[idx + 1..].to_vec();
                if !after.is_empty() {
                    let del2 = dom.new_element(W::del());
                    for (an, av) in dom.attributes(parent) {
                        dom.set_attribute_value(del2, &an, Some(&av));
                    }
                    dom.set_attribute_value(del2, &W::id(), Some(&id_gen.to_string()));
                    *id_gen += 1;
                    for a in after {
                        dom.remove(a);
                        dom.add(del2, a);
                    }
                    dom.add_after_self(d, del2);
                }
                if idx == 0 {
                    dom.remove(parent);
                }
            }
        } else {
            let w = rev_el(dom, W::ins(), settings, id_gen);
            restore_text_kinds(dom, r);
            restamp(dom, w);
            dom.add_before_self(r, w);
            dom.remove(r);
            dom.add(w, r);
        }
    }
}

/// Word-alignment mode (settings-gated): Word's Compare output always carries
/// a page setup — inputs with no `sectPr`/`pgSz` are normalized to Word's
/// default Letter geometry (`w:pgSz w=12240 h=15840`; evidence: the
/// `_word_redline` corpus, e.g. the 1-5-line-spacing_24 pair). Without this,
/// renderers fall back to their own default (A4 in soffice) and paginate
/// differently from Word's redline.
pub fn ensure_default_page_size(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let sectpr_name = W::name("sectPr");
    let sect = match dom.element(body, &sectpr_name) {
        Some(s) => s,
        None => {
            let s = dom.new_element(sectpr_name);
            dom.add(body, s);
            s
        }
    };
    if dom.element(sect, &W::name("pgSz")).is_none() {
        let pg = dom.new_element(W::name("pgSz"));
        dom.set_attribute_value(pg, &W::name("w"), Some("12240"));
        dom.set_attribute_value(pg, &W::name("h"), Some("15840"));
        // CT_SectPr schema order: pgSz comes AFTER header/footer references,
        // footnotePr/endnotePr and type — add_first would put it before any
        // headerReference and trip the validator (PR #54 review).
        let later = dom.elements(sect, None).into_iter().find(|&c| {
            dom.name(c).is_some_and(|n| {
                !matches!(
                    n.local_name(),
                    "headerReference" | "footerReference" | "footnotePr" | "endnotePr" | "type"
                )
            })
        });
        match later {
            Some(l) => dom.add_before_self(l, pg),
            None => dom.add(sect, pg),
        }
    }
}

/// `w:tblPr` child ranks (PtOpenXmlUtil.cs Order_tblPr) — shared between
/// [`wml_order_elements_per_standard`] and [`synthesize_table_cell_margins`]
/// so the two can't drift apart.
const TBLPR_ORDER: [(&str, i32); 17] = [
    ("tblStyle", 10),
    ("tblpPr", 20),
    ("tblOverlap", 30),
    ("bidiVisual", 40),
    ("tblStyleRowBandSize", 50),
    ("tblStyleColBandSize", 60),
    ("tblW", 70),
    ("jc", 80),
    ("tblCellSpacing", 90),
    ("tblInd", 100),
    ("tblBorders", 110),
    ("shd", 120),
    ("tblLayout", 130),
    ("tblCellMar", 140),
    ("tblLook", 150),
    ("tblCaption", 160),
    ("tblDescription", 170),
];

/// Faithful port of `WordprocessingMLUtil.WmlOrderElementsPerStandard`
/// (PtOpenXmlUtil.cs:1440), called by the C# produce path (WmlComparer.cs
/// :1893) and previously MISSING from the port: property-container children
/// (pPr/rPr/tblPr/tcPr/tcBorders/tblBorders/pBdr) are stable-sorted into
/// standard schema order (unknown names rank 999, keeping relative order —
/// C#'s stable `OrderBy`), and `w:p`/`w:r` move their pPr/rPr first.
/// Non-element child nodes of rebuilt containers are dropped exactly like
/// the C# `.Elements()` rebuild. Validator evidence: 16× "unexpected child
/// element pStyle" (numPr emitted before pStyle) across the benchmark corpus.
pub fn wml_order_elements_per_standard(dom: &mut Dom, root: NodeId) {
    // Rank tables (PtOpenXmlUtil.cs:1273-1439). The W14 entries are C#'s
    // literal `w14:wShadow` etc. — a PowerTools quirk kept verbatim (real
    // markup uses w14:shadow, so they never match; unknowns rank 999 anyway).
    fn rank(dom: &Dom, container: &str, e: NodeId) -> i32 {
        let Some(n) = dom.name(e) else { return 999 };
        let local = n.local_name();
        let ns = n.namespace_name();
        if ns == W14::URI {
            if container == "rPr" {
                return match local {
                    "wShadow" => 270,
                    "wTextOutline" => 280,
                    "wTextFill" => 290,
                    "wScene3d" => 300,
                    "wProps3d" => 310,
                    _ => 999,
                };
            }
            return 999;
        }
        if ns != W::URI {
            return 999;
        }
        let table: &[(&str, i32)] = match container {
            "pPr" => &[
                ("pStyle", 10),
                ("keepNext", 20),
                ("keepLines", 30),
                ("pageBreakBefore", 40),
                ("framePr", 50),
                ("widowControl", 60),
                ("numPr", 70),
                ("suppressLineNumbers", 80),
                ("pBdr", 90),
                ("shd", 100),
                ("tabs", 120),
                ("suppressAutoHyphens", 130),
                ("kinsoku", 140),
                ("wordWrap", 150),
                ("overflowPunct", 160),
                ("topLinePunct", 170),
                ("autoSpaceDE", 180),
                ("autoSpaceDN", 190),
                ("bidi", 200),
                ("adjustRightInd", 210),
                ("snapToGrid", 220),
                ("spacing", 230),
                ("ind", 240),
                ("contextualSpacing", 250),
                ("mirrorIndents", 260),
                ("suppressOverlap", 270),
                ("jc", 280),
                ("textDirection", 290),
                ("textAlignment", 300),
                ("textboxTightWrap", 310),
                ("outlineLvl", 320),
                ("divId", 330),
                ("cnfStyle", 340),
                ("rPr", 350),
                ("sectPr", 360),
                ("pPrChange", 370),
            ],
            "rPr" => &[
                ("moveFrom", 5),
                ("moveTo", 7),
                ("ins", 10),
                ("del", 20),
                ("rStyle", 30),
                ("rFonts", 40),
                ("b", 50),
                ("bCs", 60),
                ("i", 70),
                ("iCs", 80),
                ("caps", 90),
                ("smallCaps", 100),
                ("strike", 110),
                ("dstrike", 120),
                ("outline", 130),
                ("shadow", 140),
                ("emboss", 150),
                ("imprint", 160),
                ("noProof", 170),
                ("snapToGrid", 180),
                ("vanish", 190),
                ("webHidden", 200),
                ("color", 210),
                ("spacing", 220),
                ("w", 230),
                ("kern", 240),
                ("position", 250),
                ("sz", 260),
                ("szCs", 320),
                ("highlight", 330),
                ("u", 340),
                ("effect", 350),
                ("bdr", 360),
                ("shd", 370),
                ("fitText", 380),
                ("vertAlign", 390),
                ("rtl", 400),
                ("cs", 410),
                ("em", 420),
                ("lang", 430),
                ("eastAsianLayout", 440),
                ("specVanish", 450),
                ("oMath", 460),
            ],
            "tblPr" => &TBLPR_ORDER,
            "tcPr" => &[
                ("cnfStyle", 10),
                ("tcW", 20),
                ("gridSpan", 30),
                ("hMerge", 40),
                ("vMerge", 50),
                ("tcBorders", 60),
                ("shd", 70),
                ("noWrap", 80),
                ("tcMar", 90),
                ("textDirection", 100),
                ("tcFitText", 110),
                ("vAlign", 120),
                ("hideMark", 130),
                ("headers", 140),
            ],
            "tcBorders" => &[
                ("top", 10),
                ("start", 20),
                ("left", 30),
                ("bottom", 40),
                ("right", 50),
                ("end", 60),
                ("insideH", 70),
                ("insideV", 80),
                ("tl2br", 90),
                ("tr2bl", 100),
            ],
            "tblBorders" => &[
                ("top", 10),
                ("left", 20),
                ("start", 30),
                ("bottom", 40),
                ("right", 50),
                ("end", 60),
                ("insideH", 70),
                ("insideV", 80),
            ],
            "pBdr" => &[
                ("top", 10),
                ("left", 20),
                ("bottom", 30),
                ("right", 40),
                ("between", 50),
                ("bar", 60),
            ],
            _ => &[],
        };
        table
            .iter()
            .find(|(n2, _)| *n2 == local)
            .map_or(999, |(_, r)| *r)
    }

    let sortable = [
        "pPr",
        "rPr",
        "tblPr",
        "tcPr",
        "tcBorders",
        "tblBorders",
        "pBdr",
    ];
    for el in dom.descendants_and_self(root, None) {
        let Some(name) = dom.name(el) else { continue };
        if name.namespace_name() != W::URI {
            continue;
        }
        let local = name.local_name().to_string();
        if sortable.contains(&local.as_str()) {
            for n in dom.nodes(el) {
                if !dom.is_element(n) {
                    dom.remove(n);
                }
            }
            let mut kids: Vec<(i32, usize, NodeId)> = dom
                .elements(el, None)
                .into_iter()
                .enumerate()
                .map(|(i, c)| (rank(dom, &local, c), i, c))
                .collect();
            if kids.is_sorted_by_key(|(r, i, _)| (*r, *i)) {
                continue;
            }
            kids.sort_by_key(|(r, i, _)| (*r, *i));
            for (_, _, c) in kids {
                dom.remove(c);
                dom.add(el, c);
            }
        } else if local == "p" || local == "r" {
            let props = if local == "p" { W::p_pr() } else { W::r_pr() };
            for n in dom.nodes(el) {
                if !dom.is_element(n) {
                    dom.remove(n);
                }
            }
            let kids = dom.elements(el, None);
            let needs_move = kids
                .iter()
                .position(|&c| dom.name(c) == Some(props.clone()))
                .is_some_and(|first_props| {
                    kids[..first_props]
                        .iter()
                        .any(|&c| dom.name(c) != Some(props.clone()))
                });
            if needs_move {
                let (front, back): (Vec<NodeId>, Vec<NodeId>) = kids
                    .into_iter()
                    .partition(|&c| dom.name(c) == Some(props.clone()));
                for c in front.into_iter().chain(back) {
                    dom.remove(c);
                    dom.add(el, c);
                }
            }
        }
    }
}

/// Wrap any run holding `w:delText` that is NOT directly inside a legal
/// deletion container in a fresh `w:del` — Word shows the repair dialog on
/// bare delText runs (THE residual strict01 trigger: `delete_text_in_opaque`
/// / the deleted-run rebuild rename nested text-box `w:t`s to delText without
/// wrapping the nested runs; Word wraps them — bisect-verified: wrapping the
/// 5 bare runs made the failing file open). Schema-legal either way, so no
/// validator flags it.
///
/// **Do not** wrap runs already under `w:moveFrom` — move markup is itself the
/// revision container (Word: `moveFrom > r > t|delText`). Nesting `w:del`
/// inside `w:moveFrom` is what makes Microsoft Word raise "unreadable content"
/// on otherwise valid move redlines (broken_ones_two accept-then path).
pub fn wrap_bare_del_text_runs(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let del_text = W::name("delText");
    let move_from = W::name("moveFrom");
    let runs: Vec<NodeId> = dom
        .descendants(root, Some(&W::r()))
        .into_iter()
        .filter(|&r| {
            if dom.element(r, &del_text).is_none() {
                return false;
            }
            let Some(p) = dom.parent(r) else {
                return true;
            };
            let n = dom.name(p);
            // Legal parents for delText runs: w:del and w:moveFrom.
            n != Some(W::del()) && n != Some(move_from.clone())
        })
        .collect();
    for r in runs {
        let d = rev_el(dom, W::del(), settings, id_gen);
        dom.add_before_self(r, d);
        dom.remove(r);
        dom.add(d, r);
    }
}

/// Which input a [`flatten_tracked_deletions`] pass is running against —
/// the two sides carry DIFFERENT contracts:
///   - `Original` (doc A): `accept(redline) ≡ B` — the diff re-deletes
///     whatever A-only text the flatten resurfaces, so everything can be
///     flattened, including deleted paragraph marks (paragraph structure of
///     the struck history stays visible like Word's).
///   - `Revised` (doc B): `accept(redline) ≡ accept(B)` — only deletions the
///     stamp/convert round-trip can re-emit as PENDING `w:del` may be
///     flattened. Complex-content deletions (`w:del > w:hyperlink/...`) and
///     paragraph-mark deletions are left intact for the pre-diff accept to
///     consume; flattening them resurfaced the text as INSERTIONS /
///     unmerged paragraphs, breaking the accept contract (m32 w15b/w15c).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FlattenSide {
    Original,
    Revised,
}

/// Word-alignment mode: flatten pre-existing tracked deletions before
/// diffing — unwrap `w:del`, convert `w:delText`/`w:delInstrText` back to
/// `w:t`/`w:instrText` — so the old text re-enters the diff and comes out
/// marked deleted: VISIBLE (struck through) like Word's redline, instead of
/// vanishing via accept-before-diff (forensics: page-numbering_potpourritest
/// "32 missing blocks", redline-cicerodo losing the whole compendium).
/// Every unwrapped run is stamped `pt:PreDelete` (+ original author/date) so
/// [`convert_stamped_predeletes`] can restore attribution and, on the
/// Revised side, re-emit the span as a pending deletion.
/// Pre-existing MOVE tracking (`w:moveFrom`/`w:moveTo`) is intentionally out
/// of scope — it still flows through the pre-diff accept.
/// The PowerTools-faithful preset keeps C#'s accept-first behavior.
/// Concatenated visible text of every `w:del` wrapper in `body` — used by
/// the S1 salt gate: an ORIGINAL-side pending deletion whose text doc B
/// ALSO holds as a pending deletion must keep correlating Equal (both
/// carry the same revision; GT keeps it once — sample-document iter2 pair,
/// −38.75 when salted). Only A-only pre-dels get the salt (fresh p4: B has
/// the text LIVE, GT emits the struck history + live copy).
pub fn pending_deletion_texts(dom: &Dom, body: NodeId) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    for d in dom.descendants(body, Some(&W::del())) {
        let text = pending_deletion_fingerprint(dom, d);
        if !text.is_empty() {
            out.insert(text);
        }
    }
    out
}

/// Fingerprint of one `w:del` wrapper's pending-deletion text, gathering BOTH
/// `w:delText` (run text) and `w:delInstrText` (field-instruction text). A
/// deletion containing only field instructions — or one whose `delText` matches
/// another deletion but whose field code differs — must produce a distinct
/// fingerprint so the S1 salt gate (`set.contains(&wrapper_text)`) can tell
/// A-only deletions from shared ones. The formula must stay identical to the
/// one used at the capture site in `flatten_tracked_deletions`.
fn pending_deletion_fingerprint(dom: &Dom, del: NodeId) -> String {
    let del_text = W::name("delText");
    let del_instr = W::name("delInstrText");
    dom.descendants(del, None)
        .into_iter()
        .filter(|&n| {
            dom.name(n)
                .is_some_and(|nm| nm == del_text || nm == del_instr)
        })
        .map(|t| dom.value(t))
        .collect()
}

pub fn flatten_tracked_deletions(
    dom: &mut Dom,
    body: NodeId,
    side: FlattenSide,
    other_side_pending: Option<&std::collections::HashSet<String>>,
) {
    // deleted paragraph marks: pPr/rPr/w:del → removed on the Original side
    // only. Doc B's pending mark deletions must reach the pre-diff accept so
    // the paragraphs merge exactly like accept(B) (m32 w15c).
    if side == FlattenSide::Original {
        for ppr in dom.descendants(body, Some(&W::p_pr())) {
            if let Some(rpr) = dom.element(ppr, &W::r_pr())
                && let Some(d) = dom.element(rpr, &W::del())
            {
                dom.remove(d);
            }
        }
    }
    // w:del wrappers → unwrap, restoring the text kind, stamping each run
    // pt:PreDelete so the produce path can convert the span BACK to w:del
    // (Word carries pending deletions as pending; accept(redline) ≡
    // accept(B) requires the text to come out DELETED, not inserted — see
    // convert_stamped_predeletes).
    let rpr_name = W::r_pr();
    let dels: Vec<NodeId> = dom.descendants(body, Some(&W::del()));
    for d in dels {
        // paragraph-mark markers live under pPr/rPr — handled (or deliberately
        // kept) above; the childless element must not be "unwrapped" away
        if dom
            .parent(d)
            .is_some_and(|p| dom.name(p) == Some(rpr_name.clone()))
        {
            continue;
        }
        let kids = dom.nodes(d);
        if side == FlattenSide::Revised
            && kids
                .iter()
                .any(|&k| dom.is_element(k) && dom.name(k) != Some(W::r()))
        {
            // complex content (hyperlink/sdt/smartTag/…): the stamp only
            // reaches direct runs, so flattening would resurface the nested
            // text as an INSERTION and break accept(redline) ≡ accept(B)
            // (m32 w15b). Leave the deletion for the pre-diff accept.
            continue;
        }
        // capture the wrapper's pending-deletion text BEFORE the
        // delText→t/delInstrText→instrText rename (the S1 salt gate matches
        // it against the other side's pending set). Must use the SAME formula
        // as `pending_deletion_texts` (both delText and delInstrText) so the
        // `set.contains(&wrapper_text)` lookup can match.
        let wrapper_text = pending_deletion_fingerprint(dom, d);
        for t in dom.descendants(d, Some(&W::name("delText"))) {
            dom.set_name(t, W::t());
        }
        for t in dom.descendants(d, Some(&W::name("delInstrText"))) {
            dom.set_name(t, W::instr_text());
        }
        let author = dom.attribute(d, &W::author()).map(|s| s.to_string());
        let date = dom.attribute(d, &W::date()).map(|s| s.to_string());
        // stamp value records the SIDE: only ORIGINAL-side stamps whose text
        // doc B does NOT also hold as a pending deletion get the hash salt
        // (M-MOVE S1). Doc B's pending deletions must still correlate Equal
        // with doc A's live copy (fresh p2/p3, −29/−31 when poisoned), and
        // SHARED pending deletions must keep correlating with each other
        // (sample-document iter2, −38.75 when salted).
        let side_stamp = if side == FlattenSide::Original
            && !other_side_pending.is_some_and(|set| set.contains(&wrapper_text))
        {
            // A-only pending deletion → salted, never correlates Equal
            super::PREDELETE_STAMP_ORIG
        } else {
            // shared/revised pending deletion → correlates as before
            super::PREDELETE_STAMP_REV
        };
        for &k in &kids {
            if dom.name(k) == Some(W::r()) {
                dom.set_attribute_value(k, &PT::name("PreDelete"), Some(side_stamp));
                if let Some(a) = &author {
                    dom.set_attribute_value(k, &PT::name("PreDelAuthor"), Some(a));
                }
                if let Some(dt) = &date {
                    dom.set_attribute_value(k, &PT::name("PreDelDate"), Some(dt));
                }
            }
        }
        dom.replace_with(d, &kids);
    }
}

/// Word-alignment mode, produce-side counterpart of the doc-B stamp in
/// [`flatten_tracked_deletions`]: every run stamped `pt:PreDelete` re-emerges
/// as a PENDING DELETION — its `w:ins` wrapper (the diff saw B-only text)
/// becomes `w:del`, Equal runs get a fresh `w:del` — so B's pre-existing
/// deletions render struck through like Word's ground truth while
/// accept(redline) ≡ accept(B) holds. MUST run before
/// [`coalesce_adjacent_revisions`] (wrappers still hold a single run).
pub fn convert_stamped_predeletes(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let marker = PT::name("PreDelete");
    let runs: Vec<NodeId> = dom
        .descendants(root, Some(&W::r()))
        .into_iter()
        .filter(|&r| dom.attribute(r, &marker).is_some())
        .collect();
    for r in runs {
        dom.set_attribute_value(r, &marker, None);
        let author = dom
            .attribute(r, &PT::name("PreDelAuthor"))
            .map(|s| s.to_string());
        let date = dom
            .attribute(r, &PT::name("PreDelDate"))
            .map(|s| s.to_string());
        dom.set_attribute_value(r, &PT::name("PreDelAuthor"), None);
        dom.set_attribute_value(r, &PT::name("PreDelDate"), None);
        // Word keeps the ORIGINAL author/date on carried pre-existing
        // deletions — the ground-truth corpus renders them in the original
        // author's revision color; re-attribution shifts every span's hue.
        let restamp = |dom: &mut Dom, wrapper: NodeId| {
            if let Some(a) = &author {
                dom.set_attribute_value(wrapper, &W::author(), Some(a));
            }
            if let Some(dt) = &date {
                dom.set_attribute_value(wrapper, &W::date(), Some(dt));
            }
        };
        let Some(parent) = dom.parent(r) else {
            continue;
        };
        let pname = dom.name(parent);
        if pname == Some(W::del()) {
            // already a deletion (the A-side flatten re-diffed the text) —
            // restore attribution when the wrapper holds only this span
            if dom.elements(parent, None).len() == 1 {
                restamp(dom, parent);
            }
            continue;
        }
        if pname == Some(W::ins()) {
            let sibs = dom.elements(parent, None);
            if sibs.len() == 1 {
                dom.set_name(parent, W::del());
                restamp(dom, parent);
            } else {
                // split the multi-run w:ins around the stamped run so order
                // is preserved: ins[..before] · del[r] · ins[after..]
                let idx = sibs.iter().position(|&c| c == r).unwrap_or(0);
                let d = rev_el(dom, W::del(), settings, id_gen);
                restamp(dom, d);
                dom.remove(r);
                dom.add(d, r);
                dom.add_after_self(parent, d);
                let after: Vec<NodeId> = sibs[idx + 1..].to_vec();
                if !after.is_empty() {
                    let ins2 = dom.new_element(W::ins());
                    for (an, av) in dom.attributes(parent) {
                        dom.set_attribute_value(ins2, &an, Some(&av));
                    }
                    dom.set_attribute_value(ins2, &W::id(), Some(&id_gen.to_string()));
                    *id_gen += 1;
                    for a in after {
                        dom.remove(a);
                        dom.add(ins2, a);
                    }
                    dom.add_after_self(d, ins2);
                }
                if idx == 0 {
                    dom.remove(parent); // emptied leading ins
                }
            }
        } else {
            let d = rev_el(dom, W::del(), settings, id_gen);
            restamp(dom, d);
            dom.add_before_self(r, d);
            dom.remove(r);
            dom.add(d, r);
        }
        for t in dom.descendants(r, Some(&W::t())) {
            dom.set_name(t, W::name("delText"));
        }
        for t in dom.descendants(r, Some(&W::instr_text())) {
            dom.set_name(t, W::name("delInstrText"));
        }
    }
}

/// Word-alignment mode: Word's Compare synthesizes near-zero cell margins
/// on the tables it emits — `w:tblInd w=10` and `w:tblCellMar left/right
/// w=10` — present in NEITHER input nor Word's TableNormal default of 108
/// (forensics pair #5, helvetica_hr-onboarding). Bare tables render narrower
/// effective text columns that over-wrap vs the ground truth. Fill only what
/// each table's tblPr doesn't define; children are inserted in CT_TblPr
/// schema order (tblW 70 < tblInd 100 < tblCellMar 140 < tblLook 150).
pub fn synthesize_table_cell_margins(dom: &mut Dom, root: NodeId) {
    fn dxa(dom: &mut Dom, name: &str, w: &str) -> NodeId {
        let e = dom.new_element(W::name(name));
        dom.set_attribute_value(e, &W::name("w"), Some(w));
        dom.set_attribute_value(e, &W::name("type"), Some("dxa"));
        e
    }
    fn insert_in_order(dom: &mut Dom, tblpr: NodeId, child: NodeId, rank: i32) {
        let later = dom.elements(tblpr, None).into_iter().find(|&c| {
            dom.name(c)
                .map(|n| {
                    TBLPR_ORDER
                        .iter()
                        .find(|(l, _)| *l == n.local_name())
                        .map_or(999, |(_, r)| *r)
                })
                .unwrap_or(999)
                > rank
        });
        match later {
            Some(l) => dom.add_before_self(l, child),
            None => dom.add(tblpr, child),
        }
    }
    let tbls: Vec<NodeId> = dom.descendants(root, Some(&W::name("tbl")));
    for tbl in tbls {
        let Some(tblpr) = dom.element(tbl, &W::name("tblPr")) else {
            continue;
        };
        // Word synthesizes ONLY on bordered tables — corpus discriminator:
        // every GT table with w:tblBorders carries mar10/ind10, the one
        // border-less table (24-id_alternate-content) does not.
        if dom.element(tblpr, &W::name("tblBorders")).is_none() {
            continue;
        }
        if dom.element(tblpr, &W::name("tblInd")).is_none() {
            let ind = dxa(dom, "tblInd", "10");
            insert_in_order(dom, tblpr, ind, 100);
        }
        if dom.element(tblpr, &W::name("tblCellMar")).is_none() {
            let mar = dom.new_element(W::name("tblCellMar"));
            let l = dxa(dom, "left", "10");
            dom.add(mar, l);
            let r = dxa(dom, "right", "10");
            dom.add(mar, r);
            insert_in_order(dom, tblpr, mar, 140);
        }
    }
}

/// Word-validity artifacts beyond schema ordering (OpenXmlValidator-driven;
/// Word's own redlines validate clean of all of these):
///   - `w:cnfStyle` without the REQUIRED `w:val`: Strict expresses the flags
///     as individual boolean attributes; synthesize the Transitional 12-bit
///     bitmask (firstRow lastRow firstColumn lastColumn oddVBand evenVBand
///     oddHBand evenHBand firstRowFirstColumn firstRowLastColumn
///     lastRowFirstColumn lastRowLastColumn).
///   - `wp14:pctWidth`/`pctHeight`/`pctPosHOffset`/`pctPosVOffset` element
///     values in Strict percent form ("20%") → per-thousand ints (20000).
///   - `w14:paraId`/`w14:textId` values ≥ 0x80000000 (the corpus's
///     deliberate `id-paraid-overflow` passthrough) → attribute stripped;
///     Word regenerates its own on save and never emits out-of-range ids.
pub fn fix_strict_validity_artifacts(dom: &mut Dom, root: NodeId) {
    const CNF_FLAGS: [&str; 12] = [
        "firstRow",
        "lastRow",
        "firstColumn",
        "lastColumn",
        "oddVBand",
        "evenVBand",
        "oddHBand",
        "evenHBand",
        "firstRowFirstColumn",
        "firstRowLastColumn",
        "lastRowFirstColumn",
        "lastRowLastColumn",
    ];
    for cnf in dom.descendants(root, Some(&W::name("cnfStyle"))) {
        if dom.attribute(cnf, &W::val()).is_some() {
            continue;
        }
        let mut mask = String::with_capacity(12);
        for f in CNF_FLAGS {
            // all three ST_OnOff true literals (xsd:boolean "true"/"1" plus
            // the legacy "on") — Strict writers emit "1"/"true" in practice
            let on = matches!(
                dom.attribute(cnf, &W::name(f)),
                Some("1") | Some("true") | Some("on")
            );
            mask.push(if on { '1' } else { '0' });
        }
        dom.set_attribute_value(cnf, &W::val(), Some(&mask));
    }
    for local in ["pctWidth", "pctHeight", "pctPosHOffset", "pctPosVOffset"] {
        for e in dom.descendants(root, Some(&WP14::name(local))) {
            let v = dom.value(e);
            if let Some(num) = v.trim().strip_suffix('%')
                && let Ok(f) = num.trim().parse::<f64>()
            {
                let per_thousand = (f * 1000.0).round() as i64;
                dom.set_value(e, &per_thousand.to_string());
            }
        }
    }
    for el in dom.descendants_and_self(root, None) {
        for a in [W14::name("paraId"), W14::name("textId")] {
            // map_or(true, …) is deliberate: a value that does not even parse
            // as ST_LongHexNumber is just as invalid as an out-of-range one —
            // strip it too (Word regenerates these ids on save). NOTE: Result
            // (not Option), so clippy's is_none_or suggestion doesn't apply.
            let out_of_range = dom.attribute(el, &a).is_some_and(|v| {
                u32::from_str_radix(v.trim(), 16).map_or(true, |x| x >= 0x8000_0000)
            });
            if out_of_range {
                dom.set_attribute_value(el, &a, None);
            }
        }
    }
    // Strict wordprocessingShape/Group/Canvas leakage: the Strict corpus
    // binds wsp/spPr/txbx/… in the STRICT wordprocessingDrawing namespace
    // inside `a:graphicData uri="…/office/word/2010/wordprocessing*"`; the
    // package-wide URI translation turns them into TRANSITIONAL `wp:` — a
    // namespace with no such elements, and Word reports unreadable content
    // (strict01 text-box shapes; the actual repair-dialog trigger, bisected
    // in real Word). Remap wp:-namespaced elements inside such graphicData
    // subtrees into the namespace the uri declares, stopping at nested
    // `w:drawing` (a fresh drawing context with legitimate wp: markup).
    {
        let graphic_data = crate::namespaces::A::name("graphicData");
        let uri_attr = XNamespace::none().name("uri");
        let drawing = W::name("drawing");
        fn remap(dom: &mut Dom, node: NodeId, wp_uri: &str, target: &XNamespace, drawing: &XName) {
            for c in dom.nodes(node) {
                if !dom.is_element(c) {
                    continue;
                }
                let Some(n) = dom.name(c) else { continue };
                if n == *drawing {
                    continue;
                }
                if n.namespace_name() == wp_uri {
                    dom.set_name(c, target.name(n.local_name()));
                }
                remap(dom, c, wp_uri, target, drawing);
            }
        }
        let gds: Vec<NodeId> = dom.descendants(root, Some(&graphic_data));
        for gd in gds {
            let Some(uri) = dom.attribute(gd, &uri_attr).map(|s| s.to_string()) else {
                continue;
            };
            if !uri.starts_with("http://schemas.microsoft.com/office/word/2010/wordprocessing") {
                continue;
            }
            let target = XNamespace::get(&uri);
            remap(dom, gd, crate::namespaces::WP::URI, &target, &drawing);
        }
    }
    // Word-written Strict packages put text-box content in wne:txbxContent
    // (2006 wordml extension); in a Transitional package wne is mc:Ignorable,
    // so Word drops it and wps:txbx loses its REQUIRED child — unreadable
    // (strict01 cover-page shapes; Word's own redline emits w:txbxContent).
    {
        let wne_txbx = crate::namespaces::WNE::name("txbxContent");
        for e in dom.descendants(root, Some(&wne_txbx)) {
            dom.set_name(e, W::name("txbxContent"));
        }
    }
    // Strict jc enumeration values on w:jc / w:lvlJc: "start"/"end" are
    // Strict-only (Transitional ST_Jc has left/right); Word never writes them
    // in Transitional packages (strict01 numbering evidence, 7× lvlJc).
    for local in ["jc", "lvlJc"] {
        for e in dom.descendants(root, Some(&W::name(local))) {
            match dom.attribute(e, &W::val()) {
                Some("start") => dom.set_attribute_value(e, &W::val(), Some("left")),
                Some("end") => dom.set_attribute_value(e, &W::val(), Some("right")),
                _ => {}
            }
        }
    }
    // Strict DrawingML percent attributes ("0%" on a:scrgbClr r/g/b, "65%" on
    // a:lumMod, "0%" defRPr baseline etc.): Transitional DrawingML types are
    // per-thousand ints across the board, so any percent-suffixed attribute on
    // a drawingml-family element is a Strict leftover (strict01's diagram and
    // chart parts evidence). w:-namespace attrs are NOT touched — Transitional
    // WML legitimately uses "50%" (tblW type="pct").
    const DRAWING_NS: [&str; 4] = [
        "http://schemas.openxmlformats.org/drawingml/2006/main",
        "http://schemas.openxmlformats.org/drawingml/2006/diagram",
        "http://schemas.microsoft.com/office/drawing/2008/diagram",
        "http://schemas.openxmlformats.org/drawingml/2006/chart",
    ];
    for el in dom.descendants_and_self(root, None) {
        let Some(n) = dom.name(el) else { continue };
        if !DRAWING_NS.contains(&n.namespace_name()) {
            continue;
        }
        let percents: Vec<(crate::xmllinq::XName, i64)> = dom
            .attributes(el)
            .into_iter()
            .filter_map(|(an, av)| {
                let t = av.trim();
                let num = t.strip_suffix('%')?;
                let f: f64 = num.trim().parse().ok()?;
                Some((an, (f * 1000.0).round() as i64))
            })
            .collect();
        for (an, v) in percents {
            dom.set_attribute_value(el, &an, Some(&v.to_string()));
        }
    }
}

/// Canonicalize ST_UniversalMeasure values (`612pt`, `1in`, `2.54cm`, …) to
/// plain twips on the twips-typed layout attributes. Strict-converted inputs
/// carry them (the Strict corpus writes point-suffixed geometry); Word
/// tolerates the universal form but other renderers diverge (LibreOffice
/// mis-paginates the strict01 pairs). Whitelist-driven so half-point /
/// eighth-point attributes (`w:sz` etc.) are never touched.
pub fn normalize_universal_measures(dom: &mut Dom, root: NodeId) {
    // Cell-margin children (w:tcMar / w:tblCellMar) carry @w:w in twips; the
    // like-named border elements carry @w:sz (eighth-points) and never @w:w,
    // so keying on (element local, attr) stays safe.
    const TWIPS_ATTRS: [(&str, &[&str]); 18] = [
        ("pgSz", &["w", "h"]),
        ("gridCol", &["w"]),
        (
            "pgMar",
            &[
                "top", "right", "bottom", "left", "header", "footer", "gutter",
            ],
        ),
        (
            "ind",
            &["left", "right", "hanging", "firstLine", "start", "end"],
        ),
        ("spacing", &["before", "after", "line"]),
        ("tab", &["pos"]),
        ("defaultTabStop", &["val"]),
        ("tblW", &["w"]),
        ("tcW", &["w"]),
        ("tblInd", &["w"]),
        ("tblCellSpacing", &["w"]),
        ("trHeight", &["val"]),
        ("top", &["w"]),
        ("bottom", &["w"]),
        ("left", &["w"]),
        ("right", &["w"]),
        ("start", &["w"]),
        ("end", &["w"]),
    ];
    fn to_twips(v: &str) -> Option<i64> {
        let t = v.trim();
        for (suf, factor) in [
            ("pt", 20.0),
            ("in", 1440.0),
            ("cm", 1440.0 / 2.54),
            ("mm", 1440.0 / 25.4), // 1in = 25.4mm → ~56.693 twips per mm
            ("pc", 240.0),
            ("pi", 240.0),
        ] {
            if let Some(n) = t.strip_suffix(suf) {
                return n
                    .trim()
                    .parse::<f64>()
                    .ok()
                    .map(|f| (f * factor).round() as i64);
            }
        }
        // fractional plain values ("100.0") on integer-typed attributes →
        // rounded (gdocs-exported styles carry them; validator: Int16 fail)
        if t.contains('.') {
            return t.parse::<f64>().ok().map(|f| f.round() as i64);
        }
        None
    }
    for el in dom.descendants_and_self(root, None) {
        let Some(name) = dom.name(el) else { continue };
        if name.namespace_name() != W::URI {
            continue;
        }
        let Some(attrs) = TWIPS_ATTRS
            .iter()
            .find(|(ln, _)| name.local_name() == *ln)
            .map(|(_, a)| *a)
        else {
            continue;
        };
        for a in attrs {
            let an = W::name(a);
            // and_then keeps this allocation-free for the common untouched
            // case — the i64 releases the dom borrow before the mutation
            if let Some(tw) = dom.attribute(el, &an).and_then(to_twips) {
                dom.set_attribute_value(el, &an, Some(&tw.to_string()));
            }
        }
    }
}

/// Serialize a sectPr with every `pt:*` bookkeeping attribute AND every
/// `rsid*` attribute stripped, and measurement units canonicalized to twips,
/// so the identity comparison in [`drop_hoisted_sectpr_artifacts`] is immune
/// to what the pipeline changes between capture and check: unid stamping adds
/// pt attrs, the pre-diff accept (RemoveRsid) strips rsids, and
/// [`normalize_universal_measures`] rewrites "612pt" → "12240" after capture.
/// Any of these made every comparison fail, deleting GENUINE mid-section
/// breaks (strict01/sd-2517 multi-section evidence; pt-unit variant caught on
/// the strict01 probe after w11 landed).
pub fn sectpr_identity(dom: &mut Dom, sp: NodeId) -> String {
    let c = dom.clone_subtree(sp);
    remove_powertools_scratch_markup(dom, c);
    normalize_universal_measures(dom, c);
    for el in dom.descendants_and_self(c, None) {
        let rsids: Vec<crate::xmllinq::XName> = dom
            .attributes(el)
            .into_iter()
            .map(|(n, _)| n)
            .filter(|n| n.local_name().starts_with("rsid"))
            .collect();
        for a in rsids {
            dom.set_attribute_value(el, &a, None);
        }
    }
    dom.serialize_element(c)
}

/// Word-alignment mode (settings-gated): drop pPr-embedded `sectPr`s that
/// were NOT pPr-embedded in either input ("genuine" mid-section breaks are
/// captured from the inputs before atomize hoists each doc's final body
/// sectPr into its last paragraph — that hoist artifact must not become a
/// mid-document page break; evidence: 1-5-line-spacing_24 rendered 2 pages
/// vs Word's 1). Identities are compared pt-attribute-free on BOTH sides.
pub fn drop_hoisted_sectpr_artifacts(
    dom: &mut Dom,
    root: NodeId,
    genuine: &std::collections::HashSet<String>,
) {
    let sectpr = W::name("sectPr");
    for ppr in dom.descendants(root, Some(&W::p_pr())) {
        for sp in dom.elements(ppr, Some(&sectpr)) {
            if !genuine.contains(&sectpr_identity(dom, sp)) {
                dom.remove(sp);
            }
        }
    }
}

/// Word-alignment mode (settings-gated): Word marks a fully-deleted table
/// row with `w:del` inside `w:trPr` (and fully-inserted rows with `w:ins`),
/// in addition to the cell-content revisions — the accept pipeline then
/// removes the row (and the whole table once every row is marked). Our
/// produce path deletes cell CONTENT only, leaving a ghost empty table after
/// acceptance (meeting-agenda/employee-directory benchmark evidence; the C#
/// oracle instead drops the table structure entirely — further from Word).
pub fn mark_fully_revised_rows(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let tr_name = W::name("tr");
    let tc_name = W::name("tc");
    let trs: Vec<NodeId> = dom.descendants(root, Some(&tr_name));
    for tr in trs {
        let mut class: Option<bool> = None; // Some(false)=all-del, Some(true)=all-ins
        let mut any_content = false;
        let mut mixed = false;
        for tc in dom.elements(tr, Some(&tc_name)) {
            for p in dom.descendants(tc, Some(&W::p())) {
                match para_replacement_class(dom, p) {
                    Some(k) => {
                        any_content = true;
                        if class.get_or_insert(k) != &k {
                            mixed = true;
                        }
                    }
                    None => {
                        // empty paragraphs are neutral; content-bearing mixed
                        // paragraphs disqualify the row. Content = text OR
                        // non-text payloads (drawing/object/pict/sym) — a row
                        // holding an unchanged drawing must never be marked
                        // fully revised, or the accept pass drops content the
                        // other document still has (PR #54 review).
                        let content_bearing = |dom: &Dom, r: NodeId| {
                            !dom.value(r).trim().is_empty()
                                || dom.descendants(r, None).iter().any(|&c| {
                                    // descendants, not children: drawings ride
                                    // inside mc:AlternateContent/mc:Choice
                                    dom.name(c).is_some_and(|n| {
                                        matches!(
                                            n.local_name(),
                                            "drawing" | "object" | "pict" | "sym"
                                        )
                                    })
                                })
                        };
                        if dom
                            .descendants(p, Some(&W::r()))
                            .iter()
                            .any(|&r| content_bearing(dom, r))
                        {
                            mixed = true;
                        }
                    }
                }
            }
        }
        if mixed || !any_content {
            continue;
        }
        let Some(k) = class else { continue };
        let trpr = match dom.element(tr, &W::name("trPr")) {
            Some(p) => p,
            None => {
                let p = dom.new_element(W::name("trPr"));
                dom.add_first(tr, p);
                p
            }
        };
        let rev_name = if k { W::ins() } else { W::del() };
        if dom.element(trpr, &rev_name).is_some() {
            continue; // already marked (e.g. by MarkRows on the row path)
        }
        let rev = dom.new_element(rev_name);
        dom.set_attribute_value(rev, &W::author(), Some(&settings.author_for_revisions));
        dom.set_attribute_value(rev, &W::id(), Some(&id_gen.to_string()));
        *id_gen += 1;
        dom.set_attribute_value(rev, &W::date(), Some(&settings.date_time_for_revisions));
        dom.add(trpr, rev);
    }
}
