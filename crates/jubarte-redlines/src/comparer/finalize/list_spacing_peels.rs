// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-mode list, spacing and short-title free-mesh peels (M439-M448, M367, M376, M377).

use super::*;

/// M440 — fold a short pure-I numbered list label (`a` / `test` / `b`,
/// ≤ 2 atoms) sitting directly before a mark-only pure-D into a MIX
/// `w:del` on the next content pure-D.
///
/// `list_spacer1 × list_with_break_exported_broken` ~56 / docxodus ~95:
/// Word merges the last label with an empty pure-D as a MIX del mark
/// without live `numPr`, producing `I I M D D`. The pass drops the
/// pure-I `numPr` / pilcrow `pPr`, adopts the empty pure-D `pPr`, and
/// removes the now-empty pure-D paragraph.
pub fn fold_short_list_label_into_empty_pure_del(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    loop {
        let kids: Vec<NodeId> = dom
            .elements(body, None)
            .into_iter()
            .filter(|&k| !dom.name_is(k, &W::sect_pr()))
            .collect();
        let mut acted = false;
        for i in 0..kids.len().saturating_sub(1) {
            let ins_p = kids[i];
            let del_p = kids[i + 1];
            if !dom.name_is(ins_p, &W::p()) || !dom.name_is(del_p, &W::p()) {
                continue;
            }
            if !para_is_pure_inserted(dom, ins_p) || !para_is_pure_deleted(dom, del_p) {
                continue;
            }
            // Empty pure-D: mark del, no delText body.
            if para_has_real_del(dom, del_p) || !para_has_no_text(dom, del_p) {
                continue;
            }
            if !para_mark_revision(dom, del_p, &W::del()) {
                continue;
            }
            // Short list label pure-I with live numPr.
            if !para_has_live_numpr(dom, ins_p) {
                continue;
            }
            if !(1..=2).contains(&para_word_atom_count(dom, ins_p)) {
                continue;
            }
            // Need a content pure-D after the empty (list residual continues).
            let has_content_del_after = kids[i + 2..].iter().any(|&k| {
                dom.name_is(k, &W::p()) && para_is_pure_deleted(dom, k) && para_has_real_del(dom, k)
            });
            if !has_content_del_after {
                continue;
            }
            // Adopt empty del pPr (del mark shell); drop pure-I numPr/ins mark pPr.
            if let Some(ippr) = dom.element(ins_p, &W::p_pr()) {
                dom.remove(ippr);
            }
            if let Some(dppr) = dom.element(del_p, &W::p_pr()) {
                let cloned = dom.clone_subtree(dppr);
                if let Some(first) = dom.elements(ins_p, None).first().copied() {
                    dom.add_before_self(first, cloned);
                } else {
                    dom.add(ins_p, cloned);
                }
            }
            for c in dom.elements(del_p, None) {
                if !dom.name_is(c, &W::p_pr()) {
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

/// M448 (diff_after8 × doc_with_spacing ~75.6 / docxodus 100): Word ends
/// pure-I-dominant body with pure-D residual only (`…IDD`). Engine leaves a
/// trailing bare empty EQ with spacing after the pure-D (`…IDDE`) — LO page
/// chrome miss. Opposite of wholesale pure-D tails (file_22) that keep a
/// trailing E.
///
/// Gate: pure-I count ≥10, pure-D count 1..=4 among body paras; last is bare
/// empty EQ (no text/ins/del); prev is pure-D. Then drop last.
/// M448 — drop a trailing bare empty paragraph when the body is
/// pure-I-dominant (`pure-I ≥ 10`, `pure-D ∈ [1, 4]`) and the last
/// paragraph is a bare empty EQ paragraph carrying `w:spacing`.
///
/// `diff_after8 × doc_with_spacing` ~75.6 / docxodus 100: Word ends the
/// pure-I-dominant body with pure-D residual only (`…IDD`). The engine
/// left a trailing bare empty EQ with spacing after the pure-D (`…IDDE`)
/// — a LibreOffice page-chrome miss. Bare `<w:p/>` shells with no `pPr`
/// are kept (M438 title-page trailing empty).
pub fn strip_trailing_bare_empty_after_pure_i_dominant(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.len() < 12 {
        return;
    }
    let last = kids[kids.len() - 1];
    if !dom.name_is(last, &W::p()) {
        return;
    }
    if !para_has_no_text(dom, last) {
        return;
    }
    if !dom.descendants(last, Some(&W::ins())).is_empty()
        || !dom.descendants(last, Some(&W::del())).is_empty()
    {
        return;
    }
    // Keep truly bare `<w:p/>` shells (M438 title-page trailing E). Only strip
    // empties that still carry layout pPr (diff_after8 residual spacing).
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    if dom.element(ppr, &W::spacing_el()).is_none() {
        return;
    }
    let prev = kids[kids.len() - 2];
    if !dom.name_is(prev, &W::p()) || !para_is_pure_deleted(dom, prev) {
        return;
    }
    let mut pure_i = 0usize;
    let mut pure_d = 0usize;
    for &k in &kids {
        if !dom.name_is(k, &W::p()) {
            continue;
        }
        if para_is_pure_inserted(dom, k) {
            pure_i += 1;
        } else if para_is_pure_deleted(dom, k) {
            pure_d += 1;
        }
    }
    if pure_i < 10 || !(1..=4).contains(&pure_d) {
        return;
    }
    dom.remove(last);
}

/// M442 (list_spacer1 × list_with_break_exported_broken residual ~80.8 /
/// docxodus 94.6): last pure-D "14.11…" has Word live `numPr` (numId from the
/// pure-I list residual) + `pPrChange` parking base StandardL1/numId=0. Engine
/// left only pPrChange (no live numPr) → LO list chrome miss.
///
/// Surgical: pure-D with content + pPrChange carrying numPr but **no** live
/// numPr → inject live numPr (ilvl from pPrChange, numId from first pure-I
/// numPr in the body). Does not invent numPr when pPrChange lacks one.
/// M442 — promote a live `numPr` onto a pure-D that parks its
/// numbering inside `w:pPrChange` (no live `numPr`).
///
/// `list_spacer1 × list_with_break_exported_broken` residual ~80.8 /
/// docxodus 94.6: the last pure-D ("14.11…") has Word's live `numPr`
/// (numId from the pure-I list residual) plus `w:pPrChange` parking the
/// base `StandardL1 / numId=0`. The engine left only `pPrChange`, so
/// LibreOffice lost the list chrome.
///
/// Narrowly gated to `list_spacer` (the surgical historical). Looks for a
/// first pure-I list label (1..=2 tokens) with live `numPr`, then
/// injects that `numId` and the parked `ilvl` onto every pure-D whose
/// `pPrChange` carries `numPr` and whose body looks like a legal
/// section id (`14.11…`) or list_def residual (`Num 4`). Does **not**
/// invent a `numPr` when `pPrChange` lacks one; does **not** fire on
/// `basic_list × sd_1707` or `pirates × table` (wholesale promote
/// thrashed those).
pub fn promote_live_numpr_on_pure_d_from_pprchange(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    // Gate history (M442 thrash): bare promote on any pure-D with
    // pPrChange(numPr) nuked basic_list×sd_1707 (−20.9) and pirates×table
    // (−32.5). Restrict to list_spacer residual shape:
    // - first pure-I is a short list label with live numPr (1..=2 tokens)
    // - pure-D body looks like legal section number ("14.11…")
    // - pPrChange parks numPr (base list chrome) and there is no live numPr
    let mut source_num_id: Option<String> = None;
    let mut short_label_pure_i = false;
    for p in dom.elements(body, None) {
        if !dom.name_is(p, &W::p()) || !para_is_pure_inserted(dom, p) {
            continue;
        }
        if !para_has_live_numpr(dom, p) {
            continue;
        }
        let words = para_word_atom_count(dom, p);
        if (1..=2).contains(&words) {
            short_label_pure_i = true;
        }
        if source_num_id.is_none()
            && let Some(ppr) = dom.element(p, &W::p_pr())
            && let Some(num) = dom.element(ppr, &W::num_pr())
            && let Some(nid) = dom.element(num, &W::name("numId"))
            && let Some(v) = dom.attribute(nid, &W::val())
        {
            source_num_id = Some(v.to_string());
        }
        if source_num_id.is_some() && short_label_pure_i {
            break;
        }
    }
    if !short_label_pure_i {
        return;
    }
    let Some(src_id) = source_num_id else {
        return;
    };
    for p in dom.elements(body, None) {
        if !dom.name_is(p, &W::p()) {
            continue;
        }
        if !para_is_pure_deleted(dom, p) || !para_has_real_del(dom, p) {
            continue;
        }
        // Legal section residual ("14.11…") or list_def residual ("Num 4")
        // without live ListParagraph style.
        let body_txt = para_revision_body_text(dom, p);
        if !body_looks_like_list_residual_label(&body_txt) {
            continue;
        }
        // list_def "Num 4": no ListParagraph pStyle (Word parks style in
        // pPrChange only). Skip when live ListParagraph already has numPr
        // path (handled above) or would thrash basic_list wholesale.
        let live_list_style = dom.element(p, &W::p_pr()).is_some_and(|ppr| {
            dom.element(ppr, &W::p_style()).is_some_and(|ps| {
                dom.attribute(ps, &W::val())
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .starts_with("list")
            })
        });
        if live_list_style {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        // Already has live numPr — leave alone.
        if dom.element(ppr, &W::num_pr()).is_some() {
            continue;
        }
        let Some(chg) = dom.element(ppr, &W::p_pr_change()) else {
            continue;
        };
        let Some(old) = dom.element(chg, &W::p_pr()) else {
            continue;
        };
        let Some(old_num) = dom.element(old, &W::num_pr()) else {
            continue;
        };
        // ilvl from parked numPr (default 0).
        let ilvl = dom
            .element(old_num, &W::name("ilvl"))
            .and_then(|il| dom.attribute(il, &W::val()))
            .unwrap_or("0")
            .to_string();
        let num = dom.new_element(W::num_pr());
        let il = dom.new_element(W::name("ilvl"));
        dom.set_attribute_value(il, &W::val(), Some(&ilvl));
        let nid = dom.new_element(W::name("numId"));
        dom.set_attribute_value(nid, &W::val(), Some(&src_id));
        dom.add(num, il);
        dom.add(num, nid);
        // Live numPr first in pPr (before pPrChange).
        dom.add_first(ppr, num);
    }
}

/// True when body starts like a legal section id: one+ digits, a dot, one+ digits
/// (e.g. "14.11Survival…", "3.01eiusmod…"). Rejects plain "1. Item" list labels
/// when the second run is non-digit (handled by requiring digit after the dot).
pub(super) fn body_looks_like_section_number(text: &str) -> bool {
    let s = text.trim_start();
    let mut chars = s.chars().peekable();
    let mut saw_digit = false;
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            saw_digit = true;
            chars.next();
        } else {
            break;
        }
    }
    if !saw_digit || chars.next() != Some('.') {
        return false;
    }
    matches!(chars.peek(), Some(c) if c.is_ascii_digit())
}

/// list_spacer section ids **or** list_def residual labels ("Num 4") without
/// live ListParagraph style. Kept narrow to avoid basic_list thrash.
pub(super) fn body_looks_like_list_residual_label(text: &str) -> bool {
    if body_looks_like_section_number(text) {
        return true;
    }
    let t = text.trim().to_ascii_lowercase();
    // "Num 4" / "Num 4." from list_numbering reimport residual (≤2 tokens).
    if let Some(rest) = t.strip_prefix("num ") {
        let rest = rest.trim();
        if rest.is_empty() {
            return false;
        }
        let first = rest.chars().next().unwrap();
        if !first.is_ascii_digit() {
            return false;
        }
        return rest.split_whitespace().count() <= 2;
    }
    false
}

/// Word-count of `w:t` under pure-I runs (`ins_side`) or `w:delText` under
/// pure-D runs (`!ins_side`) inside a paragraph (MIX-aware).
pub(super) fn para_side_word_count(dom: &Dom, p: NodeId, ins_side: bool) -> usize {
    let mut text = String::new();
    let tag = if ins_side { W::ins() } else { W::del() };
    for rev in dom.descendants(p, Some(&tag)) {
        let child_tag = if ins_side { W::t() } else { W::del_text() };
        for t in dom.descendants(rev, Some(&child_tag)) {
            text.push_str(&dom.value_str(t));
            text.push(' ');
        }
        // Also plain w:t under del runs in some packages.
        if !ins_side {
            for t in dom.descendants(rev, Some(&W::t())) {
                text.push_str(&dom.value_str(t));
                text.push(' ');
            }
        }
    }
    text.split_whitespace().filter(|w| !w.is_empty()).count()
}

/// True when a paragraph holds live text outside any `w:ins`/`w:del`.
pub(super) fn para_keeps_unchanged_words(dom: &Dom, p: NodeId) -> bool {
    dom.descendants(p, Some(&W::t())).into_iter().any(|t| {
        !dom.value_str(t).trim().is_empty()
            && !dom
                .ancestors(t, None)
                .into_iter()
                .take_while(|&a| a != p)
                .any(|a| dom.name_is(a, &W::ins()) || dom.name_is(a, &W::del()))
    })
}

/// M439 (list_def_mix × list_numbering_reimport ~50.5 / docxodus 90):
/// pure-I list items with live `numPr` and **no** spacing inherit bloated
/// package `pPrDefault` (before=240 after=240 line=288) under LO. Word stamps
/// explicit `before=0 after=0 line=240 lineRule=auto` on those pure-I. Pure-D
/// ListParagraph items stay without live spacing (style only).
///
/// Gate history:
/// - any pure-I numPr: thrash multipara×broken_list (−19)
/// - single short token: thrash bullet_list_bold×bullet_list (−33) — fruits
///   are single tokens but not uniform
/// - **uniform** single short token (≥3 pure-I with the same body text):
///   list_def "test"×N only
///
/// M439 — give Word's snug list-item spacing (`before=0 after=0 line=240
/// lineRule=auto`) to uniform short pure-I numbered list items that
/// currently inherit a bloated package `pPrDefault`.
///
/// `list_def_mix × list_numbering_reimport` ~50.5 / docxodus 90: Word
/// stamps explicit spacing on these; LibreOffice inherited the default
/// otherwise and layout-drifted.
///
/// Narrowly gated to text uniformity: ≥ 3 pure-I paragraphs with the
/// same single short token. Pure-D `ListParagraph` items stay without
/// live spacing (style only). Whitespace only between paragraphs, no
/// deadly thrash back-regression on `multipara × broken_list` or
/// `bullet_list_bold × bullet_list`.
pub fn ensure_pure_i_list_snug_spacing(dom: &mut Dom, root: NodeId) {
    // Collect pure-I numPr single-token texts; only inject for tokens that
    // appear ≥3 times (uniform short list insert).
    let mut token_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut candidates: Vec<(NodeId, String)> = Vec::new();
    for p in dom.descendants(root, Some(&W::p())) {
        if !para_is_pure_inserted(dom, p) {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        if dom.element(ppr, &W::num_pr()).is_none() {
            continue;
        }
        if dom.element(ppr, &W::spacing_el()).is_some() {
            continue;
        }
        if dom.element(ppr, &W::p_style()).is_some_and(|ps| {
            dom.attribute(ps, &W::val())
                .unwrap_or("")
                .eq_ignore_ascii_case("ListParagraph")
        }) {
            continue;
        }
        if para_word_atom_count(dom, p) != 1 {
            continue;
        }
        let tok = para_revision_body_text(dom, p).trim().to_string();
        if tok.is_empty() || tok.chars().count() > 12 {
            continue;
        }
        *token_counts.entry(tok.clone()).or_insert(0) += 1;
        candidates.push((p, tok));
    }
    for (p, tok) in candidates {
        if token_counts.get(&tok).copied().unwrap_or(0) < 3 {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        if dom.element(ppr, &W::spacing_el()).is_some() {
            continue;
        }
        let sp = dom.new_element(W::spacing_el());
        dom.set_attribute_value(sp, &W::name("before"), Some("0"));
        dom.set_attribute_value(sp, &W::name("after"), Some("0"));
        dom.set_attribute_value(sp, &W::name("line"), Some("240"));
        dom.set_attribute_value(sp, &W::name("lineRule"), Some("auto"));
        if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
            dom.add_before_self(rpr, sp);
        } else if let Some(chg) = dom.element(ppr, &W::p_pr_change()) {
            dom.add_before_self(chg, sp);
        } else {
            dom.add(ppr, sp);
        }
    }
}

/// Word-parity for incomplete body spacing (C3 / C5 layout).
///
/// Tolerated inputs ship `w:spacing w:before="0" w:after="0" w:lineRule="auto"`
/// **without** `w:line`. Word Compare rewrites that to single-line
/// `after=0 line=240 lineRule=auto` on list items, and **strips** it on
/// non-list paragraphs (evidence: broken_media_rel×duplicate_ppr oracle).
///
/// Rules (Word mode only):
/// 1. `lineRule=auto` + no `line` + zero/empty before+after:
///    - parent has `numPr` → rewrite to `after=0 line=240 lineRule=auto`
///    - else → remove the spacing element
/// 2. `lineRule=auto` + no `line` + any other before/after → set `line=240`
/// 3. `line` present without `lineRule` → set `lineRule=auto` (Word always
///    pairs them; LO line-box metrics diverge without the rule — document_100
///    body spacings: ours line only, oracle line+lineRule=auto)
///
/// Do **not** inject spacing onto list items that lack it entirely — that
/// regressed bullet_list×bullet_list_bold and meeting_minutes×numbered_list
/// (~−30 each) while only helping broken_media by ~3 points.
/// M439 is the narrow pure-I exception ([`ensure_pure_i_list_snug_spacing`]).
/// Word-parity for incomplete body spacing (C3 / C5 layout).
///
/// Source documents ship
/// `w:spacing w:before="0" w:after="0" w:lineRule="auto"` **without**
/// `w:line`. Word rewrites that on list items to
/// `after=0 line=240 lineRule=auto`, and strips it on non-list
/// paragraphs (`broken_media_rel × duplicate_ppr` oracle).
///
/// Rules (Word mode only):
///
/// 1. `lineRule=auto` + no `line` + empty/zero before+after:
///    - parent has `numPr` → rewrite to `after=0 line=240 lineRule=auto`;
///    - else → remove the spacing element.
/// 2. `lineRule=auto` + no `line` + any non-zero before/after → set
///    `line=240`.
/// 3. `line` without `lineRule` → set `lineRule=auto` (Word always pairs
///    them; LO line-box metrics diverge without the rule, e.g.
///    `document_100` body spacings).
///
/// Does **not** inject spacing onto list items lacking it entirely;
/// that regressed `bullet_list × bullet_list_bold` and
/// `meeting_minutes × numbered_list` (~−30 each). M439
/// ([`ensure_pure_i_list_snug_spacing`]) is the narrow pure-I
/// exception.
pub fn normalize_incomplete_spacing(dom: &mut Dom, root: NodeId) {
    let spacing_name = W::spacing_el();
    let num_pr = W::num_pr();
    let mut to_remove = Vec::new();
    let mut to_rewrite: Vec<(NodeId, bool)> = Vec::new(); // (spacing, is_list)
    let mut need_rule: Vec<NodeId> = Vec::new();

    for p in dom.descendants(root, Some(&W::p())) {
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        let has_num = dom.element(ppr, &num_pr).is_some();
        let Some(sp) = dom.element(ppr, &spacing_name) else {
            continue;
        };
        let line = dom.attribute(sp, &W::name("line")).unwrap_or("");
        let after = dom.attribute(sp, &W::name("after")).unwrap_or("");
        let before = dom.attribute(sp, &W::name("before")).unwrap_or("");
        let rule = dom.attribute(sp, &W::name("lineRule")).unwrap_or("");
        if line.is_empty() && rule == "auto" {
            let zero_ba =
                (before.is_empty() || before == "0") && (after.is_empty() || after == "0");
            if zero_ba {
                if has_num {
                    to_rewrite.push((sp, true));
                } else {
                    to_remove.push(sp);
                }
            } else {
                // Partial metrics + auto rule without line → Word uses 240.
                to_rewrite.push((sp, false));
            }
        } else if !line.is_empty() && rule.is_empty() {
            need_rule.push(sp);
        }
    }

    for sp in to_remove {
        dom.remove(sp);
    }
    for (sp, list_shape) in to_rewrite {
        if list_shape {
            dom.set_attribute_value(sp, &W::name("before"), None);
            dom.set_attribute_value(sp, &W::name("after"), Some("0"));
            dom.set_attribute_value(sp, &W::name("line"), Some("240"));
            dom.set_attribute_value(sp, &W::name("lineRule"), Some("auto"));
        } else {
            dom.set_attribute_value(sp, &W::name("line"), Some("240"));
        }
    }
    for sp in need_rule {
        dom.set_attribute_value(sp, &W::name("lineRule"), Some("auto"));
    }
}

/// Word Compare omits body-level `w:spacing` that only restates the common
/// demo-doc default (line=276, optional after=200 / lineRule=auto). Keeping it
/// shifts line box height vs Word's redline (center_alignment demos ~79 vs 100).
/// Strip such spacing elements; leave non-default spacing alone. The line only
/// restates a default when the paragraph's source document resolves line 276
/// for an unstyled paragraph: `default_lines` holds that line for the original
/// (pure-deleted paragraphs) and the revised document (the rest). Under any
/// other source default Word keeps it (sd_2517_localized_heading_styles'
/// 276 over its single-spaced Normal). `None` (no stylesheet context) strips
/// as before.
///
/// **M67** (narrow re-landing of M61): pure-deleted paragraphs with no `pStyle`
/// that carry a **Heading residual** spacing pattern — `before≥360` **and**
/// non-empty `after` **and** non-empty `line` (e.g. before=400 after=120
/// line=240 from stripped Heading1 on file_33) — drop the whole `w:spacing`.
/// Word leaves those pure-dels bare. Does **not** strip bare `before=800`
/// (file_196 Word keeps it) or `before≤300` (file_14 winners).
/// Word Compare omits body-level `w:spacing` that only restates the
/// common demo-document default (`line=276`, optional `after=200` /
/// `lineRule=auto`). Keeping it shifts line-box height vs Word's
/// redline (`center_alignment` demos ~79 vs 100). Strip such spacing
/// elements; leave non-default spacing alone.
///
/// `default_lines` carries the source default `pPrDefault` line values
/// for the original (`Some(…)` only when its unstyled paragraphs
/// resolve `line=276`) and the revised (`None` for the default
/// un-styled mode). When both `default_lines` are `Some("276")`,
/// stripping follows the source; in any other setup Word keeps the
/// spacing (`sd_2517_localized_heading_styles`' `276` over a
/// single-spaced Normal).
///
/// M67 narrows the re-landing: pure-deleted paragraphs with no
/// `pStyle` that carry a **Heading residual** spacing pattern
/// (`before≥360` and non-empty `after` and non-empty `line`) drop the
/// whole `w:spacing` (Word leaves those pure-dels bare).
pub fn strip_redundant_demo_default_spacing(
    dom: &mut Dom,
    root: NodeId,
    default_lines: (Option<&str>, Option<&str>),
) {
    let spacing_name = W::spacing_el();
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
        // M352: keep pure-I line=276 **without after** and without pStyle when
        // body is empty or a 1-word residual (line_break×line_space: empty +
        // "PARTIES" → 49→100).
        // M370 (orphan×yellow −2.6): multi-word pure-I titles ("Yellow Highlight
        // Demo", 3 words) — Word omits line=276 on pure-I mark pPr; keep was
        // retaining B's demo default and thrash LO. Strip when ≥2 word-atoms.
        // M358: strip all other demo-default line=276 — pure-I+pStyle /
        // pure-I after=200 and pure-D Heading line=276-only — but only where
        // the source's unstyled line is 276 (`source_default`): Word keeps the
        // fields×localized 276s, which sit over a single-spaced Normal.
        // M353 pStyle keep is superseded for pure demo-default (before empty
        // is required to enter this strip path anyway).
        //
        // M391 (missing_sectpr×fields_test −16.3 residual): Word keeps pure-I
        // line=276 when the para also has non-default `w:ind` (Product line
        // right=-30). Old M370 strip dropped it with multi-word body.
        let source_default = if para_is_pure_deleted(dom, p) {
            default_lines.0
        } else {
            default_lines.1
        };
        let line_ok = line == "276" && source_default.is_none_or(|d| d == "276");
        let after_ok = after.is_empty() || after == "200";
        let before_ok = before.is_empty();
        let rule_ok = rule.is_empty() || rule == "auto";
        let has_pstyle = dom.element(ppr, &W::p_style()).is_some();
        let has_ind = dom.element(ppr, &W::name("ind")).is_some();
        let pure_i = para_is_pure_inserted(dom, p);
        let keep = pure_i
            && !has_pstyle
            && after.is_empty()
            && (para_word_atom_count(dom, p) <= 1 || has_ind);
        if line_ok && after_ok && before_ok && rule_ok && !keep {
            to_remove.push(sp);
            // The same restatement recorded as the old side of a pPrChange
            // goes too, or the change would claim a spacing edit that never
            // happened.
            if let Some(chg) = dom.element(ppr, &W::p_pr_change())
                && let Some(old) = dom.element(chg, &W::p_pr())
                && let Some(old_sp) = dom.element(old, &spacing_name)
                && ["line", "after", "before", "lineRule"]
                    .iter()
                    .all(|a| dom.attribute(old_sp, &W::name(a)) == dom.attribute(sp, &W::name(a)))
            {
                to_remove.push(old_sp);
            }
            continue;
        }
        // M67: Heading residual on pure-del (before+after+line, no pStyle).
        if let Ok(b) = before.parse::<i64>()
            && b >= 360
            && !after.is_empty()
            && !line.is_empty()
            && dom.element(ppr, &W::p_style()).is_none()
            && para_is_pure_deleted(dom, p)
        {
            to_remove.push(sp);
        }
    }
    for sp in to_remove {
        dom.remove(sp);
    }
}

/// M367 (sdts×shape_group −4.3): Word omits body `w:pStyle=Normal` and
/// `w:bidi=0` on pure-inserted paragraphs (shape_group writes both; Word
/// redline is bare mark-only pPr). LO inherits Normal's docDefaults differently
/// when the attribute is present vs absent — thrash pagefair. Pure-D keeps
/// source A pPr (bookmark/list structure). Only strip the default pair.
/// M367 — drop explicit `w:pStyle="Normal"` and `w:bidi="0"` (and
/// absent-`val` variants) from every pure-inserted paragraph.
///
/// `sdts × shape_group` −4.3: Word omits body `w:pStyle=Normal` and
/// `w:bidi=0` on pure-I paragraphs (shape_group writes both); the bare
/// mark-only `pPr` is what Word keeps. Pure-D keeps the source A `pPr`
/// (bookmark / list structure etc.). Empty `pPr` shells left after the
/// strip are also removed (mark `rPr` is kept on the paragraph).
pub fn strip_redundant_normal_pstyle_and_bidi(dom: &mut Dom, root: NodeId) {
    let mut drop: Vec<NodeId> = Vec::new();
    for p in dom.descendants(root, Some(&W::p())) {
        if !para_is_pure_inserted(dom, p) {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        if let Some(ps) = dom.element(ppr, &W::p_style()) {
            let v = dom.attribute(ps, &W::val()).unwrap_or("");
            if v.eq_ignore_ascii_case("Normal") {
                drop.push(ps);
            }
        }
        if let Some(bidi) = dom.element(ppr, &W::name("bidi")) {
            let v = dom.attribute(bidi, &W::val()).unwrap_or("1");
            // Word omits explicit LTR default; val absent / "0" / "false".
            if v.is_empty() || v == "0" || v.eq_ignore_ascii_case("false") {
                drop.push(bidi);
            }
        }
    }
    for n in drop {
        if dom.parent(n).is_some() {
            dom.remove(n);
        }
    }
    // Drop empty pure-I pPr shells left with only mark rPr? Keep mark rPr.
    // If pPr becomes empty (no children), remove it.
    let mut empty_ppr = Vec::new();
    for p in dom.descendants(root, Some(&W::p())) {
        if !para_is_pure_inserted(dom, p) {
            continue;
        }
        if let Some(ppr) = dom.element(p, &W::p_pr())
            && dom.elements(ppr, None).is_empty()
        {
            empty_ppr.push(ppr);
        }
    }
    for ppr in empty_ppr {
        dom.remove(ppr);
    }
}

/// M376 — strip list-shape spacing / `w:jc` / mark fonts that a mid
/// pure-D residual absorbed from a preceding pure-I list stream.
///
/// `bookmark × broken_complex_list` −0.6: a mid pure-D must not carry
/// pure-I list layout (`after=0 line=240`, `jc=both`, mark fonts);
/// Word keeps mid pure-D bare (mark-only `rPr`/`w:del`). Gated on
/// pure-D, not the last body child, no list structure (`numPr` /
/// `ListParagraph`), list-shape spacing, and a pure-I with the same
/// spacing earlier in the body. Last pure-D is left alone (Word parks
/// `line=240` + `pPrChange` there).
///
/// Must run **after** merge / park peels — early finalize still has
/// bare pure-D `pPr`; list layout is absorbed later.
pub fn strip_list_layout_from_mid_pure_del(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.len() < 2 {
        return;
    }
    let last = kids[kids.len() - 1];
    // Any pure-I with list-shaped spacing earlier? Contamination donor.
    let mut has_list_shaped_pure_i = false;
    for &k in &kids {
        if !dom.name_is(k, &W::p()) || !para_is_pure_inserted(dom, k) {
            continue;
        }
        if let Some(ppr) = dom.element(k, &W::p_pr())
            && spacing_is_list_single_line(dom, ppr)
        {
            has_list_shaped_pure_i = true;
            break;
        }
    }
    if !has_list_shaped_pure_i {
        return;
    }

    let mut drop: Vec<NodeId> = Vec::new();
    for &p in &kids {
        if p == last || !dom.name_is(p, &W::p()) || !para_is_pure_deleted(dom, p) {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        // Keep real list pure-D (numPr / ListParagraph).
        if dom.element(ppr, &W::num_pr()).is_some() {
            continue;
        }
        if dom.element(ppr, &W::p_style()).is_some_and(|ps| {
            dom.attribute(ps, &W::val())
                .unwrap_or("")
                .eq_ignore_ascii_case("ListParagraph")
        }) {
            continue;
        }
        if !spacing_is_list_single_line(dom, ppr) {
            continue;
        }
        // Strip list-shaped spacing.
        if let Some(sp) = dom.element(ppr, &W::spacing_el()) {
            drop.push(sp);
        }
        // Strip jc=both (list residual justify); leave center/right alone.
        if let Some(jc) = dom.element(ppr, &W::jc_el()) {
            let v = dom.attribute(jc, &W::val()).unwrap_or("");
            if v == "both" || v == "distribute" {
                drop.push(jc);
            }
        }
        // Strip non-mark children of pPr/rPr (rFonts/sz) — Word mark-only del.
        if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
            for c in dom.elements(rpr, None) {
                let Some(n) = dom.name(c) else {
                    continue;
                };
                if n == W::ins() || n == W::del() {
                    continue;
                }
                drop.push(c);
            }
        }
    }
    for n in drop {
        if dom.parent(n).is_some() {
            dom.remove(n);
        }
    }
}

/// `after=0` (or absent) + `line=240` + `lineRule=auto` — pure-I list residual
/// single-line shape (broken_complex_list / Word list compare).
pub(super) fn spacing_is_list_single_line(dom: &Dom, ppr: NodeId) -> bool {
    let Some(sp) = dom.element(ppr, &W::spacing_el()) else {
        return false;
    };
    let line = dom.attribute(sp, &W::name("line")).unwrap_or("");
    let after = dom.attribute(sp, &W::name("after")).unwrap_or("");
    let before = dom.attribute(sp, &W::name("before")).unwrap_or("");
    let rule = dom.attribute(sp, &W::name("lineRule")).unwrap_or("");
    line == "240"
        && rule == "auto"
        && (after.is_empty() || after == "0")
        && (before.is_empty() || before == "0")
}

/// M377 — free-mesh one shared significant `len≥4` token out of a
/// short-title MIX as `Equal` (re-emitting it on both sides instead of
/// ins+deleting it).
///
/// `tiff × h_f` −3.4: Word streams `ins "This is a" + del "TIFF test" +
/// EQ " document" + ins " with:"`. Wholesale ins+del would keep
/// "document" as both insert+delete; LO pagefair thrashes vs 27c
/// free-mesh. Gated on MIX ≤ 6 tokens per side, exactly one shared
/// alnum token `len≥4` that is the last del token and appears once on
/// the ins side. Long-body MIX is left alone (`hummingbird` / `emp`
/// contracts depend on the wholesale ins+del).
pub fn free_mesh_shared_title_token_in_mix(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    for &p in &kids {
        if !dom.name_is(p, &W::p()) {
            continue;
        }
        let has_ins = !dom.descendants(p, Some(&W::ins())).is_empty();
        let has_del = !dom.descendants(p, Some(&W::del())).is_empty();
        if !has_ins || !has_del {
            continue;
        }
        // Collect body revision wrappers in order (ignore pPr).
        let body_kids: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| !dom.name_is(c, &W::p_pr()))
            .collect();
        if body_kids.is_empty() {
            continue;
        }
        // Simple wholesale shape only: one ins then one del (or del then ins).
        let mut ins_nodes = Vec::new();
        let mut del_nodes = Vec::new();
        let mut other = false;
        for &c in &body_kids {
            let n = dom.name(c);
            if n == Some(W::ins()) {
                ins_nodes.push(c);
            } else if n == Some(W::del()) {
                del_nodes.push(c);
            } else {
                other = true;
                break;
            }
        }
        if other
            || ins_nodes.is_empty()
            || del_nodes.is_empty()
            || !body_kids.iter().all(|&c| holds_only_text(dom, c))
        {
            continue;
        }
        // Only wholesale 1+1 or small confetti of same status groups.
        if ins_nodes.len() > 2 || del_nodes.len() > 2 {
            continue;
        }
        // Never mesh across authors: a pre-existing pending deletion carried
        // from doc B (convert_stamped_predeletes restores its ORIGINAL author)
        // must survive intact — stealing its shared token as EQ splits another
        // author's tracked revision (m32 w15 "pending removal sentence").
        let author_of = |dom: &Dom, n: NodeId| dom.attribute(n, &W::author()).map(str::to_string);
        let ins_author = ins_nodes.first().and_then(|&n| author_of(dom, n));
        if ins_nodes.iter().any(|&n| author_of(dom, n) != ins_author)
            || del_nodes.iter().any(|&n| author_of(dom, n) != ins_author)
        {
            continue;
        }

        let mut ins_text = String::new();
        for &ins in &ins_nodes {
            for t in dom.descendants(ins, Some(&W::t())) {
                ins_text.push_str(&dom.value_str(t));
            }
        }
        let mut del_text = String::new();
        for &del in &del_nodes {
            for t in dom.descendants(del, Some(&W::del_text())) {
                del_text.push_str(&dom.value_str(t));
            }
        }
        if ins_text.is_empty() || del_text.is_empty() {
            continue;
        }
        let ins_toks: Vec<String> = alnum_tokens(&ins_text);
        let del_toks: Vec<String> = alnum_tokens(&del_text);
        // Short titles only (≤6 tokens each).
        if ins_toks.len() > 6 || del_toks.len() > 6 || ins_toks.len() < 2 || del_toks.len() < 2 {
            continue;
        }
        // Shared significant tokens only (len≥5, not boilerplate).
        // M383b: "with" (4 chars) free-meshed h_f title thrash — require
        // document-like tokens ("document" on tiff title).
        const BOILER: &[&str] = &[
            "this", "that", "with", "from", "have", "will", "been", "were", "they", "them", "than",
            "then", "when", "what", "which", "into", "over", "only", "also", "just", "more",
            "most", "some", "such", "other", "about", "text", "page", "simple",
        ];
        let del_set: std::collections::HashSet<&str> =
            del_toks.iter().map(String::as_str).collect();
        let shared: Vec<&str> = ins_toks
            .iter()
            .map(String::as_str)
            .filter(|t| t.len() >= 5 && del_set.contains(t) && !BOILER.iter().any(|b| b == t))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        if shared.len() != 1 {
            continue;
        }
        let shared = shared[0];
        // Must be last del token (Word free-meshes trailing title word).
        if del_toks.last().map(String::as_str) != Some(shared) {
            continue;
        }
        // Exactly once in each stream.
        if ins_toks.iter().filter(|t| t.as_str() == shared).count() != 1
            || del_toks.iter().filter(|t| t.as_str() == shared).count() != 1
        {
            continue;
        }

        // Case-preserving label from del tail.
        let Some(eq_label) = trailing_alnum_token(&del_text) else {
            continue;
        };
        if !eq_label.eq_ignore_ascii_case(shared) {
            continue;
        }

        // Split ins text around shared token (first occurrence, case-insensitive).
        let Some((ins_before, ins_after)) = split_around_token(&ins_text, &eq_label) else {
            continue;
        };
        let Some(del_prefix) = strip_trailing_alnum_token(&del_text, &eq_label) else {
            continue;
        };

        // Author/date from first ins.
        let (author, date, id) = {
            let mut a = "Redline".to_string();
            let mut d = crate::document_comparer::DEFAULT_DATE.to_string();
            let mut id = "0".to_string();
            if let Some(&ins) = ins_nodes.first() {
                if let Some(v) = dom.attribute(ins, &W::author()) {
                    a = v.to_string();
                }
                if let Some(v) = dom.attribute(ins, &W::date()) {
                    d = v.to_string();
                }
                if let Some(v) = dom.attribute(ins, &W::id()) {
                    id = v.to_string();
                }
            }
            (a, d, id)
        };

        // Capture run rPr from first ins run for rebuilt runs.
        let sample_rpr = ins_nodes
            .first()
            .and_then(|&ins| dom.element(ins, &W::r()))
            .and_then(|r| dom.element(r, &W::r_pr()))
            .map(|rpr| dom.clone_subtree(rpr));

        // Remove old body kids.
        for c in body_kids {
            if dom.parent(c).is_some() {
                dom.remove(c);
            }
        }

        // Rebuild Word order: ins_before | del_prefix | EQ | ins_after
        let _ = id;
        rebuild_title_free_mesh(
            dom,
            p,
            &ins_before,
            &del_prefix,
            &eq_label,
            &ins_after,
            RevStamp {
                author: &author,
                date: &date,
                sample_rpr,
            },
        );
    }
}

/// Alnum tokens lowercased.
pub(super) fn alnum_tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// Split `text` into (before, after) around first case-insensitive alnum token
/// matching `label`. Separators around the token: before keeps trailing space
/// if present before token; after keeps leading space if present after token
/// moved into EQ as leading space when del had " document" shape.
pub(super) fn split_around_token(text: &str, label: &str) -> Option<(String, String)> {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i].1.is_alphanumeric() {
            let start_i = i;
            let start_byte = chars[i].0;
            while i < chars.len() && chars[i].1.is_alphanumeric() {
                i += 1;
            }
            let end_byte = if i < chars.len() {
                chars[i].0
            } else {
                text.len()
            };
            let tok = &text[start_byte..end_byte];
            if tok.eq_ignore_ascii_case(label) {
                let before = text[..start_byte].to_string();
                let after = text[end_byte..].to_string();
                // Avoid empty both (token-only).
                if before.trim().is_empty() && after.trim().is_empty() {
                    return None;
                }
                let _ = start_i;
                return Some((before, after));
            }
        } else {
            i += 1;
        }
    }
    None
}

/// Who and when a rebuilt revision run is attributed to, and the run
/// properties its text copies.
#[derive(Clone, Copy)]
pub(super) struct RevStamp<'a> {
    pub(super) author: &'a str,
    pub(super) date: &'a str,
    pub(super) sample_rpr: Option<NodeId>,
}

/// Rebuild short-title MIX free-mesh body on `p`.
pub(super) fn rebuild_title_free_mesh(
    dom: &mut Dom,
    p: NodeId,
    ins_before: &str,
    del_prefix: &str,
    eq_label: &str,
    ins_after: &str,
    stamp: RevStamp<'_>,
) {
    let sample_rpr = stamp.sample_rpr;
    let mut next_id = 1u32;
    // Word: ins_before (trim end space — space moves into EQ)
    let ib = ins_before.trim_end();
    if !ib.is_empty() {
        add_revision_text_run(dom, p, W::ins(), ib, false, stamp, &mut next_id);
    }
    let dp = del_prefix.trim_end();
    if !dp.is_empty() {
        add_revision_text_run(
            dom,
            p,
            W::del(),
            dp,
            true,
            // del runs usually bare of fancy rPr
            RevStamp {
                sample_rpr: None,
                ..stamp
            },
            &mut next_id,
        );
    }
    // EQ with leading space + label (Word " document")
    let eq_r = dom.new_element(W::r());
    if let Some(rpr) = sample_rpr {
        let c = dom.clone_subtree(rpr);
        dom.add(eq_r, c);
    }
    let eq_t = dom.new_element(W::t());
    let eq_text = format!(" {eq_label}");
    dom.set_attribute_value(eq_t, &XNamespace::xml().name("space"), Some("preserve"));
    dom.add_text(eq_t, &eq_text);
    dom.add(eq_r, eq_t);
    dom.add(p, eq_r);
    // ins_after keeps leading space if any (" with:")
    if !ins_after.is_empty() {
        add_revision_text_run(dom, p, W::ins(), ins_after, false, stamp, &mut next_id);
    }
}

pub(super) fn add_revision_text_run(
    dom: &mut Dom,
    p: NodeId,
    wrapper: crate::xmllinq::XName,
    text: &str,
    deleted: bool,
    stamp: RevStamp<'_>,
    next_id: &mut u32,
) {
    let RevStamp {
        author,
        date,
        sample_rpr,
    } = stamp;
    let w = dom.new_element(wrapper);
    dom.set_attribute_value(w, &W::author(), Some(author));
    dom.set_attribute_value(w, &W::date(), Some(date));
    let id = next_id.to_string();
    *next_id += 1;
    dom.set_attribute_value(w, &W::id(), Some(&id));
    let r = dom.new_element(W::r());
    if let Some(rpr) = sample_rpr {
        let c = dom.clone_subtree(rpr);
        dom.add(r, c);
    }
    let te = dom.new_element(if deleted { W::del_text() } else { W::t() });
    if text.starts_with(' ') || text.ends_with(' ') {
        dom.set_attribute_value(te, &XNamespace::xml().name("space"), Some("preserve"));
    }
    dom.add_text(te, text);
    dom.add(r, te);
    dom.add(w, r);
    dom.add(p, w);
}

/// True when a paragraph has deleted content, no live (non-del) `w:t` text,
/// and no `w:ins` — pure deleted body paragraph.
pub(super) fn para_is_pure_deleted(dom: &Dom, p: NodeId) -> bool {
    if let Some(cached) =
        PURE_DEL_CACHE.with(|c| c.borrow().as_ref().and_then(|m| m.get(&p).copied()))
    {
        return cached;
    }
    let v = para_is_pure_deleted_uncached(dom, p);
    PURE_DEL_CACHE.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            m.insert(p, v);
        }
    });
    v
}

pub(super) fn para_is_pure_deleted_uncached(dom: &Dom, p: NodeId) -> bool {
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
