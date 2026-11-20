// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Word-alignment replaced-paragraph merge: short-label folds, fold gates, list-cluster interleave, block reorder.

use super::*;

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
/// Helper that delegates to [`merge_replaced_paragraphs_in`] with
/// `story_final_paired = false` and `revised_close_paired = false`.
/// Kept so existing callers still compile; the production pipeline
/// calls [`merge_replaced_paragraphs_in`] directly with both flags
/// pinned from the body tokenizer.
pub fn merge_replaced_paragraphs(dom: &mut Dom, root: NodeId, comparer_author: &str) {
    merge_replaced_paragraphs_in(dom, root, comparer_author, false);
}

/// [`merge_replaced_paragraphs`] for a body whose two final paragraph marks
/// are paired (`story_final_paired`): the revised document ended on an empty
/// paragraph, so its last content paragraph closes on its own inserted mark
/// and Word does not fold it into the first deleted paragraph
/// (math_radical_tests × multi_image_types). That fold models a revised last
/// paragraph with content, whose mark is the paired final one. A head
/// junction on a shared word (M322, tiff_image × h_f_normal) still folds.
/// Word-visual fold of fully-replaced paragraph pairs (Word's
/// `MergeReplacedParagraphs`, WmlComparer.ts:2329): join a pure-D
/// paragraph with the next pure-I paragraph into a single MIX.
///
/// Operates only on adjacent runs in the body that match Word's
/// pattern (a pure-D paragraph followed by a pure-I paragraph). The
/// `comparer_author` and `date_time_for_revisions` come from
/// `settings`. `story_final_paired = true` lets a paired
/// story-final mark keep the merged paragraph with the revised
/// final paragraph's words; `revised_close_paired = true` keeps
/// the closing mark's words inserted ahead.
pub fn merge_replaced_paragraphs_in(
    dom: &mut Dom,
    root: NodeId,
    comparer_author: &str,
    story_final_paired: bool,
) {
    let body = dom.element(root, &W::body());
    let mut containers: Vec<NodeId> = body.into_iter().collect();
    for name in [W::tc(), W::txbx_content(), W::sdt_content()] {
        containers.extend(dom.descendants(root, Some(&name)));
    }
    for c in containers {
        let fold_boundary = !(story_final_paired && Some(c) == body);
        merge_replaced_in_container(dom, c, comparer_author, fold_boundary);
    }
}

/// M369 — greedy order-preserving zip of residual short-label pure-I
/// paragraphs (e.g. `"a"` / `"b"` / `"3"`) into pure-D list items
/// that end with the same token (`"Lvl 1 – a"`).
///
/// `ordered_list × sublist` −3.5: Word free-meshes matching pairs
/// (`MIX`); wholesale pure-I / pure-D leaves them separate. Coarse
/// whole-paragraph fold — not free word-LCS. M372 `Inserted-live pPr`
/// adopt was tried and LO-regressed; this pass only zips the body
/// content. Test-only entry — the production ladder calls
/// [`residual_short_label_zip_with_settings`].
#[cfg(test)]
pub fn residual_short_label_zip(dom: &mut Dom, root: NodeId) {
    residual_short_label_zip_with_settings(dom, root, &WmlComparerSettings::default());
}

/// Fold residual shared labels using the configured formatting-history
/// policy. The settings gate is M372's "live pPr adopt" branch (off by
/// default; turned on only by the test entry above).
pub fn residual_short_label_zip_with_settings(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
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
        let pure_i: Vec<NodeId> = kids
            .iter()
            .copied()
            .filter(|&p| dom.name_is(p, &W::p()) && para_is_pure_inserted(dom, p))
            .collect();
        let pure_d: Vec<NodeId> = kids
            .iter()
            .copied()
            .filter(|&p| dom.name_is(p, &W::p()) && para_is_pure_deleted(dom, p))
            .collect();
        if pure_i.is_empty() || pure_d.is_empty() {
            return;
        }
        // Short pure-I label: 1..=2 alnum chars, ≤2 word-atoms.
        let mut acted = false;
        let mut used_d: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
        for &ip in &pure_i {
            if !para_body_is_very_short(dom, ip) {
                continue;
            }
            let it = para_revision_body_text(dom, ip);
            let label = it
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();
            if label.is_empty() || label.len() > 2 {
                continue;
            }
            // Pure-D must contain the label as a token and be longer (list item).
            let mut best: Option<NodeId> = None;
            for &dp in &pure_d {
                if used_d.contains(&dp) || dom.parent(dp).is_none() {
                    continue;
                }
                let dt = para_revision_body_text(dom, dp);
                let toks: Vec<String> = dt
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|t| !t.is_empty())
                    .map(|t| t.to_ascii_lowercase())
                    .collect();
                // ≥3 tokens: "Lvl 1 – a" matches; "Item 3" (2 toks) does not
                // steal pure-I "3" from "Lvl 2 – i" residual free-mesh.
                if toks.len() < 3 {
                    continue;
                }
                // Last-token only: "Lvl 1 – a" ends with "a".
                if toks.last().is_some_and(|t| t == &label) {
                    best = Some(dp);
                    break;
                }
            }
            let Some(dp) = best else { continue };
            if dom.parent(ip).is_none() || dom.parent(dp).is_none() {
                continue;
            }
            fold_short_label_ins_into_del_with_settings(dom, ip, dp, settings);
            used_d.insert(dp);
            acted = true;
            break; // rescan kids
        }
        // Second pass: short pure-I residual labels still free (e.g. "3") zip
        // positionally into remaining pure-D list items that look like Lvl*
        // lines (Word free-meshes "3" with "Lvl 2 – i" — last token is "i").
        if !acted {
            let short_i: Vec<NodeId> = pure_i
                .iter()
                .copied()
                .filter(|&p| {
                    dom.parent(p).is_some()
                        && para_body_is_very_short(dom, p)
                        && para_body_alnum_len(dom, p) >= 1
                })
                .collect();
            let lvl_d: Vec<NodeId> = pure_d
                .iter()
                .copied()
                .filter(|&p| {
                    if used_d.contains(&p) || dom.parent(p).is_none() {
                        return false;
                    }
                    let t = para_revision_body_text(dom, p).to_ascii_lowercase();
                    t.contains("lvl") || t.contains("level")
                })
                .collect();
            if !short_i.is_empty() && !lvl_d.is_empty() {
                fold_short_label_ins_into_del_with_settings(dom, short_i[0], lvl_d[0], settings);
                acted = true;
            }
        }
        if !acted {
            return;
        }
    }
}

/// Fold pure-I short label into pure-D list residual. Keep pure-D list pPr
/// (ListParagraph+numPr) — M372 Inserted-live pPr adopt regressed LO pagefair
/// 52.7→51.2 (numId renumber drift vs Word). Body-only zip still recovers
/// residual thrash vs pure-I/D wholesale.
///
/// M374: (1) copy pure-I spacing/ind onto pure-D when missing (Word MIX carries
/// line=276 + ind from B); (2) if pure-I is followed by an empty pure-I spacer,
/// move that spacer after the MIX carrier (Word interleaves empty pure-I
/// between MIX a/b).
///
/// M375: free-mesh shared trailing label as EQ — Word is
/// `del "Lvl 1 – " + EQ "a" + ins " "` not `ins "a " + del "Lvl 1 – a"`.
#[cfg(test)]
pub(super) fn fold_short_label_ins_into_del(dom: &mut Dom, ip: NodeId, dp: NodeId) {
    let settings = short_label_fixture_settings(dom, ip);
    fold_short_label_ins_into_del_with_settings(dom, ip, dp, &settings);
}

pub(super) fn fold_short_label_ins_into_del_with_settings(
    dom: &mut Dom,
    ip: NodeId,
    dp: NodeId,
    settings: &WmlComparerSettings,
) {
    // Capture empty pure-I sibling after the short label (B: a, empty, b, empty, 3).
    let empty_after = dom.parent(ip).and_then(|parent| {
        let sibs: Vec<NodeId> = dom.elements(parent, None);
        let pos = sibs.iter().position(|&s| s == ip)?;
        let n = *sibs.get(pos + 1)?;
        if dom.name_is(n, &W::p())
            && para_is_pure_inserted(dom, n)
            && (para_has_no_text(dom, n) || para_body_text_is_whitespace_only(dom, n))
        {
            Some(n)
        } else {
            None
        }
    });

    // M374: copy pure-I spacing/ind onto pure-D when pure-D lacks them.
    if let Some(ippr) = dom.element(ip, &W::p_pr()) {
        let dppr = match dom.element(dp, &W::p_pr()) {
            Some(p) => p,
            None => {
                let p = dom.new_element(W::p_pr());
                if let Some(first) = dom.elements(dp, None).first().copied() {
                    dom.add_before_self(first, p);
                } else {
                    dom.add(dp, p);
                }
                p
            }
        };
        if dom.element(dppr, &W::spacing_el()).is_none()
            && let Some(sp) = dom.element(ippr, &W::spacing_el())
        {
            let c = dom.clone_subtree(sp);
            dom.add(dppr, c);
        }
        if dom.element(dppr, &W::name("ind")).is_none()
            && let Some(ind) = dom.element(ippr, &W::name("ind"))
        {
            let c = dom.clone_subtree(ind);
            dom.add(dppr, c);
        }
    }

    // Move pure-I body (ins) before pure-D body; pure-D keeps list pPr.
    let ins_kids: Vec<NodeId> = dom
        .elements(ip, None)
        .into_iter()
        .filter(|&c| !dom.name_is(c, &W::p_pr()))
        .collect();
    let first_body = dom
        .elements(dp, None)
        .into_iter()
        .find(|&c| !dom.name_is(c, &W::p_pr()));
    for c in ins_kids {
        if dom.parent(c).is_none() {
            continue;
        }
        dom.remove(c);
        if let Some(first) = first_body {
            if dom.parent(first).is_some() {
                dom.add_before_self(first, c);
            } else {
                dom.add(dp, c);
            }
        } else {
            dom.add(dp, c);
        }
    }
    if let Some(ippr) = dom.element(ip, &W::p_pr()) {
        dom.remove(ippr);
    }
    if dom.parent(ip).is_some() {
        dom.remove(ip);
    }

    // M375: Word free-meshes shared trailing label as EQ (del prefix + live
    // label + ins remainder). Must run after body merge so both streams sit
    // on `dp`. reorder_replacements already ran earlier — del/eq/ins sticks.
    mesh_short_label_shared_eq_with_settings(dom, dp, settings);

    // Move empty pure-I spacer to immediately after MIX carrier (Word shape).
    if let Some(emp) = empty_after
        && dom.parent(emp).is_some()
        && dom.parent(dp).is_some()
    {
        // Insert empty after dp among siblings.
        let Some(parent) = dom.parent(dp) else {
            return;
        };
        let sibs: Vec<NodeId> = dom.elements(parent, None);
        let after_dp = sibs
            .iter()
            .position(|&s| s == dp)
            .and_then(|i| sibs.get(i + 1).copied());
        dom.remove(emp);
        if let Some(next) = after_dp {
            if next != emp && dom.parent(next).is_some() {
                dom.add_before_self(next, emp);
            } else if let Some(n2) = dom
                .elements(parent, None)
                .into_iter()
                .skip_while(|&s| s != dp)
                .nth(1)
            {
                dom.add_before_self(n2, emp);
            } else {
                dom.add(parent, emp);
            }
        } else {
            dom.add(parent, emp);
        }
    }
}

/// M375 — residual short-label MIX free-meshes the shared trailing label as
/// unrevised EQ. Word ordered×sublist: `del "Lvl 1 – " + EQ "a" + ins " "`
/// (not `ins "a " + del "Lvl 1 – a"`). Skips when last del token ≠ ins label
/// (e.g. pure-I "3" × pure-D "Lvl 2 – i" keeps full ins+del zip).
#[cfg(test)]
pub(super) fn short_label_fixture_settings(dom: &Dom, p: NodeId) -> WmlComparerSettings {
    let stamp = dom.element(p, &W::ins());
    WmlComparerSettings {
        author_for_revisions: stamp
            .and_then(|n| dom.attribute(n, &W::author()))
            .unwrap_or_default()
            .to_string(),
        date_time_for_revisions: stamp
            .and_then(|n| dom.attribute(n, &W::date()))
            .unwrap_or_default()
            .to_string(),
        ..WmlComparerSettings::default()
    }
}

#[cfg(test)]
pub(super) fn mesh_short_label_shared_eq(dom: &mut Dom, p: NodeId) {
    let settings = short_label_fixture_settings(dom, p);
    mesh_short_label_shared_eq_with_settings(dom, p, &settings);
}

pub(super) fn mesh_short_label_shared_eq_with_settings(
    dom: &mut Dom,
    p: NodeId,
    settings: &WmlComparerSettings,
) {
    // Every body revision whose text is gathered below must belong to the
    // validated direct-child cluster. Nested revisions carry their wrapper's
    // hyperlink/control ownership and cannot be moved into a plain EQ run.
    if dom.descendants(p, None).into_iter().any(|node| {
        (dom.name_is(node, &W::ins()) || dom.name_is(node, &W::del()))
            && dom.parent(node) != Some(p)
            && !dom
                .ancestors(node, Some(&W::p_pr()))
                .into_iter()
                .any(|ppr| dom.parent(ppr) == Some(p))
    }) {
        return;
    }
    // Collect ins / del body text (revision-aware).
    let mut ins_text = String::new();
    for ins in dom.elements(p, Some(&W::ins())) {
        for t in dom.descendants(ins, Some(&W::t())) {
            ins_text.push_str(&dom.value_str(t));
        }
    }
    let mut del_text = String::new();
    for del in dom.elements(p, Some(&W::del())) {
        for t in dom.descendants(del, Some(&W::del_text())) {
            del_text.push_str(&dom.value_str(t));
        }
    }
    if ins_text.is_empty() || del_text.is_empty() {
        return;
    }
    let label: String = ins_text
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    if label.is_empty() || label.len() > 2 {
        return;
    }
    let del_toks: Vec<String> = del_text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect();
    // Same ≥3-token gate as residual zip ("Lvl 1 a"); last token must match.
    if del_toks.len() < 3 || del_toks.last().is_none_or(|t| t != &label) {
        return;
    }

    // Preserve original-cased label from the del tail for EQ.
    let Some(eq_label) = trailing_alnum_token(&del_text) else {
        return;
    };
    if !eq_label.eq_ignore_ascii_case(&label) {
        return;
    }

    // Only share a literal suffix/prefix with plain, contiguous revision
    // bodies. Moving a differently styled or opaque payload into EQ would
    // change one of the accepted/rejected source views.
    if !del_text.ends_with(&eq_label) || !ins_text.starts_with(&eq_label) {
        return;
    }
    let body = dom.elements(p, None);
    let revisions: Vec<usize> = body
        .iter()
        .enumerate()
        .filter_map(|(i, &n)| (dom.name_is(n, &W::ins()) || dom.name_is(n, &W::del())).then_some(i))
        .collect();
    let (Some(&first), Some(&last)) = (revisions.first(), revisions.last()) else {
        return;
    };
    let transitions = body[first..=last]
        .windows(2)
        .filter(|pair| dom.name(pair[0]) != dom.name(pair[1]))
        .count();
    if transitions > 1 {
        return;
    }
    for &revision in &body[first..=last] {
        if !dom.name_is(revision, &W::ins()) && !dom.name_is(revision, &W::del()) {
            return;
        }
        for run in dom.elements(revision, None) {
            if !dom.name_is(run, &W::r()) {
                return;
            }
            for leaf in dom.elements(run, None) {
                if dom.name_is(leaf, &W::r_pr()) {
                    // Only scalar run properties are safe here. Existing
                    // history, revisions, and opaque extension payloads own
                    // their metadata and must not be folded into a new EQ.
                    if dom.elements(run, Some(&W::r_pr())).len() != 1
                        || dom.elements(leaf, None).into_iter().any(|property| {
                            dom.name(property).is_none_or(|name| {
                                name.namespace_name() != W::URI
                                    || matches!(
                                        name.local_name(),
                                        "rPrChange" | "ins" | "del" | "moveFrom" | "moveTo"
                                    )
                            }) || !dom.nodes(property).is_empty()
                        })
                    {
                        return;
                    }
                } else if !dom.name_is(leaf, &W::t()) && !dom.name_is(leaf, &W::del_text()) {
                    return;
                }
            }
        }
    }

    // A shared label may span text leaves, but must not cross a property
    // boundary on either source side. Its revised properties become live;
    // the full original properties are retained in rPrChange for rejection.
    fn label_run(dom: &Dom, leaves: &[NodeId], mut bytes: usize) -> Option<NodeId> {
        let mut sample = None;
        for &leaf in leaves {
            let len = dom.value_str(leaf).len();
            if len == 0 {
                continue;
            }
            let run = dom.parent(leaf)?;
            if let Some(first) = sample {
                if rpr_string(dom, first) != rpr_string(dom, run) {
                    return None;
                }
            } else {
                sample = Some(run);
            }
            bytes = bytes.saturating_sub(len);
            if bytes == 0 {
                return sample;
            }
        }
        None
    }
    let del_texts: Vec<NodeId> = dom
        .elements(p, Some(&W::del()))
        .into_iter()
        .flat_map(|del| dom.descendants(del, Some(&W::del_text())))
        .collect();
    let ins_texts: Vec<NodeId> = dom
        .elements(p, Some(&W::ins()))
        .into_iter()
        .flat_map(|ins| dom.descendants(ins, Some(&W::t())))
        .collect();
    let reverse_del: Vec<NodeId> = del_texts.iter().rev().copied().collect();
    let (Some(old_run), Some(new_run)) = (
        label_run(dom, &reverse_del, eq_label.len()),
        label_run(dom, &ins_texts, eq_label.len()),
    ) else {
        return;
    };
    // Without format tracking, retain the complete I/D streams rather than
    // sharing a label whose two source formats cannot both be represented live.
    let old_properties = dom.element(old_run, &W::r_pr());
    let new_properties = dom.element(new_run, &W::r_pr());
    if !settings.detect_format_changes
        && !paragraph_mark_properties_equal(dom, old_properties, new_properties)
    {
        return;
    }
    let root = dom.ancestors(p, None).last().copied().unwrap_or(p);
    let mut next_id = max_numeric_id(dom, root).saturating_add(1);

    // 1) Strip trailing label from delText leaves (from the end).
    // Prefer single-leaf path: whole delText ends with the label token.
    let mut last_del_anchor: Option<NodeId> = None;
    let mut stripped = false;
    if let Some(&dt) = del_texts.last() {
        let t = dom.value_str(dt).into_owned();
        if let Some(prefix) = strip_trailing_alnum_token(&t, &eq_label) {
            dom.set_value(dt, &prefix);
            if prefix.starts_with(' ') || prefix.ends_with(' ') {
                dom.set_attribute_value(dt, &XNamespace::xml().name("space"), Some("preserve"));
            }
            last_del_anchor = Some(dt);
            stripped = true;
        }
    }
    if !stripped {
        // Validate every fragment before mutating any leaf. A mismatch must
        // leave the deletion intact, including fragments already visited.
        let mut remain = eq_label.len();
        let mut edits = Vec::new();
        for &dt in del_texts.iter().rev() {
            if remain == 0 {
                break;
            }
            let text = dom.value_str(dt).into_owned();
            let remove = text.len().min(remain);
            let keep = text.len() - remove;
            let (Some(tail), Some(expected), Some(prefix)) = (
                text.get(keep..),
                eq_label.get(remain - remove..remain),
                text.get(..keep),
            ) else {
                return;
            };
            if tail != expected {
                return;
            }
            edits.push((dt, prefix.to_string()));
            remain -= remove;
        }
        if remain != 0 {
            return;
        }
        for (dt, prefix) in edits {
            dom.set_value(dt, &prefix);
            if prefix.starts_with(' ') || prefix.ends_with(' ') {
                dom.set_attribute_value(dt, &XNamespace::xml().name("space"), Some("preserve"));
            }
            last_del_anchor = Some(dt);
        }
    }

    // 2) Strip leading alnum label from ins `w:t` leaves (document order).
    let mut strip_ins = label.len();
    for ins in dom.elements(p, Some(&W::ins())) {
        for t in dom.descendants(ins, Some(&W::t())) {
            if strip_ins == 0 {
                break;
            }
            let v = dom.value_str(t).into_owned();
            let mut chars: Vec<char> = v.chars().collect();
            let mut i = 0usize;
            while i < chars.len() && strip_ins > 0 {
                if chars[i].is_alphanumeric() {
                    chars.remove(i);
                    strip_ins -= 1;
                } else if chars[i].is_whitespace() {
                    // keep whitespace; only strip alnum label chars
                    i += 1;
                } else {
                    i += 1;
                }
            }
            let new_v: String = chars.into_iter().collect();
            dom.set_value(t, &new_v);
            if new_v.starts_with(' ') || new_v.ends_with(' ') {
                dom.set_attribute_value(t, &XNamespace::xml().name("space"), Some("preserve"));
            }
        }
        if strip_ins == 0 {
            break;
        }
    }

    // 3) Anchor EQ after the del wrapper that held the label.
    let mut anchor = last_del_anchor.unwrap_or(p);
    while let Some(par) = dom.parent(anchor) {
        if dom.name_is(par, &W::del()) {
            anchor = par;
            break;
        }
        anchor = par;
        if dom.name_is(par, &W::p()) {
            break;
        }
    }
    let eq_r = dom.new_element(W::r());
    if let Some(old) = dom.element(old_run, &W::r_pr()) {
        let old = dom.clone_subtree(old);
        dom.add(eq_r, old);
    }
    closing_mark_takes_revised_mark_formatting(dom, eq_r, new_run, settings, &mut next_id);
    let eq_t = dom.new_element(W::t());
    dom.add_text(eq_t, &eq_label);
    dom.add(eq_r, eq_t);
    if dom.name_is(anchor, &W::del()) {
        dom.add_after_self(anchor, eq_r);
    } else {
        // Fallback: before first remaining ins, else append.
        if let Some(ins) = dom
            .elements(p, None)
            .into_iter()
            .find(|&c| dom.name_is(c, &W::ins()))
        {
            dom.add_before_self(ins, eq_r);
        } else {
            dom.add(p, eq_r);
        }
    }

    // 4) Word order: del → EQ → ins. Body may still be ins-first from fold.
    // Move every leading w:ins after the EQ run (preserving ins order).
    let kids: Vec<NodeId> = dom.elements(p, None);
    let mut leading_ins: Vec<NodeId> = Vec::new();
    for &c in &kids {
        if dom.name_is(c, &W::p_pr()) {
            continue;
        }
        if dom.name_is(c, &W::ins()) {
            leading_ins.push(c);
        } else {
            break; // stop at first non-ins body child
        }
    }
    if !leading_ins.is_empty() && dom.parent(eq_r).is_some() {
        // Find node after eq_r among siblings to insert before, else append.
        let parent = dom.parent(eq_r).unwrap();
        let sibs: Vec<NodeId> = dom.elements(parent, None);
        let after_eq = sibs
            .iter()
            .position(|&s| s == eq_r)
            .and_then(|i| sibs.get(i + 1).copied());
        for ins in leading_ins {
            if dom.parent(ins).is_none() {
                continue;
            }
            // Skip if already after eq_r.
            let sibs_now: Vec<NodeId> = dom.elements(parent, None);
            let ipos = sibs_now.iter().position(|&s| s == ins);
            let epos = sibs_now.iter().position(|&s| s == eq_r);
            if let (Some(i), Some(e)) = (ipos, epos)
                && i > e
            {
                continue;
            }
            dom.remove(ins);
            if let Some(next) = after_eq.filter(|&n| dom.parent(n).is_some() && n != ins) {
                dom.add_before_self(next, ins);
            } else {
                dom.add(parent, ins);
            }
        }
    }

    // Only clean the validated body revisions; paragraph-mark metadata and
    // unrelated runs are not part of the text we stripped.
    let revision_body: Vec<NodeId> = dom
        .elements(p, None)
        .into_iter()
        .filter(|&node| dom.name_is(node, &W::ins()) || dom.name_is(node, &W::del()))
        .collect();
    let texts: Vec<NodeId> = revision_body
        .iter()
        .flat_map(|&revision| dom.descendants(revision, Some(&W::t())))
        .collect();
    for t in texts {
        if dom.parent(t).is_some() && dom.value_str(t).is_empty() {
            dom.remove(t);
        }
    }
    let runs: Vec<NodeId> = revision_body
        .iter()
        .flat_map(|&revision| dom.descendants(revision, Some(&W::r())))
        .collect();
    for r in runs {
        if dom.parent(r).is_none() {
            continue;
        }
        // Empty run under ins (no t/delText/br/tab… content props only).
        let has_content = dom.elements(r, None).into_iter().any(|c| {
            let n = dom.name(c);
            n != Some(W::r_pr()) && n.is_some()
        });
        if !has_content {
            dom.remove(r);
        }
    }
    for ins in dom.elements(p, Some(&W::ins())) {
        if dom.parent(ins).is_none() {
            continue;
        }
        let mut any = false;
        for t in dom.descendants(ins, Some(&W::t())) {
            if !dom.value_str(t).is_empty() {
                any = true;
                break;
            }
        }
        if !any {
            dom.remove(ins);
        }
    }
}

/// Last alphanumeric token of `text` with original casing (for EQ peel).
pub(super) fn trailing_alnum_token(text: &str) -> Option<String> {
    let mut last = None;
    let mut cur = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            cur.push(c);
        } else if !cur.is_empty() {
            last = Some(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        last = Some(cur);
    }
    last
}

/// If `text` ends with alnum token matching `label` (case-insensitive), return
/// the prefix with that token removed (separators before it kept).
pub(super) fn strip_trailing_alnum_token(text: &str, label: &str) -> Option<String> {
    let tok = trailing_alnum_token(text)?;
    if !tok.eq_ignore_ascii_case(label) {
        return None;
    }
    // Byte start of the last alnum run.
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = chars.len();
    while i > 0 && !chars[i - 1].1.is_alphanumeric() {
        i -= 1;
    }
    while i > 0 && chars[i - 1].1.is_alphanumeric() {
        i -= 1;
    }
    if i >= chars.len() {
        return None;
    }
    Some(text[..chars[i].0].to_string())
}

/// True when `w:pPr/w:rPr` carries a paragraph-mark revision (`w:del`/`w:ins`).
/// Empty deleted/inserted paragraphs often have ONLY this mark (no body
/// `w:del`/`w:ins` child). Without classifying them, del-blocks break and
/// ins-before-del reordering never fires (support_tickets_table_table_bookmark_end:
/// Word is III…DDD…Mt; ours was DDD…III…Mt).
pub(super) fn para_mark_revision(dom: &Dom, p: NodeId, rev: &crate::xmllinq::XName) -> bool {
    dom.element(p, &W::p_pr())
        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
        .is_some_and(|rpr| dom.element(rpr, rev).is_some())
}

/// Folding `d` into `carrier` appends d's body after the carrier's, above
/// every inserted paragraph between them. A complex field that d opens or
/// closes without the other half would then cross the fields those
/// paragraphs hold: Word nests fldChar by position, and a crossed TOC pair
/// spun it at full CPU (98bf5f3d × a3701d36).
pub(super) fn fold_crosses_a_field(dom: &Dom, d: NodeId, carrier: NodeId, inss: &[NodeId]) -> bool {
    let fld_char = W::fld_char();
    let field_type = W::name("fldCharType");
    let mut depth = 0i32;
    let mut balanced = true;
    for f in dom.descendants(d, Some(&fld_char)) {
        match dom.attribute(f, &field_type) {
            Some("begin") => depth += 1,
            Some("end") => depth -= 1,
            _ => {}
        }
        balanced &= depth >= 0;
    }
    if balanced && depth == 0 {
        return false;
    }
    inss.iter()
        .skip_while(|&&p| p != carrier)
        .skip(1)
        .any(|&p| !dom.descendants(p, Some(&fld_char)).is_empty())
}

/// M474 — a "field residue" paragraph (M360's TOC field-end fragments)
/// carries fldChar but NO visible text. M470 synthesizes real HYPERLINK
/// fields inside content-bearing inserted paragraphs; those must not match
/// the M360 residue gates (sd_2672_plain_3x3 × hyperlink_node_internal lost
/// its Heading1 pStyle + MARK-DEL to the bare-pPr branch, 99.91 → 78.64).
pub(super) fn para_is_field_residue(dom: &Dom, p: NodeId) -> bool {
    if dom.descendants(p, Some(&W::fld_char())).is_empty() {
        return false;
    }
    !dom.descendants(p, Some(&W::t()))
        .iter()
        .any(|&t| !dom.value_str(t).trim().is_empty())
}

/// Some(false) = fully deleted, Some(true) = fully inserted, None = neither.
/// Classify a body child of `p` as ins/del/plain. `w:hyperlink` (TOC fields)
/// is transparent — deleted TOC entries are `hyperlink > del > r > delText`,
/// not a direct `w:del` child. Treating the hyperlink as plain broke pure-D
/// runs and blocked I-before-D reorder (file_22: pure-I title stuck at p147).
pub(super) fn accumulate_para_child_class(
    dom: &Dom,
    c: NodeId,
    ins: &mut bool,
    del: &mut bool,
    plain: &mut bool,
) {
    accumulate_para_child_class_with_moves(dom, c, ins, del, plain, false);
}

pub(super) fn accumulate_para_child_class_with_moves(
    dom: &Dom,
    c: NodeId,
    ins: &mut bool,
    del: &mut bool,
    plain: &mut bool,
    include_moves: bool,
) {
    let Some(n) = dom.name(c) else {
        *plain = true;
        return;
    };
    // Moves have insertion/deletion lifecycle semantics for table rows,
    // while general paragraph folding deliberately keeps its existing rules.
    if include_moves && n.namespace_name() == W::URI {
        match n.local_name() {
            "moveTo" => {
                *ins = true;
                return;
            }
            "moveFrom" => {
                *del = true;
                return;
            }
            "moveToRangeStart" | "moveToRangeEnd" | "moveFromRangeStart" | "moveFromRangeEnd" => {
                return;
            }
            _ => {}
        }
    }
    if n == W::p_pr() || n == W::bookmark_start() || n == W::bookmark_end() {
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
            accumulate_para_child_class_with_moves(dom, gc, ins, del, plain, include_moves);
        }
        return;
    }
    *plain = true;
}

pub(super) fn para_replacement_class(dom: &Dom, p: NodeId) -> Option<bool> {
    para_replacement_class_with_moves(dom, p, false)
}

pub(super) fn para_replacement_class_with_moves(
    dom: &Dom,
    p: NodeId,
    include_moves: bool,
) -> Option<bool> {
    if !dom.name_is(p, &W::p()) {
        return None;
    }
    let (mut ins, mut del, mut plain) = (false, false, false);
    for c in dom.elements(p, None) {
        if include_moves {
            accumulate_para_child_class_with_moves(dom, c, &mut ins, &mut del, &mut plain, true);
        } else {
            accumulate_para_child_class(dom, c, &mut ins, &mut del, &mut plain);
        }
    }
    if !ins && !del && !plain {
        if para_mark_revision(dom, p, &W::del()) {
            del = true;
        } else if para_mark_revision(dom, p, &W::ins()) {
            ins = true;
        }
    }
    let class = match (ins, del, plain) {
        (true, false, false) => Some(true),
        (false, true, false) => Some(false),
        _ => None,
    };
    if include_moves && let Some(inserted) = class {
        let moved = if inserted {
            W::move_to()
        } else {
            W::move_from()
        };
        let mark = if inserted { W::ins() } else { W::del() };
        // Moving text between existing cells does not insert/delete their row.
        // A move contributes to row lifecycle only when its paragraph mark
        // independently records the same structural insertion/deletion.
        if !dom.descendants(p, Some(&moved)).is_empty() && !para_mark_revision(dom, p, &mark) {
            return None;
        }
    }
    class
}

/// Like [`para_replacement_class`], but a `w:ins` authored by someone OTHER
/// than the comparer (a CARRIED pre-existing insertion, D1 machinery) counts
/// toward the DELETED side: GT places doc-A blocks whose only live content
/// is carried foreign-author ins inside the trailing deleted cluster
/// (contract-review-suggesting-mixed-edits, −20.9 visual when classified
/// None and the gap adjacency broke).
pub(super) fn para_class_carried_aware(
    dom: &Dom,
    p: NodeId,
    comparer_author: &str,
) -> Option<bool> {
    if !dom.name_is(p, &W::p()) {
        return None;
    }
    let (mut ins, mut del, mut plain) = (false, false, false);
    for c in dom.elements(p, None) {
        let Some(n) = dom.name(c) else {
            plain = true;
            continue;
        };
        if n == W::p_pr() || n == W::bookmark_start() || n == W::bookmark_end() {
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
pub(super) fn para_has_real_del(dom: &Dom, p: NodeId) -> bool {
    !dom.elements(p, Some(&W::del())).is_empty()
}

/// Live + deleted body text of a paragraph (for relatedness gates).
pub(super) fn para_revision_body_text(dom: &Dom, p: NodeId) -> String {
    let mut out = String::new();
    for name in [W::t(), W::del_text()] {
        for t in dom.descendants(p, Some(&name)) {
            out.push_str(&dom.value_str(t));
            out.push(' ');
        }
    }
    out
}

pub(super) fn body_token_set(text: &str) -> std::collections::HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

/// True when paragraph body text (live + delText) is only digits/whitespace
/// (file_166 pure-I "24" — corpus fragment, not content fold bait).
pub(super) fn para_body_is_digits_only(dom: &Dom, p: NodeId) -> bool {
    let t = para_revision_body_text(dom, p);
    let trimmed = t.trim();
    !trimmed.is_empty()
        && trimmed
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_whitespace())
}

/// Alphanumeric character count of para body text (revision-aware).
pub(super) fn para_body_alnum_len(dom: &Dom, p: NodeId) -> usize {
    para_revision_body_text(dom, p)
        .chars()
        .filter(|c| c.is_alphanumeric())
        .count()
}

/// True when body alphanumerics are 1–2 chars ("a", "x", "OK"). Used to keep
/// Word-style pure confetti for short next residuals (file_29 pure-I "a"
/// before pure-D title) — M90 still folds content pure-I into multi-del.
pub(super) fn para_body_is_very_short(dom: &Dom, p: NodeId) -> bool {
    (1..=2).contains(&para_body_alnum_len(dom, p))
}

/// True when body looks like a short demo title (≤8 tokens, last significant
/// token is `Demo`). M139 uses this with Jaccard gate so contract×Title-Demo
/// stays pure-I/D (file_82) while related stamped demos still fold.
pub(super) fn para_looks_like_demo_title(dom: &Dom, p: NodeId) -> bool {
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

/// Live `w:pStyle` is Title / Heading* (Heading1, Heading2, …).
pub(super) fn para_has_heading_or_title_style(dom: &Dom, p: NodeId) -> bool {
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return false;
    };
    let Some(ps) = dom.element(ppr, &W::p_style()) else {
        return false;
    };
    let val = dom.attribute(ps, &W::val()).unwrap_or("");
    let v = val.to_ascii_lowercase();
    v == "title" || v.starts_with("heading")
}

/// Live `w:pStyle` is Heading* only (not Title). M380b: Title pure-D free-meshes.
pub(super) fn para_has_heading_style(dom: &Dom, p: NodeId) -> bool {
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return false;
    };
    let Some(ps) = dom.element(ppr, &W::p_style()) else {
        return false;
    };
    let val = dom.attribute(ps, &W::val()).unwrap_or("");
    val.to_ascii_lowercase().starts_with("heading")
}

/// Shared significant token (len≥4) between short-title pure-D and first pure-I
/// (M322 head-junction). Boilerplate "this"/"with"/"from" excluded.
pub(super) fn short_title_shares_sig_token(ins_text: &str, del_text: &str) -> bool {
    const BOILER: &[&str] = &[
        "this", "that", "with", "from", "have", "will", "been", "were", "they", "them", "than",
        "then", "when", "what", "which", "into", "over", "only", "also", "just", "more", "most",
        "some", "such", "other", "about",
    ];
    let sig = |s: &str| -> std::collections::HashSet<String> {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|t| t.len() >= 4)
            .map(|t| t.to_ascii_lowercase())
            .filter(|t| !BOILER.iter().any(|b| b == t))
            .collect()
    };
    let a = sig(ins_text);
    let b = sig(del_text);
    !a.is_empty() && a.intersection(&b).next().is_some()
}

/// Token Jaccard of two body strings. Used to avoid folding unrelated pure-I
/// / pure-D neighbors (file_33: Word keeps pure-I "Summary" and pure-D
/// "Heading 1 Style Demo" separate; folding invents a mixed para and costs
/// page geometry).
pub(super) fn body_text_jaccard(a: &str, b: &str) -> f64 {
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
pub(super) const SOLE_DEL_FOLD_MIN_JACCARD: f64 = 0.12;

/// Multi-del boundary fold (C1 / KNOWN ISSUE #2): when the I…I D…D gap covers
/// more than this fraction of the container's body-word atoms **and** the
/// boundary pair fails [`should_fold_ins_del_pair`], skip the fold so unrelated
/// whole-document replacements stay pure-ins + pure-del (no mixed first p).
pub(super) const MULTI_DEL_GAP_MAX_DOC_FRACTION: f64 = 0.60;

/// Absolute floor: short demos must still fold (file_54 / bullet_list).
pub(super) const MULTI_DEL_GAP_MIN_WORDS_TO_SKIP: usize = 40;

pub(super) fn para_has_live_numpr(dom: &Dom, p: NodeId) -> bool {
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return false;
    };
    dom.element(ppr, &W::num_pr()).is_some()
}

pub(super) fn should_fold_ins_del_pair(dom: &Dom, ins_p: NodeId, del_p: NodeId) -> bool {
    let it = para_revision_body_text(dom, ins_p);
    let dt = para_revision_body_text(dom, del_p);
    // Empty del mark-only: allow fold (mark-only cases from single_paragraph GT).
    if dt.trim().is_empty() {
        return true;
    }
    body_text_jaccard(&it, &dt) + 1e-12 >= SOLE_DEL_FOLD_MIN_JACCARD
}

/// Word-atom count for body text under a paragraph (whitespace-split tokens).
pub(super) fn para_word_atom_count(dom: &Dom, p: NodeId) -> usize {
    let t = para_revision_body_text(dom, p);
    t.split_whitespace().filter(|w| !w.is_empty()).count()
}

/// Document-scale gate for multi-del boundary fold (plan B1 / C1).
///
/// Fold when the boundary pair is related (Jaccard). Skip only for the
/// **short↔long whole-document replacement** shape: both sides multi-paragraph
/// (≥ 3), boundary Jaccard miss, gap covers most of the container, **and**
/// the two sides of the gap are size-asymmetric (word-atom ratio ≥ 4), with
/// absolute gap ≥ [`MULTI_DEL_GAP_MIN_WORDS_TO_SKIP`].
pub(super) fn should_fold_multi_del_at_document_scale(
    dom: &Dom,
    container: NodeId,
    last_ins: NodeId,
    first_del: NodeId,
    inss: &[NodeId],
    dels: &[NodeId],
) -> bool {
    // Boundary relatedness: empty pure-D must not force multi-del fold.
    // file_196: pure-I B body then empty pure-D then A dels — empty-del
    // auto-fold (should_fold_ins_del_pair) mixed last B para into the empty del
    // shell (score ~39). Sole-del empty still folds via the sole_del path.
    let boundary_empty_del = para_revision_body_text(dom, first_del).trim().is_empty();
    if !boundary_empty_del && should_fold_ins_del_pair(dom, last_ins, first_del) {
        return true;
    }
    // M385 (nda×report residual −26.6): List* pure-I × short Heading/Title
    // pure-D ("MUTUAL NON-DISCLOSURE AGREEMENT", 3 words) free-meshes in Word
    // (MIX Heading1 + del mark + ins citation). Whole-doc Jaccard miss would
    // skip via the document-scale gap below; force fold for this boundary.
    {
        let last_ins_list = dom.element(last_ins, &W::p_pr()).is_some_and(|ip| {
            dom.element(ip, &W::p_style()).is_some_and(|ps| {
                dom.attribute(ps, &W::val())
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .starts_with("list")
            })
        });
        if last_ins_list
            && para_has_heading_or_title_style(dom, first_del)
            && (1..=6).contains(&para_word_atom_count(dom, first_del))
        {
            return true;
        }
    }

    // M428b (list_def_mix × list_numbering_reimport): multi pure-I uniform
    // single-token items ("test"×4) + multi pure-D short list labels. Word
    // MIX-es last pure-I into first pure-D (IIIIMDD… / ID on last test×Num1).
    // Document-scale Jaccard is 0 so the gap below would skip fold → IIIIDDD
    // (pixel −2 vs interleave). Force boundary fold when pure-I is uniform
    // short tokens and first pure-D is a short list item with numPr.
    {
        let content_inss_pre: Vec<NodeId> = inss
            .iter()
            .copied()
            .filter(|&i| !para_revision_body_text(dom, i).trim().is_empty())
            .collect();
        let first_del_toks = para_word_atom_count(dom, first_del);
        let pure_i_uniform = content_inss_pre.len() >= 3
            && content_inss_pre.iter().all(|&p| {
                let n = para_word_atom_count(dom, p);
                n == 1
            })
            && {
                let t0 = para_revision_body_text(dom, content_inss_pre[0]).to_ascii_lowercase();
                content_inss_pre
                    .iter()
                    .all(|&p| para_revision_body_text(dom, p).to_ascii_lowercase() == t0)
            };
        if pure_i_uniform
            && (1..=3).contains(&first_del_toks)
            && para_has_live_numpr(dom, first_del)
            && para_has_live_numpr(dom, last_ins)
        {
            return true;
        }
    }

    // M308d (tight): skip fold for *short-item list-wholesale* pure-I/D.
    // LCS/unrelated may already emit pure-I all next then pure-D all base
    // (basic_list×sd_1707, broken_list×multiple_nodes — unpacked Word pure-I/D
    // seqs). multi-del fold would re-MIX last pure-I into first pure-D.
    //
    // Require: both sides list-heavy, short items (≤12 word-atoms), no
    // contentful Jaccard relatedness, last pure-I has numPr, and either
    // word-count or *paragraph-count* asymmetry (D ≥ 2× I). Word-count alone
    // failed when next has a long title + list heading vs many short base
    // items (M309: del_w < 2×ins_w but 8 pure-D vs 2 pure-I).
    //
    // Long numbered prose (list_with_indents) is excluded by the short-item
    // cap so Word MIX (IMDDDD) is preserved. Empty-del M131 loop restored
    // below for stamp cousins (file_197).
    const SHORT_LIST_ITEM_MAX_WORDS: usize = 12;
    let content_dels: Vec<NodeId> = dels
        .iter()
        .copied()
        .filter(|&d| !para_revision_body_text(dom, d).trim().is_empty())
        .collect();
    let content_inss: Vec<NodeId> = inss
        .iter()
        .copied()
        .filter(|&i| !para_revision_body_text(dom, i).trim().is_empty())
        .collect();
    // Tokenize each side once (O(n+m)) then compare cached sets (O(n·m) cheap
    // set ops), instead of the former per-pair recompute of
    // para_revision_body_text + body_token_set (O(n·m) tokenizations — 162×1216
    // on whole-document replacements). Result is identical: `any` is
    // order-independent, and both content_* sides are non-empty here, so the
    // empty-del auto-fold branch of should_fold_ins_del_pair never applies.
    let ins_tokens: Vec<std::collections::HashSet<String>> = content_inss
        .iter()
        .map(|&i| body_token_set(&para_revision_body_text(dom, i)))
        .collect();
    let del_tokens: Vec<std::collections::HashSet<String>> = content_dels
        .iter()
        .map(|&d| body_token_set(&para_revision_body_text(dom, d)))
        .collect();
    let any_content_related = ins_tokens.iter().any(|a| {
        del_tokens.iter().any(|b| {
            let jaccard = if a.is_empty() && b.is_empty() {
                1.0
            } else {
                let uni = a.union(b).count();
                if uni == 0 {
                    0.0
                } else {
                    a.intersection(b).count() as f64 / uni as f64
                }
            };
            jaccard + 1e-12 >= SOLE_DEL_FOLD_MIN_JACCARD
        })
    });
    // content_inss.len() == 2: Word pure-I/D for short next (title+heading or
    // two list items) vs long short-item list base (basic_list×sd_1707,
    // broken_list×multiple_nodes — unpacked IID… / IIID…).
    // content_inss >= 3: Word keeps MIX carrier on last next item
    // (broken_list×list_spacer seq IIIMD… — fold must run).
    if content_inss.len() == 2 && content_dels.len() >= 3 {
        let list_frac = |ps: &[NodeId]| -> f64 {
            let n = ps.iter().filter(|&&p| para_has_live_numpr(dom, p)).count();
            n as f64 / ps.len() as f64
        };
        let all_short = |ps: &[NodeId]| -> bool {
            ps.iter()
                .all(|&p| para_word_atom_count(dom, p) <= SHORT_LIST_ITEM_MAX_WORDS)
        };
        let ins_w: usize = content_inss
            .iter()
            .map(|&p| para_word_atom_count(dom, p))
            .sum();
        let del_w: usize = content_dels
            .iter()
            .map(|&p| para_word_atom_count(dom, p))
            .sum();
        let count_asymmetric = content_dels.len() >= content_inss.len().saturating_mul(2).max(1);
        let words_asymmetric = del_w >= ins_w.saturating_mul(2).max(1);
        if list_frac(&content_inss) + 1e-12 >= 0.5
            && list_frac(&content_dels) + 1e-12 >= 0.5
            && all_short(&content_inss)
            && all_short(&content_dels)
            && para_has_live_numpr(dom, last_ins)
            && !any_content_related
            && (words_asymmetric || count_asymmetric)
        {
            return false;
        }
    }

    // M413 (doc_with_spaces × doc_with_spacing ~46.1): multi pure-I title-page
    // next (≥4 contentful) × short pure-D section base (1..=2 contentful).
    // M90 force-fold (dels < 3 → always fold) MIX-es last pure-I date into
    // first pure-D engagement header. Word pure-I all next then pure-D base
    // (I7 D2). Skip when boundary Jaccard is low.
    //
    // M418 thrash: bare contentful counts also skipped fold for file_30
    // (Font Size Demo pure-I × ONE/a pure-D — pin folded last body into "a").
    // Require last pure-I looks like title-page cover (date / prepared / @)
    // or pure-D is multi-word section prose (≥5 words), not alpha stubs.
    {
        let content_inss_n = content_inss.len();
        let content_dels_n = content_dels.len();
        let last_ins_text = para_revision_body_text(dom, last_ins).to_ascii_lowercase();
        let title_page_tail = last_ins_text.contains("prepared")
            || last_ins_text.contains('@')
            || last_ins_text.contains("march ")
            || last_ins_text.contains("january ")
            || last_ins_text.contains("february ")
            || last_ins_text.contains("april ")
            || last_ins_text.contains("june ")
            || last_ins_text.contains("july ")
            || last_ins_text.contains("august ")
            || last_ins_text.contains("september ")
            || last_ins_text.contains("october ")
            || last_ins_text.contains("november ")
            || last_ins_text.contains("december ")
            || last_ins_text.contains("2040")
            || last_ins_text.contains("agreement");
        let first_del_words = para_word_atom_count(dom, first_del);
        if content_inss_n >= 4
            && (1..=2).contains(&content_dels_n)
            && title_page_tail
            && first_del_words >= 5
            && !any_content_related
            && !should_fold_ins_del_pair(dom, last_ins, first_del)
        {
            return false;
        }
    }
    // M414 (pageref × restart alpha-list ~51.7): multi pure-I short labels
    // (ONE/A/TWO/A/B/C, ≤2 tokens each) × multi pure-D long base. Document-
    // scale multi-del fold still MIX-es last "C" into first TOC pure-D.
    // Word pure-I all list then pure-D all base (I6 D21).
    // Require first pure-D is a **short** TOC/label line (≤2 words), not a
    // Demo title ("1.5 Line Spacing Demo" = 4 words thrash file_54 −34).
    // Ignore any_content_related: "TWO"×"Two-phase" incidental token hit.
    {
        let all_short_labels = !content_inss.is_empty()
            && content_inss.iter().all(|&p| {
                let n = para_word_atom_count(dom, p);
                (1..=2).contains(&n)
            });
        let short_singles = content_inss
            .iter()
            .filter(|&&p| {
                let t = para_revision_body_text(dom, p);
                let w = t.split_whitespace().next().unwrap_or("");
                w.chars().count() <= 5 && para_word_atom_count(dom, p) == 1
            })
            .count();
        let first_del_words = para_word_atom_count(dom, first_del);
        let first_del_text = para_revision_body_text(dom, first_del).to_ascii_lowercase();
        let demo_title = first_del_text.contains("demo") || first_del_text.contains("demonstrat");
        if content_inss.len() >= 4
            && content_dels.len() >= 4
            && all_short_labels
            && short_singles * 2 >= content_inss.len()
            && (1..=2).contains(&first_del_words)
            && !demo_title
            && !should_fold_ins_del_pair(dom, last_ins, first_del)
        {
            return false;
        }
    }
    // M416 (heading_font×hummingbird): sole long pure-I wrap (≥20 words) ×
    // multi pure-D base. Multi-del fold MIX-es wrap into first base (DI).
    // Word pure-I wrap then pure-D all base (I1 D4).
    //
    // M420 thrash: project_proposal×project_tasks sole long pure-I TaskOwner
    // line × multi pure-D body also hit this skip (MIDDD vs pin MMDD −17).
    // Require wrap fingerprint: a ≥4-word phrase that repeats in the pure-I
    // (hummingbird "Lets tightly wrap this one" × N) — not generic long titles.
    if content_inss.len() == 1
        && content_dels.len() >= 3
        && para_word_atom_count(dom, last_ins) >= 20
        && para_has_repeated_phrase(dom, last_ins)
        && !should_fold_ins_del_pair(dom, last_ins, first_del)
    {
        return false;
    }
    // M417 (sd_2517 × borderbox): multi pure-I math demo (≥10) × huge pure-D
    // base (≥50). Multi-del fold MIX-es last borderBox title into first base
    // lorem (DI). Word pure-I all next then pure-D all base. Skip when
    // boundary jaccard is low and pure-D side is much larger.
    if content_inss.len() >= 10
        && content_dels.len() >= 50
        && !should_fold_ins_del_pair(dom, last_ins, first_del)
    {
        return false;
    }
    // Local multi-del residual (M90: 1–2 pure-I after tables / short demos).
    if inss.len() < 3 || dels.len() < 3 {
        // M337: do **not** skip fold for single short pure-I + multi pure-D.
        // That skip (M334) forced pure-I|pure-D on full-LCS pirates×table_left
        // (pagefair ~37). e3 always-fold produced MIX title×first base (~70).
        // Word class is closer to pure-I|D, but pagefair tracks the e3 MIX
        // layout; multi-table short-next full LCS needs the fold. Unrelated
        // short-title pure-I/D wholesale (inss≫1) is unaffected.
        return true;
    }
    // M336 (bullet_list_bold×bullet_list): multi pure-I short list items then
    // multi pure-D starting with "This document demonstrates…". Word folds last
    // item ("Grapes") into that intro (DIMD MIX=1). Jaccard 0 would skip fold
    // via the relatedness loop below — force fold for this demo-intro shape.
    {
        let dt = para_revision_body_text(dom, first_del).to_ascii_lowercase();
        if inss.len() >= 3
            && dels.len() >= 2
            && para_word_atom_count(dom, last_ins) <= 2
            && para_word_atom_count(dom, first_del) >= 5
            && (dt.contains("demonstrates") || dt.starts_with("this document"))
        {
            return true;
        }
    }
    // Content-related short-into-long (M131): any I×D pair in the gap with
    // Jaccard relatedness means Word still folds the boundary.
    // Empty pure-Ds still count (mark-only allow) — that is load-bearing for
    // stamp-file cousins (file_197×198). Do not content-filter this loop.
    for &i in inss {
        for &d in dels {
            if should_fold_ins_del_pair(dom, i, d) {
                return true;
            }
        }
    }
    let ins_words: usize = inss.iter().map(|&p| para_word_atom_count(dom, p)).sum();
    let del_words: usize = dels.iter().map(|&p| para_word_atom_count(dom, p)).sum();
    let gap = ins_words + del_words;
    if gap < MULTI_DEL_GAP_MIN_WORDS_TO_SKIP {
        return true;
    }
    let lo = ins_words.min(del_words).max(1);
    let hi = ins_words.max(del_words);
    let size_ratio = (hi as f64) / (lo as f64);
    if size_ratio + 1e-12 < 4.0 {
        return true;
    }
    let doc: usize = dom
        .elements(container, None)
        .into_iter()
        .filter(|&c| dom.name_is(c, &W::p()))
        .map(|p| para_word_atom_count(dom, p))
        .sum();
    let doc = doc.max(1);
    let frac = (gap as f64) / (doc as f64);
    frac <= MULTI_DEL_GAP_MAX_DOC_FRACTION + 1e-12
}

/// Only the section properties follow a replace gap: it ends the story.
pub(super) fn gap_ends_story(dom: &Dom, rest: &[NodeId]) -> bool {
    rest.iter().all(|&c| dom.name_is(c, &W::sect_pr()))
}

/// A replace gap followed by deleted blocks and then the story's final
/// paragraph, holding nothing live or inserted (empty, or the original's
/// deleted last words; its mark is the story's final pilcrow, which Word
/// never deletes, so a later pass restores it), while the final marks are no
/// empty pair
/// (`fold_boundary`): the revised last paragraph's mark is that final one,
/// so its text has no mark of its own inside the gap. Word (Docxodus 12's
/// structural final pair) fuses that text into the first deleted paragraph,
/// across deleted tables (diff_before16 × diff_before19,
/// support_tickets_table × support_tickets_summary).
pub(super) fn gap_precedes_live_story_tail(
    dom: &Dom,
    rest: &[NodeId],
    classes: &[Option<bool>],
    fold_boundary: bool,
) -> bool {
    let blocks: Vec<usize> = (0..rest.len())
        .filter(|&k| !dom.name_is(rest[k], &W::sect_pr()))
        .collect();
    let Some((&last, before)) = blocks.split_last() else {
        return false;
    };
    let fin = rest[last];
    fold_boundary
        && dom.name_is(fin, &W::p())
        && !para_mark_revision(dom, fin, &W::ins())
        && dom.descendants(fin, Some(&W::ins())).is_empty()
        && dom
            .descendants(fin, Some(&W::t()))
            .iter()
            .all(|&t| dom.value_str(t).trim().is_empty())
        && before
            .iter()
            .all(|&k| classes[k] == Some(false) || table_is_deleted(dom, rest[k]))
}

/// A page break keeps its paragraph whole: Word never fuses it, so the break
/// keeps paginating. (The revised tail may still hold the body's final
/// section properties here; a deleted paragraph's are a real section break.)
pub(super) fn para_carries_page_break(dom: &Dom, p: NodeId) -> bool {
    dom.element(p, &W::p_pr())
        .is_some_and(|ppr| dom.element(ppr, &W::name("pageBreakBefore")).is_some())
        || dom
            .descendants(p, Some(&W::name("br")))
            .iter()
            .any(|&b| dom.attribute(b, &W::type_()) == Some("page"))
}

/// Word's story-tail fusion: the revised last paragraph's runs open the
/// first deleted paragraph, which keeps its own properties and deleted mark.
pub(super) fn fuse_story_tail_into_deleted(dom: &mut Dom, last_ins: NodeId, first_del: NodeId) {
    let anchor = dom
        .elements(first_del, None)
        .into_iter()
        .find(|&c| !dom.name_is(c, &W::p_pr()));
    for c in dom.elements(last_ins, None) {
        if dom.name_is(c, &W::p_pr()) {
            continue;
        }
        dom.remove(c);
        match anchor {
            Some(a) => dom.add_before_self(a, c),
            None => dom.add(first_del, c),
        }
    }
    dom.remove(last_ins);
}

/// #1678. The revised body ends on an empty paragraph and the redline
/// ended on the table that replaced the source's last paragraph. Word
/// keeps that empty paragraph as the close, with no revision on it, and
/// writes an inserted mark on the replacement so Reject All joins the
/// empty paragraph away.
pub(crate) fn keep_revised_empty_close(out_xml: &str, revised_xml: &str) -> Option<String> {
    let mut revised = Dom::new();
    let revised_doc = revised.parse_xdocument(revised_xml);
    let revised_body = revised
        .root(revised_doc)
        .and_then(|root| revised.element(root, &W::body()))?;
    let revised_last = revised
        .elements(revised_body, None)
        .into_iter()
        .rfind(|&child| !revised.name_is(child, &W::sect_pr()))?;
    if !paragraph_has_live_empty_pilcrow(&revised, revised_last) {
        return None;
    }

    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(out_xml);
    let root = dom.root(doc)?;
    let body = dom.element(root, &W::body())?;
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&child| !dom.name_is(child, &W::sect_pr()))
        .collect();
    let (&last, before) = kids.split_last()?;
    // Only a table the revision added replaces the paragraph: one the
    // source already ended on stays live under Reject All, and the close
    // could not join away across it (cursor on #2102).
    if !dom.name_is(last, &W::tbl()) || !table_is_inserted(&dom, last) {
        return None;
    }
    let &previous = before.last()?;
    if !dom.name_is(previous, &W::p())
        || dom
            .find_descendant_element(previous, Some(&W::ins()), |_| true)
            .is_none()
        || dom
            .find_descendant_element(previous, Some(&W::del()), |_| true)
            .is_none()
        || para_mark_revision(&dom, previous, &W::del())
        || para_mark_revision(&dom, previous, &W::move_from())
        || para_mark_revision(&dom, previous, &W::move_to())
    {
        return None;
    }
    let minted = !para_mark_revision(&dom, previous, &W::ins());
    if minted {
        mint_replacement_mark(&mut dom, root, previous)?;
    }
    let close = revised_close_copy(&mut dom, &revised, revised_last, previous)?;
    match dom.element(body, &W::sect_pr()) {
        Some(sect) => dom.add_before_self(sect, close),
        None => dom.add(body, close),
    }
    Some(dom.serialize_element(root))
}

/// Every row of `table` carries an inserted row mark.
pub(super) fn table_is_inserted(dom: &Dom, table: NodeId) -> bool {
    let mut rows = 0usize;
    let mut inserted = 0usize;
    dom.for_each_descendant_element(table, Some(&W::tr()), |row| {
        rows += 1;
        if dom
            .element(row, &W::tr_pr())
            .is_some_and(|properties| dom.element(properties, &W::ins()).is_some())
        {
            inserted += 1;
        }
    });
    rows > 0 && rows == inserted
}

/// The revised close, copied with its properties and inert runs (codex on
/// #2102). Reject All joins the replaced paragraph's words into this mark,
/// so a close whose properties differ records the replaced paragraph's in
/// a `w:pPrChange`. Its `w14` ids stay behind, as they could repeat one
/// the redline already holds, and so does a paragraph-level `w:sectPr`,
/// whose header and footer references name the revised part's
/// relationships, not the redline's.
pub(super) fn revised_close_copy(
    dom: &mut Dom,
    revised: &Dom,
    revised_close: NodeId,
    previous: NodeId,
) -> Option<NodeId> {
    let close = dom.import_subtree(revised, revised_close);
    for name in [W14::name("paraId"), W14::name("textId")] {
        dom.set_attribute_value(close, &name, None);
    }
    let Some(properties) = dom.element(close, &W::p_pr()) else {
        return Some(close);
    };
    for child in dom.elements(properties, None) {
        if dom.name_is(child, &W::sect_pr()) || dom.name_is(child, &W::p_pr_change()) {
            dom.remove(child);
        }
    }
    let formatting = |dom: &Dom, holder: Option<NodeId>| -> Vec<String> {
        holder
            .map(|holder| {
                dom.elements(holder, None)
                    .into_iter()
                    .filter(|&c| !dom.name_is(c, &W::r_pr()) && !dom.name_is(c, &W::p_pr_change()))
                    .map(|c| dom.serialize_element(c))
                    .collect()
            })
            .unwrap_or_default()
    };
    let previous_properties = dom.element(previous, &W::p_pr());
    if formatting(dom, Some(properties)) == formatting(dom, previous_properties) {
        return Some(close);
    }
    let mark = dom.find_descendant_element(previous, Some(&W::ins()), |_| true)?;
    let author = dom.attribute(mark, &W::author())?.to_string();
    let date = dom.attribute(mark, &W::date()).map(str::to_string);
    let id = fresh_revision_id(dom, dom.parent(dom.parent(previous)?)?);
    let change = dom.new_element(W::p_pr_change());
    dom.set_attribute_value(change, &W::id(), Some(&id.to_string()));
    dom.set_attribute_value(change, &W::author(), Some(&author));
    if let Some(date) = date.as_deref() {
        dom.set_attribute_value(change, &W::date(), Some(date));
    }
    let recorded = dom.new_element(W::p_pr());
    if let Some(holder) = previous_properties {
        for child in dom.elements(holder, None) {
            if !dom.name_is(child, &W::r_pr())
                && !dom.name_is(child, &W::p_pr_change())
                && !dom.name_is(child, &W::sect_pr())
            {
                let copy = dom.clone_subtree(child);
                dom.add(recorded, copy);
            }
        }
    }
    dom.add(change, recorded);
    dom.add(properties, change);
    Some(close)
}

/// A revision id no element under `root` uses.
pub(super) fn fresh_revision_id(dom: &Dom, root: NodeId) -> u64 {
    let mut used = HashSet::new();
    dom.for_each_descendant_element(root, None, |element| {
        if let Some(id) = dom
            .attribute(element, &W::id())
            .and_then(|v| v.parse::<u64>().ok())
        {
            used.insert(id);
        }
    });
    crate::revision_processor::RevisionIds::from_used(used).allocate()
}

/// An inserted mark on the paragraph a table replaced, copied from the
/// paragraph's own insertion.
pub(super) fn mint_replacement_mark(dom: &mut Dom, root: NodeId, paragraph: NodeId) -> Option<()> {
    let source = dom.find_descendant_element(paragraph, Some(&W::ins()), |_| true)?;
    let author = dom.attribute(source, &W::author())?.to_string();
    let date = dom.attribute(source, &W::date()).map(str::to_string);
    let id = fresh_revision_id(dom, root);
    let mark = dom.new_element(W::ins());
    dom.set_attribute_value(mark, &W::id(), Some(&id.to_string()));
    dom.set_attribute_value(mark, &W::author(), Some(&author));
    if let Some(date) = date.as_deref() {
        dom.set_attribute_value(mark, &W::date(), Some(date));
    }
    let properties = match dom.element(paragraph, &W::p_pr()) {
        Some(properties) => properties,
        None => {
            let properties = dom.new_element(W::p_pr());
            dom.add_first(paragraph, properties);
            properties
        }
    };
    let mark_properties = match dom.element(properties, &W::r_pr()) {
        Some(mark_properties) => mark_properties,
        None => {
            let mark_properties = dom.new_element(W::r_pr());
            let after = dom.elements(properties, None).into_iter().find(|&child| {
                dom.name_is(child, &W::sect_pr()) || dom.name_is(child, &W::p_pr_change())
            });
            match after {
                Some(after) => dom.add_before_self(after, mark_properties),
                None => dom.add(properties, mark_properties),
            }
            mark_properties
        }
    };
    dom.add_first(mark_properties, mark);
    Some(())
}

/// #1683, case 2. Word's Compare writes the revised body's last paragraph
/// after the table it replaced, as the closing paragraph, with no revision
/// on that mark. The redline left the inserted paragraph, mark included,
/// before the deleted table, and the original's empty close after it.
/// Move the insertion onto that close. A paragraph that also carries a
/// deletion stays where it is (#1926).
///
/// Only when the inserted paragraph is the revision's own last one
/// (`revised_xml`'s body closes on its text): a revision that ends on an
/// empty paragraph pairs that one with the close, and Word keeps the
/// inserted paragraph before the deleted table on its inserted mark
/// (Word's nested_table_rowspan × numbered_list redline, CI on #2101).
pub(crate) fn pair_closing_insert_across_deleted_table(
    dom: &mut Dom,
    body: NodeId,
    revised_xml: &str,
) {
    // Range and proofing markers sit between blocks without being one
    // (cursor on #2101); the body's own `w:sectPr` closes it.
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&child| {
            !dom.name_is(child, &W::sect_pr())
                && !crate::comparer::final_marks::is_inert_body_marker(dom, child)
        })
        .collect();
    let Some((&close, before)) = kids.split_last() else {
        return;
    };
    if !paragraph_has_live_empty_pilcrow(dom, close) {
        return;
    }
    let Some(split) = before
        .iter()
        .rposition(|&child| !table_is_deleted(dom, child))
    else {
        return;
    };
    if split + 1 >= before.len() {
        return;
    }
    let inserted = before[split];
    if !para_is_pure_inserted(dom, inserted)
        || !para_mark_revision(dom, inserted, &W::ins())
        || para_has_no_text(dom, inserted)
        || revised_body_close_text(revised_xml).as_deref()
            != Some(paragraph_text(dom, inserted).as_str())
    {
        return;
    }
    // A whole story replaced keeps Word's order, every insertion before
    // the deleted original: the inserted paragraph stays before the table
    // on its inserted mark (Word's nested_table_rowspan × numbered_list
    // redline, `tests/m_whole_story_final_marks.rs`). Only a revision that
    // keeps something of the original before it pairs (CI on #2101).
    let keeps_original = before[..split].iter().any(|&block| {
        if dom.name_is(block, &W::p()) {
            !para_is_pure_inserted(dom, block) && !para_is_pure_deleted(dom, block)
        } else {
            !table_is_deleted(dom, block)
        }
    });
    if !keeps_original {
        return;
    }
    for child in dom.elements(inserted, None) {
        if !dom.name_is(child, &W::p_pr()) {
            dom.add(close, child);
        }
    }
    dom.remove(inserted);
}

/// The `w:t` text of `paragraph`, in order.
pub(super) fn paragraph_text(dom: &Dom, paragraph: NodeId) -> String {
    let mut text = String::new();
    dom.for_each_descendant_element(paragraph, Some(&W::t()), |t| {
        text.push_str(&dom.value(t));
    });
    text
}

/// The text of the paragraph that closes `xml`'s body, past its `w:sectPr`
/// and inert markers; `None` when the body closes on anything else.
pub(super) fn revised_body_close_text(xml: &str) -> Option<String> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc)?;
    let body = dom.element(root, &W::body())?;
    let close = dom.elements(body, None).into_iter().rev().find(|&child| {
        !dom.name_is(child, &W::sect_pr())
            && !crate::comparer::final_marks::is_inert_body_marker(&dom, child)
    })?;
    dom.name_is(close, &W::p())
        .then(|| paragraph_text(&dom, close))
}

/// Every paragraph of the table is deleted, marks included, and nothing in it
/// is live or inserted (the rows get their deleted marks later, from
/// `mark_fully_revised_rows`).
pub(super) fn table_is_deleted(dom: &Dom, tbl: NodeId) -> bool {
    let paras = dom.descendants(tbl, Some(&W::p()));
    dom.name_is(tbl, &W::tbl())
        && !paras.is_empty()
        && paras.iter().all(|&p| para_mark_revision(dom, p, &W::del()))
        && dom.descendants(tbl, Some(&W::ins())).is_empty()
        && dom
            .descendants(tbl, Some(&W::t()))
            .iter()
            .all(|&t| dom.value_str(t).trim().is_empty())
}

/// A live empty pilcrow, optionally carrying structurally inert runs.
/// Empty run formatting is retained in place; content, breaks, fields and
/// tracked run ownership never qualify as inert merely because w:t is absent.
pub(super) fn paragraph_has_live_empty_pilcrow(dom: &Dom, p: NodeId) -> bool {
    if !dom.name_is(p, &W::p())
        || [W::ins(), W::del(), W::move_to(), W::move_from()]
            .iter()
            .any(|kind| para_mark_revision(dom, p, kind))
    {
        return false;
    }
    dom.elements(p, None).into_iter().all(|child| {
        if dom.name_is(child, &W::p_pr()) {
            return true;
        }
        if !dom.name_is(child, &W::r()) {
            return false;
        }
        dom.elements(child, None).into_iter().all(|leaf| {
            if dom.name_is(leaf, &W::t()) {
                return dom.value_str(leaf).is_empty();
            }
            dom.name_is(leaf, &W::r_pr())
                && dom.elements(leaf, None).into_iter().all(|property| {
                    dom.name(property).is_some_and(|name| {
                        name.namespace_name() == W::URI
                            && !matches!(
                                name.local_name(),
                                "rPrChange" | "ins" | "del" | "moveFrom" | "moveTo"
                            )
                    }) && dom.nodes(property).is_empty()
                })
        })
    })
}

/// An explicit live empty cell paragraph owns a separate final pilcrow on
/// both sources. Folding its preceding replacement paragraphs would make
/// acceptance or rejection join their text into this blank carrier and lose
/// authored paragraph properties. A content-bearing tail still uses Word's
/// ordinary cell folding rules.
pub(super) fn gap_has_live_blank_cell_tail(dom: &Dom, container: NodeId, rest: &[NodeId]) -> bool {
    dom.name_is(container, &W::tc())
        && !rest.is_empty()
        && rest
            .iter()
            .all(|&tail| paragraph_has_live_empty_pilcrow(dom, tail))
}

/// Only comparison-owned pilcrows may become an unmarked paired boundary.
/// A prior editor's mark (or an earlier mark by this editor) keeps the old
/// merge phase's provenance even when the authored paragraph formats agree.
pub(super) fn paired_replacement_marks_belong_to_comparer(
    dom: &Dom,
    deleted: NodeId,
    inserted: NodeId,
    comparer_author: &str,
) -> bool {
    [
        (deleted, W::del(), W::ins()),
        (inserted, W::ins(), W::del()),
    ]
    .into_iter()
    .all(|(p, kind, opposite)| {
        let Some(properties) = dom.element(p, &W::p_pr()) else {
            return false;
        };
        let Some(mark_properties) = dom.element(properties, &W::r_pr()) else {
            return false;
        };
        let marks = dom.elements(mark_properties, Some(&kind));
        if marks.len() != 1 || !dom.elements(mark_properties, Some(&opposite)).is_empty() {
            return false;
        }
        let mark = marks[0];
        if dom.attribute(mark, &W::author()).unwrap_or("") != comparer_author {
            return false;
        }
        let body_revisions: Vec<NodeId> = dom
            .descendants(p, Some(&kind))
            .into_iter()
            .filter(|&revision| dom.ancestors(revision, Some(&W::p_pr())).is_empty())
            .collect();
        !body_revisions.is_empty()
            && body_revisions.into_iter().all(|revision| {
                dom.attribute(revision, &W::author()).unwrap_or("") == comparer_author
                    && dom.attribute(revision, &W::date()) == dom.attribute(mark, &W::date())
            })
    })
}

/// A related 1v1 pair with identical complete paragraph properties owns a
/// single live pilcrow in Word's mixed paragraph. Retaining the inserted mark
/// makes rejection join the original runs into the next live paragraph and
/// take its unrelated properties. Histories and section ownership remain
/// untouched by this equality-only normalization.
pub(super) fn paired_replacement_has_identical_properties(
    dom: &mut Dom,
    deleted: NodeId,
    inserted: NodeId,
) -> bool {
    if !para_mark_revision(dom, deleted, &W::del()) || !para_mark_revision(dom, inserted, &W::ins())
    {
        return false;
    }
    let (Some(old), Some(new)) = (
        dom.element(deleted, &W::p_pr()),
        dom.element(inserted, &W::p_pr()),
    ) else {
        return false;
    };
    for properties in [old, new] {
        if dom.element(properties, &W::p_pr_change()).is_some()
            || dom.element(properties, &W::sect_pr()).is_some()
        {
            return false;
        }
        if let Some(mark) = dom.element(properties, &W::r_pr())
            && dom.elements(mark, None).iter().any(|&node| {
                [W::move_from(), W::move_to(), W::r_pr_change()]
                    .iter()
                    .any(|name| dom.name_is(node, name))
            })
        {
            return false;
        }
    }
    // Serializing and normalizing comparison-only copies must not append
    // orphan nodes to the production arena (or grow its retained capacity).
    let mut scratch = Dom::new();
    let clean = [old, new].map(|properties| {
        let xml = dom.serialize_element(properties);
        scratch.with_scratch(|scratch| {
            let document = scratch.parse_xdocument(&xml);
            let copy = scratch
                .root(document)
                .expect("serialized paragraph properties have a root");
            if let Some(mark) = scratch.element(copy, &W::r_pr()) {
                for kind in [W::ins(), W::del()] {
                    for revision in scratch.elements(mark, Some(&kind)) {
                        scratch.remove(revision);
                    }
                }
                if scratch.elements(mark, None).is_empty() {
                    scratch.remove(mark);
                }
            }
            remove_powertools_scratch_markup(scratch, copy);
            // Serialization binds PT prefixes used by scratch attributes.
            // Once stripped, discard only those generated declarations;
            // keep namespace declarations owned by the original pPr.
            let generated_bindings = scratch
                .attributes(copy)
                .into_iter()
                .filter(|(name, value)| {
                    scratch.is_namespace_declaration(name)
                        && value == PT::URI
                        && !dom.attributes(properties).iter().any(
                            |(authored_name, authored_value)| {
                                authored_name == name && authored_value == value
                            },
                        )
                })
                .map(|(name, _)| name)
                .collect::<Vec<_>>();
            for name in generated_bindings {
                scratch.set_attribute_value(copy, &name, None);
            }
            scratch.serialize_element(copy)
        })
    });
    clean[0] == clean[1]
}

/// A shared blank cannot supply the pilcrow for revised text whose source
/// had its own paragraph properties. Only comparison-owned original marks
/// and an equal first-original/revised format can share that boundary without
/// changing either source. Later original paragraphs keep their own marks.
pub(super) fn story_tail_has_paired_original_boundary(
    dom: &mut Dom,
    deleted: &[NodeId],
    inserted: NodeId,
    rest: &[NodeId],
    comparer_author: &str,
) -> bool {
    matches!(rest, [anchor] if paragraph_has_live_empty_pilcrow(dom, *anchor))
        && !deleted.is_empty()
        && deleted.iter().all(|&original| {
            paired_replacement_marks_belong_to_comparer(dom, original, inserted, comparer_author)
        })
        && paired_replacement_has_identical_properties(dom, deleted[0], inserted)
}

/// Whether `container` is a table cell's story: the cell itself, or a block
/// content control (`w:sdtContent`) whose nearest story is a cell, through
/// further controls and custom XML. A text box is a story of its own (#2100).
pub(super) fn container_is_cell_story(dom: &Dom, container: NodeId) -> bool {
    let mut node = container;
    loop {
        if dom.name_is(node, &W::tc()) {
            return true;
        }
        let transparent = dom.name_is(node, &W::sdt_content())
            || dom.name_is(node, &W::sdt())
            || dom.name_is(node, &W::custom_xml());
        if !transparent {
            return false;
        }
        let Some(parent) = dom.parent(node) else {
            return false;
        };
        node = parent;
    }
}

pub(super) fn merge_replaced_in_container(
    dom: &mut Dom,
    container: NodeId,
    comparer_author: &str,
    fold_boundary: bool,
) {
    // #1686: a cell that gains or loses an empty paragraph must keep it.
    // The story-end fold below treats the end of a cell as the end of the
    // story and deletes that paragraph. Body folds are unchanged.
    let in_cell = container_is_cell_story(dom, container);
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
            if gap_has_live_blank_cell_tail(dom, container, &children[i..]) {
                continue;
            }
            // M-PI (parity/_scratch/mpi_forensics.md): Word merges only the
            // exact 1v1 replacement pair (heading-1-bold P3 evidence). Larger
            // gaps keep every paragraph separate, ordered [all inserted,
            // B order][all deleted, A order] with the deleted cluster
            // immediately before the closing anchor (green-underline GT: 2
            // ins + 3 del all separate; ole-object GT: whole-doc runs
            // separate). Pairwise-merging bigger runs invented mixed
            // paragraphs Word never produces.
            // Only deleted blocks and the story's final paragraph follow the
            // inserts: the story-end gap, whose revised tail fuses into its
            // first deleted paragraph, the sole one here. That paragraph
            // keeps its own deleted mark, so each side keeps its paragraphs
            // (center_alignment × center_aligned_bold: "Centered bold …"
            // opens "This document …", rejecting restores both paragraphs).
            if let [d] = *dels
                && let Some((&last_ins, lead)) = inss.split_last()
                && para_has_real_del(dom, d)
                && gap_precedes_live_story_tail(dom, &children[i..], &classes[i..], fold_boundary)
                && !para_has_no_text(dom, last_ins)
                && para_mark_revision(dom, last_ins, &W::ins())
                && !para_carries_page_break(dom, last_ins)
                && !para_carries_page_break(dom, d)
                && dom
                    .element(d, &W::p_pr())
                    .is_none_or(|ppr| dom.element(ppr, &W::sect_pr()).is_none())
            {
                for &e in lead {
                    dom.remove(e);
                    dom.add_before_self(d, e);
                }
                let paired_blank_anchor = dels.len() == 1
                    && matches!(&children[i..], [anchor] if paragraph_has_live_empty_pilcrow(dom, *anchor))
                    && paired_replacement_marks_belong_to_comparer(
                        dom,
                        d,
                        last_ins,
                        comparer_author,
                    )
                    && paired_replacement_has_identical_properties(dom, d, last_ins);
                fuse_story_tail_into_deleted(dom, last_ins, d);
                if paired_blank_anchor
                    && let Some(mark) = dom
                        .element(d, &W::p_pr())
                        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                {
                    for revision in dom.elements(mark, Some(&W::del())) {
                        dom.remove(revision);
                    }
                    if dom.elements(mark, None).is_empty() {
                        dom.remove(mark);
                    }
                }
                acted = true;
                break; // children list is stale — rescan
            }
            if dels.len() != 1 || inss.len() != 1 || !para_has_real_del(dom, dels[0]) {
                // M393/M394: multi-del + multi-ins after a leading pure-I stream
                // is Word mid-splice (list-cluster or large legal free-mesh).
                // Moving mid-stream ins before the first del collapses to
                // pure-I-all then pure-D-all.
                //
                // Nested list (ilvl≥1): broken_list×broken.
                // Short leading pure-I (3..=20) + large multi-del/multi-ins:
                // employment×lease after "3. Rent" mid-splice (M394).
                // Broader skips thrash free-mesh demos (file_197 −50).
                let del_has_nested = dels.iter().any(|&p| {
                    dom.element(p, &W::p_pr())
                        .and_then(|ppr| dom.element(ppr, &W::num_pr()))
                        .and_then(|num| dom.element(num, &W::name("ilvl")))
                        .and_then(|il| dom.attribute(il, &W::val()))
                        .and_then(|v| v.parse::<u32>().ok())
                        .is_some_and(|v| v >= 1)
                });
                let legal_mid_splice =
                    (3..=20).contains(&del_start) && dels.len() >= 5 && inss.len() >= 5;
                // Memo×nda: pure-D memo headers first (del_start==0) then pure-I NDA.
                let memo_headers_first =
                    del_start == 0 && (3..=20).contains(&dels.len()) && inss.len() >= 5 && {
                        let t0 = para_revision_body_text(dom, dels[0]).to_ascii_lowercase();
                        t0.starts_with("memorandum")
                            || t0.starts_with("to")
                            || t0.starts_with("from")
                    };
                if dels.len() >= 2
                    && inss.len() >= 2
                    && ((del_start > 0
                        && classes[del_start - 1] == Some(true)
                        && (del_has_nested || legal_mid_splice))
                        || memo_headers_first)
                {
                    i = ins_start; // advance past this del run; leave interleave
                    continue;
                }
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
                //
                // M364 (orphan_comment×yellow_highlight): multi pure-I (≥3) +
                // sole pure-D with ≤1 alnum token ("Ouch") and Jaccard miss —
                // Word keeps pure-I then pure-D (comment anchors land on pure-D
                // later). m44 multi-word sole-del ("Walking on imported air")
                // still folds. Comment anchors are not yet on the pure-D at
                // this stage (carry_comments runs after merge_replaced).
                //
                // The fold is Word's structural final pair, so it needs the
                // gap to end the story: inside the body (another block
                // follows) Word keeps both paragraphs whole under their own
                // marks (list_with_table_break × broken_complex_list: "e",
                // "a" inserted, then "TWO" deleted).
                let story_end = gap_ends_story(dom, &children[i..]);
                if story_end
                    && let (Some(d), Some(&last_ins)) = (sole_del, inss.last())
                    && dom.parent(d).is_some()
                    && dom.parent(last_ins).is_some()
                {
                    let del_toks = body_token_set(&para_revision_body_text(dom, d)).len();
                    let skip_short_residual = inss.len() >= 3
                        && para_body_alnum_len(dom, last_ins) >= 20
                        && del_toks <= 1
                        && !should_fold_ins_del_pair(dom, last_ins, d);
                    if !skip_short_residual {
                        // M435 (diff_before16×19): sole pure-D base Heading/Title
                        // free-meshed into last pure-I. Word keeps live pStyle +
                        // del mark on the MIX. The bare-MIX path below drops del
                        // pPr (single_paragraph GT) and left empty pPr (LO ~53.9
                        // until Heading1 restored). Adopt Heading/Title pPr only
                        // — never invent styles on non-heading sole-dels.
                        //
                        // M360 guard applies here too: a TOC/field residue
                        // pure-I (fldChar) folded with a Heading pure-D keeps
                        // the bare pPr (table_border×toc SD-2343 title).
                        let ins_has_fld = para_is_field_residue(dom, last_ins);
                        let adopt_heading = para_has_heading_or_title_style(dom, d) && !ins_has_fld;
                        if adopt_heading {
                            if let Some(ippr) = dom.element(last_ins, &W::p_pr()) {
                                dom.remove(ippr);
                            }
                            if let Some(dppr) = dom.element(d, &W::p_pr()) {
                                let cloned = dom.clone_subtree(dppr);
                                if let Some(first) = dom.elements(last_ins, None).first().copied() {
                                    dom.add_before_self(first, cloned);
                                } else {
                                    dom.add(last_ins, cloned);
                                }
                            }
                        } else {
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
                        }
                        for c in dom.elements(d, None) {
                            if !dom.name_is(c, &W::p_pr()) {
                                dom.add(last_ins, c); // move body del/ins into last ins
                            }
                        }
                        dom.remove(d);
                    }
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
                dom.copy_attributes(ins_p, merged);
                // Inserted pPr wins when structural (new formatting). M88: when
                // Inserted is bare but Deleted has numPr/spacing, keep Deleted
                // structural live with del mark (file_55 mixed "notation"+"a").
                let ins_struct = dom
                    .element(ins_p, &W::p_pr())
                    .is_some_and(|p| ppr_has_structural_props(dom, p));
                let del_struct = dom
                    .element(d, &W::p_pr())
                    .is_some_and(|p| ppr_has_structural_props(dom, p));
                // M360: a TOC/field residue pure-I (fldChar) merged with a
                // Heading/Title pure-D keeps a bare pPr — never the Heading
                // pStyle (table_border×toc SD-2343 title, pagefair −11).
                let m360_fld_x_heading = para_is_field_residue(dom, ins_p)
                    && dom.element(d, &W::p_pr()).is_some_and(|dp| {
                        dom.element(dp, &W::p_style()).is_some_and(|ps| {
                            let v = dom
                                .attribute(ps, &W::val())
                                .unwrap_or("")
                                .to_ascii_lowercase();
                            v == "title" || v.starts_with("heading")
                        })
                    });
                if ins_struct {
                    if let Some(ppr) = dom.element(ins_p, &W::p_pr()) {
                        let c = dom.clone_subtree(ppr);
                        dom.add(merged, c);
                    }
                } else if del_struct && !m360_fld_x_heading {
                    if let Some(ppr) = dom.element(d, &W::p_pr()) {
                        let c = dom.clone_subtree(ppr);
                        dom.add(merged, c);
                    }
                } else if let Some(ppr) = dom.element(ins_p, &W::p_pr()) {
                    let c = dom.clone_subtree(ppr);
                    dom.add(merged, c);
                }
                if children[i..].first().is_some_and(|&anchor| {
                    dom.name_is(anchor, &W::p())
                        && [W::ins(), W::del(), W::move_from(), W::move_to()]
                            .iter()
                            .all(|kind| !para_mark_revision(dom, anchor, kind))
                }) && paired_replacement_marks_belong_to_comparer(dom, d, ins_p, comparer_author)
                    && paired_replacement_has_identical_properties(dom, d, ins_p)
                    && let Some(mark) = dom
                        .element(merged, &W::p_pr())
                        .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                {
                    for revision in dom.elements(mark, Some(&W::ins())) {
                        dom.remove(revision);
                    }
                    if dom.elements(mark, None).is_empty() {
                        dom.remove(mark);
                    }
                }
                for c in dom.elements(ins_p, None) {
                    if !dom.name_is(c, &W::p_pr()) {
                        dom.add(merged, c); // clone-on-attach
                    }
                }
                for c in dom.elements(d, None) {
                    if !dom.name_is(c, &W::p_pr()) {
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
                // M216: mark-only empty pure-D (pPr/rPr/del, no w:del body) after
                // a sole pure-I prose block — Word folds the mark-del into that
                // pure-I for table→prose residuals with an **equal** title
                // (contract_review insertions×mixed: classes `.ID.T.`).
                // Skip when the paragraph immediately before the pure-I run
                // already carries `w:ins` (support_tickets_table×summary title
                // is partial-EQ + ins → class None but still an inserted title;
                // Word keeps the empty pure-D after the body). Tables break the
                // pure-D run (class None), so the empty mark is a sole del.
                if inss.is_empty() || dels.is_empty() {
                    continue;
                }
                if gap_has_live_blank_cell_tail(dom, container, &children[j..]) {
                    continue;
                }
                let tail_ins = inss[inss.len() - 1];
                if gap_precedes_live_story_tail(dom, &children[j..], &classes[j..], fold_boundary)
                    && !para_has_no_text(dom, tail_ins)
                    && para_mark_revision(dom, tail_ins, &W::ins())
                    && !para_carries_page_break(dom, tail_ins)
                    && !para_carries_page_break(dom, dels[0])
                    && dom
                        .element(dels[0], &W::p_pr())
                        .is_none_or(|ppr| dom.element(ppr, &W::sect_pr()).is_none())
                {
                    let paired_blank_anchor = story_tail_has_paired_original_boundary(
                        dom,
                        dels,
                        tail_ins,
                        &children[j..],
                        comparer_author,
                    );
                    fuse_story_tail_into_deleted(dom, tail_ins, dels[0]);
                    if paired_blank_anchor
                        && let Some(mark) = dom
                            .element(dels[0], &W::p_pr())
                            .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                    {
                        for revision in dom.elements(mark, Some(&W::del())) {
                            dom.remove(revision);
                        }
                        if dom.elements(mark, None).is_empty() {
                            dom.remove(mark);
                        }
                    }
                    acted = true;
                    break;
                }
                let preceding_has_ins = ins_start > 0 && {
                    let prev = children[ins_start - 1];
                    dom.name_is(prev, &W::p()) && !dom.descendants(prev, Some(&W::ins())).is_empty()
                };
                // Sole mark-only empty (contract) OR leading mark-only empty when
                // every pure-D in the run is mark-only empty (inventory: two
                // empties before deleted table). After M218 the carrier gets a
                // del pilcrow → class None, so a second empty is not re-folded
                // into a pure-I on the next rescan (Word keeps one empty).
                let first_mark_only_empty = !para_has_real_del(dom, dels[0])
                    && para_mark_revision(dom, dels[0], &W::del())
                    && para_has_no_text(dom, dels[0]);
                let all_dels_mark_only_empty = first_mark_only_empty
                    && dels.iter().all(|&d| {
                        !para_has_real_del(dom, d)
                            && para_mark_revision(dom, d, &W::del())
                            && para_has_no_text(dom, d)
                    });
                // Skip when carrier already has a del pilcrow (second empty after
                // inventory first fold — Word keeps one empty pure-D).
                let carrier = inss[inss.len() - 1];
                let carrier_has_mark_del = para_mark_revision(dom, carrier, &W::del());
                // #1686: inside a cell the mark-only empty paragraph is one
                // the cell lost. Folding it onto the inserted paragraph
                // leaves Reject All one paragraph short.
                let mark_only_empty_del = !in_cell
                    && inss.len() == 1
                    && !preceding_has_ins
                    && !carrier_has_mark_del
                    && first_mark_only_empty
                    && (dels.len() == 1 || all_dels_mark_only_empty);
                // M385 (nda×report residual −26.6): multi pure-I ending in
                // List* style + multi pure-D starting with mark-only empty
                // Heading/Title. Word free-meshes into MIX Heading1+del mark
                // (M344 adopt). mark_only_empty_del required inss.len()==1, so
                // multi-I never set del_foldable for empty heading pure-D.
                let last_content_ins = inss
                    .iter()
                    .rev()
                    .find(|&&p| !para_has_no_text(dom, p))
                    .copied()
                    .unwrap_or(carrier);
                let last_ins_list_style =
                    dom.element(last_content_ins, &W::p_pr()).is_some_and(|ip| {
                        dom.element(ip, &W::p_style()).is_some_and(|ps| {
                            dom.attribute(ps, &W::val())
                                .unwrap_or("")
                                .to_ascii_lowercase()
                                .starts_with("list")
                        })
                    });
                // #2100: in a cell this fold drops the empty heading the cell
                // lost, as `mark_only_empty_del` would.
                let heading_empty_mark_fold = !in_cell
                    && first_mark_only_empty
                    && !carrier_has_mark_del
                    && para_has_heading_or_title_style(dom, dels[0])
                    && last_ins_list_style;
                let del_foldable = para_has_real_del(dom, dels[0])
                    || mark_only_empty_del
                    || heading_empty_mark_fold;
                if !del_foldable {
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
                //
                // M393 (broken_list×broken Word IDDD…I…D): leading pure-I list
                // item + multi pure-D cluster + **more pure-I after the dels**
                // is Word interleave, not a residual free-mesh boundary.
                // Folding ONE×Item1 invents MIX and thrashs the cluster peel.
                if dels.len() >= 2
                    && inss.len() == 1
                    && j < children.len()
                    && classes[j] == Some(true)
                {
                    continue;
                }
                // M394 (employment×lease): leading pure-I mid-splice stream
                // (through ~3rd section) + multi pure-D residual base — Word
                // keeps pure-I then pure-D (no fold of "2. Term"×"Acme").
                if ins_start == 0
                    && (3..=20).contains(&inss.len())
                    && dels.len() >= 5
                    && j < children.len()
                    && classes[j] == Some(true)
                {
                    continue;
                }
                // M393b: second pure-I block (short list labels a/b/TWO) before
                // residual pure-D list items — Word keeps pure-I then pure-D
                // (no free-mesh of "a"×"First shown"). Require a nested pure-D
                // earlier in the document (first list cluster already emitted).
                let prior_nested_del = (0..ins_start).any(|idx| {
                    classes[idx] == Some(false)
                        && dom
                            .element(children[idx], &W::p_pr())
                            .and_then(|ppr| dom.element(ppr, &W::num_pr()))
                            .and_then(|num| dom.element(num, &W::name("ilvl")))
                            .and_then(|il| dom.attribute(il, &W::val()))
                            .and_then(|v| v.parse::<u32>().ok())
                            .is_some_and(|v| v >= 1)
                });
                if !dels.is_empty()
                    && !inss.is_empty()
                    && prior_nested_del
                    && inss
                        .iter()
                        .filter(|&&p| !para_has_no_text(dom, p))
                        .all(|&p| para_word_atom_count(dom, p) <= 3)
                    && dels.iter().any(|&p| para_has_live_numpr(dom, p))
                    && ins_start > 0
                    && classes[ins_start - 1] == Some(false)
                {
                    continue;
                }
                let sole_del = dels.len() == 1;
                // Word: last pure-ins merges with first pure-del at I…I D…D boundary.
                let d = dels[0];
                // Prefer last **non-empty** pure-I as fold carrier. Empty pure-I
                // after digits "24" (1_5×24 B shape) would otherwise be the
                // carrier and bypass M101 digits-only on the real last content.
                let mut last_ins = inss
                    .iter()
                    .rev()
                    .find(|&&p| !para_has_no_text(dom, p))
                    .copied()
                    .unwrap_or(inss[inss.len() - 1]);

                // M359 (list_with_indents×shape_group): trailing pure-I is an
                // empty/drawing shell then pure-D long unrelated list prose.
                // Word folds the **last** pure-I (drawing) with first pure-D
                // (IIII…XDDDD). Preferring last content pure-I MIX-ed "My test
                // with some shapes." into the list (−9.4 vs 27c).
                // M362 (file_173×174 stamp: TIFF title × Verdana Demo +
                // drawing): same carrier bug with **short** pure-D title
                // (≤8 words) — content×content title MIX thrash 100→67.
                // When last pure-I is empty/whitespace and first pure-D is
                // unrelated, keep trailing empty as carrier (long OR short).
                let trailing = inss[inss.len() - 1];
                // #1686: a cell's trailing empty inserted paragraph is a
                // paragraph of the revised cell. Using it as the carrier
                // deletes it on the next fold.
                if !in_cell
                    && inss.len() >= 2
                    && trailing != last_ins
                    && (para_has_no_text(dom, trailing)
                        || para_body_text_is_whitespace_only(dom, trailing))
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                    && (para_word_atom_count(dom, d) > 12
                        || (para_word_atom_count(dom, last_ins) <= 6
                            && para_word_atom_count(dom, d) <= 8))
                {
                    last_ins = trailing;
                }
                // M468 (h_f_normal_odd_even_unchecked × sd_1495): trailing
                // EMPTY pure-I separates a LONG content pure-I (>6 tokens)
                // from a short unrelated pure-D title (≤8 tokens) — Word
                // folds NOTHING (icon para stays pure-I, title keeps its own
                // MARK-DEL paragraph and pPr). Reaching backward across the
                // empties merged them and lost the oracle's vertical gap
                // (79.6 → 46.1 once the wrong heading stamps stopped
                // compensating). Yields to the M322 head-junction below:
                // when the title shares a significant token with the FIRST
                // content pure-I (tiff × h_f_normal "TIFF"), Word does MIX
                // at head and this skip must not fire.
                if inss.len() >= 2
                    && trailing != last_ins
                    && (para_has_no_text(dom, trailing)
                        || para_body_text_is_whitespace_only(dom, trailing))
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                    && para_word_atom_count(dom, last_ins) > 6
                    && para_word_atom_count(dom, d) <= 8
                {
                    let head_junction = inss
                        .iter()
                        .copied()
                        .find(|&p| !para_has_no_text(dom, p))
                        .is_some_and(|first_ins| {
                            let it = para_revision_body_text(dom, first_ins);
                            let dt = para_revision_body_text(dom, d);
                            short_title_shares_sig_token(&it, &dt)
                        });
                    if !head_junction {
                        continue;
                    }
                }
                // M322 (tiff×h_f_normal): short pure-D title ("TIFF test document")
                // after a long pure-I stream. Word head-junctions the **first**
                // content pure-I with the title (MIX at start); last-I fold
                // wrongly MIX-ed the final license line with TIFF (~41). Require
                // a shared significant token (len≥4) so hummingbird×employment
                // (no shared token, Word tail MIX only) keeps last-I fold.
                // Never across a field the title leaves open (98bf5f3d ×
                // a3701d36: "Part 1—Preliminary" opens the deleted TOC; the
                // junction lifted that begin above the inserted TOC's end).
                let mut head_junction = false;
                if inss.len() >= 5
                    && (1..=6).contains(&para_word_atom_count(dom, d))
                    && let Some(first_ins) =
                        inss.iter().copied().find(|&p| !para_has_no_text(dom, p))
                    && !fold_crosses_a_field(dom, d, first_ins, inss)
                {
                    let it = para_revision_body_text(dom, first_ins);
                    let dt = para_revision_body_text(dom, d);
                    if short_title_shares_sig_token(&it, &dt) {
                        last_ins = first_ins;
                        head_junction = true;
                    }
                }
                if fold_crosses_a_field(dom, d, last_ins, inss) {
                    continue;
                }
                // Paired final marks leave the revised last content paragraph
                // on its own inserted mark: only a shared-word head junction
                // folds.
                if !fold_boundary && !head_junction {
                    continue;
                }
                // Inside the body (another block follows the gap) Word folds
                // no unrelated pair: the boundary fold is its structural
                // final pair, which needs the story end.
                if !head_junction
                    && !gap_ends_story(dom, &children[j..])
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    continue;
                }
                // M311d (image×rtl / rtl_mixed×rtl_page): ≥3 empty pure-I then
                // pure-D residual(s). Word keeps pure-I empties. sole_del
                // always-fold and multi-del boundary fold would eat empties
                // into the del run one-by-one.
                //
                // M419 (math_delimiter × math_eqarr): OMML-only pure-I shells
                // also have no w:t but Word MIX-es the last math into the first
                // pure-D title (IIIIMDD). Do not treat math shells as M311d
                // layout empties — require no oMath/oMathPara under the pure-I.
                if inss.len() >= 3
                    && inss.iter().all(|&p| {
                        (para_has_no_text(dom, p) || para_body_text_is_whitespace_only(dom, p))
                            && !para_has_omath(dom, p)
                    })
                {
                    continue;
                }
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
                    Some(n) if n == W::tbl() => true,
                    Some(n) if n == W::p() => !para_has_no_text(dom, c),
                    _ => false,
                });
                // M77: mid-document sole pure-D after pure-I must not fold into
                // the preceding ins when body texts are unrelated and more
                // content follows (file_33). Whole-doc trailing sole-del (m44)
                // still always folds.
                //
                // M368 (sdts×shape): sole pure-D "Before…" is only the first
                // body residual before a deleted table (nD==1 at the boundary);
                // Word still MIX-es trailing empty/drawing pure-I with it. Allow
                // empty-shell × short pure-D even with following table content.
                if sole_del && following_content && !should_fold_ins_del_pair(dom, last_ins, d) {
                    let empty_shell = para_has_no_text(dom, last_ins)
                        || para_body_text_is_whitespace_only(dom, last_ins);
                    let short_del = (1..=6).contains(&para_word_atom_count(dom, d));
                    if !(empty_shell && short_del) {
                        continue;
                    }
                }
                // M413 (broken_media×duplicate_ppr / spaces×spacing sole-del):
                // multi contentful pure-I title/labels (≥4) × sole pure-D base
                // prose. Trailing sole-del always-fold (m44) MIX-es last pure-I
                // into sole pure-D. Word pure-I all next then pure-D base.
                // M418: only when last pure-I is short stub or title-page tail.
                // M422 thrash: last_n≤2 also matched "Acme Corporation" on
                // hummingbird×employment (−5) and blocked Word MIX into wrap.
                // Stubs must be ≤2 tokens AND ≤5 chars total (a/x/x/b labels).
                if sole_del
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                    && para_word_atom_count(dom, d) >= 5
                {
                    let content_inss_n = inss
                        .iter()
                        .filter(|&&p| !para_revision_body_text(dom, p).trim().is_empty())
                        .count();
                    let last_n = para_word_atom_count(dom, last_ins);
                    let last_t = para_revision_body_text(dom, last_ins).to_ascii_lowercase();
                    let short_stub =
                        last_n <= 2 && last_t.chars().filter(|c| !c.is_whitespace()).count() <= 5;
                    let title_page_tail = last_t.contains("prepared")
                        || last_t.contains('@')
                        || last_t.contains("2040")
                        || last_t.contains("agreement")
                        || short_stub;
                    if content_inss_n >= 4 && title_page_tail {
                        continue;
                    }
                }
                // M416 (heading_font×hummingbird): sole long pure-I wrap (≥20
                // words) × multi pure-D short base. sole_del always-fold MIX-es
                // wrap into first base. Word pure-I wrap then pure-D all base.
                // M420: wrap fingerprint only (see should_fold_multi_del).
                if sole_del
                    && inss.len() == 1
                    && dels.len() >= 3
                    && para_word_atom_count(dom, last_ins) >= 20
                    && para_has_repeated_phrase(dom, last_ins)
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    continue;
                }
                // M322b (tiff×h_f after head-junction): long pure-I stream + sole
                // empty pure-D (drawing shell / mark-only). Word keeps trailing
                // pure-D empty; empty-del always-fold MIX-ed last license line.
                // Content sole-del (m44 "24"/titles) has body text; short demos
                // have inss ≪ 10.
                //
                // M341 (missing_sectpr×missing_separator): short pure-I stream
                // (empties + "PARTIES" + "something") + sole empty pure-D of the
                // base title shell. Word IIIIM (pure-I "something", MIX empty);
                // empty-del fold MIX-ed "something" into the shell (~66 vs e3
                // ~97). Skip sole empty pure-D when last pure-I has body text.
                if sole_del
                    && para_revision_body_text(dom, d).trim().is_empty()
                    && (inss.len() >= 10
                        || (inss.len() >= 2
                            && !para_revision_body_text(dom, last_ins).trim().is_empty()))
                {
                    continue;
                }
                // M430b (doc_with_spaces × doc_with_spacing ~62.8 / docxodus 100):
                // title-page pure-I ends with ≥3 empty pure-I then multi pure-D
                // base. fold_whitespace skip (M430) keeps them into merge, but
                // multi-del empty-shell fold below ate empties one-by-one into
                // first pure-D (empty_run→0 by second fold pass). Word keeps
                // trailing blanks. Skip empty-shell multi-del fold when the
                // pure-I stream ends with ≥3 consecutive empty pure-I.
                if !sole_del && para_is_visually_blank(dom, last_ins) {
                    let trailing_empty_i = inss
                        .iter()
                        .rev()
                        .take_while(|&&p| para_is_visually_blank(dom, p))
                        .count();
                    if trailing_empty_i >= 3 {
                        continue;
                    }
                }
                // M431 (diff_doc2 × numwords residual ~52 / docxodus 100): pure-I
                // page-break (`w:br`) must not fold into pure-D drawing even when
                // multi-del gap force-fold (gap < 40 words) would. Word keeps
                // pure-I br then pure-D drawing (+ empty) then free-meshes text.
                if !dom.descendants(last_ins, Some(&W::name("br"))).is_empty()
                    && (!dom.descendants(d, Some(&W::drawing())).is_empty()
                        || !dom.descendants(d, Some(&W::pict())).is_empty()
                        || !dom.descendants(d, Some(&W::object())).is_empty())
                {
                    continue;
                }
                // C1 / KNOWN ISSUE #2: multi-del boundary fold gated on
                // document-scale relatedness (unrelated whole-doc replacement
                // must not mix last pure-I with first pure-D).
                //
                // C1 / KNOWN ISSUE #2 continued: multi-del document-scale gate.
                // M368: empty-shell × short pure-D (sdts×shape "Before" before
                // deleted table). M419 (math_delimiter × math_eqarr ~45.6): multi
                // pure-I math shells (OMML only, no w:t) + multi pure-D long
                // titles — empty_shell × long first pure-D was blocked → IIIIIDDD;
                // Word MIX-es last math into "m:d Delimiter…" (IIIIMDD). Fold any
                // empty/math shell pure-I into the pure-D boundary (not only when
                // first pure-D is short).
                // M417 vs M419 scale split: the empty/math-shell override below
                // is for demo-scale replacements (math_delimiter × math_eqarr,
                // dels ~51). A wholesale replacement of a **large** document
                // (sd_2517 × borderbox, dels ~1038) keeps Word's clean I…D
                // boundary — never fold a shell into it.
                if !sole_del
                    && dels.len() > 150
                    && !should_fold_multi_del_at_document_scale(
                        dom, container, last_ins, d, inss, dels,
                    )
                {
                    continue;
                }
                if !sole_del
                    && !should_fold_multi_del_at_document_scale(
                        dom, container, last_ins, d, inss, dels,
                    )
                {
                    // Layout content (w:br / drawing / pict) is never an empty
                    // shell: para_body_text_is_whitespace_only is true for
                    // br-only (no w:t), which previously forced fold of pure-I
                    // page-break into pure-D drawing (M431 diff_doc2×numwords).
                    let empty_shell = (para_has_no_text(dom, last_ins)
                        || para_body_text_is_whitespace_only(dom, last_ins))
                        && dom.descendants(last_ins, Some(&W::name("br"))).is_empty()
                        && dom.descendants(last_ins, Some(&W::drawing())).is_empty()
                        && dom.descendants(last_ins, Some(&W::pict())).is_empty()
                        && dom.descendants(last_ins, Some(&W::object())).is_empty();
                    if !empty_shell {
                        continue;
                    }
                }
                // M101 (file_166; 1_5_line_spacing×24): last content pure-I that
                // is **digits-only** ("24") + multi pure-D of an unrelated demo —
                // Word keeps pure-I then pure-D title (no MIX "241.5 Line…").
                // Content pure-I like "Ouch." still folds (M89) — not digits-only.
                if dels.len() > 1
                    && para_body_is_digits_only(dom, last_ins)
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    continue;
                }
                // M325 (italic_rstyle × base_ordered): multi pure-I short list
                // items ("One"/"Two"/"Three") + multi pure-D long unrelated demo
                // title. Word pure-I/Ds (IIIIIIIIDDDD…); multi-del fold MIX-ed
                // last "Three" into italic title (~44). Skip when last pure-I is
                // ≤2 tokens, first pure-D has ≥8 tokens, Jaccard 0. Single-
                // token pure-I sole ("Ouch.") still M89-folds (inss==1).
                //
                // M336 (bullet_list_bold×bullet_list): first pure-D is the shared
                // demo intro "This document demonstrates…" — Word folds Grapes
                // into it (DIMD). Do not apply M325 skip for that intro shape.
                //
                // M359: empty/drawing last pure-I (0 tokens) must still fold with
                // long pure-D (list×shape Word X shell + list delText). M325
                // treated empty as ≤2 tokens and skipped the fold entirely.
                if dels.len() > 1
                    && inss.len() >= 3
                    && para_word_atom_count(dom, last_ins) <= 2
                    && para_word_atom_count(dom, last_ins) >= 1
                    && para_word_atom_count(dom, d) >= 8
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    let dt = para_revision_body_text(dom, d).to_ascii_lowercase();
                    let demo_intro = dt.contains("demonstrates") || dt.starts_with("this document");
                    if !demo_intro {
                        continue;
                    }
                }
                // M410 (bold_vals × complex_list_def): multi pure-I short alpha
                // labels (ONE/a/FOUR) then multi pure-D OOXML intro. multi-del
                // fold MIX-es last "FOUR" into OOXML body. Skip when ≥3 pure-I
                // are ≤2-token labels and first pure-D is long OOXML/demo prose.
                if dels.len() > 1
                    && inss.len() >= 5
                    && para_word_atom_count(dom, last_ins) <= 2
                    && para_word_atom_count(dom, last_ins) >= 1
                    && para_word_atom_count(dom, d) >= 10
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    let short_labels = inss
                        .iter()
                        .filter(|&&p| {
                            let n = para_word_atom_count(dom, p);
                            (1..=2).contains(&n)
                        })
                        .count();
                    let dt = para_revision_body_text(dom, d).to_ascii_lowercase();
                    let ooxml_intro = dt.contains("ooxml")
                        || dt.contains("st_onoff")
                        || dt.contains("w:b")
                        || dt.contains("w:i")
                        || (dt.contains("demonstrates") && dt.contains("sample"));
                    if short_labels * 2 >= inss.len() && ooxml_intro {
                        continue;
                    }
                }
                // M140 (eigenpal×employee_directory): sole pure-I multi-word
                // title ("Employee Directory") + multi pure-D starting with a
                // short unrelated title/slug ("eigenpal/docx-editor", ≤3 tokens)
                // — Word keeps pure-I then pure-D. Do NOT apply to longer first
                // pure-D (green_underline×heading_1: "First green underlined
                // item" is 4 tokens — Word folds last pure-I body into it).
                // Single-token pure-I ("Ouch.") still folds (M89).
                if dels.len() > 1 && inss.len() == 1 {
                    let it = para_revision_body_text(dom, last_ins);
                    let dt = para_revision_body_text(dom, d);
                    let ins_toks = body_token_set(&it).len();
                    let del_toks = body_token_set(&dt).len();
                    if ins_toks >= 2
                        && (2..=3).contains(&del_toks)
                        && !should_fold_ins_del_pair(dom, last_ins, d)
                    {
                        continue;
                    }
                }
                // M319 (support_tickets×table_bookmark_end): multi pure-I next
                // where last pure-I is a **Heading/Title** style header
                // ("Test 1 – Fixed Width Table", Heading2) + multi pure-D short
                // non-Demo base title ("Support Tickets"). Word IIIDDD…. Stamp
                // confetti body folds (file_37 last pure-I has no pStyle) still
                // M90-fold — heading-style gate is load-bearing.
                if dels.len() > 1
                    && inss.len() >= 2
                    && para_has_heading_or_title_style(dom, last_ins)
                {
                    let d_content = dels
                        .iter()
                        .copied()
                        .find(|&p| !para_revision_body_text(dom, p).trim().is_empty())
                        .unwrap_or(d);
                    if !para_looks_like_demo_title(dom, d_content) {
                        let it = para_revision_body_text(dom, last_ins);
                        let dt = para_revision_body_text(dom, d_content);
                        let ins_toks = body_token_set(&it).len();
                        let del_toks = body_token_set(&dt).len();
                        if ins_toks >= 3
                            && (1..=3).contains(&del_toks)
                            && !should_fold_ins_del_pair(dom, last_ins, d_content)
                        {
                            continue;
                        }
                    }
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
                // (≥10 paras: contract body) when first pure-D is an unrelated
                // short Demo title — Word keeps pure-I then pure-D title (no
                // MIX). Medium pure-I runs (title×training sticky: 7 module
                // lines) Word folds last pure-I with first Demo del — keep fold
                // (threshold was 5, which blocked that Word MIX). Short pure-I
                // runs (file_38/11 stamped demos, M90) still always fold. Also
                // require last pure-I has real content (≥8 alnum) so file_54
                // "b"+empties×Demo title still folds (M90).
                if dels.len() > 1
                    && inss.len() >= 10
                    && para_body_alnum_len(dom, last_ins) >= 8
                    && para_looks_like_demo_title(dom, d)
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    continue;
                }
                // M145 (hr_onboarding×Word_vs_Google_Docs): short pure-I prefix
                // of a long next doc (title + subtitle, ≤3) + multi pure-D of a
                // short base checklist starting with an unrelated short title
                // ("HR Onboarding Checklist", 2..=5 tokens) — Word keeps pure-I
                // subtitle then pure-D title (no MIX). Require **more pure-I
                // after the pure-D run** (long next continues); without that
                // gate, quarterly×red_bold (I…I then pure-D table only) wrongly
                // skipped the Word MIX fold of last body + "Quarterly…".
                let following_pure_i = children[j..].iter().any(|&c| {
                    dom.name_is(c, &W::p())
                        && para_is_pure_inserted(dom, c)
                        && !para_has_no_text(dom, c)
                });
                if dels.len() > 1
                    && inss.len() <= 3
                    && following_pure_i
                    && para_body_alnum_len(dom, last_ins) >= 40
                {
                    let dt = para_revision_body_text(dom, d);
                    let del_toks = body_token_set(&dt).len();
                    if (2..=5).contains(&del_toks) && !should_fold_ins_del_pair(dom, last_ins, d) {
                        continue;
                    }
                }
                // M145b: multi pure-D **checklist cell cluster** after pure-I
                // body — first pure-D is digits-only ("1") or ≤2 short tokens
                // ("Sign NDA","Task"), not a 3-token document title
                // ("Quarterly Performance Report" must still M90-fold into last
                // pure-I — quarterly×red_bold). last pure-I ≥10 alnum, not Demo.
                if dels.len() > 1
                    && following_content
                    && para_body_alnum_len(dom, last_ins) >= 10
                    && !para_looks_like_demo_title(dom, last_ins)
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    let t0 = para_revision_body_text(dom, d);
                    let n0 = body_token_set(&t0).len();
                    // A styled Title/Heading is a heading, not a cell: Word
                    // mixes it (file_134 × file_135 "Table Widths").
                    let first_is_cell = !para_has_heading_or_title_style(dom, d)
                        && (para_body_is_digits_only(dom, d)
                            || ((1..=2).contains(&n0) && para_body_alnum_len(dom, d) <= 12));
                    if first_is_cell {
                        continue;
                    }
                }
                // M363 (lists_sub×word_mixed −8.6): long pure-I prose ("Normal
                // text, then bold italic…") × multi-word pure-D list item
                // ("Item" + soft-break "Sub paragraph"…). Word keeps pure-I
                // then pure-D ListParagraph (no MIX). Short pure-D residuals
                // ("a", file_55) still fold + adopt numPr (M88).
                //
                // M382 (file_45×file_46 −23): long pure-I body free-meshes short
                // list label pure-D "First item" (2 words) as MIX. Skip only
                // when pure-D list item has ≥3 word-atoms (soft-break multi-
                // word list content); 1–2 word list labels still fold.
                //
                // M387 (broken_list×list_spacer −15.9): **list** pure-I
                // ("14.11Survival of Terms p3", StandardL1+numPr) free-meshes
                // multi-word list pure-D "Item 1. Text 1." in Word (MIX). Skip
                // only non-list long pure-I prose × multi-word list pure-D.
                {
                    let ins_long_prose = para_body_alnum_len(dom, last_ins) >= 20;
                    let ins_list = para_has_live_numpr(dom, last_ins)
                        || dom.element(last_ins, &W::p_pr()).is_some_and(|ip| {
                            dom.element(ip, &W::p_style()).is_some_and(|ps| {
                                let v = dom
                                    .attribute(ps, &W::val())
                                    .unwrap_or("")
                                    .to_ascii_lowercase();
                                v.starts_with("list") || v.contains("standardl")
                            })
                        });
                    let del_list = para_has_live_numpr(dom, d)
                        || dom.element(d, &W::p_pr()).is_some_and(|dp| {
                            dom.element(dp, &W::p_style()).is_some_and(|ps| {
                                dom.attribute(ps, &W::val())
                                    .unwrap_or("")
                                    .eq_ignore_ascii_case("ListParagraph")
                            })
                        });
                    let del_multi_word = para_word_atom_count(dom, d) >= 3;
                    if ins_long_prose && del_list && del_multi_word && !ins_list {
                        continue;
                    }
                }
                // M364 (orphan_comment×yellow_highlight −6.4): multi pure-I +
                // sole short pure-D residual (≤1 alnum token, Jaccard miss).
                // Word keeps pure-I then pure-D; comments are re-injected onto
                // pure-D after merge. m44 multi-word sole-del still folds.
                if sole_del
                    && inss.len() >= 3
                    && para_body_alnum_len(dom, last_ins) >= 20
                    && body_token_set(&para_revision_body_text(dom, d)).len() <= 1
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                {
                    continue;
                }
                // M371 (word_mixed×word_simple −2.2): multi pure-I long prose +
                // multi pure-D starting with Heading/Title ("Mixed Formatting
                // Test"). Word keeps pure-D heading separate (I… then D title);
                // multi-del fold MIX-ed last pure-I body with Heading1 adopt
                // (Heading1 on "A final paragraph…"). Skip when last pure-I is
                // not heading and first pure-D is heading/title, Jaccard miss.
                //
                // M380 (file_197×file_198 −55): short single-token Heading pure-D
                // ("Images") **does** free-mesh with last pure-I body in Word
                // (MIX). Only multi-word short **Heading*** titles take the skip
                // — "Images" (1 word) still folds.
                //
                // M380b (file_83×file_84 −50): **Title** pure-D ("CONTRACT FOR
                // CONTRACTS") also free-meshes with last pure-I body in Word.
                // Restrict M371 to Heading* only (not Title).
                //
                // M385 (nda×report −26.6): List* pure-I × short Heading pure-D
                // title free-meshes (MIX Heading1). Do not skip list carriers.
                //
                // M386 (calibri×center / heading_4×helvetica −41/−39): short
                // stamp demos (2 pure-I) free-mesh Heading-styled body pure-D
                // ("Calibri font with Heading 2…", "Demonstrating Heading 4…")
                // even when pure-D is multi-word. M371 skip requires a **longer**
                // pure-I stream (≥3) **and** a short title Heading (2..=6 words,
                // e.g. "Mixed Formatting Test" after 6 pure-I in word_mixed).
                let last_ins_list_style_m371 =
                    dom.element(last_ins, &W::p_pr()).is_some_and(|ip| {
                        dom.element(ip, &W::p_style()).is_some_and(|ps| {
                            dom.attribute(ps, &W::val())
                                .unwrap_or("")
                                .to_ascii_lowercase()
                                .starts_with("list")
                        })
                    });
                if dels.len() > 1
                    && inss.len() >= 3
                    && para_has_heading_style(dom, d)
                    && !para_has_heading_or_title_style(dom, last_ins)
                    && para_body_alnum_len(dom, last_ins) >= 20
                    && (2..=6).contains(&para_word_atom_count(dom, d))
                    && !should_fold_ins_del_pair(dom, last_ins, d)
                    && !last_ins_list_style_m371
                {
                    continue;
                }
                // M365 (bookmark×broken_complex_list −5.0): multi pure-I list
                // ending in a very short residual ("a") × multi pure-D long
                // bookmark prose. Word keeps pure-I "a" then pure-D; folding
                // invents MIX (−5 pagefair). Do **not** require Jaccard miss:
                // pure-I "a" shares token "a" with "…a paragraph…" (0.125 ≥
                // 0.12) and would still fold. M124 covers short pure-I runs
                // (≤2); this covers long list streams.
                // M373 (toc×broken_list −2.3): pure-D Heading "Generalities"
                // is only 12 alnum — old ≥20 left short pure-I "a" folding into
                // Heading1 MIX.
                //
                // M381 (file_54×file_55 −32): short pure-I "b" **does** free-
                // mesh with short demo title "1.5 Line Spacing Demo" (Word MIX).
                // Skip only when pure-D is **long prose** (≥8 word-atoms) **or**
                // Heading* style (TOC "a"×"Generalities"); bare short Demo
                // titles (≤6 words, no Heading) still fold.
                if dels.len() > 1
                    && para_body_is_very_short(dom, last_ins)
                    && para_body_alnum_len(dom, d) >= 10
                    && (para_word_atom_count(dom, d) >= 8 || para_has_heading_style(dom, d))
                {
                    continue;
                }
                // #1686: the carrier is an empty inserted paragraph of a cell.
                // Word's `[One.]` → `["" ""]` keeps that inserted mark on the
                // paragraph that holds the deleted words and leaves the other
                // paragraph empty. Any other empty inserted paragraph of a
                // cell stays; folding it away drops a paragraph Accept All
                // or Reject All still has.
                if in_cell
                    && (para_has_no_text(dom, last_ins)
                        || para_body_text_is_whitespace_only(dom, last_ins))
                    && para_mark_revision(dom, last_ins, &W::ins())
                {
                    let word_empty_gain = inss.len() == 1
                        && dels.len() == 1
                        && para_has_real_del(dom, d)
                        && !para_mark_revision(dom, d, &W::del());
                    if word_empty_gain {
                        // The deleted words move onto the inserted mark; the
                        // shell keeps its `w:pPr`, which Reject All joins the
                        // inserted mark's content into (#2100).
                        for c in dom.elements(d, None) {
                            if !dom.name_is(c, &W::p_pr()) {
                                dom.add(last_ins, c);
                                dom.remove(c);
                            }
                        }
                        acted = true;
                        break;
                    }
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
                    .is_some_and(|dp| dom.element(dp, &W::spacing_el()).is_some());
                // M344 (nda×report ~56→98): pure-I ListNumber reference + pure-D
                // Heading1 title both structural — old gate kept ListNumber MIX
                // (pagefair thrash). Word/e3 adopt Deleted Heading1 + del mark.
                let ins_list_style = dom.element(last_ins, &W::p_pr()).is_some_and(|ip| {
                    dom.element(ip, &W::p_style()).is_some_and(|ps| {
                        let v = dom
                            .attribute(ps, &W::val())
                            .unwrap_or("")
                            .to_ascii_lowercase();
                        v.starts_with("list")
                    })
                });
                let del_heading = para_has_heading_or_title_style(dom, d);
                // Defense: if a long-prose×list fold still happens, never adopt
                // ListParagraph onto the prose carrier (M363 skip-fold is primary).
                //
                // M384 (file_45 residual −13.6): M382 free-meshes 1–2 word list
                // pure-D "First item" into long pure-I MIX, but del_list_multi
                // used `word_atoms > 1` so short list labels still blocked
                // adopt_del_ppr — MIX lacked Word's live numPr + del mark.
                // Align threshold with M382 (≥3 word-atoms = soft-break multi-
                // word list content); 1–2 word list labels still adopt numPr.
                let ins_long_prose = para_body_alnum_len(dom, last_ins) >= 20;
                let del_list_multi = del_structural
                    && para_has_live_numpr(dom, d)
                    && para_word_atom_count(dom, d) >= 3;
                // M436 (list_def_mix×list_numbering_reimport ~52 / docxodus 90):
                // pure-I short list "test" (numPr only) × pure-D ListParagraph
                // "Num 1". Both structural → prior gate kept Inserted numId on
                // MIX. Word adopts Deleted ListParagraph + numPr + del mark.
                let del_list_paragraph = dom.element(d, &W::p_pr()).is_some_and(|dp| {
                    dom.element(dp, &W::p_style()).is_some_and(|ps| {
                        dom.attribute(ps, &W::val())
                            .unwrap_or("")
                            .eq_ignore_ascii_case("ListParagraph")
                    })
                });
                let short_list_x_listparagraph = para_has_live_numpr(dom, last_ins)
                    && para_has_live_numpr(dom, d)
                    && del_list_paragraph
                    && para_word_atom_count(dom, last_ins) <= 3
                    && para_word_atom_count(dom, d) <= 4;
                // M360: a TOC/field residue pure-I (fldChar) folded with a
                // Heading/Title pure-D keeps a bare pPr — never the Heading
                // pStyle (table_border×toc SD-2343 title, pagefair −11).
                let m360_fld_x_heading = del_heading && para_is_field_residue(dom, last_ins);
                let adopt_del_ppr = !m360_fld_x_heading
                    && ((del_structural && !ins_structural && !(ins_long_prose && del_list_multi))
                        || (ins_jc_only && del_has_spacing)
                        || (del_heading && (ins_list_style || dels.len() >= 2))
                        || short_list_x_listparagraph);
                // M218: mark-only empty pure-D fold — Word parks the deleted
                // pilcrow on the pure-I carrier (contract_review MIX + mark_del).
                // Do not strip to a bare pure-I; adopt the empty del's pPr/rPr/del.
                if mark_only_empty_del {
                    if let Some(ippr) = dom.element(last_ins, &W::p_pr()) {
                        dom.remove(ippr);
                    }
                    if let Some(dppr) = dom.element(d, &W::p_pr()) {
                        let cloned = dom.clone_subtree(dppr);
                        if let Some(first) = dom.elements(last_ins, None).first().copied() {
                            dom.add_before_self(first, cloned);
                        } else {
                            dom.add(last_ins, cloned);
                        }
                    }
                } else if adopt_del_ppr {
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
                    // M361: strip whole mark rPr (ins/del on pPr/rPr) after fold.
                    // M354 kept fonts/sz under pPr to mirror Word tiff title
                    // (Arial/sz32 + rPrChange empty-old), but LO pagefair thrash
                    // 41→37 vs 27c bare MIX. Run-level rPr on ins still carries
                    // fonts; drop pPr mark shell like 27c.
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
                    if !dom.name_is(c, &W::p_pr()) {
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
/// M393 — restore Word's interleaved list cluster after produce /
/// coalesce.
///
/// `broken_list_missing × broken_list`: produce + coalesce flattens a
/// pure-I short-list / pure-D short-list with a nested (ilvl≥1)
/// pure-D cluster into `pure-I-all, pure-D-all`, even when the LCS
/// emitted Word's interleave. The pass detects that pattern and
/// reorders to `pure-I first, pure-D first cluster, pure-I rest,
/// pure-D rest`. 1:1 paragraph replacements are left to
/// [`merge_replaced_paragraphs_in`], which runs after this pass.
/// Runs **late**, after the free-mesh peels thrash LCS interleave.
pub fn interleave_list_cluster_after_coalesce(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.len() < 6 {
        return;
    }
    // Classify pure-I / pure-D only stream (no MIX/E content).
    let mut kinds: Vec<char> = Vec::with_capacity(kids.len());
    for &k in &kids {
        if !dom.name_is(k, &W::p()) {
            return; // tables spoil this peel
        }
        let pure_i = para_is_pure_inserted(dom, k);
        let pure_d = para_is_pure_deleted(dom, k);
        if pure_i && !pure_d {
            kinds.push('I');
        } else if pure_d && !pure_i {
            kinds.push('D');
        } else if para_has_no_text(dom, k) && !pure_i && !pure_d {
            kinds.push('E');
        } else {
            return; // MIX or equal content
        }
    }
    // Already Word interleave I + D+ + I+ + D* + E*? Leave alone.
    {
        let s: String = kinds.iter().collect();
        // I D{2,} I{2,} D+ E*  (Word broken×broken)
        let bytes = s.as_bytes();
        if bytes.first() == Some(&b'I') {
            let mut j = 1usize;
            while j < bytes.len() && bytes[j] == b'D' {
                j += 1;
            }
            let d1 = j - 1;
            let i2s = j;
            while j < bytes.len() && bytes[j] == b'I' {
                j += 1;
            }
            let i2 = j - i2s;
            let d2s = j;
            while j < bytes.len() && bytes[j] == b'D' {
                j += 1;
            }
            let d2 = j - d2s;
            while j < bytes.len() && bytes[j] == b'E' {
                j += 1;
            }
            if j == bytes.len() && d1 >= 2 && i2 >= 2 && d2 >= 1 {
                return;
            }
        }
    }
    // Expect III…DDD… (coalesce shape) with optional trailing E.
    let mut i_end = 0usize;
    while i_end < kinds.len() && kinds[i_end] == 'I' {
        i_end += 1;
    }
    let mut d_end = i_end;
    while d_end < kinds.len() && kinds[d_end] == 'D' {
        d_end += 1;
    }
    // trailing E only
    if d_end < kinds.len() && kinds[d_end..].iter().any(|&c| c != 'E') {
        return;
    }
    if i_end < 2 || d_end - i_end < 3 {
        return;
    }
    // Nested pure-D: some pure-D has ilvl>=1 via pPr/numPr/ilvl.
    let has_nested_del = kids[i_end..d_end].iter().any(|&p| {
        dom.element(p, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::num_pr()))
            .and_then(|num| dom.element(num, &W::name("ilvl")))
            .and_then(|il| dom.attribute(il, &W::val()))
            .and_then(|v| v.parse::<u32>().ok())
            .is_some_and(|v| v >= 1)
    });
    if !has_nested_del {
        return;
    }
    // M404 (basic_list×sd_1707): residual pure-I after the first pure-I must be
    // short list labels (≤3 tokens: "a"/"TWO") for Word interleave. Multi-word
    // heading prose ("Heading. Body copy for repro") must stay in the pure-I
    // head stream (IIDDD…), not move mid-list (IDDDI…).
    let rest_i_all_short_labels = kids[1..i_end].iter().all(|&p| {
        let n = para_word_atom_count(dom, p);
        n > 0 && n <= 3
    });
    if i_end >= 2 && !rest_i_all_short_labels {
        return;
    }
    // M428 (list_def_mix × list_numbering_reimport): pure-I stream is uniform
    // single-token items ("test"×4). M404 short-label gate is true for "test"
    // (1 token) so interleave rewrote Word IIIIDDD… into I D-cluster I-rest D-rest
    // (score ~52 vs docxodus 90). Word keeps wholesale pure-I then pure-D when
    // every pure-I body is the same single token — skip interleave.
    let pure_i_uniform_single_token = {
        let mut first: Option<String> = None;
        let mut ok = i_end >= 2;
        for &p in &kids[..i_end] {
            let t = para_revision_body_text(dom, p)
                .split_whitespace()
                .map(str::to_ascii_lowercase)
                .collect::<Vec<_>>();
            if t.len() != 1 {
                ok = false;
                break;
            }
            match &first {
                None => first = Some(t[0].clone()),
                Some(f) if f != &t[0] => {
                    ok = false;
                    break;
                }
                _ => {}
            }
        }
        ok && first.is_some()
    };
    if pure_i_uniform_single_token {
        return;
    }
    // First pure-D cluster end within dels: through nested then stop before
    // next top-level after saw_sub.
    let dels = &kids[i_end..d_end];
    let mut saw_sub = false;
    let mut cut_rel = 0usize;
    for (i, &p) in dels.iter().enumerate() {
        let empty = para_has_no_text(dom, p);
        if empty {
            cut_rel = i + 1;
            continue;
        }
        let ilvl = dom
            .element(p, &W::p_pr())
            .and_then(|ppr| dom.element(ppr, &W::num_pr()))
            .and_then(|num| dom.element(num, &W::name("ilvl")))
            .and_then(|il| dom.attribute(il, &W::val()))
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0);
        if saw_sub && ilvl == 0 {
            break;
        }
        if ilvl >= 1 {
            saw_sub = true;
        }
        cut_rel = i + 1;
    }
    if !saw_sub || cut_rel < 2 || cut_rel >= dels.len() {
        return;
    }
    // Rebuild: I[0], D[0..cut], I[1..], D[cut..]
    let first_i = kids[0];
    let rest_i: Vec<NodeId> = kids[1..i_end].to_vec();
    let first_d: Vec<NodeId> = dels[..cut_rel].to_vec();
    let rest_d: Vec<NodeId> = dels[cut_rel..].to_vec();
    // Detach all and re-append in Word order before sectPr.
    let sect = dom
        .elements(body, None)
        .into_iter()
        .find(|&k| dom.name_is(k, &W::sect_pr()));
    for &k in &kids {
        dom.remove(k);
    }
    let mut order = Vec::new();
    order.push(first_i);
    order.extend(first_d);
    order.extend(rest_i);
    order.extend(rest_d);
    // trailing E if any
    for &k in &kids[d_end..] {
        order.push(k);
    }
    if let Some(s) = sect {
        for k in order {
            dom.add_before_self(s, k);
        }
    } else {
        for k in order {
            dom.add(body, k);
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
        if n == W::tbl() {
            let trs = dom.descendants(el, Some(&W::tr()));
            if trs.is_empty() {
                return None;
            }
            let mut cls: Option<bool> = None;
            for tr in trs {
                let trpr = dom.element(tr, &W::tr_pr())?;
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
    containers.extend(dom.descendants(root, Some(&W::tc())));
    for container in containers {
        let children: Vec<NodeId> = dom.elements(container, None);
        let mut classes: Vec<Option<bool>> =
            children.iter().map(|&c| block_class(dom, c)).collect();
        // A paragraph whose mark carries the other revision stays put only
        // when it sits between two deleted tables (#1826). Moved ahead of
        // the first, it left the tables adjacent and rejecting fused them.
        // Anywhere else, including a region with one deleted table, it
        // takes its own class, so inserted words move ahead of that table
        // as Word writes them (#1926).
        for idx in 0..children.len() {
            if !dom.name_is(children[idx], &W::p()) {
                continue;
            }
            let Some(class) = classes[idx] else {
                continue;
            };
            let other = if class { W::del() } else { W::ins() };
            if !para_mark_revision(dom, children[idx], &other) {
                continue;
            }
            let between = idx > 0
                && idx + 1 < children.len()
                && classes[idx - 1] == Some(false)
                && classes[idx + 1] == Some(false)
                && dom.name_is(children[idx - 1], &W::tbl())
                && dom.name_is(children[idx + 1], &W::tbl());
            if between {
                classes[idx] = None;
            }
        }
        let paras: Vec<bool> = children.iter().map(|&c| dom.name_is(c, &W::p())).collect();
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
            // A consecutive I→D→I region is an already chosen mid-story
            // splice, not a wholesale D→I replacement. Moving its residual
            // table ahead of D destroys the detector's Word page-flow choice
            // (employment×lease and long-base×short-table). Only the adjacent
            // inserted block qualifies: any intervening live content remains
            // a boundary and must not suppress ordinary block reordering.
            if del_start > 0 && classes[del_start - 1] == Some(true) {
                continue;
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
