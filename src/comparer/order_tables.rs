// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Hand order tables for property containers (Ring 1½ schema oracle).
//!
//! These ranks must stay in sync with the tables inside
//! [`super::finalize::wml_order_elements_per_standard`]. The schema-consistency
//! test (`tests/schema_consistency.rs`) fails if they drift from the WML XSD
//! particle order (except allowlisted PowerTools divergences).

/// `w:pPr` child ranks (PtOpenXmlUtil Order_pPr).
pub const PPR_ORDER: &[(&str, i32)] = &[
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
];

/// `w:numPr` child ranks (`CT_NumPr` sequence).
///
/// Not a PowerTools table — `WmlOrderElementsPerStandard` has no `numPr`
/// container, so a source writing `numId` before `ilvl` was copied through and
/// the output failed schema validation. Word normalises the order when it
/// writes a comparison (0 occurrences across 504 corpus oracles against 10 of
/// ours), so matching Word means normalising.
pub const NUMPR_ORDER: &[(&str, i32)] = &[
    ("ilvl", 10),
    ("numId", 20),
    ("numberingChange", 30),
    ("ins", 40),
];

/// `w:rPr` child ranks (PtOpenXmlUtil Order_rPr), including moveFrom/moveTo
/// PowerTools divergences allowlisted in the schema oracle.
pub const RPR_ORDER: &[(&str, i32)] = &[
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
];

/// `w:tblPr` child ranks (PtOpenXmlUtil Order_tblPr).
pub const TBLPR_ORDER: &[(&str, i32)] = &[
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
