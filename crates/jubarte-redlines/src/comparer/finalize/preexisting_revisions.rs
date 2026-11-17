// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Pre-existing tracked revisions: flatten and convert stamped insertions and deletions, plus the passes beside them.

use super::*;

/// Word-alignment mode: drop the embedded `pPr/sectPr` from a paragraph
/// whose paragraph mark is `w:del`-marked.
///
/// Word renders the deleted paragraph on the same page; a stray
/// section break would produce a whole blank page
/// (`mcdoc_meeting-agenda-table-2`). Genuine authored mid-section
/// breaks survive deletion — Word keeps the deleted content's
/// pagination across pages; those signatures come from
/// [`drop_hoisted_sectpr_artifacts`] which collected them earlier.
pub fn drop_sectpr_from_deleted_marks(
    dom: &mut Dom,
    root: NodeId,
    genuine: &std::collections::HashSet<String>,
) {
    let sectpr = W::sect_pr();
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

/// Word-alignment mode — flatten doc A's PRE-EXISTING tracked
/// INSERTIONS with a stamp.
///
/// Word carries them as pending `w:ins` (original author / date) and
/// they SURVIVE accepting the redline, exactly like accepting Word's
/// own output (forensics #9/#10). The stamped runs re-emerge as
/// pending ins via [`convert_stamped_preins`]. This is the DISCLOSED
/// word-mode accept-contract alignment: `accept(redline)` keeps this
/// text like Word does; the faithful preset keeps accept-first.
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
                if dom.name_is(k, &W::r()) {
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
                .any(|&a| a != i && dom.name_is(a, &W::del()));
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
/// Word-alignment mode — re-emit the runs flattened by
/// [`flatten_tracked_insertions_stamped`] back as pending `w:ins`.
///
/// A run stamped `pt:PreIns` (from B) renames its text kind, splits on
/// revision shape, and is wrapped in `w:ins`; `delText` reverts to `w:t`
/// and an existing `Equal` run gets a fresh `w:ins` of its own. Runs
/// before [`coalesce_adjacent_revisions`] so the wrappers still hold
/// one run each.
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
            for t in dom.descendants(run, Some(&W::del_text())) {
                dom.set_name(t, W::t());
            }
            for t in dom.descendants(run, Some(&W::del_instr_text())) {
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
                    dom.copy_attributes(parent, del2);
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
/// Word-alignment mode — make sure the rebuilt `<w:document>` carries
/// at least the `pgSz` Word expects when the document is missing one.
///
/// Otherwise renderers fall back to their own default (A4 in
/// LibreOffice) and paginate differently from Word's redline.
pub fn ensure_default_page_size(dom: &mut Dom, root: NodeId) {
    let Some(body) = dom.element(root, &W::body()) else {
        return;
    };
    let sectpr_name = W::sect_pr();
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
        dom.set_attribute_value(pg, &W::w(), Some("12240"));
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
/// Wrap bare `w:delText` runs (text-box content nested inside a deleted
/// drawing) inside a `w:del` like Word does.
///
/// Schema-legal but validators are blind to it — bisected on the
/// strict01 cover page. Wrapping in `w:del` is the right fix; a `w:t`
/// inside `w:moveFrom` (Word's own emitted shape) is what triggers the
/// "unreadable content" repair dialog, so this pass only acts on
/// `delText` runs not already inside a `w:del`. Mints a fresh `w:id`
/// per wrapped run from `id_gen`.
pub fn wrap_bare_del_text_runs(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let del_text = W::del_text();
    let move_from = W::move_from();
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
            n != Some(W::del()) && n.as_ref() != Some(&move_from)
        })
        .collect();
    for r in runs {
        let d = rev_el(dom, W::del(), settings, id_gen);
        dom.add_before_self(r, d);
        dom.remove(r);
        dom.add(d, r);
    }
    delete_stories_of_deleted_text_boxes(dom, root, settings, id_gen);
}

/// A text box's `w:txbxContent` is a story of its own: the `w:del` around its
/// anchor run does not reach into it. When the anchor is deleted, Word's
/// Compare deletes the story too — every run in a `w:del` and every paragraph
/// mark deleted (en bb113e88's FILENAME text box). Runs the diff left without
/// a status (field chars, a field code: they carry no text) stayed bare, and
/// the deleted-run rename still turned their `w:instrText` into
/// `w:delInstrText`. Word refused those files (en 30ff840c, bb113e88); the
/// OpenXmlValidator passes them. Adjacent bare runs share one `w:del`, as in
/// Word's output.
pub(super) fn delete_stories_of_deleted_text_boxes(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let txbx = W::txbx_content();
    let revision = [W::ins(), W::del(), W::move_from(), W::move_to()];
    // Nearest enclosing text box of `n`, or None when a revision wrapper comes
    // first (the run is already covered) or there is no text box.
    let bare_in = |dom: &Dom, n: NodeId| -> Option<NodeId> {
        let mut cur = dom.parent(n);
        while let Some(p) = cur {
            match dom.name(p) {
                Some(nm) if nm == txbx => return Some(p),
                Some(nm) if revision.contains(&nm) => return None,
                _ => cur = dom.parent(p),
            }
        }
        None
    };
    // The deletion wrapping a text box's anchor in the enclosing story.
    let anchor_del = |dom: &Dom, tb: NodeId| -> Option<NodeId> {
        let mut cur = dom.parent(tb);
        while let Some(p) = cur {
            match dom.name(p) {
                Some(nm) if nm == txbx => return None,
                Some(nm) if nm == W::del() => return Some(p),
                _ => cur = dom.parent(p),
            }
        }
        None
    };
    if dom.descendants(root, Some(&txbx)).is_empty() {
        return;
    }
    // Document order: an outer text box is deleted before a nested one's
    // anchor is looked up, so nested deleted boxes are found too.
    for r in dom.descendants(root, Some(&W::r())) {
        let Some(tb) = bare_in(dom, r) else {
            continue;
        };
        let Some(outer) = anchor_del(dom, tb) else {
            continue;
        };
        let prev = dom
            .nodes_before_self(r)
            .into_iter()
            .rev()
            .find(|&n| dom.is_element(n));
        let d = match prev {
            // Joins the w:del this pass opened for the preceding bare run.
            Some(p)
                if dom.name_is(p, &W::del())
                    && dom.attribute(p, &PT::name("TxbxDel")).is_some() =>
            {
                p
            }
            _ => {
                let d = rev_el(dom, W::del(), settings, id_gen);
                for a in [W::author(), W::date()] {
                    if let Some(v) = dom.attribute(outer, &a).map(str::to_string) {
                        dom.set_attribute_value(d, &a, Some(&v));
                    }
                }
                dom.set_attribute_value(d, &PT::name("TxbxDel"), Some("1"));
                dom.add_before_self(r, d);
                d
            }
        };
        dom.remove(r);
        dom.add(d, r);
    }
    for d in dom.descendants(root, Some(&W::del())) {
        dom.set_attribute_value(d, &PT::name("TxbxDel"), None);
    }
    for tb in dom.descendants(root, Some(&txbx)) {
        let Some(outer) = anchor_del(dom, tb) else {
            continue;
        };
        for p in dom.descendants(tb, Some(&W::p())) {
            if dom.ancestors(p, Some(&txbx)).first() != Some(&tb) {
                continue; // a nested text box's paragraph: its own anchor decides
            }
            mark_paragraph_mark_deleted(dom, p, outer, settings, id_gen);
        }
    }
}

/// Give `p`'s paragraph mark a `w:del` (attribution copied from `like`) unless
/// the mark already carries a revision. The revision leads `CT_ParaRPr`.
pub(super) fn mark_paragraph_mark_deleted(
    dom: &mut Dom,
    p: NodeId,
    like: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) {
    let ppr = match dom.element(p, &W::p_pr()) {
        Some(x) => x,
        None => {
            let x = dom.new_element(W::p_pr());
            dom.add_first(p, x);
            x
        }
    };
    let rpr = match dom.element(ppr, &W::r_pr()) {
        Some(x) => x,
        None => {
            let x = dom.new_element(W::r_pr());
            // CT_PPr: rPr follows every paragraph property but sectPr/pPrChange.
            match dom
                .elements(ppr, None)
                .into_iter()
                .find(|&c| dom.name_is(c, &W::sect_pr()) || dom.name_is(c, &W::p_pr_change()))
            {
                Some(tail) => dom.add_before_self(tail, x),
                None => dom.add(ppr, x),
            }
            x
        }
    };
    let revision = [W::ins(), W::del(), W::move_from(), W::move_to()];
    if revision.iter().any(|r| dom.element(rpr, r).is_some()) {
        return;
    }
    let d = rev_el(dom, W::del(), settings, id_gen);
    for a in [W::author(), W::date()] {
        if let Some(v) = dom.attribute(like, &a).map(str::to_string) {
            dom.set_attribute_value(d, &a, Some(&v));
        }
    }
    dom.add_first(rpr, d);
}

/// Which input [`flatten_tracked_deletions`] is running against — the two
/// sides carry DIFFERENT contracts:
///
/// - `Original` (`body1`): `accept(redline) ≡ B`. Everything may be
///   flattened, including deleted paragraph marks (struck history stays
///   visible like Word's).
/// - `Revised` (`body2`): `accept(redline) ≡ accept(B)`. Only deletions
///   the stamp / convert round-trip can re-emit as pending `w:del` may
///   be flattened. Complex-content deletions (`w:del > w:hyperlink/...`)
///   and paragraph-mark deletions are left intact for the pre-diff
///   accept to consume — flattening them resurfaced the text as
///   insertions or unmerged paragraphs (m32 w15b/w15c).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FlattenSide {
    /// Base / original document (`body1`).
    Original,
    /// Revised document (`body2`).
    Revised,
}

/// Collect the concatenated visible text of every `w:del` wrapper under
/// `body` (both `w:delText` and `w:delInstrText`).
///
/// Used by the `S1` salt gate: an ORIGINAL-side pending deletion whose
/// text doc B also holds as a pending deletion must keep correlating
/// `Equal` (both carry the same revision; Word keeps it once — the
/// `sample-document` iter2 pair scores −38.75 when salted). Only A-only
/// pre-dels get the salt (fresh revision p4: B has the text LIVE, Word
/// emits the struck history + live copy).
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
pub(super) fn pending_deletion_fingerprint(dom: &Dom, del: NodeId) -> String {
    let del_text = W::del_text();
    let del_instr = W::del_instr_text();
    dom.descendants(del, None)
        .into_iter()
        .filter(|&n| {
            dom.name(n)
                .is_some_and(|nm| nm == del_text || nm == del_instr)
        })
        .map(|t| dom.value(t))
        .collect()
}

/// Word-alignment mode: flatten pre-existing tracked deletions before
/// diffing — unwrap `w:del`, convert `w:delText`/`w:delInstrText` back to
/// `w:t`/`w:instrText` — so the old text re-enters the diff and comes out
/// marked deleted: VISIBLE (struck through) like Word's redline, instead of
/// vanishing via accept-before-diff.
///
/// Every unwrapped run is stamped `pt:PreDelete` (+ original author/date) so
/// [`convert_stamped_predeletes`] can restore attribution and, on the
/// Revised side, re-emit the span as a pending deletion.
/// Word-alignment mode — flatten pre-existing tracked deletions
/// before diffing.
///
/// Unwraps `w:del` and converts `w:delText` / `w:delInstrText` back to
/// `w:t` / `w:instrText`, so the old text re-enters the diff and comes
/// out marked deleted (struck through) like Word's redline, instead of
/// vanishing via accept-before-diff. Every unwrapped run is stamped
/// `pt:PreDelete` (plus the original author / date) so
/// [`convert_stamped_predeletes`] can restore attribution and, on the
/// REVISED side, re-emit the span as a pending `w:del`. `side`
/// selects between the Original (everything flattenable) and the
/// Revised (only stamp / convertible) contract; `other_side_pending`
/// is the corpus lookup against [`pending_deletion_texts`] that
/// gates the S1 salt for A-only pre-dels.
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
        if dom.parent(d).is_some_and(|p| dom.name_is(p, &rpr_name)) {
            continue;
        }
        let kids = dom.nodes(d);
        if side == FlattenSide::Revised
            && kids
                .iter()
                .any(|&k| dom.is_element(k) && !dom.name_is(k, &W::r()))
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
        for t in dom.descendants(d, Some(&W::del_text())) {
            dom.set_name(t, W::t());
        }
        for t in dom.descendants(d, Some(&W::del_instr_text())) {
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
            if dom.name_is(k, &W::r()) {
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
/// Word-alignment mode — re-emit the runs flattened by
/// [`flatten_tracked_deletions`] back as pending `w:del`.
///
/// The stamped `pt:PreDelete` runs split on author / date and rewrap
/// in a fresh `w:del`. Pure-equal runs (no other revision inside them)
/// keep their equality when the carrier was an Insertion; corpora
/// like missing_sectpr × missing_sectpr depend on it. Must run before
/// [`coalesce_adjacent_revisions`] (the wrappers still hold one run
/// each) so a `accept(redline) ≡ accept(B)` holds for the round trip.
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
                    dom.copy_attributes(parent, ins2);
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
            dom.set_name(t, W::del_text());
        }
        for t in dom.descendants(r, Some(&W::instr_text())) {
            dom.set_name(t, W::del_instr_text());
        }
    }
}
