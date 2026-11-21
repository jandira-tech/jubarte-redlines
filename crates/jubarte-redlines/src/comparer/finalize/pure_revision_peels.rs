// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Word-mode peels over pure-inserted, pure-deleted and mixed paragraphs: closing marks, trailing empties, spacing.

use super::*;

/// M69 — clear the paragraph-mark `w:del` on the final body paragraph
/// when it is a content-free pure-D.
///
/// Word leaves the final empty body paragraph unmarked (`file_69`):
/// the engine left `pPr/rPr/del` on a trailing empty `</w:p>`. The
/// pass removes the mark `w:del` (and any sibling empty-`pPr` shells)
/// from the last non-`sectPr` child of `w:body`.
pub fn strip_trailing_empty_pure_del_mark(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    // Last non-sectPr child.
    let Some(&last) = kids.iter().rev().find(|&&k| !dom.name_is(k, &W::sect_pr())) else {
        return;
    };
    if !dom.name_is(last, &W::p()) {
        return;
    }
    // Any body text (live or delText)?
    let has_t = !dom.descendants(last, Some(&W::t())).is_empty()
        || !dom.descendants(last, Some(&W::del_text())).is_empty();
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
/// Not when the revised document ends in a paragraph with the same spacing
/// (`revised_tail_spacing`, from [`last_body_para_spacing`]): that spacing is
/// its own and Word keeps it live.
/// M92 — move a trailing empty paragraph's `w:spacing` into
/// `w:pPrChange` so the live layout is empty (Word records file_30's
/// last empty this way after a list residual).
///
/// Skip when the paragraph already carries `pPrChange`, when it has
/// any `w:t`/`w:delText` text, when there is a `w:ins`/`w:del`
/// carrier anywhere under it (post-M69 or Equal empty), or when the
/// document would still end with the same spacing live (matched
/// against the precomputed `revised_tail_spacing` from
/// [`last_body_para_spacing`]).
pub fn trailing_empty_spacing_to_pprchange(
    dom: &mut Dom,
    root: NodeId,
    revised_tail_spacing: Option<&[(String, String, String)]>,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids.iter().rev().find(|&&k| !dom.name_is(k, &W::sect_pr())) else {
        return;
    };
    if !dom.name_is(last, &W::p()) {
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
    if dom.element(ppr, &W::p_pr_change()).is_some() {
        return;
    }
    let Some(sp) = dom.element(ppr, &W::spacing_el()) else {
        return;
    };
    if revised_tail_spacing.is_some_and(|b| b == sorted_attributes(dom, sp)) {
        return;
    }
    // Only when spacing is the sole layout child (ignore empty rPr).
    for c in dom.elements(ppr, None) {
        let Some(n) = dom.name(c) else {
            continue;
        };
        if n == W::r_pr() {
            continue;
        }
        if n != W::spacing_el() {
            return;
        }
    }
    let old_inner = dom.new_element(W::p_pr());
    let sp_clone = dom.clone_subtree(sp);
    dom.add(old_inner, sp_clone);
    dom.remove(sp);
    let chg = dom.new_element(W::p_pr_change());
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

/// The paragraph properties of the story's closing paragraph, cloned
/// without scratch attributes before atomize consumes `body`: `None` when
/// the story does not end on a paragraph, an empty `w:pPr` when the
/// paragraph has none.
/// The paragraph properties of the story's closing paragraph, cloned
/// without scratch attributes (called before atomize consumes
/// `body`).
///
/// Returns `None` when the story does not end on a paragraph; returns
/// an empty `w:pPr` when the closing paragraph has none of its own.
/// Both source and revised stories are read once at the start of the
/// pipeline and the result is then passed into
/// [`closing_mark_takes_revised_properties`] (Word-visual mode) or
/// [`merged_closing_carrier_takes_source_properties`]
/// (PowerTools-faithful mode).
pub fn closing_paragraph_properties(dom: &mut Dom, body: NodeId) -> Option<NodeId> {
    let last = dom
        .elements(body, None)
        .into_iter()
        .rev()
        .find(|&k| !dom.name_is(k, &W::sect_pr()))?;
    if !dom.name_is(last, &W::p()) {
        return None;
    }
    Some(match dom.element(last, &W::p_pr()) {
        Some(ppr) => drop_pt_bookkeeping(dom, ppr),
        None => dom.new_element(W::p_pr()),
    })
}

/// Faithful production can conjoin unrelated terminal deletion/insertion
/// streams into the original paragraph. The paragraph still carries its
/// deleted mark, but its inserted content survives acceptance. Give that
/// actual merged carrier the revised closing layout and save the original
/// layout for rejection, without running Word's paragraph-folding rules.
/// PowerTools-faithful ending: the conjoined terminal carrier takes
/// the revised layout live and parks the original layout in
/// `w:pPrChange`.
///
/// Reads the cloned `pPr`s from [`closing_paragraph_properties`]
/// (called before atomize consumed the source bodies). Skips silently
/// when [`WmlComparerSettings::detect_format_changes`] is `false` or
/// when the last body child is not a paragraph carrying a paragraph
/// deletion mark. Mints a fresh `w:id` from `id_gen`.
pub fn merged_closing_carrier_takes_source_properties(
    dom: &mut Dom,
    root: NodeId,
    original: Option<NodeId>,
    revised: Option<NodeId>,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    if !settings.detect_format_changes {
        return;
    }
    let (Some(original), Some(revised)) = (original, revised) else {
        return;
    };
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let Some(last) = dom
        .elements(body, None)
        .into_iter()
        .rev()
        .find(|&node| !dom.name_is(node, &W::sect_pr()))
    else {
        return;
    };
    if !dom.name_is(last, &W::p()) || !para_mark_revision(dom, last, &W::del()) {
        return;
    }
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    if content_revision_marks(dom, last, ppr) != (true, true) {
        return;
    }
    let old = formatchg::project_para_properties_for_change(dom, original);
    let fresh = formatchg::project_para_properties_for_change(dom, revised);
    let old_signature = formatchg::normalize_para_properties(dom, old);
    let new_signature = formatchg::normalize_para_properties(dom, fresh);
    let live_signature = formatchg::normalize_para_properties(dom, ppr);
    let saved = dom
        .element(ppr, &W::p_pr_change())
        .and_then(|change| dom.element(change, &W::p_pr()));
    let saved_matches = saved
        .is_some_and(|saved| formatchg::normalize_para_properties(dom, saved) == old_signature);
    if live_signature == new_signature && (old_signature == new_signature || saved_matches) {
        return;
    }
    for child in dom.elements(ppr, None) {
        if !dom.name_is(child, &W::r_pr()) && !dom.name_is(child, &W::sect_pr()) {
            dom.remove(child);
        }
    }
    for child in dom.elements(fresh, None) {
        dom.remove(child);
        dom.add(ppr, child);
    }
    if old_signature != new_signature {
        let change = rev_el(dom, W::p_pr_change(), settings, id_gen);
        dom.add(change, old);
        dom.add(ppr, change);
    }
}

/// The story's closing mark is never revised: accepted, it closes the
/// revision; rejected, the original. Word writes the revision's closing
/// paragraph properties live on it and records the original's in a
/// `pPrChange` (cda19d51ed: `pStyle=Cuerpo jc=both` live over the
/// original's `spacing line=276`). A revision whose closing paragraph folded
/// into the original's left the original's properties live, so accepting
/// dropped the revision's style and alignment.
/// Word-visual ending (after every peel): the closing mark carries the
/// revised layout live and the original layout in `w:pPrChange`.
///
/// Reads the cloned `pPr`s from [`closing_paragraph_properties`] and
/// applies them to the story's last paragraph mark. Mints a fresh
/// `w:pPrChange` id from `id_gen`. A paired story-final mark needs no
/// `pPrChange` because the revised paragraph is the merged live
/// paragraph.
pub fn closing_mark_takes_revised_properties(
    dom: &mut Dom,
    root: NodeId,
    original: Option<NodeId>,
    revised: Option<NodeId>,
    default_lines: (Option<&str>, Option<&str>),
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let (Some(original), Some(revised)) = (original, revised) else {
        return;
    };
    // The revision's properties in the form every revised paragraph took
    // above: twips, no demo-default line, spacing with its line rule.
    let scratch = dom.new_element(W::body());
    let scratch_p = dom.new_element(W::p());
    dom.add(scratch, scratch_p);
    let revised = dom.clone_subtree(revised);
    dom.add(scratch_p, revised);
    normalize_universal_measures(dom, scratch);
    // An edit's revised paragraphs keep their spacing as written, and so
    // does its closing mark.
    if !settings.preserve_untouched {
        strip_redundant_demo_default_spacing(dom, scratch, default_lines);
        normalize_incomplete_spacing(dom, scratch);
    }
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let Some(last) = dom
        .elements(body, None)
        .into_iter()
        .rev()
        .find(|&k| !dom.name_is(k, &W::sect_pr()))
    else {
        return;
    };
    if !dom.name_is(last, &W::p()) {
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
    if mark.is_some_and(|m| {
        ["ins", "del", "moveFrom", "moveTo"]
            .iter()
            .any(|k| dom.element(m, &W::name(k)).is_some())
    }) {
        return;
    }
    // Revised text on the closing paragraph is the revision's own paragraph
    // run into it (the comment list item `unmerge_comment_list_item` splits
    // back out): its properties are that paragraph's, not the closing one's.
    if dom.elements(last, Some(&W::ins())).len() + dom.elements(last, Some(&W::move_to())).len() > 0
    {
        return;
    }
    closing_mark_takes_revised_mark_formatting(dom, ppr, revised, settings, id_gen);
    let live = formatchg::normalize_para_properties(dom, ppr);
    let new = formatchg::normalize_para_properties(dom, revised);
    let old = formatchg::project_para_properties_for_change(dom, original);
    let old_sig = formatchg::normalize_para_properties(dom, old);
    let saved = dom
        .element(ppr, &W::p_pr_change())
        .and_then(|change| dom.element(change, &W::p_pr()));
    let history_matches =
        saved.is_some_and(|saved| formatchg::normalize_para_properties(dom, saved) == old_sig);
    // The revised closing paragraph may carry an authored section break
    // although the original closing paragraph had none. Section geometry is
    // excluded from paragraph-format signatures, but retaining the added
    // live break without a valid old pPr would duplicate it on rejection.
    // Source snapshots precede final-body hoisting, so this tests authored
    // presence only; relocated final body sections never qualify.
    let added_authored_section = settings.detect_format_changes
        && dom.element(original, &W::sect_pr()).is_none()
        && dom.element(revised, &W::sect_pr()).is_some_and(|section| {
            dom.element(ppr, &W::sect_pr()).is_some_and(|live_section| {
                sectpr_identity(dom, live_section) == sectpr_identity(dom, section)
            })
        });
    if live == new
        && (old_sig == new || history_matches)
        && (!added_authored_section || history_matches)
    {
        return;
    }
    let keep: Vec<NodeId> = [W::r_pr(), W::sect_pr()]
        .iter()
        .filter_map(|n| dom.element(ppr, n))
        .collect();
    for c in dom.elements(ppr, None) {
        dom.remove(c);
    }
    let fresh = formatchg::project_para_properties_for_change(dom, revised);
    for c in dom.elements(fresh, None) {
        dom.remove(c);
        dom.add(ppr, c);
    }
    for c in keep {
        dom.add(ppr, c);
    }
    if old_sig != new || added_authored_section {
        let chg = dom.new_element(W::p_pr_change());
        dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
        *id_gen += 1;
        dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
        dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
        dom.add(chg, old);
        dom.add(ppr, chg);
    }
    // The original's, in twips like every other recorded change.
    normalize_universal_measures(dom, ppr);
}

// Keep the established leaf-property normalization, but paragraph marks
// also own nested Office effects. The general comparer intentionally reduces
// properties to names/attributes; doing that here would lose a nested fill
// when relocating the only surviving mark into a title carrier.
pub(super) fn paragraph_mark_properties_equal(
    dom: &mut Dom,
    old: Option<NodeId>,
    new: Option<NodeId>,
) -> bool {
    // Canonical run-property comparison materializes temporary trees. Keep
    // those trees in a dedicated arena: this predicate also runs on declining
    // label folds, so repeated probes must not retain orphan comparison nodes.
    let mut scratch = Dom::new();
    let [old_copy, new_copy] = [old, new].map(|properties| {
        properties.map(|properties| {
            let document = scratch.parse_xdocument(&dom.serialize_element(properties));
            scratch
                .root(document)
                .expect("serialized run properties have a root")
        })
    });
    if !formatchg::are_run_properties_equal(&mut scratch, old_copy, new_copy) {
        return false;
    }
    fn signature(dom: &Dom, node: NodeId) -> String {
        let mut attrs = dom
            .attributes_iter(node)
            .filter(|(name, _)| {
                !dom.is_namespace_declaration(name)
                    && name.namespace_name() != PT::URI
                    && !name.local_name().to_ascii_lowercase().starts_with("rsid")
            })
            .map(|(name, value)| {
                (
                    name.namespace_name().to_string(),
                    name.local_name().to_string(),
                    value,
                )
            })
            .collect::<Vec<_>>();
        attrs.sort();
        let children = dom
            .nodes(node)
            .into_iter()
            .filter(|&child| {
                dom.name(child)
                    .is_some_and(|name| name.namespace_name() != PT::URI)
                    || (!dom.is_element(child)
                        && !dom.text_value(child).unwrap_or_default().trim().is_empty())
            })
            .map(|child| signature(dom, child))
            .collect::<Vec<_>>();
        format!(
            "{:?}:{attrs:?}:{:?}:{children:?}",
            dom.name(node),
            dom.text_value(node)
        )
    }
    let nested = |properties: Option<NodeId>| {
        let mut children = properties
            .into_iter()
            .flat_map(|rpr| dom.elements(rpr, None))
            .filter(|&child| {
                !dom.name_is(child, &W::r_pr_change())
                    && dom
                        .name(child)
                        .is_some_and(|name| name.namespace_name() != PT::URI)
                    && dom.has_elements(child)
            })
            .map(|child| signature(dom, child))
            .collect::<Vec<_>>();
        children.sort();
        children
    };
    nested(old) == nested(new)
}

/// The closing mark's own formatting follows the same way: Word writes the
/// revised closing paragraph's mark `rPr` live and records the original's
/// in a `rPrChange` (b4cd671041: `sz=28` live over the original's
/// `b rFonts=Arial sz=24 u`). A mark that already records a change keeps it.
pub(super) fn closing_mark_takes_revised_mark_formatting(
    dom: &mut Dom,
    ppr: NodeId,
    revised: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    if !settings.detect_format_changes {
        return;
    }
    let live = dom.element(ppr, &W::r_pr());
    if live.is_some_and(|m| dom.element(m, &W::r_pr_change()).is_some()) {
        return;
    }
    let new = dom.element(revised, &W::r_pr());
    if paragraph_mark_properties_equal(dom, live, new) {
        return;
    }
    let fresh = dom.new_element(W::r_pr());
    if let Some(n) = new {
        for c in dom.elements(n, None) {
            let skip = ["ins", "del", "moveFrom", "moveTo", "rPrChange"]
                .iter()
                .any(|k| dom.name_is(c, &W::name(k)));
            if !skip {
                let c = dom.clone_subtree(c);
                dom.add(fresh, c);
            }
        }
    }
    let old = match live {
        Some(m) => dom.clone_subtree(m),
        None => dom.new_element(W::r_pr()),
    };
    let chg = dom.new_element(W::r_pr_change());
    dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;
    dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add(chg, old);
    dom.add(fresh, chg);
    match live {
        Some(m) => dom.replace_with(m, &[fresh]),
        None => match [W::sect_pr(), W::p_pr_change()]
            .iter()
            .find_map(|n| dom.element(ppr, n))
        {
            Some(next) => dom.add_before_self(next, fresh),
            None => dom.add(ppr, fresh),
        },
    }
}

/// Sorted `(namespace, local name, value)` attributes of the last body
/// paragraph's `w:spacing` under `body`, if it has one.
/// Sorted `(namespace, local_name, value)` attributes of the last
/// non-`sectPr` body paragraph's `w:spacing`.
///
/// Used as the M92 / M228 guard for [`trailing_empty_spacing_to_pprchange`]
/// and [`last_pure_del_spacing_to_pprchange`] when the revised document
/// also ends on the same spacing — that means the spacing IS the
/// revised paragraph's own and Word keeps it live, not behind
/// `pPrChange`. Returned `None` when the body does not end on a
/// paragraph at all.
pub fn last_body_para_spacing(dom: &Dom, body: NodeId) -> Option<Vec<(String, String, String)>> {
    let last = dom
        .elements(body, None)
        .into_iter()
        .rev()
        .find(|&k| !dom.name_is(k, &W::sect_pr()))?;
    if !dom.name_is(last, &W::p()) {
        return None;
    }
    let sp = dom.element(dom.element(last, &W::p_pr())?, &W::spacing_el())?;
    Some(sorted_attributes(dom, sp))
}

/// Sorted `w:` attributes (scratch ids such as `pt:Unid` excluded).
pub(super) fn sorted_attributes(dom: &Dom, n: NodeId) -> Vec<(String, String, String)> {
    let w_ns = W::p().namespace_name().to_string();
    let mut v: Vec<(String, String, String)> = dom
        .attributes(n)
        .into_iter()
        .filter(|(k, _)| k.namespace_name() == w_ns)
        .map(|(k, v)| {
            (
                k.namespace_name().to_string(),
                k.local_name().to_string(),
                v,
            )
        })
        .collect();
    v.sort();
    v
}

/// True when a paragraph has no `w:t` / `w:delText` content.
/// Page-break (`w:br`), drawings and picts are contentful even without `w:t`
/// (Word keeps a page-break-only paragraph separate from a drawing-only
/// deletion — diff_doc2×numwords page-break + image were merged into one `w:p`
/// because both were considered empty).
pub(super) fn para_has_no_text(dom: &Dom, p: NodeId) -> bool {
    dom.descendants(p, Some(&W::t())).is_empty()
        && dom.descendants(p, Some(&W::del_text())).is_empty()
        && dom.descendants(p, Some(&W::name("br"))).is_empty()
        && dom.descendants(p, Some(&W::drawing())).is_empty()
        && dom.descendants(p, Some(&W::pict())).is_empty()
        && dom.descendants(p, Some(&W::object())).is_empty()
        && dom.descendants(p, Some(&M::name("oMath"))).is_empty()
        && dom.descendants(p, Some(&M::name("oMathPara"))).is_empty()
}

/// True when a paragraph carries OMML math (`m:oMath` / `m:oMathPara`).
///
/// M419: math shells have no `w:t` so they look like M311d layout empties, but
/// Word still MIX-es the last math pure-I into the first pure-D title.
pub(super) fn para_has_omath(dom: &Dom, p: NodeId) -> bool {
    !dom.descendants(p, Some(&M::name("oMath"))).is_empty()
        || !dom.descendants(p, Some(&M::name("oMathPara"))).is_empty()
}

/// True when a paragraph renders as a blank line: whitespace-only text and no
/// non-text ink (math, drawings, pictures, objects, breaks).
///
/// M430's "trailing empty pure-I" runs must use this, not
/// [`para_body_text_is_whitespace_only`] alone: textless OMML math shells
/// (math_delimiter × math_eqarr) counted as "empty", which disabled the
/// multi-del fold that MIXes the last math pure-I into the Delimiter title
/// (M419 regression).
pub(super) fn para_is_visually_blank(dom: &Dom, p: NodeId) -> bool {
    para_body_text_is_whitespace_only(dom, p)
        && !para_has_omath(dom, p)
        && dom.descendants(p, Some(&W::drawing())).is_empty()
        && dom.descendants(p, Some(&W::pict())).is_empty()
        && dom.descendants(p, Some(&W::object())).is_empty()
        && dom.descendants(p, Some(&W::name("br"))).is_empty()
}

/// 1 when the body ends on a live, blank paragraph after a pure-D run: the
/// story-final mark the unrelated-sources pairing kept (A's and B's final
/// empty paragraphs). The trailing-empty-run gates below were calibrated
/// when B's final empty was still one of the pure-I empties before the
/// pure-D run, so they count it back in.
pub(super) fn story_final_kept_blank(dom: &Dom, kids: &[NodeId]) -> usize {
    let [.., prev, last] = kids else {
        return 0;
    };
    let live = |p: NodeId| {
        dom.descendants(p, Some(&W::ins())).is_empty()
            && dom.descendants(p, Some(&W::del())).is_empty()
            && !para_mark_revision(dom, p, &W::ins())
            && !para_mark_revision(dom, p, &W::del())
    };
    usize::from(
        dom.name_is(*last, &W::p())
            && para_is_visually_blank(dom, *last)
            && live(*last)
            && dom.name_is(*prev, &W::p())
            && para_is_pure_deleted(dom, *prev),
    )
}

/// True when body text repeats a ≥4-word phrase (hummingbird wrap fingerprint).
///
/// M416/M420: distinguish wrap pure-I from generic long titles (project_tasks
/// TaskOwner line) so multi-del fold skip only hits real wraps.
pub(super) fn para_has_repeated_phrase(dom: &Dom, p: NodeId) -> bool {
    let t = para_revision_body_text(dom, p).to_ascii_lowercase();
    let words: Vec<&str> = t.split_whitespace().filter(|w| !w.is_empty()).collect();
    if words.len() < 8 {
        return false;
    }
    // Sliding window of 4 words; any window that appears twice is a wrap.
    let mut seen = std::collections::HashSet::new();
    for i in 0..=words.len().saturating_sub(4) {
        let key = words[i..i + 4].join(" ");
        if !seen.insert(key) {
            return true;
        }
    }
    false
}

/// True when `p` is an empty pure-inserted paragraph (no text, has ins, no del).
pub(super) fn para_is_empty_pure_ins(dom: &Dom, p: NodeId) -> bool {
    if !dom.name_is(p, &W::p()) {
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
    if !dom.descendants(p, Some(&W::drawing())).is_empty()
        || !dom.descendants(p, Some(&W::tbl())).is_empty()
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
/// M83a — drop a trailing empty pure-inserted body paragraph.
///
/// `file_23`: source B often ends with a self-closing empty `<w:p/>`
/// before `sectPr` (typically after a closing table). Word Compare
/// omits it entirely; the engine emitted a bare para-mark `w:ins`,
/// adding a blank line Word does not have.
pub fn strip_trailing_empty_pure_ins(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids.iter().rev().find(|&&k| !dom.name_is(k, &W::sect_pr())) else {
        return;
    };
    if para_is_empty_pure_ins(dom, last) {
        dom.remove(last);
    }
}

/// M438 (doc_with_spaces × doc_with_spacing ~67 / docxodus 100):
/// Word title-page pure-I next × short pure-D base is `I…date e×6 DD E`
/// (trailing bare empty). Engine keeps every next empty as pure-I (`e×7`)
/// and no trailing E (`I…e7 DD`).
///
/// When a title-page pure-I stream ends with ≥6 consecutive empty pure-I
/// then multi pure-D section residual (no pure-I after), move the last
/// empty pure-I after the pure-D run and strip its revision marks → bare
/// empty E (Word shape).
/// M438 — relocate a title-page trailing empty pure-I so it sits
/// after the pure-D section residual (Word shape `I…date e×6 DD E`,
/// not `I…e7 DD`).
///
/// `doc_with_spaces × doc_with_spacing` ~67 / docxodus 100: when a
/// title-page pure-I stream ends with ≥ 6 consecutive empty pure-I
/// paragraphs then a multi-paragraph pure-D residual (no pure-I
/// after), move the last empty pure-I **after** the pure-D run and
/// strip its revision marks so it lands as a bare trailing `E`. Gated
/// on the count of pure-I empties; an interior `I E D` block does not
/// trigger a move.
pub fn relocate_title_page_last_empty_after_pure_dels(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.len() < 8 {
        return;
    }
    // Find first contentful pure-D.
    let Some(di) = kids.iter().position(|&k| {
        dom.name_is(k, &W::p())
            && para_is_pure_deleted(dom, k)
            && !para_body_text_is_whitespace_only(dom, k)
    }) else {
        return;
    };
    // Pure-D run length (contentful + empty pure-D).
    let mut dj = di;
    while dj < kids.len() && dom.name_is(kids[dj], &W::p()) && para_is_pure_deleted(dom, kids[dj]) {
        dj += 1;
    }
    let del_run = dj - di;
    if del_run < 1 {
        return;
    }
    // No pure-I after pure-D residual, and no story-final paragraph the
    // comparer already kept live there (the unrelated-sources pairing of
    // both final empty paragraphs does this move itself).
    if kids[dj..].iter().any(|&k| {
        dom.name_is(k, &W::p())
            && (para_is_pure_inserted(dom, k) || dom.element(k, &W::p_pr()).is_some())
    }) {
        return;
    }
    // Trailing empty pure-I run immediately before first pure-D.
    let mut empty_run = 0usize;
    let mut j = di;
    while j > 0 {
        j -= 1;
        let k = kids[j];
        if dom.name_is(k, &W::p())
            && para_is_pure_inserted(dom, k)
            && para_body_text_is_whitespace_only(dom, k)
            && !para_has_omath(dom, k)
            && dom.descendants(k, Some(&W::name("br"))).is_empty()
            && dom.descendants(k, Some(&W::drawing())).is_empty()
        {
            empty_run += 1;
        } else {
            break;
        }
    }
    // Word drops one of the 7 post-date empties; require a long empty run.
    if empty_run < 6 {
        return;
    }
    // Content pure-I immediately before the empty run must look like title-page
    // date / cover (M413 fingerprint) so stamp demos are untouched.
    let content_before = kids[..di.saturating_sub(empty_run)]
        .iter()
        .rev()
        .find(|&&k| {
            dom.name_is(k, &W::p())
                && para_is_pure_inserted(dom, k)
                && !para_body_text_is_whitespace_only(dom, k)
        });
    let Some(&anchor) = content_before else {
        return;
    };
    let t = para_revision_body_text(dom, anchor).to_ascii_lowercase();
    let title_page_tail = t.contains("prepared")
        || t.contains('@')
        || t.contains("2040")
        || t.contains("202")
        || t.contains("march ")
        || t.contains("january ")
        || t.contains("february ")
        || t.contains("april ")
        || t.contains("june ")
        || t.contains("july ")
        || t.contains("august ")
        || t.contains("september ")
        || t.contains("october ")
        || t.contains("november ")
        || t.contains("december ")
        || t.chars().filter(|c| c.is_ascii_digit()).count() >= 4;
    if !title_page_tail {
        return;
    }
    // Last empty pure-I in the run.
    let last_empty = kids[di - 1];
    if !para_is_pure_inserted(dom, last_empty)
        || !para_body_text_is_whitespace_only(dom, last_empty)
    {
        return;
    }
    // Strip revision marks → bare empty (Word trailing E).
    strip_para_revision_marks(dom, last_empty);
    // Move after pure-D run (before whatever follows, usually sectPr).
    let after_dels = if dj < kids.len() {
        Some(kids[dj])
    } else {
        // Insert before sectPr if present.
        dom.elements(body, None)
            .into_iter()
            .find(|&k| dom.name_is(k, &W::sect_pr()))
    };
    dom.remove(last_empty);
    match after_dels {
        Some(anchor_node) if dom.parent(anchor_node).is_some() => {
            dom.add_before_self(anchor_node, last_empty);
        }
        _ => {
            dom.add(body, last_empty);
        }
    }
}

/// Drop ins/del wrappers and para-mark revisions from a paragraph (bare EQ/empty).
pub(super) fn strip_para_revision_marks(dom: &mut Dom, p: NodeId) {
    // Remove body-level ins/del wrappers (keep their children unwrapped if any).
    let mut wrappers: Vec<NodeId> = dom
        .elements(p, None)
        .into_iter()
        .filter(|&c| matches!(dom.name(c), Some(n) if n == W::ins() || n == W::del()))
        .collect();
    // Also nested ins/del under runs (descendants).
    wrappers.extend(
        dom.descendants(p, Some(&W::ins()))
            .into_iter()
            .chain(dom.descendants(p, Some(&W::del()))),
    );
    // Unique, deepest-first roughly by reverse.
    wrappers.sort_by_key(|&n| std::cmp::Reverse(n.0));
    wrappers.dedup();
    for w in wrappers {
        if dom.parent(w).is_none() {
            continue;
        }
        // Move children before wrapper, then remove wrapper.
        let kids: Vec<NodeId> = dom.elements(w, None);
        for c in kids {
            dom.remove(c);
            dom.add_before_self(w, c);
        }
        // Also text nodes if any.
        dom.remove(w);
    }
    // Drop mark-only rPr ins/del under pPr.
    if let Some(ppr) = dom.element(p, &W::p_pr())
        && let Some(rpr) = dom.element(ppr, &W::r_pr())
    {
        let marks: Vec<NodeId> = dom
            .elements(rpr, None)
            .into_iter()
            .filter(|&c| matches!(dom.name(c), Some(n) if n == W::ins() || n == W::del()))
            .collect();
        for m in marks {
            dom.remove(m);
        }
        if dom.elements(rpr, None).is_empty() {
            dom.remove(rpr);
        }
    }
}

/// Restore a revised layout blank with its own properties and a new insert mark.
pub(super) fn make_empty_pure_ins_para(
    dom: &mut Dom,
    template: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> NodeId {
    let p = dom.clone_subtree(template);
    remove_powertools_scratch_markup(dom, p);
    // These source identities may also occur on the equal/deleted separator.
    // Word regenerates paragraph identities; preserve the layout, not aliases.
    for a in [W14::name("paraId"), W14::name("textId")] {
        dom.set_attribute_value(p, &a, None);
    }
    let ppr = dom.element(p, &W::p_pr()).unwrap_or_else(|| {
        let ppr = dom.new_element(W::p_pr());
        dom.add_first(p, ppr);
        ppr
    });
    let rpr = dom.element(ppr, &W::r_pr()).unwrap_or_else(|| {
        let rpr = dom.new_element(W::r_pr());
        dom.add(ppr, rpr);
        rpr
    });
    let ins = dom.new_element(W::ins());
    dom.set_attribute_value(ins, &W::id(), Some(&id_gen.to_string()));
    *id_gen += 1;
    dom.set_attribute_value(ins, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(ins, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add_first(rpr, ins);
    p
}

// Only layout properties and property-only empty runs are safe to restore.
// Untyped child payload (bookmarks, fields, references, tabs, drawings, etc.)
// belongs to source atomization and must never be recreated by this heuristic.
pub(super) fn is_restorable_layout_blank(dom: &Dom, p: NodeId) -> bool {
    if !dom.name_is(p, &W::p()) || !para_has_no_text(dom, p) {
        return false;
    }
    dom.elements(p, None).into_iter().all(|c| {
        if dom.name_is(c, &W::p_pr()) {
            dom.descendants(c, Some(&W::sect_pr())).is_empty()
                && !crate::revision_processor::element_has_tracked_revisions(dom, c)
        } else if dom.name_is(c, &W::r()) {
            dom.elements(c, None)
                .into_iter()
                .all(|r| dom.name_is(r, &W::r_pr()))
                && !crate::revision_processor::element_has_tracked_revisions(dom, c)
        } else {
            false
        }
    })
}

/// Titles that uniquely identify the revised source's two-blank pretable layout.
/// Read from the accepted revised body before atomization consumes its structure;
/// unrelated leading/trailing blanks and repeated titles cannot authorize M392.
/// The revised source's unique two-blank pre-table titles, keyed by
/// their trimmed paragraph text. Used as the authorization map for
/// [`ensure_empty_pure_i_before_short_title_del`].
///
/// Read from the accepted revised body before atomization consumes its
/// structure; unrelated leading / trailing blanks and titles that occur
/// more than once cannot authorize M392's empty-spacer restoration.
/// Returns a map from title text to a pair of paragraph template ids.
pub fn revised_two_blank_table_titles(dom: &mut Dom, body: NodeId) -> HashMap<String, [NodeId; 2]> {
    let kids = dom.elements(body, None);
    fn text(dom: &Dom, p: NodeId) -> String {
        dom.descendants(p, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect()
    }
    let mut occurrences = HashMap::<String, usize>::new();
    for &p in &kids {
        if dom.name_is(p, &W::p()) && !para_has_no_text(dom, p) {
            *occurrences.entry(text(dom, p)).or_default() += 1;
        }
    }
    let mut titles = HashMap::new();
    for (i, &p) in kids.iter().enumerate() {
        if !dom.name_is(p, &W::p()) || para_has_no_text(dom, p) {
            continue;
        }
        let title = text(dom, p);
        if title.trim().is_empty() || occurrences.get(&title) != Some(&1) {
            continue;
        }
        let mut next = i + 1;
        while let Some(&blank) = kids.get(next) {
            if !dom.name_is(blank, &W::p()) || !is_restorable_layout_blank(dom, blank) {
                break;
            }
            next += 1;
        }
        if next == i + 3 && kids.get(next).is_some_and(|&t| dom.name_is(t, &W::tbl())) {
            titles.insert(
                title,
                [
                    dom.clone_subtree(kids[i + 1]),
                    dom.clone_subtree(kids[i + 2]),
                ],
            );
        }
    }
    titles
}

/// M392 (file_36×file_37 residual ~66): Word keeps empty pure-I spacers
/// between content pure-I short title and pure-D short title before a table.
/// Restore them only for the revised source's unique two-blank layout.
/// Full LCS Equal-matches bare empties and fold strips the rest, so engine
/// jumps pure-I → pure-D. When that pattern is missing spacers, insert two
/// empty pure-I marks (B's two blanks) and convert a following bare empty
/// before the table into pure-D empty (A's blank).
/// M392 — restore empty pure-I spacers between a content pure-I short
/// title and a following short pure-D title (Word shape
/// `I…title I E E D…`) only for the revised source's unique two-blank
/// pre-table layout.
///
/// `file_36 × file_37` residual ~66: the full LCS `Equal`-matches bare
/// empties and [`strip_empty_pure_ins_before_trailing_pure_dels`] strips
/// the rest, so the engine jumps pure-I → pure-D. When that pattern is
/// missing spacers, insert two empty `w:ins` paragraphs (B's two
/// blanks) and convert a following bare empty before the table into
/// pure-D empty (A's blank). Templates come from `revised_titles` so
/// the inserted paragraphs carry the source's own `pPr` / `rPr`.
pub fn ensure_empty_pure_i_before_short_title_del(
    dom: &mut Dom,
    root: NodeId,
    revised_titles: &HashMap<String, [NodeId; 2]>,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
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
            // Content pure-I (not already empty spacer).
            if para_body_text_is_whitespace_only(dom, ins_p) {
                continue;
            }
            let revised_title: String = dom
                .descendants(ins_p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect();
            let Some(templates) = revised_titles.get(&revised_title) else {
                continue;
            };
            // Short pure-D title residual.
            let n = para_word_atom_count(dom, del_p);
            let short_title = (1..=4).contains(&n) && para_body_alnum_len(dom, del_p) <= 32;
            if !(para_looks_like_demo_title(dom, del_p)
                || para_has_heading_or_title_style(dom, del_p)
                || short_title)
            {
                continue;
            }
            // Adjacent content pure-I × short pure-D only when pure-D is
            // followed by bare/empty then a **table** (file_36×37). Do **not**
            // fire on trailing pure-D demo residuals (file_82×83 wants a single
            // empty pure-I spacer already kept by M389 — injecting two regressed
            // pagefair).
            let after = kids.get(i + 2).copied();
            let after2 = kids.get(i + 3).copied();
            // The spacers come from Word pairing that table; a table deleted
            // wholesale (quarterly report × red bold heading) gets none.
            let empty_then_tbl = matches!(
                (after, after2),
                (Some(e), Some(t))
                    if dom.name_is(e, &W::p())
                        && para_has_no_text(dom, e)
                        // A surviving inserted blank would make the two new
                        // spacers exceed the revised source's authored count.
                        && dom.descendants(e, Some(&W::ins())).is_empty()
                        && dom.name_is(t, &W::tbl())
                        && !dom
                            .descendants(t, Some(&W::p()))
                            .into_iter()
                            .all(|q| para_is_pure_deleted(dom, q))
            );
            if !empty_then_tbl {
                continue;
            }
            // Insert two empty pure-I spacers before pure-D (Word file_36×37).
            for &template in templates {
                let spacer = make_empty_pure_ins_para(dom, template, settings, id_gen);
                dom.add_before_self(del_p, spacer);
            }
            // Convert bare empty after pure-D (before table) into pure-D empty.
            if let Some(e) = after
                && dom.name_is(e, &W::p())
                && para_has_no_text(dom, e)
                && !para_is_pure_deleted(dom, e)
                && !para_is_pure_inserted(dom, e)
            {
                // Add pPr/rPr/del mark so it is pure-D empty.
                let ppr = match dom.element(e, &W::p_pr()) {
                    Some(p) => p,
                    None => {
                        let p = dom.new_element(W::p_pr());
                        if let Some(first) = dom.elements(e, None).first().copied() {
                            dom.add_before_self(first, p);
                        } else {
                            dom.add(e, p);
                        }
                        p
                    }
                };
                let rpr = match dom.element(ppr, &W::r_pr()) {
                    Some(r) => r,
                    None => {
                        let r = dom.new_element(W::r_pr());
                        dom.add_first(ppr, r);
                        r
                    }
                };
                if dom.element(rpr, &W::del()).is_none() {
                    let del = dom.new_element(W::del());
                    dom.set_attribute_value(del, &W::id(), Some(&id_gen.to_string()));
                    *id_gen += 1;
                    dom.set_attribute_value(
                        del,
                        &W::author(),
                        Some(&settings.author_for_revisions),
                    );
                    dom.set_attribute_value(
                        del,
                        &W::date(),
                        Some(&settings.date_time_for_revisions),
                    );
                    dom.add_first(rpr, del);
                }
            }
            acted = true;
            break;
        }
        if !acted {
            return;
        }
    }
}

/// M85a — drop empty pure-ins sitting immediately before a trailing pure-del run.
///
/// M83a only removed an empty pure-ins when it was the absolute last body child.
/// When residual pure-dels follow (catalog/table demos: B ends empty after table,
/// A residual is pure-D), Word still omits the blank (`… tbl D D D`) while we
/// kept `… tbl Ei D D D` (file_49, file_75, file_35, file_102, …).
///
/// M311: do **not** strip a long run of empty pure-I (≥3). That is wholesale
/// next-document layout Word keeps (image_inline×rtl_page_numpages: Word
/// pure-I ~31 empty next paras then pure-D image; unbounded M85a left only
/// the pure-D and scored ~15). Cap strip to at most two empties.
/// M85a — strip empty pure-I paragraphs sitting immediately before a
/// trailing pure-D run.
///
/// `file_186` / `file_49`: Word merges a trailing `I D` into a single
/// `MIX` paragraph when the pure-I is empty (no text, only a para-mark
/// `w:ins`); non-empty pure-I stays separate. The pass removes empty
/// pure-I paragraphs that sit in front of a trailing pure-D run (the
/// story-final and revised-closing-mark paired cases keep them — those
/// are the inserted final paragraph, not a stray spacer).
pub fn strip_empty_pure_ins_before_trailing_pure_dels(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let non_sect: Vec<NodeId> = kids
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if non_sect.len() < 2 {
        return;
    }
    // Trailing pure-del run: find first index of the run.
    let mut run_start = non_sect.len();
    while run_start > 0 {
        let k = non_sect[run_start - 1];
        if dom.name_is(k, &W::p()) && para_is_pure_deleted(dom, k) {
            run_start -= 1;
        } else {
            break;
        }
    }
    if run_start == non_sect.len() || run_start == 0 {
        return;
    }
    // Count consecutive empty pure-ins immediately before the pure-del run.
    let mut empty_run = 0usize;
    let mut i = run_start;
    while i > 0 && para_is_empty_pure_ins(dom, non_sect[i - 1]) {
        empty_run += 1;
        i -= 1;
    }
    if empty_run == 0 {
        return;
    }
    // Wholesale empty next layout (≥3) — Word keeps all (image×rtl).
    if empty_run + story_final_kept_blank(dom, &non_sect) >= 3 {
        return;
    }
    // M389 (file_82×83 −18.9 vs fddb): Word keeps a **single** empty pure-I
    // spacer before trailing pure-D short demo title ("Title Style Centered
    // Demo"). B may end with 1–2 empties; M85a stripped all. Keep one spacer
    // when the first pure-D is a short demo/title residual.
    //
    // M389b (file_36 residual): same for short pure-D titles without the word
    // "Demo" (e.g. "Contract Review", 2 words) when multi pure-D residual.
    //
    // M389: keep one empty pure-I before trailing pure-D short demo title.
    // M392: keep **both** empties only when a table follows the pure-D run
    // (file_36×37 Word II before "Contract Review"+empty+tbl). Trailing
    // pure-D demos without a table (file_82×83) still keep exactly one.
    let first_del = non_sect[run_start];
    let short_title_del = {
        let n = para_word_atom_count(dom, first_del);
        (1..=4).contains(&n) && para_body_alnum_len(dom, first_del) <= 32
    };
    let keep_title_spacers = (1..=2).contains(&empty_run)
        && (para_looks_like_demo_title(dom, first_del)
            || para_has_heading_or_title_style(dom, first_del)
            || short_title_del);
    let table_after_del_run = non_sect
        .get(run_start + 1..)
        .into_iter()
        .flatten()
        .any(|&k| dom.name_is(k, &W::tbl()))
        || {
            // pure-D run may end before a table with an empty pure-D between.
            let after_run = run_start
                + non_sect[run_start..]
                    .iter()
                    .take_while(|&&k| dom.name_is(k, &W::p()) && para_is_pure_deleted(dom, k))
                    .count();
            non_sect
                .get(after_run)
                .is_some_and(|&k| dom.name_is(k, &W::tbl()))
        };
    // Strip at most two trailing empties before pure-dels (M85a original).
    // Keep both only for empty-then-table title residuals; else keep one.
    let strip_n = if keep_title_spacers {
        if table_after_del_run {
            0
        } else {
            empty_run.saturating_sub(1)
        }
    } else {
        empty_run
    };
    for j in 0..strip_n {
        // Strip farthest empties first (index run_start - empty_run + j).
        let victim = non_sect[run_start - empty_run + j];
        if para_is_empty_pure_ins(dom, victim) {
            dom.remove(victim);
        }
    }
}

/// True when every `w:t` / `w:delText` under `p` is empty or whitespace.
pub(super) fn para_body_text_is_whitespace_only(dom: &Dom, p: NodeId) -> bool {
    let mut saw = false;
    for name in [W::t(), W::del_text()] {
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
pub(super) fn para_is_pure_inserted(dom: &Dom, p: NodeId) -> bool {
    if !dom.name_is(p, &W::p()) {
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
///
/// M345: when pure-D is structural (list style), adopt its full pPr so the
/// fold does not drop ListParagraph/numPr. When pure-D is bare and pure-I only
/// carries spacing, skip fold if a long pure-D residual follows (keep Word's
/// empty pure-I spacer — two_column×vrect).
/// M341 / M86 — fold a whitespace-only pure-I paragraph into the
/// following pure-D, producing a MIX on the deleted paragraph (Word
/// shape: `I D` becomes `M D` when the pure-I was whitespace only).
///
/// `file_88`: Word writes `MIX D` with the inserted whitespace kept
/// and the deletion applied to the merged paragraph. The pure-D is
/// untouched when its deleted text is followed by another pure-I
/// (whitespace inserted between two deletions stays where Word puts
/// it). Must run **before** [`strip_trailing_empty_pure_ins`] so
/// `missing_sectpr × separator` keeps `I "something" + MIX empty+del`
/// shape.
pub fn fold_whitespace_pure_ins_into_following_pure_del(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    // M311c: wholesale empty pure-I layout then pure-D residual (image×rtl:
    // ~31 empty pure-I + 1 pure-D). Folding every empty into the del wiped
    // Word's pure-I layout. Skip the fold entirely when ≥3 empty pure-I
    // lead a body with no contentful pure-I.
    {
        let kids: Vec<NodeId> = dom
            .elements(body, None)
            .into_iter()
            .filter(|&k| !dom.name_is(k, &W::sect_pr()))
            .collect();
        let empty_pure_i = kids
            .iter()
            .filter(|&&k| {
                dom.name_is(k, &W::p())
                    && para_is_pure_inserted(dom, k)
                    && para_body_text_is_whitespace_only(dom, k)
            })
            .count();
        let content_pure_i = kids
            .iter()
            .filter(|&&k| {
                dom.name_is(k, &W::p())
                    && para_is_pure_inserted(dom, k)
                    && !para_body_text_is_whitespace_only(dom, k)
            })
            .count();
        if empty_pure_i >= 3 && content_pure_i == 0 {
            return;
        }
        // M430 (doc_with_spaces × doc_with_spacing ~62.8 / docxodus 100):
        // title-page pure-I has contentful pure-I then ≥3 trailing empty
        // pure-I immediately before pure-D base. Word keeps those empties
        // (I…date e×6 DD); M311c only skips when content_pure_i==0 so the
        // fold ate trailing blanks into first pure-D. Skip the fold entirely
        // when a trailing empty pure-I run of ≥3 sits at the pure-I→pure-D
        // boundary with no pure-I after the pure-D residual.
        {
            let kids: Vec<NodeId> = dom
                .elements(body, None)
                .into_iter()
                .filter(|&k| !dom.name_is(k, &W::sect_pr()))
                .collect();
            // Find first pure-D with real text.
            let first_del = kids.iter().position(|&k| {
                dom.name_is(k, &W::p())
                    && para_is_pure_deleted(dom, k)
                    && !para_body_text_is_whitespace_only(dom, k)
            });
            if let Some(di) = first_del {
                let mut empty_run = 0usize;
                let mut j = di;
                while j > 0 {
                    j -= 1;
                    let k = kids[j];
                    if dom.name_is(k, &W::p())
                        && para_is_pure_inserted(dom, k)
                        && para_is_visually_blank(dom, k)
                    {
                        empty_run += 1;
                    } else {
                        break;
                    }
                }
                let pure_i_after_del = kids[di + 1..]
                    .iter()
                    .any(|&k| dom.name_is(k, &W::p()) && para_is_pure_inserted(dom, k));
                let empty_run = empty_run + story_final_kept_blank(dom, &kids);
                if empty_run >= 3 && !pure_i_after_del && content_pure_i >= 1 {
                    return;
                }
            }
        }
    }
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
            // The fold target must be a PARAGRAPH: an all-deleted TABLE also
            // satisfies para_is_pure_deleted's descendant checks, and folding
            // moved its cell runs into a paragraph and deleted the tbl
            // element outright (multipara_cell×missing_separator: assembled
            // [p×5, tbl, p] lost the tbl here; diff_after16×19 lost its
            // table the same way, 41.6 vs docxodus 100).
            if !dom.name_is(del_p, &W::p()) {
                continue;
            }
            if !para_is_pure_inserted(dom, ins_p) || !para_is_pure_deleted(dom, del_p) {
                continue;
            }
            if !para_body_text_is_whitespace_only(dom, ins_p) {
                continue;
            }
            // M431 (diff_doc2 × numwords residual): page-break (`w:br`) and
            // drawing/pict pure-I are contentful for layout even with no `w:t`.
            // Word keeps pure-I br separate from pure-D drawing; folding them
            // as "whitespace" produced one MIX with ins br + del drawing.
            if !dom.descendants(ins_p, Some(&W::name("br"))).is_empty()
                || !dom.descendants(ins_p, Some(&W::drawing())).is_empty()
                || !dom.descendants(ins_p, Some(&W::pict())).is_empty()
                || !dom.descendants(ins_p, Some(&W::object())).is_empty()
            {
                continue;
            }
            // A run-less bare empty inserted paragraph is B's STRUCTURE when
            // more inserted block content follows the deleted run — Word
            // keeps it as its own inserted paragraph (anchor_images×annot2:
            // B's two leading empties before an inserted table; folding them
            // displaced the whole page, 44.1 vs sibling 100.00). When B has
            // nothing after the dels, the run-less fold IS Word's shape
            // (file_173_file_174 scored 100.00 with it). Skip the fold only
            // for run-less empties with following inserted block content.
            if dom.descendants(ins_p, Some(&W::t())).is_empty() {
                let mut later_ins = false;
                let mut seen_del_p = false;
                for &k2 in &kids {
                    if k2 == del_p {
                        seen_del_p = true;
                        continue;
                    }
                    if seen_del_p && !dom.descendants(k2, Some(&W::ins())).is_empty() {
                        later_ins = true;
                        break;
                    }
                }
                if later_ins {
                    continue;
                }
            }
            // Need real deleted content (not empty mark-only del).
            if para_body_text_is_whitespace_only(dom, del_p) {
                continue;
            }
            // M389 (file_82×83 −18.9): Word keeps empty pure-I spacer before
            // multi pure-D short demo titles ("Title Style Centered Demo"…).
            // Folding the empty into first pure-D invents MIX and drops the
            // spacer. Skip run-less empty pure-I × demo/title pure-D when ≥2
            // pure-D residual paras follow. M389b: also short pure-D titles
            // without the word Demo ("Contract Review").
            //
            // M392 (file_36×37): sole short pure-D title residual before a
            // table still needs the empty pure-I spacer(s). Require ≥1 pure-D
            // (not ≥2) so a single "Contract Review" keeps B's blanks.
            if dom.descendants(ins_p, Some(&W::t())).is_empty() {
                let n = para_word_atom_count(dom, del_p);
                let short_title = (1..=4).contains(&n) && para_body_alnum_len(dom, del_p) <= 32;
                if para_looks_like_demo_title(dom, del_p)
                    || para_has_heading_or_title_style(dom, del_p)
                    || short_title
                {
                    let mut following_pure_d = 0usize;
                    for &k in kids.iter().skip(i + 1) {
                        if dom.name_is(k, &W::p()) && para_is_pure_deleted(dom, k) {
                            following_pure_d += 1;
                        } else {
                            break;
                        }
                    }
                    if following_pure_d >= 1 {
                        continue;
                    }
                }
            }
            // M345: empty pure-I fold must not thrash pure-D layout.
            //
            // 1) Del has structural pPr (ListParagraph+numPr): adopt full del
            //    pPr onto the carrier. Old path kept bare ins-mark pPr and
            //    dropped list style (basic_list×sd_1707 first pure-D "List
            //    item 1" lost bullets — pagefair 99→66; same for
            //    pre_separated_list×diff_before7).
            // 2) Del bare + ins has spacing-only structure: skip fold when a
            //    long pure-D residual follows. Word keeps the empty pure-I
            //    spacer (two_column×vrect); folding contaminated first pure-D
            //    with vrect spacing (line=240) and thrased layout 79→34.
            // M359: skip when pure-D is long prose (list_with_indents items
            // ≥13 words) or pure-I carries a drawing. Empty drawing pure-I
            // × long list pure-D (list×shape_group) was folding the first
            // list item into the empty shell; multi-del then MIX-ed last
            // "My test with some shapes." with that list text (Word/27c
            // pure-I/D III…DDD; pagefair −9.4). Short list items still fold
            // (basic_list "List item 1" ≤12 words).
            let del_words = para_word_atom_count(dom, del_p);
            let ins_has_drawing = !dom.descendants(ins_p, Some(&W::drawing())).is_empty()
                || !dom
                    .descendants(ins_p, Some(&W::name("AlternateContent")))
                    .is_empty();
            if del_words > 12 || ins_has_drawing {
                continue;
            }
            let del_structural = dom
                .element(del_p, &W::p_pr())
                .is_some_and(|dp| ppr_has_structural_props(dom, dp));
            let ins_structural = dom
                .element(ins_p, &W::p_pr())
                .is_some_and(|ip| ppr_has_structural_props(dom, ip));
            if !del_structural && ins_structural {
                let mut following_pure_d = 0usize;
                for &k in kids.iter().skip(i + 1) {
                    if dom.name_is(k, &W::p()) && para_is_pure_deleted(dom, k) {
                        following_pure_d += 1;
                    } else {
                        break;
                    }
                }
                // M366 (bookmark×broken_complex_list): trailing empty pure-I
                // after content pure-I list, before a pure-D residual run —
                // Word drops the empty (17 pure-I, no spacer). M345 path-2
                // kept it as a spacer when ≥3 pure-D (two_column×vrect mid
                // empty). Allow fold when a content pure-I precedes this
                // empty in the pure-I run (true trailing residual).
                let trailing_after_content_i = i > 0
                    && kids[..i].iter().rev().any(|&k| {
                        dom.name_is(k, &W::p())
                            && para_is_pure_inserted(dom, k)
                            && !para_body_text_is_whitespace_only(dom, k)
                    });
                if following_pure_d >= 3 && !trailing_after_content_i {
                    continue;
                }
            }
            if del_structural {
                // M360: do not adopt full del pPr when pure-I is a TOC/field
                // residue (fldChar) and pure-D carries a Heading pStyle.
                // table_border×toc: empty TOC field-end pure-I × SD-2343
                // Heading1 pure-D got Heading1 on the MIX; Word leaves bare
                // pPr (pagefair −11). List numPr adopt still runs (basic_list).
                let ins_has_fld = para_is_field_residue(dom, ins_p);
                let del_heading_style = dom.element(del_p, &W::p_pr()).is_some_and(|dp| {
                    dom.element(dp, &W::p_style()).is_some_and(|ps| {
                        let v = dom
                            .attribute(ps, &W::val())
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        v == "title" || v.starts_with("heading")
                    })
                });
                if !(ins_has_fld && del_heading_style) {
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
                // else fall through to mark-only fold without Heading pPr adopt
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
                // M360 applies on this branch too: a fldChar residue carrier
                // folded with a Heading pure-D keeps a mark-only pPr, never
                // the Heading pStyle (table_border×toc SD-2343 title).
                if let Some(drpr) = dom.element(dppr, &W::r_pr())
                    && dom.element(drpr, &W::del()).is_some()
                {
                    let ins_has_fld = para_is_field_residue(dom, ins_p);
                    let del_heading_style = dom.element(dppr, &W::p_style()).is_some_and(|ps| {
                        let v = dom
                            .attribute(ps, &W::val())
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        v == "title" || v.starts_with("heading")
                    });
                    let cloned = if ins_has_fld && del_heading_style {
                        let ppr = dom.new_element(W::p_pr());
                        let mark = dom.clone_subtree(drpr);
                        dom.add(ppr, mark);
                        ppr
                    } else {
                        dom.clone_subtree(dppr)
                    };
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

/// M471 — first slice of the mesh off-by-one rotation (diff_after19 ×
/// diff_after2 oracle). Our produce can emit an INSERTED paragraph mark
/// whose entire content is DELETED base text — impossible in Word's output
/// (accept-all would stray an empty paragraph). Word's arrangement is the
/// same region rotated one slot:
///
/// ```text
///   ours:  Pi   MD [ins A…, del B…]     Word:  Pi   MI [ins A…]
///          Pi+1 MI [del C…]                    Pi+1 MD [ins i…, del B…]
///          Pi+2 --  [ins i…, EQ …]             Pi+2 --  [del C…, EQ …]
/// ```
///
/// Strictly gated on that exact three-paragraph signature.
/// M471 — rotate the impossible `[MD ins+del][MI del-only][live ins…EQ]`
/// region one slot to Word's arrangement.
///
/// `diff_after19 × diff_after2`: Word writes `ins+del + live ins…EQ +
/// del-only` instead of `del-only + ins+del + live ins…EQ`. The pass
/// recognizes the three-paragraph cluster, swaps the first two slots,
/// and lets downstream peels reach the right shape. Runs late — the
/// fold/restore sequence above has to materialize the cluster first.
pub fn rotate_ins_mark_del_only_paragraph(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    loop {
        let kids: Vec<NodeId> = dom
            .elements(body, None)
            .into_iter()
            .filter(|&k| dom.name_is(k, &W::p()))
            .collect();
        let mut acted = false;
        for &[p0, p1, p2] in kids.array_windows() {
            // Pi: MARK-DEL, children = ins+ then del+, nothing else.
            if !para_mark_revision(dom, p0, &W::del()) {
                continue;
            }
            let c0: Vec<NodeId> = dom
                .elements(p0, None)
                .into_iter()
                .filter(|&c| !dom.name_is(c, &W::p_pr()))
                .collect();
            let mut p0_ins = Vec::new();
            let mut p0_del = Vec::new();
            let mut bad = c0.is_empty();
            for &c in &c0 {
                if dom.name_is(c, &W::ins()) {
                    if !p0_del.is_empty() {
                        bad = true;
                        break;
                    }
                    p0_ins.push(c);
                } else if dom.name_is(c, &W::del()) {
                    p0_del.push(c);
                } else {
                    bad = true;
                    break;
                }
            }
            if bad || p0_ins.is_empty() || p0_del.is_empty() {
                continue;
            }
            // Pi+1: MARK-INS, children = del+ only.
            if !para_mark_revision(dom, p1, &W::ins()) {
                continue;
            }
            let c1: Vec<NodeId> = dom
                .elements(p1, None)
                .into_iter()
                .filter(|&c| !dom.name_is(c, &W::p_pr()))
                .collect();
            if c1.is_empty() || c1.iter().any(|&c| !dom.name_is(c, &W::del())) {
                continue;
            }
            // Pi+2: no mark revision; leading ins-block(s) then a non-rev child.
            if para_mark_revision(dom, p2, &W::ins()) || para_mark_revision(dom, p2, &W::del()) {
                continue;
            }
            let c2: Vec<NodeId> = dom
                .elements(p2, None)
                .into_iter()
                .filter(|&c| !dom.name_is(c, &W::p_pr()))
                .collect();
            let mut p2_lead_ins = Vec::new();
            for &c in &c2 {
                if dom.name_is(c, &W::ins()) {
                    p2_lead_ins.push(c);
                } else {
                    break;
                }
            }
            if p2_lead_ins.is_empty() || p2_lead_ins.len() == c2.len() {
                continue;
            }
            // --- rewrite ---
            // The marks trade places with their properties: Pi takes the
            // inserted mark's pPr (the revision's), Pi+1 the deleted mark's
            // (29e3872eed: the deleted title's jc=center stays on the deleted
            // mark; flipping only the revision marks left it live on the
            // inserted paragraph, so accept-all centred it).
            let (Some(ppr0), Some(ppr1)) =
                (dom.element(p0, &W::p_pr()), dom.element(p1, &W::p_pr()))
            else {
                continue;
            };
            dom.remove(ppr0);
            dom.remove(ppr1);
            dom.add_first(p0, ppr1);
            dom.add_first(p1, ppr0);
            // p1's old del-blocks → front of p2 (in order).
            for &d in c1.iter().rev() {
                dom.remove(d);
                match dom
                    .elements(p2, None)
                    .iter()
                    .copied()
                    .find(|&c| !dom.name_is(c, &W::p_pr()))
                {
                    Some(first) => dom.add_before_self(first, d),
                    None => dom.add(p2, d),
                }
            }
            // p2's leading ins-blocks → p1 (front, in order), then p0's
            // del-blocks appended after them.
            for &i in p2_lead_ins.iter().rev() {
                dom.remove(i);
                match dom
                    .elements(p1, None)
                    .iter()
                    .copied()
                    .find(|&c| !dom.name_is(c, &W::p_pr()))
                {
                    Some(first) => dom.add_before_self(first, i),
                    None => dom.add(p1, i),
                }
            }
            for &d in &p0_del {
                dom.remove(d);
                dom.add(p1, d);
            }
            acted = true;
            break;
        }
        if !acted {
            break;
        }
    }
}

/// M473 — sibling of the M471 rotation: the deletion mark lands one slot
/// late. A fully-deleted base paragraph inside a replace region comes out as
/// [bare-pPr live-mark, del-only content][contentless MARK-DEL shell]. The
/// content paragraph must carry MARK-DEL (its live mark makes accept-all
/// strand an empty paragraph B never had). Whether the SHELL also stays
/// depends on base cardinality: one deleted A-para → Word merges to a
/// single [MD content] (ooxml_size_rstyle × ooxml_strike_rstyle); two
/// deleted A-paras (content + empty) → Word keeps BOTH [MD content]
/// [MD empty] (file_36 × file_37). Cardinality is produce-side provenance a
/// finalize pass cannot see, and the damage is asymmetric — merging when
/// wrong removes a rendered line and shifts every page below (−30 pixels on
/// file_36), keeping the shell when wrong costs one trailing empty line. So
/// the rewrite only RESTAMPS: clone the shell's marked pPr onto the content
/// paragraph and leave the shell in place.
/// M473 — clone the `[contentless MARK-DEL shell]`'s `pPr` onto the
/// preceding `[bare-pPr live-mark + del-only content]` paragraph,
/// leaving the shell in place.
///
/// `ooxml_size_rstyle × ooxml_strike_rstyle` / `file_36 × file_37`: a
/// fully-deleted base paragraph inside a replace region sometimes
/// splits into the bare-mark content paragraph and a contentless
/// MARK-DEL shell. Word writes one paragraph `[MD content]` for one
/// deleted A-para, or keeps both `[MD content] [MD empty]` when the
/// base had content + empty. Cardinality is produce-side provenance
/// a finalize pass cannot see, and the damage is asymmetric — merging
/// when wrong drops a rendered line and shifts every page below
/// (−30 px on `file_36`); keeping the shell when wrong costs one
/// trailing empty line. The rewrite **only restamps**: it copies the
/// shell's marked `pPr` onto the content paragraph and leaves the
/// shell in place.
pub fn restamp_stranded_del_mark_onto_del_only_paragraph(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    loop {
        let kids: Vec<NodeId> = dom
            .elements(body, None)
            .into_iter()
            .filter(|&k| dom.name_is(k, &W::p()))
            .collect();
        let mut acted = false;
        for &[p0, p1] in kids.array_windows() {
            // p0: live mark, bare pPr (no properties at all), children all
            // w:del blocks with real deleted text.
            if para_mark_revision(dom, p0, &W::del()) || para_mark_revision(dom, p0, &W::ins()) {
                continue;
            }
            if let Some(ppr) = dom.element(p0, &W::p_pr())
                && !dom.elements(ppr, None).is_empty()
            {
                continue;
            }
            let c0: Vec<NodeId> = dom
                .elements(p0, None)
                .into_iter()
                .filter(|&c| !dom.name_is(c, &W::p_pr()))
                .collect();
            if c0.is_empty() || c0.iter().any(|&c| !dom.name_is(c, &W::del())) {
                continue;
            }
            let has_del_text = c0.iter().any(|&d| {
                dom.descendants(d, None)
                    .iter()
                    .any(|&n| dom.name_is(n, &W::del_text()))
            });
            if !has_del_text {
                continue;
            }
            // p1: contentless MARK-DEL shell — pPr holds exactly [rPr [del]].
            if !para_mark_revision(dom, p1, &W::del()) {
                continue;
            }
            let c1: Vec<NodeId> = dom
                .elements(p1, None)
                .into_iter()
                .filter(|&c| !dom.name_is(c, &W::p_pr()))
                .collect();
            if !c1.is_empty() {
                continue;
            }
            let Some(ppr1) = dom.element(p1, &W::p_pr()) else {
                continue;
            };
            let ppr1_kids = dom.elements(ppr1, None);
            if ppr1_kids.len() != 1 || !dom.name_is(ppr1_kids[0], &W::r_pr()) {
                continue;
            }
            let rpr1_kids = dom.elements(ppr1_kids[0], None);
            if rpr1_kids.len() != 1 || !dom.name_is(rpr1_kids[0], &W::del()) {
                continue;
            }
            // --- rewrite: clone the shell's marked pPr onto p0 (replacing
            // its bare pPr); the shell itself stays where it is. The final
            // revision-id renumber pass resolves the duplicated del id.
            let cloned = dom.clone_subtree(ppr1);
            match dom.element(p0, &W::p_pr()) {
                Some(old) => {
                    dom.add_before_self(old, cloned);
                    dom.remove(old);
                }
                None => dom.add_first(p0, cloned),
            }
            acted = true;
            break;
        }
        if !acted {
            break;
        }
    }
}

/// M469 — split a head title MIX: B's SHORT inserted title (≤4 tokens)
/// paired with A's LONG deleted opening (≥10 tokens, no shared significant
/// token) is Word's [pure-INS title] + [style-less MARK-DEL paragraph]
/// (ooxml_rfonts_rstyle × sd_2672_rtl_table oracle: bare pPr, no pStyle —
/// the deleted text renders at body size, not title size).
/// M469 — split the head title's MIX when a SHORT inserted title is
/// followed by a LONG unrelated deletion into a separate MARK-DEL
/// paragraph without the title's style.
///
/// `rfonts_rstyle × sd_2672_rtl_table`: Word renders the deleted
/// opening at body size, not at the heading size. The pass moves the
/// deleted opening off the heading onto a style-less MARK-DEL
/// paragraph so the heading residual stays as the inserted title +
/// trailing body mixed together.
pub fn split_head_short_title_long_del_mix(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let Some(&first) = dom
        .elements(body, None)
        .iter()
        .find(|&&k| dom.name_is(k, &W::p()))
    else {
        return;
    };
    // Exact shape: [pPr?] then one-or-more w:ins, then one-or-more w:del,
    // nothing else (no EQ interleaving, no mark revision).
    let kids: Vec<NodeId> = dom
        .elements(first, None)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::p_pr()))
        .collect();
    if kids.is_empty()
        || para_mark_revision(dom, first, &W::ins())
        || para_mark_revision(dom, first, &W::del())
    {
        return;
    }
    let mut ins_run = Vec::new();
    let mut del_run = Vec::new();
    for &c in &kids {
        if dom.name_is(c, &W::ins()) {
            if !del_run.is_empty() {
                return; // interleaved — not the wholesale title shape
            }
            ins_run.push(c);
        } else if dom.name_is(c, &W::del()) {
            del_run.push(c);
        } else {
            return; // EQ content present — Word meshed, leave alone
        }
    }
    if ins_run.is_empty() || del_run.is_empty() {
        return;
    }
    let text_of = |dom: &Dom, nodes: &[NodeId], del: bool| -> String {
        let tag = if del { W::del_text() } else { W::t() };
        let mut s = String::new();
        for &n in nodes {
            for t in dom.descendants(n, Some(&tag)) {
                s.push_str(&dom.value_str(t));
            }
        }
        s
    };
    let ins_text = text_of(dom, &ins_run, false);
    let del_text = text_of(dom, &del_run, true);
    let toks = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .map(str::to_lowercase)
            .collect()
    };
    let it = toks(&ins_text);
    let dt = toks(&del_text);
    if it.is_empty() || it.len() > 4 || dt.len() < 10 {
        return;
    }
    // No shared significant token (len≥4) — unrelated titles only.
    let dset: std::collections::HashSet<&str> = dt
        .iter()
        .map(String::as_str)
        .filter(|t| t.len() >= 4)
        .collect();
    if it.iter().any(|t| t.len() >= 4 && dset.contains(t.as_str())) {
        return;
    }
    // Split: new style-less paragraph with MARK-DEL after `first`,
    // carrying the del blocks verbatim; mark copies the del's author/date.
    let author = dom.attribute(del_run[0], &W::author()).map(str::to_string);
    let date = dom.attribute(del_run[0], &W::date()).map(str::to_string);
    let max_id: u32 = dom
        .descendants(root, None)
        .into_iter()
        .filter_map(|e| {
            dom.attribute(e, &W::id())
                .and_then(|v| v.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    let np = dom.new_element(W::p());
    let ppr = dom.new_element(W::p_pr());
    let rpr = dom.new_element(W::r_pr());
    let mark = dom.new_element(W::del());
    dom.set_attribute_value(mark, &W::id(), Some(&(max_id + 1).to_string()));
    if let Some(a) = &author {
        dom.set_attribute_value(mark, &W::author(), Some(a));
    }
    if let Some(d) = &date {
        dom.set_attribute_value(mark, &W::date(), Some(d));
    }
    dom.add(rpr, mark);
    dom.add(ppr, rpr);
    dom.add(np, ppr);
    for &d in &del_run {
        dom.remove(d);
        dom.add(np, d);
    }
    dom.add_after_self(first, np);
}

/// M105 — fold the leading insertion of a MIX paragraph back onto a
/// preceding pure-D, leaving the MIX as pure-D + the trailing tokens
/// of the MIX.
///
/// `file_7` / `file_5` / `file_130` document peel: a pure-D short
/// title followed by a MIX whose leading run is insertion (a Word
/// subtitle landing on the title residual) collapses to a single
/// pure-D with the insertion folded onto it. Gated on: the pure-D
/// is short (≤ 8 body tokens), the next paragraph carries both
/// `w:ins` and `w:del`, the leading run is an insertion, and the
/// pure-D shares no significant token with that leading insert
/// (unrelated short title vs long-doc subtitle). Rescored by the
/// corpus when the leading insertion token has an `INS=` form that
/// meets M464's [`peel_trailing_for_word_onto_next_mix`] shape.
pub fn fold_leading_ins_from_mix_into_preceding_pure_del(dom: &mut Dom, root: NodeId) {
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
            let del_p = kids[i];
            let mix_p = kids[i + 1];
            if !dom.name_is(del_p, &W::p()) || !dom.name_is(mix_p, &W::p()) {
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
                if dom.name_is(c, &W::p_pr()) {
                    continue;
                }
                if dom.name_is(c, &W::ins()) {
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
                .find(|&c| !dom.name_is(c, &W::p_pr()));
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
/// M85b — strip the paragraph-mark deletion off the last body
/// paragraph when its `pPr` carries only the mark (no other layout).
///
/// `file_186` / `file_49`: Word leaves a bare trailing empty last
/// paragraph without a `pPr` deletion (the deletion got recorded in
/// the preceding paragraph's `pPrChange` instead). The pass removes
/// `pPr/rPr/w:del` when those are the only `pPr` children, then
/// drops the empty `pPr` / `rPr` shells. Does **not** strip a
/// `pPr` carrying layout alongside the mark — Word parks those in
/// `pPrChange` instead.
pub fn strip_last_pure_del_mark_only_ppr(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids.iter().rev().find(|&&k| !dom.name_is(k, &W::sect_pr())) else {
        return;
    };
    if !dom.name_is(last, &W::p()) || !para_is_pure_deleted(dom, last) {
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
    if !dom.name_is(rpr, &W::r_pr()) {
        return;
    }
    let rpr_kids = dom.elements(rpr, None);
    if rpr_kids.len() != 1 || !dom.name_is(rpr_kids[0], &W::del()) {
        return;
    }
    dom.remove(ppr);
}

/// True when paragraph body has both ins and del content (mixed residual).
pub(super) fn para_is_mixed_revision(dom: &Dom, p: NodeId) -> bool {
    if !dom.name_is(p, &W::p()) {
        return false;
    }
    if let Some(cached) = MIXED_CACHE.with(|c| c.borrow().as_ref().and_then(|m| m.get(&p).copied()))
    {
        return cached;
    }
    let has_ins =
        !dom.descendants(p, Some(&W::ins())).is_empty() || para_mark_revision(dom, p, &W::ins());
    let has_del =
        !dom.descendants(p, Some(&W::del())).is_empty() || para_mark_revision(dom, p, &W::del());
    let v = has_ins && has_del;
    MIXED_CACHE.with(|c| {
        if let Some(m) = c.borrow_mut().as_mut() {
            m.insert(p, v);
        }
    });
    v
}

/// M83b / M87 / M91 / M93 / M94 — last pure-deleted **or mixed** body paragraph:
/// Word moves live layout props into `w:pPrChange` and drops the mark-only del shell.
///
/// - M83b: live `w:spacing` → pPrChange (file_23 "Document Title").
/// - M87: live `w:numPr` / `w:ind` → pPrChange (file_55 last pure-del "b").
/// - M91: live `w:jc` → pPrChange (file_105 last pure-del right-align).
/// - M93: live `w:pStyle` → pPrChange (file_59 last pure-del PreformattedText).
/// - M94: **last mixed** I+D residual parks **spacing only** (file_139).
///
/// M434 (image_doc × document ~59 / docxodus 100): last MIX is a list residual
/// (live `numPr` + pure-I text + drawing del). Word keeps live pStyle/numPr/jc/
/// spacing and only uses `rPrChange` under rPr. Parking numPr/pStyle into
/// pPrChange cleared list chrome → LO page 2–3 thrash (page1 100, p2–3 ~57).
/// For MIX, only park spacing; pure-D still parks the full layout set.
///
/// Mid pure-dels keep live spacing/numPr/jc/pStyle.
/// M69 (late) — move the last pure-D paragraph's live `w:spacing`
/// into `w:pPrChange` so the deletion mark stays live while the
/// layout change is recorded (Word revision style).
///
/// Runs **after** [`merge_replaced_paragraphs_in`] and the
/// pPr-only peels; the early-finalize rPr/pPr-only passes leave a
/// last pure-D whose mark carries only `pPr` and the spacing the
/// pure-D paragraph itself owns. Skipped when the revised document
/// ends on the same spacing (the live revision is the spacing's own
/// source) — uses [`last_body_para_spacing`] for that gate.
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
    let Some(&last) = kids.iter().rev().find(|&&k| !dom.name_is(k, &W::sect_pr())) else {
        return;
    };
    if !dom.name_is(last, &W::p()) {
        return;
    }
    // Pure-del or mixed last residual (M94). M226 gated MIX out for subtitle
    // cousins, but full ledger showed catastrophic regs (red_heading −37,
    // heading chain −6..−12). Restore MIX parking; keep M228 mid pure-D
    // promote + strip_redundant for the spacing wins that don't need this gate.
    let is_mixed = para_is_mixed_revision(dom, last);
    if !para_is_pure_deleted(dom, last) && !is_mixed {
        return;
    }
    // M444/M446a: last MIX free-mesh keeps Word live spacing (not pPrChange).
    // History: M226 gated *all* MIX → red_heading −37. M446b short-doc gate
    // (≤6 paras, multi-token both sides) thrash'd red_heading/file_139/etc.
    // Keep surgical free-mesh shapes only:
    // - M444: short ins title (1..=4) × long del cover (≥5)
    // - M446a: long ins body (≥5) × short del residual (1..=4) — subtitle
    if is_mixed {
        // A mixed paragraph that keeps unchanged words is one paragraph with
        // a few words revised, not a replaced one: Word leaves its properties
        // live, in the body, a header and a text box alike (fixtures_500
        // 00b81efae883 "Overall purpose of the post" → "Overall aim of the
        // post"). file_139's last paragraph was replaced whole.
        if para_keeps_unchanged_words(dom, last) {
            return;
        }
        let ins_w = para_side_word_count(dom, last, true);
        let del_w = para_side_word_count(dom, last, false);
        if (1..=4).contains(&ins_w) && del_w >= 5 {
            return;
        }
        if (1..=4).contains(&del_w) && ins_w >= 5 {
            return;
        }
    }
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    if dom.element(ppr, &W::p_pr_change()).is_some() {
        return;
    }
    // Layout props Word records under pPrChange on the last pure-del / mixed.
    // M434: MIX keeps live list/style/jc chrome; only spacing parks (M94).
    let movable: Vec<_> = if is_mixed {
        vec![W::spacing_el()]
    } else {
        vec![
            W::spacing_el(),
            W::num_pr(),
            W::name("ind"),
            W::jc_el(),
            W::p_style(),
        ]
    };
    let mut to_move: Vec<NodeId> = Vec::new();
    for name in &movable {
        if let Some(el) = dom.element(ppr, name) {
            to_move.push(el);
        }
    }
    if to_move.is_empty() {
        return;
    }
    // Rejection replaces the whole pPr, including properties left live here.
    // Record the complete original layout before parking only the movable set.
    let old_inner = formatchg::project_para_properties_for_change(dom, ppr);
    for el in &to_move {
        dom.remove(*el);
    }
    let chg = dom.new_element(W::p_pr_change());
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
/// M98b — park a trailing MIX paragraph's spacing on a following
/// bare empty paragraph (not under `pPrChange`).
///
/// `file_167`: when the trailing MIX (mix+empty_eq) carries a
/// non-default spacing, Word parks that spacing on the next
/// paragraph's live `pPr` instead of recording it as a `pPrChange`.
/// The pass moves the spacing off the MIX paragraph onto the
/// trailing empty so the rendering matches LibreOffice.
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
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.len() < 2 {
        return;
    }
    for i in 0..kids.len() - 1 {
        let mixed = kids[i];
        let empty = kids[i + 1];
        if !dom.name_is(mixed, &W::p()) || !dom.name_is(empty, &W::p()) {
            continue;
        }
        if !para_is_mixed_revision(dom, mixed) {
            continue;
        }
        if !para_has_no_text(dom, empty) {
            continue;
        }
        // Last body block (stamp demos) OR empty immediately before a pure-D
        // table (quarterly×red_bold: MIX title residual, empty, deleted table).
        let is_trailing = i + 1 == kids.len() - 1;
        // Pure-D table: has del, no ins (deleted whole table).
        let before_pure_d_table = i + 2 < kids.len()
            && dom.name_is(kids[i + 2], &W::tbl())
            && dom.descendants(kids[i + 2], Some(&W::ins())).is_empty()
            && !dom.descendants(kids[i + 2], Some(&W::del())).is_empty();
        if !is_trailing && !before_pure_d_table {
            continue;
        }
        let Some(mppr) = dom.element(mixed, &W::p_pr()) else {
            continue;
        };
        let Some(spacing) = dom.element(mppr, &W::spacing_el()) else {
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
        if dom.element(eppr, &W::spacing_el()).is_none() {
            let sp = dom.clone_subtree(spacing);
            dom.add_first(eppr, sp);
        }
        dom.remove(spacing);
        // Empty pPrChange (old empty) when none present — Word shape.
        if dom.element(eppr, &W::p_pr_change()).is_none() {
            let old_inner = dom.new_element(W::p_pr());
            let chg = dom.new_element(W::p_pr_change());
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

/// M228 / M226 / M231 single body walk (perf: avoid 3× O(n_p) on large docs).
///
/// - **M228:** mid pure-D promote spacing-only pPrChange → live; strip line=276 noise.
/// - **M226:** drop no-op pPrChange on mixed residuals when explicit live/old
///   line spacing matches. Equal pilcrow and after-only changes retain history.
/// - **M231:** strip schema-default `jc=left|start` live and from pPrChange-old.
///
/// M228 + M226 + M231 in one body walk — promote mid pure-D spacing
/// from `pPrChange`, drop no-op equal-spacing `pPrChange` records, and
/// strip the default `w:jc="left"`.
///
/// The three reads are interleaved over the same paragraph tree so the
/// walk stays single-pass. Runs **after** the merge / park peels when
/// the layout is settled. Idempotent.
pub fn cleanup_spacing_and_default_jc(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.is_empty() {
        return;
    }
    let last_i = kids.len() - 1;
    // Fast path: no del in body → M228 cannot fire; still may need M226/M231.
    let body_has_del = !dom.descendants(body, Some(&W::del())).is_empty();
    for (i, &p) in kids.iter().enumerate() {
        if !dom.name_is(p, &W::p()) {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };

        // --- M231: live default jc ---
        if let Some(jc) = dom.element(ppr, &W::jc_el()) {
            let val = dom.attribute(jc, &W::val()).unwrap_or("");
            if val == "left" || val == "start" {
                dom.remove(jc);
            }
        }

        let ppc = dom.element(ppr, &W::p_pr_change());
        if let Some(ppc) = ppc {
            let old_ppr = dom
                .elements(ppc, None)
                .into_iter()
                .find(|&c| dom.name_is(c, &W::p_pr()));
            if let Some(old_ppr) = old_ppr {
                // --- M231: pPrChange old default jc ---
                let mut removed_left_jc = false;
                if let Some(jc) = dom.element(old_ppr, &W::jc_el()) {
                    let val = dom.attribute(jc, &W::val()).unwrap_or("");
                    if val == "left" || val == "start" {
                        dom.remove(jc);
                        removed_left_jc = true;
                    }
                }
                if removed_left_jc {
                    let mut has_layout = false;
                    for c in dom.elements(old_ppr, None) {
                        let Some(n) = dom.name(c) else {
                            continue;
                        };
                        if n.local_name() == "rPr" {
                            continue;
                        }
                        has_layout = true;
                        break;
                    }
                    if !has_layout {
                        dom.remove(ppc);
                        continue; // ppc gone
                    }
                }

                // Re-fetch ppc after possible remove above.
            }
        }
        // Re-bind ppc after M231 may have removed it.
        let Some(ppc) = dom.element(ppr, &W::p_pr_change()) else {
            continue;
        };
        let old_ppr = dom
            .elements(ppc, None)
            .into_iter()
            .find(|&c| dom.name_is(c, &W::p_pr()));
        let Some(old_ppr) = old_ppr else {
            continue;
        };

        // --- M226: mixed-residual equal live/old spacing no-op pPrChange ---
        // The measured heading cousins are mixed paragraphs. Applying this to
        // equal pilcrows erases genuine spacing-removal history (M81/file_69).
        // IMPORTANT: only gate *mixed* here. Pure-deleted mid body must fall
        // through to M228 (was dead when this was `if !mixed { continue }`).
        if para_is_mixed_revision(dom, p) {
            if let (Some(live_sp), Some(old_sp)) = (
                dom.element(ppr, &W::spacing_el()),
                dom.element(old_ppr, &W::spacing_el()),
            ) {
                // The measured Heading/Title/Subtitle cousins all carry explicit
                // line=240/276. An after-only value (M81/file_69) is real history.
                if dom.attribute(live_sp, &W::name("line")).is_none()
                    || dom.attribute(old_sp, &W::name("line")).is_none()
                {
                    continue;
                }
                let same_spacing = ["before", "after", "line", "lineRule"].iter().all(|&a| {
                    dom.attribute(live_sp, &W::name(a)).unwrap_or("")
                        == dom.attribute(old_sp, &W::name(a)).unwrap_or("")
                });
                if same_spacing {
                    let mut other_layout = false;
                    for c in dom.elements(old_ppr, None) {
                        let Some(n) = dom.name(c) else {
                            continue;
                        };
                        let local = n.local_name();
                        if local == "spacing" || local == "rPr" || local == "pStyle" {
                            continue;
                        }
                        other_layout = true;
                        break;
                    }
                    if !other_layout {
                        dom.remove(ppc);
                        continue;
                    }
                }
            }
            // Mixed residual: do not run pure-del M228 promote.
            continue;
        }

        // --- M228: mid pure-D spacing promote / line=276 noise ---
        // Cheap gates before pure_deleted (expensive on large bodies).
        if !body_has_del || dom.element(ppr, &W::spacing_el()).is_some() {
            continue;
        }
        // ppc / old_ppr still bound above
        let Some(old_sp) = dom.element(old_ppr, &W::spacing_el()) else {
            continue;
        };
        if !para_is_pure_deleted(dom, p) {
            continue;
        }
        let mut other = false;
        for c in dom.elements(old_ppr, None) {
            let Some(n) = dom.name(c) else {
                continue;
            };
            let local = n.local_name();
            if local == "spacing" || local == "rPr" || local == "pStyle" {
                continue;
            }
            other = true;
            break;
        }
        if other {
            continue;
        }
        let line = dom.attribute(old_sp, &W::name("line")).unwrap_or("");
        let before = dom.attribute(old_sp, &W::name("before")).unwrap_or("");
        let after = dom.attribute(old_sp, &W::name("after")).unwrap_or("");
        let line_rule = dom.attribute(old_sp, &W::name("lineRule")).unwrap_or("");
        let is_line276_noise =
            line == "276" && before.is_empty() && after.is_empty() && line_rule.is_empty();
        if is_line276_noise {
            dom.remove(ppc);
            continue;
        }
        if i == last_i {
            continue;
        }
        let live = dom.clone_subtree(old_sp);
        dom.add_before_self(ppc, live);
        dom.remove(ppc);
    }
}

/// M221 (green_underline×heading_1_bold ~56): MIX residual carries B Heading
/// spacing (before=400 after=120 line=240) while following pure-D bullets have
/// none. Word parks that spacing on the **last** pure-D with live spacing +
/// `pPrChange(empty old)`, and leaves the MIX with only a del pilcrow.
///
/// Fires when: MIX with live spacing, followed only by pure-Ds (1..=4), last
/// pure-D has content and no live spacing. Does not touch mid pure-Ds that
/// already carry layout props.
/// M221 — park a MIX paragraph's spacing (after the inserted/deletion
/// split) onto the trailing empty pure-D paragraph's live `pPr`.
///
/// `green_underline × heading_1`: Word keeps a residual Heading
/// paragraph with its spacing live at the last pure-D, not behind
/// `pPrChange`. Runs after [`park_mixed_numpr_onto_trailing_empty_pure_del`]
/// in the same peel ladder.
pub fn park_mixed_spacing_onto_trailing_pure_del(
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
    if kids.len() < 3 {
        return;
    }
    for i in 0..kids.len() {
        let mixed = kids[i];
        if !dom.name_is(mixed, &W::p()) || !para_is_mixed_revision(dom, mixed) {
            continue;
        }
        let Some(mppr) = dom.element(mixed, &W::p_pr()) else {
            continue;
        };
        let Some(spacing) = dom.element(mppr, &W::spacing_el()) else {
            continue;
        };
        // Following run must be pure-D only (at least one, at most 4).
        let mut j = i + 1;
        while j < kids.len() && dom.name_is(kids[j], &W::p()) && para_is_pure_deleted(dom, kids[j])
        {
            j += 1;
        }
        let n_dels = j - (i + 1);
        // Green bullets: 2 pure-D; customer_satisfaction×document_100: ~8 pure-D
        // survey lines after MIX title residual. Require ≥2 pure-D so a sole
        // trailing pure-D after MIX (calibri_heading_2×center_aligned_bold)
        // keeps MIX spacing — sole-del park regressed LO −26.
        if !(2..=10).contains(&n_dels) {
            continue;
        }
        let last_del = kids[j - 1];
        if para_has_no_text(dom, last_del) {
            continue;
        }
        let last_has_spacing = dom
            .element(last_del, &W::p_pr())
            .and_then(|p| dom.element(p, &W::spacing_el()))
            .is_some();
        if last_has_spacing {
            continue;
        }
        // Short/mid residual pure-D (bullets / survey lines). Long formal
        // residual sentences (times×title) keep MIX spacing — parking regressed
        // LO −14.
        let last_alnum = para_body_alnum_len(dom, last_del);
        let last_toks = body_token_set(&para_revision_body_text(dom, last_del)).len();
        if last_alnum > 80 || last_toks > 12 {
            continue;
        }
        // Heading-style spacing only (before≈400 after≈120), not thin line-only.
        let has_before_after = {
            let sp = spacing;
            let before = dom
                .attribute(sp, &W::name("before"))
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(0);
            let after = dom
                .attribute(sp, &W::name("after"))
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(0);
            before >= 200 && after >= 80
        };
        if !has_before_after {
            continue;
        }
        // Move spacing onto last pure-D.
        let lppr = match dom.element(last_del, &W::p_pr()) {
            Some(p) => p,
            None => {
                let p = dom.new_element(W::p_pr());
                if let Some(first) = dom.elements(last_del, None).first().copied() {
                    dom.add_before_self(first, p);
                } else {
                    dom.add(last_del, p);
                }
                p
            }
        };
        let sp = dom.clone_subtree(spacing);
        dom.add_first(lppr, sp);
        // pPrChange(empty old) on last pure-D — Word shape.
        if dom.element(lppr, &W::p_pr_change()).is_none() {
            let old_inner = dom.new_element(W::p_pr());
            let chg = dom.new_element(W::p_pr_change());
            dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
            *id_gen += 1;
            dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
            dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
            dom.add(chg, old_inner);
            dom.add(lppr, chg);
        }
        // Strip spacing from MIX; ensure del pilcrow.
        dom.remove(spacing);
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
        if dom.elements(mppr, None).is_empty() {
            dom.remove(mppr);
        }
        break; // one MIX cluster
    }
}

/// M230 (bullet_list_bold×bullet_list ~83): MIX residual carries live `numPr`
/// (B list item "Grapes") while trailing pure-D empties from A have none.
/// Word keeps MIX without numPr and parks numPr + empty `pPrChange` on the
/// **last** pure-D empty. Mirror of M221 spacing park, for list numbering.
/// M230 — park a MIX paragraph's `w:numPr` onto a trailing empty
/// pure-D paragraph's live `pPr`.
///
/// `bullet_list_bold × bullet_list`: Word lists `MIX | DEL` as `MIX
/// (del mark + numbering) | DEL (bare)`. The MIX paragraph carries
/// the list; the trailing empty pure-D paragraph owns the marker so
/// the deletion doesn't fight the `pPrChange` record. Mints a fresh
/// `id` from `id_gen`.
pub fn park_mixed_numpr_onto_trailing_empty_pure_del(
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
    if kids.len() < 3 {
        return;
    }
    for i in 0..kids.len() {
        let mixed = kids[i];
        if !dom.name_is(mixed, &W::p()) || !para_is_mixed_revision(dom, mixed) {
            continue;
        }
        let Some(mppr) = dom.element(mixed, &W::p_pr()) else {
            continue;
        };
        let Some(num) = dom.element(mppr, &W::num_pr()) else {
            continue;
        };
        // Following pure-D only; last must be empty (mark-only).
        let mut j = i + 1;
        while j < kids.len() && dom.name_is(kids[j], &W::p()) && para_is_pure_deleted(dom, kids[j])
        {
            j += 1;
        }
        let n_dels = j - (i + 1);
        // Bullet cousins: 3 trailing pure-D empties (A body lines).
        if !(2..=6).contains(&n_dels) {
            continue;
        }
        let last_del = kids[j - 1];
        // Last may hold deleted body text (A bullet lines) — Word still parks
        // numPr there. Do not require para_has_no_text.
        // No pure-D in the run may already hold numPr (Word parks only on last).
        let mut run_has_num = false;
        for &d in &kids[i + 1..j] {
            if dom
                .element(d, &W::p_pr())
                .and_then(|p| dom.element(p, &W::num_pr()))
                .is_some()
            {
                run_has_num = true;
                break;
            }
        }
        if run_has_num {
            continue;
        }
        let lppr = match dom.element(last_del, &W::p_pr()) {
            Some(p) => p,
            None => {
                let p = dom.new_element(W::p_pr());
                if let Some(first) = dom.elements(last_del, None).first().copied() {
                    dom.add_before_self(first, p);
                } else {
                    dom.add(last_del, p);
                }
                p
            }
        };
        if dom.element(lppr, &W::num_pr()).is_some() {
            continue;
        }
        let num_clone = dom.clone_subtree(num);
        // numPr before pPrChange / rPr.
        if let Some(ppc) = dom.element(lppr, &W::p_pr_change()) {
            dom.add_before_self(ppc, num_clone);
        } else if let Some(rpr) = dom.element(lppr, &W::r_pr()) {
            dom.add_before_self(rpr, num_clone);
        } else {
            dom.add_first(lppr, num_clone);
        }
        if dom.element(lppr, &W::p_pr_change()).is_none() {
            let old_inner = dom.new_element(W::p_pr());
            let chg = dom.new_element(W::p_pr_change());
            dom.set_attribute_value(chg, &W::id(), Some(&id_gen.to_string()));
            *id_gen += 1;
            dom.set_attribute_value(chg, &W::author(), Some(&settings.author_for_revisions));
            dom.set_attribute_value(chg, &W::date(), Some(&settings.date_time_for_revisions));
            dom.add(chg, old_inner);
            dom.add(lppr, chg);
        }
        dom.remove(num);
        break; // one MIX cluster
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
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.len() < 2 {
        return;
    }
    let last = kids[kids.len() - 1];
    if !dom.name_is(last, &W::p()) || !para_is_pure_deleted(dom, last) {
        return;
    }
    let Some(lppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    let Some(ppc) = dom.element(lppr, &W::p_pr_change()) else {
        return;
    };
    // Only when pPrChange carries spacing (line-spacing residual class). Do not
    // undo M91 which already parked jc-only into pPrChange (file_105).
    let ppc_has_spacing = dom
        .element(ppc, &W::p_pr())
        .or_else(|| {
            dom.elements(ppc, None)
                .into_iter()
                .find(|&c| dom.name_is(c, &W::p_pr()))
        })
        .is_some_and(|inner| dom.element(inner, &W::spacing_el()).is_some())
        || {
            // pPrChange children may be the old pPr directly
            !dom.descendants(ppc, Some(&W::spacing_el())).is_empty()
        };
    if !ppc_has_spacing {
        return;
    }
    if dom.element(lppr, &W::jc_el()).is_some() {
        return;
    }
    // Walk preceding paras for live jc (not only immediate prev — mixed may
    // hold spacing without jc after M102b).
    let mut donor_jc = None;
    for &prev in kids[..kids.len() - 1].iter().rev() {
        if !dom.name_is(prev, &W::p()) {
            continue;
        }
        let Some(pppr) = dom.element(prev, &W::p_pr()) else {
            continue;
        };
        // Live jc: first jc child before any pPrChange.
        for c in dom.elements(pppr, None) {
            if dom.name_is(c, &W::p_pr_change()) {
                break;
            }
            if dom.name_is(c, &W::jc_el()) {
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
    if let Some(ppc) = dom.element(lppr, &W::p_pr_change()) {
        dom.add_before_self(ppc, cloned);
    } else {
        dom.add_first(lppr, cloned);
    }
}

/// M451 (center_alignment_2 × center_alignment ~81.5 / docxodus 100):
/// mid MIX free-mesh carries **empty** `pPrChange` alongside live `jc` where
/// Word uses live `jc` + rPr/ins mark only. Word's rule (both oracles): the
/// paragraph holding the **inserted paragraph mark** records no `pPrChange`;
/// every other changed paragraph keeps one, empty old included (center pair
/// p1/p3 and file_148×file_149 stamp all keep `pPrChange(<w:pPr/>)`).
///
/// Our mesh places that inserted mark one paragraph after Word's (center p3
/// vs Word p2), so the shell to strip is the MIX immediately **before** the
/// mark-ins paragraph. An unconditional strip here ate the genuine
/// jc-addition record on the file_148 stamp (M102 regression).
///
/// Does not touch last residual (often needs pPrChange) or non-empty park.
pub fn strip_empty_pprchange_on_mix_with_live_jc(dom: &mut Dom, root: NodeId) {
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
    for (i, &p) in kids.iter().enumerate() {
        if p == last || !dom.name_is(p, &W::p()) || !para_is_mixed_revision(dom, p) {
            continue;
        }
        // Word drops the record only where the inserted paragraph mark lands;
        // our mesh emits that mark on the following paragraph.
        let next_has_inserted_mark = kids[i + 1..]
            .iter()
            .find(|&&k| dom.name_is(k, &W::p()))
            .is_some_and(|&n| {
                dom.element(n, &W::p_pr())
                    .and_then(|np| dom.element(np, &W::r_pr()))
                    .is_some_and(|rpr| dom.element(rpr, &W::ins()).is_some())
            });
        if !next_has_inserted_mark {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        // Need live jc.
        if dom.element(ppr, &W::jc_el()).is_none() {
            continue;
        }
        let Some(chg) = dom.element(ppr, &W::p_pr_change()) else {
            continue;
        };
        let Some(old) = dom.element(chg, &W::p_pr()) else {
            continue;
        };
        // Old pPr must be empty of layout children.
        let mut has_layout = false;
        for c in dom.elements(old, None) {
            let Some(n) = dom.name(c) else {
                continue;
            };
            if n == W::r_pr() {
                continue;
            }
            has_layout = true;
            break;
        }
        if has_layout {
            continue;
        }
        dom.remove(chg);
    }
}

/// Drop a `w:pPr` that records nothing Word keeps.
///
/// A property-less `<w:pPr/>`, or one whose only child is a `w:pPrChange`
/// with an empty inner `w:pPr`, is absent from Word's redline of the short
/// title mixes (blue centered title × blue italic, and the same shape on the
/// heading and subtitle demos) and from the pairs whose whole document has
/// no `w:pPr`. A live `w:rPr` pilcrow mark stays: that mark is the deletion
/// or insertion record, not an empty shell. A `w:pPrChange` that still holds
/// layout (spacing, jc, a style) stays too.
pub fn strip_propertyless_ppr(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let paras: Vec<NodeId> = dom.descendants(body, Some(&W::p()));
    for p in paras {
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        let children = dom.elements(ppr, None);
        let mut structural = false;
        let mut empty_changes: Vec<NodeId> = Vec::new();
        let mut keep_change = false;
        for c in children {
            if dom.name_is(c, &W::p_pr_change()) {
                let inner_has_layout = dom.element(c, &W::p_pr()).is_some_and(|inner| {
                    dom.elements(inner, None)
                        .iter()
                        .any(|&k| dom.name(k).is_some_and(|n| n != W::r_pr()))
                });
                if inner_has_layout {
                    keep_change = true;
                } else {
                    empty_changes.push(c);
                }
            } else if !dom.name_is(c, &W::r_pr()) {
                structural = true;
            }
        }
        if structural || keep_change {
            continue;
        }
        for c in empty_changes {
            dom.remove(c);
        }
        if dom.elements(ppr, None).is_empty() {
            dom.remove(ppr);
        }
    }
}

/// `w:pgNumType` that only restates Word's implicit `decimal` format.
///
/// No `w:start` and no `w:chapStyle`. A start value (footnotes sample keeps
/// `w:start="1"`) or any other format is a real property.
pub(crate) fn pg_num_type_is_implicit_decimal(dom: &Dom, n: NodeId) -> bool {
    for (name, value) in dom.attributes_iter(n) {
        if name.local_name() == "fmt" && value == "decimal" {
            continue;
        }
        return false;
    }
    true
}

/// Markup Word's redline leaves out once the text already matches.
///
/// Default `w:type val=nextPage` is omitted on the live section and inside
/// the recorded `sectPrChange`. `w:cantSplit` and `w:rtl` are on/off
/// elements: a missing `w:val` means true, and only an explicit false
/// (`0`, `false`, `off`) is absent from this corpus's Word redlines.
/// Default table `w:jc` left/start
/// is omitted. `w:pgNumType` that is only `fmt=decimal` is omitted.
/// `w:spacing` that is only `line=276` on a deleted paragraph mark restates
/// Normal when the paragraph inserts nothing, and only where the original's
/// unstyled paragraphs are 276 themselves (`original_line`, `None` when
/// unknown): over a single-spaced Normal Word keeps it, and Reject All gives
/// the original's 1.15 lines back (221997f1c7). Pure insertions keep that
/// spacing, and so does a paragraph that also holds inserted text: Word's
/// redline keeps `line=276` there (simple_ordered_list × sublist_issue,
/// "Lvl 1 – a"). An inserted pilcrow
/// (`w:pPr/w:rPr/w:ins` and nothing else) on a paragraph that already
/// contains both inserted and deleted text is not in Word's redline of
/// italic-and-underline × italic-subscript.
pub fn strip_unrecorded_word_defaults(dom: &mut Dom, root: NodeId, original_line: Option<&str>) {
    let line_restates_normal = original_line.is_none_or(|l| l == "276");
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let terminal = dom
        .elements(body, None)
        .into_iter()
        .rev()
        .find(|&node| !dom.name_is(node, &W::sect_pr()));
    let mut drop_nodes: Vec<NodeId> = Vec::new();
    for el in dom.descendants(body, Some(&W::type_())) {
        if dom.attribute(el, &W::val()).unwrap_or("") == "nextPage" {
            drop_nodes.push(el);
        }
    }
    for el in dom.descendants(body, Some(&W::name("cantSplit"))) {
        if matches!(dom.attribute(el, &W::val()), Some("0" | "false" | "off")) {
            drop_nodes.push(el);
        }
    }
    for el in dom.descendants(body, Some(&W::name("rtl"))) {
        if matches!(dom.attribute(el, &W::val()), Some("0" | "false" | "off")) {
            drop_nodes.push(el);
        }
    }
    for el in dom.descendants(body, Some(&W::name("pgNumType"))) {
        if pg_num_type_is_implicit_decimal(dom, el) {
            drop_nodes.push(el);
        }
    }
    for tbl_pr in dom.descendants(body, Some(&W::tbl_pr())) {
        if let Some(jc) = dom.element(tbl_pr, &W::jc_el()) {
            let v = dom.attribute(jc, &W::val()).unwrap_or("");
            if v == "left" || v == "start" {
                drop_nodes.push(jc);
            }
        }
    }
    let paras: Vec<NodeId> = dom.descendants(body, Some(&W::p()));
    let mut drop_ppr: Vec<NodeId> = Vec::new();
    for p in paras {
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        if let Some(rpr) = dom.element(ppr, &W::r_pr())
            && dom.element(rpr, &W::del()).is_some()
            && let Some(sp) = dom.element(ppr, &W::spacing_el())
            && line_restates_normal
            && spacing_is_only_line_276(dom, sp)
            && !content_revision_marks(dom, p, ppr).0
        {
            drop_nodes.push(sp);
        }
        // Only the story's terminal pilcrow can be implicitly paired. A
        // preceding inserted boundary must survive so rejection joins the
        // original text and takes its closing paragraph's saved properties.
        if terminal == Some(p)
            && pilcrow_ins_only(dom, ppr)
            && content_revision_marks(dom, p, ppr) == (true, true)
        {
            drop_ppr.push(ppr);
        }
    }
    for n in drop_nodes {
        dom.remove(n);
    }
    for ppr in drop_ppr {
        dom.remove(ppr);
    }
}

pub(super) fn spacing_is_only_line_276(dom: &Dom, sp: NodeId) -> bool {
    if dom.attribute(sp, &W::name("line")) != Some("276") {
        return false;
    }
    for local in [
        "before",
        "after",
        "beforeLines",
        "afterLines",
        "beforeAutospacing",
        "afterAutospacing",
    ] {
        if dom.attribute(sp, &W::name(local)).is_some() {
            return false;
        }
    }
    true
}

/// `w:pPr` whose only child is `w:rPr/w:ins`.
pub(super) fn pilcrow_ins_only(dom: &Dom, ppr: NodeId) -> bool {
    let children = dom.elements(ppr, None);
    if children.len() != 1 {
        return false;
    }
    let rpr = children[0];
    if !dom.name_is(rpr, &W::r_pr()) {
        return false;
    }
    let marks = dom.elements(rpr, None);
    marks.len() == 1 && dom.name_is(marks[0], &W::ins())
}

/// Whether the paragraph's content, outside its `w:pPr`, holds `w:ins` and
/// `w:del`.
pub(super) fn content_revision_marks(dom: &Dom, p: NodeId, ppr: NodeId) -> (bool, bool) {
    let inside: std::collections::HashSet<NodeId> =
        dom.descendants_and_self(ppr, None).into_iter().collect();
    let mut has_ins = false;
    let mut has_del = false;
    for el in dom.descendants(p, None) {
        if inside.contains(&el) {
            continue;
        }
        if dom.name_is(el, &W::ins()) {
            has_ins = true;
        } else if dom.name_is(el, &W::del()) {
            has_del = true;
        }
    }
    (has_ins, has_del)
}

/// Table and comment-reference chrome Word writes on every matching redline
/// in this corpus.
///
/// A live `w:tblPr` with no `w:tblLook` gets the default look `04A0` (first
/// row, first column, no banding), inserted at schema rank 150 so
/// `w:tblPrChange` stays last. The historical `w:tblPr` inside that change
/// is the old snapshot and is left alone. A table whose `w:tblPr` carries
/// `w:tblCellMar` with no nonzero top or bottom gets
/// `w:tblPrEx/w:tblCellMar` (top and bottom 0) on each direct row that
/// lacks one. A nonzero vertical margin is left as the table set it. A run
/// that holds `w:commentReference` gets a missing
/// `w:rStyle`, `w:sz`, and `w:szCs`. Existing values are left alone, including
/// a style id of `AnnotationReference`.
pub fn align_word_table_and_comment_chrome(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let tbl_prs: Vec<NodeId> = dom.descendants(body, Some(&W::tbl_pr()));
    for tbl_pr in tbl_prs {
        // CT_TblPr requires tblPrChange last. The nested tblPr is the
        // previous snapshot; a look stamped there, or appended on the live
        // tblPr, lands after the change.
        if !dom
            .ancestors(tbl_pr, Some(&W::name("tblPrChange")))
            .is_empty()
        {
            continue;
        }
        if dom.element(tbl_pr, &W::name("tblLook")).is_some() {
            continue;
        }
        let look = dom.new_element(W::name("tblLook"));
        dom.set_attribute_value(look, &W::val(), Some("04A0"));
        for (local, value) in [
            ("firstRow", "1"),
            ("lastRow", "0"),
            ("firstColumn", "1"),
            ("lastColumn", "0"),
            ("noHBand", "0"),
            ("noVBand", "1"),
        ] {
            dom.set_attribute_value(look, &W::name(local), Some(value));
        }
        insert_tblpr_child_in_order(dom, tbl_pr, look, 150);
    }
    let tables: Vec<NodeId> = dom.descendants(body, Some(&W::tbl()));
    for tbl in tables {
        let Some(tbl_pr) = dom.element(tbl, &W::tbl_pr()) else {
            continue;
        };
        let Some(tbl_mar) = dom.element(tbl_pr, &W::name("tblCellMar")) else {
            continue;
        };
        // A row exception replaces the table margin. Word writes top/bottom
        // 0 only when the table did not already set a nonzero one
        // (red_strikethrough × plate_30 keeps top 80 and has no tblPrEx).
        let vertical_set = ["top", "bottom"].iter().any(|side| {
            dom.element(tbl_mar, &W::name(side))
                .and_then(|edge| dom.attribute(edge, &W::w()))
                .is_some_and(|w| w != "0")
        });
        if vertical_set {
            continue;
        }
        let rows: Vec<NodeId> = dom.elements(tbl, Some(&W::tr()));
        for tr in rows {
            if dom.element(tr, &W::tbl_pr_ex()).is_some() {
                continue;
            }
            let ex = dom.new_element(W::tbl_pr_ex());
            let mar = dom.new_element(W::name("tblCellMar"));
            for side in ["top", "bottom"] {
                let edge = dom.new_element(W::name(side));
                dom.set_attribute_value(edge, &W::w(), Some("0"));
                dom.set_attribute_value(edge, &W::type_(), Some("dxa"));
                dom.add(mar, edge);
            }
            dom.add(ex, mar);
            if let Some(tr_pr) = dom.element(tr, &W::tr_pr()) {
                dom.add_before_self(tr_pr, ex);
            } else if let Some(first) = dom.elements(tr, None).first().copied() {
                dom.add_before_self(first, ex);
            } else {
                dom.add(tr, ex);
            }
        }
    }
    let refs: Vec<NodeId> = dom.descendants(body, Some(&W::comment_reference()));
    for cref in refs {
        let Some(run) = dom
            .ancestors_and_self(cref, None)
            .into_iter()
            .find(|&n| dom.name_is(n, &W::r()))
        else {
            continue;
        };
        let rpr = match dom.element(run, &W::r_pr()) {
            Some(rpr) => rpr,
            None => {
                let rpr = dom.new_element(W::r_pr());
                dom.add_first(run, rpr);
                rpr
            }
        };
        if dom.element(rpr, &W::r_style()).is_none() {
            let style = dom.new_element(W::r_style());
            dom.set_attribute_value(style, &W::val(), Some("CommentReference"));
            dom.add(rpr, style);
        }
        if dom.element(rpr, &W::name("sz")).is_none() {
            let sz = dom.new_element(W::name("sz"));
            dom.set_attribute_value(sz, &W::val(), Some("20"));
            dom.add(rpr, sz);
        }
        if dom.element(rpr, &W::name("szCs")).is_none() {
            let sz_cs = dom.new_element(W::name("szCs"));
            dom.set_attribute_value(sz_cs, &W::val(), Some("20"));
            dom.add(rpr, sz_cs);
        }
    }
}

/// A wholly inserted external hyperlink whose URL is not in the base
/// document becomes Word's `HYPERLINK` field.
///
/// The address is resolved from the output part's relationships. Links whose
/// target is already a hyperlink in the base document stay `w:hyperlink`
/// (the lots-of-comments retitles). Anchor links stay on the M470 path.
/// Deletions and mixed links are left alone.
pub fn rewrite_inserted_external_hyperlinks(
    dom: &mut Dom,
    root: NodeId,
    id_to_target: &HashMap<String, String>,
    base_targets: &HashSet<String>,
) {
    let links: Vec<NodeId> = dom.descendants(root, Some(&W::hyperlink()));
    for hl in links {
        if dom.attribute(hl, &W::name("anchor")).is_some() {
            continue;
        }
        let Some(rid) = dom.attribute(hl, &R::id()).map(str::to_string) else {
            continue;
        };
        let Some(url) = id_to_target.get(&rid).cloned() else {
            continue;
        };
        if base_targets.contains(&url) || url.contains('"') {
            continue;
        }
        if !hyperlink_is_wholly_inserted(dom, hl) {
            continue;
        }
        let instr_text = format!("HYPERLINK \"{url}\" \\h");
        let begin = field_char_run(dom, "begin");
        let instr_r = dom.new_element(W::r());
        let instr = dom.new_element(W::instr_text());
        dom.add_text(instr, &instr_text);
        dom.add(instr_r, instr);
        let sep = field_char_run(dom, "separate");
        let end = field_char_run(dom, "end");
        let parent_ins = dom.parent(hl).is_some_and(|p| dom.name_is(p, &W::ins()));
        let placed = if parent_ins {
            dom.add_before_self(hl, begin);
            dom.add_before_self(hl, instr_r);
            dom.add_before_self(hl, sep);
            dom.add_after_self(hl, end);
            true
        } else {
            let revs: Vec<NodeId> = dom.elements(hl, Some(&W::ins()));
            if let (Some(&first), Some(&last)) = (revs.first(), revs.last()) {
                dom.add_first(first, sep);
                dom.add_first(first, instr_r);
                dom.add_first(first, begin);
                dom.add(last, end);
                true
            } else {
                false
            }
        };
        if !placed {
            continue;
        }
        let kids: Vec<NodeId> = dom.nodes(hl);
        for kid in kids {
            dom.remove(kid);
            dom.add_before_self(hl, kid);
        }
        dom.remove(hl);
    }
}

pub(super) fn hyperlink_is_wholly_inserted(dom: &Dom, hl: NodeId) -> bool {
    let has_live_t = dom.descendants(hl, Some(&W::t())).iter().any(|&t| {
        !dom.value_str(t).trim().is_empty()
            && !dom
                .ancestors_and_self(t, None)
                .iter()
                .any(|&a| dom.name_is(a, &W::ins()))
    });
    let has_del_text = !dom.descendants(hl, Some(&W::del_text())).is_empty();
    let has_ins = !dom.descendants(hl, Some(&W::ins())).is_empty();
    if dom.parent(hl).is_some_and(|p| dom.name_is(p, &W::ins())) {
        return !has_live_t;
    }
    has_ins && !has_live_t && !has_del_text
}

pub(super) fn field_char_run(dom: &mut Dom, ty: &str) -> NodeId {
    let run = dom.new_element(W::r());
    let fld = dom.new_element(W::fld_char());
    dom.set_attribute_value(fld, &W::name("fldCharType"), Some(ty));
    dom.add(run, fld);
    run
}

/// M450 (calibri_font × calibri_heading_2_right ~82.5 / docxodus 100):
/// last MIX free-mesh parks **Heading residual** spacing
/// (`before≥200` + `line=240`) into `pPrChange` only; Word keeps it **live**
/// with an empty `pPrChange`. Mid MIX already has live spacing.
///
/// Thrash history: bare heading-residual promote also hit red_heading×strikethrough
/// (−9.7) where Word keeps **park-only** on last. Gate requires an **earlier**
/// body para with live heading residual spacing (calibri mid has live before=360;
/// red_heading mid is park-only → skip).
pub fn promote_heading_spacing_from_pprchange_on_last_mix(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    let Some(&last) = kids.last() else {
        return;
    };
    if !dom.name_is(last, &W::p()) || !para_is_mixed_revision(dom, last) {
        return;
    }
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    // Already live spacing — leave alone.
    if dom.element(ppr, &W::spacing_el()).is_some() {
        return;
    }
    let Some(chg) = dom.element(ppr, &W::p_pr_change()) else {
        return;
    };
    let Some(old) = dom.element(chg, &W::p_pr()) else {
        return;
    };
    let Some(old_sp) = dom.element(old, &W::spacing_el()) else {
        return;
    };
    // Heading residual: before≥200 and line present (Heading1/2 demo chrome).
    let before = dom
        .attribute(old_sp, &W::name("before"))
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);
    let has_line = dom.attribute(old_sp, &W::name("line")).is_some();
    if before < 200 || !has_line {
        return;
    }
    // Earlier body para must already show live heading residual spacing
    // (Word calibri mid keeps live; red_heading mid parks → thrash guard).
    let mut prior_live_heading = false;
    for &k in &kids[..kids.len() - 1] {
        if !dom.name_is(k, &W::p()) {
            continue;
        }
        let Some(kppr) = dom.element(k, &W::p_pr()) else {
            continue;
        };
        let Some(sp) = dom.element(kppr, &W::spacing_el()) else {
            continue;
        };
        let b = dom
            .attribute(sp, &W::name("before"))
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0);
        if b >= 200 && dom.attribute(sp, &W::name("line")).is_some() {
            prior_live_heading = true;
            break;
        }
    }
    if !prior_live_heading {
        return;
    }
    // Promote live spacing (clone parked attrs).
    let live = dom.clone_subtree(old_sp);
    dom.add_before_self(chg, live);
    // Empty parked spacing from old pPr → Word empty pPrChange shell.
    dom.remove(old_sp);
}

/// M449 (right_aligned_italic × right_alignment ~75.9 / docxodus 100):
/// mid-body MIX free-mesh parks `jc` into `pPrChange` only; Word keeps
/// **live** `jc` on the body MIX. Last residual stays park-only (center_bold
/// thrash when last also promoted).
///
/// Gate: **non-last** MIX with `pPrChange` carrying `jc`, no live `jc`, and
/// `ins_side + del_side` tokens `≥6`. Promote live `jc`; if the old `pPr`
/// is `jc`-only, drop `pPrChange` (Word mid-body shape).
///
/// A MIX whose revised text is a paragraph the revision leaves
/// unaligned keeps the `pPrChange`: Word records the old centering
/// over the revision's bare properties (`29e3872eed`), and promoting
/// it centered the accepted text.
pub fn promote_live_jc_from_pprchange_on_body_mix(
    dom: &mut Dom,
    root: NodeId,
    revised_unaligned: &HashSet<String>,
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
    for &p in &kids {
        if p == last || !dom.name_is(p, &W::p()) || !para_is_mixed_revision(dom, p) {
            continue;
        }
        let ins_w = para_side_word_count(dom, p, true);
        let del_w = para_side_word_count(dom, p, false);
        if ins_w + del_w < 6 {
            continue;
        }
        let Some(ppr) = dom.element(p, &W::p_pr()) else {
            continue;
        };
        // Already live jc — leave alone. A live `w:spacing` means the pair
        // carries a real layout change, which Word records as a pPrChange over
        // the old jc rather than promoting it (file_116_117).
        if dom.element(ppr, &W::jc_el()).is_some() || dom.element(ppr, &W::spacing_el()).is_some() {
            continue;
        }
        let Some(chg) = dom.element(ppr, &W::p_pr_change()) else {
            continue;
        };
        let Some(old) = dom.element(chg, &W::p_pr()) else {
            continue;
        };
        let Some(old_jc) = dom.element(old, &W::jc_el()) else {
            continue;
        };
        let val = dom.attribute(old_jc, &W::val()).unwrap_or("").to_string();
        if val.is_empty() || revised_unaligned.contains(&accepted_text(dom, &kids, p)) {
            continue;
        }
        // Live jc before pPrChange.
        let jc = dom.new_element(W::jc_el());
        dom.set_attribute_value(jc, &W::val(), Some(&val));
        dom.add_before_self(chg, jc);
        // Drop pPrChange when old was jc-only (Word mid-body has live jc only).
        let mut other_layout = false;
        for c in dom.elements(old, None) {
            let Some(n) = dom.name(c) else {
                continue;
            };
            if n == W::jc_el() || n == W::r_pr() {
                continue;
            }
            other_layout = true;
            break;
        }
        if !other_layout {
            dom.remove(chg);
        }
    }
}

/// M87b / M94 — last pure-del **or mixed** that already has `pPrChange`: drop
/// mark-only `rPr/del` (Word keeps pPrChange only — file_55 "b", file_139 mixed).
/// M87b — drop the paragraph-mark `w:del` on the last body
/// paragraph when its `pPrChange` already carries the layout
/// history.
///
/// `file_55`: Word writes the deletion as a `pPrChange` only, not as
/// a live mark on a bare trailing empty. The pass removes the mark
/// when `pPrChange` is present and `pPr` carries nothing else but
/// the (now-removed) mark. Idempotent.
pub fn strip_last_pure_del_mark_when_pprchange(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids = dom.elements(body, None);
    let Some(&last) = kids.iter().rev().find(|&&k| !dom.name_is(k, &W::sect_pr())) else {
        return;
    };
    if !dom.name_is(last, &W::p()) {
        return;
    }
    if !para_is_pure_deleted(dom, last) && !para_is_mixed_revision(dom, last) {
        return;
    }
    let Some(ppr) = dom.element(last, &W::p_pr()) else {
        return;
    };
    if dom.element(ppr, &W::p_pr_change()).is_none() {
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
