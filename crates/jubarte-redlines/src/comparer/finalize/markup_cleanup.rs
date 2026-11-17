// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Markup cleanup: run coalescing, alternate content, content controls, property order, hyperlink unwrapping.

use super::*;

// ── CoalesceAdjacentRunsWithIdenticalFormatting (PtOpenXmlUtil.ts:4458) ───────
// Merges adjacent single-w:t runs with identical rPr into one run. Used both in
// PreProcessMarkup (on the inputs, so source fragmentation doesn't inflate the
// diff) and in the produce finalization (merges adjacent w:del; w:ins keep
// distinct ids so don't merge).

pub(super) const DONT_CONSOLIDATE: &str = "DontConsolidate";

pub(super) fn xml_space_attr(text: &str) -> Option<&'static str> {
    match (text.chars().next(), text.chars().last()) {
        (Some(f), _) if f.is_whitespace() => Some("preserve"),
        (_, Some(l)) if l.is_whitespace() => Some("preserve"),
        _ => None,
    }
}

pub(super) fn rpr_string(dom: &Dom, r: NodeId) -> String {
    match dom.element(r, &W::r_pr()) {
        Some(rpr) => dom.serialize_element(rpr),
        None => String::new(),
    }
}

pub(super) fn coalesce_key(dom: &Dom, ce: NodeId) -> String {
    let Some(name) = dom.name(ce) else {
        return DONT_CONSOLIDATE.to_string();
    };
    if name == W::r() {
        let non_rpr = dom
            .elements(ce, None)
            .into_iter()
            .filter(|&e| !dom.name_is(e, &W::r_pr()))
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
            .filter(|&e| !dom.name_is(e, &W::r_pr()))
            .count();
        let has_del_text = dom
            .elements(ce, None)
            .into_iter()
            .any(|c| dom.element(c, &W::del_text()).is_some());
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

pub(super) fn run_text_concat(dom: &Dom, r: NodeId) -> String {
    let mut s = String::new();
    for d in dom.descendants(r, None) {
        let n = dom.name(d).unwrap();
        if n == W::t() || n == W::del_text() || n == W::instr_text() {
            s.push_str(&dom.value_str(d));
        }
    }
    s
}

/// Merge adjacent `w:r` and `w:del` runs of identical text + formatting
/// into one — port of `CoalesceAdjacentRunsWithIdenticalFormatting`
/// (PtOpenXmlUtil.ts:4458).
///
/// A run holding only `w:t` (or `w:instrText`) whose `w:rPr` matches the
/// previous run's is merged by appending its text and removing the
/// wrapper. Adjacent `w:del` wrappers with one `w:delText` child also
/// merge when their author/date match; run- and `w:del`-level stamps
/// (`PreIns` / `PreDelete` / `*Author` / `*Date`) segregate the key so
/// pending-deletion runs are not silently merged away. Returns the
/// rebuilt container; caller replaces the original via assignment.
///
/// Used both during preprocessing (on the inputs, so source fragmentation
/// does not inflate the diff) and as the final produce-side coalescer
/// that merges adjacent `w:del`. `w:ins` runs keep distinct ids and are
/// not merged.
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
            let dt = dom.new_element(W::del_text());
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

/// Resolve feature-gating `mc:AlternateContent` against the
/// MCE rule (ISO/IEC 29500-3 §10.1.2).
///
/// Word's Compare RESOLVES feature-gating `mc:AlternateContent`
/// (replaces it with its `mc:Choice` content) but KEEPS drawing/VML
/// fallbacks (DrawingML `Choice` + VML `Fallback`). Opaque run-children
/// (`mc:AlternateContent`, `w:pict`, `w:object`) atomize as a single
/// unit and so are kept verbatim. The MCE rule picks the first
/// `mc:Choice` whose `@Requires` namespaces Word understands
/// (`choice_requires_understood`), falling back to `mc:Fallback`.
///
/// Run on both inputs before diffing so an AltContent run-child on one
/// side does not become a "unreadable content" repair target.
pub fn resolve_alternate_content(dom: &mut Dom, root: NodeId) {
    resolve_alternate_content_with(dom, root, word_understands_namespace);
}

/// [`resolve_alternate_content`] for a consumer that understands the
/// namespaces `understood` accepts: the renderer cannot draw every Choice
/// Word can (#1876).
pub(crate) use crate::markup_simplifier::{
    resolve_alternate_content_with, selected_alternate_branch, word_understands_namespace,
};

/// Strip non-standard `w:`-namespace children from `<w:sdtPr>`.
///
/// Agreement/form tools emit `<w:fieldType>` / `<w:fieldColor>` / …
/// inside `sdtPr` in the `w:` namespace — not valid `CT_SdtPr` content;
/// real Word reports "unreadable content" and recovers by dropping
/// them. This pass matches so the redline is Word-valid.
/// Extension-namespace children (`w14:…` / `w15:…`) are untouched.
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
    for sdtpr in dom.descendants(root, Some(&W::sdt_pr())) {
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

/// Unwrap every `w:sdt` under `root` to its `sdtContent` children,
/// deepest first.
///
/// M390 (`missing_sectpr × fields_test`): Word Compare flattens content
/// controls (`w:sdt`) inside pure-I / pure-D residual paragraphs but
/// keeps equal-form-field controls on unchanged paragraphs. This is the
/// unconditional variant; prefer
/// [`unwrap_content_controls_in_pure_revisions`] after the produce /
/// merge step to scope the unwrap to residual paragraphs.
pub fn unwrap_content_controls(dom: &mut Dom, root: NodeId) {
    loop {
        let sdts: Vec<NodeId> = dom.descendants(root, Some(&W::sdt()));
        if sdts.is_empty() {
            break;
        }
        // Prefer leaf SDTs (no nested w:sdt) so nested controls unwrap safely.
        let mut leaf: Vec<NodeId> = sdts
            .iter()
            .copied()
            .filter(|&s| {
                dom.descendants(s, Some(&W::sdt()))
                    .into_iter()
                    .all(|n| n == s)
            })
            .collect();
        if leaf.is_empty() {
            // Nested only — force outermost (descendants list is pre-order;
            // take last = deepest-ish fallback).
            leaf = sdts;
        }
        let mut progressed = false;
        for sdt in leaf {
            if dom.parent(sdt).is_none() {
                continue;
            }
            let kids: Vec<NodeId> = if let Some(content) = dom.element(sdt, &W::sdt_content()) {
                dom.nodes(content)
            } else {
                Vec::new()
            };
            // Reparent content children before the sdt, then drop the wrapper.
            for k in kids {
                if dom.parent(k).is_some() {
                    dom.remove(k);
                    dom.add_before_self(sdt, k);
                }
            }
            dom.remove(sdt);
            progressed = true;
        }
        if !progressed {
            break;
        }
    }
}

/// M390 — [`unwrap_content_controls`], but only under pure-I / pure-D
/// / MIX residual paragraphs.
///
/// `EQ` form-field matches keep their `w:sdt` (the corpus document
/// `fields_attrs1 × sample` keeps four equal content controls). Run
/// after the produce / merge step so the unwrap sees the final
/// revision status on every paragraph.
pub fn unwrap_content_controls_in_pure_revisions(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let paras: Vec<NodeId> = dom.descendants(body, Some(&W::p()));
    for p in paras {
        let has_ins = !dom.descendants(p, Some(&W::ins())).is_empty()
            || para_mark_revision(dom, p, &W::ins());
        let has_del = !dom.descendants(p, Some(&W::del())).is_empty()
            || para_mark_revision(dom, p, &W::del());
        // Pure-I, pure-D, or MIX residual — not unmarked EQ.
        if !has_ins && !has_del {
            continue;
        }
        if dom.descendants(p, Some(&W::sdt())).is_empty() {
            continue;
        }
        unwrap_content_controls(dom, p);
    }
}

/// True when `node` sits in a Choice or Fallback that Word does not show.
#[doc(hidden)]
pub fn in_unselected_alternate(dom: &Dom, node: NodeId) -> bool {
    dom.ancestors(node, None).into_iter().any(|ancestor| {
        if !dom.name_is(ancestor, &MC::choice()) && !dom.name_is(ancestor, &MC::fallback()) {
            return false;
        }
        let Some(alternate) = dom.parent(ancestor) else {
            return false;
        };
        if !dom.name_is(alternate, &MC::alternate_content()) {
            return false;
        }
        selected_alternate_branch(dom, alternate, word_understands_namespace) != Some(ancestor)
    })
}

/// True if `prefix` is bound by an `xmlns:prefix` declaration on `node` or any of
/// its ancestors (XML namespace scoping).
pub(super) fn prefix_in_scope(dom: &Dom, node: NodeId, prefix: &str) -> bool {
    dom.ancestors_and_self(node, None).iter().any(|&anc| {
        dom.attributes_iter(anc)
            .any(|(name, _)| dom.is_namespace_declaration(name) && name.local_name() == prefix)
    })
}

/// Apply [`coalesce_adjacent_runs`] to every `w:p` under `root`.
///
/// Each paragraph's runs are merged per the gating rules in
/// [`coalesce_adjacent_runs`]; non-run children (`w:pPr`, etc.) stay
/// where they are. Mutates `root` in place; nothing to return.
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

/// Enforce the OOXML child order for paragraph-mark revision markers.
///
/// OOXML requires (a) `ins` / `del` / `moveFrom` / `moveTo` to be the
/// FIRST children of the paragraph's `w:rPr` (`CT_ParaRPr` sequence), and
/// (b) the `w:rPr` itself to follow the paragraph's content properties
/// (`pStyle`, …) and sit just before `w:sectPr` / `w:pPrChange` (or last
/// when neither is present). Re-assembly otherwise leaves the marker
/// after `w:lang` / … or places paraRPr before `pStyle`; ooxmlsdk
/// tolerates the mis-order, real Word rejects it as "unreadable content"
/// (contract-acc, hyperlink, NumberingImplicitNumId, nda).
pub fn fix_paragraph_mark_revision_order(dom: &mut Dom, root: NodeId) {
    let is_marker = |dom: &Dom, c: NodeId| {
        matches!(
            dom.name(c).as_ref().map(|n| n.local_name()),
            Some("ins") | Some("del") | Some("moveFrom") | Some("moveTo")
        )
    };
    for ppr in dom.descendants(root, Some(&W::p_pr())) {
        let Some(rpr) = dom.element(ppr, &W::r_pr()) else {
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
            .element(ppr, &W::sect_pr())
            .or_else(|| dom.element(ppr, &W::p_pr_change()));
        dom.remove(rpr);
        match anchor {
            Some(a) => dom.add_before_self(a, rpr),
            None => dom.add(ppr, rpr),
        }
    }
}

/// Move every paragraph's `w:pPr` to the FIRST child position (OOXML
/// schema requires `w:pPr` first under `w:p`).
///
/// Re-assembly places the paragraph mark (`w:pPr`) LAST; Word and
/// LibreOffice silently ignore every paragraph-formatting property in
/// that order (`w:jc`, spacing, indentation, numbering). This pass
/// recurses the tree under `node` and moves `pPr` to the front.
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
/// Hyperlink-styled runs; we kept 133 live hyperlinks). LO layout of
/// field anchors differs from Word's plain Hyperlink runs — unwrap for
/// Word-mode.
///
/// Only anchor-based internal hyperlinks (TOC/bookmark jumps, no `r:id`)
/// are unwrapped. An `r:id`-bearing hyperlink is a live external target:
/// dropping its wrapper here would discard the `r:id` before
/// `reconcile_dangling_relationships` runs, silently destroying the link
/// (m16/m29 regression).
pub fn unwrap_hyperlinks_to_styled_runs(dom: &mut Dom, root: NodeId) {
    let hyperlinks: Vec<NodeId> = dom
        .descendants(root, Some(&W::hyperlink()))
        .into_iter()
        .filter(|&hl| dom.attribute(hl, &R::id()).is_none())
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
            if dom.element(rpr, &W::r_style()).is_none() {
                let rs = dom.new_element(W::r_style());
                dom.set_attribute_value(rs, &W::val(), Some("Hyperlink"));
                dom.add_first(rpr, rs);
            }
        }
        // M470 — a WHOLLY deleted/inserted anchor hyperlink keeps Word's
        // FIELD form: [fldChar begin][(del)InstrText HYPERLINK \l "<anchor>"]
        // [fldChar separate] …content… [fldChar end] inside the revision.
        // Plain unwrapping left the inner PAGEREF field chars unbalanced and
        // LO mis-rendered TOC leaders/page numbers (file_22 × file_23 oracle,
        // 115pp vs 116pp). Live hyperlinks keep the file_21 styled-run unwrap.
        wrap_revised_hyperlink_as_field(dom, hl);
        // Hoist children before the hyperlink, then drop the wrapper.
        let kids: Vec<NodeId> = dom.nodes(hl);
        for k in kids {
            dom.remove(k);
            dom.add_before_self(hl, k);
        }
        dom.remove(hl);
    }
}

/// M470 helper — when `hl`'s text content is wholly deleted (or wholly
/// inserted), synthesize the HYPERLINK field wrapper runs inside the same
/// revision context. No-op for live or mixed hyperlinks.
pub(super) fn wrap_revised_hyperlink_as_field(dom: &mut Dom, hl: NodeId) {
    let Some(anchor) = dom.attribute(hl, &W::name("anchor")).map(str::to_string) else {
        return;
    };
    let has_live_t = dom.descendants(hl, Some(&W::t())).iter().any(|&t| {
        !dom.value_str(t).trim().is_empty()
            && !dom
                .ancestors_and_self(t, None)
                .iter()
                .any(|&a| dom.name_is(a, &W::ins()))
    });
    let has_del_text = !dom.descendants(hl, Some(&W::del_text())).is_empty();
    let has_ins = !dom.descendants(hl, Some(&W::ins())).is_empty();
    let parent_rev = dom
        .parent(hl)
        .filter(|&p| dom.name_is(p, &W::del()) || dom.name_is(p, &W::ins()));
    // Classify: deleted (delText, no live text), inserted (all under ins),
    // or the hyperlink itself sits inside a revision wrapper.
    let (is_del, is_ins) = if let Some(p) = parent_rev {
        (dom.name_is(p, &W::del()), dom.name_is(p, &W::ins()))
    } else if has_del_text && !has_live_t {
        (true, false)
    } else if has_ins && !has_live_t && !has_del_text {
        (false, true)
    } else {
        return; // live or mixed — plain unwrap
    };
    if !is_del && !is_ins {
        return;
    }
    let instr_name = if is_del {
        W::del_instr_text()
    } else {
        W::instr_text()
    };
    let mk_fld = |dom: &mut Dom, ty: &str| -> NodeId {
        let r = dom.new_element(W::r());
        let f = dom.new_element(W::fld_char());
        dom.set_attribute_value(f, &W::name("fldCharType"), Some(ty));
        dom.add(r, f);
        r
    };
    let begin = mk_fld(dom, "begin");
    let instr_r = dom.new_element(W::r());
    let instr = dom.new_element(instr_name);
    // M474: carry the tooltip switch — Word writes `\o "<tip>"` after the
    // anchor (sd_2672 × hyperlink_node_internal oracle).
    let mut instr_text = format!("HYPERLINK \\l \"{anchor}\"");
    if let Some(tip) = dom.attribute(hl, &W::name("tooltip")) {
        let tip = tip.to_string();
        instr_text.push_str(&format!(" \\o \"{tip}\""));
    }
    dom.add_text(instr, &instr_text);
    dom.add(instr_r, instr);
    let sep = mk_fld(dom, "separate");
    let end = mk_fld(dom, "end");

    if parent_rev.is_some() {
        // del(hl(runs)) — field runs become hl-siblings inside the wrapper;
        // the caller's unwrap then flattens hl between them.
        dom.add_before_self(hl, begin);
        dom.add_before_self(hl, instr_r);
        dom.add_before_self(hl, sep);
        dom.add_after_self(hl, end);
        return;
    }
    // hl(rev(runs)…) — prepend the trio into the first revision child and
    // append the end into the last one, keeping the field inside w:del/w:ins.
    let rev_name = if is_del { W::del() } else { W::ins() };
    let revs: Vec<NodeId> = dom.elements(hl, Some(&rev_name));
    if let (Some(&first), Some(&last)) = (revs.first(), revs.last()) {
        dom.add_first(first, sep);
        dom.add_first(first, instr_r);
        dom.add_first(first, begin);
        dom.add(last, end);
    } else {
        // delText runs directly under hl (no rev wrapper): field runs join
        // them bare; the enclosing paragraph mark carries the revision.
        match dom.elements(hl, None).first().copied() {
            Some(firstc) => {
                dom.add_before_self(firstc, sep);
                dom.add_before_self(firstc, instr_r);
                dom.add_before_self(firstc, begin);
            }
            None => {
                dom.add(hl, begin);
                dom.add(hl, instr_r);
                dom.add(hl, sep);
            }
        }
        dom.add(hl, end);
    }
}
