// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.F — markup finalization: turn the `pt:Status`-tagged tree into real
//! `w:ins` / `w:del` / `w:rPrChange` / move markup, conjoin paragraph
//! marks, renumber revision ids, and strip scratch markup. Port of
//! `MarkContentAsDeletedOrInserted` (WmlComparer.cs:2568),
//! `ConjoinDeletedInsertedParagraphMarks` (:2515), `FixUpRevisionIds`
//! (:2769), `IgnorePt14Namespace` (:2912), `RemovePowerToolsScratchMarkup`
//! (`CleanPartTransform` :1165).
//!
//! Public surface: more than a hundred small passes — the ones the
//! `compare_bodies_faithful_with_notes` pipeline runs are the load-bearing
//! ones (e.g. [`mark_content_as_deleted_or_inserted`],
//! [`conjoin_paragraph_marks`], [`fix_up_revision_ids`], [`ignore_pt14_namespace`],
//! [`remove_powertools_scratch_markup`]). The bulk of the file is the
//! Word-mode `M4.F` peel ladder the pipeline runs after
//! `merge_replaced_paragraphs_in`; each peel has its own public entry
//! point so the `--keep-fidelity`/`-pgNumType/` tests can call individual
//! passes.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::namespaces::{M, MC, PT, R, W, W14, WP14};
use crate::xmllinq::{Dom, NodeId, XName, XNamespace};

use super::WmlComparerSettings;
use super::formatchg;
use super::lcs_table;
use super::tables::ALLOWABLE_RUN_CHILDREN;
use super::{PREDELETE_STAMP_ORIG, PREDELETE_STAMP_REV};

mod alignment_peels;
mod free_mesh;
mod inherited_repairs;
mod ladder_rungs;
mod list_spacing_peels;
mod markup_cleanup;
mod merge_replaced;
mod midstream_peels;
mod preexisting_revisions;
mod pure_revision_peels;
mod revision_wrappers;
mod schema_order;
mod validity;

pub use alignment_peels::*;
pub use free_mesh::*;
pub use inherited_repairs::*;
pub use ladder_rungs::*;
pub use list_spacing_peels::*;
pub use markup_cleanup::*;
pub use merge_replaced::*;
pub use midstream_peels::*;
pub use preexisting_revisions::*;
pub use pure_revision_peels::*;
pub use revision_wrappers::*;
pub use schema_order::*;
pub use validity::*;

// Per-paragraph pure-del / mixed classification cache for finalize peels.
// Enabled only inside `with_para_classification_cache` so multi-pass peels
// do not re-walk each paragraph's descendants (large docs: 10k+ paras × N peels).
thread_local! {
    static PURE_DEL_CACHE: RefCell<Option<HashMap<NodeId, bool>>> = const { RefCell::new(None) };
    static MIXED_CACHE: RefCell<Option<HashMap<NodeId, bool>>> = const { RefCell::new(None) };
}

/// Enable the per-paragraph pure-del / mixed classification caches the
/// `M4.F` peels share.
///
/// Peels fill them lazily on first touch (cached key per paragraph); call
/// [`end_para_classification_cache`] after the peels finish so a stale
/// cache does not leak into the next document run. Do **not** eagerly
/// classify every body paragraph — that costs `O(n_p)` pure-del walks
/// and dominated `redline × 5lb` when most paragraphs are never queried.
pub fn begin_para_classification_cache() {
    PURE_DEL_CACHE.with(|c| *c.borrow_mut() = Some(HashMap::new()));
    MIXED_CACHE.with(|c| *c.borrow_mut() = Some(HashMap::new()));
}

/// Drop the thread-local pure-del / mixed paragraph classification
/// caches; called by the pipeline at the end of the peel ladder so the
/// next document run starts with empty caches.
pub fn end_para_classification_cache() {
    PURE_DEL_CACHE.with(|c| *c.borrow_mut() = None);
    MIXED_CACHE.with(|c| *c.borrow_mut() = None);
}

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
                || n == W::del_text()
                || ALLOWABLE_RUN_CHILDREN.contains(&n)
                || n == MC::alternate_content()
                || n == W::pict()
                || n == W::object()
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
                        || n == &W::move_from()
                        || n == &W::move_to() =>
                {
                    continue;
                }
                Some(n) if n == &W::t() => dom.set_name(c, W::del_text()),
                Some(n) if n == &W::instr_text() => dom.set_name(c, W::del_instr_text()),
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

/// `MarkContentAsDeletedOrInsertedTransform` (WmlComparer.cs:2568) — the
/// recursive body of `M4.F.1/F.2/F.3`.
///
/// Walks `node` and rewrites each `w:r` / `w:pPr` / `w:fldSimple` /
/// `w:AlternateContent` whose `pt:Status` says inserted, deleted, moved or
/// format-changed into the corresponding `w:ins` / `w:del` / move range
/// pair / `w:rPrChange` / `w:pPrChange` wrappers. Other elements are
/// rebuilt verbatim. `id_gen` mints fresh revision ids as wrappers are
/// emitted; the caller is responsible for renumbering the final tree via
/// [`fix_up_revision_ids`].
///
/// This is the lower-level hook used by [`mark_content_as_deleted_or_inserted`];
/// callers normally want that wrapper, not this entry point.
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

    // An empty field is a whole atom with no run to carry the status: Word's
    // redline spells it as a complex field inside the revision (b15).
    if name == W::fld_simple() && dom.elements(node, None).is_empty() {
        let status = dom.attribute(node, &PT::status()).map(str::to_string);
        if let Some(moved @ ("MovedSource" | "MovedDestination")) = status.as_deref() {
            // As moved runs: range markers around a moveFrom / moveTo whose
            // code stays w:instrText (moved text stays w:t).
            let (start, kind, end) = if moved == "MovedSource" {
                ("moveFromRangeStart", "moveFrom", W::move_from_range_end())
            } else {
                ("moveToRangeStart", "moveTo", W::move_to_range_end())
            };
            let move_name = dom
                .attribute(node, &PT::name("MoveName"))
                .unwrap_or("move1")
                .to_string();
            let range_id = id_gen.to_string();
            *id_gen += 1;
            let range_start = dom.new_element(W::name(start));
            dom.set_attribute_value(range_start, &W::id(), Some(&range_id));
            dom.set_attribute_value(range_start, &W::name("name"), Some(&move_name));
            dom.set_attribute_value(
                range_start,
                &W::author(),
                Some(&settings.author_for_revisions),
            );
            dom.set_attribute_value(
                range_start,
                &W::date(),
                Some(&settings.date_time_for_revisions),
            );
            let wrap = rev_el(dom, W::name(kind), settings, id_gen);
            for run in complex_field_runs(dom, node, false) {
                dom.add(wrap, run);
            }
            let range_end = dom.new_element(end);
            dom.set_attribute_value(range_end, &W::id(), Some(&range_id));
            return vec![range_start, wrap, range_end];
        }
        let deleted = match status.as_deref() {
            Some("Deleted") => true,
            Some("Inserted") => false,
            _ => return vec![dom.clone_subtree(node)],
        };
        let wrap = rev_el(
            dom,
            if deleted { W::del() } else { W::ins() },
            settings,
            id_gen,
        );
        for run in complex_field_runs(dom, node, deleted) {
            dom.add(wrap, run);
        }
        return vec![wrap];
    }

    if name == W::r() {
        let txbx = W::txbx_content();
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
                        W::move_from_range_start(),
                        W::move_from(),
                        W::move_from_range_end(),
                    )
                } else {
                    (
                        W::move_to_range_start(),
                        W::move_to(),
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
                let chg = dom.new_element(W::r_pr_change());
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
        // A spacing B cleared stays out of the live pPr: Word's redline of
        // file_69 × file_70 (ac6cd8d92f) keeps after=20 live only on the
        // deleted drawing residual, and the title's pPrChange alone; copying
        // it live left it on the accepted title.
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
                if dom.element(rpr, &W::r_pr_change()).is_none() {
                    let old_rpr = parse_rpr(dom, Some(&old_rpr_s));
                    let chg = dom.new_element(W::r_pr_change());
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
                let chg = dom.new_element(W::p_pr_change());
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
            return drop_pt_bookkeeping(dom, root);
        }
    }
    dom.new_element(W::r_pr())
}

/// Clone a parsed old-properties element without the scratch `pt14:*`
/// attributes it was serialized with, nor the `xmlns` declarations the
/// standalone serialization bound (`pt14`, and `w` on a fragment whose
/// parent document already declares it). They would otherwise survive
/// into the redline and Reject All would not match the source mark.
fn drop_pt_bookkeeping(dom: &mut Dom, root: NodeId) -> NodeId {
    let out = dom.clone_subtree(root);
    for el in dom.descendants_and_self(out, None) {
        let drop: Vec<XName> = (0..dom.attr_count(el))
            .map(|i| dom.attr_at(el, i))
            .filter(|(n, v)| {
                n.namespace_name() == PT::URI
                    || (dom.is_namespace_declaration(n) && (*v == PT::URI || *v == W::URI))
            })
            .map(|(n, _)| n.clone())
            .collect();
        for n in drop {
            dom.set_attribute_value(el, &n, None);
        }
    }
    out
}

/// Parse an `OldPPr` attribute string into a `w:pPr` element (empty on failure).
fn parse_ppr(dom: &mut Dom, s: Option<&str>) -> NodeId {
    if let Some(s) = s {
        let doc = dom.parse_xdocument(s);
        if let Some(root) = dom.root(doc) {
            return drop_pt_bookkeeping(dom, root);
        }
    }
    dom.new_element(W::p_pr())
}

/// M4.F — run [`mark_content_transform`] over `root` and return the new
/// root.
///
/// Each `w:r` / `w:pPr` with a `pt:Status` is wrapped in the matching
/// `w:ins` / `w:del` / `w:moveFrom` / `w:moveTo` / `w:rPrChange` /
/// `w:pPrChange` carrier. Revision ids are minted from `id_gen`; renumber
/// the whole tree through [`fix_up_revision_ids`] afterwards so ids are
/// sequential `1..n` and disjoint from any comment ids.
pub fn mark_content_as_deleted_or_inserted(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
    id_gen: &mut u32,
) -> NodeId {
    let v = mark_content_transform(dom, root, settings, id_gen);
    let root = v
        .into_iter()
        .next()
        .expect("root transform yields one node");
    join_move_ranges(dom, root);
    root
}

/// Marking emits a triple per run (and empty field), but a move names one
/// range on each side. Word's Reject All duplicates text when the run splits
/// give the sides different range counts (#1214). Keep every move wrapper and
/// intervening endpoint in place; retain only the outer range boundaries.
fn join_move_ranges(dom: &mut Dom, root: NodeId) {
    for side in ["moveFrom", "moveTo"] {
        let starts = dom.descendants(root, Some(&W::name(&format!("{side}RangeStart"))));
        if starts.len() < 2 {
            continue;
        }
        let ends: HashMap<String, NodeId> = dom
            .descendants(root, Some(&W::name(&format!("{side}RangeEnd"))))
            .into_iter()
            .filter_map(|end| dom.attribute(end, &W::id()).map(|id| (id.to_string(), end)))
            .collect();
        let mut ranges: HashMap<String, (NodeId, NodeId)> = HashMap::new();
        for start in starts {
            let Some(name) = dom.attribute(start, &W::name("name")).map(str::to_string) else {
                continue;
            };
            let Some(end) = dom
                .attribute(start, &W::id())
                .and_then(|id| ends.get(id))
                .copied()
            else {
                continue;
            };
            if let Some((first, last)) = ranges.get_mut(&name) {
                let id = dom.attribute(*first, &W::id()).unwrap().to_string();
                dom.remove(start);
                dom.remove(*last);
                dom.set_attribute_value(end, &W::id(), Some(&id));
                *last = end;
            } else {
                ranges.insert(name, (start, end));
            }
        }
    }
}

/// True when pPr carries structural body props (pStyle/numPr/spacing/jc/ind/…).
/// Ignores rPr / sectPr / pPrChange / pt: attrs (not layout structure).
fn ppr_has_structural_props(dom: &Dom, ppr: NodeId) -> bool {
    for c in dom.elements(ppr, None) {
        let Some(n) = dom.name(c) else {
            continue;
        };
        if n == W::r_pr()
            || n == W::sect_pr()
            || n == W::p_pr_change()
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
            || n == W::sect_pr()
            || n == W::p_pr_change()
            || n.namespace_name() == PT::URI
        {
            continue;
        }
        if n == W::jc_el() {
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
            && dom.element(ppr, &W::p_pr_change()).is_none()
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
            let chg = dom.new_element(W::p_pr_change());
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

/// M4.F.4 — apply the Conjoin transform to `root`.
///
/// A `w:p` carrying two or more `w:pPr` (Inserted + Deleted variants) is
/// collapsed to one clean `pPr` plus the live runs; the dropped `pPr` is
/// either recorded in `w:pPrChange` (M78) or kept live as a delete mark
/// (M81b), per the rule baked into the `crate::comparer::conjoin_transform`
/// helper. Pass the comparer `author_for_revisions` and
/// `date_time_for_revisions` via `settings`.
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

/// M4.F.5 — `FixUpRevisionIds` (WmlComparer.cs:2769): renumber every
/// `w:id` of every revision element under each `root` from `1`.
///
/// `w:moveFromRangeStart` / `w:moveToRangeStart` allocate a fresh id;
/// their matching `*RangeEnd` shares the same id. Comment ids
/// (`commentRangeStart` / `commentRangeEnd` / `commentReference`) are
/// reserved so revision renumbering never reuses them — Word keeps them
/// disjoint. Re-run this after any later pass that mints ids
/// (`convert_stamped_*`, `wrap_bare_del_text_runs`, `mark_fully_revised_rows`,
/// etc.); the pipeline runs it twice for that reason.
pub fn fix_up_revision_ids(dom: &mut Dom, roots: &[NodeId]) {
    // Every tracked-change / move / property-change carrier that uses w:id.
    // Missing any of these lets renumbered move ranges collide with e.g.
    // tblPrChange ids — Word then raises "unreadable content".
    let rev_names = [
        W::ins(),
        W::del(),
        W::move_from(),
        W::move_to(),
        W::move_from_range_start(),
        W::move_from_range_end(),
        W::move_to_range_start(),
        W::move_to_range_end(),
        W::r_pr_change(),
        W::p_pr_change(),
        W::name("tblPrChange"),
        W::name("tblGridChange"),
        W::name("trPrChange"),
        W::tc_pr_change(),
        W::name("sectPrChange"),
        W::numbering_change(),
        W::cell_ins(),
        W::cell_del(),
        W::cell_merge(),
    ];
    // Comment anchors keep their own ids (must stay aligned with comments.xml).
    // Reserved so revision renumber never reuses them.
    let reserved_names = [
        W::comment_range_start(),
        W::comment_range_end(),
        W::comment_reference(),
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
        W::move_from_range_start(),
        W::move_to_range_start(),
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

/// M4.F.7 — `IgnorePt14Namespace` (WmlComparer.cs:2912): declare
/// `xmlns:pt14` on `root` and add `pt14` to `mc:Ignorable` (idempotent).
///
/// Word treats the PowerTools scratch namespace as ignorable; this keeps
/// the redline's root in the same shape Word expects. Safe to call
/// multiple times.
pub fn ignore_pt14_namespace(dom: &mut Dom, root: NodeId) {
    let pt14 = XNamespace::xmlns().name("pt14");
    if dom.attribute(root, &pt14).is_none() {
        dom.set_attribute_value(root, &pt14, Some(PT::URI));
    }
    let ignorable = MC::ignorable();
    let cur = dom.attribute(root, &ignorable).unwrap_or("").to_string();
    let mut toks: Vec<&str> = cur.split_whitespace().collect();
    if !toks.contains(&"pt14") {
        toks.push("pt14");
        dom.set_attribute_value(root, &ignorable, Some(&toks.join(" ")));
    }
}

/// Word's ignorable extension namespaces, with the prefix Word writes for
/// each, in the order Word lists them in `mc:Ignorable`.
const WORD_IGNORABLE_NAMESPACES: [(&str, &str); 10] = [
    (
        "w14",
        "http://schemas.microsoft.com/office/word/2010/wordml",
    ),
    (
        "w15",
        "http://schemas.microsoft.com/office/word/2012/wordml",
    ),
    (
        "w16se",
        "http://schemas.microsoft.com/office/word/2015/wordml/symex",
    ),
    (
        "w16cid",
        "http://schemas.microsoft.com/office/word/2016/wordml/cid",
    ),
    (
        "w16",
        "http://schemas.microsoft.com/office/word/2018/wordml",
    ),
    (
        "w16cex",
        "http://schemas.microsoft.com/office/word/2018/wordml/cex",
    ),
    (
        "w16sdtdh",
        "http://schemas.microsoft.com/office/word/2020/wordml/sdtdatahash",
    ),
    (
        "w16sdtfl",
        "http://schemas.microsoft.com/office/word/2024/wordml/sdtformatlock",
    ),
    (
        "w16du",
        "http://schemas.microsoft.com/office/word/2023/wordml/word16du",
    ),
    (
        "wp14",
        "http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing",
    ),
];

/// Elements merged in from the other document (B's numbering definitions,
/// styles, notes) keep their Word 2010+ extension attributes but not the
/// source root's `mc:Ignorable` contract, so the part claims markup it never
/// declared ignorable (`w15:restartNumberingAfterBreak`, `w16cid:durableId`
/// under a Mac-era numbering root). Every Word extension namespace used in the
/// part gets its prefix bound on the root and listed in `mc:Ignorable`, the
/// way Word writes its own parts. Only Word's own list qualifies: namespaces
/// such as `wps` live inside `mc:Choice` and must never be ignorable.
/// Make every Word-2010+ extension namespace that the part uses
/// ignorable on `root`.
///
/// Elements merged in from the other document (B's numbering
/// definitions, styles, notes) keep their Word 2010+ extension
/// attributes but not the source root's `mc:Ignorable` contract, so the
/// part claims markup it never declared ignorable
/// (`w15:restartNumberingAfterBreak`, `w16cid:durableId`, …). For each
/// Word extension namespace the part actually uses, this binds its
/// prefix on the root and lists it in `mc:Ignorable`, the way Word writes
/// its own parts. Only Word's own namespace list qualifies; namespaces
/// such as `wps` (gated by an MC prefix list inside `mc:Choice`) must
/// never be marked ignorable.
pub fn declare_extension_namespaces_ignorable(dom: &mut Dom, root: NodeId) {
    let xmlns = XNamespace::xmlns();
    let mut used = [false; WORD_IGNORABLE_NAMESPACES.len()];
    for el in dom.descendants_and_self(root, None) {
        let mark = |used: &mut [bool], uri: &str| {
            if let Some(i) = WORD_IGNORABLE_NAMESPACES
                .iter()
                .position(|(_, u)| *u == uri)
            {
                used[i] = true;
            }
        };
        if let Some(n) = dom.name(el) {
            mark(&mut used, n.namespace_name());
        }
        for (a, _) in dom.attributes_iter(el) {
            if !dom.is_namespace_declaration(a) {
                mark(&mut used, a.namespace_name());
            }
        }
    }
    if !used.contains(&true) {
        return;
    }
    let ignorable = MC::ignorable();
    let mut tokens: Vec<String> = dom
        .attribute(root, &ignorable)
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_string)
        .collect();
    let before = tokens.len();
    for (i, (prefix, uri)) in WORD_IGNORABLE_NAMESPACES.iter().enumerate() {
        if !used[i] {
            continue;
        }
        let bound = dom.attribute(root, &xmlns.name(prefix)).map(str::to_string);
        match bound.as_deref() {
            Some(u) if u == *uri => {}
            // The prefix means something else in this part: leave it alone.
            Some(_) => continue,
            None => dom.set_attribute_value(root, &xmlns.name(prefix), Some(uri)),
        }
        if !tokens.iter().any(|t| t == prefix) {
            tokens.push((*prefix).to_string());
        }
    }
    if tokens.len() != before {
        let mc = xmlns.name("mc");
        if dom.attribute(root, &mc).is_none()
            && !dom
                .attributes(root)
                .iter()
                .any(|(a, v)| dom.is_namespace_declaration(a) && v == MC::URI)
        {
            dom.set_attribute_value(root, &mc, Some(MC::URI));
        }
        dom.set_attribute_value(root, &ignorable, Some(&tokens.join(" ")));
    }
}

/// Binds every prefix an MC prefix list names (`mc:Choice/@Requires`,
/// `mc:Ignorable`, `mc:ProcessContent`, …) where the list names it. Content
/// grafted from the other document's part leaves that part's root
/// declarations behind: a B header's connector under `<mc:Choice
/// Requires="wps">` landed in A's header, whose root never bound `wps`, and
/// the serializer declared it only on the shape inside the Choice. Word then
/// offers to repair the file (redlines_en_500, baf3657261 vs 8e835e4a58). The
/// binding goes on the part root, where Word writes it; a prefix the root
/// already binds is in scope everywhere, so none is overwritten. A prefix
/// with no conventional namespace is left alone.
/// Bind every prefix a Markup-Compatibility prefix-list
/// (`mc:Choice/@Requires`, `mc:Ignorable`, `mc:ProcessContent`, …) names
/// but `root` has not bound.
///
/// Content grafted from the other document's part leaves that part's
/// root declarations behind — a B header's connector under
/// `<mc:Choice Requires="wps">` lands in A's header, whose root never
/// bound `wps`, and the serializer declares it only on the shape inside
/// the Choice. Word then offers to repair the file
/// (`redlines_en_500`, `baf3657261` vs `8e835e4a58`). Bind the prefix on
/// the part root, where Word writes it; a prefix the root already binds
/// is in scope everywhere, so nothing is overwritten. Prefixes with no
/// conventional namespace are left alone.
pub fn bind_compatibility_prefixes(dom: &mut Dom, root: NodeId) {
    use crate::xmllinq::serialize::{is_namespace_prefix_list, well_known_namespace};
    let xmlns = XNamespace::xmlns();
    for el in dom.descendants_and_self(root, None) {
        let Some(element) = dom.name(el) else {
            continue;
        };
        let lists: Vec<String> = dom
            .attributes_iter(el)
            .filter(|(name, _)| is_namespace_prefix_list(&element, name))
            .map(|(_, value)| value.to_string())
            .collect();
        for list in lists {
            for token in list.split_whitespace() {
                let prefix = token.split_once(':').map_or(token, |(prefix, _)| prefix);
                if prefix.is_empty() || prefix == "xml" || prefix_in_scope(dom, el, prefix) {
                    continue;
                }
                let Some(uri) = well_known_namespace(prefix) else {
                    continue;
                };
                dom.set_attribute_value(root, &xmlns.name(prefix), Some(uri));
            }
        }
    }
}

/// Strip every `pt:*` attribute across `root` and its descendants.
///
/// Port of `RemovePowerToolsScratchMarkup` (`CleanPartTransform` at
/// WmlComparer.cs:1165). Call this **after** every stamping pass that
/// uses `pt:*` for its bookkeeping (the pre-diff flattening, the
/// `convert_stamped_predeletes` / `convert_stamped_preins` passes), so the
/// redline serialized into `Out` carries no scratch marks. Idempotent.
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

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/word_lcs_tests.rs"]
mod word_lcs_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_tests.rs"]
mod coverage_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_round_two_tests.rs"]
mod coverage_round_two_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_round_three_tests.rs"]
mod coverage_round_three_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_round_next_tests.rs"]
mod coverage_round_next_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/fragmented_label_regression_tests.rs"]
mod fragmented_label_regression_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/deep_production_path_tests.rs"]
mod deep_production_path_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/row_move_ownership_regressions.rs"]
mod row_move_ownership_regressions;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/parked_paragraph_history_regressions.rs"]
mod parked_paragraph_history_regressions;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/closing_layout_history_regressions.rs"]
mod closing_layout_history_regressions;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/row_nontext_ownership_regressions.rs"]
mod row_nontext_ownership_regressions;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/split_paragraph_boundary_regressions.rs"]
mod split_paragraph_boundary_regressions;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_final_batch_finalize_tests.rs"]
mod coverage_final_batch_finalize_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/coverage_real_revision_merge_matrix_tests.rs"]
mod coverage_real_revision_merge_matrix_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/short_label_format_history_tests.rs"]
mod short_label_format_history_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/consecutive_block_splice_tests.rs"]
mod consecutive_block_splice_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/demo_title_pilcrow_history_tests.rs"]
mod demo_title_pilcrow_history_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/paired_row_mark_ownership_tests.rs"]
mod paired_row_mark_ownership_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/closing_authored_section_boundary_tests.rs"]
mod closing_authored_section_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/finalizer_distinct_boundary_contract_tests.rs"]
mod finalizer_distinct_boundary_contract_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/finalizer_remaining_source_boundary_tests.rs"]
mod finalizer_remaining_source_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/finalizer_merge_source_guard_tests.rs"]
mod finalizer_merge_source_guard_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/finalizer_reached_predicate_boundary_tests.rs"]
mod finalizer_reached_predicate_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/public_body_heuristic_story_scope_tests.rs"]
mod public_body_heuristic_story_scope_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/default_source_property_carrier_tests.rs"]
mod default_source_property_carrier_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/existing_mark_layout_ownership_tests.rs"]
mod existing_mark_layout_ownership_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/interleave_terminal_source_ownership_tests.rs"]
mod interleave_terminal_source_ownership_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/whitespace_source_mark_carrier_tests.rs"]
mod whitespace_source_mark_carrier_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/fragmented_source_label_boundary_tests.rs"]
mod fragmented_source_label_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/nested_deleted_text_source_ownership_tests.rs"]
mod nested_deleted_text_source_ownership_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/ladder_suffix_source_boundary_tests.rs"]
mod ladder_suffix_source_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/reached_word_document_policy_tests.rs"]
mod reached_word_document_policy_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/mid_list_optional_source_property_tests.rs"]
mod mid_list_optional_source_property_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/live_opaque_row_source_boundary_tests.rs"]
mod live_opaque_row_source_boundary_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/deleted_carrier_echo_source_owner_tests.rs"]
mod deleted_carrier_echo_source_owner_tests;

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
#[path = "tests/redline_pass_sweep_tests.rs"]
mod redline_pass_sweep_tests;
