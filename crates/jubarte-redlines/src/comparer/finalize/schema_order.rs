// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Schema order for property elements and whole parts (`WmlOrderElementsPerStandard`).

use super::*;

/// `w:tblPr` child ranks (PtOpenXmlUtil.cs Order_tblPr) — shared between
/// `wml_order_elements_per_standard`, [`synthesize_table_cell_margins`]
/// and [`align_word_table_and_comment_chrome`] so the three can't drift
/// apart. Names absent from this list rank 999, which keeps `w:tblPrChange`
/// after `w:tblLook` (150).
pub(super) const TBLPR_ORDER: [(&str, i32); 17] = [
    ("tblStyle", 10),
    ("tblpPr", 20),
    ("tblOverlap", 30),
    ("bidiVisual", 40),
    ("tblStyleRowBandSize", 50),
    ("tblStyleColBandSize", 60),
    ("tblW", 70),
    ("jc", 80),
    ("tblCellSpacing", 90),
    ("tblInd", 100),
    ("tblBorders", 110),
    ("shd", 120),
    ("tblLayout", 130),
    ("tblCellMar", 140),
    ("tblLook", 150),
    ("tblCaption", 160),
    ("tblDescription", 170),
];

/// Insert `child` among `tblpr`'s children so every later sibling outranks it.
pub(super) fn insert_tblpr_child_in_order(dom: &mut Dom, tblpr: NodeId, child: NodeId, rank: i32) {
    let later = dom.elements(tblpr, None).into_iter().find(|&c| {
        dom.name(c)
            .map(|n| {
                TBLPR_ORDER
                    .iter()
                    .find(|(l, _)| *l == n.local_name())
                    .map_or(999, |(_, r)| *r)
            })
            .unwrap_or(999)
            > rank
    });
    match later {
        Some(l) => dom.add_before_self(l, child),
        None => dom.add(tblpr, child),
    }
}

/// Faithful port of `WordprocessingMLUtil.WmlOrderElementsPerStandard`
/// (PtOpenXmlUtil.cs:1440), called by the C# produce path (WmlComparer.cs
/// :1893) and previously MISSING from the port: property-container children
/// (pPr/rPr/tblPr/tcPr/tcBorders/tblBorders/pBdr) are stable-sorted into
/// standard schema order (unknown names rank 999, keeping relative order —
/// C#'s stable `OrderBy`), and `w:p`/`w:r` move their pPr/rPr first.
/// Non-element child nodes of rebuilt containers are dropped exactly like
/// the C# `.Elements()` rebuild. Validator evidence: 16× "unexpected child
/// element pStyle" (numPr emitted before pStyle) across the benchmark corpus.
/// Order property-container children per the OOXML standard so the
/// produced document passes a strict validator's order check.
///
/// Missing schema order caused 16 `unexpected child element pStyle`
/// errors (numPr emitted before `pStyle`) across the benchmark corpus
/// in the original C# port; the equivalent pre-pass now lives at
/// `WmlOrderElementsPerStandard` (WmlComparer.cs:2434).
pub fn wml_order_elements_per_standard(dom: &mut Dom, root: NodeId) {
    order_property_elements(dom, root, true);
}

pub(super) fn order_property_elements(dom: &mut Dom, root: NodeId, comparison: bool) {
    // Rank tables (PtOpenXmlUtil.cs:1273-1439). The W14 entries are C#'s
    // literal `w14:wShadow` etc. — a PowerTools quirk kept verbatim (real
    // markup uses w14:shadow, so they never match; unknowns rank 999 anyway).
    fn rank(dom: &Dom, container: &str, e: NodeId) -> i32 {
        let Some(n) = dom.name(e) else { return 999 };
        let local = n.local_name();
        let ns = n.namespace_name();
        if ns == W14::URI {
            if container == "rPr" {
                return match local {
                    "wShadow" => 270,
                    "wTextOutline" => 280,
                    "wTextFill" => 290,
                    "wScene3d" => 300,
                    "wProps3d" => 310,
                    _ => 999,
                };
            }
            return 999;
        }
        if ns != W::URI {
            return 999;
        }
        let table: &[(&str, i32)] = match container {
            "pPr" => &[
                ("pStyle", 10),
                ("keepNext", 20),
                ("keepLines", 30),
                ("pageBreakBefore", 40),
                ("framePr", 50),
                ("widowControl", 60),
                ("numPr", 70),
                ("suppressLineNumbers", 80),
                ("pBdr", 90),
                ("shd", 100),
                ("tabs", 120),
                ("suppressAutoHyphens", 130),
                ("kinsoku", 140),
                ("wordWrap", 150),
                ("overflowPunct", 160),
                ("topLinePunct", 170),
                ("autoSpaceDE", 180),
                ("autoSpaceDN", 190),
                ("bidi", 200),
                ("adjustRightInd", 210),
                ("snapToGrid", 220),
                ("spacing", 230),
                ("ind", 240),
                ("contextualSpacing", 250),
                ("mirrorIndents", 260),
                ("suppressOverlap", 270),
                ("jc", 280),
                ("textDirection", 290),
                ("textAlignment", 300),
                ("textboxTightWrap", 310),
                ("outlineLvl", 320),
                ("divId", 330),
                ("cnfStyle", 340),
                ("rPr", 350),
                ("sectPr", 360),
                ("pPrChange", 370),
            ],
            "rPr" => &[
                ("moveFrom", 5),
                ("moveTo", 7),
                ("ins", 10),
                ("del", 20),
                ("rStyle", 30),
                ("rFonts", 40),
                ("b", 50),
                ("bCs", 60),
                ("i", 70),
                ("iCs", 80),
                ("caps", 90),
                ("smallCaps", 100),
                ("strike", 110),
                ("dstrike", 120),
                ("outline", 130),
                ("shadow", 140),
                ("emboss", 150),
                ("imprint", 160),
                ("noProof", 170),
                ("snapToGrid", 180),
                ("vanish", 190),
                ("webHidden", 200),
                ("color", 210),
                ("spacing", 220),
                ("w", 230),
                ("kern", 240),
                ("position", 250),
                ("sz", 260),
                ("szCs", 320),
                ("highlight", 330),
                ("u", 340),
                ("effect", 350),
                ("bdr", 360),
                ("shd", 370),
                ("fitText", 380),
                ("vertAlign", 390),
                ("rtl", 400),
                ("cs", 410),
                ("em", 420),
                ("lang", 430),
                ("eastAsianLayout", 440),
                ("specVanish", 450),
                ("oMath", 460),
                // Not in the PowerTools table: the change record closes the rPr,
                // after the w14 properties that rank 999 (Word's own order).
                ("rPrChange", 1000),
            ],
            "tblPr" => &TBLPR_ORDER,
            "tcPr" => &[
                ("cnfStyle", 10),
                ("tcW", 20),
                ("gridSpan", 30),
                ("hMerge", 40),
                ("vMerge", 50),
                ("tcBorders", 60),
                ("shd", 70),
                ("noWrap", 80),
                ("tcMar", 90),
                ("textDirection", 100),
                ("tcFitText", 110),
                ("vAlign", 120),
                ("hideMark", 130),
                ("headers", 140),
            ],
            "tcBorders" => crate::comparer::order_tables::TCBORDERS_ORDER,
            "tblBorders" => crate::comparer::order_tables::TBLBORDERS_ORDER,
            "pBdr" => crate::comparer::order_tables::PBDR_ORDER,
            // Sources write margins in any order; the schema has one (#1712).
            "tblCellMar" | "tcMar" => crate::comparer::order_tables::CELL_MARGINS_ORDER,
            // Not a PowerTools table — C# has no numPr container, so a source
            // writing numId before ilvl was copied through and rejected by the
            // validator (Sch_UnexpectedElementContentExpectingComplex on
            // w:ilvl). CT_NumPr sequence; kept in sync with
            // order_tables::NUMPR_ORDER by tests/schema_consistency.rs.
            "numPr" => crate::comparer::order_tables::NUMPR_ORDER,
            _ => &[],
        };
        table
            .iter()
            .find(|(n2, _)| *n2 == local)
            .map_or(999, |(_, r)| *r)
    }

    let sortable = [
        "pPr",
        "rPr",
        "tblPr",
        "tcPr",
        "tcBorders",
        "tblBorders",
        "pBdr",
        "numPr",
        "tblCellMar",
        "tcMar",
    ];
    for el in dom.descendants_and_self(root, None) {
        let Some(name) = dom.name(el) else { continue };
        if name.namespace_name() != W::URI {
            continue;
        }
        let local = name.local_name().to_string();
        if sortable.contains(&local.as_str()) {
            for n in dom.nodes(el) {
                if !dom.is_element(n)
                    && (comparison || (dom.is_text(n) && !dom.value(n).trim().is_empty()))
                {
                    dom.remove(n);
                }
            }
            let children = dom.elements(el, None);
            if !comparison {
                // Decide whether repair is needed without canonicalizing a
                // valid choice or an ignorable extension's position. Rank 999
                // is NOT an extension test: tblPrChange, tcPrChange and cell
                // revision markers have that rank too and close the sequence.
                let valid_order = if local == "rPr" {
                    // EG_RPrBase is a choice; only rPrChange must be last.
                    children
                        .iter()
                        .map(|&c| dom.name_is(c, &W::r_pr_change()))
                        .is_sorted()
                } else {
                    children
                        .iter()
                        .filter(|&&c| dom.name(c).is_some_and(|n| n.namespace_name() == W::URI))
                        .map(|&c| rank(dom, &local, c))
                        .is_sorted()
                };
                if valid_order {
                    continue;
                }
            }
            // Once repair is required, use the original ordering for EVERY
            // element, including rank-999 children. Filtering any out here
            // strands them before the sorted children appended below.
            let mut kids: Vec<(i32, usize, NodeId)> = children
                .into_iter()
                .enumerate()
                .map(|(i, c)| (rank(dom, &local, c), i, c))
                .collect();
            if kids.is_sorted_by_key(|(r, i, _)| (*r, *i)) {
                continue;
            }
            kids.sort_by_key(|(r, i, _)| (*r, *i));
            for (_, _, c) in kids {
                dom.remove(c);
                dom.add(el, c);
            }
        } else if local == "p" || local == "r" {
            let props = if local == "p" { W::p_pr() } else { W::r_pr() };
            for n in dom.nodes(el) {
                if !dom.is_element(n)
                    && (comparison || (dom.is_text(n) && !dom.value(n).trim().is_empty()))
                {
                    dom.remove(n);
                }
            }
            let kids = dom.elements(el, None);
            let needs_move = kids
                .iter()
                .position(|&c| dom.name_is(c, &props))
                .is_some_and(|first_props| {
                    kids[..first_props].iter().any(|&c| !dom.name_is(c, &props))
                });
            if needs_move {
                let (front, back): (Vec<NodeId>, Vec<NodeId>) =
                    kids.into_iter().partition(|&c| dom.name_is(c, &props));
                for c in front.into_iter().chain(back) {
                    dom.remove(c);
                    dom.add(el, c);
                }
            }
        }
    }
}

/// Schema order for a whole package part, run by the package validity sweep
/// after every other pass has had its say — so a property appended late (a
/// style merged from B, a `w:jc` added after the body's ordering pass) still
/// lands where the schema wants it.
///
/// Also enforces what `wml_order_elements_per_standard` cannot express as an
/// order: the `w:pPr` inside `w:pPrChange` is CT_PPrBase, which has no `w:rPr`,
/// `w:sectPr` or nested `w:pPrChange` (Sch_InvalidElementContentExpectingComplex
/// in 2 of the 2026-09-26 English redlines Word refused).
/// Schema-order enforcement across all parts the redline references
/// (footnotes, endnotes, comments, custom XML, headers, footers).
///
/// Two 2026-09-26 English redlines Word refused
/// (`Sch_InvalidElementContentExpectingComplex`) failed because a
/// `w:rPr` landed before the parent `w:sectPr` or nested inside
/// `w:pPrChange`. Passes through every part root once.
pub fn enforce_part_schema_order(dom: &mut Dom, root: NodeId) {
    part_schema_order(dom, root, true);
}

/// Repair ordering errors without sorting schema choices or deleting valid
/// comments and whitespace as the comparison's canonicalizer does.
pub(crate) fn repair_part_schema_order(dom: &mut Dom, root: NodeId) {
    part_schema_order(dom, root, false);
}

pub(super) fn part_schema_order(dom: &mut Dom, root: NodeId, comparison: bool) {
    order_property_elements(dom, root, comparison);
    let not_base = [W::r_pr(), W::sect_pr(), W::p_pr_change()];
    for chg in dom.descendants(root, Some(&W::p_pr_change())) {
        for ppr in dom.elements(chg, Some(&W::p_pr())) {
            for c in dom.elements(ppr, None) {
                if dom.name(c).is_some_and(|n| not_base.contains(&n)) {
                    dom.remove(c);
                }
            }
        }
    }
}
