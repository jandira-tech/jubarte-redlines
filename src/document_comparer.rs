// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

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
use crate::namespaces::{R, W, W14};
use crate::opc::{OpcError, PartFs};
use crate::xmllinq::{Dom, NodeId, XName};

/// `xml` (part `part_b` of `pkg2`) with each relationship it references
/// carried onto `part_a` of `out` under a fresh id (an image with A's bytes
/// reuses A's relationship), so B's content means what it meant in B once
/// it is diffed into A's part. None when a relationship cannot be carried.
fn carry_revised_part_relationships(
    out: &mut PartFs,
    part_a: &str,
    (pkg2, part_b): (&PartFs, &str),
    xml: &str,
) -> Option<String> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(xml);
    let root = dom.root(doc)?;
    let mut minted: std::collections::HashMap<(String, Option<&str>), String> =
        std::collections::HashMap::new();
    for el in dom.descendants_and_self(root, None) {
        for (name, rid) in dom.attributes(el) {
            if !crate::comparer::tables::S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(&name) {
                continue;
            }
            let suffix = dom
                .name(el)
                .and_then(|n| crate::comparer::parts::required_rel_type_suffix(n.local_name()));
            let key = (rid.clone(), suffix);
            let id = match minted.get(&key) {
                Some(id) => id.clone(),
                None => {
                    let id = crate::comparer::parts::carry_relationship(
                        out,
                        part_a,
                        pkg2,
                        part_b,
                        &rid,
                        |ty| suffix.is_none_or(|s| ty.ends_with(s)),
                    )?;
                    minted.insert(key, id.clone());
                    id
                }
            };
            dom.set_attribute_value(el, &name, Some(&id));
        }
    }
    Some(dom.serialize_document(doc))
}

/// Every relationship `xml` (part `part_b` of `pkg2`) references resolves in
/// `part_a`'s rels to the same type, mode and target, and an internal target
/// to the same bytes. A redlined part written over `part_a` keeps A's rels,
/// so B's references then still mean what they meant in B.
fn part_rels_agree(
    (pkg1, part_a): (&PartFs, &str),
    (pkg2, part_b): (&PartFs, &str),
    xml: &str,
) -> bool {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(xml);
    let Some(root) = dom.root(document) else {
        return false;
    };
    let ids: std::collections::BTreeSet<String> = dom
        .descendants_and_self(root, None)
        .into_iter()
        .flat_map(|e| dom.attributes(e))
        .filter(|(name, _)| crate::comparer::tables::S_RELATIONSHIP_ATTRIBUTE_NAMES.contains(name))
        .map(|(_, value)| value)
        .collect();
    if ids.is_empty() {
        return true;
    }
    let (Some(rels_a), Some(rels_b)) = (pkg1.read_rels_for(part_a), pkg2.read_rels_for(part_b))
    else {
        return false;
    };
    ids.iter().all(|id| {
        let find =
            |rels: &crate::opc::Relationships| rels.items.iter().find(|r| &r.id == id).cloned();
        let (Some(a), Some(b)) = (find(rels_a), find(rels_b)) else {
            return false;
        };
        if a.rel_type != b.rel_type || a.target_mode != b.target_mode {
            return false;
        }
        if a.target_mode.as_deref() == Some("External") {
            return a.target == b.target;
        }
        let bytes_a = pkg1.part_bytes(&pkg1.resolve_rel_target(part_a, &a.target));
        let bytes_b = pkg2.part_bytes(&pkg2.resolve_rel_target(part_b, &b.target));
        bytes_a.is_some() && bytes_a == bytes_b
    })
}

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

/// The revised part a header/footer part is diffed against: the one the
/// same section references for the same kind and type (the `at`-th such
/// reference in each document), or none when A's part is unchanged in B.
/// Keyed by kind and type alone, every section's default footer met the last
/// section's ("Page 1 of 4" against "Page 4 of 4"), and an unchanged footer
/// came out deleted and inserted again.
fn pair_header_footer(
    pkg1: &PartFs,
    pkg2: &PartFs,
    refs_b: &[(String, String, String)],
    (kind, ty, part_a): (&str, &str, &str),
    at: usize,
) -> Option<String> {
    let same_slot: Vec<&str> = refs_b
        .iter()
        .filter(|(k, t, _)| k == kind && t == ty)
        .map(|(_, _, p)| p.as_str())
        .collect();
    let bytes_a = pkg1.part_bytes(part_a);
    if same_slot.iter().any(|&p| pkg2.part_bytes(p) == bytes_a) {
        return None;
    }
    same_slot
        .get(at)
        .or(same_slot.last())
        .map(|p| p.to_string())
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
                .iter()
                .copied()
                .find(|&s| dom.attribute(s, &W::name("default")) == Some("1"))
        })
        // LibreOffice writes `style0` named "Normal" and no default flag;
        // Word still treats it as Normal.
        .or_else(|| {
            styles.into_iter().find(|&s| {
                dom.element(s, &W::name("name"))
                    .and_then(|n| dom.attribute(n, &W::val()))
                    .is_some_and(|v| v.eq_ignore_ascii_case("Normal"))
            })
        })
}

/// A style's `pPr/spacing` as (after, line, lineRule), each None when absent.
fn normal_spacing(dom: &Dom, style: NodeId) -> Option<(String, String, String)> {
    let ppr = dom.element(style, &W::name("pPr"))?;
    let sp = dom.element(ppr, &W::name("spacing"))?;
    let get = |n: &str| dom.attribute(sp, &W::name(n)).unwrap_or("").to_string();
    Some((get("after"), get("line"), get("lineRule")))
}

/// `before` companion to [`normal_spacing`]/[`docdefaults_ppr_spacing`] —
/// M479: the Normal-merge truth table was derived over {after, line} only;
/// an A-docDefaults `before` (paragraph_spacing_missing: before=240) leaked
/// through unneutralized, adding 12pt above every Normal paragraph. Word
/// writes the explicit neutralizer (oracle Normal: before="0").
fn spacing_before_attr(dom: &Dom, sp_holder: Option<NodeId>) -> Option<String> {
    let sp = sp_holder?;
    dom.attribute(sp, &W::name("before")).map(str::to_string)
}

/// Word's own paragraph spacing, which it applies to a document with no
/// `w:docDefaults` at all (a LibreOffice export): after 160, line 278
/// (multi_section_nested_table_rowspan, table_bookmark_end ×
/// table_vmerge_colspan — both Word redlines write it into Normal).
const WORD_FACTORY_SPACING: (&str, &str, &str) = ("160", "278", "auto");

/// B's `docdefaults_ppr_spacing`, with Word's factory spacing standing in
/// when B has no `w:docDefaults` at all.
fn b_docdefaults_ppr_spacing(dom: &Dom, b_root: NodeId) -> Option<(String, String, String)> {
    if dom.element(b_root, &W::name("docDefaults")).is_none() {
        let (a, l, r) = WORD_FACTORY_SPACING;
        return Some((a.to_string(), l.to_string(), r.to_string()));
    }
    docdefaults_ppr_spacing(dom, b_root)
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

/// M487 — effective paragraph spacing for `style_id` under one stylesheet:
/// (after, before, line, lineRule), each resolved through the basedOn chain
/// then docDefaults, with OOXML implicit defaults materialized ("0", "0",
/// "240", "auto") so two stylesheets always compare attr-by-attr.
///
/// In a table, `table_style`'s pPr sits between the two, as Word applies
/// it: over the default paragraph style's chain, under any other style's
/// chain (b4cd671041: a Table Grid cell's Normal paragraph shows the
/// table's `after=0 line=240` over Normal's 200/276; a List Paragraph
/// based on that Normal shows 200/276).
///
/// Also returns, per attr, whether B declares it in the style chain or its
/// docDefaults, and whether the table style supplied it.
fn effective_para_spacing(
    dom: &Dom,
    styles_root: NodeId,
    by_id: &std::collections::HashMap<String, NodeId>,
    style_id: &str,
    table_style: Option<&str>,
) -> ([String; 4], [bool; 4], [bool; 4]) {
    let attr_names = ["after", "before", "line", "lineRule"];
    let chain_vals = |start: Option<NodeId>| -> [Option<String>; 4] {
        let mut vals: [Option<String>; 4] = [None, None, None, None];
        let mut cur = start;
        for _ in 0..12 {
            let Some(s) = cur else { break };
            if let Some(ppr) = dom.element(s, &W::p_pr())
                && let Some(sp) = dom.element(ppr, &W::name("spacing"))
            {
                for (i, n) in attr_names.iter().enumerate() {
                    if vals[i].is_none()
                        && let Some(v) = dom.attribute(sp, &W::name(n))
                    {
                        vals[i] = Some(v.to_string());
                    }
                }
            }
            cur = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))
                .and_then(|v| by_id.get(v).copied());
        }
        vals
    };
    let style = by_id.get(style_id).copied();
    let para = chain_vals(style);
    let table = chain_vals(table_style.and_then(|t| by_id.get(t).copied()));
    let mut dd: [Option<String>; 4] = [None, None, None, None];
    if let Some(d) = dom.element(styles_root, &W::name("docDefaults"))
        && let Some(pd) = dom.element(d, &W::name("pPrDefault"))
        && let Some(ppr) = dom.element(pd, &W::p_pr())
        && let Some(sp) = dom.element(ppr, &W::name("spacing"))
    {
        for (i, n) in attr_names.iter().enumerate() {
            dd[i] = dom.attribute(sp, &W::name(n)).map(str::to_string);
        }
    }
    let is_default = style.is_some_and(|s| {
        dom.attribute(s, &W::name("type")) == Some("paragraph")
            && matches!(dom.attribute(s, &W::name("default")), Some("1" | "true"))
    });
    // provenance: declared = SOMEWHERE in B's style chain or dd.
    // Word bakes an attr onto inserted paragraphs ONLY when B is entirely
    // silent on it — the paragraph's look is the OOXML implicit default and
    // the output's dd would override it (rstyle_combos: implicit 0/240 IS
    // baked) — or when B's table style gave it (b4cd671041). Values B
    // declares — even in its dd — are otherwise never baked (m370:
    // dd-declared after=200/line=276 stays off the pure-I title).
    let defaults = ["0", "0", "240", "auto"];
    let mut vals: [String; 4] = std::array::from_fn(|i| defaults[i].to_string());
    let mut declared = [false; 4];
    let mut from_table = [false; 4];
    for i in 0..4 {
        let first = if is_default {
            table[i]
                .as_ref()
                .map(|v| (v, true))
                .or(para[i].as_ref().map(|v| (v, false)))
        } else {
            para[i]
                .as_ref()
                .map(|v| (v, false))
                .or(table[i].as_ref().map(|v| (v, true)))
        };
        if let Some((v, t)) = first.or(dd[i].as_ref().map(|v| (v, false))) {
            vals[i] = v.clone();
            from_table[i] = t;
            declared[i] = !t;
        }
    }
    (vals, declared, from_table)
}

/// Revision record element local names that carry a `w:id` identifying the
/// change, including list, move-range and table-cell records. Word treats a colliding id on any of these as the same revision
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
    "moveFromRangeStart",
    "moveFromRangeEnd",
    "moveToRangeStart",
    "moveToRangeEnd",
    "numberingChange",
    "cellIns",
    "cellDel",
    "cellMerge",
];

/// The next free revision id under `styles_root`: one greater than the maximum
/// numeric `w:id` on any revision record present (0 when none).
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

/// `w:pPr` on/off children whose absence reads as off, so "0" neutralizes them.
const PPR_ON_OFF: &[&str] = &[
    "keepNext",
    "keepLines",
    "pageBreakBefore",
    "widowControl",
    "suppressLineNumbers",
    "suppressAutoHyphens",
    "kinsoku",
    "wordWrap",
    "overflowPunct",
    "topLinePunct",
    "autoSpaceDE",
    "autoSpaceDN",
    "bidi",
    "adjustRightInd",
    "snapToGrid",
    "contextualSpacing",
    "mirrorIndents",
    "suppressOverlap",
];

/// `w:pPr` on/off children a document that never declares them reads as on
/// (Word's defaults): an original that turns one off in its docDefaults,
/// against a revision silent on it, gets it written on in Normal (38 of the
/// 470 Word redlines whose original alone declares one, 440c36d875).
const PPR_DEFAULT_ON: &[&str] = &["widowControl", "autoSpaceDE", "autoSpaceDN"];

/// An on/off element set to off.
fn on_off_is_off(dom: &Dom, el: NodeId) -> bool {
    matches!(dom.attribute(el, &W::val()), Some("0" | "false" | "off"))
}

/// What [`doc_default_ppr_delta`] writes into the live Normal pPr.
struct DocDefaultPprDelta {
    /// Whole pPr children, by local name.
    elements: Vec<(String, NodeId)>,
    /// Extra `w:spacing` attributes (`beforeLines`/`afterLines`).
    spacing: Vec<(String, String)>,
}

/// The docDefaults paragraph properties the after/before/line rule of
/// [`merge_normal_style_spacing`] does not cover, as Word writes them into the
/// live Normal. The redline keeps A's docDefaults, so B's own paragraph
/// defaults would be lost; Word stores B's effective value (B Normal, else
/// B docDefaults) wherever it differs from A's docDefaults, and neutralizes
/// what only A declares (file_103 × file_104 and the reverse file_104 ×
/// file_105: `ind`, `jc`, `beforeLines`/`afterLines`; sd_1494 ×
/// sdpr_titleonly: a `nil` edge per A border).
///
/// A property with no known neutral value is left alone.
fn doc_default_ppr_delta(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
    b_style: Option<NodeId>,
) -> DocDefaultPprDelta {
    let dd_ppr = |dom: &Dom, root: NodeId| -> Option<NodeId> {
        let dd = dom.element(root, &W::name("docDefaults"))?;
        let pd = dom.element(dd, &W::name("pPrDefault"))?;
        dom.element(pd, &W::p_pr())
    };
    let a_dd = dd_ppr(dom, out_root);
    let b_dd = dd_ppr(dom, b_root);
    let b_own = b_style.and_then(|s| dom.element(s, &W::p_pr()));
    let child = |dom: &Dom, ppr: Option<NodeId>, local: &str| {
        ppr.and_then(|p| dom.element(p, &W::name(local)))
    };
    let mut names: Vec<String> = Vec::new();
    for ppr in [a_dd, b_dd].into_iter().flatten() {
        for c in dom.elements(ppr, None) {
            let Some(n) = dom.name(c) else { continue };
            let local = n.local_name().to_string();
            if n.namespace_name() == W::URI
                && !matches!(local.as_str(), "spacing" | "rPr" | "pPrChange" | "sectPr")
                && !names.contains(&local)
            {
                names.push(local);
            }
        }
    }
    let mut elements = Vec::new();
    for local in names {
        let a = child(dom, a_dd, &local);
        let b = child(dom, b_own, &local).or_else(|| child(dom, b_dd, &local));
        if a.map(|n| style_prop_signature(dom, n)) == b.map(|n| style_prop_signature(dom, n)) {
            continue;
        }
        let el = match (b, a) {
            (Some(b), _) => dom.clone_subtree(b),
            (None, Some(a)) => {
                let el = dom.new_element(W::name(&local));
                match local.as_str() {
                    "ind" => {
                        for (n, _) in dom.attributes(a) {
                            if n.namespace_name() == W::URI {
                                dom.set_attribute_value(el, &n, Some("0"));
                            }
                        }
                    }
                    "jc" => {
                        dom.set_attribute_value(el, &W::val(), Some("left"));
                    }
                    // Word's default font alignment (440c36d875: baseline
                    // in the original's docDefaults, auto in Word's Normal).
                    "textAlignment" => {
                        dom.set_attribute_value(el, &W::val(), Some("auto"));
                    }
                    l if PPR_DEFAULT_ON.contains(&l) => {
                        if !on_off_is_off(dom, a) {
                            continue; // already the default
                        }
                    }
                    "pBdr" => {
                        for edge in dom.elements(a, None) {
                            let Some(n) = dom.name(edge) else { continue };
                            let e = dom.new_element(n);
                            dom.set_attribute_value(e, &W::val(), Some("nil"));
                            dom.add(el, e);
                        }
                    }
                    l if PPR_ON_OFF.contains(&l) => {
                        if on_off_is_off(dom, a) {
                            continue; // already the default
                        }
                        dom.set_attribute_value(el, &W::val(), Some("0"));
                    }
                    _ => continue,
                }
                el
            }
            (None, None) => continue,
        };
        elements.push((local, el));
    }
    let sp = |dom: &Dom, ppr: Option<NodeId>| child(dom, ppr, "spacing");
    let (a_sp, b_sp, b_own_sp) = (sp(dom, a_dd), sp(dom, b_dd), sp(dom, b_own));
    let mut attrs: Vec<(String, String)> = Vec::new();
    for s in [a_sp, b_sp].into_iter().flatten() {
        for (n, _) in dom.attributes(s) {
            let local = n.local_name();
            if n.namespace_name() == W::URI
                && matches!(local, "beforeLines" | "afterLines")
                && !attrs.iter().any(|(k, _)| k == local)
            {
                attrs.push((local.to_string(), String::new()));
            }
        }
    }
    let get = |dom: &Dom, s: Option<NodeId>, n: &str| {
        s.and_then(|s| dom.attribute(s, &W::name(n)))
            .map(str::to_string)
    };
    attrs.retain_mut(|(name, v)| {
        let b = get(dom, b_own_sp, name).or_else(|| get(dom, b_sp, name));
        if b == get(dom, a_sp, name) {
            return false;
        }
        *v = b.unwrap_or_else(|| "0".to_string());
        true
    });
    DocDefaultPprDelta {
        elements,
        spacing: attrs,
    }
}

/// M64/M70/M72: after the spacing merge, Normal's other declared pPr
/// children follow the revision — B's `w:ind` replaces A's, A-only children
/// drop into the pPrChange record, B's own children are copied in schema
/// order.
fn sync_normal_ppr_with_revised(dom: &mut Dom, ppr: NodeId, b_style: Option<NodeId>) {
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
    // M72 refined by file_198: Word's live Normal after the merge is B's
    // EFFECTIVE block — spacing (computed above) plus B's OWN declared
    // pPr children. A-origin non-spacing props (widowControl/tabs/
    // suppressAutoHyphens on file_77, where B declares none) drop into
    // pPrChange old; B-origin ones (the same names on file_198's
    // LO-flavored B Normal) SURVIVE live — dropping them loses widow
    // control and drifts pagination.
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
    // Copy B Normal's remaining declared pPr children (widowControl,
    // tabs, suppressAutoHyphens, …) in schema order.
    if let Some(bppr) = b_style.and_then(|bs| dom.element(bs, &W::name("pPr"))) {
        let skip = ["spacing", "ind", "pPrChange", "rPr"];
        let b_kids: Vec<NodeId> = dom.elements(bppr, None);
        for bc in b_kids {
            let Some(n) = dom.name(bc) else { continue };
            if skip.iter().any(|s| n == W::name(s)) {
                continue;
            }
            let local = n.local_name().to_string();
            if dom.element(ppr, &W::name(&local)).is_some() {
                continue;
            }
            let clone = dom.clone_subtree(bc);
            insert_child_by_rank(dom, ppr, clone, &local, &ppr_child_rank);
        }
    }
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
    let a_stored = stored(dom, Some(a_style));
    let b_style = find_normal_style(dom, b_root);
    let b_stored = stored(dom, b_style);
    let a_dd = dd_val(dom, out_root);
    let b_dd =
        b_docdefaults_ppr_spacing(dom, b_root).filter(|(a, l, _)| !a.is_empty() || !l.is_empty());
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

    // ── Word's Normal-spacing target (derived 2026-08-04) ────────────────────
    // The previous case cascade here (stored/dd/factory arms) was fit pair-by-
    // pair and both over- and under-fired: 44 of 760 corpus pairs carried the
    // wrong LIVE Normal spacing (e.g. basic_table_shading×basic_tracked_change:
    // Word live = none, ours = factory 160/278 → global vertical drift).
    // Rebuilding the truth table from every corpus Word oracle (zero ambiguous
    // input groups) gives one closed form, verified 751/752:
    //
    //   Word rewrites Normal so B's EFFECTIVE spacing survives under the
    //   OUTPUT stylesheet's (= A's) docDefaults:
    //     ctx(attr)   = A.docDefaults spacing attr, else app default
    //                   (after=0, line=240, lineRule=auto)
    //     b_eff(attr) = B Normal stored attr, else B.docDefaults attr,
    //                   else app default   — cascade is PER ATTRIBUTE
    //     target      = { attr where b_eff(attr) != ctx(attr) } over
    //                   {after, line}; when line is written, lineRule =
    //                   b_eff(lineRule) rides along; empty target ⇒ clear.
    //
    //   The bare+bare gate stays: Word never materializes dd into an
    //   untouched Normal even when docDefaults differ (file_19/145).
    //
    // The per-attribute cascade is what the old arms could not express: Word
    // mixes sources within one spacing element (B stored line=259 + B dd
    // after=160 → live after=160 line=259) and OMITS attrs already provided
    // by A's docDefaults (A dd line=276 == B line → live a0/l- only).
    // The old FACTORY_NORMAL_SPACING / EMPTY_B_SINGLE_LINE_NORMAL constants
    // fall out: 0/240 is just b_eff = app defaults under a non-default A dd,
    // and 160/278 was B's own dd all along on the pairs that motivated it.
    let raw_a_dd = docdefaults_ppr_spacing(dom, out_root);
    let raw_b_dd = b_docdefaults_ppr_spacing(dom, b_root);
    let raw_b_sp = b_style.and_then(|s| normal_spacing(dom, s));
    // M479 — `before` rides the same per-attribute cascade (the truth table
    // was derived over {after, line}; A-dd before=240 leaked live).
    let sp_node = |dom: &Dom, root: NodeId| -> Option<NodeId> {
        let dd = dom.element(root, &W::name("docDefaults"))?;
        let pd = dom.element(dd, &W::name("pPrDefault"))?;
        let pp = dom.element(pd, &W::p_pr())?;
        dom.element(pp, &W::name("spacing"))
    };
    let stored_sp_node = |dom: &Dom, style: Option<NodeId>| -> Option<NodeId> {
        let ppr = dom.element(style?, &W::name("pPr"))?;
        dom.element(ppr, &W::name("spacing"))
    };
    let ctx_before = spacing_before_attr(dom, sp_node(dom, out_root)).unwrap_or_else(|| "0".into());
    let b_eff_before = spacing_before_attr(dom, stored_sp_node(dom, b_style))
        .or_else(|| spacing_before_attr(dom, sp_node(dom, b_root)))
        .unwrap_or_else(|| "0".into());
    let target_before: String = if b_eff_before != ctx_before {
        b_eff_before
    } else {
        String::new()
    };
    let pick = |raw: &Option<(String, String, String)>, i: usize| -> Option<String> {
        raw.as_ref().and_then(|t| {
            let v = match i {
                0 => &t.0,
                1 => &t.1,
                _ => &t.2,
            };
            (!v.is_empty()).then(|| v.clone())
        })
    };
    const APP_DEFAULT: [&str; 3] = ["0", "240", "auto"];
    let ctx = |i: usize| pick(&raw_a_dd, i).unwrap_or_else(|| APP_DEFAULT[i].to_string());
    let b_eff = |i: usize| {
        pick(&raw_b_sp, i)
            .or_else(|| pick(&raw_b_dd, i))
            .unwrap_or_else(|| APP_DEFAULT[i].to_string())
    };
    let b_target: Option<(String, String, String)> =
        if !a_structured && !b_structured && a_stored.is_none() && b_stored.is_none() {
            // both Normals bare: Word leaves the style untouched (m106 requires a
            // structured B, so it cannot land here)
            return false;
        } else {
            let after = if b_eff(0) != ctx(0) {
                b_eff(0)
            } else {
                String::new()
            };
            let line = if b_eff(1) != ctx(1) {
                b_eff(1)
            } else {
                String::new()
            };
            let rule = if line.is_empty() {
                String::new()
            } else {
                b_eff(2)
            };
            if after.is_empty() && line.is_empty() {
                None
            } else {
                Some((after, line, rule))
            }
        };
    let DocDefaultPprDelta {
        elements: dd_elements,
        spacing: dd_spacing,
    } = doc_default_ppr_delta(dom, out_root, b_root, b_style);
    let dd_delta = !dd_elements.is_empty() || !dd_spacing.is_empty();
    // Normal's other declared paragraph properties follow B as well (M72):
    // b42b3ae070's revision keeps the spacing but drops `jc=both`, and
    // Word's redline records the justification in a pPrChange.
    let rest_signature = |dom: &Dom, style: Option<NodeId>| -> Vec<String> {
        let skip = ["spacing", "pPrChange", "rPr"];
        let mut sig: Vec<String> = style
            .and_then(|s| dom.element(s, &W::name("pPr")))
            .map(|p| dom.elements(p, None))
            .unwrap_or_default()
            .into_iter()
            .filter(|&c| {
                dom.name(c)
                    .is_some_and(|n| !skip.iter().any(|k| n == W::name(k)))
            })
            .map(|c| style_prop_signature(dom, c))
            .collect();
        sig.sort();
        sig
    };
    let rest_differs = rest_signature(dom, Some(a_style)) != rest_signature(dom, b_style);
    // Identity: A already has the same explicit spacing we would write.
    if target_before.is_empty()
        && !dd_delta
        && !rest_differs
        && let (Some(a), Some(b)) = (&a_stored, &b_target)
        && a == b
    {
        return false;
    }
    // Clearing when A already has no stored spacing is a no-op — except M106,
    // where Word still tracks dd spacing in pPrChange next to rPrChange —
    // and except a live `before` delta (M479), which must be written even
    // when after/line need nothing.
    if b_target.is_none()
        && a_stored.is_none()
        && !m106_same_dd_clear
        && target_before.is_empty()
        && !dd_delta
        && !rest_differs
    {
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
        // No after-materialization here: the delta rule already writes an
        // explicit "0" when B's effective after must override A's docDefaults
        // (file_196), and Word OMITS after when the context already supplies
        // it (12 corpus oracles carry line=... with no after attribute).
        let after = after.as_str();
        let set = |dom: &mut Dom, name: &str, v: &str| {
            dom.set_attribute_value(
                spacing,
                &W::name(name),
                if v.is_empty() { None } else { Some(v) },
            );
        };
        set(dom, "before", &target_before);
        set(dom, "after", after);
        set(dom, "line", line);
        set(dom, "lineRule", rule);
        sync_normal_ppr_with_revised(dom, ppr, b_style);
    } else if !target_before.is_empty() {
        // M479 — before-only delta: write the lone neutralizer, clearing any
        // stale after/line (paragraph_spacing_missing × pci_table oracle:
        // Normal live spacing = before="0" only).
        let spacing = match dom.element(ppr, &W::name("spacing")) {
            Some(s) => s,
            None => {
                let s = dom.new_element(W::name("spacing"));
                dom.add_first(ppr, s);
                s
            }
        };
        for a in ["after", "line", "lineRule"] {
            dom.set_attribute_value(spacing, &W::name(a), None);
        }
        dom.set_attribute_value(spacing, &W::name("before"), Some(&target_before));
    } else if let Some(sp) = dom.element(ppr, &W::name("spacing")) {
        // Clear explicit spacing — Word leaves empty pPr (file_22).
        dom.remove(sp);
    }
    if b_target.is_none() && rest_differs {
        sync_normal_ppr_with_revised(dom, ppr, b_style);
    }
    if !dd_spacing.is_empty() {
        let spacing = match dom.element(ppr, &W::name("spacing")) {
            Some(s) => s,
            None => {
                let s = dom.new_element(W::name("spacing"));
                insert_child_by_rank(dom, ppr, s, "spacing", &ppr_child_rank);
                s
            }
        };
        for (name, v) in &dd_spacing {
            dom.set_attribute_value(spacing, &W::name(name), Some(v));
        }
    }
    for (local, el) in dd_elements {
        if let Some(old) = dom.element(ppr, &W::name(&local)) {
            dom.remove(old);
        }
        insert_child_by_rank(dom, ppr, el, &local, &ppr_child_rank);
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
                // The style's own properties replace Normal's: Word records
                // Balloon Text's Tahoma 8 pt (a67dcf9e05), and Reject All
                // must give it back.
                for own in dom.elements(rpr, None) {
                    let Some(name) = dom.name(own) else { continue };
                    if name == W::name("rPrChange") {
                        continue;
                    }
                    let copy = dom.clone_subtree(own);
                    if let Some(stale) = dom.element(clone, &name) {
                        // Per font slot: a slot the style leaves open keeps
                        // Normal's old face (eastAsiaTheme, 0800162a66).
                        if name == W::name("rFonts") {
                            complete_attributes(dom, copy, stale, "rFonts");
                        }
                        dom.remove(stale);
                    }
                    add_rpr_child_in_order(dom, clone, copy, name.local_name());
                }
                dom.add(chg, clone);
                dom.add(rpr, chg);
                changed = true;
            }
        }
    }
    changed
}

// ---------------------------------------------------------------------------
// Workstream S — style-chain resolution.
// ---------------------------------------------------------------------------

/// A `w:basedOn` chain longer than this is treated as malformed and truncated.
/// Word's own limit on style inheritance depth is well below it; the cap only
/// exists so a pathological stylesheet cannot make resolution quadratic.
const MAX_STYLE_CHAIN_DEPTH: usize = 32;

/// `w:style` child order (wml.xsd `CT_Style` sequence). A `w:pPr` or `w:rPr`
/// materialized on a table style must land before `w:tblPr`/`w:trPr`/`w:tcPr`,
/// not at the end — appending produced `TableGrid` with `tblPr` before `rPr`,
/// which the OOXML validator rejects.
const STYLE_CHILD_ORDER: &[&str] = &[
    "name",
    "aliases",
    "basedOn",
    "next",
    "link",
    "autoRedefine",
    "hidden",
    "uiPriority",
    "semiHidden",
    "unhideWhenUsed",
    "qFormat",
    "locked",
    "personal",
    "personalCompose",
    "personalReply",
    "rsid",
    "pPr",
    "rPr",
    "tblPr",
    "trPr",
    "tcPr",
    "tblStylePr",
];

/// Insert `child` under `parent` at the position `order` gives its `local`
/// name: immediately before the first existing child that sorts after it, else
/// appended. Unknown names sort last, so they never displace a known one.
fn insert_child_by_rank(
    dom: &mut Dom,
    parent: NodeId,
    child: NodeId,
    local: &str,
    rank_of: &dyn Fn(&str) -> usize,
) {
    let new_rank = rank_of(local);
    let successor = dom.elements(parent, None).into_iter().find(|&e| {
        dom.name(e)
            .is_some_and(|n| rank_of(n.local_name()) > new_rank)
    });
    match successor {
        Some(s) => dom.add_before_self(s, child),
        None => dom.add(parent, child),
    }
}

/// Rank of a `w:style` child in [`STYLE_CHILD_ORDER`].
fn style_child_rank(local: &str) -> usize {
    STYLE_CHILD_ORDER
        .iter()
        .position(|&n| n == local)
        .unwrap_or(usize::MAX)
}

/// Rank of a `w:pPr` child in [`crate::comparer::order_tables::PPR_ORDER`].
fn ppr_child_rank(local: &str) -> usize {
    crate::comparer::order_tables::PPR_ORDER
        .iter()
        .find(|(n, _)| *n == local)
        .map(|(_, r)| *r as usize)
        .unwrap_or(usize::MAX)
}

/// M492: deleted paragraphs keep their A-ORIGINAL direct spacing.
/// Word preserves the source paragraph's own w:spacing on del-marked
/// paragraphs (file_22 × file_23 oracle: 31 deleted paras carry their
/// line=240/atLeast declarations; we stripped all but one — each
/// stripped para renders at the taller docDefaults line and the
/// accumulated height drifts the pagination, the long-standing
/// 115-vs-116-page mystery). Finalize passes can't see provenance, so
/// restore here from A's package: any del-marked paragraph whose
/// A-source para (matched by w14:paraId) declared w:spacing gets the
/// original attributes back when the output paragraph lost them.
///
/// Only `w:` attributes are restored: A's working copy carries the
/// comparer's `pt14:Unid` stamps, which must not come back as `w:Unid`.
/// Returns the rewritten `out_xml`, or `None` when nothing changed.
/// Word's redline font table is the union of both documents' tables. Keeping
/// only the original's leaves fonts that arrive with the revised text without
/// their charset/panose/family entry, and Word substitutes blind (file_46 ×
/// file_47: Times New Roman where Word's own redline used Hiragino Mincho for
/// Liberation Serif / Droid Sans Fallback). B's fonts the output does not name
/// are appended; embedded-font children stay behind, since their
/// relationships and obfuscation keys belong to B's package.
/// Returns the rewritten `out_xml`, or `None` when nothing changed.
fn merge_revised_font_table(out_xml: &str, b_xml: &str) -> Option<String> {
    let font = W::name("font");
    let name = W::name("name");
    let mut dom = Dom::new();
    let od = dom.parse_xdocument(out_xml);
    let bd = dom.parse_xdocument(b_xml);
    let (out_root, b_root) = (dom.root(od)?, dom.root(bd)?);
    let known: std::collections::HashSet<String> = dom
        .elements(out_root, Some(&font))
        .into_iter()
        .filter_map(|f| dom.attribute(f, &name).map(str::to_string))
        .collect();
    let mut changed = false;
    for f in dom.elements(b_root, Some(&font)) {
        if dom.attribute(f, &name).is_none_or(|n| known.contains(n)) {
            continue;
        }
        let copy = dom.clone_subtree(f);
        for c in dom.elements(copy, None) {
            if dom.name(c).is_some_and(|n| {
                n.namespace_name() == W::URI && n.local_name().starts_with("embed")
            }) {
                dom.remove(c);
            }
        }
        dom.add(out_root, copy);
        changed = true;
    }
    changed.then(|| dom.serialize_element(out_root))
}

fn restore_deleted_paragraph_spacing(a_xml: &str, out_xml: &str) -> Option<String> {
    let w14_pid = crate::namespaces::W14::name("paraId");
    let mut a_spacing: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    let mut ad = Dom::new();
    let d = ad.parse_xdocument(a_xml);
    let r = ad.root(d)?;
    for pnode in ad.descendants(r, Some(&W::p())) {
        let Some(pid) = ad.attribute(pnode, &w14_pid).map(str::to_string) else {
            continue;
        };
        if let Some(ppr) = ad.element(pnode, &W::p_pr())
            && let Some(sp) = ad.element(ppr, &W::name("spacing"))
        {
            let attrs: Vec<(String, String)> = ad
                .attributes(sp)
                .into_iter()
                .filter(|(n, _)| n.namespace_name() == W::URI)
                .map(|(n, v)| (n.local_name().to_string(), v))
                .collect();
            if !attrs.is_empty() {
                a_spacing.insert(pid, attrs);
            }
        }
    }
    if a_spacing.is_empty() {
        return None;
    }
    let mut pd = Dom::new();
    let d = pd.parse_xdocument(out_xml);
    let root = pd.root(d)?;
    let mut changed = false;
    for pnode in pd.descendants(root, Some(&W::p())) {
        let Some(pid) = pd.attribute(pnode, &w14_pid).map(str::to_string) else {
            continue;
        };
        let Some(attrs) = a_spacing.get(&pid) else {
            continue;
        };
        let Some(ppr) = pd.element(pnode, &W::p_pr()) else {
            continue;
        };
        let mark_del = pd
            .element(ppr, &W::r_pr())
            .is_some_and(|r| pd.element(r, &W::name("del")).is_some());
        if !mark_del || pd.element(ppr, &W::name("spacing")).is_some() {
            continue;
        }
        let sp = pd.new_element(W::name("spacing"));
        for (n, v) in attrs {
            pd.set_attribute_value(sp, &W::name(n), Some(v));
        }
        insert_child_by_rank(&mut pd, ppr, sp, "spacing", &ppr_child_rank);
        changed = true;
    }
    changed.then(|| pd.serialize_element(root))
}

/// Children of a style's `w:pPr`/`w:rPr` that are not formatting: revision
/// records, section properties, and revision-save ids.
fn is_style_prop_noise(name: &crate::xmllinq::XName) -> bool {
    matches!(
        name.local_name(),
        "pPrChange" | "rPrChange" | "sectPr" | "sectPrChange" | "rsid"
    )
}

/// Order-independent signature of one formatting element: expanded name, its
/// non-`rsid` attributes sorted by name, then its children's signatures in
/// document order (child order carries meaning inside `w:tabs`, `w:pBdr`, …).
fn style_prop_signature(dom: &Dom, node: NodeId) -> String {
    let Some(n) = dom.name(node) else {
        return String::new();
    };
    let mut out = n.clark();
    let mut attrs: Vec<(String, String)> = dom
        .attributes(node)
        .into_iter()
        .filter(|(a, _)| {
            !a.local_name().to_ascii_lowercase().starts_with("rsid")
                && !dom.is_namespace_declaration(a)
        })
        .map(|(a, v)| (a.clark(), v))
        .collect();
    attrs.sort();
    for (a, v) in attrs {
        out.push('\u{1}');
        out.push_str(&a);
        out.push('=');
        out.push_str(&v);
    }
    for c in dom.elements(node, None) {
        out.push('\u{2}');
        out.push_str(&style_prop_signature(dom, c));
    }
    out
}

/// Signature of a style's DECLARED `w:pPr`/`w:rPr` block: the formatting
/// children it writes itself, ignoring everything it inherits. Empty string
/// when the block is absent or holds only noise.
fn declared_props_signature(dom: &Dom, style: NodeId, local: &str) -> String {
    let Some(block) = dom.element(style, &W::name(local)) else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    for c in dom.elements(block, None) {
        let Some(n) = dom.name(c) else { continue };
        if is_style_prop_noise(&n) {
            continue;
        }
        parts.push(style_prop_signature(dom, c));
    }
    parts.sort();
    parts.join("\u{3}")
}

/// Property elements whose ATTRIBUTES are independent inherited values: a link
/// in the chain that sets some of them leaves the rest inherited instead of
/// replacing the element wholesale.
///
/// `w:rFonts` is the one that decides real documents. Oracle evidence
/// (`Hello_docx_world × multi_image_types`): A's `Heading3Char` writes
/// `asciiTheme="minorHAnsi" hAnsiTheme="minorHAnsi"` and B's omits both, and
/// Word records **no change** on any of `Heading3Char`..`Heading9Char` —
/// because omitting them inherits exactly `minorHAnsi` from `docDefaults`.
/// Comparing the element whole marks all seven as changed. `w:lang` behaves the
/// same way (`val` / `eastAsia` / `bidi` are separate slots).
///
/// The same per-attribute rule is already relied on by
/// [`effective_normal_rpr_metrics`] for the footer-metric cascade.
const ATTR_MERGED_PROPS: &[&str] = &["rFonts", "lang"];

/// `w:rFonts` attribute pairs that are alternatives for one typeface slot:
/// naming either clears the other, so a nearer link that switches a slot to an
/// explicit face must not leave the farther link's theme reference standing.
const RFONTS_ALTERNATIVE_SLOTS: [[&str; 2]; 4] = [
    ["ascii", "asciiTheme"],
    ["hAnsi", "hAnsiTheme"],
    ["eastAsia", "eastAsiaTheme"],
    ["cs", "cstheme"],
];

/// A style's EFFECTIVE `w:pPr`/`w:rPr`, resolved the way Word resolves a style
/// chain: `w:docDefaults` first, then each `w:basedOn` ancestor from the root of
/// the chain downwards, then the style's own declared properties. Keyed by
/// element name so a nearer link overrides a farther one, except for
/// [`ATTR_MERGED_PROPS`], which merge attribute-by-attribute.
fn effective_style_props(
    dom: &Dom,
    styles_root: NodeId,
    by_id: &std::collections::HashMap<String, NodeId>,
    style: NodeId,
    local: &str,
    default_local: &str,
) -> std::collections::BTreeMap<String, String> {
    // Walk basedOn to the root of the chain, guarding against cycles.
    let mut chain: Vec<NodeId> = Vec::new();
    let mut seen: std::collections::HashSet<NodeId> = std::collections::HashSet::new();
    let mut cur = Some(style);
    while let Some(s) = cur {
        if !seen.insert(s) || chain.len() >= MAX_STYLE_CHAIN_DEPTH {
            break;
        }
        chain.push(s);
        cur = dom
            .element(s, &W::name("basedOn"))
            .and_then(|b| dom.attribute(b, &W::val()))
            .and_then(|v| by_id.get(v).copied());
    }
    chain.reverse();

    type Props = std::collections::BTreeMap<String, String>;
    type Slots = std::collections::BTreeMap<String, Props>;
    let mut props: Props = Props::new();
    let mut slots: Slots = Slots::new();

    let absorb = |props: &mut Props, slots: &mut Slots, block: NodeId| {
        for c in dom.elements(block, None) {
            let Some(n) = dom.name(c) else { continue };
            if is_style_prop_noise(&n) {
                continue;
            }
            if !ATTR_MERGED_PROPS.contains(&n.local_name()) {
                props.insert(n.clark(), style_prop_signature(dom, c));
                continue;
            }
            let slot = slots.entry(n.clark()).or_default();
            for (a, v) in dom.attributes(c) {
                if a.local_name().to_ascii_lowercase().starts_with("rsid")
                    || dom.is_namespace_declaration(&a)
                {
                    continue;
                }
                if n.local_name() == "rFonts" {
                    for pair in RFONTS_ALTERNATIVE_SLOTS {
                        if pair.contains(&a.local_name()) {
                            // Slots are keyed by EXPANDED name; the alternative
                            // shares this attribute's namespace.
                            for other in pair {
                                slot.remove(&a.namespace().name(other).clark());
                            }
                        }
                    }
                }
                slot.insert(a.clark(), v);
            }
        }
    };
    if let Some(dd) = dom
        .element(styles_root, &W::name("docDefaults"))
        .and_then(|d| dom.element(d, &W::name(default_local)))
        .and_then(|d| dom.element(d, &W::name(local)))
    {
        absorb(&mut props, &mut slots, dd);
    }
    for s in chain {
        if let Some(block) = dom.element(s, &W::name(local)) {
            absorb(&mut props, &mut slots, block);
        }
    }
    for (name, attrs) in slots {
        let sig = attrs
            .into_iter()
            .map(|(a, v)| format!("{a}={v}"))
            .collect::<Vec<_>>()
            .join("\u{1}");
        props.insert(name, sig);
    }
    props
}

/// The key a style is matched across the two stylesheets by: `(type, name)`,
/// falling back to the styleId when the style carries no `w:name`.
///
/// Matching on the NAME rather than the id is what makes this survive
/// [`canonicalize_style_ids`], which has already rewritten the output's ids
/// (`SD_StrikeChar` → `SDStrikeChar`) while the revised package still holds the
/// originals. The canonical id is a pure function of the name, so equal names
/// are exactly the styles that canonicalization would have unified.
/// Detached copies of the styles of `out_root` whose type and name `b_root`
/// does not define, each with the key that finds it again.
fn styles_the_revision_lacks(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
) -> Vec<((String, String), NodeId)> {
    let b_keys: std::collections::HashSet<(String, String)> = dom
        .elements(b_root, Some(&W::name("style")))
        .into_iter()
        .filter_map(|s| style_match_key(dom, s))
        .collect();
    let only_a: Vec<((String, String), NodeId)> = dom
        .elements(out_root, Some(&W::name("style")))
        .into_iter()
        .filter_map(|s| style_match_key(dom, s).map(|k| (k, s)))
        .filter(|(k, _)| !b_keys.contains(k))
        .collect();
    only_a
        .into_iter()
        .map(|(k, s)| (k, dom.clone_subtree(s)))
        .collect()
}

/// Put each saved style back in place of the one its key finds.
fn restore_styles(
    dom: &mut Dom,
    styles_root: NodeId,
    saved: Vec<((String, String), NodeId)>,
) -> bool {
    let mut changed = false;
    for (key, copy) in saved {
        let Some(live) = dom
            .elements(styles_root, Some(&W::name("style")))
            .into_iter()
            .find(|&s| style_match_key(dom, s).as_ref() == Some(&key))
        else {
            continue;
        };
        if dom.serialize_element(live) != dom.serialize_element(copy) {
            dom.replace_with(live, &[copy]);
            changed = true;
        }
    }
    changed
}

fn style_match_key(dom: &Dom, style: NodeId) -> Option<(String, String)> {
    let ty = dom
        .attribute(style, &W::name("type"))
        .unwrap_or("paragraph")
        .to_string();
    let key = dom
        .element(style, &W::name("name"))
        .and_then(|n| dom.attribute(n, &W::val()))
        .map(|v| v.to_ascii_lowercase())
        .or_else(|| {
            dom.attribute(style, &W::name("styleId"))
                .map(|v| v.to_ascii_lowercase())
        })?;
    Some((ty, key))
}

/// Word writes a root style's recorded old properties out in full against the
/// docDefaults (c719b900f0, f1257ca7ea, 221577c35b, 4eff11f045): the old
/// block holds every property the docDefaults set, at the docDefaults value
/// where the original's style leaves it to them, with `w:rFonts`, `w:lang`
/// and `w:spacing` completed attribute by attribute (`w:lang
/// w:val="es-ES"` gains the default `eastAsia`/`bidi`, `w:spacing
/// w:after="0"` the default `line`). A root style with one record gets the
/// other too. Word's Reject All of a record that leaves a property out
/// writes a value of its own (sz=20 and a Times New Roman complex script
/// over the original's 11pt Arial, the live line pitch over the default),
/// so the complete record is what rejects to the original. Only paragraph
/// styles without `w:basedOn` qualify: below them the fallback is the parent
/// style.
fn complete_root_style_change_records(
    dom: &mut Dom,
    styles_root: NodeId,
    settings: &WmlComparerSettings,
) -> bool {
    let docdefaults = dom.element(styles_root, &W::name("docDefaults"));
    let defaults = |dom: &Dom, default_local: &str, block_local: &str| {
        docdefaults
            .and_then(|d| dom.element(d, &W::name(default_local)))
            .and_then(|d| dom.element(d, &W::name(block_local)))
            .filter(|&b| !dom.elements(b, None).is_empty())
    };
    let blocks = [
        ("pPr", "pPrDefault", "pPrChange"),
        ("rPr", "rPrDefault", "rPrChange"),
    ];
    let record = |dom: &Dom, style: NodeId, block_local: &str, change_local: &str| {
        dom.element(style, &W::name(block_local))
            .and_then(|b| dom.element(b, &W::name(change_local)))
    };
    let mut changed = false;
    for style in dom.elements(styles_root, Some(&W::name("style"))) {
        if dom
            .attribute(style, &W::name("type"))
            .unwrap_or("paragraph")
            != "paragraph"
            || dom.element(style, &W::name("basedOn")).is_some()
            || blocks
                .iter()
                .all(|(b, _, c)| record(dom, style, b, c).is_none())
        {
            continue;
        }
        for (block_local, default_local, change_local) in blocks {
            let Some(defaults) = defaults(dom, default_local, block_local) else {
                continue;
            };
            let old = match record(dom, style, block_local, change_local)
                .and_then(|c| dom.element(c, &W::name(block_local)))
            {
                Some(old) => old,
                None => {
                    let block = match dom.element(style, &W::name(block_local)) {
                        Some(block) => block,
                        None => {
                            let block = dom.new_element(W::name(block_local));
                            add_child_in_rank_order(dom, style, block, style_child_rank);
                            block
                        }
                    };
                    let old = dom.new_element(W::name(block_local));
                    for c in dom.elements(block, None) {
                        let copy = dom.clone_subtree(c);
                        dom.add(old, copy);
                    }
                    append_style_change_record(
                        dom,
                        styles_root,
                        block,
                        old,
                        change_local,
                        settings,
                    );
                    old
                }
            };
            changed |= complete_from_defaults(dom, old, defaults, block_local);
        }
    }
    changed
}

/// Complete the old records of each paragraph style with a `w:basedOn` from
/// what the original's docDefaults and chain gave the same style (matched by
/// type and name), so Reject All gives back the original's look rather than
/// Word's built-ins: see
/// [`crate::revision_processor::style_records::complete_from_original_chain`].
/// Styles the original lacks keep their records as written.
fn complete_based_style_change_records(dom: &mut Dom, styles_root: NodeId, a_root: NodeId) -> bool {
    let a_by_key: std::collections::HashMap<(String, String), NodeId> = dom
        .elements(a_root, Some(&W::name("style")))
        .into_iter()
        .filter_map(|s| Some((style_match_key(dom, s)?, s)))
        .collect();
    // The original's styles each block of which the redline records.
    let mut recorded: std::collections::HashMap<&str, std::collections::HashSet<NodeId>> =
        std::collections::HashMap::new();
    for style in dom.elements(styles_root, Some(&W::name("style"))) {
        let Some(&a_style) = style_match_key(dom, style).and_then(|k| a_by_key.get(&k)) else {
            continue;
        };
        for (block_local, change_local) in [("pPr", "pPrChange"), ("rPr", "rPrChange")] {
            if dom
                .element(style, &W::name(block_local))
                .and_then(|b| dom.element(b, &W::name(change_local)))
                .is_some()
            {
                recorded.entry(block_local).or_default().insert(a_style);
            }
        }
    }
    let mut changed = false;
    for style in dom.elements(styles_root, Some(&W::name("style"))) {
        if dom
            .attribute(style, &W::name("type"))
            .unwrap_or("paragraph")
            != "paragraph"
            || dom.element(style, &W::name("basedOn")).is_none()
        {
            continue;
        }
        let Some(&a_style) = style_match_key(dom, style).and_then(|k| a_by_key.get(&k)) else {
            continue;
        };
        for (block_local, change_local) in [("pPr", "pPrChange"), ("rPr", "rPrChange")] {
            let restored = recorded.get(block_local);
            let is_recorded = |s: NodeId| restored.is_some_and(|r| r.contains(&s));
            let Some(old) = dom
                .element(style, &W::name(block_local))
                .and_then(|b| dom.element(b, &W::name(change_local)))
                .and_then(|c| dom.element(c, &W::name(block_local)))
            else {
                continue;
            };
            changed |= crate::revision_processor::style_records::complete_from_original_chain(
                dom,
                old,
                a_root,
                a_style,
                block_local,
                &is_recorded,
            );
        }
    }
    changed
}

/// The properties whose recorded old value Word completes attribute by
/// attribute from the docDefaults.
const RECORD_COMPLETED_PROPS: &[&str] = &["rFonts", "lang", "spacing"];

/// `w:spacing` attributes that are alternatives for one value: a recorded
/// `w:before` must not gain the default `w:beforeAutospacing`, which wins.
const SPACING_ALTERNATIVE_SLOTS: [[&str; 3]; 2] = [
    ["before", "beforeAutospacing", "beforeLines"],
    ["after", "afterAutospacing", "afterLines"],
];

/// Give the recorded `old` block every property of the docDefaults `defaults`
/// block it lacks, and complete its [`RECORD_COMPLETED_PROPS`].
fn complete_from_defaults(dom: &mut Dom, old: NodeId, defaults: NodeId, block_local: &str) -> bool {
    let mut changed = false;
    for default in dom.elements(defaults, None) {
        let Some(name) = dom.name(default) else {
            continue;
        };
        if is_style_prop_noise(&name) {
            continue;
        }
        let local = name.local_name().to_string();
        match dom.element(old, &name) {
            Some(recorded) if RECORD_COMPLETED_PROPS.contains(&local.as_str()) => {
                changed |= complete_attributes(dom, recorded, default, &local);
            }
            Some(_) => {}
            None => {
                let copy = dom.clone_subtree(default);
                if block_local == "rPr" {
                    add_rpr_child_in_order(dom, old, copy, &local);
                } else {
                    add_child_in_rank_order(dom, old, copy, ppr_child_rank);
                }
                changed = true;
            }
        }
    }
    changed
}

/// Give `recorded` each attribute of `default` it lacks, unless it names an
/// alternative for that slot (an explicit `w:ascii` over `w:asciiTheme`).
fn complete_attributes(dom: &mut Dom, recorded: NodeId, default: NodeId, local: &str) -> bool {
    let slots: Vec<&[&str]> = match local {
        "rFonts" => RFONTS_ALTERNATIVE_SLOTS
            .iter()
            .map(|p| p.as_slice())
            .collect(),
        "spacing" => SPACING_ALTERNATIVE_SLOTS
            .iter()
            .map(|p| p.as_slice())
            .collect(),
        _ => Vec::new(),
    };
    let mut changed = false;
    for (attr, value) in dom.attributes(default) {
        if dom.is_namespace_declaration(&attr) || dom.attribute(recorded, &attr).is_some() {
            continue;
        }
        let taken = slots
            .iter()
            .filter(|slot| slot.contains(&attr.local_name()))
            .flat_map(|slot| slot.iter())
            .any(|other| {
                dom.attribute(recorded, &attr.namespace().name(other))
                    .is_some()
            });
        if !taken {
            dom.set_attribute_value(recorded, &attr, Some(&value));
            changed = true;
        }
    }
    changed
}

/// Insert `child` under `parent` after the last existing child that `rank`
/// puts before it, or first.
fn add_child_in_rank_order(dom: &mut Dom, parent: NodeId, child: NodeId, rank: fn(&str) -> usize) {
    let own = dom.name(child).map_or(usize::MAX, |n| rank(n.local_name()));
    let anchor = dom
        .elements(parent, None)
        .into_iter()
        .rfind(|&e| dom.name(e).is_some_and(|n| rank(n.local_name()) < own));
    match anchor {
        Some(a) => dom.add_after_self(a, child),
        None => dom.add_first(parent, child),
    }
}

/// Wrap `old` (a `w:pPr`/`w:rPr` clone) in a `w:pPrChange`/`w:rPrChange` record
/// and append it to `block`, which is where CT_PPr / CT_RPr put it.
fn append_style_change_record(
    dom: &mut Dom,
    styles_root: NodeId,
    block: NodeId,
    old: NodeId,
    change_local: &str,
    settings: &WmlComparerSettings,
) {
    let chg = dom.new_element(W::name(change_local));
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
    dom.add(chg, old);
    dom.add(block, chg);
}

/// Workstream S — adopt the REVISED document's definition of every style both
/// documents define, recording the ORIGINAL definition on the `w:style` itself.
///
/// Word's Compare stylesheet resolves the style chain on both sides and, where
/// a style resolves differently, writes B's declared properties live with A's
/// inside a `w:pPrChange` / `w:rPrChange` on the `w:style` element. Oracle
/// evidence (`two_column_two_page × vrect_node`): 18 styles marked, `Title`
/// live at B's `sz=56` with A's `sz=52 color=17365D` in `w:rPrChange` and A's
/// `pBdr` + `after=300` in `w:pPrChange`. Over the 564 corpus pairs that have a
/// Word oracle, 52.5% of the oracle stylesheets carry style-level change
/// markup.
///
/// [`crate::comparer::footnotes::copy_missing_styles`] is keyed on
/// `(type, styleId)` and skips an id already present whatever its body, so
/// before this pass the output kept **A's** definition and recorded nothing:
/// content on both sides rendered with the original's fonts, sizes and borders
/// and the change was invisible. 136 of 597 corpus pairs carry at least one
/// such live collision; they score a mean 59.7 against 79.4 for the pairs
/// without one.
///
/// Two guards keep this from manufacturing empty change records:
/// - the EFFECTIVE properties must differ (so a stylesheet reaching the same
///   result through `basedOn` is not marked), and
/// - the DECLARED properties must differ (so a difference that actually lives
///   in an ancestor is recorded on that ancestor, once, and not restated on
///   every style below it).
///
/// `Normal` is excluded — it is owned by [`merge_normal_style_spacing`] /
/// [`merge_normal_style_rpr`], whose rules are calibrated against a separate
/// body of oracle evidence. Styles already carrying change markup are left
/// alone, which is also what lets [`cascade_normal_change_to_based_styles`]
/// (M111) keep acting as the fallback for styles this pass does not touch.
fn merge_revised_style_definitions(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
    settings: &WmlComparerSettings,
    a_declared_keys: &std::collections::HashSet<(String, String)>,
) -> bool {
    let style_nm = W::name("style");
    let style_id = W::name("styleId");
    let normal_out = find_normal_style(dom, out_root);
    let normal_b = find_normal_style(dom, b_root);

    let index_by_id = |dom: &Dom, root: NodeId| -> std::collections::HashMap<String, NodeId> {
        dom.elements(root, Some(&style_nm))
            .into_iter()
            .filter_map(|s| Some((dom.attribute(s, &style_id)?.to_string(), s)))
            .collect()
    };
    let out_by_id = index_by_id(dom, out_root);
    let b_by_id = index_by_id(dom, b_root);

    let mut b_by_key: std::collections::HashMap<(String, String), NodeId> =
        std::collections::HashMap::new();
    for s in dom.elements(b_root, Some(&style_nm)) {
        if let Some(k) = style_match_key(dom, s) {
            b_by_key.entry(k).or_insert(s);
        }
    }

    let mut changed = false;
    for style in dom.elements(out_root, Some(&style_nm)) {
        if Some(style) == normal_out {
            continue;
        }
        let Some(key) = style_match_key(dom, style) else {
            continue;
        };
        let Some(&b_style) = b_by_key.get(&key) else {
            continue;
        };
        if Some(b_style) == normal_b {
            continue;
        }
        for (local, default_local, change_local) in [
            ("pPr", "pPrDefault", "pPrChange"),
            ("rPr", "rPrDefault", "rPrChange"),
        ] {
            let a_declared = declared_props_signature(dom, style, local);
            let b_declared = declared_props_signature(dom, b_style, local);
            if a_declared == b_declared {
                continue;
            }
            let a_eff =
                effective_style_props(dom, out_root, &out_by_id, style, local, default_local);
            let b_eff = effective_style_props(dom, b_root, &b_by_id, b_style, local, default_local);
            if a_eff == b_eff {
                continue;
            }
            // Already tracked (an inbound stylesheet with pending redline, or an
            // earlier pass) — do not stack a second record on the same block.
            if dom
                .element(style, &W::name(local))
                .is_some_and(|blk| dom.element(blk, &W::name(change_local)).is_some())
            {
                continue;
            }

            // Old = A's declared block, change history stripped: the inner pPr
            // of a pPrChange is CT_PPrBase and may not itself carry one.
            let old = match dom.element(style, &W::name(local)) {
                Some(blk) => {
                    let clone = dom.clone_subtree(blk);
                    for c in dom.descendants(clone, Some(&W::name(change_local))) {
                        dom.remove(c);
                    }
                    clone
                }
                None => dom.new_element(W::name(local)),
            };

            // Live = B's declared block. Materialize A's when absent, keeping
            // CT_Style order (pPr precedes rPr).
            let block = match dom.element(style, &W::name(local)) {
                Some(blk) => {
                    let stale: Vec<NodeId> = dom
                        .elements(blk, None)
                        .into_iter()
                        .filter(|&c| dom.name(c).is_none_or(|n| !is_style_prop_noise(&n)))
                        .collect();
                    for c in stale {
                        dom.remove(c);
                    }
                    blk
                }
                None => {
                    let blk = dom.new_element(W::name(local));
                    insert_child_by_rank(dom, style, blk, local, &style_child_rank);
                    blk
                }
            };
            if let Some(b_block) = dom.element(b_style, &W::name(local)) {
                for c in dom.elements(b_block, None) {
                    let Some(n) = dom.name(c) else { continue };
                    if is_style_prop_noise(&n) {
                        continue;
                    }
                    let clone = dom.clone_subtree(c);
                    dom.add(block, clone);
                }
            }
            append_style_change_record(dom, out_root, block, old, change_local, settings);
            changed = true;
        }
    }

    // Phase 2 (S2) — styles COPIED from B (absent from A's stylesheet) bake
    // B's docDefaults-level run metrics and carry style-level change records.
    //
    // Oracle (image_inline_and_block × rtl_page_numpages): A's stylesheet
    // never declares Footer/FootnoteText/Strong1. The output stylesheet keeps
    // A's docDefaults (theme fonts, sz 24, kern 2, ligatures
    // standardContextual), so a verbatim copy of B's style renders with A's
    // metrics — the R2 cluster's cumulative vertical drift. Word declares the
    // neutralizing delta live on each copied style (rFonts TNR, sz 20, szCs
    // 20, kern 0, w14:ligatures none — implicit defaults materialized: sz 20,
    // kern 0, ligatures none) and marks it with `w:rPrChange` (old = declared
    // + lang) and `w:pPrChange` (old = declared pPr).
    {
        let dd_rpr = |dom: &Dom, root: NodeId| -> Option<NodeId> {
            let dd = dom.element(root, &W::name("docDefaults"))?;
            let rd = dom.element(dd, &W::name("rPrDefault"))?;
            dom.element(rd, &W::r_pr())
        };
        let a_dd = dd_rpr(dom, out_root);
        let b_dd = dd_rpr(dom, b_root);
        let attr_map =
            |dom: &Dom, e: Option<NodeId>| -> std::collections::BTreeMap<String, String> {
                e.map(|e| {
                    dom.attributes(e)
                        .into_iter()
                        .map(|(n, v)| (format!("{n:?}"), v))
                        .collect()
                })
                .unwrap_or_default()
            };
        let val_or = |dom: &Dom, dd: Option<NodeId>, local: &str, default: &str| -> String {
            dd.and_then(|d| dom.element(d, &W::name(local)))
                .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
                .unwrap_or_else(|| default.to_string())
        };
        let lig_name = W14::name("ligatures");
        let lig_val = W14::name("val");
        let lig_or = |dom: &Dom, dd: Option<NodeId>| -> String {
            dd.and_then(|d| dom.element(d, &lig_name))
                .and_then(|e| dom.attribute(e, &lig_val).map(str::to_string))
                .unwrap_or_else(|| "none".to_string())
        };
        let a_fonts = a_dd.and_then(|d| dom.element(d, &W::name("rFonts")));
        let b_fonts = b_dd.and_then(|d| dom.element(d, &W::name("rFonts")));
        let fonts_differ = attr_map(dom, a_fonts) != attr_map(dom, b_fonts);
        // NOTE (M476 falsified, M477 evidence): an ascii-family master gate
        // ("same family ⇒ copy verbatim, no bake") recovered tab_test ×
        // table_autofit (+12.8) but broke NINE same-family pairs whose
        // oracles DO bake (nested_comments × math_matrix −35, doc_with_graphs
        // −26, pagination_blank −18.6, …): Word's rule is per-attribute, not
        // per-family — bake exactly the attrs the style's B-side declaration
        // CHAIN does not itself provide (the b_chain_has gates below).
        // M464 — pPr spacing deltas between the two docDefaults (implicit
        // defaults before/after 0, line 240). A B-only style resolving a
        // spacing attr through B's dd renders wrong under A's dd (file_13 ×
        // file_14 oracle bakes after=0 line=240 on every copied style).
        let dd_ppr_spacing = |dom: &Dom, root: NodeId| -> Option<NodeId> {
            let dd = dom.element(root, &W::name("docDefaults"))?;
            let pd = dom.element(dd, &W::name("pPrDefault"))?;
            let pp = dom.element(pd, &W::p_pr())?;
            dom.element(pp, &W::name("spacing"))
        };
        let a_sp = dd_ppr_spacing(dom, out_root);
        let b_sp = dd_ppr_spacing(dom, b_root);
        let sp_val = |dom: &Dom, sp: Option<NodeId>, attr: &str, default: &str| -> String {
            sp.and_then(|s| dom.attribute(s, &W::name(attr)).map(str::to_string))
                .unwrap_or_else(|| default.to_string())
        };
        let mut sp_deltas: Vec<(&str, String)> = Vec::new();
        for (attr, default) in [("before", "0"), ("after", "0"), ("line", "240")] {
            let av = sp_val(dom, a_sp, attr, default);
            let bv = sp_val(dom, b_sp, attr, default);
            if av != bv {
                sp_deltas.push((attr, bv));
            }
        }
        let b_line_rule =
            b_sp.and_then(|s| dom.attribute(s, &W::name("lineRule")).map(str::to_string));
        // Output-stylesheet basedOn chains, for "does the declared chain
        // already provide this attr" checks.
        let out_by_id: std::collections::HashMap<String, NodeId> = dom
            .elements(out_root, Some(&style_nm))
            .into_iter()
            .filter_map(|s| dom.attribute(s, &style_id).map(|i| (i.to_string(), s)))
            .collect();
        let chain_has_sp_attr = |dom: &Dom, start: NodeId, attr: &str| -> bool {
            let mut s = start;
            for _ in 0..12 {
                if dom
                    .element(s, &W::p_pr())
                    .and_then(|p| dom.element(p, &W::name("spacing")))
                    .and_then(|sp| dom.attribute(sp, &W::name(attr)))
                    .is_some()
                {
                    return true;
                }
                let Some(based) = dom
                    .element(s, &W::name("basedOn"))
                    .and_then(|b| dom.attribute(b, &W::val()))
                else {
                    return false;
                };
                match out_by_id.get(based) {
                    Some(&n) => s = n,
                    None => return false,
                }
            }
            false
        };
        // Chain-aware rPr check (file_198 × file_199): a copied style whose
        // look is declared somewhere on its B-SIDE basedOn chain (Liberation
        // Serif on B's Normal, later promoted into the output) must not get
        // dd values baked over it — Word renders the chain's declaration.
        // Walk B's chain, not the output's: at bake time the output Normal
        // is still A's (promotion runs later). Mirrors chain_has_sp_attr.
        let b_by_id_for_chain: std::collections::HashMap<String, NodeId> = dom
            .elements(b_root, Some(&style_nm))
            .into_iter()
            .filter_map(|s| dom.attribute(s, &style_id).map(|i| (i.to_string(), s)))
            .collect();
        // M479 — same B-side rule for SPACING attrs: at bake time the output
        // Normal has not yet been promoted, so an out-chain check misses
        // spacing B's chain provides (pci_table Heading7-9 inherit B-Normal's
        // stored line=259; the bake stamped B-dd line=278 over it).
        let b_chain_has_sp_attr = |dom: &Dom, start: NodeId, attr: &str| -> bool {
            let mut s = start;
            for _ in 0..12 {
                if dom
                    .element(s, &W::p_pr())
                    .and_then(|p| dom.element(p, &W::name("spacing")))
                    .and_then(|sp| dom.attribute(sp, &W::name(attr)))
                    .is_some()
                {
                    return true;
                }
                let Some(based) = dom
                    .element(s, &W::name("basedOn"))
                    .and_then(|b| dom.attribute(b, &W::val()))
                else {
                    return false;
                };
                match b_by_id_for_chain.get(based) {
                    Some(&n) => s = n,
                    None => return false,
                }
            }
            false
        };
        let b_chain_has_rpr_elem =
            |dom: &Dom, start: NodeId, name: &crate::xmllinq::XName| -> bool {
                let mut s = start;
                for _ in 0..12 {
                    if dom
                        .element(s, &W::r_pr())
                        .and_then(|r| dom.element(r, name))
                        .is_some()
                    {
                        return true;
                    }
                    let Some(based) = dom
                        .element(s, &W::name("basedOn"))
                        .and_then(|b| dom.attribute(b, &W::val()))
                    else {
                        return false;
                    };
                    match b_by_id_for_chain.get(based) {
                        Some(&n) => s = n,
                        None => return false,
                    }
                }
                false
            };
        // (local, B-effective value) for whitelist props whose A/B effective
        // values differ; implicit defaults sz/szCs 20, kern 0.
        let mut deltas: Vec<(&str, String)> = Vec::new();
        for (local, default) in [("kern", "0"), ("sz", "20"), ("szCs", "20")] {
            let av = val_or(dom, a_dd, local, default);
            let bv = val_or(dom, b_dd, local, default);
            if av != bv {
                deltas.push((local, bv));
            }
        }
        let ligs = (lig_or(dom, a_dd), lig_or(dom, b_dd));
        let b_lang = b_dd.or(a_dd).and_then(|d| dom.element(d, &W::name("lang")));
        if fonts_differ || !deltas.is_empty() || ligs.0 != ligs.1 || !sp_deltas.is_empty() {
            for style in dom.elements(out_root, Some(&style_nm)) {
                // Styles pair by (type, name), never id: the revision's
                // header may be `Encabezado` where the output says `Header`
                // (cda19d51ed), the original's List Paragraph `a4`.
                let Some(key) = style_match_key(dom, style) else {
                    continue;
                };
                if a_declared_keys.contains(&key) {
                    continue;
                }
                let Some(&b_side) = b_by_key.get(&key) else {
                    continue;
                };
                if Some(style) == normal_out {
                    continue;
                }
                // Paragraph styles only — Word leaves the linked *Char
                // styles (FooterChar, Hyperlink, …) unmarked in the oracle.
                let stype = dom
                    .attribute(style, &W::name("type"))
                    .unwrap_or("")
                    .to_string();
                if stype != "paragraph" {
                    continue;
                }
                // Already tracked — leave alone (same rule as phase 1).
                let already = ["rPrChange", "pPrChange"]
                    .iter()
                    .any(|c| !dom.descendants(style, Some(&W::name(c))).is_empty());
                if already {
                    continue;
                }
                // rPr: old = declared + lang; live = declared + delta.
                let declared = dom.element(style, &W::r_pr());
                let old = match declared {
                    Some(r) => dom.clone_subtree(r),
                    None => dom.new_element(W::r_pr()),
                };
                if dom.element(old, &W::name("lang")).is_none()
                    && let Some(l) = b_lang
                {
                    let lc = dom.clone_subtree(l);
                    dom.add(old, lc);
                }
                let live = match declared {
                    Some(r) => r,
                    None => {
                        let r = dom.new_element(W::r_pr());
                        insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                        r
                    }
                };
                let b_side = Some(b_side);
                let b_chain_has = |dom: &Dom, name: &crate::xmllinq::XName| -> bool {
                    b_side.is_some_and(|bs| b_chain_has_rpr_elem(dom, bs, name))
                };
                if fonts_differ
                    && dom.element(live, &W::name("rFonts")).is_none()
                    && !b_chain_has(dom, &W::name("rFonts"))
                    && let Some(bf) = b_fonts
                {
                    let fc = dom.clone_subtree(bf);
                    match dom.elements(live, None).first().copied() {
                        Some(first) => dom.add_before_self(first, fc),
                        None => dom.add(live, fc),
                    }
                }
                for (local, bv) in &deltas {
                    if dom.element(live, &W::name(local)).is_none()
                        && !b_chain_has(dom, &W::name(local))
                    {
                        let e = dom.new_element(W::name(local));
                        dom.set_attribute_value(e, &W::val(), Some(bv));
                        dom.add(live, e);
                    }
                }
                if ligs.0 != ligs.1
                    && dom.element(live, &lig_name).is_none()
                    && !b_chain_has(dom, &lig_name)
                {
                    let e = dom.new_element(lig_name.clone());
                    dom.set_attribute_value(e, &lig_val, Some(&ligs.1));
                    dom.add(live, e);
                }
                append_style_change_record(dom, out_root, live, old, "rPrChange", settings);
                // pPr (paragraph styles): bake B-effective spacing deltas
                // (M464), then record; old = post-bake clone (Word old==live,
                // tiff BodyText oracle).
                if stype == "paragraph" {
                    let ppr = match dom.element(style, &W::p_pr()) {
                        Some(p) => p,
                        None => {
                            let p = dom.new_element(W::p_pr());
                            insert_child_by_rank(dom, style, p, "pPr", &style_child_rank);
                            p
                        }
                    };
                    for (attr, bv) in &sp_deltas {
                        if chain_has_sp_attr(dom, style, attr)
                            || b_side.is_some_and(|bs| b_chain_has_sp_attr(dom, bs, attr))
                        {
                            continue;
                        }
                        let sp = match dom.element(ppr, &W::name("spacing")) {
                            Some(s) => s,
                            None => {
                                let s = dom.new_element(W::name("spacing"));
                                insert_child_by_rank(dom, ppr, s, "spacing", &ppr_child_rank);
                                s
                            }
                        };
                        if dom.attribute(sp, &W::name(attr)).is_none() {
                            dom.set_attribute_value(sp, &W::name(attr), Some(bv));
                            if *attr == "line" && dom.attribute(sp, &W::name("lineRule")).is_none()
                            {
                                let rule = b_line_rule.as_deref().unwrap_or("auto");
                                dom.set_attribute_value(sp, &W::name("lineRule"), Some(rule));
                            }
                        }
                    }
                    let old_p = dom.clone_subtree(ppr);
                    for c in dom.descendants(old_p, Some(&W::name("pPrChange"))) {
                        dom.remove(c);
                    }
                    append_style_change_record(dom, out_root, ppr, old_p, "pPrChange", settings);
                }
                changed = true;
            }
        }
    }
    changed
}

/// M462 — Word's factory docDefaults (current blank-document scaffold).
/// Used when A has no styles part: the output keeps FACTORY defaults (not
/// B's) and every copied B style bakes its B-effective metrics so it still
/// renders as it did in B (tiff_image × two_column oracle).
const FACTORY_DD_RPR: &str = r#"<w:rPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:rFonts w:asciiTheme="minorHAnsi" w:eastAsiaTheme="minorEastAsia" w:hAnsiTheme="minorHAnsi" w:cstheme="minorBidi"/><w:kern w:val="2"/><w:sz w:val="24"/><w:szCs w:val="24"/><w:lang w:val="en-US" w:eastAsia="en-US" w:bidi="ar-SA"/><w14:ligatures w14:val="standardContextual"/></w:rPr>"#;
const FACTORY_DD_PPR: &str = r#"<w:pPr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:spacing w:after="160" w:line="278" w:lineRule="auto"/></w:pPr>"#;

/// M462 — A has no styles part: rewrite the adopted B stylesheet so its
/// docDefaults become Word's FACTORY defaults, baking each style's
/// B-effective metrics into the style itself. Word renders A's bare
/// paragraphs at factory metrics (sz=24 after=160 line=278) while B-styled
/// content keeps B's look via the per-style bake; adopting B's docDefaults
/// wholesale instead rendered A's content 15% tighter (17 pages vs the
/// oracle's 20 on tiff_image × two_column).
fn factory_scaffold_bake_b_styles(
    dom: &mut Dom,
    root: NodeId,
    settings: &WmlComparerSettings,
) -> bool {
    let style_nm = W::name("style");
    let style_id_nm = W::name("styleId");
    // --- capture B's docDefaults values ---
    let dd = dom.element(root, &W::name("docDefaults"));
    let b_ppr_spacing = dd
        .and_then(|d| dom.element(d, &W::name("pPrDefault")))
        .and_then(|p| dom.element(p, &W::p_pr()))
        .and_then(|p| dom.element(p, &W::name("spacing")));
    let b_sp = |dom: &Dom, attr: &str| -> Option<String> {
        b_ppr_spacing.and_then(|s| dom.attribute(s, &W::name(attr)).map(str::to_string))
    };
    let b_before = b_sp(dom, "before");
    let b_after = b_sp(dom, "after");
    let b_line = b_sp(dom, "line");
    let b_line_rule = b_sp(dom, "lineRule");
    let b_dd_rpr = dd
        .and_then(|d| dom.element(d, &W::name("rPrDefault")))
        .and_then(|r| dom.element(r, &W::r_pr()));
    let b_rv = |dom: &Dom, local: &str| -> Option<String> {
        b_dd_rpr
            .and_then(|r| dom.element(r, &W::name(local)))
            .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
    };
    let b_kern = b_rv(dom, "kern");
    let b_sz = b_rv(dom, "sz");
    let b_sz_cs = b_rv(dom, "szCs");
    let lig_name = W14::name("ligatures");
    let lig_val = W14::name("val");
    let b_lig = b_dd_rpr
        .and_then(|r| dom.element(r, &lig_name))
        .and_then(|e| dom.attribute(e, &lig_val).map(str::to_string));
    let b_fonts_attrs: std::collections::BTreeMap<String, String> = b_dd_rpr
        .and_then(|r| dom.element(r, &W::name("rFonts")))
        .map(|f| {
            dom.attributes(f)
                .into_iter()
                .map(|(n, v)| (format!("{n:?}"), v))
                .collect()
        })
        .unwrap_or_default();
    let b_fonts_node = b_dd_rpr.and_then(|r| dom.element(r, &W::name("rFonts")));
    let b_fonts_clone = b_fonts_node.map(|f| dom.clone_subtree(f));

    // --- replace docDefaults content with factory ---
    if let Some(dd) = dd {
        let rd = dom.element(dd, &W::name("rPrDefault"));
        let pd = dom.element(dd, &W::name("pPrDefault"));
        for e in [rd, pd].into_iter().flatten() {
            dom.remove(e);
        }
        let rp_doc = dom.parse_xdocument(FACTORY_DD_RPR);
        let pp_doc = dom.parse_xdocument(FACTORY_DD_PPR);
        let rd_new = dom.new_element(W::name("rPrDefault"));
        if let Some(r) = dom.root(rp_doc) {
            dom.add(rd_new, r);
        }
        let pd_new = dom.new_element(W::name("pPrDefault"));
        if let Some(p) = dom.root(pp_doc) {
            dom.add(pd_new, p);
        }
        dom.add_first(dd, pd_new);
        dom.add_first(dd, rd_new);
    }
    let factory_fonts: std::collections::BTreeMap<String, String> = [
        ("asciiTheme", "minorHAnsi"),
        ("eastAsiaTheme", "minorEastAsia"),
        ("hAnsiTheme", "minorHAnsi"),
        ("cstheme", "minorBidi"),
    ]
    .iter()
    .map(|(k, v)| (format!("{:?}", W::name(k)), v.to_string()))
    .collect();

    // --- per-style bake ---
    let styles: Vec<NodeId> = dom.elements(root, Some(&style_nm));
    let by_id: std::collections::HashMap<String, NodeId> = styles
        .iter()
        .filter_map(|&s| dom.attribute(s, &style_id_nm).map(|i| (i.to_string(), s)))
        .collect();
    // Value of a declared spacing attr / rPr child anywhere in the basedOn
    // chain (self first). Cycles guarded by a depth cap.
    let chain_spacing = |dom: &Dom, mut s: NodeId, attr: &str| -> Option<String> {
        for _ in 0..12 {
            if let Some(v) = dom
                .element(s, &W::p_pr())
                .and_then(|p| dom.element(p, &W::name("spacing")))
                .and_then(|sp| dom.attribute(sp, &W::name(attr)))
            {
                return Some(v.to_string());
            }
            let based = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))?;
            s = *by_id.get(based)?;
        }
        None
    };
    let chain_rpr = |dom: &Dom, mut s: NodeId, local: &str| -> Option<String> {
        for _ in 0..12 {
            if let Some(v) = dom
                .element(s, &W::r_pr())
                .and_then(|r| dom.element(r, &W::name(local)))
                .and_then(|e| dom.attribute(e, &W::val()))
            {
                return Some(v.to_string());
            }
            let based = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))?;
            s = *by_id.get(based)?;
        }
        None
    };
    let chain_has_rfonts = |dom: &Dom, mut s: NodeId| -> bool {
        for _ in 0..12 {
            if dom
                .element(s, &W::r_pr())
                .and_then(|r| dom.element(r, &W::name("rFonts")))
                .is_some()
            {
                return true;
            }
            let Some(based) = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))
            else {
                return false;
            };
            match by_id.get(based) {
                Some(&n) => s = n,
                None => return false,
            }
        }
        false
    };
    let chain_lig = |dom: &Dom, mut s: NodeId| -> Option<String> {
        for _ in 0..12 {
            if let Some(v) = dom
                .element(s, &W::r_pr())
                .and_then(|r| dom.element(r, &lig_name))
                .and_then(|e| dom.attribute(e, &lig_val))
            {
                return Some(v.to_string());
            }
            let based = dom
                .element(s, &W::name("basedOn"))
                .and_then(|b| dom.attribute(b, &W::val()))?;
            s = *by_id.get(based)?;
        }
        None
    };

    let mut changed = false;
    for style in styles {
        let sid = dom.attribute(style, &style_id_nm).unwrap_or("").to_string();
        let stype = dom.attribute(style, &W::name("type")).unwrap_or("");
        let is_para = stype == "paragraph";
        let is_char = stype == "character";
        if (!is_para && !is_char) || sid == "Normal" {
            continue;
        }
        let q_format = dom.element(style, &W::name("qFormat")).is_some();
        let old_ppr = dom.element(style, &W::p_pr()).map(|p| {
            let c = dom.clone_subtree(p);
            for ch in dom.descendants(c, Some(&W::name("pPrChange"))) {
                dom.remove(ch);
            }
            c
        });
        let old_rpr = dom.element(style, &W::r_pr()).map(|r| {
            let c = dom.clone_subtree(r);
            for ch in dom.descendants(c, Some(&W::name("rPrChange"))) {
                dom.remove(ch);
            }
            c
        });
        let mut touched_ppr = false;
        let mut touched_rpr = false;

        if is_para {
            // spacing: before/after default "0", line default "240".
            let deltas: Vec<(&str, String)> = [
                ("before", &b_before, "0"),
                ("after", &b_after, "160"),
                ("line", &b_line, "278"),
            ]
            .into_iter()
            .filter_map(|(attr, b_dd_v, factory)| {
                if chain_spacing(dom, style, attr).is_some() {
                    return None; // declared chain wins in both contexts
                }
                let default = if attr == "line" { "240" } else { "0" };
                let eff_b = b_dd_v.clone().unwrap_or_else(|| default.to_string());
                let eff_out = factory.to_string();
                (eff_b != eff_out).then_some((attr, eff_b))
            })
            .collect();
            if !deltas.is_empty() {
                let ppr = match dom.element(style, &W::p_pr()) {
                    Some(p) => p,
                    None => {
                        let p = dom.new_element(W::p_pr());
                        insert_child_by_rank(dom, style, p, "pPr", &style_child_rank);
                        p
                    }
                };
                let sp = match dom.element(ppr, &W::name("spacing")) {
                    Some(s) => s,
                    None => {
                        let s = dom.new_element(W::name("spacing"));
                        insert_child_by_rank(dom, ppr, s, "spacing", &ppr_child_rank);
                        s
                    }
                };
                for (attr, v) in &deltas {
                    dom.set_attribute_value(sp, &W::name(attr), Some(v));
                    if *attr == "line" && dom.attribute(sp, &W::name("lineRule")).is_none() {
                        let rule = b_line_rule.clone().unwrap_or_else(|| "auto".to_string());
                        dom.set_attribute_value(sp, &W::name("lineRule"), Some(&rule));
                    }
                }
                touched_ppr = true;
            }
        }

        // rPr metric bake (paragraph AND character styles).
        let mut rpr_deltas: Vec<(&str, String)> = Vec::new();
        for (local, b_dd_v, factory, default) in [
            ("kern", &b_kern, "2", "0"),
            ("sz", &b_sz, "24", "20"),
            ("szCs", &b_sz_cs, "24", "20"),
        ] {
            if chain_rpr(dom, style, local).is_some() {
                continue;
            }
            let eff_b = b_dd_v.clone().unwrap_or_else(|| default.to_string());
            if eff_b != factory {
                rpr_deltas.push((local, eff_b));
            }
        }
        let want_fonts = !chain_has_rfonts(dom, style)
            && !b_fonts_attrs.is_empty()
            && b_fonts_attrs != factory_fonts;
        let want_lig = q_format
            && chain_lig(dom, style).is_none()
            && b_lig.clone().unwrap_or_else(|| "none".to_string()) != "standardContextual";
        if !rpr_deltas.is_empty() || want_fonts || want_lig {
            let rpr = match dom.element(style, &W::r_pr()) {
                Some(r) => r,
                None => {
                    let r = dom.new_element(W::r_pr());
                    insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                    r
                }
            };
            if want_fonts
                && dom.element(rpr, &W::name("rFonts")).is_none()
                && let Some(fc) = b_fonts_clone
            {
                let clone = dom.clone_subtree(fc);
                dom.add_first(rpr, clone);
            }
            for (local, v) in &rpr_deltas {
                let e = dom.new_element(W::name(local));
                dom.set_attribute_value(e, &W::val(), Some(v));
                add_rpr_child_in_order(dom, rpr, e, local);
            }
            if want_lig && dom.element(rpr, &lig_name).is_none() {
                let e = dom.new_element(lig_name.clone());
                let v = b_lig.clone().unwrap_or_else(|| "none".to_string());
                dom.set_attribute_value(e, &lig_val, Some(&v));
                dom.add(rpr, e);
            }
            touched_rpr = true;
        }

        // Word marks touched PARAGRAPH styles with both change records; the
        // linked *Char styles stay unmarked (S2 rule).
        if is_para && (touched_ppr || touched_rpr) {
            let already = ["rPrChange", "pPrChange"]
                .iter()
                .any(|c| !dom.descendants(style, Some(&W::name(c))).is_empty());
            if !already {
                let ppr = match dom.element(style, &W::p_pr()) {
                    Some(p) => p,
                    None => {
                        let p = dom.new_element(W::p_pr());
                        insert_child_by_rank(dom, style, p, "pPr", &style_child_rank);
                        p
                    }
                };
                let old_p = old_ppr.unwrap_or_else(|| dom.new_element(W::p_pr()));
                append_style_change_record(dom, root, ppr, old_p, "pPrChange", settings);
                let rpr = match dom.element(style, &W::r_pr()) {
                    Some(r) => r,
                    None => {
                        let r = dom.new_element(W::r_pr());
                        insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                        r
                    }
                };
                let old_r = old_rpr.unwrap_or_else(|| dom.new_element(W::r_pr()));
                append_style_change_record(dom, root, rpr, old_r, "rPrChange", settings);
            }
        }
        changed |= touched_ppr || touched_rpr;
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
    // M460: only Word's own single-line normalization (line=240) propagates to
    // headings; a Normal carrying B's non-240 line (e.g. 276, basic_comment ×
    // cli_legacy) leaves Heading/Title/ListParagraph line-less in the oracle.
    if dom.attribute(normal_sp, &W::name("line")) != Some("240") {
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
            // CT_PPrBase order, not "first" and not "just before pPrChange":
            // once workstream S puts B's declared `ind`/`contextualSpacing` on
            // ListParagraph, both of those land spacing after its successors.
            insert_child_by_rank(dom, ppr, sp, "spacing", &ppr_child_rank);
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
            let Some(rpr) = dom.element(style, &W::name("rPr")) else {
                continue;
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

/// Run-metric keys the footer merge resolves and compares: each rFonts slot
/// as its (explicit, theme) attribute pair, plus sz/szCs values (the
/// properties that set a footer line's box height). A slot's theme attribute
/// overrides its explicit one, so the two resolve together.
const RPR_METRIC_FONT_SLOTS: [(&str, &str); 4] = [
    ("ascii", "asciiTheme"),
    ("hAnsi", "hAnsiTheme"),
    ("eastAsia", "eastAsiaTheme"),
    ("cs", "cstheme"),
];

/// A style tree's `docDefaults/rPrDefault/rPr` node, if present.
/// The line pitch a paragraph with no spacing of its own resolves to under
/// `styles_xml`: the default paragraph style's chain, then docDefaults, then
/// Word's single line (240).
/// Whether a styles part defines a default table style (`TableNormal` in
/// Word-authored documents; generated documents often ship none).
fn has_default_table_style(styles_xml: &str) -> bool {
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(styles_xml);
    let Some(root) = dom.root(d) else {
        return false;
    };
    dom.elements(root, Some(&W::name("style")))
        .into_iter()
        .any(|s| {
            dom.attribute(s, &W::name("type")) == Some("table")
                && matches!(dom.attribute(s, &W::name("default")), Some("1" | "true"))
        })
}

fn default_paragraph_line(styles_xml: &str) -> String {
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(styles_xml);
    let Some(root) = dom.root(d) else {
        return "240".to_string();
    };
    let style_nm = W::name("style");
    let styles = dom.elements(root, Some(&style_nm));
    let by_id = |id: &str| {
        styles
            .iter()
            .copied()
            .find(|&s| dom.attribute(s, &W::name("styleId")) == Some(id))
    };
    let line_of = |ppr: Option<NodeId>| {
        ppr.and_then(|p| dom.element(p, &W::name("spacing")))
            .and_then(|sp| dom.attribute(sp, &W::name("line")))
            .map(str::to_string)
    };
    let mut cur = styles.iter().copied().find(|&s| {
        dom.attribute(s, &W::name("type")) == Some("paragraph")
            && matches!(dom.attribute(s, &W::name("default")), Some("1" | "true"))
    });
    for _ in 0..12 {
        let Some(s) = cur else { break };
        if let Some(line) = line_of(dom.element(s, &W::p_pr())) {
            return line;
        }
        cur = dom
            .element(s, &W::name("basedOn"))
            .and_then(|b| dom.attribute(b, &W::val()))
            .and_then(by_id);
    }
    let dd = dom
        .element(root, &W::name("docDefaults"))
        .and_then(|d| dom.element(d, &W::name("pPrDefault")))
        .and_then(|d| dom.element(d, &W::p_pr()));
    line_of(dd).unwrap_or_else(|| "240".to_string())
}

fn rpr_default(dom: &Dom, styles_root: NodeId) -> Option<NodeId> {
    let dd = dom.element(styles_root, &W::name("docDefaults"))?;
    let rd = dom.element(dd, &W::name("rPrDefault"))?;
    dom.element(rd, &W::name("rPr"))
}

/// Normal's EFFECTIVE run metrics: each value from the style's stored rPr
/// when present, else from docDefaults' rPrDefault (per-attribute, the way
/// Word resolves a style chain). Returns each rFonts slot of
/// [`RPR_METRIC_FONT_SLOTS`] as (explicit, theme), then [sz, szCs], each None
/// when defined nowhere. A slot resolves as a pair from the first source that
/// declares either attribute: a theme font (`w:asciiTheme="minorHAnsi"`, the
/// usual docDefaults form) is as much a declaration as a named one.
type FontSlot = (Option<String>, Option<String>);
fn effective_normal_rpr_metrics(
    dom: &Dom,
    styles_root: NodeId,
    normal: Option<NodeId>,
) -> ([FontSlot; 4], [Option<String>; 2]) {
    let stored = normal.and_then(|s| dom.element(s, &W::name("rPr")));
    let default = rpr_default(dom, styles_root);
    let font_slot = |(attr, theme): (&str, &str)| -> FontSlot {
        for src in [stored, default] {
            let Some(f) = src.and_then(|r| dom.element(r, &W::name("rFonts"))) else {
                continue;
            };
            let named = dom.attribute(f, &W::name(attr)).map(str::to_string);
            let themed = dom.attribute(f, &W::name(theme)).map(str::to_string);
            if named.is_some() || themed.is_some() {
                return (named, themed);
            }
        }
        (None, None)
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
    (
        RPR_METRIC_FONT_SLOTS.map(font_slot),
        [sz_val("sz"), sz_val("szCs")],
    )
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
/// Copy B's package chrome when the A-based package lacks it.
///
/// Settings/fontTable/webSettings are present on Word redlines whenever the
/// revised side carries them (C3). The theme is not: Word never takes B's
/// (see [`ensure_factory_package_chrome`]).
/// Full docDefaults swap regressed sales_report×sample_document — leave
/// docDefaults to the Normal merge path; only fill **missing** chrome parts.
fn adopt_revised_styles_chrome(out: &mut PartFs, pkg2: &PartFs, out_main: &str) {
    // people.xml: Word redlines always carry author identity when B has comments
    // (C2 residual layout for document_100×lots_of_comments).
    const CHROME: [(&str, &str, &str, &str); 4] = [
        (
            "word/settings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
            "settings.xml",
        ),
        (
            "word/webSettings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings",
            "webSettings.xml",
        ),
        (
            "word/fontTable.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml",
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable",
            "fontTable.xml",
        ),
        (
            "word/people.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.people+xml",
            "http://schemas.microsoft.com/office/2011/relationships/people",
            "people.xml",
        ),
    ];
    for (part, ctype, rel_type, target) in CHROME {
        if out.part_bytes(part).is_some() {
            continue;
        }
        let Some(bytes) = pkg2.part_bytes(part).map(<[u8]>::to_vec) else {
            continue;
        };
        out.set_part(part, bytes);
        out.add_content_type_override(&format!("/{part}"), ctype);
        let has_rel = out
            .read_rels_for(out_main)
            .is_some_and(|r| r.items.iter().any(|i| i.rel_type == rel_type));
        if !has_rel {
            out.add_document_relationship(out_main, rel_type, target);
        }
    }
}

/// Minimal Word-like package chrome for thin demo packages (C5).
///
/// Word Compare always saves `settings` / `theme` / `fontTable` even when both
/// inputs were bare (styles-only) demos. Without them LO falls back to factory
/// faces that diverge from Word's redline PDF (blue_bold / quarterly_heading
/// / right_aligned_italic class). Inject only when still missing after
/// [`adopt_revised_styles_chrome`].
fn ensure_factory_package_chrome(out: &mut PartFs, out_main: &str) {
    // settings
    if out.part_bytes("word/settings.xml").is_none() {
        const SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:settings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:zoom w:percent="100"/>
  <w:defaultTabStop w:val="720"/>
  <w:characterSpacingControl w:val="doNotCompress"/>
  <w:compat/>
</w:settings>"#;
        out.set_part("word/settings.xml", SETTINGS.as_bytes().to_vec());
        out.add_content_type_override(
            "/word/settings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml",
        );
        let has_rel = out
            .read_rels_for(out_main)
            .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/settings")));
        if !has_rel {
            out.add_document_relationship(
                out_main,
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
                "settings.xml",
            );
        }
    }
    // webSettings (Word always writes this; LO uses it for some wrap/compat)
    if out.part_bytes("word/webSettings.xml").is_none() {
        const WEB: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:webSettings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:optimizeForBrowser/>
  <w:allowPNG/>
</w:webSettings>"#;
        out.set_part("word/webSettings.xml", WEB.as_bytes().to_vec());
        out.add_content_type_override(
            "/word/webSettings.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml",
        );
        let has_rel = out
            .read_rels_for(out_main)
            .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/webSettings")));
        if !has_rel {
            out.add_document_relationship(
                out_main,
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings",
                "webSettings.xml",
            );
        }
    }
    // fontTable
    if out.part_bytes("word/fontTable.xml").is_none() {
        const FONTS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:fonts xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:font w:name="Calibri"><w:panose1 w:val="020F0502020204030204"/><w:charset w:val="00"/><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>
  <w:font w:name="Times New Roman"><w:panose1 w:val="02020603050405020304"/><w:charset w:val="00"/><w:family w:val="roman"/><w:pitch w:val="variable"/></w:font>
  <w:font w:name="Arial"><w:panose1 w:val="020B0604020202020204"/><w:charset w:val="00"/><w:family w:val="swiss"/><w:pitch w:val="variable"/></w:font>
</w:fonts>"#;
        out.set_part("word/fontTable.xml", FONTS.as_bytes().to_vec());
        out.add_content_type_override(
            "/word/fontTable.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.fontTable+xml",
        );
        let has_rel = out
            .read_rels_for(out_main)
            .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/fontTable")));
        if !has_rel {
            out.add_document_relationship(
                out_main,
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable",
                "fontTable.xml",
            );
        }
    }
    // theme: Word gives an original without a referenced theme its own
    // default theme, never the revision's (an unreferenced theme part does
    // not count).
    if referenced_theme_part(out, out_main).is_none() {
        const THEME_PART: &str = "word/theme/theme1.xml";
        out.set_part(
            THEME_PART,
            crate::word_default_theme::WORD_DEFAULT_THEME
                .as_bytes()
                .to_vec(),
        );
        out.add_content_type_override(
            &format!("/{THEME_PART}"),
            "application/vnd.openxmlformats-officedocument.theme+xml",
        );
        let stale: Vec<String> = out
            .read_rels_for(out_main)
            .map(|r| {
                r.items
                    .iter()
                    .filter(|i| i.rel_type.ends_with("/relationships/theme"))
                    .map(|i| i.rel_type.clone())
                    .collect()
            })
            .unwrap_or_default();
        for rel_type in stale {
            out.remove_relationships_by_type(out_main, &rel_type);
        }
        let target = crate::opc::relative_rel_target(out_main, THEME_PART);
        out.add_document_relationship(out_main, THEME_REL, &target);
    }
}

const THEME_REL: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme";

/// The theme part `main` references, when the package holds it.
fn referenced_theme_part(out: &PartFs, main: &str) -> Option<String> {
    out.read_rels_for(main)?
        .items
        .iter()
        .filter(|r| r.rel_type.ends_with("/relationships/theme"))
        .map(|r| out.resolve_rel_target(main, &r.target))
        .find(|part| out.part_bytes(part).is_some())
}

/// Word-canonical `w:styleId` for a human-readable `w:name` (C3 / C5).
///
/// Word Compare rewrites numeric/`styleN` ids to ECMA-style ids (`heading 1` →
/// `Heading1`, `Normal Table` → `TableNormal`). LO resolves layout from the
/// id for many built-ins; leaving `w:styleId="2"` with `w:name="heading 1"`
/// keeps the correct face in Word but wrong metrics under LO (tolerated-input
/// demos score ~47 with matching body text).
fn word_canonical_style_id(name: &str) -> String {
    let n = name.trim();
    // Case-insensitive built-ins (ECMA-376 + Word Compare observations).
    let lower = n.to_ascii_lowercase();
    match lower.as_str() {
        "normal" => return "Normal".into(),
        "heading 1" => return "Heading1".into(),
        "heading 2" => return "Heading2".into(),
        "heading 3" => return "Heading3".into(),
        "heading 4" => return "Heading4".into(),
        "heading 5" => return "Heading5".into(),
        "heading 6" => return "Heading6".into(),
        "heading 7" => return "Heading7".into(),
        "heading 8" => return "Heading8".into(),
        "heading 9" => return "Heading9".into(),
        "default paragraph font" => return "DefaultParagraphFont".into(),
        "normal table" => return "TableNormal".into(),
        "no list" => return "NoList".into(),
        "list paragraph" => return "ListParagraph".into(),
        "footnote text" => return "FootnoteText".into(),
        "footnote reference" => return "FootnoteReference".into(),
        "endnote text" => return "EndnoteText".into(),
        "endnote reference" => return "EndnoteReference".into(),
        "title" => return "Title".into(),
        "subtitle" => return "Subtitle".into(),
        "hyperlink" => return "Hyperlink".into(),
        "strong" => return "Strong".into(),
        "emphasis" => return "Emphasis".into(),
        "quote" => return "Quote".into(),
        "intense quote" => return "IntenseQuote".into(),
        "caption" => return "Caption".into(),
        // Built-ins whose id is not their PascalCased name: the comment
        // styles above all (B's comments.xml keeps `CommentReference`, and
        // renaming it to `AnnotationReference` stranded every reference).
        "annotation text" => return "CommentText".into(),
        "annotation reference" => return "CommentReference".into(),
        "annotation subject" => return "CommentSubject".into(),
        "macro" => return "MacroText".into(),
        "toa heading" => return "TOAHeading".into(),
        "table of figures" => return "TableofFigures".into(),
        "table of authorities" => return "TableofAuthorities".into(),
        "text body" => return "Textbody".into(),
        "preformatted text" => return "PreformattedText".into(),
        "document title" => return "DocumentTitle".into(),
        "highlighted style" => return "HighlightedStyle".into(),
        "red bold character" => return "RedBoldCharacter".into(),
        "blue italic character" => return "BlueItalicCharacter".into(),
        "heading 1 char" => return "Heading1Char".into(),
        "heading 2 char" => return "Heading2Char".into(),
        "heading 3 char" => return "Heading3Char".into(),
        "heading 4 char" => return "Heading4Char".into(),
        "heading 5 char" => return "Heading5Char".into(),
        "heading 6 char" => return "Heading6Char".into(),
        "title char" => return "TitleChar".into(),
        "footnote text char" => return "FootnoteTextChar".into(),
        "default" => return "Default".into(),
        "heading" => return "Heading".into(),
        "list" => return "List".into(),
        "index" => return "Index".into(),
        // Table-of-contents built-ins: styleId is the ALL-CAPS `TOC1`..`TOC9`
        // (name "toc 1"..), which the generic PascalCase below would mangle to
        // `Toc1`. That renames a live built-in to a custom id, so LibreOffice
        // (and Word) drop the built-in TOC indents/dot-leader tabs and the
        // whole table of contents reflows — tanking the visual redline score.
        "toc 1" => return "TOC1".into(),
        "toc 2" => return "TOC2".into(),
        "toc 3" => return "TOC3".into(),
        "toc 4" => return "TOC4".into(),
        "toc 5" => return "TOC5".into(),
        "toc 6" => return "TOC6".into(),
        "toc 7" => return "TOC7".into(),
        "toc 8" => return "TOC8".into(),
        "toc 9" => return "TOC9".into(),
        "toc heading" => return "TOCHeading".into(),
        _ => {}
    }
    // Generic: split on anything but letters and digits (Word ids hold no
    // spaces or punctuation: "Normal (Web)" is NormalWeb), PascalCase each token.
    n.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| {
            let mut cs = t.chars();
            match cs.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + cs.as_str(),
            }
        })
        .collect()
}

/// Word mode: copy the revised stylesheet's styles that the output lacks,
/// pairing styles by type and name as Word does, not by id. A B style whose
/// name the output already holds (Brazilian `Normal` against Dutch
/// `Standaard`, both named "Normal") is not copied; the returned map sends
/// its B id to the output style's id. Styles the names do not pair fall back
/// to the id rule of [`crate::comparer::footnotes::copy_missing_styles`].
fn copy_missing_styles_by_name(
    dom: &mut Dom,
    to_root: NodeId,
    from_root: NodeId,
) -> std::collections::HashMap<String, String> {
    let style_nm = W::name("style");
    let style_id = W::name("styleId");
    let mut by_name: std::collections::HashMap<(String, String), String> =
        std::collections::HashMap::new();
    for s in dom.elements(to_root, Some(&style_nm)) {
        if let (Some(key), Some(id)) = (style_match_key(dom, s), dom.attribute(s, &style_id)) {
            by_name.entry(key).or_insert_with(|| id.to_string());
        }
    }
    let mut b_to_out = std::collections::HashMap::new();
    let mut unpaired = Vec::new();
    for s in dom.elements(from_root, Some(&style_nm)) {
        let paired = style_match_key(dom, s).and_then(|k| by_name.get(&k));
        match (paired, dom.attribute(s, &style_id)) {
            (Some(out_id), Some(b_id)) => {
                if b_id != out_id {
                    b_to_out.insert(b_id.to_string(), out_id.clone());
                }
            }
            _ => unpaired.push(s),
        }
    }
    // Copy the unpaired styles from a scratch stylesheet so the id rule sees
    // only them.
    let scratch = dom.new_element(W::name("styles"));
    for s in unpaired {
        let clone = dom.clone_subtree(s);
        dom.add(scratch, clone);
    }
    crate::comparer::footnotes::copy_missing_styles(dom, to_root, scratch);
    b_to_out
}

/// Rename `w:styleId` values to Word-canonical ids derived from `w:name`.
///
/// Returns the old→new map (only entries that actually change). Also rewrites
/// `basedOn` / `next` / `link` inside the stylesheet. Skips a rename when the
/// target id is already claimed by a different style that is not itself renaming
/// away (no silent merge).
fn canonicalize_style_ids(
    dom: &mut Dom,
    styles_root: NodeId,
) -> std::collections::HashMap<String, String> {
    let style_nm = W::name("style");
    let style_id = W::name("styleId");
    let name_el = W::name("name");
    let styles: Vec<NodeId> = dom.elements(styles_root, Some(&style_nm));

    // Pass 1: desired renames (ignore collisions).
    let mut desired: Vec<(NodeId, String, String)> = Vec::new();
    for s in &styles {
        let Some(old) = dom.attribute(*s, &style_id).map(|v| v.to_string()) else {
            continue;
        };
        let Some(name) = dom
            .element(*s, &name_el)
            .and_then(|e| dom.attribute(e, &W::val()))
            .map(|v| v.to_string())
        else {
            continue;
        };
        let new_id = word_canonical_style_id(&name);
        if new_id.is_empty() || new_id == old {
            continue;
        }
        desired.push((*s, old, new_id));
    }

    // Pass 2: drop collisions — target held by a non-renaming style, or two
    // styles racing for the same target (first wins; prefer already-matching).
    let leaving: std::collections::HashSet<String> =
        desired.iter().map(|(_, old, _)| old.clone()).collect();
    let mut taken_targets: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut renames: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut plan: Vec<(NodeId, String)> = Vec::new();
    for (s, old, new_id) in desired {
        if taken_targets.contains(&new_id) {
            continue;
        }
        // Occupied by a style that stays put?
        let occupied_by_stayer = styles.iter().any(|&other| {
            dom.attribute(other, &style_id) == Some(new_id.as_str()) && !leaving.contains(&new_id)
        });
        if occupied_by_stayer {
            continue;
        }
        taken_targets.insert(new_id.clone());
        renames.insert(old, new_id.clone());
        plan.push((s, new_id));
    }
    for (s, new_id) in &plan {
        dom.set_attribute_value(*s, &style_id, Some(new_id));
    }
    // Rewrite basedOn / next / link vals that point at renamed ids.
    for local in ["basedOn", "next", "link"] {
        let nm = W::name(local);
        for e in dom.descendants(styles_root, Some(&nm)) {
            if let Some(v) = dom.attribute(e, &W::val())
                && let Some(nv) = renames.get(v)
            {
                dom.set_attribute_value(e, &W::val(), Some(nv));
            }
        }
    }
    renames
}

/// Apply a styleId rename map to `pStyle` / `rStyle` / `tblStyle` under `root`.
fn remap_style_refs(
    dom: &mut Dom,
    root: NodeId,
    renames: &std::collections::HashMap<String, String>,
) -> usize {
    if renames.is_empty() {
        return 0;
    }
    let mut n = 0;
    for local in ["pStyle", "rStyle", "tblStyle"] {
        let nm = W::name(local);
        for e in dom.descendants(root, Some(&nm)) {
            if let Some(v) = dom.attribute(e, &W::val())
                && let Some(nv) = renames.get(v)
            {
                dom.set_attribute_value(e, &W::val(), Some(nv));
                n += 1;
            }
        }
    }
    n
}

/// Word reads a docDefaults with no pPrDefault as its built-in paragraph
/// defaults, 160 after and 278 auto lines, while a present empty pPrDefault
/// means single spacing (ceaf5f4). Word's redline writes the original's
/// built-in defaults out and tracks the revision's Normal against them
/// (221577c35b), so every style merge below reads the values Word lays out.
fn materialize_builtin_paragraph_defaults(dom: &mut Dom, root: NodeId) -> bool {
    let Some(dd) = dom.element(root, &W::name("docDefaults")) else {
        return false;
    };
    if dom.element(dd, &W::name("pPrDefault")).is_some() {
        return false;
    }
    let spacing = dom.new_element(W::name("spacing"));
    for (k, v) in [("after", "160"), ("line", "278"), ("lineRule", "auto")] {
        dom.set_attribute_value(spacing, &W::name(k), Some(v));
    }
    let ppr = dom.new_element(W::p_pr());
    dom.add(ppr, spacing);
    let pd = dom.new_element(W::name("pPrDefault"));
    dom.add(pd, ppr);
    dom.add(dd, pd);
    true
}

/// When the A-based styles part lacks `docDefaults` or `latentStyles`, copy
/// them from B (Word redlines always carry both when B has a full stylesheet).
/// Full styles swap is intentionally avoided — it regressed sales_report pairs.
fn adopt_missing_styles_structure(dom: &mut Dom, out_root: NodeId, b_root: NodeId) -> bool {
    let mut changed = false;
    for local in ["docDefaults", "latentStyles"] {
        let nm = W::name(local);
        if dom.element(out_root, &nm).is_some() {
            continue;
        }
        let Some(src) = dom.element(b_root, &nm) else {
            continue;
        };
        let cloned = dom.clone_subtree(src);
        // docDefaults / latentStyles sort before w:style children.
        if let Some(first_style) = dom.element(out_root, &W::name("style")) {
            dom.add_before_self(first_style, cloned);
        } else {
            dom.add_first(out_root, cloned);
        }
        changed = true;
    }
    changed
}

/// header/footer→Normal (footer knife-edge line box). M71 always runs it in
/// Word mode so no-HF pairs like file_197 also get B's Calibri dd; M65 still
/// skips both-bare Normal (file_170).
///
/// GT evidence (sample-document × sd-2517-localized-heading-styles): GT Normal
/// rPr = Times New Roman sz/szCs 24 + rPrChange(old = Inter sz 22).
/// M467 — prune the merged Normal's live rPr of metric attrs whose value the
/// output docDefaults already supply (Word writes only the delta):
/// `kern` (dd absent = "0"), `sz`/`szCs`, `w14:ligatures` (dd absent =
/// "none"), and per-slot `rFonts` attrs (both concrete and `*Theme` names).
/// Oracle tab_test × table_autofit: B's Normal stores Arial kern=0 sz=20
/// szCs=22 eastAsiaTheme ligatures-none; A-dd has no kern, szCs=22, the same
/// eastAsiaTheme, no ligatures — Word's output Normal is Arial + sz=20 only.
/// basic_comment keeps kern=0 because A-dd kern=2 differs (m461).
fn prune_normal_rpr_context_equal_attrs(dom: &mut Dom, out_root: NodeId) -> bool {
    let Some(normal) = find_normal_style(dom, out_root) else {
        return false;
    };
    let Some(rpr) = dom.element(normal, &W::name("rPr")) else {
        return false;
    };
    let dd = rpr_default(dom, out_root);
    let dd_val = |dom: &Dom, local: &str, default: &str| -> String {
        dd.and_then(|d| dom.element(d, &W::name(local)))
            .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
            .unwrap_or_else(|| default.to_string())
    };
    let mut changed = false;
    for (local, default) in [("kern", "0"), ("sz", ""), ("szCs", "")] {
        let Some(e) = dom.element(rpr, &W::name(local)) else {
            continue;
        };
        let v = dom.attribute(e, &W::val()).unwrap_or("").to_string();
        let ddv = dd_val(dom, local, default);
        if !v.is_empty() && v == ddv {
            dom.remove(e);
            changed = true;
        }
    }
    let lig_name = W14::name("ligatures");
    let lig_val_name = W14::name("val");
    if let Some(e) = dom.element(rpr, &lig_name) {
        let v = dom.attribute(e, &lig_val_name).unwrap_or("").to_string();
        let ddv = dd
            .and_then(|d| dom.element(d, &lig_name))
            .and_then(|x| dom.attribute(x, &lig_val_name).map(str::to_string))
            .unwrap_or_else(|| "none".to_string());
        if v == ddv {
            dom.remove(e);
            changed = true;
        }
    }
    if let Some(fonts) = dom.element(rpr, &W::name("rFonts")) {
        let dd_fonts = dd.and_then(|d| dom.element(d, &W::name("rFonts")));
        for attr in [
            "ascii",
            "hAnsi",
            "eastAsia",
            "cs",
            "asciiTheme",
            "hAnsiTheme",
            "eastAsiaTheme",
            "cstheme",
        ] {
            let Some(v) = dom.attribute(fonts, &W::name(attr)).map(str::to_string) else {
                continue;
            };
            let ddv = dd_fonts.and_then(|f| dom.attribute(f, &W::name(attr)).map(str::to_string));
            if ddv.as_deref() == Some(v.as_str()) {
                dom.set_attribute_value(fonts, &W::name(attr), None);
                changed = true;
            }
        }
        if dom.attributes(fonts).is_empty() {
            dom.remove(fonts);
            changed = true;
        }
    }
    changed
}

/// M480b — docDefaults-delta DISABLING neutralizers on both-sides merged
/// styles.
///
/// Word's both-sides algorithm (mined from 132 oracle pairs, 98.7% of the
/// 2022 heading-axis rows): the output keeps A's docDefaults; every style
/// whose formatting blocks differ takes B's declaration plus a tracked
/// redefinition; and Word writes a per-attribute neutralizer for each
/// docDefaults delta the style's post-merge output chain fails to provide
/// (evals__memorandum × evals__nda: every merged heading carries kern 0 +
/// w14:ligatures none against A-dd kern 2 + ligatures). Character styles
/// count the post-merge Normal as a provider — runs resolve docDefaults →
/// paragraph layer → character layer — so a stamped Normal covers them
/// (tab_test × table_autofit oracle). Table styles never take stamps
/// (0/31 oracle rows).
///
/// Only the DISABLING direction fires here (A-dd kerns/ligates, B-dd
/// doesn't): the oracle neutralized 81/81 such pairs. The ENABLING
/// direction has a skip class Word applies whose gate is still unmined
/// (tab_test, table_widths, superdoc_hyperlink_cases) — writing there
/// re-breaks those pairs (the M480a falsification), so it stays out.
fn bake_bothsides_dd_disabling_neutralizers(
    dom: &mut Dom,
    out_root: NodeId,
    b_root: NodeId,
) -> bool {
    let kern_name = W::name("kern");
    let lig_name = W14::name("ligatures");
    let lig_val = W14::name("val");
    let dd_out = rpr_default(dom, out_root);
    let dd_b = rpr_default(dom, b_root);
    let kern_of = |dom: &Dom, holder: Option<NodeId>| -> Option<String> {
        holder
            .and_then(|h| dom.element(h, &kern_name))
            .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
    };
    let lig_of = |dom: &Dom, holder: Option<NodeId>| -> Option<String> {
        holder
            .and_then(|h| dom.element(h, &lig_name))
            .and_then(|e| dom.attribute(e, &lig_val).map(str::to_string))
    };
    let do_kern = kern_of(dom, dd_out).is_some_and(|v| v.parse::<i64>().unwrap_or(0) > 0)
        && kern_of(dom, dd_b).is_none_or(|v| v == "0");
    let do_lig = lig_of(dom, dd_out).is_some_and(|v| v != "none")
        && lig_of(dom, dd_b).is_none_or(|v| v == "none");
    if !do_kern && !do_lig {
        return false;
    }

    let style_nm = W::name("style");
    let style_id = W::name("styleId");
    let type_nm = W::name("type");
    let based_nm = W::name("basedOn");
    let normal = find_normal_style(dom, out_root);
    let by_id: std::collections::HashMap<String, NodeId> = dom
        .elements(out_root, Some(&style_nm))
        .into_iter()
        .filter_map(|s| Some((dom.attribute(s, &style_id)?.to_string(), s)))
        .collect();
    let based_of = |dom: &Dom, s: NodeId| -> Option<NodeId> {
        let v = dom
            .element(s, &based_nm)
            .and_then(|b| dom.attribute(b, &W::val()))?;
        by_id.get(v).copied()
    };
    // Nearest live declaration wins; walking direct rPr children only keeps
    // the probe on the live block (change records nest one level deeper).
    let chain_declares = |dom: &Dom, start: NodeId, name: &crate::xmllinq::XName| -> bool {
        let mut cur = Some(start);
        for _ in 0..12 {
            let Some(s) = cur else { break };
            if dom
                .element(s, &W::r_pr())
                .is_some_and(|r| dom.element(r, name).is_some())
            {
                return true;
            }
            cur = based_of(dom, s);
        }
        false
    };

    // Parents first: a stamped ancestor then provides for its descendants.
    let mut styles: Vec<(usize, NodeId)> = dom
        .elements(out_root, Some(&style_nm))
        .into_iter()
        .map(|s| {
            let mut depth = 0usize;
            let mut cur = based_of(dom, s);
            while let Some(p) = cur {
                depth += 1;
                if depth >= 12 {
                    break;
                }
                cur = based_of(dom, p);
            }
            (depth, s)
        })
        .collect();
    styles.sort_by_key(|&(d, _)| d);

    let mut changed = false;
    for (_, style) in styles {
        if Some(style) == normal {
            continue; // Normal takes the M72/M461/M478 merge path
        }
        match dom.attribute(style, &type_nm) {
            Some("paragraph") | Some("character") => {}
            _ => continue,
        }
        // Only styles the compare actually redefined (tracked records from the
        // both-sides merge) participate — untouched styles stay byte-stable.
        let tracked = dom
            .element(style, &W::p_pr())
            .is_some_and(|p| dom.element(p, &W::name("pPrChange")).is_some())
            || dom
                .element(style, &W::r_pr())
                .is_some_and(|r| dom.element(r, &W::name("rPrChange")).is_some());
        if !tracked {
            continue;
        }
        let is_char = dom.attribute(style, &type_nm) == Some("character");
        for (on, name, mk) in [
            (
                do_kern,
                &kern_name,
                (|dom: &mut Dom| {
                    let e = dom.new_element(W::name("kern"));
                    dom.set_attribute_value(e, &W::val(), Some("0"));
                    e
                }) as fn(&mut Dom) -> NodeId,
            ),
            (do_lig, &lig_name, |dom: &mut Dom| {
                let e = dom.new_element(W14::name("ligatures"));
                dom.set_attribute_value(e, &W14::name("val"), Some("none"));
                e
            }),
        ] {
            if !on || chain_declares(dom, style, name) {
                continue;
            }
            // Character runs resolve through the paragraph layer before the
            // character chain: a declaring post-merge Normal already covers.
            if is_char && normal.is_some_and(|n| chain_declares(dom, n, name)) {
                continue;
            }
            let rpr = match dom.element(style, &W::r_pr()) {
                Some(r) => r,
                None => {
                    let r = dom.new_element(W::r_pr());
                    insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                    r
                }
            };
            let e = mk(dom);
            add_rpr_child_in_order(dom, rpr, e, name.local_name());
            changed = true;
        }
    }
    changed
}

/// Word's redefined paragraph styles hold B's effective metrics as a delta
/// against the output context. Mined from the 747 pool redlines (4,924 tracked
/// paragraph styles present in B): for each rFonts slot, `sz`, `szCs` and the
/// spacing `before`/`after`/`line`, Word writes B's effective value exactly
/// when it differs from what the output style's parent chain and docDefaults
/// resolve, and writes nothing otherwise (rFonts 4,922, sz/szCs/before/after
/// 4,924, line 4,898 of 4,924). B's value resolves through B's own chain and
/// docDefaults, then the factory defaults (sz 20, spacing 0/0/240 auto). A
/// font slot resolves from the nearest rFonts declaring its concrete or theme
/// name. Parents run first, so a child reads its parent's resolved values.
/// B's counterpart is the style of the same (type, name), as Word pairs
/// them: ids name nothing across documents (1b4d's original `a3` is
/// Normal (Web), its revision's `a3` another style).
fn resolve_redefined_style_metrics(dom: &mut Dom, out_root: NodeId, b_root: NodeId) -> bool {
    let style_nm = W::name("style");
    let index = |dom: &Dom, root: NodeId| -> std::collections::HashMap<String, NodeId> {
        dom.elements(root, Some(&style_nm))
            .into_iter()
            .filter_map(|s| Some((dom.attribute(s, &W::name("styleId"))?.to_string(), s)))
            .collect()
    };
    let out_idx = index(dom, out_root);
    let b_idx = index(dom, b_root);
    let mut b_by_key: std::collections::HashMap<(String, String), NodeId> =
        std::collections::HashMap::new();
    for s in dom.elements(b_root, Some(&style_nm)) {
        if let Some(k) = style_match_key(dom, s) {
            b_by_key.entry(k).or_insert(s);
        }
    }
    let parent = |dom: &Dom, idx: &std::collections::HashMap<String, NodeId>, s: NodeId| {
        dom.element(s, &W::name("basedOn"))
            .and_then(|b| dom.attribute(b, &W::val()))
            .and_then(|v| idx.get(v).copied())
    };
    // Chain from `start` (inclusive) up to 12 styles, then docDefaults.
    let chain = |dom: &Dom,
                 idx: &std::collections::HashMap<String, NodeId>,
                 start: Option<NodeId>|
     -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = start;
        while let Some(c) = cur {
            if out.len() >= 12 || out.contains(&c) {
                break;
            }
            out.push(c);
            cur = parent(dom, idx, c);
        }
        out
    };
    let dd_ppr = |dom: &Dom, root: NodeId| {
        dom.element(root, &W::name("docDefaults"))
            .and_then(|d| dom.element(d, &W::name("pPrDefault")))
            .and_then(|d| dom.element(d, &W::p_pr()))
    };
    // Each holder is a style (reads its pPr/rPr) or a docDefaults pPr/rPr.
    let rpr_of = |dom: &Dom, n: NodeId, is_style: bool| {
        if is_style {
            dom.element(n, &W::r_pr())
        } else {
            Some(n)
        }
    };
    let ppr_of = |dom: &Dom, n: NodeId, is_style: bool| {
        if is_style {
            dom.element(n, &W::p_pr())
        } else {
            Some(n)
        }
    };
    let holders = |dom: &Dom, styles: Vec<NodeId>, root: NodeId, para: bool| {
        let mut h: Vec<(NodeId, bool)> = styles.into_iter().map(|s| (s, true)).collect();
        let d = if para {
            dd_ppr(dom, root)
        } else {
            rpr_default(dom, root)
        };
        if let Some(d) = d {
            h.push((d, false));
        }
        h
    };
    let font_slot = |dom: &Dom, hs: &[(NodeId, bool)], c: &str, t: &str| -> FontSlot {
        for &(n, st) in hs {
            if let Some(f) = rpr_of(dom, n, st).and_then(|r| dom.element(r, &W::name("rFonts"))) {
                let cv = dom.attribute(f, &W::name(c)).map(str::to_string);
                let tv = dom.attribute(f, &W::name(t)).map(str::to_string);
                if cv.is_some() || tv.is_some() {
                    return (cv, tv);
                }
            }
        }
        (None, None)
    };
    let slot_key = |v: &FontSlot| match v {
        (_, Some(t)) => Some(format!("t:{t}")),
        (Some(c), None) => Some(format!("c:{c}")),
        _ => None,
    };
    let run_val = |dom: &Dom, hs: &[(NodeId, bool)], local: &str| -> String {
        hs.iter()
            .find_map(|&(n, st)| {
                rpr_of(dom, n, st)
                    .and_then(|r| dom.element(r, &W::name(local)))
                    .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
            })
            .unwrap_or_else(|| "20".to_string())
    };
    let spacing_val = |dom: &Dom, hs: &[(NodeId, bool)], attr: &str, default: &str| -> String {
        hs.iter()
            .find_map(|&(n, st)| {
                ppr_of(dom, n, st)
                    .and_then(|p| dom.element(p, &W::name("spacing")))
                    .and_then(|e| dom.attribute(e, &W::name(attr)).map(str::to_string))
            })
            .unwrap_or_else(|| default.to_string())
    };
    // (line, lineRule) resolve together from the nearest declaring `line`.
    let line_val = |dom: &Dom, hs: &[(NodeId, bool)]| -> (String, String) {
        hs.iter()
            .find_map(|&(n, st)| {
                let sp = ppr_of(dom, n, st).and_then(|p| dom.element(p, &W::name("spacing")))?;
                let line = dom.attribute(sp, &W::name("line"))?.to_string();
                let rule = dom
                    .attribute(sp, &W::name("lineRule"))
                    .unwrap_or("auto")
                    .to_string();
                Some((line, rule))
            })
            .unwrap_or_else(|| ("240".to_string(), "auto".to_string()))
    };

    let mut styles: Vec<(usize, NodeId)> = dom
        .elements(out_root, Some(&style_nm))
        .into_iter()
        .map(|s| (chain(dom, &out_idx, Some(s)).len(), s))
        .collect();
    styles.sort_by_key(|&(d, _)| d);
    let mut changed = false;
    for (_, style) in styles {
        if dom.attribute(style, &W::name("type")) != Some("paragraph")
            || dom
                .attribute(style, &W::name("default"))
                .is_some_and(|v| v == "1" || v == "true")
        {
            continue;
        }
        let tracked = dom
            .element(style, &W::p_pr())
            .is_some_and(|p| dom.element(p, &W::name("pPrChange")).is_some())
            || dom
                .element(style, &W::r_pr())
                .is_some_and(|r| dom.element(r, &W::name("rPrChange")).is_some());
        let Some(&b_style) = style_match_key(dom, style).and_then(|k| b_by_key.get(&k)) else {
            continue;
        };
        if !tracked {
            continue;
        }
        let b_chain = chain(dom, &b_idx, Some(b_style));
        let o_chain = chain(dom, &out_idx, parent(dom, &out_idx, style));
        let b_r = holders(dom, b_chain.clone(), b_root, false);
        let o_r = holders(dom, o_chain.clone(), out_root, false);
        let b_p = holders(dom, b_chain, b_root, true);
        let o_p = holders(dom, o_chain, out_root, true);

        // --- run metrics ---
        let fonts: Vec<(FontSlot, bool)> = RPR_METRIC_FONT_SLOTS
            .iter()
            .map(|(c, t)| {
                let bv = font_slot(dom, &b_r, c, t);
                let differs = slot_key(&bv) != slot_key(&font_slot(dom, &o_r, c, t));
                (bv, differs)
            })
            .collect();
        let sizes: Vec<(&str, String, bool)> = ["sz", "szCs"]
            .into_iter()
            .map(|l| {
                let bv = run_val(dom, &b_r, l);
                let differs = bv != run_val(dom, &o_r, l);
                (l, bv, differs)
            })
            .collect();
        let need_rpr =
            fonts.iter().any(|f| f.1 && slot_key(&f.0).is_some()) || sizes.iter().any(|s| s.2);
        let rpr = match dom.element(style, &W::r_pr()) {
            Some(r) => Some(r),
            None if need_rpr => {
                let r = dom.new_element(W::r_pr());
                insert_child_by_rank(dom, style, r, "rPr", &style_child_rank);
                Some(r)
            }
            None => None,
        };
        if let Some(rpr) = rpr {
            // A missing rFonts is created only when a slot will be written.
            let rf = dom.element(rpr, &W::name("rFonts")).or_else(|| {
                fonts
                    .iter()
                    .any(|(bv, differs)| *differs && (bv.0.is_some() || bv.1.is_some()))
                    .then(|| {
                        let f = dom.new_element(W::name("rFonts"));
                        dom.add_first(rpr, f);
                        f
                    })
            });
            if let Some(rf) = rf {
                for ((c, t), (bv, differs)) in RPR_METRIC_FONT_SLOTS.iter().zip(&fonts) {
                    let before = (
                        dom.attribute(rf, &W::name(c)).map(str::to_string),
                        dom.attribute(rf, &W::name(t)).map(str::to_string),
                    );
                    let want = if *differs { bv.clone() } else { (None, None) };
                    if before != want {
                        dom.set_attribute_value(rf, &W::name(c), want.0.as_deref());
                        dom.set_attribute_value(rf, &W::name(t), want.1.as_deref());
                        changed = true;
                    }
                }
                if dom.attributes(rf).is_empty() {
                    dom.remove(rf);
                }
            }
            for (local, bv, differs) in sizes {
                let existing = dom.element(rpr, &W::name(local));
                match (existing, differs) {
                    (Some(e), false) => {
                        dom.remove(e);
                        changed = true;
                    }
                    (Some(e), true) => {
                        if dom.attribute(e, &W::val()) != Some(bv.as_str()) {
                            dom.set_attribute_value(e, &W::val(), Some(&bv));
                            changed = true;
                        }
                    }
                    (None, true) => {
                        let e = dom.new_element(W::name(local));
                        dom.set_attribute_value(e, &W::val(), Some(&bv));
                        add_rpr_child_in_order(dom, rpr, e, local);
                        changed = true;
                    }
                    (None, false) => {}
                }
            }
        }

        // --- spacing ---
        let mut want: Vec<(&str, Option<String>)> = Vec::new();
        for (attr, default) in [("before", "0"), ("after", "0")] {
            let bv = spacing_val(dom, &b_p, attr, default);
            let differs = bv != spacing_val(dom, &o_p, attr, default);
            want.push((attr, differs.then_some(bv)));
        }
        let (bl, br) = line_val(dom, &b_p);
        let line_differs = (bl.clone(), br.clone()) != line_val(dom, &o_p);
        want.push(("line", line_differs.then(|| bl.clone())));
        want.push(("lineRule", line_differs.then(|| br.clone())));
        let need_spacing = want.iter().any(|w| w.1.is_some());
        let ppr = match dom.element(style, &W::p_pr()) {
            Some(p) => Some(p),
            None if need_spacing => {
                let p = dom.new_element(W::p_pr());
                insert_child_by_rank(dom, style, p, "pPr", &style_child_rank);
                Some(p)
            }
            None => None,
        };
        let Some(ppr) = ppr else { continue };
        let sp = match dom.element(ppr, &W::name("spacing")) {
            Some(sp) => Some(sp),
            None if need_spacing => {
                let sp = dom.new_element(W::name("spacing"));
                insert_child_by_rank(dom, ppr, sp, "spacing", &ppr_child_rank);
                Some(sp)
            }
            None => None,
        };
        let Some(sp) = sp else { continue };
        for (attr, v) in want {
            if dom.attribute(sp, &W::name(attr)) != v.as_deref() {
                dom.set_attribute_value(sp, &W::name(attr), v.as_deref());
                changed = true;
            }
        }
        if dom.attributes(sp).is_empty() {
            dom.remove(sp);
        }
    }
    changed
}

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
    // file_103 × file_104 refines the gate: a stored pPr counts too (A bare,
    // B pPr only → Word writes B's run defaults). 227 corpus Word redlines
    // agree: with either side structured and differing run defaults, Word
    // always writes Normal rPr.
    let structured = |s: NodeId| {
        dom.element(s, &W::name("rPr")).is_some() || dom.element(s, &W::p_pr()).is_some()
    };
    if !structured(a_style) && !b_style.is_some_and(structured) {
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
    let (font_slots, [sz, sz_cs]) = b_effective;
    let fonts = match dom.element(rpr, &W::name("rFonts")) {
        Some(f) => f,
        None => {
            let f = dom.new_element(W::name("rFonts"));
            dom.add_first(rpr, f);
            f
        }
    };
    for ((attr, theme), (named, themed)) in RPR_METRIC_FONT_SLOTS.iter().zip(&font_slots) {
        dom.set_attribute_value(fonts, &W::name(attr), named.as_deref());
        dom.set_attribute_value(fonts, &W::name(theme), themed.as_deref());
    }
    // sz/szCs must follow EG_RPrBase order (rFonts < b..webHidden < color <
    // spacing < w < kern < position < sz < szCs). Anchoring them to rFonts —
    // as this code previously did — places them before any of color/spacing/
    // w/kern/position Normal already carries, breaking CT_RPr order and
    // tripping Word's repair. Insert after the last existing predecessor so
    // the new/updated sz/szCs land in their schema slot.
    for (name, v) in [("sz", &sz), ("szCs", &sz_cs)] {
        let existing = dom.element(rpr, &W::name(name));
        // CT_HpsMeasure makes w:val REQUIRED. Creating the element and then
        // passing None stripped the attribute but left `<w:szCs/>` behind —
        // not "no size", but XML Word refuses to open (Sch_MissRequiredAttribute
        // at styles.xml w:style[1]/w:rPr/w:szCs). ECMA-376 spells "no value"
        // here as the element's absence, so that is what we write.
        // B silent while A's docDefaults set a size: removing ours would
        // inherit A's, so Word writes the implicit 20 half-points (17 of 17
        // corpus Word redlines).
        let a_dd_size = rpr_default(dom, out_root)
            .and_then(|r| dom.element(r, &W::name(name)))
            .is_some();
        let Some(val) = v.as_deref().or(a_dd_size.then_some("20")) else {
            if let Some(e) = existing {
                dom.remove(e);
            }
            continue;
        };
        let e = match existing {
            Some(e) => e,
            None => {
                let e = dom.new_element(W::name(name));
                add_rpr_child_in_order(dom, rpr, e, name);
                e
            }
        };
        dom.set_attribute_value(e, &W::val(), Some(val));
    }
    // M461 generalized by file_198 — Word's live Normal is B's FULL
    // effective rPr: every stored B child survives (color, lang — not just
    // kern/ligatures), and metrics B keeps in its docDefaults (kern,
    // w14:ligatures) are materialized when the two documents' docDefaults
    // disagree on them (oracle Normal: Liberation fonts + color 00000A +
    // kern 2 + lang zh-CN/hi-IN + ligatures with A-dd declaring none).
    let lig_name = W14::name("ligatures");
    let b_rpr = b_style.and_then(|s| dom.element(s, &W::name("rPr")));
    let metric = |n: &crate::xmllinq::XName| {
        ["rFonts", "sz", "szCs", "lang", "kern", "rPrChange"]
            .iter()
            .any(|s| *n == W::name(s))
    };
    // d8b0c2ae01: a property only A's Normal stores (color 0000FF) is not
    // B's; Word records it in the rPrChange and leaves it off the live rPr,
    // or writes B's docDefaults value when B declares one there.
    for ac in dom.elements(rpr, None) {
        let Some(n) = dom.name(ac) else { continue };
        if metric(&n) || n != W::name(n.local_name()) {
            continue;
        }
        if b_rpr.is_some_and(|b| dom.element(b, &n).is_some()) {
            continue;
        }
        match rpr_default(dom, b_root).and_then(|r| dom.element(r, &n)) {
            Some(bd) => {
                let clone = dom.clone_subtree(bd);
                dom.replace_with(ac, &[clone]);
            }
            None => dom.remove(ac),
        }
    }
    if let Some(b_rpr) = b_rpr {
        let b_kids: Vec<NodeId> = dom.elements(b_rpr, None);
        for bc in b_kids {
            let Some(n) = dom.name(bc) else { continue };
            if metric(&n) && n != W::name("kern") {
                continue; // metric slots written above
            }
            if n == lig_name {
                if dom.element(rpr, &lig_name).is_none() {
                    let clone = dom.clone_subtree(bc);
                    dom.add(rpr, clone); // w14 extension: last, pre-rPrChange
                }
                continue;
            }
            let local = n.local_name().to_string();
            let clone = dom.clone_subtree(bc);
            match dom.element(rpr, &W::name(&local)) {
                // B's value wins over A's (Word's live Normal is B's).
                Some(old) => dom.replace_with(old, &[clone]),
                None => add_rpr_child_in_order(dom, rpr, clone, &local),
            }
        }
    }
    // dd-held metrics: materialize B's kern/ligatures when the docDefaults
    // disagree and neither the merged rPr nor B's stored rPr carries them.
    {
        let dd_elem = |dom: &Dom, root: NodeId, name: &crate::xmllinq::XName| -> Option<NodeId> {
            rpr_default(dom, root).and_then(|r| dom.element(r, name))
        };
        // A B with no docDefaults at all reads with Word's factory run
        // defaults: kern 2 and standard contextual ligatures.
        let b_factory = dom.element(b_root, &W::name("docDefaults")).is_none();
        let kern_name = W::name("kern");
        let a_kern = dd_elem(dom, out_root, &kern_name)
            .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string));
        let b_kern = if b_factory {
            Some("2".to_string())
        } else {
            dd_elem(dom, b_root, &kern_name)
                .and_then(|e| dom.attribute(e, &W::val()).map(str::to_string))
        };
        // B-dd lacks kern (implicit 0) while A-dd kerns: Word materializes
        // the neutralizer — kern 2 left live wraps every long line
        // differently (list_numbering × list_spacer1 oracle: effective kern 0
        // on all 14 Normal-based styles).
        if a_kern != b_kern && dom.element(rpr, &kern_name).is_none() {
            let e = dom.new_element(kern_name);
            dom.set_attribute_value(e, &W::val(), Some(b_kern.as_deref().unwrap_or("0")));
            add_rpr_child_in_order(dom, rpr, e, "kern");
        }
        let a_lig = dd_elem(dom, out_root, &lig_name)
            .and_then(|e| dom.attribute(e, &W14::name("val")).map(str::to_string));
        let b_lig = if b_factory {
            Some("standardContextual".to_string())
        } else {
            dd_elem(dom, b_root, &lig_name)
                .and_then(|e| dom.attribute(e, &W14::name("val")).map(str::to_string))
        };
        if a_lig != b_lig && dom.element(rpr, &lig_name).is_none() {
            let e = dom.new_element(lig_name.clone());
            dom.set_attribute_value(
                e,
                &W14::name("val"),
                Some(b_lig.as_deref().unwrap_or("none")),
            );
            dom.add(rpr, e);
        }
        // Language: only the attributes B's effective value changes against
        // A's docDefaults, a missing one read as Word's en-US / en-US / ar-SA
        // (159 of 161 corpus Word redlines).
        const IMPLICIT_LANG: [(&str, &str); 3] =
            [("val", "en-US"), ("eastAsia", "en-US"), ("bidi", "ar-SA")];
        let lang_name = W::name("lang");
        let lang_attr = |dom: &Dom, lang: Option<NodeId>, attr: &str| {
            lang.and_then(|l| dom.attribute(l, &W::name(attr)).map(str::to_string))
        };
        let a_lang = dd_elem(dom, out_root, &lang_name);
        let b_lang = b_style
            .and_then(|s| dom.element(s, &W::name("rPr")))
            .and_then(|r| dom.element(r, &lang_name));
        let b_dd_lang = dd_elem(dom, b_root, &lang_name);
        let delta: Vec<(&str, String)> = IMPLICIT_LANG
            .iter()
            .filter_map(|&(attr, implicit)| {
                let b = lang_attr(dom, b_lang, attr)
                    .or_else(|| lang_attr(dom, b_dd_lang, attr))
                    .unwrap_or_else(|| implicit.to_string());
                let a = lang_attr(dom, a_lang, attr).unwrap_or_else(|| implicit.to_string());
                (a != b).then_some((attr, b))
            })
            .collect();
        if let Some(old) = dom.element(rpr, &lang_name) {
            dom.remove(old);
        }
        if !delta.is_empty() {
            let e = dom.new_element(lang_name);
            for (attr, v) in &delta {
                dom.set_attribute_value(e, &W::name(attr), Some(v));
            }
            add_rpr_child_in_order(dom, rpr, e, "lang");
        }
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

/// Wrap each `w:br` a producer left directly under a paragraph (the schema
/// forbids it) in a run of its own with no properties, `<w:r><w:br/></w:r>`,
/// the way Word reads it. Left bare, the break sat outside the insertion or
/// deletion around it, so Word's Reject All kept every inserted break
/// (bc0135eaa1: 89 of them, 5 pages instead of 2).
fn wrap_bare_breaks(dom: &mut Dom, root: NodeId) {
    let bare: Vec<NodeId> = dom
        .descendants(root, Some(&W::name("br")))
        .into_iter()
        .filter(|&br| {
            dom.parent(br)
                .and_then(|p| dom.name(p))
                .is_some_and(|n| n == W::p())
        })
        .collect();
    for br in bare {
        let run = dom.new_element(W::r());
        dom.add_before_self(br, run);
        dom.remove(br);
        dom.add(run, br);
    }
}

/// The styleIds the package's stylesheet defines, or `None` without one.
fn defined_style_ids(pkg: &PartFs, main: &str) -> Option<std::collections::HashSet<String>> {
    let part = pkg.read_rels_for(main).and_then(|rels| {
        rels.items
            .iter()
            .find(|r| r.target_mode.is_none() && r.rel_type.ends_with("/styles"))
            .map(|r| pkg.resolve_rel_target(main, &r.target))
    })?;
    let xml = pkg.part_string(&part)?;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc)?;
    Some(crate::comparer::footnotes::defined_style_ids(&dom, root))
}

/// C.1/C.2 — `WmlComparer.PreProcessMarkup` (:434) at package level:
/// `ChangeFootnoteEndnoteReferencesToUniqueRange` (:1627) then
/// `AddFootnotesEndnotesParts` (:1604). C.3–C.5 extend it with
/// FillInEmptyFootnotesEndnotes, DetachExternalData and
/// AddUnidsToMarkupInContentParts in the C# order. Returns the names of the
/// parts it rewrote or created (empty = pure no-op, bytes untouched). An
/// orphaned footnote/endnote reference is an [`invalid_content`] error — C#
/// throws DocxodusException when no ComparisonLog is wired (:1676), and the
/// compare path wires none.
pub fn pre_process_markup(
    pkg: &mut PartFs,
    starting_id_for_footnotes_endnotes: i32,
) -> Result<Vec<String>, OpcError> {
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
        return Ok(Vec::new());
    };
    let fn_xml = fn_part.as_deref().and_then(|p| pkg.part_string(p));
    let en_xml = en_part.as_deref().and_then(|p| pkg.part_string(p));

    let mut dom = Dom::new();
    let main_doc = dom.parse_xdocument(&main_xml);
    let Some(main_root) = dom.root(main_doc) else {
        return Ok(Vec::new());
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
    // doc WITH references but no notes part errs inside, where C# throws).
    if has_refs || fn_root.is_some() || en_root.is_some() {
        crate::comparer::footnotes::change_footnote_endnote_references_to_unique_range(
            &mut dom,
            main_root,
            fn_root,
            en_root,
            starting_id_for_footnotes_endnotes,
            false,
        )
        .map_err(invalid_content)?;
    }

    // Before the unids, so the runs the repair adds get theirs.
    let style_ids = defined_style_ids(pkg, &main);
    for r in [Some(main_root), fn_root, en_root].into_iter().flatten() {
        wrap_bare_breaks(&mut dom, r);
        // Word reads a paragraph or run naming a style its own stylesheet
        // lacks as unstyled. Kept until the stylesheets merge, the reference
        // took the other document's style of that id (221577c35b: the
        // original's Normal text turned into the revision's headings).
        if let Some(ids) = &style_ids {
            crate::comparer::footnotes::strip_unresolved_style_refs(&mut dom, r, ids);
        }
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
    // with references but no part errs above, never reaches creation).
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
    Ok(changed)
}

/// Markup the engine refuses (C# throws DocxodusException): an `Err` the
/// caller can handle, never a panic, which would abort a WASM instance.
/// `InvalidData` is std's kind for well-formed-but-unacceptable input.
fn invalid_content(msg: String) -> OpcError {
    OpcError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, msg))
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

/// Word Compare leaves the body-level final `sectPr` without
/// headerReference/footerReference when an earlier mid-body section break
/// already defines the same (kind, type) slot — later sections inherit.
/// Evidence (docx_lots_of_comments_*, verdana×strict01, word_clean_strict01×…):
/// Word's redline has HF only on mid `pPr/sectPr`; the body final is empty.
/// Our pipeline sometimes leaves A's (or adopted) refs on the final as well,
/// which dual-binds chrome and diverges from Word. Strip only the **body
/// direct-child** final; mid multi-section even/default/first copies stay.
/// A slot the revision's own final `sectPr` sets (`revised_final`) stays:
/// Word's redline sections are the revision's (f8c1ce3e92 keeps the final
/// section's first-page header and footer after a title-page section).
fn strip_final_sectpr_inherited_header_footer(
    dom: &mut Dom,
    result_root: NodeId,
    revised_final: &std::collections::HashSet<(bool, String)>,
) {
    let href = W::name("headerReference");
    let fref = W::name("footerReference");
    let type_name = W::name("type");
    let Some(body) = dom.element(result_root, &W::body()) else {
        return;
    };
    // Body-level final sectPr is a direct child of w:body (not pPr/sectPr).
    let Some(final_sect) = dom.element(body, &W::name("sectPr")) else {
        return;
    };
    let mut earlier_slots: std::collections::HashSet<(bool, String)> =
        std::collections::HashSet::new();
    for sect in dom.descendants(body, Some(&W::name("sectPr"))) {
        if sect == final_sect {
            continue;
        }
        for e in dom.elements(sect, None) {
            let Some(n) = dom.name(e) else { continue };
            let is_header = if n == href {
                true
            } else if n == fref {
                false
            } else {
                continue;
            };
            let ty = dom
                .attribute(e, &type_name)
                .unwrap_or("default")
                .to_string();
            earlier_slots.insert((is_header, ty));
        }
    }
    if earlier_slots.is_empty() {
        return;
    }
    let mut to_remove: Vec<NodeId> = Vec::new();
    for e in dom.elements(final_sect, None) {
        let Some(n) = dom.name(e) else { continue };
        let is_header = if n == href {
            true
        } else if n == fref {
            false
        } else {
            continue;
        };
        let ty = dom
            .attribute(e, &type_name)
            .unwrap_or("default")
            .to_string();
        let slot = (is_header, ty);
        if earlier_slots.contains(&slot) && !revised_final.contains(&slot) {
            to_remove.push(e);
        }
    }
    for n in to_remove {
        dom.remove(n);
    }
}

/// The (is header, `w:type`) header/footer slots the body-level final
/// `sectPr` of `pkg`'s main document sets explicitly.
fn final_header_footer_slots(pkg: &PartFs) -> std::collections::HashSet<(bool, String)> {
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let Some(xml) = pkg.part_string(&main) else {
        return Default::default();
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(sect) = dom
        .root(doc)
        .and_then(|r| dom.element(r, &W::body()))
        .and_then(|b| dom.element(b, &W::name("sectPr")))
    else {
        return Default::default();
    };
    dom.elements(sect, None)
        .into_iter()
        .filter_map(|e| {
            let n = dom.name(e)?;
            let is_header = if n == W::name("headerReference") {
                true
            } else if n == W::name("footerReference") {
                false
            } else {
                return None;
            };
            let ty = dom.attribute(e, &W::name("type")).unwrap_or("default");
            Some((is_header, ty.to_string()))
        })
        .collect()
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
    settings: &WmlComparerSettings,
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
        let newly_adopted = out.part_bytes(&part).is_none();
        if newly_adopted {
            out.set_part(&part, bytes);
            // M378 (tiff×h_f −3.3): B-only header/footer content is pure-I in
            // Word (base had no HF). Copying B live left page numbers + labels
            // as Equal and thrash LO pagefair. Mark body content as inserted.
            mark_adopted_hf_content_as_inserted(out, &part, settings);
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
        // word/-parts get the dir-relative form, anything else the absolute
        // OPC form ("/customXml/…") so the target still resolves.
        let rel_target = crate::opc::relative_rel_target(out_main, &part);
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

/// M378 — wrap body content of a newly adopted B-only header/footer as pure-I.
/// Word Compare marks PAGE fields + labels as inserts when the original had no
/// HF; a live copy of B's part renders without revision chrome (−3 pagefair on
/// tiff×h_f_normal).
fn mark_adopted_hf_content_as_inserted(
    out: &mut PartFs,
    part: &str,
    settings: &WmlComparerSettings,
) {
    let Some(xml) = out.part_string(part) else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    // Root is w:hdr or w:ftr.
    let mut next_id: u32 = 1;
    let author = settings.author_for_revisions.as_str();
    let date = settings.date_time_for_revisions.as_str();
    let paras: Vec<NodeId> = dom.descendants(root, Some(&W::p()));
    for p in paras {
        // Skip if already has revision markup.
        if !dom.descendants(p, Some(&W::ins())).is_empty()
            || !dom.descendants(p, Some(&W::del())).is_empty()
        {
            continue;
        }
        let kids: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| dom.name(c) != Some(W::p_pr()))
            .collect();
        if kids.is_empty() {
            continue;
        }
        // Any contentful child? (run / hyperlink / sdt / drawing container)
        let has_content = kids.iter().any(|&c| {
            let Some(n) = dom.name(c) else {
                return false;
            };
            n == W::r()
                || n == W::hyperlink()
                || n == W::name("sdt")
                || n == W::name("drawing")
                || n.local_name() == "AlternateContent"
        });
        if !has_content {
            continue;
        }
        // Single wrapper around all body kids (Word PAGE field shape).
        let ins = dom.new_element(W::ins());
        dom.set_attribute_value(ins, &W::author(), Some(author));
        dom.set_attribute_value(ins, &W::date(), Some(date));
        let id = next_id.to_string();
        next_id += 1;
        dom.set_attribute_value(ins, &W::id(), Some(&id));
        // Insert wrapper before first body child, then move kids into it.
        if let Some(&first) = kids.first() {
            if dom.parent(first).is_some() {
                dom.add_before_self(first, ins);
            } else {
                dom.add(p, ins);
            }
        } else {
            dom.add(p, ins);
        }
        for c in kids {
            if dom.parent(c).is_none() {
                continue;
            }
            dom.remove(c);
            dom.add(ins, c);
        }
        // Word also stamps mark ins on pPr/rPr for PAGE-number paras.
        if let Some(ppr) = dom.element(p, &W::p_pr()) {
            let rpr = match dom.element(ppr, &W::r_pr()) {
                Some(r) => r,
                None => {
                    let r = dom.new_element(W::r_pr());
                    // rPr last-ish before pPrChange if any; otherwise append.
                    if let Some(ppc) = dom.element(ppr, &W::name("pPrChange")) {
                        dom.add_before_self(ppc, r);
                    } else {
                        dom.add(ppr, r);
                    }
                    r
                }
            };
            if dom.element(rpr, &W::ins()).is_none() && dom.element(rpr, &W::del()).is_none() {
                let mark = dom.new_element(W::ins());
                dom.set_attribute_value(mark, &W::author(), Some(author));
                dom.set_attribute_value(mark, &W::date(), Some(date));
                let mid = next_id.to_string();
                next_id += 1;
                dom.set_attribute_value(mark, &W::id(), Some(&mid));
                // Mark first under rPr (Word: ins then rStyle).
                if let Some(first) = dom.elements(rpr, None).first().copied() {
                    dom.add_before_self(first, mark);
                } else {
                    dom.add(rpr, mark);
                }
            }
        }
    }
    out.set_part(part, dom.serialize_element(root).into_bytes());
}

/// M383 — when the revised document has **no** headers/footers but the original
/// does, Word marks all original HF content pure-D. Eng left A HF live (−45 on
/// h_f_normal_odd_even_firstpg×basic_footnotes). Inverse of M378. The rule
/// holds per kind: a revision that keeps its headers but has no footer drops
/// the original's footers too (bc0135eaa1).
fn mark_a_only_hf_content_as_deleted(
    out: &mut PartFs,
    pkg1: &PartFs,
    pkg2: &PartFs,
    settings: &WmlComparerSettings,
) {
    let b_refs = header_footer_refs(pkg2);
    let mut seen = std::collections::HashSet::new();
    for (kind, _, part) in header_footer_refs(pkg1) {
        // B references this kind: the slot-level content diff owns the markup.
        if b_refs.iter().any(|(k, _, _)| *k == kind) || !seen.insert(part.clone()) {
            continue;
        }
        // Part may be stored under the same name in out (A-based package).
        mark_hf_part_content_as_deleted(out, &part, settings);
    }
}

/// Word's redline of a header/footer whose revised story ends with a table:
/// the original's closing paragraph, emptied by the diff, loses its mark,
/// since the revised story needs none, and keeps its own properties
/// unrecorded (bc0135eaa1). Accepted, the story ends with the table.
fn delete_story_closing_mark(dom: &mut Dom, story: NodeId, settings: &WmlComparerSettings) {
    let Some(&last) = dom.elements(story, None).last() else {
        return;
    };
    let live_content = [
        W::t(),
        W::name("drawing"),
        W::name("pict"),
        W::name("object"),
    ];
    if !dom.name_is(last, &W::p())
        || live_content
            .iter()
            .any(|n| !dom.descendants(last, Some(n)).is_empty())
    {
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
    if mark
        .is_some_and(|r| dom.element(r, &W::ins()).is_some() || dom.element(r, &W::del()).is_some())
    {
        return;
    }
    // The diff recorded B's blank properties over A's; Word keeps A's.
    if let Some(change) = dom.element(ppr, &W::name("pPrChange")) {
        let old = dom.element(change, &W::p_pr());
        dom.remove(change);
        for c in dom.elements(ppr, None) {
            if Some(c) != mark {
                dom.remove(c);
            }
        }
        for c in old.map(|o| dom.elements(o, None)).unwrap_or_default() {
            let clone = dom.clone_subtree(c);
            match mark {
                Some(r) => dom.add_before_self(r, clone),
                None => dom.add(ppr, clone),
            }
        }
    }
    let mark = match mark {
        Some(r) => r,
        None => {
            let r = dom.new_element(W::r_pr());
            dom.add(ppr, r);
            r
        }
    };
    let del = dom.new_element(W::del());
    let id = next_free_revision_id(dom, story).to_string();
    dom.set_attribute_value(del, &W::id(), Some(&id));
    dom.set_attribute_value(del, &W::author(), Some(&settings.author_for_revisions));
    dom.set_attribute_value(del, &W::date(), Some(&settings.date_time_for_revisions));
    dom.add_first(mark, del);
}

/// Word's redline of a header/footer the revised document drops: the
/// content goes pure-D and every paragraph mark but the story's last is
/// deleted (a paragraph without `pPr` included). The last mark stays: it is
/// the blank paragraph the revised side implies, so its properties reset to
/// that paragraph's (the kind's own `Header`/`Footer` style at most) with
/// the old ones in a `pPrChange` and `rPrChange`. The last paragraph of a
/// content control keeps its mark as well. (Word also restyles that blank
/// paragraph with a style it picks by index from the other document, e.g.
/// `ListParagraph` for a footer; that is not copied.)
fn mark_hf_part_content_as_deleted(out: &mut PartFs, part: &str, settings: &WmlComparerSettings) {
    let Some(xml) = out.part_string(part) else {
        return;
    };
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let Some(root) = dom.root(doc) else {
        return;
    };
    let mut next_id: u32 = 1;
    let author = settings.author_for_revisions.as_str();
    let date = settings.date_time_for_revisions.as_str();
    let mut revision = |dom: &mut Dom, name: XName| -> NodeId {
        let mark = dom.new_element(name);
        dom.set_attribute_value(mark, &W::author(), Some(author));
        dom.set_attribute_value(mark, &W::date(), Some(date));
        dom.set_attribute_value(mark, &W::id(), Some(&next_id.to_string()));
        next_id += 1;
        mark
    };
    let own_style = if dom.name(root) == Some(W::name("ftr")) {
        "Footer"
    } else {
        "Header"
    };
    let story_last = dom.elements(root, Some(&W::p())).last().copied();
    let paras: Vec<NodeId> = dom.descendants(root, Some(&W::p()));
    for p in paras {
        if !dom.descendants(p, Some(&W::ins())).is_empty()
            || !dom.descendants(p, Some(&W::del())).is_empty()
        {
            continue;
        }
        let kids: Vec<NodeId> = dom
            .elements(p, None)
            .into_iter()
            .filter(|&c| dom.name(c) != Some(W::p_pr()))
            .collect();
        let has_content = kids.iter().any(|&c| {
            let Some(n) = dom.name(c) else {
                return false;
            };
            n == W::r()
                || n == W::hyperlink()
                || n == W::name("sdt")
                || n == W::name("drawing")
                || n.local_name() == "AlternateContent"
        });
        if has_content {
            let del = revision(&mut dom, W::del());
            dom.add_before_self(kids[0], del);
            for c in kids {
                dom.remove(c);
                // Rename w:t → w:delText under this run tree for pure-D.
                for t in dom.descendants(c, Some(&W::t())) {
                    dom.set_name(t, W::name("delText"));
                }
                dom.add(del, c);
            }
        }
        let ends_content_control = dom.parent(p).is_some_and(|c| {
            dom.name(c) == Some(W::name("sdtContent"))
                && dom.elements(c, Some(&W::p())).last() == Some(&p)
        });
        if Some(p) == story_last {
            reset_to_blank_paragraph(&mut dom, p, own_style, &mut revision);
        } else if !ends_content_control {
            // Word's PAGE + label shape: the mark goes with the content.
            let ppr = child_or_first(&mut dom, p, W::p_pr());
            let rpr = child_before(&mut dom, ppr, W::r_pr(), &["sectPr", "pPrChange"]);
            if dom.element(rpr, &W::ins()).is_none() && dom.element(rpr, &W::del()).is_none() {
                let mark = revision(&mut dom, W::del());
                match dom.elements(rpr, None).first().copied() {
                    Some(first) => dom.add_before_self(first, mark),
                    None => dom.add(rpr, mark),
                }
            }
        }
    }
    out.set_part(part, dom.serialize_element(root).into_bytes());
}

/// Reset the story's last paragraph to the blank one Word's redline keeps:
/// no properties but the kind's own style, the old paragraph properties in
/// a `pPrChange` and the old mark formatting in an `rPrChange`.
fn reset_to_blank_paragraph(
    dom: &mut Dom,
    p: NodeId,
    own_style: &str,
    revision: &mut impl FnMut(&mut Dom, XName) -> NodeId,
) {
    let Some(ppr) = dom.element(p, &W::p_pr()) else {
        return;
    };
    let keep = |dom: &Dom, c: NodeId| {
        let Some(n) = dom.name(c) else {
            return true;
        };
        matches!(n.local_name(), "rPr" | "sectPr" | "pPrChange")
            || (n == W::name("pStyle") && dom.attribute(c, &W::val()) == Some(own_style))
    };
    let old: Vec<NodeId> = dom
        .elements(ppr, None)
        .into_iter()
        .filter(|&c| !keep(dom, c))
        .collect();
    if !old.is_empty() && dom.element(ppr, &W::name("pPrChange")).is_none() {
        let change = revision(dom, W::name("pPrChange"));
        let recorded = dom.new_element(W::p_pr());
        // The recorded properties are the whole old set, the kept style
        // included.
        for c in dom.elements(ppr, None) {
            if !matches!(
                dom.name(c).map(|n| n.local_name().to_string()).as_deref(),
                Some("rPr" | "sectPr" | "pPrChange")
            ) {
                let copy = dom.clone_subtree(c);
                dom.add(recorded, copy);
            }
        }
        for c in old {
            dom.remove(c);
        }
        dom.add(change, recorded);
        dom.add(ppr, change);
    }
    if let Some(rpr) = dom.element(ppr, &W::r_pr()) {
        let props: Vec<NodeId> = dom
            .elements(rpr, None)
            .into_iter()
            .filter(|&c| {
                !dom.name(c).is_some_and(|n| {
                    matches!(
                        n.local_name(),
                        "ins" | "del" | "moveFrom" | "moveTo" | "rPrChange"
                    )
                })
            })
            .collect();
        if !props.is_empty() && dom.element(rpr, &W::name("rPrChange")).is_none() {
            let change = revision(dom, W::name("rPrChange"));
            let recorded = dom.new_element(W::r_pr());
            for c in props {
                dom.remove(c);
                dom.add(recorded, c);
            }
            dom.add(change, recorded);
            dom.add(rpr, change);
        }
    }
}

/// `parent`'s `name` child, created as its first child when absent.
fn child_or_first(dom: &mut Dom, parent: NodeId, name: XName) -> NodeId {
    if let Some(c) = dom.element(parent, &name) {
        return c;
    }
    let c = dom.new_element(name);
    match dom.elements(parent, None).first().copied() {
        Some(first) => dom.add_before_self(first, c),
        None => dom.add(parent, c),
    }
    c
}

/// `parent`'s `name` child, created before the first of `later` (schema
/// order) or last when none is present.
fn child_before(dom: &mut Dom, parent: NodeId, name: XName, later: &[&str]) -> NodeId {
    if let Some(c) = dom.element(parent, &name) {
        return c;
    }
    let c = dom.new_element(name);
    let anchor = dom.elements(parent, None).into_iter().find(|&n| {
        dom.name(n)
            .is_some_and(|nm| later.contains(&nm.local_name()))
    });
    match anchor {
        Some(a) => dom.add_before_self(a, c),
        None => dom.add(parent, c),
    }
    c
}

/// M379 — when the original has no real footnotes/endnotes separators but the
/// revised document does, copy B's notes part into the package. Word carries
/// separator + continuationSeparator (often with drawings) even when body has
/// zero footnote refs (tiff×h_f_normal). Empty shells thrash LO page geometry.
fn adopt_b_notes_when_a_lacks_separators(
    out: &mut PartFs,
    pkg1: &PartFs,
    pkg2: &PartFs,
    out_main: &str,
) {
    for (part, rel_suffix, ct) in [
        (
            "word/footnotes.xml",
            "footnotes",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
        ),
        (
            "word/endnotes.xml",
            "endnotes",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.endnotes+xml",
        ),
    ] {
        let a_has_sep = pkg1.part_string(part).is_some_and(|x| {
            x.contains("w:type=\"separator\"") || x.contains("w:type='separator'")
        });
        if a_has_sep {
            continue;
        }
        let Some(b_xml) = pkg2.part_string(part) else {
            continue;
        };
        if !(b_xml.contains("w:type=\"separator\"") || b_xml.contains("w:type='separator'")) {
            continue;
        }
        // Also skip if out already has a real separator part (paired compare).
        let out_has_sep = out.part_string(part).is_some_and(|x| {
            x.contains("w:type=\"separator\"") || x.contains("w:type='separator'")
        });
        if out_has_sep {
            continue;
        }
        // When B.3/B.4 already produced the part (rectified content defs,
        // renumbered 1..n), a wholesale copy of B's part would clobber those
        // defs with B's PRE-rectify ids (c2: ref 1 vs def 2001 dangling).
        // Merge only B's STRUCTURAL notes (separator/continuationSeparator/
        // continuationNotice) in front of the rectified defs instead.
        if let Some(out_xml) = out.part_string(part) {
            let mut sd = Dom::new();
            let od = sd.parse_xdocument(&out_xml);
            let bd = sd.parse_xdocument(&b_xml);
            if let (Some(or), Some(br)) = (sd.root(od), sd.root(bd)) {
                let def = if rel_suffix == "footnotes" {
                    crate::namespaces::W::footnote()
                } else {
                    crate::namespaces::W::endnote()
                };
                let structural: Vec<_> = sd
                    .elements(br, Some(&def))
                    .into_iter()
                    .filter(|&n| crate::comparer::footnotes::is_structural_note(&sd, n))
                    .collect();
                let first_out = sd.elements(or, Some(&def)).first().copied();
                for s in structural {
                    let cloned = sd.clone_subtree(s);
                    match first_out {
                        Some(f) => sd.add_before_self(f, cloned),
                        None => sd.add(or, cloned),
                    }
                }
                out.set_part(part, sd.serialize_element(or).into_bytes());
            }
            continue;
        }
        out.set_part(part, b_xml.into_bytes());
        out.add_content_type_override(&format!("/{part}"), ct);
        // Ensure main-document relationship.
        let has_rel = out.read_rels_for(out_main).is_some_and(|r| {
            r.items
                .iter()
                .any(|i| i.rel_type.ends_with(&format!("/{rel_suffix}")))
        });
        if !has_rel {
            let target = crate::opc::relative_rel_target(out_main, part);
            out.add_document_relationship(
                out_main,
                &format!(
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{rel_suffix}"
                ),
                &target,
            );
        }
        // Carry notes-part rels + media (separator drawings).
        if let Some(nrels) = pkg2.read_rels_for(part) {
            let mut rels_xml = String::from(
                "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
                 <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
            );
            for nr in &nrels.items {
                let mode = nr
                    .target_mode
                    .as_deref()
                    .map(|m| format!(" TargetMode=\"{m}\""))
                    .unwrap_or_default();
                let mut rel_target_out = nr.target.clone();
                if nr.target_mode.as_deref() != Some("External") {
                    let t = pkg2.resolve_rel_target(part, &nr.target);
                    if let Some(tb) = pkg2.part_bytes(&t).map(<[u8]>::to_vec) {
                        let t_out = unique_part_name(out, &t, &tb);
                        if out.part_bytes(&t_out).is_none() {
                            out.set_part(&t_out, tb);
                        }
                        if t_out != t {
                            rel_target_out = t_out.rsplit('/').next().unwrap_or(&t_out).to_string();
                        }
                    }
                }
                let xe = |s: &str| {
                    s.replace('&', "&amp;")
                        .replace('<', "&lt;")
                        .replace('"', "&quot;")
                };
                rels_xml.push_str(&format!(
                    "<Relationship Id=\"{}\" Type=\"{}\" Target=\"{}\"{}/>",
                    xe(&nr.id),
                    xe(&nr.rel_type),
                    xe(&rel_target_out),
                    mode
                ));
            }
            rels_xml.push_str("</Relationships>");
            let base = part.rsplit('/').next().unwrap_or(part);
            out.set_part(&format!("word/_rels/{base}.rels"), rels_xml.into_bytes());
        }
    }
}

/// M483 — re-cache `w:color` hex values against the package's theme.
///
/// A `w:color` with `w:themeColor` renders by its cached `w:val` hex, not by
/// live theme resolution (LO and Word both paint the cache). Styles brought
/// over from B were cached under B's theme; the output ships A's theme, so
/// every themed color is visibly wrong until re-cached. Word's oracle
/// re-resolves (tab_test H1Char: B declaration kept but val rewritten to
/// A-theme accent1 shade BF = 365F91). Change-record baselines resolve to
/// the same theme, so rewriting every color element is a no-op for them.
fn reresolve_theme_color_hexes(dom: &mut Dom, styles_root: NodeId, theme_xml: &str) -> bool {
    let slot_hex = |slot: &str| -> Option<String> {
        let i = theme_xml.find(&format!("<a:{slot}>"))?;
        let seg = &theme_xml[i..theme_xml[i..]
            .find(&format!("</a:{slot}>"))
            .map_or(theme_xml.len(), |e| i + e)];
        let hex = if let Some(j) = seg.find("srgbClr val=\"") {
            &seg[j + 13..j + 19]
        } else {
            let j = seg.find("lastClr=\"")?;
            &seg[j + 9..j + 15]
        };
        Some(hex.to_uppercase())
    };
    let slot_of = |name: &str| -> Option<&'static str> {
        Some(match name {
            "accent1" => "a:accent1",
            "accent2" => "a:accent2",
            "accent3" => "a:accent3",
            "accent4" => "a:accent4",
            "accent5" => "a:accent5",
            "accent6" => "a:accent6",
            "text1" => "a:dk1",
            "text2" => "a:dk2",
            "background1" => "a:lt1",
            "background2" => "a:lt2",
            "hyperlink" => "a:hlink",
            "followedHyperlink" => "a:folHlink",
            _ => return None,
        })
    };
    // Word applies themeShade/themeTint as HSL LUMINANCE scaling, not RGB
    // multiply (linear RGB would give accent1 4F81BD shade BF = 3B618E), and
    // truncates each channel: 156082 tint 3F = B2DEF2, shade BF = 0F4761
    // (139 Word samples: 86 exact, the rest one step off; rounding gets 22).
    let apply = |hex: &str, factor: &str, toward_white: bool| -> Option<String> {
        let f = u32::from_str_radix(factor, 16).ok()? as f64 / 255.0;
        let r = u32::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
        let g = u32::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
        let b = u32::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
        let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
        let l = (mx + mn) / 2.0;
        let (h, s) = if (mx - mn).abs() < 1e-9 {
            (0.0, 0.0)
        } else {
            let d = mx - mn;
            let s = if l > 0.5 {
                d / (2.0 - mx - mn)
            } else {
                d / (mx + mn)
            };
            let h = if (mx - r).abs() < 1e-9 {
                ((g - b) / d + if g < b { 6.0 } else { 0.0 }) / 6.0
            } else if (mx - g).abs() < 1e-9 {
                ((b - r) / d + 2.0) / 6.0
            } else {
                ((r - g) / d + 4.0) / 6.0
            };
            (h, s)
        };
        let l2 = if toward_white {
            1.0 - (1.0 - l) * f
        } else {
            l * f
        };
        let hue = |p: f64, q: f64, mut t: f64| -> f64 {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            }
        };
        let (r2, g2, b2) = if s.abs() < 1e-9 {
            (l2, l2, l2)
        } else {
            let q = if l2 < 0.5 {
                l2 * (1.0 + s)
            } else {
                l2 + s - l2 * s
            };
            let p = 2.0 * l2 - q;
            (
                hue(p, q, h + 1.0 / 3.0),
                hue(p, q, h),
                hue(p, q, h - 1.0 / 3.0),
            )
        };
        Some(
            [r2, g2, b2]
                .iter()
                .map(|c| format!("{:02X}", (c * 255.0).floor().clamp(0.0, 255.0) as u32))
                .collect(),
        )
    };

    // Every themed hex a style carries, not only `w:color`'s: shading
    // colours and fills and border colours are re-cached too (2e3f1e26,
    // 512b24be: Word's redline leaves none stale).
    let color_name = W::name("color");
    let slots = [
        ("themeColor", "themeShade", "themeTint", "color"),
        ("themeFill", "themeFillShade", "themeFillTint", "fill"),
    ]
    .map(|(t, sh, ti, hex)| (W::name(t), W::name(sh), W::name(ti), W::name(hex)));
    let mut changed = false;
    for e in dom.descendants(styles_root, None) {
        for (theme_attr, shade_attr, tint_attr, hex_attr) in &slots {
            let Some(name) = dom.attribute(e, theme_attr).map(str::to_string) else {
                continue;
            };
            let Some(slot) = slot_of(&name) else { continue };
            let Some(base) = slot_hex(&slot[2..]) else {
                continue;
            };
            let hex_attr = if dom.name(e).as_ref() == Some(&color_name) {
                W::val()
            } else {
                hex_attr.clone()
            };
            let Some(cur) = dom.attribute(e, &hex_attr).map(str::to_uppercase) else {
                continue;
            };
            let expect = if let Some(sh) = dom.attribute(e, shade_attr).map(str::to_string) {
                apply(&base, &sh, false)
            } else if let Some(ti) = dom.attribute(e, tint_attr).map(str::to_string) {
                apply(&base, &ti, true)
            } else {
                Some(base)
            };
            let Some(expect) = expect else { continue };
            // rounding tolerance: correctly-cached values may differ by ±2 per
            // channel from our HSL math — leave those (they're already right);
            // only genuinely stale caches (different theme) get rewritten.
            let close = cur.len() == 6
                && (0..3).all(|i| {
                    let a = u32::from_str_radix(&cur[i * 2..i * 2 + 2], 16).unwrap_or(999);
                    let b = u32::from_str_radix(&expect[i * 2..i * 2 + 2], 16).unwrap_or(0);
                    a.abs_diff(b) <= 2
                });
            if !close && cur != "AUTO" {
                dom.set_attribute_value(e, &hex_attr, Some(&expect));
                changed = true;
            }
        }
    }
    changed
}

/// M481 — Word-repair: wire core parts the package carries but the main
/// document never references. superdoc_hyperlink_cases ships styles.xml
/// WITHOUT a styles relationship in document.xml.rels; a spec-following
/// consumer (LO included) then never loads the stylesheet and the whole
/// document renders in fallback fonts. Word repairs the relationship on
/// open (oracle rels: rId1 -> styles.xml), so match it at compare time —
/// same family as the dangling-numbering repair.
const CUSTOM_PROPS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties";

/// The part the package's custom-properties relationship reaches.
fn custom_props_part(pkg: &PartFs) -> Option<String> {
    pkg.package_relationships()
        .items
        .iter()
        .find(|r| r.rel_type == CUSTOM_PROPS_REL)
        .map(|r| r.target.trim_start_matches('/').to_string())
        .filter(|p| pkg.part_bytes(p).is_some())
}

/// Word's redline keeps the custom document properties of both sides: the
/// revision's alone when the original has none (83 of the 96 Word redlines
/// where only the revision has them), else the union, the original's value
/// winning where both name a property (every conflict among the 47 with
/// both). Without them a `DOCPROPERTY` field of the accepted text reads
/// "Error! Unknown document property name." (f8c1ce3e92).
fn merge_custom_properties(out: &mut PartFs, pkg2: &PartFs) {
    let Some(b_part) = custom_props_part(pkg2) else {
        return;
    };
    let Some(b_xml) = pkg2.part_string(&b_part) else {
        return;
    };
    let Some(a_part) = custom_props_part(out) else {
        if out.part_bytes(&b_part).is_some() {
            return;
        }
        out.set_part(&b_part, b_xml.into_bytes());
        out.add_content_type_override(
            &format!("/{b_part}"),
            "application/vnd.openxmlformats-officedocument.custom-properties+xml",
        );
        out.add_package_relationship(CUSTOM_PROPS_REL, &b_part);
        return;
    };
    let Some(a_xml) = out.part_string(&a_part) else {
        return;
    };
    let mut dom = Dom::new();
    let a_doc = dom.parse_xdocument(&a_xml);
    let b_doc = dom.parse_xdocument(&b_xml);
    let (Some(a_root), Some(b_root)) = (dom.root(a_doc), dom.root(b_doc)) else {
        return;
    };
    let name = XName::get("name", "");
    let pid = XName::get("pid", "");
    let a_props = dom.elements(a_root, None);
    let mut next_pid = a_props
        .iter()
        .filter_map(|&p| dom.attribute(p, &pid)?.parse::<u32>().ok())
        .max()
        .unwrap_or(1);
    let a_names: std::collections::HashSet<String> = a_props
        .iter()
        .filter_map(|&p| dom.attribute(p, &name).map(str::to_string))
        .collect();
    let mut added = false;
    for bp in dom.elements(b_root, None) {
        let Some(n) = dom.attribute(bp, &name) else {
            continue;
        };
        if a_names.contains(n) {
            continue;
        }
        let clone = dom.clone_subtree(bp);
        next_pid += 1;
        dom.set_attribute_value(clone, &pid, Some(&next_pid.to_string()));
        dom.add(a_root, clone);
        added = true;
    }
    if added {
        out.set_part(&a_part, dom.serialize_document(a_doc).into_bytes());
    }
}

fn repair_missing_core_relationships(out: &mut PartFs, out_main: &str) {
    const CORE: [(&str, &str); 5] = [
        ("word/styles.xml", "styles"),
        ("word/settings.xml", "settings"),
        ("word/webSettings.xml", "webSettings"),
        ("word/fontTable.xml", "fontTable"),
        ("word/theme/theme1.xml", "theme"),
    ];
    for (part, rel_suffix) in CORE {
        if out.part_bytes(part).is_none() {
            continue;
        }
        let has_rel = out.read_rels_for(out_main).is_some_and(|r| {
            r.items
                .iter()
                .any(|i| i.rel_type.ends_with(&format!("/{rel_suffix}")))
        });
        if !has_rel {
            let target = crate::opc::relative_rel_target(out_main, part);
            out.add_document_relationship(
                out_main,
                &format!(
                    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{rel_suffix}"
                ),
                &target,
            );
        }
    }
}

/// D.6 — `WmlComparer.GetRevisions` (:3940) byte facade: list every tracked
/// revision in a redline `.docx` — main-part groups, footnote/endnote
/// definition groups, `w:rPrChange` format changes, then (settings-gated)
/// move detection. `TestForInvalidContent` failures are an [`invalid_content`]
/// error where C# throws.
pub fn get_revisions(
    docx: &[u8],
    settings: &crate::comparer::WmlComparerSettings,
) -> Result<Vec<crate::comparer::WmlComparerRevision>, OpcError> {
    use crate::comparer::{preprocess, revisions};

    let pkg = PartFs::open(docx)?;
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = pkg
        .part_string(&main)
        .ok_or_else(|| OpcError::PartNotFound(main.clone()))?;
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom
        .root(d)
        .ok_or_else(|| OpcError::PartNotFound(format!("{main}: no root element")))?;

    // C# :3948–:3949 — TestForInvalidContent (throws) +
    // RemoveExistingPowerToolsMarkup on main and both notes parts.
    preprocess::test_for_invalid_content(&dom, root).map_err(invalid_content)?;
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
        .ok_or_else(|| OpcError::PartNotFound(format!("{main}: no w:body")))?;
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

/// Serialize one revision to the stable JSON object shape shared by the CLI
/// (`jubarte revisions --json`) and the wasm `getRevisions` binding. Full
/// string escaping: backslash, quote, and EVERY control char < 0x20 (document
/// text can carry `\t`, `\r`, vertical tabs, …).
pub fn revision_to_json(r: &crate::comparer::WmlComparerRevision) -> String {
    fn esc(s: &str) -> String {
        let mut o = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\\' => o.push_str("\\\\"),
                '"' => o.push_str("\\\""),
                '\n' => o.push_str("\\n"),
                '\r' => o.push_str("\\r"),
                '\t' => o.push_str("\\t"),
                c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
                c => o.push(c),
            }
        }
        o
    }
    let format_change = r.format_change.as_ref().map_or("null".to_string(), |fc| {
        let props: Vec<String> = fc
            .changed_properties
            .iter()
            .map(|p| format!("\"{}\"", esc(p)))
            .collect();
        format!("{{\"changedProperties\":[{}]}}", props.join(","))
    });
    format!(
        "{{\"type\":\"{:?}\",\"author\":\"{}\",\"date\":\"{}\",\"part\":\"{}\",\"moveGroupId\":{},\"isMoveSource\":{},\"formatChange\":{},\"text\":\"{}\"}}",
        r.revision_type,
        esc(r.author.as_deref().unwrap_or("")),
        esc(r.date.as_deref().unwrap_or("")),
        esc(&r.part_name),
        r.move_group_id
            .map_or("null".to_string(), |v| v.to_string()),
        r.is_move_source
            .map_or("null".to_string(), |v| v.to_string()),
        format_change,
        esc(r.text.as_deref().unwrap_or("")),
    )
}

/// Serialize a revision list to a single JSON array string — the wasm
/// `getRevisions` binding shape (the CLI prints one object per line instead).
pub fn revisions_to_json(revs: &[crate::comparer::WmlComparerRevision]) -> String {
    let items: Vec<String> = revs.iter().map(revision_to_json).collect();
    format!("[{}]", items.join(","))
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
fn main_part_xml(docx: &[u8]) -> Option<String> {
    let pkg = PartFs::open(docx).ok()?;
    let main = pkg
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let xml = pkg.part_string(&main)?;
    Some(normalize_strict_namespaces(&xml).into_owned())
}

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

    // Accept-before-diff flattens a revised insertion into live text, so a late
    // pass can no longer see where that insertion ended. Keep both main parts
    // from this moment. Faithful mode never reads them.
    let ladder_sources = if settings.merge_replaced_paragraphs {
        match (
            main_part_xml(&original_owned),
            main_part_xml(&modified_owned),
        ) {
            (Some(original_xml), Some(revised_xml)) => Some((original_xml, revised_xml)),
            _ => None,
        }
    } else {
        None
    };

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
        return crate::comparer::fixups::fix_up_drawing_ids_in_package(&original_owned);
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
        )?
    } else {
        Vec::new()
    };
    pre_process_markup(
        &mut pkg2,
        settings.starting_id_for_footnotes_endnotes + 2000,
    )?;

    let main1 = pkg1
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());
    let main2 = pkg2
        .main_document_part()
        .unwrap_or_else(|| "word/document.xml".to_string());

    let xml1 = pkg1.part_string(&main1).ok_or_else(|| {
        OpcError::PartNotFound(format!("original main document missing: {main1}"))
    })?;
    let xml2 = pkg2.part_string(&main2).ok_or_else(|| {
        OpcError::PartNotFound(format!("modified main document missing: {main2}"))
    })?;
    // Strict/ISO OOXML uses purl.oclc.org namespace URIs; normalize to Transitional
    // (the only variant our XName tables model) so the body/markup is recognized.
    let xml1 = normalize_strict_namespaces(&xml1);
    let xml2 = normalize_strict_namespaces(&xml2);

    // Parse both into one arena so the comparer can work across them.
    let mut dom = Dom::new();
    let d1 = dom.parse_xdocument(&xml1);
    let d2 = dom.parse_xdocument(&xml2);
    let root1 = dom
        .root(d1)
        .ok_or_else(|| OpcError::PartNotFound("original has no root element".into()))?;
    let root2 = dom
        .root(d2)
        .ok_or_else(|| OpcError::PartNotFound("modified has no root element".into()))?;
    let body1 = merged_body(&mut dom, root1)
        .ok_or_else(|| OpcError::PartNotFound("original has no w:body".into()))?;
    let body2 = merged_body(&mut dom, root2)
        .ok_or_else(|| OpcError::PartNotFound("modified has no w:body".into()))?;

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

    // Word mode: each side's unstyled-paragraph line pitch, for the
    // demo-default spacing strip.
    if settings.merge_replaced_paragraphs {
        for (pkg, root) in [(&pkg1, root1), (&pkg2, root2)] {
            let line = pkg
                .part_string("word/styles.xml")
                .map_or_else(|| "240".to_string(), |x| default_paragraph_line(&x));
            dom.set_attribute_value(root, &crate::namespaces::PT::default_line(), Some(&line));
            if pkg
                .part_string("word/styles.xml")
                .is_some_and(|x| has_default_table_style(&x))
            {
                dom.set_attribute_value(
                    root,
                    &crate::namespaces::PT::has_default_table_style(),
                    Some("1"),
                );
            }
        }
    }
    let result_root = crate::comparer::compare_bodies_faithful_with_notes(
        &mut dom,
        root1,
        root2,
        body1,
        body2,
        settings,
        Some(&mut notes_ctx),
    );
    for root in [root1, root2] {
        dom.set_attribute_value(root, &crate::namespaces::PT::default_line(), None);
        dom.set_attribute_value(
            root,
            &crate::namespaces::PT::has_default_table_style(),
            None,
        );
    }

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
        adopt_revised_header_footer(&mut dom, result_root, &pkg2, &mut out, &main1, settings);
        // Word inheritance: drop body-final HF slots already set on an earlier
        // mid-section break (dual chrome otherwise). Mid multi-section copies stay.
        strip_final_sectpr_inherited_header_footer(
            &mut dom,
            result_root,
            &final_header_footer_slots(&pkg2),
        );
        // M383: A-only HF (B has no headers/footers) → pure-D (Word).
        mark_a_only_hf_content_as_deleted(&mut out, &pkg1, &pkg2, settings);
    }

    // M35: comments carryover is a package-validity invariant, not a
    // Word-visual formatting pass. Both supported comparer presets must union
    // comment parts and re-inject their anchors; otherwise PowerTools-faithful
    // output can retain an original comment definition after its source
    // paragraph becomes a deletion while silently losing the anchor triplet.
    let has_comments = pkg1.part_string("word/comments.xml").is_some()
        || pkg2.part_string("word/comments.xml").is_some();
    crate::comparer::comments::carry_comments(
        &mut dom,
        result_root,
        (&pkg1, &main1),
        (&pkg2, &main2),
        (&mut out, &main1),
        &settings.author_for_revisions,
    );
    if has_comments {
        // Comment anchors keep source ids (aligned with comments.xml). Re-run
        // revision renumber with those ids reserved so move/tblPrChange never
        // share an id with commentRange* (Word "unreadable content").
        crate::comparer::finalize::fix_up_revision_ids(&mut dom, &[result_root]);
    }

    // M472 — re-assert Word's ins-before-del replacement order after the
    // comment-carry rebuilds run blocks (diff_before10 × diff_before11: the
    // early reorder's work was undone by comment anchoring, leaving
    // [ins][del][ins] where Word emits ins…ins del). Idempotent and
    // text-preserving.
    crate::comparer::finalize::reorder_replacements_ins_before_del(&mut dom, result_root);

    // M463 — rewrite outer-wrapped math revisions into Word's internal form.
    // Must run after every mesh/finalize pass (they reason about the outer
    // wrap) and before serialize.
    crate::comparer::produce::convert_outer_math_wraps_to_internal(&mut dom, result_root, settings);

    // Word keeps both documents' body bookmarks (union by name); without
    // them every updated TOC/REF field prints "Error! Bookmark not defined."
    // Runs on the finished body so no later pass moves the markers.
    crate::comparer::bookmarks::carry_bookmarks(
        &mut dom,
        result_root,
        (&pkg1, &main1),
        (&pkg2, &main2),
        &settings.author_for_revisions,
    );

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
    // Word-mode also: adopt missing docDefaults/latentStyles from B, then
    // canonicalize styleIds (numeric/`styleN` → Heading1/…) and remap refs.
    let mut style_renames: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    // POSTSTEP-STYLES-CACHE-01: the style-validity passes each re-parse the
    // (often large) output stylesheet — 3-4× per compare dominates medium-doc
    // poststeps. Capture the defined style-id set here, after styles-copy has
    // finished mutating `word/styles.xml` (copy-missing + canonicalize), so the
    // pStyle/rStyle strip below reuses it instead of re-parsing. Nothing between
    // here and that strip changes the style-id SET (the main-doc / spacing
    // passes touch the body, not styles.xml), so the cached set is exact.
    let mut cached_defined_styles: Option<std::collections::HashSet<String>> = None;
    // POSTSTEP-STYLES-CACHE-02: keep the parsed styles arena (out stylesheet
    // `tr`, revised stylesheet `fr`) alive so the M-PAG Normal-merge below can
    // reuse it instead of re-parsing both stylesheets. styles-copy still
    // serializes the stylesheet (so a compare that skips M-PAG is unaffected);
    // M-PAG then mutates the same in-memory `tr` and re-serializes. serialize→
    // parse is a lossless structural round-trip, so `tr` is identical to what
    // M-PAG would have re-parsed.
    let mut styles_arena: Option<(Dom, NodeId, NodeId)> = None;
    for (part, is_styles) in [("word/styles.xml", true), ("word/numbering.xml", false)] {
        match (out.part_string(part), pkg2.part_string(part)) {
            (Some(to_xml), Some(from_xml)) => {
                let mut sd = Dom::new();
                let td = sd.parse_xdocument(&to_xml);
                let fd = sd.parse_xdocument(&from_xml);
                if let (Some(tr), Some(fr)) = (sd.root(td), sd.root(fd)) {
                    if is_styles {
                        if settings.merge_replaced_paragraphs {
                            let a_ids = crate::comparer::footnotes::defined_style_ids(&sd, tr);
                            materialize_builtin_paragraph_defaults(&mut sd, tr);
                            materialize_builtin_paragraph_defaults(&mut sd, fr);
                            let b_to_out = copy_missing_styles_by_name(&mut sd, tr, fr);
                            let _ = adopt_missing_styles_structure(&mut sd, tr, fr);
                            style_renames = canonicalize_style_ids(&mut sd, tr);
                            // B's content names paired styles by B's ids; send
                            // them to the output style's final id. An id A
                            // also declares names A's own style: leave it. The
                            // copied B styles link to them by B's ids too.
                            let mut b_renames = std::collections::HashMap::new();
                            for (b_id, out_id) in b_to_out {
                                let target = style_renames.get(&out_id).cloned().unwrap_or(out_id);
                                if b_id != target && !a_ids.contains(&b_id) {
                                    b_renames.insert(b_id, target);
                                }
                            }
                            for local in ["basedOn", "next", "link"] {
                                for e in sd.descendants(tr, Some(&W::name(local))) {
                                    if let Some(to) =
                                        sd.attribute(e, &W::val()).and_then(|v| b_renames.get(v))
                                    {
                                        let to = to.clone();
                                        sd.set_attribute_value(e, &W::val(), Some(&to));
                                    }
                                }
                            }
                            style_renames.extend(b_renames);
                        } else {
                            crate::comparer::footnotes::copy_missing_styles(&mut sd, tr, fr);
                        }
                    } else {
                        let (num_remap, copied_bullets) =
                            crate::comparer::footnotes::copy_missing_numbering(&mut sd, tr, fr);
                        // B's picture bullets draw through B's numbering rels;
                        // give each its own relationships, since the same ids
                        // may already name A's images here.
                        for bullet in copied_bullets {
                            for el in sd.descendants_and_self(bullet, None) {
                                for (an, rid) in sd.attributes(el) {
                                    if an.namespace_name() != crate::namespaces::R::URI {
                                        continue;
                                    }
                                    let carried = crate::comparer::parts::carry_relationship(
                                        &mut out,
                                        part,
                                        &pkg2,
                                        part,
                                        &rid,
                                        |_| true,
                                    );
                                    sd.set_attribute_value(el, &an, carried.as_deref());
                                }
                            }
                        }
                        // M482: colliding B numIds were renumbered in the
                        // merged numbering part — rewrite the refs inside
                        // B-INSERTED paragraphs (mark rPr carries w:ins) or
                        // they resolve against A's same-id definitions
                        // (bullets where B's decimals should render).
                        // Word mode also moves unchanged paragraphs to B's
                        // list and records A's numId in a pPrChange: the
                        // circle bullet struck, the disc inserted (native
                        // bullet circle × disc).
                        if !num_remap.is_empty()
                            && let Some(doc_xml) = out.part_string(&main1)
                        {
                            let mut pd = Dom::new();
                            let dd = pd.parse_xdocument(&doc_xml);
                            if let Some(droot) = pd.root(dd) {
                                let mut changed = false;
                                // Comment anchors and carried bookmarks keep their
                                // ids; stay clear of both (Word never shares one).
                                let mut next_id = [
                                    "commentRangeStart",
                                    "commentRangeEnd",
                                    "commentReference",
                                    "bookmarkStart",
                                    "bookmarkEnd",
                                ]
                                .into_iter()
                                .flat_map(|n| pd.descendants(droot, Some(&W::name(n))))
                                .filter_map(|c| pd.attribute(c, &W::id())?.parse::<u32>().ok())
                                .map(|n| n + 1)
                                .fold(next_free_revision_id(&pd, droot), u32::max);
                                for p in pd.descendants(droot, Some(&W::name("p"))) {
                                    let Some(ppr) = pd.element(p, &W::p_pr()) else {
                                        continue;
                                    };
                                    let mark = pd.element(ppr, &W::r_pr());
                                    let mark_has = |n: &str| {
                                        mark.is_some_and(|r| pd.element(r, &W::name(n)).is_some())
                                    };
                                    let mark_inserted = mark_has("ins");
                                    let unchanged = settings.merge_replaced_paragraphs
                                        && !mark_inserted
                                        && !mark_has("del");
                                    if !mark_inserted && !unchanged {
                                        continue;
                                    }
                                    let Some(nid) = pd
                                        .element(ppr, &W::name("numPr"))
                                        .and_then(|np| pd.element(np, &W::name("numId")))
                                    else {
                                        continue;
                                    };
                                    let cur = pd.attribute(nid, &W::val()).map(str::to_string);
                                    let Some(new_id) =
                                        cur.as_deref().and_then(|c| num_remap.get(c)).cloned()
                                    else {
                                        continue;
                                    };
                                    if unchanged && pd.element(ppr, &W::name("pPrChange")).is_none()
                                    {
                                        let old_ppr = pd.new_element(W::p_pr());
                                        for c in pd.elements(ppr, None) {
                                            if pd.name(c).is_some_and(|n| {
                                                n == W::r_pr() || n == W::name("sectPr")
                                            }) {
                                                continue;
                                            }
                                            let clone = pd.clone_subtree(c);
                                            pd.add(old_ppr, clone);
                                        }
                                        let chg = pd.new_element(W::name("pPrChange"));
                                        pd.set_attribute_value(
                                            chg,
                                            &W::name("id"),
                                            Some(&next_id.to_string()),
                                        );
                                        next_id += 1;
                                        pd.set_attribute_value(
                                            chg,
                                            &W::name("author"),
                                            Some(&settings.author_for_revisions),
                                        );
                                        pd.set_attribute_value(
                                            chg,
                                            &W::name("date"),
                                            Some(&settings.date_time_for_revisions),
                                        );
                                        pd.add(chg, old_ppr);
                                        pd.add(ppr, chg);
                                    }
                                    pd.set_attribute_value(nid, &W::val(), Some(&new_id));
                                    changed = true;
                                }
                                if changed {
                                    out.set_part(&main1, pd.serialize_element(droot).into_bytes());
                                }
                            }
                        }
                    }
                    if is_styles {
                        cached_defined_styles =
                            Some(crate::comparer::footnotes::defined_style_ids(&sd, tr));
                    }
                    out.set_part(part, sd.serialize_element(tr).into_bytes());
                    if is_styles {
                        // Hand the parsed arena (out `tr`, revised `fr`) to M-PAG.
                        styles_arena = Some((sd, tr, fr));
                    }
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
            // A has no styles part, B does: copy B, canonicalize, then M462 —
            // swap in Word's FACTORY docDefaults/theme and bake B-effective
            // metrics into each style (Word scaffolds from its blank document,
            // not from B: tiff_image × two_column oracle).
            (None, Some(from_xml)) if is_styles && settings.merge_replaced_paragraphs => {
                let mut sd = Dom::new();
                let fd = sd.parse_xdocument(&from_xml);
                if let Some(fr) = sd.root(fd) {
                    style_renames = canonicalize_style_ids(&mut sd, fr);
                    factory_scaffold_bake_b_styles(&mut sd, fr, settings);
                    let theme_parts: Vec<String> = out
                        .parts()
                        .into_iter()
                        .filter(|p| p.starts_with("word/theme/") && p.ends_with(".xml"))
                        .collect();
                    for tp in theme_parts {
                        out.set_part(
                            &tp,
                            crate::word_default_theme::WORD_DEFAULT_THEME
                                .as_bytes()
                                .to_vec(),
                        );
                    }
                    out.set_part(part, sd.serialize_element(fr).into_bytes());
                    out.add_content_type_override(
                        "/word/styles.xml",
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                    );
                    let has_rel = out
                        .read_rels_for(&main1)
                        .is_some_and(|r| r.items.iter().any(|i| i.rel_type.ends_with("/styles")));
                    if !has_rel {
                        out.add_document_relationship(
                            &main1,
                            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                            "styles.xml",
                        );
                    }
                }
            }
            // A has styles, B has none: still canonicalize A's ids (Word does).
            (Some(to_xml), None) if is_styles && settings.merge_replaced_paragraphs => {
                let mut sd = Dom::new();
                let td = sd.parse_xdocument(&to_xml);
                if let Some(tr) = sd.root(td) {
                    style_renames = canonicalize_style_ids(&mut sd, tr);
                    out.set_part(part, sd.serialize_element(tr).into_bytes());
                }
            }
            _ => {}
        }
    }
    // Remap pStyle/rStyle/tblStyle in every XML part that can carry them.
    if settings.merge_replaced_paragraphs && !style_renames.is_empty() {
        let part_names: Vec<String> = out
            .parts()
            .into_iter()
            .filter(|p| {
                p.ends_with(".xml")
                    && (p.starts_with("word/document")
                        || p.starts_with("word/header")
                        || p.starts_with("word/footer")
                        || p.starts_with("word/footnotes")
                        || p.starts_with("word/endnotes")
                        || p.starts_with("word/comments")
                        || p == "word/styles.xml")
            })
            .collect();
        for part in part_names {
            let Some(xml) = out.part_string(&part) else {
                continue;
            };
            let mut pd = Dom::new();
            let doc = pd.parse_xdocument(&xml);
            let Some(root) = pd.root(doc) else {
                continue;
            };
            // styles.xml basedOn/next/link already remapped inside canonicalize.
            if part == "word/styles.xml" {
                continue;
            }
            if remap_style_refs(&mut pd, root, &style_renames) > 0 {
                out.set_part(&part, pd.serialize_element(root).into_bytes());
            }
        }
    }

    // The revised text brings its fonts: their table entries come too.
    if let (Some(out_fonts), Some(b_fonts)) = (
        out.part_string("word/fontTable.xml"),
        pkg2.part_string("word/fontTable.xml"),
    ) && let Some(merged) = merge_revised_font_table(&out_fonts, &b_fonts)
    {
        out.set_part("word/fontTable.xml", merged.into_bytes());
    }
    // Word-mode: adopt B's package chrome (settings/fontTable/theme) when A is
    // thin. When BOTH sides are bare demos (C5 formatting one-pagers), Word
    // still saves factory settings/theme/fontTable — inject if still missing.
    if settings.merge_replaced_paragraphs {
        adopt_revised_styles_chrome(&mut out, &pkg2, &main1);
        ensure_factory_package_chrome(&mut out, &main1);
        repair_missing_core_relationships(&mut out, &main1);
        merge_custom_properties(&mut out, &pkg2);
        // M492: deleted paragraphs keep their A-ORIGINAL direct spacing.
        if let (Some(a_xml), Some(out_xml)) = (
            pkg1.part_string("word/document.xml"),
            out.part_string(&main1),
        ) && let Some(restored) = restore_deleted_paragraph_spacing(&a_xml, &out_xml)
        {
            out.set_part(&main1, restored.into_bytes());
        }
        // M491: B's DOCUMENT-FINAL paragraph mark never materializes as a
        // mid-document insertion — Word EQ-pairs the two documents' final
        // marks. Our windowed diff can splice B's terminal empty ¶ into the
        // body as an ins-para; the extra rendered line cascades every page
        // break below it (h_f_normal × sd_1495: paraId 5352EB15, 46.66 with
        // its content-ceiling at 97.13; 16-pair census). Remove the
        // ins-marked EMPTY paragraph carrying B's final paraId unless it is
        // itself the output's final paragraph.
        if let (Some(b_xml), Some(out_xml)) = (
            pkg2.part_string("word/document.xml"),
            out.part_string(&main1),
        ) {
            let b_final_pid = {
                let mut bd = Dom::new();
                let d = bd.parse_xdocument(&b_xml);
                bd.root(d)
                    .and_then(|r| bd.element(r, &W::body()))
                    .and_then(|body| {
                        let bdom = bd;
                        let paras: Vec<NodeId> = bdom
                            .elements(body, None)
                            .into_iter()
                            .filter(|&k| bdom.name_is(k, &W::p()))
                            .collect();
                        paras.last().and_then(|&p| {
                            bdom.attribute(p, &crate::namespaces::W14::name("paraId"))
                                .map(str::to_string)
                        })
                    })
            };
            if let Some(bf) = b_final_pid {
                let mut pd = Dom::new();
                let d = pd.parse_xdocument(&out_xml);
                if let Some(root) = pd.root(d)
                    && let Some(body) = pd.element(root, &W::body())
                {
                    let paras: Vec<NodeId> = pd
                        .elements(body, None)
                        .into_iter()
                        .filter(|&k| pd.name_is(k, &W::p()))
                        .collect();
                    let w14_pid = crate::namespaces::W14::name("paraId");
                    let target = paras.iter().enumerate().find(|&(i, &p)| {
                        i + 1 != paras.len() && pd.attribute(p, &w14_pid) == Some(bf.as_str())
                    });
                    if let Some((idx, &p)) = target {
                        let is_ins_empty = |pd: &Dom, q: NodeId| {
                            let mark_ins = pd
                                .element(q, &W::p_pr())
                                .and_then(|pr| pd.element(pr, &W::r_pr()))
                                .is_some_and(|r| pd.element(r, &W::name("ins")).is_some());
                            // A paragraph whose only content is a real embedded
                            // object (DrawingML image/shape, OLE object, a VML
                            // picture, a text box, or an AlternateContent shape)
                            // carries no w:t, but it is NOT empty — removing it
                            // drops the graphic (vrect_node × wmf_emf: B's final ¶
                            // is a full-page WMF, misjudged empty → whole page lost,
                            // 18.21). A BARE w:pict horizontal rule (o:hr `v:rect`,
                            // no imagedata/textbox) is decorative, not content, and
                            // must stay droppable — checking for the real content
                            // markers (not the w:pict wrapper) avoids regressing the
                            // trailing-hr-spacer pairs (sd_2517 hr rules).
                            let has_embedded = [
                                W::name("drawing"),
                                W::object(),
                                crate::namespaces::MC::name("AlternateContent"),
                                crate::namespaces::VML::name("imagedata"),
                                W::name("txbxContent"),
                                crate::namespaces::WNE::name("txbxContent"),
                            ]
                            .iter()
                            .any(|nm| !pd.descendants(q, Some(nm)).is_empty());
                            mark_ins
                                && !has_embedded
                                && !pd
                                    .descendants(q, Some(&W::t()))
                                    .iter()
                                    .any(|&t| !pd.value_str(t).trim().is_empty())
                                && pd.descendants(q, Some(&W::del_text())).is_empty()
                        };
                        // Run gate: Word trims B's final ¶ out of a RUN of
                        // inserted empties (h_f: three in a row, two kept);
                        // a LONE inserted empty spacer stays — Word keeps it
                        // under another identity (file_82 × file_83, M389:
                        // stripping it cost −18.9).
                        let prev_also_empty = idx > 0 && is_ins_empty(&pd, paras[idx - 1]);
                        if is_ins_empty(&pd, p) && prev_also_empty {
                            pd.remove(p);
                            out.set_part(&main1, pd.serialize_element(root).into_bytes());
                        }
                    }
                }
            }
        }
    }

    // Word-parity: strip pStyle/rStyle that styles.xml does not define. LO maps
    // built-in names (Heading1, Title, …) even when the style entry is absent;
    // Word's redline omits the attribute entirely (heading_1_bold×heading_1_style).
    if settings.merge_replaced_paragraphs {
        // POSTSTEP-STYLES-CACHE-01: reuse the defined-style-id set captured after
        // styles-copy instead of re-parsing word/styles.xml. Fall back to a fresh
        // parse only when styles-copy did not run (no output stylesheet).
        let defined = cached_defined_styles.clone().or_else(|| {
            out.part_string("word/styles.xml").map(|sx| {
                let mut sd = Dom::new();
                let doc = sd.parse_xdocument(&sx);
                sd.root(doc)
                    .map(|r| crate::comparer::footnotes::defined_style_ids(&sd, r))
                    .unwrap_or_default()
            })
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
    if settings.merge_replaced_paragraphs {
        // POSTSTEP-STYLES-CACHE-02: reuse the arena styles-copy already parsed;
        // fall back to a fresh parse of both stylesheets only when it did not run.
        let arena = styles_arena.take().or_else(|| {
            match (
                out.part_string("word/styles.xml"),
                pkg2.part_string("word/styles.xml"),
            ) {
                (Some(out_xml), Some(b_xml)) => {
                    let mut sd = Dom::new();
                    let od = sd.parse_xdocument(&out_xml);
                    let bd = sd.parse_xdocument(&b_xml);
                    match (sd.root(od), sd.root(bd)) {
                        (Some(or), Some(br)) => Some((sd, or, br)),
                        _ => None,
                    }
                }
                _ => None,
            }
        });
        if let Some((mut sd, or, br)) = arena {
            // Workstream S runs FIRST, while the output stylesheet still holds
            // A's definitions: `merge_normal_style_*` below rewrites Normal to
            // B's values, and every style based on Normal would then resolve its
            // "original" chain against the already-revised Normal.
            // S2: the styles A actually declared, by (type, name) — styles in
            // `or` beyond this set were copied from B and take the phase-2
            // docDefaults bake.
            let a_declared_keys: std::collections::HashSet<(String, String)> = pkg1
                .part_string("word/styles.xml")
                .map(|xml| {
                    let mut ad = Dom::new();
                    let doc = ad.parse_xdocument(&xml);
                    ad.root(doc)
                        .map(|r| {
                            ad.elements(r, Some(&W::name("style")))
                                .into_iter()
                                .filter_map(|s| style_match_key(&ad, s))
                                .collect()
                        })
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            // Word leaves a style the revision's stylesheet lacks as the
            // original defines it, without a change record (51 of 51 accept
            // pairs), whatever Normal and the docDefaults become under it:
            // the passes below only see it to be put back.
            let only_a = styles_the_revision_lacks(&mut sd, or, br);
            let mut changed =
                merge_revised_style_definitions(&mut sd, or, br, settings, &a_declared_keys);
            changed |= merge_normal_style_spacing(&mut sd, or, br, settings);
            // M-PAG mechanism 2b / M71: rewrite Normal rPr to B's effective
            // metrics when they differ. Formerly gated on header/footer→Normal
            // (footer knife-edge). That skipped file_197 (no HF): Word writes
            // B's Calibri dd onto Normal + rPrChange(A Ubuntu); we kept A.
            // M65 still skips both-bare (file_170). HF-linked cases unchanged.
            changed |= merge_normal_style_rpr(&mut sd, or, br, settings);
            // M467: Word keeps only per-attr DELTAS vs the output docDefaults
            // on the merged Normal (tab_test × table_autofit: kern/szCs/
            // eastAsiaTheme/ligatures dropped as redundant, Arial+sz=20 kept).
            changed |= prune_normal_rpr_context_equal_attrs(&mut sd, or);
            // M480b: dd-delta disabling neutralizers on both-sides merged
            // styles — after the Normal merges (so a stamped Normal reads as
            // a provider), before M111 adds cascade records.
            changed |= bake_bothsides_dd_disabling_neutralizers(&mut sd, or, br);
            // M111: cascade Normal pPrChange/rPrChange onto basedOn=Normal styles
            // (ListParagraph/BodyText/Header/… — file_130 Word has ~30).
            changed |= cascade_normal_change_to_based_styles(&mut sd, or, settings);
            // M79: Word single-line on Heading/Title/ListParagraph (file_33 3→2pp).
            changed |= normalize_word_paragraph_style_line(&mut sd, or);
            // M80: Title/ListParagraph/Highlighted Arial + Heading Latin inherit.
            changed |= align_paragraph_style_fonts_with_normal(&mut sd, or);
            // Redefined styles take B's effective metrics as a delta against
            // the output context (Word's rule, mined over 4,924 styles). Runs
            // last so it settles what the heuristic passes above wrote.
            changed |= resolve_redefined_style_metrics(&mut sd, or, br);
            // Root styles record their old properties in full against the
            // docDefaults, the way Word's Reject All needs them.
            changed |= complete_root_style_change_records(&mut sd, or, settings);
            // Below the roots, the record carries what the original's chain
            // gave the style, which Reject All would otherwise read as built-in.
            let a_styles = pkg1.part_string("word/styles.xml").and_then(|xml| {
                let doc = sd.parse_xdocument(&xml);
                sd.root(doc)
            });
            if let Some(a_root) = a_styles {
                changed |= complete_based_style_change_records(&mut sd, or, a_root);
            }
            changed |= restore_styles(&mut sd, or, only_a);
            // M483: re-cache themed color hexes against the shipped theme —
            // must run AFTER the merge writes B's blocks (their w:val hexes
            // were cached under B's theme).
            if let Some(theme_xml) = out.part_string("word/theme/theme1.xml") {
                changed |= reresolve_theme_color_hexes(&mut sd, or, &theme_xml);
            }
            if changed {
                out.set_part("word/styles.xml", sd.serialize_element(or).into_bytes());
            }
            // M487 — bake B-effective paragraph spacing onto B-INSERTED
            // paragraphs. The output ships A's docDefaults, so an inserted
            // paragraph styled by B renders with the wrong spacing unless its
            // pPr carries B's effective values directly. Word synthesizes
            // exactly this (rstyle_combos × pre_separated_list oracle: every
            // inserted ListParagraph gains `w:spacing w:after="0" w:line=
            // "240" w:lineRule="auto"` that B's source never declared;
            // content-ceiling for the pair 44.83 → 100.00). The M480
            // neutralizer principle at the document.xml level.
            {
                let index =
                    |dom: &Dom, root: NodeId| -> std::collections::HashMap<String, NodeId> {
                        dom.elements(root, Some(&W::name("style")))
                            .into_iter()
                            .filter_map(|s| {
                                Some((dom.attribute(s, &W::name("styleId"))?.to_string(), s))
                            })
                            .collect()
                    };
                let out_idx = index(&sd, or);
                let b_idx = index(&sd, br);
                // B's counterpart of an output style id: the style of the same
                // (type, name), as Word pairs them — ids name nothing across
                // documents (b4cd671041's revision calls List Paragraph
                // `Listaszerbekezds` and Table Grid `Rcsostblzat`).
                let mut b_by_key: std::collections::HashMap<(String, String), String> =
                    std::collections::HashMap::new();
                for (id, &s) in &b_idx {
                    if let Some(k) = style_match_key(&sd, s) {
                        b_by_key.entry(k).or_insert_with(|| id.clone());
                    }
                }
                let b_id_for = |out_id: &str| -> Option<String> {
                    out_idx
                        .get(out_id)
                        .and_then(|&s| style_match_key(&sd, s))
                        .and_then(|k| b_by_key.get(&k).cloned())
                        .or_else(|| b_idx.contains_key(out_id).then(|| out_id.to_string()))
                };
                let (b_default, out_default) = {
                    let dp = |dom: &Dom, root: NodeId| -> Option<String> {
                        dom.elements(root, Some(&W::name("style")))
                            .into_iter()
                            .find(|&s| {
                                dom.attribute(s, &W::name("type")) == Some("paragraph")
                                    && matches!(
                                        dom.attribute(s, &W::name("default")),
                                        Some("1") | Some("true")
                                    )
                            })
                            .and_then(|s| dom.attribute(s, &W::name("styleId")).map(str::to_string))
                    };
                    (dp(&sd, br), dp(&sd, or))
                };
                if let Some(doc_xml) = out.part_string(&main1) {
                    let mut pd = Dom::new();
                    let dd = pd.parse_xdocument(&doc_xml);
                    if let Some(droot) = pd.root(dd) {
                        let mut doc_changed = false;
                        for p in pd.descendants(droot, Some(&W::name("p"))) {
                            let Some(ppr) = pd.element(p, &W::p_pr()) else {
                                continue;
                            };
                            let mark_inserted = pd
                                .element(ppr, &W::r_pr())
                                .is_some_and(|r| pd.element(r, &W::name("ins")).is_some());
                            if !mark_inserted {
                                continue;
                            }
                            // EMPTY inserted paragraphs keep their bare pPr —
                            // the oracle never bakes spacing onto them, and the
                            // empty line's default height decides the page
                            // break (rstyle_combos: baking onto the one empty
                            // separator para cost the whole second page,
                            // 98.18 → 44.70).
                            let has_text = pd
                                .descendants(p, Some(&W::t()))
                                .iter()
                                .any(|&t| !pd.value_str(t).trim().is_empty());
                            if !has_text {
                                continue;
                            }
                            // STYLED OR NUMBERED paragraphs only: the oracle
                            // bakes pStyle'd paras and pStyle-less LIST paras
                            // (numPr present — rstyle "List item 2"/"Back"),
                            // never truly-bare ones (m370 title;
                            // tiff×h_f_normal regressed −56 when its 52 bare
                            // inserted paras took the bake).
                            let pstyle = pd
                                .element(ppr, &W::name("pStyle"))
                                .and_then(|ps| pd.attribute(ps, &W::val()))
                                .map(str::to_string);
                            let has_numpr = pd.element(ppr, &W::name("numPr")).is_some();
                            if pstyle.is_none() && !has_numpr {
                                continue;
                            }
                            let (Some(out_style), Some(b_style)) = (match pstyle {
                                Some(id) => (Some(id.clone()), b_id_for(&id)),
                                None => (out_default.clone(), b_default.clone()),
                            }) else {
                                continue; // style not from B — no B-effective target
                            };
                            // The innermost table's style, on each side.
                            let out_table = pd
                                .ancestors(p, Some(&W::tbl()))
                                .first()
                                .and_then(|&t| pd.element(t, &W::tbl_pr()))
                                .and_then(|t| pd.element(t, &W::name("tblStyle")))
                                .and_then(|t| pd.attribute(t, &W::val()))
                                .map(str::to_string);
                            let b_table = out_table.as_deref().and_then(b_id_for);
                            let (b_eff, b_declared, _) = effective_para_spacing(
                                &sd,
                                br,
                                &b_idx,
                                &b_style,
                                b_table.as_deref(),
                            );
                            let (o_eff, _, _) = effective_para_spacing(
                                &sd,
                                or,
                                &out_idx,
                                &out_style,
                                out_table.as_deref(),
                            );
                            if b_eff == o_eff {
                                continue;
                            }
                            let sp = pd.element(ppr, &W::name("spacing"));
                            let attr_names = ["after", "before", "line", "lineRule"];
                            let mut to_write: Vec<(usize, String)> = Vec::new();
                            for i in 0..4 {
                                if b_eff[i] == o_eff[i] || b_declared[i] {
                                    continue;
                                }
                                let declared = sp.is_some_and(|s| {
                                    pd.attribute(s, &W::name(attr_names[i])).is_some()
                                });
                                if !declared {
                                    to_write.push((i, b_eff[i].clone()));
                                }
                            }
                            // line without lineRule renders as exact twips —
                            // carry the rule whenever line is written.
                            if to_write.iter().any(|(i, _)| *i == 2)
                                && !to_write.iter().any(|(i, _)| *i == 3)
                                && !sp.is_some_and(|s| {
                                    pd.attribute(s, &W::name("lineRule")).is_some()
                                })
                            {
                                to_write.push((3, b_eff[3].clone()));
                            }
                            if to_write.is_empty() {
                                continue;
                            }
                            let sp = match sp {
                                Some(s) => s,
                                None => {
                                    let s = pd.new_element(W::name("spacing"));
                                    insert_child_by_rank(
                                        &mut pd,
                                        ppr,
                                        s,
                                        "spacing",
                                        &ppr_child_rank,
                                    );
                                    s
                                }
                            };
                            for (i, v) in to_write {
                                pd.set_attribute_value(sp, &W::name(attr_names[i]), Some(&v));
                                doc_changed = true;
                            }
                        }
                        if doc_changed {
                            out.set_part(&main1, pd.serialize_element(droot).into_bytes());
                        }
                    }
                }
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
    //
    // Remap style refs on notes_ctx *before* serialize/writeback so B.4 does
    // not clobber an earlier styles pass with un-remapped note content.
    if settings.merge_replaced_paragraphs && !style_renames.is_empty() {
        for note_root in [notes_ctx.fn_with_revisions, notes_ctx.en_with_revisions]
            .into_iter()
            .flatten()
        {
            remap_style_refs(&mut dom, note_root, &style_renames);
        }
    }
    let mut footnote_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut endnote_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (part, root, is_fn) in [
        (fn1.as_str(), notes_ctx.fn_with_revisions, true),
        (en1.as_str(), notes_ctx.en_with_revisions, false),
    ] {
        if let Some(r) = root {
            let def = if is_fn { W::footnote() } else { W::endnote() };
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
    // M379: after B.4 notes writeback — if A had no separators, B.4 left empty
    // shells or skipped; copy B's separator/continuationSeparator notes.
    if settings.merge_replaced_paragraphs {
        adopt_b_notes_when_a_lacks_separators(&mut out, &pkg1, &pkg2, &main1);
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
            out.set_part(
                "word/settings.xml",
                sd.serialize_element(sroot).into_bytes(),
            );
        }
    }
    // M4.H.x: header/footer CONTENT diff (Word redlines header/footer changes; we
    // previously only copied the original's). Match A's parts to B's by reference
    // (kind,type), diff the content and write the redline into the output's
    // (original's) part. That part keeps A's rels, so a part whose B references
    // (a logo, a hyperlink) would resolve to something else there is skipped.
    {
        let refs_b = header_footer_refs(&pkg2);
        let mut ordinals: std::collections::HashMap<(String, String), usize> =
            std::collections::HashMap::new();
        let mut diffed = std::collections::HashSet::new();
        for (kind, ty, part_a) in header_footer_refs(&pkg1) {
            let ordinal = ordinals.entry((kind.clone(), ty.clone())).or_default();
            let at = *ordinal;
            *ordinal += 1;
            if !diffed.insert(part_a.clone()) {
                continue;
            }
            let Some(part_b) = pair_header_footer(&pkg1, &pkg2, &refs_b, (&kind, &ty, &part_a), at)
            else {
                continue;
            };
            let part_b = &part_b;
            if let (Some(xa), Some(xb)) = (pkg1.part_string(&part_a), pkg2.part_string(part_b)) {
                // B's references that mean something else in A's part (both
                // headers picture their own logo as rId1) move to fresh ids.
                let xb = if part_rels_agree((&pkg1, &part_a), (&pkg2, part_b), &xb) {
                    xb
                } else {
                    match carry_revised_part_relationships(&mut out, &part_a, (&pkg2, part_b), &xb)
                    {
                        Some(x) => x,
                        None => continue,
                    }
                };
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
                    let b_ends_with_table = hd
                        .elements(rb, None)
                        .last()
                        .is_some_and(|&e| hd.name_is(e, &W::tbl()));
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
                    if b_ends_with_table {
                        delete_story_closing_mark(&mut hd, container, settings);
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

    // Every other part resolves its own r:* references too (B's comments, notes,
    // headers/footers and picture bullets arrive without B's per-part rels).
    crate::comparer::parts::reconcile_part_relationships(&mut out, &main1, &pkg1, &pkg2);
    // …and every copied part keeps the content type its source declared (a B
    // header's `.wdp` HD Photo arrived without B's `wdp` Default).
    out.adopt_missing_content_types(&[&pkg2, &pkg1]);

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
                // Invalidity we inherit rather than create: a source whose own
                // styles/numbering Word already rejects would otherwise ship inside
                // our redline and be blamed on us.
                crate::comparer::finalize::repair_inherited_invalidity(&mut vd, vr);
                // Headers and footers deleted wholesale never pass through the body
                // finalize pipeline, so this is where their w:del/w:hyperlink
                // inversion gets fixed.
                crate::comparer::finalize::hoist_hyperlinks_out_of_revisions(&mut vd, vr);
                crate::comparer::finalize::enforce_deleted_text_kinds(&mut vd, vr);
                crate::comparer::finalize::remove_powertools_scratch_markup(&mut vd, vr);
                // Produce strips an empty pPr shell, then later package steps
                // (style cascade, spacing restore) can take the live children
                // back off and leave the shell. Word's redline of the short
                // title mixes has neither the shell nor the children.
                if part == main1 {
                    crate::comparer::finalize::strip_propertyless_ppr(&mut vd, vr);
                    // Word-visual chrome. Faithful mode runs none of these;
                    // the body path already gates them the same way.
                    if settings.merge_replaced_paragraphs {
                        // Package steps can put a default nextPage back into the
                        // recorded sectPr, or leave line=276 on a deleted mark.
                        let original_line = pkg1
                            .part_string("word/styles.xml")
                            .map(|x| default_paragraph_line(&x));
                        crate::comparer::finalize::strip_unrecorded_word_defaults(
                            &mut vd,
                            vr,
                            original_line.as_deref(),
                        );
                        let id_to_target: std::collections::HashMap<String, String> = out
                            .read_rels_for(&main1)
                            .map(|rels| {
                                rels.items
                                    .iter()
                                    .filter(|r| r.rel_type.ends_with("/hyperlink"))
                                    .map(|r| (r.id.clone(), r.target.clone()))
                                    .collect()
                            })
                            .unwrap_or_default();
                        let base_targets: std::collections::HashSet<String> = pkg1
                            .read_rels_for(&main1)
                            .map(|rels| {
                                rels.items
                                    .iter()
                                    .filter(|r| r.rel_type.ends_with("/hyperlink"))
                                    .map(|r| r.target.clone())
                                    .collect()
                            })
                            .unwrap_or_default();
                        crate::comparer::finalize::rewrite_inserted_external_hyperlinks(
                            &mut vd,
                            vr,
                            &id_to_target,
                            &base_targets,
                        );
                        crate::comparer::finalize::align_word_table_and_comment_chrome(&mut vd, vr);
                    }
                    if let Some((original_xml, revised_xml)) = ladder_sources.as_ref() {
                        crate::comparer::finalize::align_remaining_ladder_rungs(
                            &mut vd,
                            vr,
                            &crate::comparer::finalize::LadderSources {
                                original_xml,
                                revised_xml,
                                settings,
                            },
                        );
                    }
                }
                // Last: every pass above may append properties out of order.
                crate::comparer::finalize::enforce_part_schema_order(&mut vd, vr);
                crate::comparer::finalize::declare_extension_namespaces_ignorable(&mut vd, vr);
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
                out.set_part(
                    "word/settings.xml",
                    sd.serialize_element(sroot).into_bytes(),
                );
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

    /// A's working copy carries `pt14:Unid` on its spacing; the restored
    /// spacing on the deleted paragraph must hold only the `w:` attributes.
    #[test]
    fn restored_deleted_spacing_leaves_scratch_unids_behind() {
        let ns = format!(
            "xmlns:w=\"{}\" xmlns:w14=\"{}\" xmlns:pt14=\"{}\"",
            W::URI,
            crate::namespaces::W14::URI,
            crate::namespaces::PT::URI
        );
        let a = format!(
            "<w:document {ns}><w:body><w:p w14:paraId=\"4B0CC135\"><w:pPr>\
             <w:spacing w:line=\"276\" w:lineRule=\"auto\" pt14:Unid=\"76\"/></w:pPr>\
             <w:r><w:t>x</w:t></w:r></w:p></w:body></w:document>"
        );
        let out = format!(
            "<w:document {ns}><w:body><w:p w14:paraId=\"4B0CC135\"><w:pPr>\
             <w:jc w:val=\"right\"/><w:rPr><w:del w:id=\"1\" w:author=\"a\"/></w:rPr></w:pPr>\
             </w:p></w:body></w:document>"
        );
        let restored = restore_deleted_paragraph_spacing(&a, &out).expect("spacing restored");
        let mut dom = Dom::new();
        let d = dom.parse_xdocument(&restored);
        let root = dom.root(d).unwrap();
        let sp = dom.descendants(root, Some(&W::name("spacing")))[0];
        let mut names: Vec<String> = dom
            .attributes(sp)
            .into_iter()
            .map(|(n, _)| n.clark())
            .collect();
        names.sort();
        let w = |l: &str| W::name(l).clark();
        assert_eq!(names, vec![w("line"), w("lineRule")], "{restored}");
    }

    /// Fonts only the revised document declares join the output's font table
    /// (file_46 × file_47: without Liberation Serif's entry Word fell back to
    /// Times New Roman where its own redline used Hiragino Mincho). Fonts the
    /// output already lists stay as they are, and embedded-font children are
    /// left behind: their relationships and obfuscation keys belong to B.
    #[test]
    fn revised_fonts_join_the_font_table() {
        let ns = format!("xmlns:w=\"{}\" xmlns:r=\"{}\"", W::URI, R::URI);
        let out = format!(
            "<w:fonts {ns}><w:font w:name=\"Calibri\"><w:charset w:val=\"00\"/></w:font></w:fonts>"
        );
        let b = format!(
            "<w:fonts {ns}><w:font w:name=\"Calibri\"><w:charset w:val=\"86\"/></w:font>\
             <w:font w:name=\"Droid Sans Fallback\"><w:charset w:val=\"86\"/>\
             <w:family w:val=\"auto\"/><w:embedRegular r:id=\"rId1\" w:fontKey=\"{{0}}\"/></w:font></w:fonts>"
        );
        let merged = merge_revised_font_table(&out, &b).expect("B's font added");
        let mut dom = Dom::new();
        let d = dom.parse_xdocument(&merged);
        let root = dom.root(d).unwrap();
        let fonts: Vec<(String, String)> = dom
            .elements(root, Some(&W::name("font")))
            .into_iter()
            .map(|f| {
                let cs = dom
                    .element(f, &W::name("charset"))
                    .and_then(|c| dom.attribute(c, &W::val()))
                    .unwrap_or("")
                    .to_string();
                (dom.attribute(f, &W::name("name")).unwrap().to_string(), cs)
            })
            .collect();
        assert_eq!(
            fonts,
            vec![
                ("Calibri".to_string(), "00".to_string()),
                ("Droid Sans Fallback".to_string(), "86".to_string())
            ],
            "{merged}"
        );
        assert!(!merged.contains("embedRegular"), "{merged}");
        assert_eq!(merge_revised_font_table(&merged, &b), None, "nothing new");
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

    /// Every `w:id` revision carrier raises the floor, not only the `*Change`
    /// records: a list, move-range or table-cell revision holding the next id
    /// would collide with a synthesized `w:pPrChange`.
    #[test]
    fn next_free_revision_id_counts_list_move_range_and_cell_carriers() {
        for (carrier, id) in [
            ("numberingChange", 40),
            ("moveFromRangeStart", 41),
            ("moveToRangeEnd", 42),
            ("cellIns", 43),
            ("cellDel", 44),
            ("cellMerge", 45),
        ] {
            let mut dom = Dom::new();
            let xml = format!(
                "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>\
                 <w:ins w:id=\"7\" w:author=\"x\" w:date=\"d\"/><w:{carrier} w:id=\"{id}\" w:author=\"x\" w:date=\"d\"/>\
                 </w:body></w:document>"
            );
            let (_root, doc) = parse(&mut dom, &xml);
            assert_eq!(next_free_revision_id(&dom, doc), id + 1, "{carrier}");
        }
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

    /// `revision_to_json` / `revisions_to_json` are the single serialization
    /// shared by the CLI (`jubarte revisions --json`) and the wasm
    /// `getRevisions` binding: exact CLI object shape, full string escaping —
    /// backslash, quote, and EVERY control char < 0x20 (document text can
    /// carry tabs, CRs, vertical tabs).
    #[test]
    fn revision_json_matches_cli_shape_and_escapes_control_chars() {
        use crate::comparer::atoms::FormatChangeInfo;
        use crate::comparer::{WmlComparerRevision, WmlComparerRevisionType};

        let inserted = WmlComparerRevision {
            revision_type: WmlComparerRevisionType::Inserted,
            text: Some("a\"b\\c\nd\te\u{000B}f".to_string()),
            author: Some("Reviewer \"X\"".to_string()),
            date: Some("2026-07-17T00:00:00Z".to_string()),
            content_element: None,
            revision_element: None,
            part_name: "word/document.xml".to_string(),
            move_group_id: Some(3),
            is_move_source: Some(true),
            format_change: None,
        };
        assert_eq!(
            revision_to_json(&inserted),
            concat!(
                "{\"type\":\"Inserted\",\"author\":\"Reviewer \\\"X\\\"\",",
                "\"date\":\"2026-07-17T00:00:00Z\",\"part\":\"word/document.xml\",",
                "\"moveGroupId\":3,\"isMoveSource\":true,\"formatChange\":null,",
                "\"text\":\"a\\\"b\\\\c\\nd\\te\\u000bf\"}"
            )
        );

        let format_changed = WmlComparerRevision {
            revision_type: WmlComparerRevisionType::FormatChanged,
            text: None,
            author: None,
            date: None,
            content_element: None,
            revision_element: None,
            part_name: "word/document.xml".to_string(),
            move_group_id: None,
            is_move_source: None,
            format_change: Some(FormatChangeInfo {
                changed_properties: vec!["bold".to_string(), "sz".to_string()],
                ..FormatChangeInfo::default()
            }),
        };
        assert_eq!(
            revision_to_json(&format_changed),
            concat!(
                "{\"type\":\"FormatChanged\",\"author\":\"\",\"date\":\"\",",
                "\"part\":\"word/document.xml\",\"moveGroupId\":null,",
                "\"isMoveSource\":null,",
                "\"formatChange\":{\"changedProperties\":[\"bold\",\"sz\"]},",
                "\"text\":\"\"}"
            )
        );

        // The wasm array shape is exactly the objects joined inside [].
        let expected_array = format!(
            "[{},{}]",
            revision_to_json(&inserted),
            revision_to_json(&format_changed)
        );
        assert_eq!(
            revisions_to_json(&[inserted, format_changed]),
            expected_array
        );
        assert_eq!(revisions_to_json(&[]), "[]");
    }

    #[test]
    fn word_canonical_style_id_preserves_toc_builtins() {
        // Regression: TOC 1..9 are ALL-CAPS built-in styleIds (TOC1..TOC9,
        // name "toc N"). The generic PascalCase fallback would mangle them to
        // Toc1.., renaming a live built-in to a custom id — LibreOffice/Word
        // then drop the built-in TOC indents + dot-leader tabs and the table of
        // contents reflows, collapsing the visual redline score.
        assert_eq!(word_canonical_style_id("toc 1"), "TOC1");
        assert_eq!(word_canonical_style_id("toc 9"), "TOC9");
        assert_eq!(word_canonical_style_id("TOC 2"), "TOC2"); // name matched case-insensitively
        assert_eq!(word_canonical_style_id("toc heading"), "TOCHeading");
        // Sibling built-ins and the generic PascalCase path are unaffected.
        assert_eq!(word_canonical_style_id("heading 1"), "Heading1");
        assert_eq!(word_canonical_style_id("document title"), "DocumentTitle");
        assert_eq!(word_canonical_style_id("my custom style"), "MyCustomStyle");
    }

    #[test]
    fn word_canonical_style_id_keeps_builtin_ids_that_differ_from_their_names() {
        // Word's own ids; the PascalCased name would rename a live built-in.
        assert_eq!(
            word_canonical_style_id("annotation reference"),
            "CommentReference"
        );
        assert_eq!(word_canonical_style_id("annotation text"), "CommentText");
        assert_eq!(
            word_canonical_style_id("annotation subject"),
            "CommentSubject"
        );
        assert_eq!(word_canonical_style_id("macro"), "MacroText");
        assert_eq!(word_canonical_style_id("toa heading"), "TOAHeading");
        assert_eq!(
            word_canonical_style_id("table of figures"),
            "TableofFigures"
        );
        assert_eq!(
            word_canonical_style_id("table of authorities"),
            "TableofAuthorities"
        );
    }

    /// Word's style ids hold letters and digits only: "Normal (Web)" is
    /// `NormalWeb` (evals comments_doc × document). Keeping the parentheses
    /// renamed B's live `NormalWeb` to a custom `Normal(Web)`.
    #[test]
    fn word_canonical_style_id_drops_punctuation() {
        assert_eq!(word_canonical_style_id("Normal (Web)"), "NormalWeb");
        assert_eq!(word_canonical_style_id("Body Text 2"), "BodyText2");
    }

    /// B's docDefaults name their fonts by theme (`w:asciiTheme="minorHAnsi"`,
    /// the form Word itself writes) and A's name Times New Roman outright.
    /// Word's live Normal carries B's theme fonts, so the output renders in
    /// Calibri as B does (instrtext_angled_brackets_bug × table_merged_cells:
    /// the theme attributes were not read, Normal kept A's Times New Roman and
    /// the pair scored 0.13 Jaccard against Word, docxodus 0.82).
    #[test]
    fn normal_takes_b_theme_fonts_from_its_doc_defaults() {
        let ns = format!("xmlns:w=\"{}\"", W::URI);
        let a = format!(
            "<w:styles {ns}><w:docDefaults><w:rPrDefault><w:rPr>\
             <w:rFonts w:ascii=\"Times New Roman\" w:eastAsia=\"Times New Roman\" \
             w:hAnsi=\"Times New Roman\" w:cs=\"Times New Roman\"/></w:rPr></w:rPrDefault>\
             </w:docDefaults><w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/><w:rPr><w:sz w:val=\"24\"/></w:rPr></w:style></w:styles>"
        );
        let b = format!(
            "<w:styles {ns}><w:docDefaults><w:rPrDefault><w:rPr>\
             <w:rFonts w:asciiTheme=\"minorHAnsi\" w:eastAsiaTheme=\"minorHAnsi\" \
             w:hAnsiTheme=\"minorHAnsi\" w:cstheme=\"minorBidi\"/><w:sz w:val=\"24\"/>\
             </w:rPr></w:rPrDefault></w:docDefaults>\
             <w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\">\
             <w:name w:val=\"Normal\"/></w:style></w:styles>"
        );
        let mut dom = Dom::new();
        let (out_root, _) = parse(&mut dom, &a);
        let (b_root, _) = parse(&mut dom, &b);
        let settings = WmlComparerSettings::default();
        assert!(merge_normal_style_rpr(
            &mut dom, out_root, b_root, &settings
        ));
        let normal = find_normal_style(&dom, out_root).expect("Normal");
        let rpr = dom.element(normal, &W::name("rPr")).expect("live rPr");
        let fonts = dom.element(rpr, &W::name("rFonts")).expect("live rFonts");
        let attr = |n: &str| dom.attribute(fonts, &W::name(n)).map(str::to_string);
        assert_eq!(attr("asciiTheme").as_deref(), Some("minorHAnsi"));
        assert_eq!(attr("hAnsiTheme").as_deref(), Some("minorHAnsi"));
        assert_eq!(attr("eastAsiaTheme").as_deref(), Some("minorHAnsi"));
        assert_eq!(attr("cstheme").as_deref(), Some("minorBidi"));
        // No explicit face left behind to contradict the theme on the page.
        assert_eq!(attr("ascii"), None);
        assert_eq!(attr("hAnsi"), None);
    }

    /// LibreOffice names its Normal `style0` and marks no paragraph style as
    /// the default; Word still finds it by its name (multi_section_nested_
    /// table_rowspan: the Word redline carries B's whole Normal, ours left it
    /// bare because B "had no Normal").
    #[test]
    fn normal_is_found_by_name_when_no_style_is_the_default() {
        let ns = format!("xmlns:w=\"{}\"", W::URI);
        let s = format!(
            "<w:styles {ns}><w:style w:styleId=\"style15\" w:type=\"paragraph\">\
             <w:name w:val=\"Heading\"/></w:style>\
             <w:style w:styleId=\"style0\" w:type=\"paragraph\"><w:name w:val=\"Normal\"/>\
             </w:style></w:styles>"
        );
        let mut dom = Dom::new();
        let (root, _) = parse(&mut dom, &s);
        let normal = find_normal_style(&dom, root).expect("Normal found by name");
        assert_eq!(dom.attribute(normal, &W::name("styleId")), Some("style0"));
    }

    /// A stylesheet with a Normal; `dd` = (rPrDefault, pPrDefault) contents,
    /// `None` for no docDefaults at all.
    fn stylesheet(dd: Option<(&str, &str)>, normal: &str) -> String {
        let ns = format!("xmlns:w=\"{}\"", W::URI);
        let dd = dd
            .map(|(r, p)| {
                format!(
                    "<w:docDefaults><w:rPrDefault><w:rPr>{r}</w:rPr></w:rPrDefault>\
                     <w:pPrDefault><w:pPr>{p}</w:pPr></w:pPrDefault></w:docDefaults>"
                )
            })
            .unwrap_or_default();
        format!(
            "<w:styles {ns}>{dd}<w:style w:type=\"paragraph\" w:default=\"1\" \
             w:styleId=\"Normal\"><w:name w:val=\"Normal\"/>{normal}</w:style></w:styles>"
        )
    }

    fn normal_pair(
        a_dd_ppr: &str,
        a_normal: &str,
        b_dd_ppr: &str,
        b_normal: &str,
    ) -> (String, String) {
        (
            stylesheet(Some(("<w:sz w:val=\"22\"/>", a_dd_ppr)), a_normal),
            stylesheet(
                Some(("<w:kern w:val=\"2\"/><w:sz w:val=\"21\"/>", b_dd_ppr)),
                b_normal,
            ),
        )
    }

    /// Run `merge_normal_style_spacing` then `merge_normal_style_rpr` over
    /// A's and B's stylesheets; returns the dom and the output root.
    fn merge_normals(a: &str, b: &str) -> (Dom, NodeId) {
        let mut dom = Dom::new();
        let (out_root, _) = parse(&mut dom, a);
        let (b_root, _) = parse(&mut dom, b);
        let settings = WmlComparerSettings::default();
        merge_normal_style_spacing(&mut dom, out_root, b_root, &settings);
        merge_normal_style_rpr(&mut dom, out_root, b_root, &settings);
        (dom, out_root)
    }

    /// `(attribute, value)` pairs of the live Normal's `container/local`.
    fn live_normal_attrs(
        dom: &Dom,
        root: NodeId,
        container: &str,
        local: &str,
    ) -> Option<Vec<(String, String)>> {
        let normal = find_normal_style(dom, root)?;
        let c = dom.element(normal, &W::name(container))?;
        let e = dom.element(c, &W::name(local))?;
        let mut v: Vec<(String, String)> = dom
            .attributes(e)
            .into_iter()
            .map(|(n, v)| (n.local_name().to_string(), v))
            .collect();
        v.sort();
        Some(v)
    }

    fn attrs(pairs: &[(&str, &str)]) -> Option<Vec<(String, String)>> {
        let mut v: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        v.sort();
        Some(v)
    }

    /// A LibreOffice B with no docDefaults at all: Word reads it with its own
    /// factory defaults (after 160, line 278, kern 2, standard contextual
    /// ligatures) and writes them into the live Normal
    /// (multi_section_nested_table_rowspan, table_bookmark_end ×
    /// table_vmerge_colspan).
    #[test]
    fn normal_uses_word_factory_defaults_when_b_has_no_doc_defaults() {
        let a = stylesheet(
            Some((
                "<w:sz w:val=\"22\"/>",
                "<w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>",
            )),
            "",
        );
        let b = stylesheet(
            None,
            "<w:pPr><w:widowControl w:val=\"false\"/></w:pPr><w:rPr><w:sz w:val=\"24\"/></w:rPr>",
        );
        let (dom, root) = merge_normals(&a, &b);
        assert_eq!(
            live_normal_attrs(&dom, root, "pPr", "spacing"),
            attrs(&[("after", "160"), ("line", "278"), ("lineRule", "auto")])
        );
        assert_eq!(
            live_normal_attrs(&dom, root, "rPr", "kern"),
            attrs(&[("val", "2")])
        );
        let normal = find_normal_style(&dom, root).unwrap();
        let rpr = dom.element(normal, &W::name("rPr")).unwrap();
        let lig = dom
            .element(rpr, &W14::name("ligatures"))
            .expect("ligatures");
        assert_eq!(
            dom.attribute(lig, &W14::name("val")),
            Some("standardContextual")
        );
    }

    /// B declares no complex-script size while A's docDefaults do: Word
    /// writes the implicit 20 half-points (17 of 17 corpus Word redlines).
    #[test]
    fn normal_writes_the_implicit_complex_script_size() {
        let a = stylesheet(
            Some(("<w:sz w:val=\"21\"/><w:szCs w:val=\"22\"/>", "")),
            "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
        );
        let b = stylesheet(Some(("<w:sz w:val=\"22\"/>", "")), "");
        let (dom, root) = merge_normals(&a, &b);
        assert_eq!(
            live_normal_attrs(&dom, root, "rPr", "sz"),
            attrs(&[("val", "22")])
        );
        assert_eq!(
            live_normal_attrs(&dom, root, "rPr", "szCs"),
            attrs(&[("val", "20")])
        );
    }

    /// Word writes only the language attributes B changes against A's
    /// docDefaults, reading a missing attribute as en-US / en-US / ar-SA
    /// (file_103 × file_104: eastAsia zh-CN alone; mcdoc_meeting_agenda:
    /// B silent, A zh-CN → eastAsia en-US).
    #[test]
    fn normal_writes_only_the_language_attributes_that_change() {
        let lang = |v: &str, ea: &str, bidi: &str| {
            format!("<w:lang w:val=\"{v}\" w:eastAsia=\"{ea}\" w:bidi=\"{bidi}\"/>")
        };
        let a = stylesheet(
            Some((
                &format!("<w:sz w:val=\"22\"/>{}", lang("en-US", "en-US", "ar-SA")),
                "",
            )),
            "",
        );
        let b = stylesheet(
            Some((
                &format!("<w:sz w:val=\"21\"/>{}", lang("en-US", "zh-CN", "ar-SA")),
                "",
            )),
            &format!("<w:rPr>{}</w:rPr>", lang("en-US", "zh-CN", "ar-SA")),
        );
        let (dom, root) = merge_normals(&a, &b);
        assert_eq!(
            live_normal_attrs(&dom, root, "rPr", "lang"),
            attrs(&[("eastAsia", "zh-CN")])
        );

        let a = stylesheet(
            Some((
                &format!("<w:sz w:val=\"22\"/>{}", lang("en-US", "zh-CN", "ar-SA")),
                "",
            )),
            "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
        );
        let b = stylesheet(Some(("<w:sz w:val=\"21\"/>", "")), "");
        let (dom, root) = merge_normals(&a, &b);
        assert_eq!(
            live_normal_attrs(&dom, root, "rPr", "lang"),
            attrs(&[("eastAsia", "en-US")])
        );
    }

    /// file_103 × file_104: A's Normal is bare, B's carries only a pPr. Word
    /// still writes B's run defaults into the live Normal (sz 21, kern 2);
    /// the structure gate is either side's pPr or rPr, not rPr alone.
    #[test]
    fn normal_takes_b_run_defaults_when_only_b_paragraph_props_are_stored() {
        let (a, b) = normal_pair(
            "<w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>",
            "",
            "",
            "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
        );
        let mut dom = Dom::new();
        let (out_root, _) = parse(&mut dom, &a);
        let (b_root, _) = parse(&mut dom, &b);
        let settings = WmlComparerSettings::default();
        assert!(merge_normal_style_rpr(
            &mut dom, out_root, b_root, &settings
        ));
        let normal = find_normal_style(&dom, out_root).expect("Normal");
        let rpr = dom.element(normal, &W::name("rPr")).expect("live rPr");
        let val = |n: &str| {
            dom.element(rpr, &W::name(n))
                .and_then(|e| dom.attribute(e, &W::val()))
                .map(str::to_string)
        };
        assert_eq!(val("sz").as_deref(), Some("21"));
        assert_eq!(val("kern").as_deref(), Some("2"));
    }

    /// Live Normal pPr, children in order, pPrChange left out.
    fn live_normal_ppr(dom: &Dom, root: NodeId) -> Vec<(String, Vec<(String, String)>)> {
        let normal = find_normal_style(dom, root).expect("Normal");
        let Some(ppr) = dom.element(normal, &W::p_pr()) else {
            return Vec::new();
        };
        dom.elements(ppr, None)
            .into_iter()
            .filter(|&c| dom.name(c).is_some_and(|n| n != W::name("pPrChange")))
            .map(|c| {
                let name = dom.name(c).unwrap().local_name().to_string();
                let attrs = dom
                    .attributes(c)
                    .into_iter()
                    .map(|(n, v)| (n.local_name().to_string(), v))
                    .collect();
                (name, attrs)
            })
            .collect()
    }

    /// file_103 × file_104: B keeps indents, justification and line-unit
    /// spacing in its docDefaults, which the redline (A's docDefaults) loses.
    /// Word writes them into the live Normal.
    #[test]
    fn normal_takes_b_doc_default_indent_and_justification() {
        let (a, b) = normal_pair(
            "<w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>",
            "",
            "<w:spacing w:beforeLines=\"50\" w:afterLines=\"50\"/>\
             <w:ind w:leftChars=\"50\" w:left=\"50\" w:firstLine=\"200\"/><w:jc w:val=\"both\"/>",
            "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
        );
        let mut dom = Dom::new();
        let (out_root, _) = parse(&mut dom, &a);
        let (b_root, _) = parse(&mut dom, &b);
        let settings = WmlComparerSettings::default();
        assert!(merge_normal_style_spacing(
            &mut dom, out_root, b_root, &settings
        ));
        let live = live_normal_ppr(&dom, out_root);
        let get = |n: &str| live.iter().find(|(k, _)| k == n).map(|(_, a)| a.clone());
        let attr = |n: &str, a: &str| {
            get(n).and_then(|v| v.into_iter().find(|(k, _)| k == a).map(|(_, v)| v))
        };
        assert_eq!(
            attr("spacing", "beforeLines").as_deref(),
            Some("50"),
            "{live:?}"
        );
        assert_eq!(
            attr("spacing", "afterLines").as_deref(),
            Some("50"),
            "{live:?}"
        );
        assert_eq!(attr("spacing", "after").as_deref(), Some("0"), "{live:?}");
        assert_eq!(attr("ind", "firstLine").as_deref(), Some("200"), "{live:?}");
        assert_eq!(attr("jc", "val").as_deref(), Some("both"), "{live:?}");
        assert!(get("widowControl").is_some(), "{live:?}");
        let order: Vec<&str> = live.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(order, ["widowControl", "spacing", "ind", "jc"]);
    }

    /// file_104 × file_105, the reverse: A's docDefaults indent and justify,
    /// B's do not. Word neutralizes each A value in the live Normal (every
    /// indent attribute 0, jc left, line-unit spacing 0).
    #[test]
    fn normal_neutralizes_a_doc_default_indent_and_justification() {
        let (a, b) = normal_pair(
            "<w:spacing w:beforeLines=\"50\" w:afterLines=\"50\"/>\
             <w:ind w:leftChars=\"50\" w:left=\"50\" w:firstLine=\"200\"/><w:jc w:val=\"both\"/>",
            "<w:pPr><w:widowControl w:val=\"0\"/></w:pPr>",
            "<w:spacing w:after=\"200\" w:line=\"276\" w:lineRule=\"auto\"/>",
            "",
        );
        let mut dom = Dom::new();
        let (out_root, _) = parse(&mut dom, &a);
        let (b_root, _) = parse(&mut dom, &b);
        let settings = WmlComparerSettings::default();
        assert!(merge_normal_style_spacing(
            &mut dom, out_root, b_root, &settings
        ));
        let mut live = live_normal_ppr(&dom, out_root);
        for (_, attrs) in &mut live {
            attrs.sort();
        }
        let s = |pairs: &[(&str, &str)]| -> Vec<(String, String)> {
            let mut v: Vec<(String, String)> = pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            v.sort();
            v
        };
        assert_eq!(
            live,
            vec![
                (
                    "spacing".to_string(),
                    s(&[
                        ("beforeLines", "0"),
                        ("afterLines", "0"),
                        ("after", "200"),
                        ("line", "276"),
                        ("lineRule", "auto")
                    ])
                ),
                (
                    "ind".to_string(),
                    s(&[("leftChars", "0"), ("left", "0"), ("firstLine", "0")])
                ),
                ("jc".to_string(), s(&[("val", "left")])),
            ]
        );
    }

    /// A's docDefaults draw paragraph borders, B's do not: Word writes a
    /// `nil` border on every edge A draws, even when no spacing changes
    /// (sd_1494_table_left_indent × sdpr_titleonly).
    #[test]
    fn normal_clears_a_doc_default_borders() {
        let (a, b) = normal_pair(
            "<w:pBdr><w:top w:val=\"single\" w:sz=\"4\" w:space=\"1\" w:color=\"auto\"/>\
             <w:bottom w:val=\"single\" w:sz=\"4\" w:space=\"1\" w:color=\"auto\"/></w:pBdr>",
            "<w:rPr><w:b/></w:rPr>",
            "",
            "",
        );
        let mut dom = Dom::new();
        let (out_root, _) = parse(&mut dom, &a);
        let (b_root, _) = parse(&mut dom, &b);
        let settings = WmlComparerSettings::default();
        assert!(merge_normal_style_spacing(
            &mut dom, out_root, b_root, &settings
        ));
        let normal = find_normal_style(&dom, out_root).expect("Normal");
        let bdr = dom
            .element(normal, &W::p_pr())
            .and_then(|p| dom.element(p, &W::name("pBdr")))
            .expect("live pBdr");
        let edges: Vec<(String, Option<String>, usize)> = dom
            .elements(bdr, None)
            .into_iter()
            .map(|e| {
                (
                    dom.name(e).unwrap().local_name().to_string(),
                    dom.attribute(e, &W::val()).map(str::to_string),
                    dom.attributes(e).len(),
                )
            })
            .collect();
        assert_eq!(
            edges,
            vec![
                ("top".to_string(), Some("nil".to_string()), 1),
                ("bottom".to_string(), Some("nil".to_string()), 1)
            ]
        );
    }
}
