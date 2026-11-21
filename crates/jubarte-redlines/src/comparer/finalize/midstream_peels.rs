// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

//! Word-mode midstream peels: revised rows, short pure-deletion splices, MIX trailing peels, demo-title fold.

use super::*;

/// Word-alignment mode (settings-gated): Word marks a fully-deleted table
/// row with `w:del` inside `w:trPr` (and fully-inserted rows with `w:ins`),
/// in addition to the cell-content revisions — the accept pipeline then
/// removes the row (and the whole table once every row is marked). Our
/// produce path deletes cell CONTENT only, leaving a ghost empty table after
/// acceptance (meeting-agenda/employee-directory benchmark evidence; the C#
/// oracle instead drops the table structure entirely — further from Word).
/// Stamp `w:trPr/w:ins` on every row whose cells all carry an
/// inserted run (`fullyRevisedRow` in the C# port).
///
/// The `revision_id` shared with the `sectPrChange` is stamped at
/// [`fix_up_revision_ids`] time; this pass only decides which rows
/// qualify (every cell is fully revised or empty) and wires up the
/// carrier.
pub fn mark_fully_revised_rows(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let tr_name = W::tr();
    let tc_name = W::tc();
    let trs: Vec<NodeId> = dom.descendants(root, Some(&tr_name));
    for tr in trs {
        let mut class: Option<bool> = None; // Some(false)=all-del, Some(true)=all-ins
        let mut any_content = false;
        let mut mixed = false;
        for tc in dom.elements(tr, Some(&tc_name)) {
            for p in dom.descendants(tc, Some(&W::p())) {
                match para_replacement_class_with_moves(dom, p, true) {
                    Some(k) => {
                        any_content = true;
                        if class.get_or_insert(k) != &k {
                            mixed = true;
                        }
                    }
                    None => {
                        // Mixed paragraphs with content or an unchanged pilcrow
                        // disqualify the row. Content = text OR
                        // non-text payloads (drawing/object/pict/sym) — a row
                        // holding an unchanged drawing must never be marked
                        // fully revised, or the accept pass drops content the
                        // other document still has (PR #54 review).
                        // Reuse the atomizer's payload vocabulary. Textless
                        // tabs, fields, references and math own real source
                        // content just as drawings do. Inspect body children
                        // so direct math/empty fields are covered, excluding
                        // paragraph-mark formatting and layout metadata.
                        let content_bearing = dom.elements(p, None).into_iter().any(|body| {
                            !dom.name_is(body, &W::p_pr())
                                && (!dom.value(body).trim().is_empty()
                                    || std::iter::once(body)
                                        .chain(dom.descendants(body, None))
                                        .any(|node| {
                                            dom.name(node).is_some_and(|name| {
                                                ALLOWABLE_RUN_CHILDREN.contains(&name)
                                                    || name == W::object()
                                                    || name == W::pict()
                                                    || name == MC::alternate_content()
                                                    || name == W::fld_simple()
                                            })
                                        }))
                        });
                        // A live pilcrow is an independent structural owner,
                        // including an empty paragraph with only formatting.
                        // Equal row properties need no trPrChange, so property
                        // history cannot be the only evidence of paired rows.
                        let live_pilcrow = [W::ins(), W::del(), W::move_to(), W::move_from()]
                            .iter()
                            .all(|kind| !para_mark_revision(dom, p, kind));
                        if content_bearing || live_pilcrow {
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
        let trpr = match dom.element(tr, &W::tr_pr()) {
            Some(p) => p,
            None => {
                let p = dom.new_element(W::tr_pr());
                dom.add_first(tr, p);
                p
            }
        };
        // Deleted text does not delete a row the other document kept.
        // A surviving paragraph mark is that row's owner, including when
        // the row properties are equal and carry no `w:trPrChange`
        // (#1672). A row the other document dropped has its paragraph
        // marks deleted, so it still receives the row mark. An existing
        // row revision still controls a wholesale lifetime.
        let already_owned =
            dom.element(trpr, &W::ins()).is_some() || dom.element(trpr, &W::del()).is_some();
        if already_owned {
            continue;
        }
        let survives_direction = |p| {
            if k {
                !para_mark_revision(dom, p, &W::ins()) && !para_mark_revision(dom, p, &W::move_to())
            } else {
                !para_mark_revision(dom, p, &W::del())
                    && !para_mark_revision(dom, p, &W::move_from())
            }
        };
        let p_name = W::p();
        if dom.elements(tr, Some(&tc_name)).into_iter().any(|tc| {
            dom.find_descendant_element(tc, Some(&p_name), survives_direction)
                .is_some()
        }) {
            continue;
        }
        let rev_name = if k { W::ins() } else { W::del() };
        if dom.element(trpr, &rev_name).is_some() {
            continue; // already marked (e.g. by MarkRows on the row path)
        }
        let rev = dom.new_element(rev_name);
        dom.set_attribute_value(rev, &W::author(), Some(&settings.author_for_revisions));
        dom.set_attribute_value(rev, &W::id(), Some(&id_gen.to_string()));
        *id_gen += 1;
        dom.set_attribute_value(rev, &W::date(), Some(&settings.date_time_for_revisions));
        super::lcs_table::add_row_mark(dom, trpr, rev);
    }
}

/// Whole-doc short-base × long-next: insert-all-next + delete-all-base trails
/// pure-D. Word nests the short original mid-stream near TOC/Tip (document_100)
/// or peels a short title pure-D after the first "1." heading (double_spacing).
///
/// Pixel note: parking pure-D *snug* after Tip with Heading1 styles regressed
/// document_100 (45→40) — LO page geometry prefers the mid-body placement that
/// results when D…D I…I reordering follows the early splice. Keep a single
/// pre-merge splice only.
/// Short pure-D base trailing after `insert-all-next` → splice the
/// pure-D row mid-stream near where a TOC / tip would jump to it.
///
/// `document_100 × comments`: Word nests the original on page 2 of
/// the redline (`…comments … insert-new … ORIGINAL small body`),
/// not at the very end. The pass detects a short trailing pure-D
/// after a wholesale insert run and moves it to the next anchor
/// position inside a TOC / section break / heading paragraph.
pub fn splice_trailing_short_pure_dels_midstream(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    if kids.len() < 10 {
        return;
    }
    // Trailing pure-deleted paragraph run.
    let del_end = kids.len();
    let mut del_start = kids.len();
    while del_start > 0 {
        let k = kids[del_start - 1];
        if !dom.name_is(k, &W::p()) || !para_is_pure_deleted(dom, k) {
            break;
        }
        del_start -= 1;
    }
    let del_count = del_end - del_start;
    if !(1..=6).contains(&del_count) {
        return;
    }
    let prefix = &kids[..del_start];
    if prefix.len() < 40 {
        return;
    }
    let pure_ins = prefix
        .iter()
        .filter(|&&k| {
            if dom.name_is(k, &W::p()) {
                para_is_pure_inserted(dom, k)
            } else if dom.name_is(k, &W::tbl()) {
                let has_ins = !dom.descendants(k, Some(&W::ins())).is_empty();
                let has_del = !dom.descendants(k, Some(&W::del())).is_empty();
                has_ins && !has_del
            } else {
                false
            }
        })
        .count();
    if pure_ins * 2 < prefix.len() {
        return;
    }
    let mut tip_toc: Option<usize> = None;
    let mut first_numbered: Option<usize> = None;
    for (i, &k) in prefix.iter().enumerate() {
        if !dom.name_is(k, &W::p()) {
            continue;
        }
        let mut text = String::new();
        for t in dom.descendants(k, Some(&W::t())) {
            text.push_str(&dom.value_str(t));
        }
        let lower = text.to_ascii_lowercase();
        let trimmed = lower.trim_start();
        if lower.contains("tip:") {
            tip_toc = Some(i);
            break;
        }
        if lower.contains("table of contents") {
            tip_toc = Some(i);
        }
        if first_numbered.is_none()
            && (trimmed.starts_with("1.") || trimmed.starts_with("1 "))
            && trimmed.chars().count() >= 8
        {
            first_numbered = Some(i);
        }
    }
    let Some(insert_after) = tip_toc.or(first_numbered) else {
        return;
    };
    if insert_after >= del_start {
        return;
    }
    let to_move: Vec<NodeId> = if tip_toc.is_some() {
        kids[del_start..del_end].to_vec()
    } else {
        let first = kids[del_start];
        if !para_is_pure_deleted(dom, first) {
            return;
        }
        let first_alnum = para_body_alnum_len(dom, first);
        if first_alnum == 0 || first_alnum > 40 {
            return;
        }
        let rest = &kids[del_start + 1..del_end];
        if rest.is_empty() {
            vec![first]
        } else {
            let rest_all_longer = rest.iter().all(|&d| {
                dom.name_is(d, &W::p())
                    && para_is_pure_deleted(dom, d)
                    && para_body_alnum_len(dom, d) > first_alnum + 10
            });
            if rest_all_longer {
                vec![first]
            } else {
                return; // sales_report equal-length lines
            }
        }
    };
    // Crossing any original content changes the rejected source order.
    // A table disappears with its structurally inserted direct rows, including
    // all nested payload owned by those rows. Revised cell text alone cannot
    // establish that ownership: an existing row may still survive rejection.
    let disappears_on_rejection = |node| {
        if dom.name_is(node, &W::p()) {
            return para_replacement_class(dom, node) == Some(true);
        }
        if !dom.name_is(node, &W::tbl()) {
            return false;
        }
        let rows = dom.elements(node, Some(&W::tr()));
        !rows.is_empty()
            && rows.iter().all(|&row| {
                dom.element(row, &W::tr_pr()).is_some_and(|properties| {
                    dom.element(properties, &W::ins()).is_some()
                        && dom.element(properties, &W::del()).is_none()
                })
            })
    };
    if prefix[insert_after + 1..]
        .iter()
        .any(|&node| !disappears_on_rejection(node))
    {
        return;
    }
    let anchor = prefix[insert_after];
    for &n in &to_move {
        dom.remove(n);
    }
    let mut prev = anchor;
    for &n in &to_move {
        dom.add_after_self(prev, n);
        prev = n;
    }
}

/// M147 — MIX that is **only** leading pure-ins digits ("24") + pure-del demo
/// title body (1_5_line_spacing×24): Word keeps pure-I "24" then pure-D title
/// as separate paragraphs. Produce/LCS nests them into one MIX ("241.5 Line…").
/// Split: pure-I digits para, pure-D remainder.
/// M147 — split digits-only pure-I + pure-D title from a MIX title
/// into a separate paragraph (Word title-only-on-MIX shape).
///
/// `1_5 × 24` Word shape: a title line with purely-numeric MIX is
/// split into the title paragraph (MIX with non-digit tokens) and a
/// separate digits-only MIX paragraph. The split paragraph carries
/// the section-heading-style digits as MIX so the section heading
/// Word redline keeps its visual chunking.
pub fn split_digits_ins_from_mixed_title(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let kids: Vec<NodeId> = dom
        .elements(body, None)
        .into_iter()
        .filter(|&k| !dom.name_is(k, &W::sect_pr()))
        .collect();
    for &mix in &kids {
        if !dom.name_is(mix, &W::p()) {
            continue;
        }
        let has_ins = !dom.descendants(mix, Some(&W::ins())).is_empty();
        let has_del = !dom.descendants(mix, Some(&W::del())).is_empty();
        if !has_ins || !has_del {
            continue;
        }
        let mix_kids: Vec<NodeId> = dom.elements(mix, None);
        let mut leading_ins: Vec<NodeId> = Vec::new();
        let mut rest: Vec<NodeId> = Vec::new();
        let mut in_leading = true;
        for &c in &mix_kids {
            if dom.name_is(c, &W::p_pr()) {
                continue;
            }
            if in_leading && dom.name_is(c, &W::ins()) {
                leading_ins.push(c);
            } else {
                in_leading = false;
                rest.push(c);
            }
        }
        if leading_ins.is_empty() || rest.is_empty() {
            continue;
        }
        // Leading ins must be digits-only; rest must be pure-del (no ins).
        let mut ins_text = String::new();
        for &ins_n in &leading_ins {
            for t in dom.descendants(ins_n, Some(&W::t())) {
                ins_text.push_str(&dom.value_str(t));
            }
        }
        let trimmed = ins_text.trim();
        if trimmed.is_empty()
            || !trimmed
                .chars()
                .all(|c| c.is_ascii_digit() || c.is_whitespace())
        {
            continue;
        }
        let rest_has_ins = rest
            .iter()
            .any(|&c| dom.name_is(c, &W::ins()) || !dom.descendants(c, Some(&W::ins())).is_empty());
        let rest_has_del = rest
            .iter()
            .any(|&c| dom.name_is(c, &W::del()) || !dom.descendants(c, Some(&W::del())).is_empty());
        if rest_has_ins || !rest_has_del {
            continue;
        }
        // Build pure-I para from leading ins; leave rest as pure-D on original.
        let pure_i = dom.new_element(W::p());
        for &ins_n in &leading_ins {
            if dom.parent(ins_n).is_some() {
                dom.remove(ins_n);
                dom.add(pure_i, ins_n);
            }
        }
        dom.add_before_self(mix, pure_i);
    }
}

/// M144 — MIX with trailing pure-ins + following pure-D body: when the trailing
/// insert and pure-D share a connector/content word (italic×justified: both
/// have `for`), peel the trailing ins into the pure-D as a mixed para via
/// simple word interleave (Word: del prefix + Equal` for ` + ins phrase +
/// del rest). Without this, residual LCS parks the whole next phrase on the
/// first body MIX and leaves pure-D last (~64 score).
/// M143 — peel a trailing insertion off a MIX paragraph onto the
/// following pure-D body when they share a token.
///
/// `italic × justified` "for a formal document look": Word streams
/// the insertion's trailing token onto the following pure-D as a
/// leading equality (`pure-D body … for a formal document look`),
/// leaving the MIX paragraph as pure-D with its remaining tokens.
/// Different from M144 (trailing `del`) and the M460 shared-sig
/// free-mesh.
pub fn peel_trailing_ins_from_mix_into_following_pure_del(dom: &mut Dom, root: NodeId) {
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
            let mix_p = kids[i];
            let del_p = kids[i + 1];
            if !dom.name_is(mix_p, &W::p()) || !dom.name_is(del_p, &W::p()) {
                continue;
            }
            if !para_is_pure_deleted(dom, del_p) {
                continue;
            }
            let has_ins = !dom.descendants(mix_p, Some(&W::ins())).is_empty();
            let _has_del = !dom.descendants(mix_p, Some(&W::del())).is_empty()
                || para_mark_revision(dom, mix_p, &W::del());
            // MIX or pure-I with trailing ins both OK (ours is MIX).
            if !has_ins {
                continue;
            }
            let del_text = para_revision_body_text(dom, del_p);
            let del_toks: Vec<String> = del_text
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| !t.is_empty())
                .map(|t| t.to_ascii_lowercase())
                .collect();
            // Content pure-D body (not short demo title).
            if del_toks.len() < 6 {
                continue;
            }
            // Trailing contiguous w:ins at end of mix. Skip trailing bare runs
            // that are only punctuation/whitespace (EQ `.` after the peeled phrase).
            let mix_kids: Vec<NodeId> = dom.elements(mix_p, None);
            let mut trailing_ins: Vec<NodeId> = Vec::new();
            let mut saw_ins = false;
            for &c in mix_kids.iter().rev() {
                if dom.name_is(c, &W::p_pr()) {
                    continue;
                }
                if dom.name_is(c, &W::ins()) {
                    trailing_ins.push(c);
                    saw_ins = true;
                    continue;
                }
                // Allow a pure-punctuation bare run after the ins (period).
                if !saw_ins && dom.name_is(c, &W::r()) {
                    let mut t = String::new();
                    for tn in dom.descendants(c, Some(&W::t())) {
                        t.push_str(&dom.value_str(tn));
                    }
                    if t.chars().all(|ch| !ch.is_alphanumeric()) {
                        continue;
                    }
                }
                break;
            }
            trailing_ins.reverse();
            if trailing_ins.is_empty() {
                continue;
            }
            // Must leave some non-ins content on mix (not peel whole para).
            let non_ins = mix_kids.iter().any(|&c| {
                let n = dom.name(c);
                n != Some(W::p_pr()) && n != Some(W::ins())
            });
            if !non_ins {
                continue;
            }
            let mut ins_text = String::new();
            for &ins_n in &trailing_ins {
                for t in dom.descendants(ins_n, Some(&W::t())) {
                    ins_text.push_str(&dom.value_str(t));
                }
            }
            let ins_toks: Vec<String> = ins_text
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| !t.is_empty())
                .map(|t| t.to_ascii_lowercase())
                .collect();
            if ins_toks.is_empty() || ins_toks.len() > 12 {
                continue;
            }
            // Share a connector or content token (len≥3) so we only peel when
            // Word-style word LCS would bridge the paras.
            let del_set: std::collections::HashSet<&str> =
                del_toks.iter().map(|s| s.as_str()).collect();
            let shared = ins_toks
                .iter()
                .any(|t| t.chars().count() >= 3 && del_set.contains(t.as_str()));
            if !shared {
                continue;
            }
            // Move trailing ins onto pure-D: insert before first del body child
            // so order is ...ins... then existing del body. Produce already has
            // del body as w:del; prepending ins yields MIX. Word interleaves
            // more finely; this lifts pure-D→MIX and relocates the phrase.
            let del_body_first = dom
                .elements(del_p, None)
                .into_iter()
                .find(|&c| !dom.name_is(c, &W::p_pr()));
            for &ins_n in &trailing_ins {
                if dom.parent(ins_n).is_none() {
                    continue;
                }
                dom.remove(ins_n);
                if let Some(first) = del_body_first {
                    dom.add_before_self(first, ins_n);
                } else {
                    dom.add(del_p, ins_n);
                }
            }
            acted = true;
            break;
        }
        if !acted {
            break;
        }
    }
}

/// M159 (text_highlight×times): after merge reorders unrelated 1v1 D-then-I to
/// I-before-D, restore Word order when pure-I is longer than a short pure-D
/// and they sit after a MIX title: `… MIX | INS | DEL | MIX …` →
/// `… MIX | DEL | INS | MIX …`. Heading 3×MIX zip never leaves pure I/D pairs,
/// so it is unaffected.
/// M159 — restore the order `short pure-D before long pure-I` after
/// the merge-replaced paragraphs reorder.
///
/// `text_highlight × times`: after the paragraph merge reorders
/// runs, a short trailing pure-D should appear again before the
/// longer pure-I (`short-del | long-ins`). The pass recognizes that
/// shape and moves the deleted paragraph back to its original place.
pub fn restore_short_del_before_long_ins(dom: &mut Dom, root: NodeId) {
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
            // M159 only after a MIX residual peel — never at body start.
            // M217 (q1_sales×quarterly): leading pure-I title then pure-D
            // title is Word's IDE… shape; the old i==0 fall-through swapped
            // short del before long ins and undid reorder_replacements.
            if i == 0 {
                continue;
            }
            let prev = kids[i - 1];
            if !dom.name_is(prev, &W::p()) {
                continue;
            }
            {
                let has_ins = !dom.descendants(prev, Some(&W::ins())).is_empty()
                    || para_mark_revision(dom, prev, &W::ins());
                let has_del = !dom.descendants(prev, Some(&W::del())).is_empty()
                    || para_mark_revision(dom, prev, &W::del());
                if !(has_ins && has_del) {
                    continue;
                }
                // A stamped filename is a comparison anchor, not the M159
                // content-title MIX. Swapping after it moves a short base
                // title ahead of the long next document's main title
                // (M104/M108), reversing Word's order.
                let prev_text = para_revision_body_text(dom, prev).to_ascii_lowercase();
                if prev_text.contains("file_")
                    || prev_text.contains(".docx")
                    || prev_text.contains(".doc")
                {
                    continue;
                }
            }
            let d_len = para_body_alnum_len(dom, del_p);
            let i_len = para_body_alnum_len(dom, ins_p);
            if d_len == 0 || d_len >= i_len || d_len > 40 {
                continue;
            }
            // Swap: move del before ins.
            if dom.parent(del_p).is_none() || dom.parent(ins_p).is_none() {
                continue;
            }
            dom.remove(del_p);
            dom.add_before_self(ins_p, del_p);
            acted = true;
            break;
        }
        if !acted {
            break;
        }
    }
}

/// M154 (justified_underline×justify_2): trailing `w:del` on a MIX followed by
/// pure-I body — peel the del into the pure-I so both become MIX. Word moves
/// "with underline formatting for a formal document look" onto the next
/// residual body (ours left pure-I ~64.8). Mirror of M144 (ins→pure-D).
/// M154 — peel a trailing deletion off a MIX paragraph onto the
/// following pure-I paragraph when they share a token.
///
/// `justified_underline × justify_2`: the MIX trailing deletion
/// (`"underline"` here) is re-streamed as a leading deletion onto
/// the following pure-I so the underlined paragraph reads cleanly.
pub fn peel_trailing_del_from_mix_into_following_pure_ins(dom: &mut Dom, root: NodeId) {
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
            let mix_p = kids[i];
            let ins_p = kids[i + 1];
            if !dom.name_is(mix_p, &W::p()) || !dom.name_is(ins_p, &W::p()) {
                continue;
            }
            if !para_is_pure_inserted(dom, ins_p) {
                continue;
            }
            let has_del = !dom.descendants(mix_p, Some(&W::del())).is_empty()
                || para_mark_revision(dom, mix_p, &W::del());
            let has_ins = !dom.descendants(mix_p, Some(&W::ins())).is_empty();
            if !has_del || !has_ins {
                continue;
            }
            if !peel_has_text_only_body(dom, mix_p) || !peel_has_text_only_body(dom, ins_p) {
                continue;
            }
            // A's closing mark belongs to its paragraph even when every
            // original word is deleted. Moving that body under another mark
            // would split A or leave an extra empty A paragraph on rejection.
            // An inserted boundary already joins the carrier to its successor;
            // otherwise only the final identical-property continuation can
            // transfer that boundary without changing either source layout.
            let transfer_final_boundary = !para_mark_revision(dom, mix_p, &W::ins());
            if transfer_final_boundary
                && (i + 1 != kids.len() - 1
                    || !final_peel_has_identical_closing_properties(dom, mix_p, ins_p))
            {
                continue;
            }
            let mix_kids: Vec<NodeId> = dom.elements(mix_p, None);
            let mut trailing_del: Vec<NodeId> = Vec::new();
            let mut saw_del = false;
            for &c in mix_kids.iter().rev() {
                if dom.name_is(c, &W::p_pr()) {
                    continue;
                }
                if dom.name_is(c, &W::del()) {
                    trailing_del.push(c);
                    saw_del = true;
                    continue;
                }
                if !saw_del && dom.name_is(c, &W::r()) {
                    let mut t = String::new();
                    for tn in dom.descendants(c, Some(&W::t())) {
                        t.push_str(&dom.value_str(tn));
                    }
                    // Only Word's single borrowed sentence period can stay
                    // here: its later repair records it on the completed side
                    // and appends it to the source continuation. Skipping any
                    // other retained suffix reverses A's literal text order.
                    if t == "."
                        && trailing_equal_period(dom, mix_p).and_then(|t| dom.parent(t)) == Some(c)
                    {
                        continue;
                    }
                }
                break;
            }
            trailing_del.reverse();
            if trailing_del.is_empty() {
                continue;
            }
            if let Some(period) = trailing_equal_period(dom, mix_p) {
                let Some(last_deleted_text) = trailing_del
                    .last()
                    .and_then(|&d| text_leaves(dom, d).last().copied())
                else {
                    continue;
                };
                let run_properties = |dom: &Dom, text: NodeId| {
                    dom.parent(text)
                        .and_then(|run| dom.element(run, &W::r_pr()))
                        .map(|rpr| dom.serialize_element(rpr))
                        .unwrap_or_default()
                };
                // The later period repair appends a character to this deleted
                // run. Decline if that would change the original period's
                // formatting; the unchanged source run keeps its own period.
                if run_properties(dom, period) != run_properties(dom, last_deleted_text) {
                    continue;
                }
            }
            // Leave non-del content on mix.
            let non_del = mix_kids.iter().any(|&c| {
                let n = dom.name(c);
                n != Some(W::p_pr()) && n != Some(W::del())
            });
            if !non_del {
                continue;
            }
            let mut del_text = String::new();
            for &d in &trailing_del {
                for t in dom.descendants(d, Some(&W::del_text())) {
                    del_text.push_str(&dom.value_str(t));
                }
                for t in dom.descendants(d, Some(&W::t())) {
                    del_text.push_str(&dom.value_str(t));
                }
            }
            let del_toks: Vec<String> = del_text
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| !t.is_empty())
                .map(|t| t.to_ascii_lowercase())
                .collect();
            // Substantial trailing phrase (formal document look class).
            if del_toks.len() < 4 || del_toks.len() > 20 {
                continue;
            }
            let ins_text = para_revision_body_text(dom, ins_p);
            let ins_toks: Vec<String> = ins_text
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| !t.is_empty())
                .map(|t| t.to_ascii_lowercase())
                .collect();
            if ins_toks.len() < 4 {
                continue;
            }
            // Prefer peeling when next body is a cousin residual (shared
            // justify/document boilerplate), when del phrase is long, or when
            // next pure-I is a long residual body that can absorb a short
            // trailing del (font_color×font_family: "is in red color" → last
            // pure-I "Different fonts…"; without peel LO ~83 vs Word MIX last).
            let ins_set: std::collections::HashSet<&str> =
                ins_toks.iter().map(|s| s.as_str()).collect();
            let shared = del_toks
                .iter()
                .any(|t| t.chars().count() >= 4 && ins_set.contains(t.as_str()));
            let long_trail = del_toks.len() >= 6;
            // Short trail onto long pure-I (font_color "is in red color" → last
            // body). Do NOT peel full residual demo sentences onto list-item
            // pure-I (title×track_changes_editing_bullet regressed 100→50).
            let next_looks_list = ins_toks.iter().any(|t| {
                t == "bullet"
                    || t == "item"
                    || t == "point"
                    || t == "first"
                    || t == "second"
                    || t == "third"
            });
            let short_copula = del_toks
                .first()
                .is_some_and(|t| t == "is" || t == "are" || t == "in");
            let short_trail_long_ins = del_toks.len() >= 4
                && del_toks.len() <= 5
                && ins_toks.len() >= 6
                && !next_looks_list
                && short_copula;
            if !shared && !long_trail && !short_trail_long_ins {
                continue;
            }
            if transfer_final_boundary {
                // The receiving final paragraph's inserted mark becomes a
                // continuation of A only after this peel. Insert a matching
                // boundary on the preceding MIX so rejecting rejoins A's
                // words. Both closing property sets were proved identical.
                let closing_mark = dom
                    .element(ins_p, &W::p_pr())
                    .and_then(|ppr| dom.element(ppr, &W::r_pr()))
                    .and_then(|rpr| dom.element(rpr, &W::ins()))
                    .expect("validated inserted closing mark");
                let mark = dom.clone_subtree(closing_mark);
                let next_id = max_numeric_id(dom, root).saturating_add(1);
                dom.set_attribute_value(mark, &W::id(), Some(&next_id.to_string()));
                let ppr = match dom.element(mix_p, &W::p_pr()) {
                    Some(ppr) => ppr,
                    None => {
                        let ppr = dom.new_element(W::p_pr());
                        dom.add_first(mix_p, ppr);
                        ppr
                    }
                };
                let rpr = match dom.element(ppr, &W::r_pr()) {
                    Some(rpr) => rpr,
                    None => {
                        let rpr = dom.new_element(W::r_pr());
                        dom.add(ppr, rpr);
                        rpr
                    }
                };
                dom.add_first(rpr, mark);
            }
            // Append trailing del after ins body content.
            for &d in &trailing_del {
                if dom.parent(d).is_none() {
                    continue;
                }
                dom.remove(d);
                dom.add(ins_p, d);
            }
            acted = true;
            break;
        }
        if !acted {
            break;
        }
    }
}

/// Keep nontext payload under its original paragraph ownership during M154.
pub(super) fn peel_has_text_only_body(dom: &Dom, node: NodeId) -> bool {
    if dom.name_is(node, &W::p_pr()) || dom.name_is(node, &W::r_pr()) {
        return true;
    }
    let Some(name) = dom.name(node) else {
        return true;
    };
    if ![W::p(), W::r(), W::t(), W::del_text(), W::ins(), W::del()].contains(&name) {
        return false;
    }
    dom.elements(node, None)
        .into_iter()
        .all(|child| peel_has_text_only_body(dom, child))
}

/// Only the final inserted carrier can take a source continuation without
/// losing its paragraph formatting. Property histories, section breaks and
/// non-text payload remain under their original ownership instead.
pub(super) fn final_peel_has_identical_closing_properties(
    dom: &mut Dom,
    mix: NodeId,
    inserted: NodeId,
) -> bool {
    if !peel_has_text_only_body(dom, mix) || !peel_has_text_only_body(dom, inserted) {
        return false;
    }
    let source = dom.element(mix, &W::p_pr());
    let Some(revised) = dom.element(inserted, &W::p_pr()) else {
        return false;
    };
    for ppr in source.into_iter().chain([revised]) {
        if dom.element(ppr, &W::p_pr_change()).is_some()
            || dom.element(ppr, &W::sect_pr()).is_some()
        {
            return false;
        }
        if let Some(rpr) = dom.element(ppr, &W::r_pr())
            && dom.elements(rpr, None).into_iter().any(|child| {
                [W::del(), W::move_from(), W::move_to(), W::r_pr_change()]
                    .iter()
                    .any(|name| dom.name_is(child, name))
            })
        {
            return false;
        }
    }
    if dom
        .element(revised, &W::r_pr())
        .and_then(|rpr| dom.element(rpr, &W::ins()))
        .is_none()
    {
        return false;
    }
    let old = source
        .map(|ppr| dom.clone_subtree(ppr))
        .unwrap_or_else(|| dom.new_element(W::p_pr()));
    let new = dom.clone_subtree(revised);
    for ppr in [old, new] {
        if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
            for inserted in dom.elements(rpr, Some(&W::ins())) {
                dom.remove(inserted);
            }
            if dom.elements(rpr, None).is_empty() {
                dom.remove(rpr);
            }
        }
        remove_powertools_scratch_markup(dom, ppr);
    }
    dom.serialize_element(old) == dom.serialize_element(new)
}

/// M143 — fold a mid-stream pure-D short "Demo" title among the
/// pure-I body of a long revised document into the first numbered
/// heading instead of leaving it as a free pure-D.
///
/// `double_spacing × eigenpal`: Word writes `1. What this is + del
/// "Double Spacing Bold Demo"` rather than an unpaired pure-D
/// between the intro paragraphs. The pass peels the pure-D right
/// after the heading and folds the deleted body into it (bare mixed
/// paragraph, no paragraph-mark revision). Test-only entry — the
/// production pipeline calls
/// [`fold_midstream_demo_title_into_numbered_heading_with_settings`].
#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(super) fn fold_midstream_demo_title_into_numbered_heading(dom: &mut Dom, root: NodeId) {
    let mut id = max_numeric_id(dom, root).saturating_add(1);
    fold_midstream_demo_title_into_numbered_heading_with_settings(
        dom,
        root,
        &WmlComparerSettings::default(),
        &mut id,
    );
}

/// Snapshot every pure-deleted paragraph that looks like a "Demo"
/// title's `w:pPr` BEFORE [`strip_redundant_demo_default_spacing`]
/// runs, so M143's fold can re-attach authored Heading properties
/// inside `w:pPrChange`.
///
/// Snapshots never reach the output except as schema-valid
/// paragraph / mark history for an actual title fold.
pub(crate) fn capture_deleted_demo_title_properties(
    dom: &mut Dom,
    root: NodeId,
) -> std::collections::HashMap<NodeId, NodeId> {
    let paragraphs = dom.descendants(root, Some(&W::p()));
    let mut properties = std::collections::HashMap::new();
    for paragraph in paragraphs {
        if para_is_pure_deleted(dom, paragraph)
            && para_looks_like_demo_title(dom, paragraph)
            && let Some(ppr) = dom.element(paragraph, &W::p_pr())
        {
            properties.insert(paragraph, dom.clone_subtree(ppr));
        }
    }
    properties
}

/// M143 with explicit settings and id-gen — production entry point.
pub fn fold_midstream_demo_title_into_numbered_heading_with_settings(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    fold_midstream_demo_title_into_numbered_heading_with_source_properties(
        dom,
        root,
        settings,
        id_gen,
        &std::collections::HashMap::new(),
    );
}

/// M143 fold that re-attaches captured authored Heading properties
/// inside `w:pPrChange` so the deleted-title's formatting history
/// survives the merge.
///
/// `source_properties` is the snapshot from
/// `capture_deleted_demo_title_properties`; an empty map keeps the
/// demo-default live spacing instead.
pub fn fold_midstream_demo_title_into_numbered_heading_with_source_properties(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
    source_properties: &std::collections::HashMap<NodeId, NodeId>,
) {
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
    // M465 — anchored documents don't fold. When the first body paragraph is
    // a MIX (matched leading title, "file_13.docx" ↔ "file_14.docx" merged),
    // Word treats the rest as one clean replace block and keeps A's deleted
    // demo doc intact at the END (file_13 × file_14 oracle). Only unanchored
    // pairs (double_spacing × eigenpal) take the mid-stream title fold.
    if let Some(&first) = kids.iter().find(|&&k| dom.name_is(k, &W::p())) {
        let has_ins_text = !dom.descendants(first, Some(&W::ins())).is_empty()
            && dom
                .descendants(first, Some(&W::t()))
                .iter()
                .any(|&t| !dom.value_str(t).trim().is_empty());
        let has_del_text = !dom.descendants(first, Some(&W::del_text())).is_empty();
        let has_live_text = dom.descendants(first, Some(&W::t())).iter().any(|&t| {
            !dom.value_str(t).trim().is_empty()
                && !dom
                    .ancestors_and_self(t, None)
                    .iter()
                    .any(|&a| dom.name_is(a, &W::ins()))
        });
        if has_ins_text && has_del_text && has_live_text {
            return;
        }
    }
    // Find pure-D demo titles that have pure-I (or any) content after them.
    let mut targets: Vec<usize> = Vec::new();
    for (i, &k) in kids.iter().enumerate() {
        if !dom.name_is(k, &W::p()) || !para_is_pure_deleted(dom, k) {
            continue;
        }
        if !para_looks_like_demo_title(dom, k) {
            continue;
        }
        let following = kids[i + 1..].iter().any(|&c| match dom.name(c) {
            Some(n) if n == W::p() => !para_has_no_text(dom, c),
            Some(n) if n == W::tbl() => true,
            _ => false,
        });
        if following {
            targets.push(i);
        }
    }
    for &di in targets.iter().rev() {
        let d = kids[di];
        if dom.parent(d).is_none() {
            continue;
        }
        // Nearest preceding pure-I numbered heading within 10 slots.
        let mut heading: Option<NodeId> = None;
        let start = di.saturating_sub(10);
        for &k in kids[start..di].iter().rev() {
            if dom.name_is(k, &W::tbl()) {
                break;
            }
            if !dom.name_is(k, &W::p()) {
                continue;
            }
            if !para_is_pure_inserted(dom, k) {
                // Only pure inserted paragraphs can supply the heading.
                continue;
            }
            let mut text = String::new();
            for t in dom.descendants(k, Some(&W::t())) {
                text.push_str(&dom.value_str(t));
            }
            let trimmed = text.trim_start();
            if (trimmed.starts_with("1.") || trimmed.starts_with("1 "))
                && trimmed.chars().count() >= 8
                && trimmed.chars().count() <= 80
            {
                heading = Some(k);
                break;
            }
        }
        let Some(h) = heading else {
            continue;
        };
        if dom.parent(h).is_none() {
            continue;
        }
        // A paragraph's mark owns independent run formatting. Never collapse
        // existing mark history or a section break into CT_PPrBase history.
        let old_properties = source_properties
            .get(&d)
            .copied()
            .or_else(|| dom.element(d, &W::p_pr()));
        let new_properties = dom.element(h, &W::p_pr());
        if [old_properties, new_properties]
            .into_iter()
            .flatten()
            .any(|properties| {
                dom.element(properties, &W::sect_pr()).is_some()
                    || dom
                        .element(properties, &W::r_pr())
                        .is_some_and(|mark| dom.element(mark, &W::r_pr_change()).is_some())
            })
        {
            continue;
        }
        fn clean_mark_properties(dom: &mut Dom, properties: Option<NodeId>) -> NodeId {
            let paragraph = dom.new_element(W::p());
            let properties = properties
                .map(|n| dom.clone_subtree(n))
                .unwrap_or_else(|| dom.new_element(W::p_pr()));
            dom.add(paragraph, properties);
            strip_para_revision_marks(dom, paragraph);
            properties
        }
        let old_mark_properties = clean_mark_properties(dom, old_properties);
        let new_mark_properties = clean_mark_properties(dom, new_properties);
        let old_mark = dom.element(old_mark_properties, &W::r_pr());
        let new_mark = dom.element(new_mark_properties, &W::r_pr());
        if !settings.detect_format_changes
            && !paragraph_mark_properties_equal(dom, old_mark, new_mark)
        {
            continue;
        }
        // The mixed carrier keeps B's heading layout live. Rejecting the
        // fold must restore all A title properties, rather than leaving B's
        // border/spacing behind. Word's M465 snapshot is empty; save the full
        // authored layout so the visible fold remains source-recoverable.
        if settings.detect_format_changes {
            let old = old_properties
                .map(|properties| formatchg::project_para_properties_for_change(dom, properties))
                .unwrap_or_else(|| dom.new_element(W::p_pr()));
            let properties = dom.element(h, &W::p_pr()).unwrap_or_else(|| {
                let properties = dom.new_element(W::p_pr());
                dom.add_first(h, properties);
                properties
            });
            if formatchg::normalize_para_properties(dom, old)
                != formatchg::normalize_para_properties(dom, properties)
            {
                for change in dom.elements(properties, Some(&W::p_pr_change())) {
                    dom.remove(change);
                }
                let change = rev_el(dom, W::p_pr_change(), settings, id_gen);
                dom.add(change, old);
                dom.add(properties, change);
            }
        }
        // Start the mark from A when recording the change; the shared helper
        // installs B live and records all A mark properties in rPrChange.
        // With tracking off the two marks compare equal, so B can stay live.
        let mark_source = if settings.detect_format_changes {
            old_mark
        } else {
            new_mark
        };
        let properties = dom.element(h, &W::p_pr()).unwrap_or_else(|| {
            let properties = dom.new_element(W::p_pr());
            dom.add_first(h, properties);
            properties
        });
        if let Some(mark) = dom.element(properties, &W::r_pr()) {
            dom.remove(mark);
        }
        if let Some(mark) = mark_source {
            let mark = dom.clone_subtree(mark);
            if let Some(change) = dom.element(properties, &W::p_pr_change()) {
                dom.add_before_self(change, mark);
            } else {
                dom.add(properties, mark);
            }
        }
        if settings.detect_format_changes {
            closing_mark_takes_revised_mark_formatting(
                dom,
                properties,
                new_mark_properties,
                settings,
                id_gen,
            );
        }
        if dom.elements(properties, None).is_empty() {
            dom.remove(properties);
        }
        // Fold the two revision bodies without deleting their live mark format.
        for c in dom.elements(d, None) {
            if !dom.name_is(c, &W::p_pr()) {
                dom.add(h, c);
            }
        }
        dom.remove(d);
    }
}
