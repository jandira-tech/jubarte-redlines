// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Borrowed periods, echoed heading deletions, paragraph property sources and the M452-M458 alignment peels.

use super::*;

/// Move a paragraph-final `.` whose shared equal segment is genuinely
/// shared (the next paragraph starts with equal text) onto the side
/// that already finished its sentence, and append a `.` to the
/// continuation.
///
/// `font color` p1 `.` → `This text`: without the fix the period sits
/// in the middle of the next sentence on one side and is missing
/// from the end of the previous sentence on the other. The pass
/// detects that shape (the next paragraph must NOT itself start with
/// equal text — that `.` was the previous sentence's shared stop) and
/// re-emits a fresh `w:id` from `id_gen`.
pub fn repair_borrowed_sentence_period(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom.elements(body, None);
    for &[prev, next] in kids.array_windows() {
        if !dom.name_is(prev, &W::p()) || !dom.name_is(next, &W::p()) {
            continue;
        }
        let Some(period_t) = trailing_equal_period(dom, prev) else {
            continue;
        };
        let leaves = text_leaves(dom, next);
        if leaves
            .first()
            .is_some_and(|&t| text_side(dom, t, next) == "eq")
        {
            continue;
        }
        let ins = side_text(dom, next, "ins");
        let del = side_text(dom, next, "del");
        let ins_needs = side_needs_borrowed_period(&ins);
        let del_needs = side_needs_borrowed_period(&del);
        if ins_needs == del_needs {
            continue;
        }
        let (wrapper, keep) = if del_needs {
            (W::ins(), "del")
        } else {
            (W::del(), "ins")
        };
        wrap_run(dom, period_t, wrapper, settings, id_gen);
        append_period(dom, next, keep);
    }
}

pub(super) fn trailing_equal_period(dom: &Dom, p: NodeId) -> Option<NodeId> {
    let t = *text_leaves(dom, p).last()?;
    if !dom.name_is(t, &W::t()) || text_side(dom, t, p) != "eq" || dom.value_str(t) != "." {
        return None;
    }
    let r = dom.parent(t)?;
    if !dom.name_is(r, &W::r()) || dom.parent(r) != Some(p) {
        return None;
    }
    Some(t)
}

pub(super) fn side_needs_borrowed_period(text: &str) -> bool {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return false;
    }
    if matches!(trimmed.chars().last(), Some('.' | '!' | '?')) {
        return false;
    }
    text.starts_with(char::is_whitespace) || trimmed.chars().next().is_some_and(char::is_lowercase)
}

pub(super) fn text_leaves(dom: &Dom, p: NodeId) -> Vec<NodeId> {
    dom.descendants(p, None)
        .into_iter()
        .filter(|&n| dom.name_is(n, &W::t()) || dom.name_is(n, &W::del_text()))
        .collect()
}

pub(super) fn text_side(dom: &Dom, t: NodeId, stop: NodeId) -> &'static str {
    if dom.name_is(t, &W::del_text()) {
        return "del";
    }
    for a in dom.ancestors_and_self(t, None) {
        if a == stop {
            break;
        }
        if dom.name_is(a, &W::ins()) || dom.name_is(a, &W::move_to()) {
            return "ins";
        }
        if dom.name_is(a, &W::del()) || dom.name_is(a, &W::move_from()) {
            return "del";
        }
    }
    "eq"
}

pub(super) fn side_text(dom: &Dom, p: NodeId, side: &str) -> String {
    let mut s = String::new();
    for t in text_leaves(dom, p) {
        if text_side(dom, t, p) == side {
            s.push_str(&dom.value_str(t));
        }
    }
    s
}

pub(super) fn wrap_run(
    dom: &mut Dom,
    t: NodeId,
    wrapper_name: crate::xmllinq::XName,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let Some(r) = dom.parent(t) else {
        return;
    };
    let w = dom.new_element(wrapper_name);
    dom.set_attribute_value(w, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;
    dom.set_attribute_value(w, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(w, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add_before_self(r, w);
    dom.remove(r);
    dom.add(w, r);
}

pub(super) fn append_period(dom: &mut Dom, p: NodeId, side: &str) {
    let Some(t) = text_leaves(dom, p)
        .into_iter()
        .rev()
        .find(|&n| text_side(dom, n, p) == side)
    else {
        return;
    };
    let next = format!("{}.", dom.value_str(t));
    dom.set_value(t, &next);
}

/// The comments redline aligns B's capability table with A's and emits A's
/// preceding section (Parity through the old "3. Capability matrix" heading)
/// as pure deletions *after* the table. Those deletions are original-only, so
/// their order is the original's order: the section, then the table's own
/// deleted rows. When the deleted run repeats the inserted heading, move the
/// run back in front of that heading.
/// Re-anchor a deleted heading that the diff echoes just before the
/// table it belongs to.
///
/// `lots-of-comments`: Word keeps the deleted section's heading
/// before the table (original order); the diff leaves it after the
/// heading's inserted replacement. This pass recognizes an inserted
/// heading that is echoed by an immediate pure-D after it and moves
/// the deletion back before the table.
pub fn restore_echoed_heading_deletions_before_table(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom.elements(body, None);
    let mut plans: Vec<(Vec<NodeId>, NodeId)> = Vec::new();
    let mut i = 0;
    while i < kids.len() {
        if !table_has_both_sides(dom, kids[i]) {
            i += 1;
            continue;
        }
        let mut ins_start = i;
        while ins_start > 0 && para_is_pure_inserted(dom, kids[ins_start - 1]) {
            ins_start -= 1;
        }
        if ins_start == i {
            i += 1;
            continue;
        }
        let mut del_end = i + 1;
        while del_end < kids.len() && block_is_original_only(dom, kids[del_end]) {
            del_end += 1;
        }
        if del_end == i + 1 {
            i += 1;
            continue;
        }
        let inserted: Vec<String> = kids[ins_start..i]
            .iter()
            .map(|&p| collapsed_text(dom, p))
            .filter(|s| !s.is_empty())
            .collect();
        let echoed = kids[i + 1..del_end].iter().any(|&b| {
            paragraph_texts(dom, b)
                .iter()
                .any(|t| inserted.iter().any(|s| s == t))
        });
        if echoed {
            plans.push((kids[i + 1..del_end].to_vec(), kids[ins_start]));
        }
        i = del_end;
    }
    for (blocks, anchor) in plans {
        // Each insert lands immediately before the anchor, so walking the
        // blocks forward leaves them in their original relative order.
        for b in blocks {
            dom.remove(b);
            dom.add_before_self(anchor, b);
        }
    }
}

pub(super) fn table_has_both_sides(dom: &Dom, n: NodeId) -> bool {
    if !dom.name_is(n, &W::tbl()) {
        return false;
    }
    let mut ins = false;
    let mut del = false;
    for t in text_leaves(dom, n) {
        match text_side(dom, t, n) {
            "ins" if !dom.value_str(t).trim().is_empty() => ins = true,
            "del" if !dom.value_str(t).trim().is_empty() => del = true,
            _ => {}
        }
        if ins && del {
            return true;
        }
    }
    false
}

/// Original-only body block: deleted or moved-from text, and no inserted or
/// moved-to text. The echoed heading in the comments pair is a `w:moveFrom`
/// paragraph, which a pure-`w:del` check misses.
pub(super) fn block_is_original_only(dom: &Dom, n: NodeId) -> bool {
    if !dom.name_is(n, &W::p()) && !dom.name_is(n, &W::tbl()) {
        return false;
    }
    if !dom.descendants(n, Some(&W::ins())).is_empty()
        || !dom.descendants(n, Some(&W::move_to())).is_empty()
    {
        return false;
    }
    text_leaves(dom, n)
        .into_iter()
        .any(|t| !dom.value_str(t).trim().is_empty() && text_side(dom, t, n) == "del")
}

pub(super) fn collapsed_text(dom: &Dom, n: NodeId) -> String {
    let mut s = String::new();
    for t in text_leaves(dom, n) {
        s.push_str(&dom.value_str(t));
    }
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

pub(super) fn paragraph_texts(dom: &Dom, block: NodeId) -> Vec<String> {
    if dom.name_is(block, &W::p()) {
        let t = collapsed_text(dom, block);
        return if t.is_empty() { Vec::new() } else { vec![t] };
    }
    dom.descendants(block, Some(&W::p()))
        .into_iter()
        .map(|p| collapsed_text(dom, p))
        .filter(|t| !t.is_empty())
        .collect()
}

/// Paragraph-owned properties captured before atomization and layout peels.
/// Text identifies a source paragraph only when it occurs exactly once on
/// that side; ambiguous or merged paragraphs cannot supply formatting history.
#[derive(Default)]
pub struct ParagraphPropertySources {
    original: HashMap<String, Vec<NodeId>>,
    revised: HashMap<String, Vec<NodeId>>,
}

impl ParagraphPropertySources {
    /// Save each input paragraph's own comparable properties, including an
    /// empty `w:pPr` for a paragraph that has none.
    pub fn capture(dom: &mut Dom, original: NodeId, revised: NodeId) -> Self {
        let mut capture = |body| {
            let mut by_text: HashMap<String, Vec<NodeId>> = HashMap::new();
            for p in dom.descendants(body, Some(&W::p())) {
                let text = revised_text(dom, p).trim().to_string();
                let properties = match dom.element(p, &W::p_pr()) {
                    Some(ppr) => formatchg::project_para_properties_for_change(dom, ppr),
                    None => dom.new_element(W::p_pr()),
                };
                by_text.entry(text).or_default().push(properties);
            }
            by_text
        };
        Self {
            original: capture(original),
            revised: capture(revised),
        }
    }

    fn for_mix(&self, dom: &Dom, p: NodeId) -> Option<(NodeId, NodeId)> {
        let text = |excluded| {
            text_leaves(dom, p)
                .into_iter()
                .filter(|&t| text_side(dom, t, p) != excluded)
                .map(|t| dom.value(t))
                .collect::<String>()
                .trim()
                .to_string()
        };
        let [old] = self.original.get(&text("ins"))?.as_slice() else {
            return None;
        };
        let [new] = self.revised.get(&text("del"))?.as_slice() else {
            return None;
        };
        Some((*old, *new))
    }
}

/// M452: a short title MIX can lose its old alignment during layout peels.
/// Record it only when its own original and revised properties differ.
/// A text-only edit must not borrow alignment from another body paragraph
/// (#1149); the history carries the source paragraph's complete comparable pPr.
pub fn park_jc_on_first_short_title_mix_from_body(
    dom: &mut Dom,
    root: NodeId,
    sources: &ParagraphPropertySources,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    if !settings.detect_format_changes {
        return;
    }
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
    // First MIX only (title free-mesh).
    let mut title: Option<NodeId> = None;
    for &k in &kids {
        if !dom.name_is(k, &W::p()) {
            continue;
        }
        if para_is_mixed_revision(dom, k) {
            title = Some(k);
            break;
        }
    }
    let Some(p) = title else {
        return;
    };
    let ins_w = para_side_word_count(dom, p, true);
    let del_w = para_side_word_count(dom, p, false);
    if ins_w + del_w == 0 || ins_w + del_w > 5 {
        return;
    }
    let Some((old, new)) = sources.for_mix(dom, p) else {
        return;
    };
    if formatchg::normalize_para_properties(dom, old)
        == formatchg::normalize_para_properties(dom, new)
    {
        return;
    }
    let Some(jc) = dom.element(old, &W::jc_el()) else {
        return;
    };
    if !matches!(dom.attribute(jc, &W::val()), Some("right" | "center")) {
        return;
    }
    let ppr = match dom.element(p, &W::p_pr()) {
        Some(ppr) => ppr,
        None => {
            let ppr = dom.new_element(W::p_pr());
            dom.add_first(p, ppr);
            ppr
        }
    };
    if dom.element(ppr, &W::jc_el()).is_some() || dom.element(ppr, &W::p_pr_change()).is_some() {
        return;
    }
    let old_inner = dom.clone_subtree(old);
    let chg = dom.new_element(W::p_pr_change());
    dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;
    dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add(chg, old_inner);
    dom.add(ppr, chg);
}

/// M453 (calibri_font × calibri_heading_2_right mid residual):
/// Word keeps live Heading residual spacing **and** an empty `pPrChange`
/// shell on mid MIX. Engine has live spacing only (last MIX already fixed
/// by M450). Also covers EQ/pure-I with live heading residual (rare).
///
/// Does **not** touch MIX with live `jc` (M451 deliberately strips empty
/// shells there — center_alignment_2 mid shape).
/// M453 — synthesize a layout-only `w:pPrChange` shell under a mid
/// MIX paragraph that has live Heading residual spacing but no
/// `pPrChange`.
///
/// `calibri` mid residual: Word still records the layout history even
/// when nothing else changes. Skips MIX paragraphs that also carry
/// live `w:jc` (M451 owns those).
pub fn ensure_empty_pprchange_on_live_heading_spacing(
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
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.len() < 2 {
        return;
    }
    let last = kids[kids.len() - 1];
    // M453d: only fire when last residual already has empty pPrChange (M450
    // calibri chain). heading_1/2 style free-mesh has live spacing on mid+last
    // **without** empty shells (Word) — thrash −6 if we inject mid-only.
    let last_has_empty_chg = (|| {
        if !dom.name_is(last, &W::p()) {
            return false;
        }
        let Some(lppr) = dom.element(last, &W::p_pr()) else {
            return false;
        };
        let Some(chg) = dom.element(lppr, &W::p_pr_change()) else {
            return false;
        };
        let Some(old) = dom.element(chg, &W::p_pr()) else {
            return false;
        };
        // Empty of layout children (ignore rPr).
        for c in dom.elements(old, None) {
            let Some(n) = dom.name(c) else {
                continue;
            };
            if n == W::r_pr() {
                continue;
            }
            return false;
        }
        true
    })();
    if !last_has_empty_chg {
        return;
    }
    for &p in &kids {
        // Only **non-last MIX**. Last already has empty shell via M450.
        if p == last || !dom.name_is(p, &W::p()) || !para_is_mixed_revision(dom, p) {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        if dom.element(ppr, &W::p_pr_change()).is_some() {
            continue;
        }
        // Skip MIX with live jc — M451 Word shape is no empty shell.
        if dom.element(ppr, &W::jc_el()).is_some() {
            continue;
        }
        // Word heading free-mesh keeps live spacing + rPr without empty shell.
        if dom.element(ppr, &W::r_pr()).is_some() {
            continue;
        }
        let Some(sp) = dom.element(ppr, &W::spacing_el()) else {
            continue;
        };
        let before = dom
            .attribute(sp, &W::name("before"))
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        if before < 200 || dom.attribute(sp, &W::name("line")).is_none() {
            continue;
        }
        let old_inner = dom.new_element(W::p_pr());
        let chg = dom.new_element(W::p_pr_change());
        dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
        *id_gen += 1;
        dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
        dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
        dom.add(chg, old_inner);
        dom.add(ppr, chg);
    }
}

/// The text of each paragraph that sets no `w:jc`.
/// The trimmed text of every REVISED paragraph that does NOT set its
/// own `w:jc`.
///
/// Used as the M449 gate (`promote_live_jc_from_pprchange_on_body_mix`):
/// if a MIX paragraph's revised text is in this set, the MIX
/// paragraph's `pPrChange` is preserved (Word records the old
/// centering over the revision's bare properties), and the live `jc`
/// stays out of `pPr`. Returned body is `body2` (the revised input)
/// before atomize consumes it.
pub fn unaligned_paragraph_texts(dom: &Dom, body: NodeId) -> HashSet<String> {
    dom.descendants(body, Some(&W::p()))
        .into_iter()
        .filter(|&p| {
            dom.element(p, &W::p_pr())
                .and_then(|ppr| dom.element(ppr, &W::jc_el()))
                .is_none()
        })
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect()
        })
        .collect()
}

/// The text `p` holds once accepted: its revised text after that of the
/// paragraphs before it whose deleted marks merge them into `p`.
pub(super) fn accepted_text(dom: &Dom, kids: &[NodeId], p: NodeId) -> String {
    let Some(at) = kids.iter().position(|&k| k == p) else {
        return revised_text(dom, p);
    };
    let first = kids[..at]
        .iter()
        .rposition(|&k| !dom.name_is(k, &W::p()) || !para_mark_revision(dom, k, &W::del()))
        .map_or(0, |i| i + 1);
    kids[first..=at]
        .iter()
        .map(|&k| revised_text(dom, k))
        .collect()
}

/// A redline paragraph's revised text: its live and inserted `w:t`.
pub(super) fn revised_text(dom: &Dom, p: NodeId) -> String {
    dom.descendants(p, Some(&W::t()))
        .into_iter()
        .filter(|&t| {
            !dom.ancestors(t, None)
                .into_iter()
                .take_while(|&a| a != p)
                .any(|a| dom.name_is(a, &W::del()))
        })
        .map(|t| dom.value(t))
        .collect()
}

/// Each paragraph's text and `w:jc` value, for paragraphs with a `w:jc`.
/// `(paragraph_text, w:jc_val)` for every ORIGINAL paragraph that
/// sets an explicit `w:jc`.
///
/// Used as the M454 gate ([`ensure_empty_pprchange_on_eq_with_live_jc`]):
/// an `Equal` paragraph with live `jc` whose source-original text
/// was `jc`-set gets the empty `pPrChange` shell — Word keeps the
/// history even when accept would produce nothing different.
pub fn paragraph_alignments(dom: &Dom, body: NodeId) -> HashSet<(String, String)> {
    dom.descendants(body, Some(&W::p()))
        .into_iter()
        .filter_map(|p| {
            let jc = dom.element(dom.element(p, &W::p_pr())?, &W::jc_el())?;
            let text: String = dom
                .descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect();
            Some((text, dom.attribute(jc, &W::val()).unwrap_or("").to_string()))
        })
        .collect()
}

/// M454 (center_alignment_2 residual ~87 → 100):
/// Word EQ title keeps live `jc` + empty `pPrChange`. Engine had live jc only.
/// The title gained its `jc=center` in the revision; a paragraph the
/// original already aligned the same way is unchanged, and Word records no
/// property change on it (fixtures_500 00b81efae883's empty justified
/// paragraphs), so `original` — the original's (text, jc) pairs — gates it.
///
/// Gate: **EQ only** (no ins/del content or marks) — pure-I empty shells
/// thrash'd comments subset (−6). No MIX (M451). No rPr mark.
/// M454 — synthesize a layout-only `w:pPrChange` shell under an
/// `Equal` paragraph with live `w:jc` whose ORIGINAL paragraph also
/// set `jc`.
///
/// `center2 title` → 100: the Equal paragraph reads `jc` live but
/// the original revision history should still exist in
/// `pPrChange`. Pure-I empty `pPrChange` shells thrashed the
/// comments subset, so this pass is restricted to `EQ` paragraphs
/// only and gates on [`paragraph_alignments`] for the original
/// `jc` provenance.
pub fn ensure_empty_pprchange_on_eq_with_live_jc(
    dom: &mut Dom,
    root: NodeId,
    original: &HashSet<(String, String)>,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
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
        if para_is_mixed_revision(dom, p)
            || para_is_pure_inserted(dom, p)
            || para_is_pure_deleted(dom, p)
        {
            continue;
        }
        // Strict EQ: no ins/del content or para-mark revision.
        if !dom.descendants(p, Some(&W::ins())).is_empty()
            || !dom.descendants(p, Some(&W::del())).is_empty()
            || para_mark_revision(dom, p, &W::ins())
            || para_mark_revision(dom, p, &W::del())
        {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        let Some(jc) = dom.element(ppr, &W::jc_el()) else {
            continue;
        };
        let text: String = dom
            .descendants(p, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect();
        let jc = dom.attribute(jc, &W::val()).unwrap_or("").to_string();
        if original.contains(&(text, jc)) {
            continue;
        }
        if dom.element(ppr, &W::p_pr_change()).is_some() {
            continue;
        }
        if dom.element(ppr, &W::r_pr()).is_some() {
            continue;
        }
        let old_inner = dom.new_element(W::p_pr());
        let chg = dom.new_element(W::p_pr_change());
        dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
        *id_gen += 1;
        dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
        dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
        dom.add(chg, old_inner);
        dom.add(ppr, chg);
    }
}

/// M458 (right_aligned_italic residual ~88 after M151 peel):
/// M151 pure-I's B1 ("This document…") then free-meshes A1×B2, which leaves an
/// **early del "This"** on the body MIX (often after a short leading ins
/// "All") that Word does not show. Surgical: if previous para is pure-I
/// starting with the same first alnum token, drop the first short del among
/// the first three content children when it only echoes that token.
/// M458 — strip a leading `w:del` echo on a body MIX paragraph when
/// its text equals the previous pure-I paragraph's text.
///
/// M151 sometimes leaves a pure-I B1 + A1×B2 trailing run on a pure-I
/// followed by a MIX whose leading run deletes "This" (a sentence
/// Word keeps as `Equal` on the body). Strip the orphan leading del;
/// the trailing ins has the same `This` as Equal already.
pub fn strip_leading_del_echoing_prev_pure_i(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    for i in 1..kids.len() {
        let prev = kids[i - 1];
        let p = kids[i];
        if !dom.name_is(prev, &W::p()) || !dom.name_is(p, &W::p()) {
            continue;
        }
        if !para_is_pure_inserted(dom, prev) || !para_is_mixed_revision(dom, p) {
            continue;
        }
        // A deleted pilcrow still owns this original paragraph. Its leading
        // deletion is source content, even when the previous B-only paragraph
        // repeats that word. Dropping it would change rejection (compound
        // boundary moves such as alpha-copper -> alpha / copper).
        if para_mark_revision(dom, p, &W::del()) {
            continue;
        }
        let prev_text = para_revision_body_text(dom, prev);
        let prev_tok = prev_text
            .split(|c: char| !c.is_alphanumeric())
            .find(|t| !t.is_empty())
            .map(|t| t.to_ascii_lowercase());
        let Some(prev_tok) = prev_tok else {
            continue;
        };
        let content: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| !dom.name_is(c, &W::p_pr()))
            .collect();
        if content.len() < 2 {
            continue;
        }
        // Scan first 3 content children for a short del that echoes prev_tok.
        let mut target: Option<NodeId> = None;
        for &c in content.iter().take(3) {
            if !dom.name_is(c, &W::del()) {
                continue;
            }
            let mut n_tok = 0usize;
            let mut first_tok: Option<String> = None;
            for t in dom.descendants(c, Some(&W::del_text())) {
                for part in dom.value_str(t).split(|ch: char| !ch.is_alphanumeric()) {
                    if part.is_empty() {
                        continue;
                    }
                    n_tok += 1;
                    if first_tok.is_none() {
                        first_tok = Some(part.to_ascii_lowercase());
                    }
                }
            }
            for t in dom.descendants(c, Some(&W::t())) {
                for part in dom.value_str(t).split(|ch: char| !ch.is_alphanumeric()) {
                    if part.is_empty() {
                        continue;
                    }
                    n_tok += 1;
                    if first_tok.is_none() {
                        first_tok = Some(part.to_ascii_lowercase());
                    }
                }
            }
            let Some(ft) = first_tok else {
                continue;
            };
            if ft == prev_tok && (1..=2).contains(&n_tok) {
                target = Some(c);
                break;
            }
        }
        let Some(del_node) = target else {
            continue;
        };
        // Leave other content.
        let other = content.iter().any(|&c| c != del_node);
        if !other {
            continue;
        }
        dom.remove(del_node);
    }
}
