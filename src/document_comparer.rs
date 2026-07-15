//! DocumentComparer façade (M5). Port of `DocumentComparer.ts` (compare path).
//!
//! `compare_documents(original, modified, author) -> Vec<u8>` opens both
//! packages, diffs their main-document bodies, and writes a redline `.docx` (the
//! original package with `word/document.xml` replaced by the tracked-revision
//! result).
//!
//! NOTE: this is the text-level compare path. Full DocumentComparer additionally
//! runs PreProcessMarkup/accept/hash and relocates footnotes/comments/related
//! parts (the M4.5–M4.6 refinements) for exact golden parity on complex docs.

use crate::comparer::{WmlComparerSettings, compare_bodies_faithful};
use crate::namespaces::{R, W};
use crate::opc::{OpcError, PartFs};
use crate::xmllinq::{Dom, NodeId};

/// The header/footer parts a document references, as (kind, type, part-name):
/// kind ∈ {"header","footer"}, type ∈ {"default","even","first"}. Read from the
/// `headerReference`/`footerReference` elements in the main document, resolved to
/// part names via the document rels.
fn header_footer_refs(pkg: &PartFs) -> Vec<(String, String, String)> {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(xml) = pkg.part_string(&main) else {
        return Vec::new();
    };
    let Some(rels) = pkg.read_rels_for(&main) else {
        return Vec::new();
    };
    let id_to_target: std::collections::HashMap<&str, &str> = rels
        .items
        .iter()
        .map(|r| (r.id.as_str(), r.target.as_str()))
        .collect();
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let Some(root) = d.root(doc) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (ref_name, kind) in [
        (W::name("headerReference"), "header"),
        (W::name("footerReference"), "footer"),
    ] {
        for r in d.descendants(root, Some(&ref_name)) {
            let ty = d
                .attribute(r, &W::name("type"))
                .unwrap_or("default")
                .to_string();
            if let Some(rid) = d.attribute(r, &R::name("id"))
                && let Some(&tgt) = id_to_target.get(rid)
            {
                let part = if tgt.starts_with("word/") {
                    tgt.to_string()
                } else {
                    format!("word/{}", tgt.trim_start_matches('/'))
                };
                out.push((kind.to_string(), ty, part));
            }
        }
    }
    out
}

/// Default pinned revision date when the caller doesn't specify one.
pub const DEFAULT_DATE: &str = "1970-01-01T00:00:00Z";

/// The `w:style[@w:type='paragraph' @w:styleId='Normal']` element under a
/// styles root, falling back to the `w:default="1"` paragraph style.
fn find_normal_style(dom: &Dom, styles_root: NodeId) -> Option<NodeId> {
    let styles: Vec<NodeId> = dom
        .elements(styles_root, Some(&W::name("style")))
        .into_iter()
        .filter(|&s| dom.attribute(s, &W::name("type")) == Some("paragraph"))
        .collect();
    styles
        .iter()
        .copied()
        .find(|&s| dom.attribute(s, &W::name("styleId")) == Some("Normal"))
        .or_else(|| {
            styles
                .into_iter()
                .find(|&s| dom.attribute(s, &W::name("default")) == Some("1"))
        })
}

/// A style's `pPr/spacing` as (after, line, lineRule), each None when absent.
fn normal_spacing(dom: &Dom, style: NodeId) -> Option<(String, String, String)> {
    let ppr = dom.element(style, &W::name("pPr"))?;
    let sp = dom.element(ppr, &W::name("spacing"))?;
    let get = |n: &str| dom.attribute(sp, &W::name(n)).unwrap_or("").to_string();
    Some((get("after"), get("line"), get("lineRule")))
}

/// The stylesheet's `docDefaults/pPrDefault/pPr/spacing` as
/// (after, line, lineRule), each "" when absent. Used only when B's Normal
/// style has no stored spacing *and* A had stored spacing to rewrite (Word
/// promotes B's docDefaults into Normal in that case — file_197_file_198).
fn docdefaults_ppr_spacing(dom: &Dom, styles_root: NodeId) -> Option<(String, String, String)> {
    let dd = dom.element(styles_root, &W::name("docDefaults"))?;
    let pd = dom.element(dd, &W::name("pPrDefault"))?;
    let ppr = dom.element(pd, &W::name("pPr"))?;
    let sp = dom.element(ppr, &W::name("spacing"))?;
    let get = |n: &str| dom.attribute(sp, &W::name(n)).unwrap_or("").to_string();
    Some((get("after"), get("line"), get("lineRule")))
}

/// Word's factory-default Normal spacing when B is empty and A had *stored*
/// Normal spacing to rewrite (GT sd-2517_sectpr-headerref / m37).
const FACTORY_NORMAL_SPACING: (&str, &str, &str) = ("160", "278", "auto");

/// When both Normals store no spacing but A still has an effective cascade
/// (docDefaults and/or a non-spacing Normal pPr), Word rewrites Normal to
/// single-line after=0 line=240 (file_77_file_78, file_33_file_34). Distinct
/// from [`FACTORY_NORMAL_SPACING`] — using 160/278 here leaves LO at +2 pages.
const EMPTY_B_SINGLE_LINE_NORMAL: (&str, &str, &str) = ("0", "240", "auto");
/// Revision record element local names that carry a `w:id` identifying the
/// change. Word treats a colliding id on any of these as the same revision
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
];

/// The next free revision id under `styles_root`: one greater than the maximum
/// numeric `w:id` on any `w:*Change` revision element present (0 when none).
/// `merge_normal_style_spacing`/`merge_normal_style_rpr` synthesize at most one
/// `w:pPrChange` and one `w:rPrChange` per compare, so a single starting id is
/// reserved here; the rPr pass bumps by one when it fires after the pPr pass.
fn next_free_revision_id(dom: &Dom, styles_root: NodeId) -> u32 {
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
fn merge_normal_style_spacing(
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
    // Full cascade for B: stored → docDefaults → `fallback` factory.
    let b_cascade =
        |dom: &Dom, styles_root: NodeId, style: Option<NodeId>, fallback: (&str, &str, &str)| {
            if let Some(s) = stored(dom, style) {
                return s;
            }
            if let Some(s) = dd_val(dom, styles_root) {
                return s;
            }
            (
                fallback.0.to_string(),
                fallback.1.to_string(),
                fallback.2.to_string(),
            )
        };
    let a_stored = stored(dom, Some(a_style));
    let b_style = find_normal_style(dom, b_root);
    let b_stored = stored(dom, b_style);
    let a_dd = dd_val(dom, out_root);
    let b_dd = dd_val(dom, b_root);
    let a_normal_has_rpr = dom.element(a_style, &W::name("rPr")).is_some();
    let a_normal_has_ppr = dom.element(a_style, &W::name("pPr")).is_some();
    let b_normal_has_rpr = b_style
        .map(|s| dom.element(s, &W::name("rPr")).is_some())
        .unwrap_or(false);
    let b_normal_has_ppr = b_style
        .map(|s| dom.element(s, &W::name("pPr")).is_some())
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

    let b_target: Option<(String, String, String)> = match (&a_stored, &b_stored) {
        (None, None) => match (&a_dd, &b_dd) {
            // file_8: identical dd both sides → leave Normal empty — unless
            // M106 (B structured rPr, A bare) needs Word's empty+pPrChange.
            (Some(a), Some(b)) if a == b && !m106_same_dd_clear => return false,
            (Some(_), Some(_)) if m106_same_dd_clear => None,
            // file_46 / 76 / 198: B Normal has non-spacing pPr (+ differing dd)
            // → Word writes B's dd. file_19 both bare → leave empty.
            (Some(_), Some(b)) if b_structured => Some(b.clone()),
            (Some(_), Some(_)) => return false,
            // file_34 A-rPr / file_104 A-pPr: promote B dd. file_69 bare → empty.
            (None, Some(b)) if a_structured => Some(b.clone()),
            (None, Some(_)) => return false,
            // file_77 A-pPr / file_33 B-rPr / file_103 B-pPr → 0/240.
            // file_145 both bare → leave empty (not 0/240).
            (Some(_), None) if a_structured || b_structured => {
                let (a, l, r) = EMPTY_B_SINGLE_LINE_NORMAL;
                Some((a.to_string(), l.to_string(), r.to_string()))
            }
            (Some(_), None) => return false,
            (None, None) => return false,
        },
        // A empty, B stored: target B's stored values only.
        (None, Some(b)) => Some(b.clone()),
        // A stored, B empty: cascade B. If that cascade equals the package
        // docDefaults already on the A-based stylesheet, Word CLEARS A's
        // stored spacing without materializing cascade onto Normal (file_22:
        // A 0/240 + shared dd 200/276 → empty Normal + pPrChange).
        (Some(_), None) => {
            let casc = b_cascade(dom, b_root, b_style, FACTORY_NORMAL_SPACING);
            if a_dd.as_ref() == Some(&casc) {
                None // clear explicit spacing
            } else {
                Some(casc)
            }
        }
        // Both stored: B's stored wins when different.
        (Some(_), Some(b)) => Some(b.clone()),
    };
    // Identity: A already has the same explicit spacing we would write.
    if let (Some(a), Some(b)) = (&a_stored, &b_target)
        && a == b
    {
        return false;
    }
    // Clearing when A already has no stored spacing is a no-op — except M106,
    // where Word still tracks dd spacing in pPrChange next to rPrChange.
    if b_target.is_none() && a_stored.is_none() && !m106_same_dd_clear {
        return false;
    }
    // Old value = A's stored pPr (empty w:pPr when absent), captured before
    // the rewrite. pPrChange's inner pPr must not itself carry a pPrChange —
    // a CT_PPrBase violation Word repairs/drops — so strip any nested
    // change history from the cloned subtree (PR #81 review: real-world
    // stylesheets with pending redline on Normal).
    // M106: when A never stored pPr, Word's pPrChange old still holds the
    // shared docDefaults spacing (after=200 line=276).
    let old_ppr = match dom.element(a_style, &W::name("pPr")) {
        Some(p) => {
            let clone = dom.clone_subtree(p);
            for c in dom.descendants(clone, Some(&W::name("pPrChange"))) {
                dom.remove(c);
            }
            clone
        }
        None => {
            let p = dom.new_element(W::name("pPr"));
            if m106_same_dd_clear && let Some((after, line, rule)) = &a_dd {
                let spacing = dom.new_element(W::name("spacing"));
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
    let ppr = match dom.element(a_style, &W::name("pPr")) {
        Some(p) => p,
        None => {
            let p = dom.new_element(W::name("pPr"));
            // pPr precedes rPr in CT_Style; insert before rPr when present.
            match dom.element(a_style, &W::name("rPr")) {
                Some(rpr) => dom.add_before_self(rpr, p),
                None => dom.add(a_style, p),
            }
            p
        }
    };
    // Apply or clear Normal spacing.
    if let Some((after, line, rule)) = &b_target {
        let spacing = match dom.element(ppr, &W::name("spacing")) {
            Some(s) => s,
            None => {
                let s = dom.new_element(W::name("spacing"));
                dom.add_first(ppr, s);
                s
            }
        };
        // Word materializes missing `w:after` as "0" when a line metric is
        // present (file_196: B stores line=276 only → after=0 line=276).
        let after = if after.is_empty() && !line.is_empty() {
            "0"
        } else {
            after.as_str()
        };
        let set = |dom: &mut Dom, name: &str, v: &str| {
            dom.set_attribute_value(
                spacing,
                &W::name(name),
                if v.is_empty() { None } else { Some(v) },
            );
        };
        set(dom, "after", after);
        set(dom, "line", line);
        set(dom, "lineRule", rule);
        // M64/M70: Normal `w:ind` follows B.
        // - B has ind (file_196 firstLine=432) → copy B's ind onto merged Normal.
        // - B has no ind (file_197 bare Normal) → drop A's leftover firstLine so
        //   Word's after/line-only Normal is not polluted with A ind.
        let b_ind = b_style
            .and_then(|bs| dom.element(bs, &W::name("pPr")))
            .and_then(|bppr| dom.element(bppr, &W::name("ind")));
        if let Some(old_ind) = dom.element(ppr, &W::name("ind")) {
            dom.remove(old_ind);
        }
        if let Some(bind) = b_ind {
            let clone = dom.clone_subtree(bind);
            // ind follows spacing in CT_PPr; place before pPrChange (added below).
            if let Some(sp) = dom.element(ppr, &W::name("spacing")) {
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
        // M72: Word live Normal after spacing merge is ONLY spacing (+ B ind).
        // A's non-spacing pPr (widowControl/tabs/suppressAutoHyphens on
        // file_77) must live in pPrChange old, not on the live style — LO
        // otherwise keeps A tab stops / widow rules and drifts pages.
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
    } else if let Some(sp) = dom.element(ppr, &W::name("spacing")) {
        // Clear explicit spacing — Word leaves empty pPr (file_22).
        dom.remove(sp);
    }
    let chg = dom.new_element(W::name("pPrChange"));
    // Word treats a colliding w:id on two w:*Change records as the same
    // revision and discards the later one. Scan the stylesheet for the next
    // free id rather than hardcoding "1" (PR #81 review).
    let id = next_free_revision_id(dom, out_root);
    dom.set_attribute_value(chg, &W::name("id"), Some(&id.to_string()));
    dom.set_attribute_value(
        chg,
        &W::name("author"),
        Some(&settings.author_for_revisions),
    );
    dom.set_attribute_value(
        chg,
        &W::name("date"),
        Some(&settings.date_time_for_revisions),
    );
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
fn cascade_normal_change_to_based_styles(
    dom: &mut Dom,
    styles_root: NodeId,
    settings: &WmlComparerSettings,
) -> bool {
    let Some(normal) = find_normal_style(dom, styles_root) else {
        return false;
    };
    let normal_ppc = dom
        .element(normal, &W::name("pPr"))
        .and_then(|p| dom.element(p, &W::name("pPrChange")));
    let normal_rpc = dom
        .element(normal, &W::name("rPr"))
        .and_then(|r| dom.element(r, &W::name("rPrChange")));
    if normal_ppc.is_none() && normal_rpc.is_none() {
        return false;
    }
    let old_spacing = normal_ppc.and_then(|ppc| {
        let old_ppr = dom.element(ppc, &W::name("pPr"))?;
        dom.element(old_ppr, &W::name("spacing"))
    });
    let old_rpr = normal_rpc.and_then(|rpc| dom.element(rpc, &W::name("rPr")));

    let mut changed = false;
    let styles: Vec<NodeId> = dom.elements(styles_root, Some(&W::name("style")));
    for style in styles {
        if dom.attribute(style, &W::name("type")) != Some("paragraph") {
            continue;
        }
        if dom.attribute(style, &W::name("styleId")) == Some("Normal") {
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
            let ppr = match dom.element(style, &W::name("pPr")) {
                Some(p) => p,
                None => {
                    let p = dom.new_element(W::name("pPr"));
                    match dom.element(style, &W::name("rPr")) {
                        Some(r) => dom.add_before_self(r, p),
                        None => dom.add(style, p),
                    }
                    p
                }
            };
            if dom.element(ppr, &W::name("pPrChange")).is_none() {
                let old_ppr = dom.new_element(W::name("pPr"));
                let mut has_spacing = false;
                for c in dom.elements(ppr, None) {
                    if dom.name(c) == Some(W::name("pPrChange")) {
                        continue;
                    }
                    if dom.name(c) == Some(W::name("spacing")) {
                        has_spacing = true;
                    }
                    let clone = dom.clone_subtree(c);
                    dom.add(old_ppr, clone);
                }
                if !has_spacing {
                    let clone = dom.clone_subtree(old_sp);
                    dom.add(old_ppr, clone);
                } else if let Some(live_sp) = dom.element(old_ppr, &W::name("spacing")) {
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
                let chg = dom.new_element(W::name("pPrChange"));
                let id = next_free_revision_id(dom, styles_root);
                dom.set_attribute_value(chg, &W::name("id"), Some(&id.to_string()));
                dom.set_attribute_value(
                    chg,
                    &W::name("author"),
                    Some(&settings.author_for_revisions),
                );
                dom.set_attribute_value(
                    chg,
                    &W::name("date"),
                    Some(&settings.date_time_for_revisions),
                );
                dom.add(chg, old_ppr);
                dom.add(ppr, chg);
                changed = true;
            }
        }

        // --- rPrChange ---
        if let Some(old_r) = old_rpr {
            let rpr = match dom.element(style, &W::name("rPr")) {
                Some(r) => r,
                None => {
                    let r = dom.new_element(W::name("rPr"));
                    dom.add(style, r);
                    r
                }
            };
            if dom.element(rpr, &W::name("rPrChange")).is_none() {
                let chg = dom.new_element(W::name("rPrChange"));
                let id = next_free_revision_id(dom, styles_root);
                dom.set_attribute_value(chg, &W::name("id"), Some(&id.to_string()));
                dom.set_attribute_value(
                    chg,
                    &W::name("author"),
                    Some(&settings.author_for_revisions),
                );
                dom.set_attribute_value(
                    chg,
                    &W::name("date"),
                    Some(&settings.date_time_for_revisions),
                );
                let clone = dom.clone_subtree(old_r);
                // strip nested rPrChange if any
                for c in dom.descendants(clone, Some(&W::name("rPrChange"))) {
                    dom.remove(c);
                }
                dom.add(chg, clone);
                dom.add(rpr, chg);
                changed = true;
            }
        }
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
fn normalize_word_paragraph_style_line(dom: &mut Dom, styles_root: NodeId) -> bool {
    // Gate: Normal must already be single-line after merge (has line) AND
    // not a "block spacing" Normal (before>0). file_33: after=0 line=240.
    // file_8: before=480 after=0 — Word leaves Headings without line; our
    // Normal may still carry a stray line attr from earlier merges.
    let Some(normal_sp) = find_normal_style(dom, styles_root)
        .and_then(|n| dom.element(n, &W::p_pr()))
        .and_then(|p| dom.element(p, &W::name("spacing")))
    else {
        return false;
    };
    if dom.attribute(normal_sp, &W::name("line")).is_none() {
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
            dom.attribute(s, &W::name("type")) == Some("paragraph")
                || dom.attribute(s, &W::name("type")).is_none()
        })
        .collect();
    for style in styles {
        let sid = dom
            .attribute(style, &W::name("styleId"))
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
        if let Some(sp) = dom.element(ppr, &W::name("spacing")) {
            let has_line = dom.attribute(sp, &W::name("line")).is_some();
            if !has_line {
                dom.set_attribute_value(sp, &W::name("line"), Some("240"));
                dom.set_attribute_value(sp, &W::name("lineRule"), Some("auto"));
                changed = true;
            }
        } else if sid == "Title" || sid == "ListParagraph" {
            let sp = dom.new_element(W::name("spacing"));
            dom.set_attribute_value(sp, &W::name("after"), Some("0"));
            dom.set_attribute_value(sp, &W::name("line"), Some("240"));
            dom.set_attribute_value(sp, &W::name("lineRule"), Some("auto"));
            // spacing first in pPr (before pPrChange if any)
            if let Some(chg) = dom.element(ppr, &W::name("pPrChange")) {
                dom.add_before_self(chg, sp);
            } else {
                dom.add_first(ppr, sp);
            }
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
fn align_paragraph_style_fonts_with_normal(dom: &mut Dom, styles_root: NodeId) -> bool {
    let Some(normal) = find_normal_style(dom, styles_root) else {
        return false;
    };
    let Some(normal_rpr) = dom.element(normal, &W::name("rPr")) else {
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
            dom.attribute(s, &W::name("type")) == Some("paragraph")
                || dom.attribute(s, &W::name("type")).is_none()
        })
        .collect();

    for style in styles {
        let sid = dom
            .attribute(style, &W::name("styleId"))
            .unwrap_or("")
            .to_string();
        if PROMOTE.contains(&sid.as_str()) {
            // Only materialize rFonts when the style has none. Do not overwrite
            // theme fonts (file_8 Title = majorHAnsi) or an existing face —
            // Word leaves those alone. file_33 Title/ListParagraph/Highlighted
            // ship with sz-only rPr and no rFonts element.
            let rpr = match dom.element(style, &W::name("rPr")) {
                Some(r) => r,
                None => continue,
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
            let Some(rpr) = dom.element(style, &W::name("rPr")) else {
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

/// Run-metric keys the footer merge resolves and compares: rFonts attributes
/// plus sz/szCs values (the properties that set a footer line's box height).
const RPR_METRIC_FONT_ATTRS: [&str; 4] = ["ascii", "hAnsi", "eastAsia", "cs"];

/// A style tree's `docDefaults/rPrDefault/rPr` node, if present.
fn rpr_default(dom: &Dom, styles_root: NodeId) -> Option<NodeId> {
    let dd = dom.element(styles_root, &W::name("docDefaults"))?;
    let rd = dom.element(dd, &W::name("rPrDefault"))?;
    dom.element(rd, &W::name("rPr"))
}

/// Normal's EFFECTIVE run metrics: each value from the style's stored rPr
/// when present, else from docDefaults' rPrDefault (per-attribute, the way
/// Word resolves a style chain). Returns [ascii, hAnsi, eastAsia, cs, sz,
/// szCs], each None when defined nowhere.
fn effective_normal_rpr_metrics(
    dom: &Dom,
    styles_root: NodeId,
    normal: Option<NodeId>,
) -> [Option<String>; 6] {
    let stored = normal.and_then(|s| dom.element(s, &W::name("rPr")));
    let default = rpr_default(dom, styles_root);
    let font_attr = |attr: &str| {
        for src in [stored, default] {
            if let Some(v) = src
                .and_then(|r| dom.element(r, &W::name("rFonts")))
                .and_then(|f| dom.attribute(f, &W::name(attr)))
            {
                return Some(v.to_string());
            }
        }
        None
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
    let [a, h, ea, cs] = RPR_METRIC_FONT_ATTRS.map(font_attr);
    [a, h, ea, cs, sz_val("sz"), sz_val("szCs")]
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
fn add_rpr_child_in_order(dom: &mut Dom, rpr: NodeId, child: NodeId, local: &str) {
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
/// header/footer→Normal (footer knife-edge line box). M71 always runs it in
/// Word mode so no-HF pairs like file_197 also get B's Calibri dd; M65 still
/// skips both-bare Normal (file_170).
///
/// GT evidence (sample-document × sd-2517-localized-heading-styles): GT Normal
/// rPr = Times New Roman sz/szCs 24 + rPrChange(old = Inter sz 22).
fn merge_normal_style_rpr(
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
    let a_has_rpr = dom.element(a_style, &W::name("rPr")).is_some();
    let b_has_rpr = b_style
        .map(|s| dom.element(s, &W::name("rPr")).is_some())
        .unwrap_or(false);
    if !a_has_rpr && !b_has_rpr {
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
        .element(a_style, &W::name("rPr"))
        .or_else(|| rpr_default(dom, out_root))
    {
        Some(r) => {
            let clone = dom.clone_subtree(r);
            for c in dom.descendants(clone, Some(&W::name("rPrChange"))) {
                dom.remove(c);
            }
            clone
        }
        None => dom.new_element(W::name("rPr")),
    };
    let rpr = match dom.element(a_style, &W::name("rPr")) {
        Some(r) => r,
        None => {
            let r = dom.new_element(W::name("rPr"));
            // rPr follows pPr in CT_Style; Normal's remaining children
            // (name/qFormat/pPr) all precede it, so append.
            dom.add(a_style, r);
            r
        }
    };
    let [ascii, hansi, east_asia, cs, sz, sz_cs] = b_effective;
    let fonts = match dom.element(rpr, &W::name("rFonts")) {
        Some(f) => f,
        None => {
            let f = dom.new_element(W::name("rFonts"));
            dom.add_first(rpr, f);
            f
        }
    };
    for (attr, v) in RPR_METRIC_FONT_ATTRS
        .iter()
        .zip([&ascii, &hansi, &east_asia, &cs])
    {
        dom.set_attribute_value(fonts, &W::name(attr), v.as_deref());
    }
    // sz/szCs must follow EG_RPrBase order (rFonts < b..webHidden < color <
    // spacing < w < kern < position < sz < szCs). Anchoring them to rFonts —
    // as this code previously did — places them before any of color/spacing/
    // w/kern/position Normal already carries, breaking CT_RPr order and
    // tripping Word's repair. Insert after the last existing predecessor so
    // the new/updated sz/szCs land in their schema slot.
    for (name, v) in [("sz", &sz), ("szCs", &sz_cs)] {
        let e = match dom.element(rpr, &W::name(name)) {
            Some(e) => e,
            None => {
                let e = dom.new_element(W::name(name));
                add_rpr_child_in_order(dom, rpr, e, name);
                e
            }
        };
        dom.set_attribute_value(e, &W::val(), v.as_deref());
    }
    let chg = dom.new_element(W::name("rPrChange"));
    // Next free id (see merge_normal_style_spacing): the pPr pass, when it
    // fired, reserved `next_free_revision_id` and Word now records that id,
    // so this rPr pass must not reuse it. Re-scan after the pPr change.
    let id = next_free_revision_id(dom, out_root);
    dom.set_attribute_value(chg, &W::name("id"), Some(&id.to_string()));
    dom.set_attribute_value(
        chg,
        &W::name("author"),
        Some(&settings.author_for_revisions),
    );
    dom.set_attribute_value(
        chg,
        &W::name("date"),
        Some(&settings.date_time_for_revisions),
    );
    dom.add(chg, old_rpr);
    dom.add(rpr, chg); // rPrChange is last in CT_RPr
    true
}

/// Merge every direct `w:body` child of `root` into one body and return it.
/// Normalize Strict/ISO OOXML namespace URIs (`http://purl.oclc.org/ooxml/<cat>/`)
/// to the Transitional URIs (`http://schemas.openxmlformats.org/<cat>/2006/`) the
/// comparer's XName tables use. Word writes either variant; we only model
/// Transitional, so a Strict document.xml otherwise has "no body" (and all markup
/// is unrecognized). No-op for Transitional docs (the common case).
fn normalize_strict_namespaces(xml: &str) -> std::borrow::Cow<'_, str> {
    if !xml.contains("purl.oclc.org/ooxml/") {
        return std::borrow::Cow::Borrowed(xml);
    }
    let s = xml
        .replace(
            "http://purl.oclc.org/ooxml/wordprocessingml/",
            "http://schemas.openxmlformats.org/wordprocessingml/2006/",
        )
        .replace(
            "http://purl.oclc.org/ooxml/officeDocument/",
            "http://schemas.openxmlformats.org/officeDocument/2006/",
        )
        .replace(
            "http://purl.oclc.org/ooxml/drawingml/",
            "http://schemas.openxmlformats.org/drawingml/2006/",
        );
    std::borrow::Cow::Owned(s)
}

/// Some producers emit multiple `w:body` elements (invalid, but real — e.g.
/// Apache POI MultipleBodyBug); Word concatenates them. Single-body docs are
/// returned unchanged (early return), so existing behavior is untouched.
fn merged_body(dom: &mut Dom, root: NodeId) -> Option<NodeId> {
    let bodies = dom.elements(root, Some(&W::body()));
    if bodies.len() <= 1 {
        return bodies.first().copied();
    }
    let sectpr = W::name("sectPr");
    let target = bodies[0];
    let mut content: Vec<NodeId> = Vec::new();
    let mut last_sectpr: Option<NodeId> = None;
    for &b in &bodies {
        for c in dom.nodes(b) {
            dom.remove(c);
            if dom.is_element(c) && dom.name(c).as_ref() == Some(&sectpr) {
                last_sectpr = Some(c);
            } else {
                content.push(c);
            }
        }
    }
    for c in content {
        dom.add(target, c);
    }
    if let Some(sp) = last_sectpr {
        dom.add(target, sp);
    }
    for &b in &bodies[1..] {
        dom.remove(b);
    }
    Some(target)
}

/// C.1/C.2 — `WmlComparer.PreProcessMarkup` (:434) at package level:
/// `ChangeFootnoteEndnoteReferencesToUniqueRange` (:1627) then
/// `AddFootnotesEndnotesParts` (:1604). C.3–C.5 extend it with
/// FillInEmptyFootnotesEndnotes, DetachExternalData and
/// AddUnidsToMarkupInContentParts in the C# order. Returns the names of the
/// parts it rewrote or created (empty = pure no-op, bytes untouched). An
/// orphaned footnote/endnote reference panics — C# throws DocxodusException
/// when no ComparisonLog is wired (:1676), and the compare path wires none.
pub fn pre_process_markup(
    pkg: &mut PartFs,
    starting_id_for_footnotes_endnotes: i32,
) -> Vec<String> {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    // Resolve the notes parts via the document rels — the package-level
    // equivalent of wDoc.MainDocumentPart.FootnotesPart/EndnotesPart.
    let mut fn_part: Option<String> = None;
    let mut en_part: Option<String> = None;
    if let Some(rels) = pkg.read_rels_for(&main) {
        for r in &rels.items {
            if r.target_mode.as_deref() == Some("External") {
                continue;
            }
            match r.rel_type.rsplit('/').next().unwrap_or("") {
                "footnotes" => fn_part = Some(pkg.resolve_rel_target(&main, &r.target)),
                "endnotes" => en_part = Some(pkg.resolve_rel_target(&main, &r.target)),
                _ => {}
            }
        }
    }

    let Some(main_xml) = pkg.part_string(&main) else {
        return Vec::new();
    };
    let fn_xml = fn_part.as_deref().and_then(|p| pkg.part_string(p));
    let en_xml = en_part.as_deref().and_then(|p| pkg.part_string(p));

    let mut dom = Dom::new();
    let main_doc = dom.parse_xdocument(&main_xml);
    let Some(main_root) = dom.root(main_doc) else {
        return Vec::new();
    };
    let fn_doc = fn_xml.as_deref().map(|x| dom.parse_xdocument(x));
    let fn_root = fn_doc.and_then(|d| dom.root(d));
    let en_doc = en_xml.as_deref().map(|x| dom.parse_xdocument(x));
    let en_root = en_doc.and_then(|d| dom.root(d));

    // Renumber only when there is something to renumber or rewrite; a doc
    // WITH references but no notes part reaches the orphan panic inside the
    // unique-range step, before any part creation.
    let fn_ref = W::name("footnoteReference");
    let en_ref = W::name("endnoteReference");
    let has_refs = dom
        .descendants(main_root, None)
        .into_iter()
        .any(|d| dom.name(d).is_some_and(|n| n == fn_ref || n == en_ref));
    let mut changed = Vec::new();
    // C.1 — unique-range renumbering (only meaningful when notes-relevant; a
    // doc WITH references but no notes part panics inside, like C# throws).
    if has_refs || fn_root.is_some() || en_root.is_some() {
        crate::comparer::footnotes::change_footnote_endnote_references_to_unique_range(
            &mut dom,
            main_root,
            fn_root,
            en_root,
            starting_id_for_footnotes_endnotes,
            false,
        )
        .unwrap_or_else(|e| panic!("{e}"));
    }

    // C.5 — `AddUnidsToMarkupInContentParts` (:600): stamp `pt:Unid` on every
    // element of main + notes parts and declare pt14 mc:Ignorable on each
    // root. Runs BEFORE FillInEmpty like C#, so the stock fill paragraphs are
    // deliberately unid-less after preprocessing.
    crate::unid::assign_to_all_elements(&mut dom, main_root);
    crate::comparer::finalize::ignore_pt14_namespace(&mut dom, main_root);
    for r in [fn_root, en_root].into_iter().flatten() {
        crate::unid::assign_to_all_elements(&mut dom, r);
        crate::comparer::finalize::ignore_pt14_namespace(&mut dom, r);
    }

    // C.3 — `FillInEmptyFootnotesEndnotes` (:513): childless note definitions
    // gain the stock reference paragraph before diffing. (C# runs it after
    // AddFootnotesEndnotesParts, but freshly created parts hold no
    // definitions, so applying it here is identical.)
    if let Some(r) = fn_root {
        crate::comparer::footnotes::fill_in_empty_footnotes_endnotes(&mut dom, r, true);
    }
    if let Some(r) = en_root {
        crate::comparer::footnotes::fill_in_empty_footnotes_endnotes(&mut dom, r, false);
    }

    // Write-back: main always (it now carries unids), notes parts when present.
    pkg.set_part(&main, dom.serialize_document(main_doc).into_bytes());
    changed.push(main.clone());
    if let (Some(p), Some(d)) = (fn_part.as_deref(), fn_doc) {
        pkg.set_part(p, dom.serialize_document(d).into_bytes());
        changed.push(p.to_string());
    }
    if let (Some(p), Some(d)) = (en_part.as_deref(), en_doc) {
        pkg.set_part(p, dom.serialize_document(d).into_bytes());
        changed.push(p.to_string());
    }

    // C.2 — `AddFootnotesEndnotesParts` (:1604): UNCONDITIONALLY add an EMPTY
    // namespace-decorated notes part (rels + content type) when one is
    // missing. No separator notes here — C# adds those only when Rectify
    // rebuilds the output part. Runs AFTER the renumbering, like C# (a doc
    // with references but no part panics above, never reaches creation).
    let dir = main.rsplit_once('/').map(|(d, _)| d).unwrap_or("word");
    for (present, local) in [
        (fn_part.is_some(), "footnotes"),
        (en_part.is_some(), "endnotes"),
    ] {
        if present {
            continue;
        }
        let part = format!("{dir}/{local}.xml");
        let xml = format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
             <w:{local} {NOTES_ROOT_NAMESPACE_ATTRS}></w:{local}>"
        );
        pkg.set_part(&part, xml.into_bytes());
        pkg.add_document_relationship(
            &main,
            &format!("http://schemas.openxmlformats.org/officeDocument/2006/relationships/{local}"),
            &format!("{local}.xml"),
        );
        pkg.add_content_type_override(
            &format!("/{part}"),
            &format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{local}+xml"),
        );
        changed.push(part);
    }

    // C.4 — `DetachExternalData` (:497): strip `c:externalData` from every
    // chart part related to the main document. External-link relationships
    // are not propagated to the destination document, so the references would
    // dangle; the chart's own rels are left untouched. (C# rewrites every
    // chart part; we only rewrite ones that actually held externalData —
    // a serialization-only difference.)
    let chart_parts: Vec<String> = pkg
        .read_rels_for(&main)
        .map(|rels| {
            rels.items
                .iter()
                .filter(|r| {
                    r.target_mode.as_deref() != Some("External") && r.rel_type.ends_with("/chart")
                })
                .map(|r| pkg.resolve_rel_target(&main, &r.target))
                .collect()
        })
        .unwrap_or_default();
    for part in chart_parts {
        let Some(xml) = pkg.part_string(&part) else {
            continue;
        };
        let mut cdom = Dom::new();
        let cdoc = cdom.parse_xdocument(&xml);
        let Some(croot) = cdom.root(cdoc) else {
            continue;
        };
        let ext: Vec<NodeId> =
            cdom.descendants(croot, Some(&crate::namespaces::C::name("externalData")));
        if ext.is_empty() {
            continue;
        }
        for e in ext {
            cdom.remove(e);
        }
        pkg.set_part(&part, cdom.serialize_document(cdoc).into_bytes());
        changed.push(part);
    }
    changed
}

/// The namespace declarations C# attaches to a freshly-created
/// `w:footnotes`/`w:endnotes` root (`NamespaceAttributes`/
/// `FreshNamespaceAttributes` :1580–:1602), verbatim.
const NOTES_ROOT_NAMESPACE_ATTRS: &str = concat!(
    "xmlns:wpc=\"http://schemas.microsoft.com/office/word/2010/wordprocessingCanvas\" ",
    "xmlns:mc=\"http://schemas.openxmlformats.org/markup-compatibility/2006\" ",
    "xmlns:o=\"urn:schemas-microsoft-com:office:office\" ",
    "xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" ",
    "xmlns:m=\"http://schemas.openxmlformats.org/officeDocument/2006/math\" ",
    "xmlns:v=\"urn:schemas-microsoft-com:vml\" ",
    "xmlns:wp14=\"http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing\" ",
    "xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" ",
    "xmlns:w10=\"urn:schemas-microsoft-com:office:word\" ",
    "xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" ",
    "xmlns:w14=\"http://schemas.microsoft.com/office/word/2010/wordml\" ",
    "xmlns:wpg=\"http://schemas.microsoft.com/office/word/2010/wordprocessingGroup\" ",
    "xmlns:wpi=\"http://schemas.microsoft.com/office/word/2010/wordprocessingInk\" ",
    "xmlns:wne=\"http://schemas.microsoft.com/office/word/2006/wordml\" ",
    "xmlns:wps=\"http://schemas.microsoft.com/office/word/2010/wordprocessingShape\" ",
    "mc:Ignorable=\"w14 wp14\""
);

/// A.11 — `RevisionProcessor.AcceptRevisions` byte facade: accept every
/// tracked revision across main + headers/footers + notes + styles parts.
pub fn accept_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError> {
    let mut pkg = PartFs::open(docx)?;
    crate::revision_processor::accept_revisions_package(&mut pkg);
    pkg.to_zip()
}

/// A.11 — `RevisionProcessor.RejectRevisions` byte facade: reject every
/// tracked revision across main + headers/footers + notes + styles parts.
pub fn reject_revisions(docx: &[u8]) -> Result<Vec<u8>, OpcError> {
    let mut pkg = PartFs::open(docx)?;
    crate::revision_processor::reject_revisions_package(&mut pkg);
    pkg.to_zip()
}

/// Collision-proof target name for copying `want` (with `bytes`) into `out`:
/// free name or byte-identical existing part → `want` unchanged; an existing
/// part with DIFFERENT content → `{dir}/redlineB[_{n}]_{base}` (first free or
/// identical candidate). Byte-based on purpose — `part_string` returns None
/// for binary parts and would treat existing images as absent.
fn unique_part_name(out: &PartFs, want: &str, bytes: &[u8]) -> String {
    match out.part_bytes(want) {
        None => want.to_string(),
        Some(existing) if existing == bytes => want.to_string(),
        Some(_) => {
            let (dir, base) = want.rsplit_once('/').unwrap_or(("word", want));
            let mut n = 0usize;
            loop {
                let candidate = if n == 0 {
                    format!("{dir}/redlineB_{base}")
                } else {
                    format!("{dir}/redlineB_{n}_{base}")
                };
                match out.part_bytes(&candidate) {
                    None => return candidate,
                    Some(existing) if existing == bytes => return candidate,
                    Some(_) => n += 1,
                }
            }
        }
    }
}

/// Word-alignment mode (settings-gated): Word's Compare presents the REVISED
/// document's headers/footers as a UNION per (kind, `w:type`) slot. A's
/// existing refs/parts stay untouched; for each (header|footer,
/// even|default|first) reference in doc B's effective final sectPr that is
/// ABSENT from the output's final sectPr, copy doc B's part (+ its rels and
/// internal targets) into the output and reference it (evidence:
/// comments_complex-style-attr — the header exists only in doc B yet renders
/// in Word's redline; page-numbering-examples vs potpourritest — Word's
/// redline carries all six slots, A's footer diffed + B's five other parts).
/// "Absent" is judged with OOXML inheritance: a slot is only filled by B when
/// no sectPr in the output body (walking every sectPr in document order, not
/// just the final one) carries that (kind, type) ref — modeling the nearest
/// preceding section's inherited refs (sd-2517: B's blank footer otherwise
/// shadowed A's 19 inherited 3-line footers).
fn adopt_revised_header_footer(
    dom: &mut Dom,
    result_root: NodeId,
    pkg2: &PartFs,
    out: &mut PartFs,
    out_main: &str,
) {
    let href = W::name("headerReference");
    let fref = W::name("footerReference");
    let Some(body) = dom.element(result_root, &W::body()) else {
        return;
    };
    let Some(out_sect) = dom.element(body, &W::name("sectPr")) else {
        return;
    };
    // Collect (kind, w:type) refs present on ANY body sectPr (mid-breaks + final).
    let body_slots = |dom: &Dom, body: NodeId| -> std::collections::HashSet<(bool, String)> {
        dom.descendants(body, Some(&W::name("sectPr")))
            .into_iter()
            .flat_map(|sect| dom.elements(sect, None))
            .filter_map(|e| {
                let n = dom.name(e)?;
                let is_header = if n == href {
                    true
                } else if n == fref {
                    false
                } else {
                    return None;
                };
                let ty = dom
                    .attribute(e, &W::name("type"))
                    .unwrap_or("default")
                    .to_string();
                Some((is_header, ty))
            })
            .collect()
    };
    // Slots the final sectPr already carries explicitly.
    let final_slots: std::collections::HashSet<(bool, String)> = dom
        .elements(out_sect, None)
        .into_iter()
        .filter_map(|e| {
            let n = dom.name(e)?;
            let is_header = if n == href {
                true
            } else if n == fref {
                false
            } else {
                return None;
            };
            let ty = dom
                .attribute(e, &W::name("type"))
                .unwrap_or("default")
                .to_string();
            Some((is_header, ty))
        })
        .collect();
    // Whole-body occupancy (A-sourced mid-section footers, etc.).
    let body_occupied = body_slots(dom, body);
    // M66: when the FINAL sectPr lacks a (kind,type) that B's final carries,
    // still adopt B's part onto the final sect — but only if that slot never
    // came from A anywhere in the body. Mid-body footers from *inserted B
    // sections* (file_21: 19 mid footers, empty final) must not block B's
    // last-section footer20; A's genuine mid footers (sd-2517) still block
    // B blank from blanking the final slot.
    //
    // `a_ever` ≈ body slots that are not solely B-insert artifacts is hard to
    // recover after merge; practical rule used by Word evidence on file_21:
    // adopt B final ref when final_slots lacks it AND (body has no such slot
    // OR the package still lacks the B part under any name). The second
    // disjunct is applied below per-ref after we resolve B's target.

    let main2 = pkg2
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(x2) = pkg2.part_string(&main2) else {
        return;
    };
    let mut d2 = Dom::new();
    let doc2 = d2.parse_xdocument(&x2);
    let Some(r2) = d2.root(doc2) else {
        return;
    };
    let Some(b2) = d2.element(r2, &W::body()) else {
        return;
    };
    let sect2 = d2
        .element(b2, &W::name("sectPr"))
        .or_else(|| d2.descendants(b2, Some(&W::name("sectPr"))).last().copied());
    let Some(sect2) = sect2 else {
        return;
    };
    let id_to_target: std::collections::HashMap<String, String> = pkg2
        .read_rels_for(&main2)
        .map(|rels| {
            rels.items
                .iter()
                .map(|r| (r.id.clone(), r.target.clone()))
                .collect()
        })
        .unwrap_or_default();

    let refs: Vec<NodeId> = d2
        .elements(sect2, None)
        .into_iter()
        .filter(|&e| d2.name(e).is_some_and(|n| n == href || n == fref))
        .collect();
    for r in refs {
        let is_header = d2.name(r) == Some(href.clone());
        let ty = d2
            .attribute(r, &W::name("type"))
            .unwrap_or("default")
            .to_string();
        let slot = (is_header, ty.clone());
        // Final sectPr already has this slot → leave it.
        if final_slots.contains(&slot) {
            continue;
        }
        let Some(rid) = d2.attribute(r, &R::name("id")) else {
            continue;
        };
        let Some(target) = id_to_target.get(rid) else {
            continue;
        };
        let src_part = pkg2.resolve_rel_target(&main2, target);
        let Some(bytes) = pkg2.part_bytes(&src_part).map(<[u8]>::to_vec) else {
            continue;
        };
        // If body already has this slot AND an identical (or any) part with this
        // basename is already packaged, skip — mid-section A footers stay.
        // file_21: mid-body has footer/default from B inserts but final is empty
        // and footer20.xml is missing → still adopt onto final.
        let basename = src_part.rsplit('/').next().unwrap_or(&src_part);
        let part_already = out.parts().iter().any(|p| p.ends_with(basename));
        if body_occupied.contains(&slot) && part_already {
            continue;
        }
        // name collision with an existing (different) part: copy B's content
        // under a fresh name instead of dropping or clobbering it — byte-safe
        // existence check (part_string is None for binary parts)
        let part = unique_part_name(out, &src_part, &bytes);
        if out.part_bytes(&part).is_none() {
            out.set_part(&part, bytes);
        }
        let ct = if is_header {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"
        } else {
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"
        };
        out.add_content_type_override(&format!("/{part}"), ct);

        // carry the part's own rels + internal targets (header images etc.);
        // internal targets get the same collision-proof treatment — an
        // existing same-named part with DIFFERENT bytes (e.g. doc A's own
        // media/image1.png) must never be overwritten, so B's payload lands
        // under a fresh name and the rel target follows it
        if let Some(hrels) = pkg2.read_rels_for(&src_part) {
            let mut rels_xml = String::from(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
            );
            for hr in &hrels.items {
                let mode = hr
                    .target_mode
                    .as_deref()
                    .map(|m| format!(" TargetMode=\"{m}\""))
                    .unwrap_or_default();
                let mut rel_target_out = hr.target.clone();
                if hr.target_mode.as_deref() != Some("External") {
                    let t = pkg2.resolve_rel_target(&src_part, &hr.target);
                    if let Some(tb) = pkg2.part_bytes(&t).map(<[u8]>::to_vec) {
                        let t_out = unique_part_name(out, &t, &tb);
                        if out.part_bytes(&t_out).is_none() {
                            out.set_part(&t_out, tb);
                        }
                        // rel target is part-dir-relative; a renamed copy
                        // stays in the same directory
                        if t_out != t {
                            rel_target_out = t_out.rsplit('/').next().unwrap_or(&t_out).to_string();
                            if let Some((tdir, _)) = t.rsplit_once('/')
                                && let Some((pdir, _)) = part.rsplit_once('/')
                                && tdir != pdir
                            {
                                let sub = tdir.strip_prefix(&format!("{pdir}/")).unwrap_or(tdir);
                                rel_target_out = format!("{sub}/{rel_target_out}");
                            }
                        }
                        if let Some(ext) = t_out.rsplit('.').next() {
                            // case-insensitive: real packages carry .PNG/.Jpg
                            let ext_lc = ext.to_ascii_lowercase();
                            let mime = match ext_lc.as_str() {
                                "png" => Some("image/png"),
                                "jpeg" | "jpg" => Some("image/jpeg"),
                                "gif" => Some("image/gif"),
                                "tiff" | "tif" => Some("image/tiff"),
                                "bmp" => Some("image/bmp"),
                                "svg" => Some("image/svg+xml"),
                                "ico" => Some("image/x-icon"),
                                "emf" => Some("image/x-emf"),
                                "wmf" => Some("image/x-wmf"),
                                _ => None,
                            };
                            if let Some(m) = mime {
                                out.add_content_type_default(ext, m);
                            }
                        }
                    }
                }
                // XML-escape attribute values — external hyperlink targets
                // legitimately carry '&' (URLs with query strings)
                let xe = |s: &str| {
                    s.replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('"', "&quot;")
                };
                rels_xml.push_str(&format!(
                    "<Relationship Id=\"{}\" Type=\"{}\" Target=\"{}\"{}/>",
                    xe(&hr.id),
                    xe(&hr.rel_type),
                    xe(&rel_target_out),
                    mode
                ));
            }
            rels_xml.push_str("</Relationships>");
            let base = part.rsplit('/').next().unwrap_or(&part);
            let dir = part.rsplit_once('/').map(|(d, _)| d).unwrap_or("word");
            out.set_part(&format!("{dir}/_rels/{base}.rels"), rels_xml.into_bytes());
        }

        // rel from the output main + reference element in the final sectPr
        let rel_type = if is_header {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header"
        } else {
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer"
        };
        // rels live in word/_rels/: word/-parts get the dir-relative form,
        // anything else the absolute OPC form ("/customXml/…") so the target
        // still resolves (review: strip_prefix fallback pointed at
        // word/<other-dir>/…, which never exists)
        let rel_target = match part.strip_prefix("word/") {
            Some(rest) => rest.to_string(),
            None => format!("/{part}"),
        };
        let new_rid = out.add_document_relationship(out_main, rel_type, &rel_target);
        let refel = dom.new_element(if is_header {
            href.clone()
        } else {
            fref.clone()
        });
        dom.set_attribute_value(refel, &W::name("type"), Some(&ty));
        dom.set_attribute_value(refel, &R::name("id"), Some(&new_rid));
        dom.add_first(out_sect, refel);
    }
}

/// D.6 — `WmlComparer.GetRevisions` (:3940) byte facade: list every tracked
/// revision in a redline `.docx` — main-part groups, footnote/endnote
/// definition groups, `w:rPrChange` format changes, then (settings-gated)
/// move detection. `TestForInvalidContent` failures panic like the C# throw.
pub fn get_revisions(
    docx: &[u8],
    settings: &crate::comparer::WmlComparerSettings,
) -> Result<Vec<crate::comparer::WmlComparerRevision>, OpcError> {
    use crate::comparer::{preprocess, revisions};

    let pkg = PartFs::open(docx)?;
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = pkg.part_string(&main).expect("main document missing");
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).expect("main document has no root");

    // C# :3948–:3949 — TestForInvalidContent (throws) +
    // RemoveExistingPowerToolsMarkup on main and both notes parts.
    preprocess::test_for_invalid_content(&dom, root).unwrap_or_else(|e| panic!("{e}"));
    preprocess::remove_existing_powertools_markup(&mut dom, root);
    let mut note_roots: Vec<(NodeId, &str, crate::xmllinq::XName)> = Vec::new();
    let (fn_part, en_part) = notes_part_names(&pkg);
    for (part, def) in [
        (fn_part.as_str(), W::footnote()),
        (en_part.as_str(), W::endnote()),
    ] {
        if let Some(x) = pkg.part_string(part) {
            let nd = dom.parse_xdocument(&x);
            if let Some(r) = dom.root(nd) {
                preprocess::remove_existing_powertools_markup(&mut dom, r);
                note_roots.push((r, part, def));
            }
        }
    }

    let body = dom
        .element(root, &W::body())
        .expect("main document has no body");
    let mut revs = revisions::get_revisions_from_body(&mut dom, body, &main, settings);
    for (r, part, def) in &note_roots {
        revs.extend(revisions::get_revisions_from_note_definitions(
            &mut dom, *r, def, part, settings,
        ));
    }
    let mut fc_parts: Vec<(NodeId, &str)> = vec![(root, main.as_str())];
    for (r, part, _) in &note_roots {
        fc_parts.push((*r, part));
    }
    revs.extend(revisions::get_format_change_revisions(&mut dom, &fc_parts));
    revisions::detect_moves(&mut revs, settings);
    Ok(revs)
}

/// `DocumentComparer.CompareDocuments(original, modified, author)`.
pub fn compare_documents(
    original: &[u8],
    modified: &[u8],
    author: &str,
) -> Result<Vec<u8>, OpcError> {
    compare_documents_with_options(original, modified, author, DEFAULT_DATE)
}

/// Compare with an explicit revision timestamp (for reproducible output).
pub fn compare_documents_with_options(
    original: &[u8],
    modified: &[u8],
    author: &str,
    date: &str,
) -> Result<Vec<u8>, OpcError> {
    compare_documents_internal(original, modified, author, date, true)
}

/// Compare with caller-supplied [`WmlComparerSettings`] (author/date/detail
/// threshold/…). The other entry points delegate here with defaults.
pub fn compare_documents_with_settings(
    original: &[u8],
    modified: &[u8],
    settings: &WmlComparerSettings,
) -> Result<Vec<u8>, OpcError> {
    compare_documents_impl(original, modified, settings, true)
}

/// `WmlComparer.CompareInternal` (:152). `pre_process_original` mirrors C#'s
/// `preProcessMarkupInOriginal` — true for Compare; Consolidate passes false
/// because its original is already preprocessed (`CompareInternal(..., false)`).
pub fn compare_documents_internal(
    original: &[u8],
    modified: &[u8],
    author: &str,
    date: &str,
    pre_process_original: bool,
) -> Result<Vec<u8>, OpcError> {
    let settings = WmlComparerSettings {
        author_for_revisions: author.to_string(),
        date_time_for_revisions: date.to_string(),
        ..WmlComparerSettings::default()
    };
    compare_documents_impl(original, modified, &settings, pre_process_original)
}

/// Resolve the footnotes/endnotes part names via the main-document rels,
/// falling back to the standard names. OPC makes the rels authoritative —
/// producers legally use nonstandard part names, and hardcoding
/// `word/footnotes.xml` silently skipped their notes (PR #51 review).
fn notes_part_names(pkg: &PartFs) -> (String, String) {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let mut fn_p = "word/footnotes.xml".to_string();
    let mut en_p = "word/endnotes.xml".to_string();
    if let Some(rels) = pkg.read_rels_for(&main) {
        for r in &rels.items {
            if r.target_mode.as_deref() == Some("External") {
                continue;
            }
            match r.rel_type.rsplit('/').next().unwrap_or("") {
                "footnotes" => fn_p = pkg.resolve_rel_target(&main, &r.target),
                "endnotes" => en_p = pkg.resolve_rel_target(&main, &r.target),
                _ => {}
            }
        }
    }
    (fn_p, en_p)
}

/// True when any package part's XML carries tracked-change markup that Word
/// Compare would fold into the final view before diffing.
fn docx_has_tracked_changes(docx: &[u8]) -> bool {
    let Ok(pkg) = PartFs::open(docx) else {
        return false;
    };
    for name in pkg.parts() {
        if !name.ends_with(".xml") {
            continue;
        }
        let Some(xml) = pkg.part_string(&name) else {
            continue;
        };
        // Coarse but cheap: real TC carriers in WordprocessingML.
        if xml.contains("<w:ins")
            || xml.contains("<w:del")
            || xml.contains("<w:moveFrom")
            || xml.contains("<w:moveTo")
            || xml.contains("<w:rPrChange")
            || xml.contains("<w:pPrChange")
        {
            return true;
        }
    }
    false
}

fn compare_documents_impl(
    original: &[u8],
    modified: &[u8],
    settings: &WmlComparerSettings,
    pre_process_original: bool,
) -> Result<Vec<u8>, OpcError> {
    // IDENTICAL-INPUT-01: same input bytes → empty redline is the (accepted)
    // original package. Avoids dual package prep, Dom parse, LCS, and produce.
    // Critical for self-compare fixtures (e.g. redline × self).
    if original == modified {
        let mut owned = crate::strict_translation::strict_to_transitional_docx(original);
        if settings.merge_replaced_paragraphs && docx_has_tracked_changes(&owned) {
            owned = accept_revisions(&owned)?;
        }
        // IDENTICAL-INPUT still runs drawing/shape id fixups: source packages
        // may carry colliding wp:docPr/@id (strict01 corpus) that the full
        // produce path renumbers; skipping left S-dup-docpr-id regressions.
        owned = crate::comparer::fixups::fix_up_drawing_ids_in_package(&owned)?;
        return Ok(owned);
    }

    // M8: normalize ISO/IEC 29500 "Strict" inputs to "Transitional" before any
    // PartFs::open sees them (mirrors the OpenXML SDK's pre-compare step).
    // Transitional packages round-trip byte-identical (zero-churn), so the
    // golden/parity paths are unaffected; only Strict inputs are rewritten.
    let mut original_owned = crate::strict_translation::strict_to_transitional_docx(original);
    let mut modified_owned = crate::strict_translation::strict_to_transitional_docx(modified);

    // Word-visual mode + either side already carries track changes: accept both
    // packages first (Word Compare of *finals*). Without this, stamp/re-emit of
    // pre-existing TC drowns real moves and inflates ins/del (broken_ones_two
    // file_8_file_9 37→56, file_27_file_28 38→61 with accept-then). PowerTools
    // faithful leaves inputs as-is.
    if settings.merge_replaced_paragraphs
        && (docx_has_tracked_changes(&original_owned) || docx_has_tracked_changes(&modified_owned))
    {
        original_owned = accept_revisions(&original_owned)?;
        modified_owned = accept_revisions(&modified_owned)?;
    }

    // After prep, packages may still be byte-identical (rare non-self paths).
    if original_owned == modified_owned {
        return Ok(crate::comparer::fixups::fix_up_drawing_ids_in_package(
            &original_owned,
        )?);
    }

    let original: &[u8] = &original_owned;
    let modified: &[u8] = &modified_owned;

    let mut pkg1 = PartFs::open(original)?;
    let mut pkg2 = PartFs::open(modified)?;

    // CompareInternal :154–:155 — disjoint footnote/endnote id spaces: doc A
    // gets starting_id+1000, doc B +2000 (block hashing ignores ref ids, so
    // correlation is unaffected; the disjoint spaces make reference-driven
    // note pairing sound).
    let changed1 = if pre_process_original {
        pre_process_markup(
            &mut pkg1,
            settings.starting_id_for_footnotes_endnotes + 1000,
        )
    } else {
        Vec::new()
    };
    pre_process_markup(
        &mut pkg2,
        settings.starting_id_for_footnotes_endnotes + 2000,
    );

    let main1 = pkg1
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let main2 = pkg2
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());

    let xml1 = pkg1
        .part_string(&main1)
        .expect("original main document missing");
    let xml2 = pkg2
        .part_string(&main2)
        .expect("modified main document missing");
    // Strict/ISO OOXML uses purl.oclc.org namespace URIs; normalize to Transitional
    // (the only variant our XName tables model) so the body/markup is recognized.
    let xml1 = normalize_strict_namespaces(&xml1);
    let xml2 = normalize_strict_namespaces(&xml2);

    // Parse both into one arena so the comparer can work across them.
    let mut dom = Dom::new();
    let d1 = dom.parse_xdocument(&xml1);
    let d2 = dom.parse_xdocument(&xml2);
    let root1 = dom.root(d1).expect("original has no root");
    let root2 = dom.root(d2).expect("modified has no root");
    let body1 = merged_body(&mut dom, root1).expect("original has no body");
    let body2 = merged_body(&mut dom, root2).expect("modified has no body");

    // B.4 — reference-driven notes processing: parse both documents' notes
    // parts AND an independent copy of the original's parts (the withRevisions
    // parts — C# gets them from the wmlResult clone of preprocessed source1)
    // into the same Dom as the bodies. The pipeline (B.2/B.3) diffs each
    // definition by its reference's correlation status and rebuilds the
    // withRevisions parts renumbered 1..n.
    fn parse_part_root(dom: &mut Dom, pkg: &PartFs, name: &str) -> Option<NodeId> {
        let xml = pkg.part_string(name)?;
        let d = dom.parse_xdocument(&xml);
        dom.root(d)
    }
    let (fn1, en1) = notes_part_names(&pkg1);
    let (fn2, en2) = notes_part_names(&pkg2);
    let mut notes_ctx = crate::comparer::NotesContext {
        fn_before: parse_part_root(&mut dom, &pkg1, &fn1),
        fn_after: parse_part_root(&mut dom, &pkg2, &fn2),
        en_before: parse_part_root(&mut dom, &pkg1, &en1),
        en_after: parse_part_root(&mut dom, &pkg2, &en2),
        fn_with_revisions: parse_part_root(&mut dom, &pkg1, &fn1),
        en_with_revisions: parse_part_root(&mut dom, &pkg1, &en1),
    };

    let result_root = crate::comparer::compare_bodies_faithful_with_notes(
        &mut dom,
        root1,
        root2,
        body1,
        body2,
        settings,
        Some(&mut notes_ctx),
    );

    // Base the output on the original package, replacing the main document
    // part. When PreProcessMarkup rewrote parts (notes renumbering), base it
    // on the PREPROCESSED original instead — C# builds wmlResult from the
    // preprocessed source1 (:197 comment: the renumbered/unid'd markup must be
    // the one appearing in the result). Note-free inputs keep the raw bytes.
    let mut out = if changed1.is_empty() {
        PartFs::open(original)?
    } else {
        PartFs::open(&pkg1.to_zip()?)?
    };
    // M4.H.3: carry over / drop dangling relationship references from inserted
    // content so the output has no dangling rId (Word-repair preventer).
    crate::comparer::parts::reconcile_dangling_relationships(
        &mut dom,
        result_root,
        &mut out,
        &[&pkg1, &pkg2],
    );
    // Word-alignment mode: adopt the revised document's headers/footers when
    // the original supplied none (must run BEFORE the result serialization —
    // it adds references to the final sectPr).
    if settings.merge_replaced_paragraphs {
        adopt_revised_header_footer(&mut dom, result_root, &pkg2, &mut out, &main1);
        // M35: comments carryover — union parts (B's byte-identical when its
        // set ⊇ A's) + anchors re-injected at the equivalent text positions.
        crate::comparer::comments::carry_comments(
            &mut dom,
            result_root,
            &pkg1,
            &main1,
            &pkg2,
            &main2,
            &mut out,
            &main1,
            &settings.author_for_revisions,
        );
        // Comment anchors keep source ids (aligned with comments.xml). Re-run
        // revision renumber with those ids reserved so move/tblPrChange never
        // share an id with commentRange* (Word "unreadable content").
        crate::comparer::finalize::fix_up_revision_ids(&mut dom, &[result_root]);
    }

    // Final drawing/shape id renumber immediately before serialize — package
    // post-steps (reconcile, header/footer adopt, comments) can clone/graft
    // drawings after the mid-produce FixUpDocPrIds pass (S-dup-docpr-id).
    crate::comparer::fixups::fix_up_doc_pr_ids(&mut dom, result_root);
    crate::comparer::fixups::fix_up_shape_ids(&mut dom, result_root);
    crate::comparer::fixups::fix_up_shape_type_ids(&mut dom, result_root);

    let result_xml = dom.serialize_element(result_root);
    out.set_part(&main1, result_xml.into_bytes());

    // M4.H.8/H.9: copy styles/numbering referenced by inserted (modified) content
    // into the output so it stays Word-valid.
    for (part, is_styles) in [("word/styles.xml", true), ("word/numbering.xml", false)] {
        match (out.part_string(part), pkg2.part_string(part)) {
            (Some(to_xml), Some(from_xml)) => {
                let mut sd = Dom::new();
                let td = sd.parse_xdocument(&to_xml);
                let fd = sd.parse_xdocument(&from_xml);
                if let (Some(tr), Some(fr)) = (sd.root(td), sd.root(fd)) {
                    if is_styles {
                        crate::comparer::footnotes::copy_missing_styles(&mut sd, tr, fr);
                    } else {
                        crate::comparer::footnotes::copy_missing_numbering(&mut sd, tr, fr);
                    }
                    out.set_part(part, sd.serialize_element(tr).into_bytes());
                }
            }
            // A has no numbering part, B does (file_21×file_22): copy B's
            // numbering wholesale + document rel. Prior match required both
            // sides present → insert-heavy next with lists lost numbering.
            (None, Some(from_xml)) if !is_styles => {
                out.set_part(part, from_xml.into_bytes());
                out.add_content_type_override(
                    &format!("/{part}"),
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
                );
                let has_num_rel = out
                    .read_rels_for(&main1)
                    .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/numbering")));
                if !has_num_rel {
                    out.add_document_relationship(
                        &main1,
                        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
                        "numbering.xml",
                    );
                }
            }
            _ => {}
        }
    }

    // Word-parity: strip pStyle/rStyle that styles.xml does not define. LO maps
    // built-in names (Heading1, Title, …) even when the style entry is absent;
    // Word's redline omits the attribute entirely (heading_1_bold×heading_1_style).
    if settings.merge_replaced_paragraphs {
        let defined = out.part_string("word/styles.xml").map(|sx| {
            let mut sd = Dom::new();
            let doc = sd.parse_xdocument(&sx);
            sd.root(doc)
                .map(|r| crate::comparer::footnotes::defined_style_ids(&sd, r))
                .unwrap_or_default()
        });
        if let Some(defined) = defined {
            // Main document + common related parts that can carry pStyle/rStyle.
            let mut part_names: Vec<String> = vec![main1.clone()];
            for p in out.parts() {
                let is_style_carrier = p.starts_with("word/header")
                    || p.starts_with("word/footer")
                    || p == "word/footnotes.xml"
                    || p == "word/endnotes.xml"
                    || p == "word/comments.xml"
                    || p.starts_with("word/comments");
                if is_style_carrier {
                    part_names.push(p);
                }
            }
            part_names.sort();
            part_names.dedup();
            for part in part_names {
                let Some(xml) = out.part_string(&part) else {
                    continue;
                };
                let mut pd = Dom::new();
                let doc = pd.parse_xdocument(&xml);
                let Some(root) = pd.root(doc) else {
                    continue;
                };
                let n = crate::comparer::footnotes::strip_unresolved_style_refs(
                    &mut pd, root, &defined,
                );
                if n > 0 {
                    out.set_part(&part, pd.serialize_element(root).into_bytes());
                }
            }
        }
    }

    // M-PAG mechanism 2 (word mode): merged Normal style. Word's redline
    // stylesheet carries the REVISED document's effective Normal spacing with
    // a w:pPrChange recording the old value; ours (A-based) kept A's Normal
    // verbatim, diffusing ~1 page of drift per ~20 (sd-2517_sectpr-headerref:
    // 111 → 117 pages with this patch, GT 116). Provenance: when B's Normal
    // pPr is empty/absent, Word resolves it to FACTORY defaults
    // (after=160 line=278 lineRule=auto — matches neither side's docDefaults).
    if settings.merge_replaced_paragraphs
        && let (Some(out_xml), Some(b_xml)) = (
            out.part_string("word/styles.xml"),
            pkg2.part_string("word/styles.xml"),
        )
    {
        let mut sd = Dom::new();
        let od = sd.parse_xdocument(&out_xml);
        let bd = sd.parse_xdocument(&b_xml);
        if let (Some(or), Some(br)) = (sd.root(od), sd.root(bd)) {
            let mut changed = merge_normal_style_spacing(&mut sd, or, br, settings);
            // M-PAG mechanism 2b / M71: rewrite Normal rPr to B's effective
            // metrics when they differ. Formerly gated on header/footer→Normal
            // (footer knife-edge). That skipped file_197 (no HF): Word writes
            // B's Calibri dd onto Normal + rPrChange(A Ubuntu); we kept A.
            // M65 still skips both-bare (file_170). HF-linked cases unchanged.
            changed |= merge_normal_style_rpr(&mut sd, or, br, settings);
            // M111: cascade Normal pPrChange/rPrChange onto basedOn=Normal styles
            // (ListParagraph/BodyText/Header/… — file_130 Word has ~30).
            changed |= cascade_normal_change_to_based_styles(&mut sd, or, settings);
            // M79: Word single-line on Heading/Title/ListParagraph (file_33 3→2pp).
            changed |= normalize_word_paragraph_style_line(&mut sd, or);
            // M80: Title/ListParagraph/Highlighted Arial + Heading Latin inherit.
            changed |= align_paragraph_style_fonts_with_normal(&mut sd, or);
            if changed {
                out.set_part("word/styles.xml", sd.serialize_element(or).into_bytes());
            }
        }
    }

    // Word-mode repair: Word synthesizes a default numbering definition when
    // the document references a numId no w:num defines — dangling numbering
    // silently renders lists as plain paragraphs; Word repairs it on open, so
    // match it at compare time (evidence: nested-table-rowspan_numbered-list).
    if settings.merge_replaced_paragraphs {
        let num_id = W::name("numId");
        let mut referenced: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for np in dom.descendants(result_root, Some(&W::name("numPr"))) {
            // numeric guard: ST_DecimalNumber ids only — also keeps the
            // synthesized w:num XML injection-proof (ids are interpolated
            // into a template downstream)
            if let Some(nid) = dom.element(np, &num_id)
                && let Some(v) = dom.attribute(nid, &W::val())
                && v != "0"
                && v.parse::<u32>().is_ok()
                && seen.insert(v.to_string())
            {
                referenced.push(v.to_string());
            }
        }
        if !referenced.is_empty() {
            let existing = out.part_string("word/numbering.xml");
            let mut nd = Dom::new();
            let (nroot, part_was_missing) = match &existing {
                Some(xml) => {
                    let d = nd.parse_xdocument(xml);
                    (nd.root(d), false)
                }
                None => {
                    let d = nd.parse_xdocument(&format!(
                        "<w:numbering xmlns:w=\"{}\"></w:numbering>",
                        W::URI
                    ));
                    (nd.root(d), true)
                }
            };
            if let Some(nroot) = nroot {
                let defined: std::collections::HashSet<String> = nd
                    .elements(nroot, Some(&W::name("num")))
                    .into_iter()
                    .filter_map(|e| nd.attribute(e, &num_id).map(|s| s.to_string()))
                    .collect();
                let dangling: Vec<String> = referenced
                    .into_iter()
                    .filter(|r| !defined.contains(r))
                    .collect();
                if !dangling.is_empty() {
                    crate::comparer::footnotes::synthesize_dangling_numbering(
                        &mut nd, nroot, &dangling,
                    );
                    out.set_part(
                        "word/numbering.xml",
                        nd.serialize_element(nroot).into_bytes(),
                    );
                    if part_was_missing {
                        out.add_content_type_override(
                            "/word/numbering.xml",
                            "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
                        );
                        let has_rel = out.read_rels_for(&main1).is_some_and(|r| {
                            r.items.iter().any(|i| i.rel_type.ends_with("/numbering"))
                        });
                        if !has_rel {
                            out.add_document_relationship(
                                &main1,
                                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
                                "numbering.xml",
                            );
                        }
                    }
                }
            }
        }
    }

    // B.4 — write the rectified withRevisions notes parts (separators +
    // referenced definitions renumbered 1..n with real revision markup) into
    // the output package. Replaces the old by-id `compare_note_parts` model,
    // whose pairing broke whenever Word renumbered notes.
    let mut footnote_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut endnote_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (part, root, is_fn) in [
        (fn1.as_str(), notes_ctx.fn_with_revisions, true),
        (en1.as_str(), notes_ctx.en_with_revisions, false),
    ] {
        if let Some(r) = root {
            let def = if is_fn {
                W::footnote()
            } else {
                W::endnote()
            };
            let ids: std::collections::HashSet<String> = dom
                .elements(r, Some(&def))
                .into_iter()
                .filter_map(|n| dom.attribute(n, &W::id()).map(str::to_string))
                .collect();
            if is_fn {
                footnote_ids = ids;
            } else {
                endnote_ids = ids;
            }
            out.set_part(part, dom.serialize_element(r).into_bytes());
        }
    }
    // settings.xml may still list special footnote/endnote ids (e.g. id=1
    // continuationNotice) that rectify dropped. Dangling settings refs make
    // Word show "unreadable content" (OpenXmlValidator Semantic).
    if let Some(sx) = out.part_string("word/settings.xml") {
        let mut sd = Dom::new();
        let sdoc = sd.parse_xdocument(&sx);
        if let Some(sroot) = sd.root(sdoc) {
            crate::comparer::footnotes::sync_settings_special_note_ids(
                &mut sd,
                sroot,
                &footnote_ids,
                &endnote_ids,
            );
            out.set_part("word/settings.xml", sd.serialize_element(sroot).into_bytes());
        }
    }
    // M4.H.x: header/footer CONTENT diff (Word redlines header/footer changes; we
    // previously only copied the original's). Match A's parts to B's by reference
    // (kind,type). v1: for matched TEXT-ONLY parts (no relationship refs — the
    // redlined part keeps the original's rels, so ref-bearing parts could dangle),
    // diff the content and write the redline into the output's (original's) part.
    {
        let refs_b: std::collections::HashMap<(String, String), String> = header_footer_refs(&pkg2)
            .into_iter()
            .map(|(k, t, p)| ((k, t), p))
            .collect();
        for (kind, ty, part_a) in header_footer_refs(&pkg1) {
            let Some(part_b) = refs_b.get(&(kind.clone(), ty.clone())) else {
                continue;
            };
            if let (Some(xa), Some(xb)) = (pkg1.part_string(&part_a), pkg2.part_string(part_b)) {
                if xa.contains("r:id=")
                    || xa.contains("r:embed=")
                    || xb.contains("r:id=")
                    || xb.contains("r:embed=")
                {
                    continue; // v1: skip relationship-bearing header/footer parts
                }
                let mut hd = Dom::new();
                let da = hd.parse_xdocument(&xa);
                let db = hd.parse_xdocument(&xb);
                if let (Some(ra), Some(rb)) = (hd.root(da), hd.root(db)) {
                    let text_of = |d: &Dom, n: crate::xmllinq::NodeId| -> String {
                        d.descendants(n, Some(&W::name("t")))
                            .into_iter()
                            .map(|t| d.value(t))
                            .collect()
                    };
                    // capture BEFORE the compare mutates the arena
                    let a_text = text_of(&hd, ra);
                    let res = compare_bodies_faithful(&mut hd, ra, rb, ra, rb, settings);
                    // compare_bodies_faithful always rebuilds into
                    // <w:document><w:body>…</w:body></w:document>, even when the
                    // source roots are w:hdr / w:ftr. Looking for a nested
                    // hdr/ftr under that wrapper is dead (PR #81 / kilo): re-wrap
                    // the body children as the original container type so the
                    // redlined part stays a valid header/footer part.
                    let container_name = if kind == "header" {
                        W::name("hdr")
                    } else {
                        W::name("ftr")
                    };
                    let Some(out_body) = hd.element(res, &W::body()) else {
                        continue;
                    };
                    let container = hd.new_element(container_name);
                    // Preserve source-root namespace decls (w already on body
                    // children; copy any extras from A's original root).
                    let xmlns_ns = crate::xmllinq::XNamespace::xmlns();
                    for (an, av) in hd.attributes(ra) {
                        if an.namespace_name() == xmlns_ns.namespace_name()
                            || an.local_name() == "Ignorable"
                        {
                            hd.set_attribute_value(container, &an, Some(&av));
                        }
                    }
                    // Body-level sectPr is document geometry from the
                    // compare_bodies_faithful wrap — not valid inside hdr/ftr.
                    for c in hd.elements(out_body, None) {
                        if hd.name(c) == Some(W::name("sectPr")) {
                            continue;
                        }
                        hd.remove(c);
                        hd.add(container, c);
                    }
                    // M-PAG mech 1 guard: the diff must never leave a slot A
                    // populates with B's wholesale content. When B's matched
                    // part is effectively empty (run-less paragraphs) the
                    // "diff" degenerates to B's paragraphs with no revision
                    // markup, silently blanking A's footer (sd-2517 vs
                    // sectpr-headerref: all 19 footers blanked, −3 rendered
                    // pages). Word RETAINS A's content for slots A populates
                    // (GT-verified), so accept the diff only when it carries
                    // revision markup or still reads as A's text; otherwise
                    // keep A's part untouched.
                    let redlined = hd.serialize_element(container);
                    let has_revisions = redlined.contains("<w:ins")
                        || redlined.contains("<w:del")
                        || redlined.contains("pPrChange")
                        || redlined.contains("rPrChange");
                    if has_revisions || text_of(&hd, container) == a_text {
                        out.set_part(&part_a, redlined.into_bytes());
                    }
                }
            }
        }
    }

    // Strict/ISO OOXML: when the original is Strict, the output package would mix
    // a Transitional comparison-result document.xml with Strict styles/numbering/
    // rels (and copied-in Transitional styles) — an invalid mixed package. Make
    // the whole package consistently Transitional. No-op for Transitional packages.
    for part in out.parts() {
        if !(part.ends_with(".xml") || part.ends_with(".rels")) {
            continue;
        }
        if let Some(s) = out.part_string(&part)
            && s.contains("purl.oclc.org/ooxml/")
        {
            let n = normalize_strict_namespaces(&s).into_owned();
            out.set_part(&part, n.into_bytes());
        }
    }
    // Word-validity normalization on every validity-swept content part — NOT
    // document.xml alone. Word opens the package (headers/footers/notes/
    // settings/styles/rels/content-types); a clean body with a corrupt notes
    // or settings part still raises "unreadable content".
    //
    // (validator sweep: 146/166 outputs carried schema errors Word's own
    // redlines don't): canonicalize universal measures / fractional ints,
    // fix Strict artifacts (cnfStyle bitmask, wp14 percents, out-of-range
    // paraIds), and strip pt:* scratch so headers/notes don't ship Unids.
    // Scope notes: `word/charts/` (DrawingML) and `word/theme/` are included
    // ON PURPOSE — the Strict percent→per-thousand rewrite covers drawingml
    // namespaces; `word/media/*.xml` is vacuous for binary payloads.
    for part in out.parts() {
        let is_swept = part == main1
            || part == "word/styles.xml"
            || part == "word/numbering.xml"
            || part == "word/footnotes.xml"
            || part == "word/endnotes.xml"
            || part == "word/settings.xml"
            || (part.starts_with("word/header") && part.ends_with(".xml"))
            || (part.starts_with("word/footer") && part.ends_with(".xml"))
            || (part.starts_with("word/diagrams/") && part.ends_with(".xml"))
            || (part.starts_with("word/charts/") && part.ends_with(".xml"))
            || (part.starts_with("word/theme/") && part.ends_with(".xml"))
            || (part.starts_with("word/media/") && part.ends_with(".xml"));
        if !is_swept {
            continue;
        }
        if let Some(x) = out.part_string(&part) {
            let mut vd = Dom::new();
            let doc = vd.parse_xdocument(&x);
            if let Some(vr) = vd.root(doc) {
                crate::comparer::finalize::normalize_universal_measures(&mut vd, vr);
                crate::comparer::finalize::fix_strict_validity_artifacts(&mut vd, vr);
                crate::comparer::finalize::remove_powertools_scratch_markup(&mut vd, vr);
                out.set_part(&part, vd.serialize_element(vr).into_bytes());
            }
        }
    }
    // Final package-level notes↔settings coherence (after the validity sweep
    // re-serialized those parts). Dangling special-note ids in settings are a
    // package bug, not a document.xml bug.
    {
        let collect_ids = |part: &str, local: &str| -> std::collections::HashSet<String> {
            let mut set = std::collections::HashSet::new();
            let Some(x) = out.part_string(part) else {
                return set;
            };
            let mut d = Dom::new();
            let doc = d.parse_xdocument(&x);
            let Some(root) = d.root(doc) else {
                return set;
            };
            let name = W::name(local);
            for n in d.elements(root, Some(&name)) {
                if let Some(id) = d.attribute(n, &W::id()) {
                    set.insert(id.to_string());
                }
            }
            set
        };
        let fn_ids = collect_ids("word/footnotes.xml", "footnote");
        let en_ids = collect_ids("word/endnotes.xml", "endnote");
        if let Some(sx) = out.part_string("word/settings.xml") {
            let mut sd = Dom::new();
            let sdoc = sd.parse_xdocument(&sx);
            if let Some(sroot) = sd.root(sdoc) {
                crate::comparer::footnotes::sync_settings_special_note_ids(
                    &mut sd, sroot, &fn_ids, &en_ids,
                );
                out.set_part("word/settings.xml", sd.serialize_element(sroot).into_bytes());
            }
        }
    }
    out.to_zip()
}

#[cfg(test)]
mod tests {
    //! Word-validity regressions for synthesized revision records. Word treats
    //! a colliding `w:id` on two `w:*Change` records as the same revision and
    //! drops the later one, and repairs an out-of-order `CT_RPr`/`CT_TblPrBase`
    //! child sequence — so the synthesized records must use a free id and land
    //! in their schema slot.
    use super::*;

    fn parse(dom: &mut Dom, xml: &str) -> (NodeId, NodeId) {
        let doc = dom.parse_xdocument(xml);
        let root = dom.root(doc).expect("root");
        let styles = dom
            .element(root, &W::name("styles"))
            .or_else(|| dom.descendants(root, None).first().copied())
            .expect("styles root");
        (root, styles)
    }

    /// `next_free_revision_id` must be one greater than the max numeric id on
    /// ANY `w:*Change` revision element in the stylesheet, never a hardcoded 1.
    #[test]
    fn next_free_revision_id_is_max_plus_one_across_change_families() {
        let mut dom = Dom::new();
        // A stylesheet whose Normal carries a high-id rPrChange and another
        // style carries a pPrChange — both must raise the floor.
        let xml = concat!(
            "<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
            "<w:style w:type=\"paragraph\" w:styleId=\"Normal\">",
            "<w:rPr><w:rFonts w:ascii=\"Times\"/><w:rPrChange w:id=\"147\" w:author=\"x\" w:date=\"d\">",
            "<w:rPr><w:sz w:val=\"22\"/></w:rPr></w:rPrChange></w:rPr>",
            "</w:style>",
            "<w:style w:type=\"paragraph\" w:styleId=\"Heading1\">",
            "<w:pPr><w:pPrChange w:id=\"93\" w:author=\"x\" w:date=\"d\">",
            "<w:pPr/></w:pPrChange></w:pPr>",
            "</w:style>",
            "</w:styles>"
        );
        let (_root, styles) = parse(&mut dom, xml);
        assert_eq!(
            next_free_revision_id(&dom, styles),
            148,
            "next free id must exceed the highest existing *Change id (147), not be 1"
        );
    }

    /// sz must be inserted after position/kern, NOT immediately after rFonts —
    /// rFonts < color < spacing < w < kern < position < sz in EG_RPrBase.
    #[test]
    fn add_rpr_child_keeps_sz_in_schema_order_after_position() {
        let mut dom = Dom::new();
        let xml = concat!(
            "<w:rPr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
            "<w:rFonts w:ascii=\"Times\"/>",
            "<w:color w:val=\"auto\"/>",
            "<w:spacing w:val=\"0\"/>",
            "<w:kern w:val=\"0\"/>",
            "<w:position w:val=\"0\"/>",
            "</w:rPr>"
        );
        let doc = dom.parse_xdocument(xml);
        let rpr = dom.root(doc).expect("root");
        let sz = dom.new_element(W::name("sz"));
        add_rpr_child_in_order(&mut dom, rpr, sz, "sz");
        let order: Vec<String> = dom
            .elements(rpr, None)
            .into_iter()
            .map(|e| dom.name(e).unwrap().local_name().to_string())
            .collect();
        let sz_pos = order.iter().position(|n| n == "sz").unwrap();
        let pos_pos = order.iter().position(|n| n == "position").unwrap();
        assert!(
            sz_pos > pos_pos,
            "sz must follow position (EG_RPrBase order), got order: {order:?}"
        );
    }
}
