// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word-validity repairs: cell margins, strict-validity artifacts, measures, section-property identity.

use super::*;

/// Word-alignment mode: Word's Compare synthesizes near-zero cell margins
/// on the tables it emits — `w:tblInd w=10` and `w:tblCellMar left/right
/// w=10` — present in NEITHER input nor Word's TableNormal default of 108
/// (forensics pair #5, helvetica_hr-onboarding). Bare tables render narrower
/// effective text columns that over-wrap vs the ground truth. Fill only what
/// each table's tblPr doesn't define; children are inserted in CT_TblPr
/// schema order (tblW 70 < tblInd 100 < tblCellMar 140 < tblLook 150).
///
/// Word stamps them only when the table's source document has no default
/// table style: across the pool corpus, 94 bordered tables from documents with
/// a `TableNormal` default stay bare. `default_table_styles` says whether the
/// (original, revised) documents define one. A wholly deleted table comes from
/// the original and any other table from the revised document.
/// Synthesize `tcMar` margins on table cells that lack them, matching
/// whatever default the two source documents define.
///
/// A wholly deleted table inherits from the ORIGINAL; any other table
/// inherits from the REVISED document. Without this pass, tables with
/// no authored `tcMar` pick up LibreOffice / Word defaults that differ
/// from either input and shift every cell width.
pub fn synthesize_table_cell_margins(
    dom: &mut Dom,
    root: NodeId,
    default_table_styles: (bool, bool),
) {
    fn dxa(dom: &mut Dom, name: &str, w: &str) -> NodeId {
        let e = dom.new_element(W::name(name));
        dom.set_attribute_value(e, &W::w(), Some(w));
        dom.set_attribute_value(e, &W::type_(), Some("dxa"));
        e
    }
    let tbls: Vec<NodeId> = dom.descendants(root, Some(&W::tbl()));
    for tbl in tbls {
        let Some(tblpr) = dom.element(tbl, &W::tbl_pr()) else {
            continue;
        };
        // Word synthesizes ONLY on bordered tables — corpus discriminator:
        // every GT table with w:tblBorders carries mar10/ind10, the one
        // border-less table (24-id_alternate-content) does not.
        if dom.element(tblpr, &W::name("tblBorders")).is_none() {
            continue;
        }
        let rows = dom.elements(tbl, Some(&W::tr()));
        let wholly_deleted = !rows.is_empty()
            && rows.iter().all(|&tr| {
                dom.element(tr, &W::tr_pr())
                    .and_then(|p| dom.element(p, &W::del()))
                    .is_some()
            });
        let source_has_default = if wholly_deleted {
            default_table_styles.0
        } else {
            default_table_styles.1
        };
        if source_has_default {
            continue;
        }
        if dom.element(tblpr, &W::tbl_ind()).is_none() {
            let ind = dxa(dom, "tblInd", "10");
            insert_tblpr_child_in_order(dom, tblpr, ind, 100);
        }
        if dom.element(tblpr, &W::name("tblCellMar")).is_none() {
            let mar = dom.new_element(W::name("tblCellMar"));
            let l = dxa(dom, "left", "10");
            dom.add(mar, l);
            let r = dxa(dom, "right", "10");
            dom.add(mar, r);
            insert_tblpr_child_in_order(dom, tblpr, mar, 140);
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
///
/// Fix Word-strict-out-of-range validity issues that did NOT trip
/// the C# schema validator but DO trip Word's repair dialog.
///
/// Examples: `w:id` that exceeds the half-open `[0, 2147483648)` range,
/// `w:viewMergedAttrs` outside the OOXML truth table. Stripped on
/// encountering; Word regenerates its own on save and never emits
/// out-of-range ids.
pub fn fix_strict_validity_artifacts(dom: &mut Dom, root: NodeId) {
    strict_validity_artifacts(dom, root, true);
}

/// Only validity repairs, retaining valid Transitional start/end alignment
/// and DrawingML percentage spellings on an identical compare (#1796).
pub(crate) fn repair_strict_validity_artifacts(dom: &mut Dom, root: NodeId) {
    strict_validity_artifacts(dom, root, false);
}

pub(super) fn strict_validity_artifacts(dom: &mut Dom, root: NodeId, comparison: bool) {
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
                && let Some(f) = crate::util::parse_finite_f64(num.trim())
            {
                let scaled = f * 1000.0;
                // A non-finite product, or one past i64, is left as written.
                if scaled.is_finite() && scaled.abs() <= i64::MAX as f64 {
                    let per_thousand = scaled.round() as i64;
                    dom.set_value(e, &per_thousand.to_string());
                }
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
        let drawing = W::drawing();
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
            dom.set_name(e, W::txbx_content());
        }
    }
    // ISO 29500 Transitional also admits start/end and percentage spellings.
    // The historical full comparison canonicalizes them for rendering parity;
    // an identical exit must not rewrite these valid source values.
    if !comparison {
        return;
    }
    // Comparison canonicalization of start/end to the left/right spellings
    // Word uses in its Transitional redlines (strict01 numbering, 7× lvlJc).
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
    // a drawingml-family element is canonicalized for comparison rendering
    // (strict01's diagram and chart parts). w:-namespace attrs stay — Transitional
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
                let f = crate::util::parse_finite_f64(num.trim())?;
                let scaled = f * 1000.0;
                (scaled.is_finite() && scaled.abs() <= i64::MAX as f64)
                    .then_some((an, scaled.round() as i64))
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
/// Canonicalize Strict-style universal measures (`"612pt"` instead of
/// `"12240"`) into twips on layout attributes.
///
/// Runs after the sectPr reinstatement so the saved geometry is
/// normalized too. Whitelist-driven so half-point / eighth-point
/// attributes (`w:sz`, …) are never touched (LO mis-paginates the
/// `strict01` pairs).
pub fn normalize_universal_measures(dom: &mut Dom, root: NodeId) {
    normalize_measures(dom, root, true, false);
}

/// [`normalize_universal_measures`] without the fractional-plain-value
/// rounding: only unit-bearing spellings ("612pt") are rewritten. The
/// render diagnostic reports a plain value (gdocs-style `1440.4`) as the
/// file spells it — a diagnostic does not repair what it describes.
pub fn normalize_unit_measures(dom: &mut Dom, root: NodeId) {
    normalize_measures(dom, root, false, false);
}

/// Repair fractional integer measures without canonicalizing valid unit-bearing
/// forms. Inputs reach this pass after Strict-to-Transitional translation:
/// ST_TwipsMeasure admits nonnegative decimals here, as does CT_TblWidth's
/// ST_MeasurementOrPercent. Only ST_SignedTwipsMeasure needs plain integers.
pub(crate) fn repair_invalid_measures(dom: &mut Dom, root: NodeId) {
    normalize_measures(dom, root, true, true);
}

pub(super) fn normalize_measures(
    dom: &mut Dom,
    root: NodeId,
    round_plain_fractional: bool,
    validity_only: bool,
) {
    // Cell-margin children (w:tcMar / w:tblCellMar) carry @w:w in twips; the
    // like-named border elements carry @w:sz (eighth-points) and never @w:w,
    // so keying on (element local, attr) stays safe.
    const TWIPS_ATTRS: [(&str, &[&str]); 22] = [
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
        ("spacing", &["before", "after", "line", "val"]),
        ("tab", &["pos"]),
        ("defaultTabStop", &["val"]),
        ("tblW", &["w"]),
        ("tcW", &["w"]),
        ("tblInd", &["w"]),
        ("tblCellSpacing", &["w"]),
        ("trHeight", &["val"]),
        ("cols", &["space"]),
        ("col", &["w", "space"]),
        (
            "tblpPr",
            &[
                "leftFromText",
                "rightFromText",
                "topFromText",
                "bottomFromText",
                "tblpX",
                "tblpY",
            ],
        ),
        ("framePr", &["w", "h", "x", "y"]),
        ("top", &["w"]),
        ("bottom", &["w"]),
        ("left", &["w"]),
        ("right", &["w"]),
        ("start", &["w"]),
        ("end", &["w"]),
    ];
    fn to_twips(v: &str, round_plain_fractional: bool) -> Option<i64> {
        if let Some(tw) = crate::util::universal_measure_twips(v) {
            return Some(tw.round() as i64);
        }
        // fractional plain values ("100.0") on integer-typed attributes →
        // rounded (gdocs-exported styles carry them; validator: Int16 fail).
        // Output-side repair only: the render diagnostic keeps the literal.
        if !round_plain_fractional {
            return None;
        }
        let t = v.trim();
        if t.contains('.') {
            let f = crate::util::parse_finite_f64(t)?;
            let rounded = f.round();
            // Do not rewrite a value that will not fit: `as i64` would saturate.
            return (rounded.abs() <= i64::MAX as f64).then_some(rounded as i64);
        }
        None
    }
    for el in dom.descendants_and_self(root, None) {
        let Some(name) = dom.name(el) else { continue };
        if name.namespace_name() != W::URI {
            continue;
        }
        let Some((local, attrs)) = TWIPS_ATTRS.iter().find(|(ln, _)| name.local_name() == *ln)
        else {
            continue;
        };
        for a in *attrs {
            // Transitional wml.xsd: the signed type is integer | universal
            // measure. Unsigned twips and table widths admit decimals. Do
            // not infer the type from a dotted spelling or the element alone
            // (e.g. spacing/@after is decimal, spacing/@line is integer).
            if validity_only
                && !matches!(
                    (*local, *a),
                    ("pgMar", "top" | "bottom")
                        | ("ind", "left" | "right" | "start" | "end")
                        | ("spacing", "line" | "val")
                        | ("tab", "pos")
                        | ("tblpPr", "tblpX" | "tblpY")
                        | ("framePr", "x" | "y")
                )
            {
                continue;
            }
            let an = W::name(a);
            // and_then keeps this allocation-free for the common untouched
            // case — the i64 releases the dom borrow before the mutation
            if let Some(tw) = dom
                .attribute(el, &an)
                .filter(|v| !validity_only || crate::util::universal_measure_twips(v).is_none())
                .and_then(|v| to_twips(v, round_plain_fractional))
            {
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
/// Canonical, equality-friendly serialized form of a `w:sectPr`.
///
/// Used to recognize a section identity across two inputs (or the
/// original and the rebuilt live section) so the Word-mode lodgers
/// ([`drop_sectpr_from_deleted_marks`], `genuine_mid_sectprs`)
/// keep genuinely authored mid-section breaks and drop only the
/// hoisted-final-sectPr artifacts. XML attribute order is normalized
/// first; default-implicit `type=nextPage` / `cols equalWidth=1` /
/// single-column-at-720 are dropped so sections that merely spell
/// out the defaults do not read as changed.
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
/// After the live section is reinstated, drop relocated final-section
/// `w:sectPr` copies that carry the `pt:BodySectPrHoist` stamp
/// (placed there during the pipeline so [`drop_sectpr_from_deleted_marks`]
/// can tell them apart from genuine authored breaks).
///
/// Identified by the stamp attribute regardless of geometry or last
/// descendant position. Genuine authored mid-section breaks and their
/// `sectPrChange` history remain untouched, including ones whose
/// geometry is identical.
pub fn drop_hoisted_sectpr_artifacts(
    dom: &mut Dom,
    root: NodeId,
    genuine: &std::collections::HashSet<String>,
) {
    let sectpr = W::sect_pr();
    for ppr in dom.descendants(root, Some(&W::p_pr())) {
        for sp in dom.elements(ppr, Some(&sectpr)) {
            if !genuine.contains(&sectpr_identity(dom, sp)) {
                dom.remove(sp);
            }
        }
    }
}
