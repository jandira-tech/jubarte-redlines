// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Schema-order ranks for property-container children.
//!
//! These constants back `super::finalize::wml_order_elements_per_standard`
//! (which uses them to resort `w:pPr`, `w:numPr`, `w:rPr`, `w:tblPr`,
//! border and cell-margin children) and are pinned by the
//! schema-consistency test in `tests/schema_consistency.rs`. The test
//! fails if they drift from the WML XSD particle order; the divergences in
//! `RPR_ORDER` (the `moveFrom`/`moveTo`/`ins`/`del` allowlist) are also
//! enforced there.
//!

/// Schema ranks for `w:pPr` children (PtOpenXmlUtil `Order_pPr`). Used by
/// [`super::finalize::wml_order_elements_per_standard`] to sort paragraph
/// property children into OOXML XSD order.
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

/// Schema ranks for `w:numPr` children (`CT_NumPr` sequence).
///
/// `WmlOrderElementsPerStandard` has no `numPr` container, so the original
/// `numId`/`ilvl` order was copied through and the output failed schema
/// validation. Word always normalises the order in its redline, so the
/// pipeline does too.
pub const NUMPR_ORDER: &[(&str, i32)] = &[
    ("ilvl", 10),
    ("numId", 20),
    ("numberingChange", 30),
    ("ins", 40),
];

/// Schema ranks for `w:tcBorders` children (`CT_TcBorders` sequence).
/// PtOpenXmlUtil ranked `start` before `left`, which reordered a valid
/// source into one the schema refuses (#1712).
pub const TCBORDERS_ORDER: &[(&str, i32)] = &[
    ("top", 10),
    ("left", 20),
    ("start", 30),
    ("bottom", 40),
    ("right", 50),
    ("end", 60),
    ("insideH", 70),
    ("insideV", 80),
    ("tl2br", 90),
    ("tr2bl", 100),
];

/// Schema ranks for `w:tblBorders` children (`CT_TblBorders` sequence).
pub const TBLBORDERS_ORDER: &[(&str, i32)] = &[
    ("top", 10),
    ("left", 20),
    ("start", 30),
    ("bottom", 40),
    ("right", 50),
    ("end", 60),
    ("insideH", 70),
    ("insideV", 80),
];

/// Schema ranks for `w:pBdr` children (`CT_PBdr` sequence).
pub const PBDR_ORDER: &[(&str, i32)] = &[
    ("top", 10),
    ("left", 20),
    ("bottom", 30),
    ("right", 40),
    ("between", 50),
    ("bar", 60),
];

/// Schema ranks for `w:tblCellMar` and `w:tcMar` children (`CT_TblCellMar`
/// and `CT_TcMar` share the sequence). PtOpenXmlUtil sorts neither, so a
/// source's order was copied through (#1712).
pub const CELL_MARGINS_ORDER: &[(&str, i32)] = &[
    ("top", 10),
    ("left", 20),
    ("start", 30),
    ("bottom", 40),
    ("right", 50),
    ("end", 60),
];

/// Schema ranks for `w:rPr` children (PtOpenXmlUtil `Order_rPr`), with
/// the `moveFrom`/`moveTo`/`ins`/`del`/`numberingChange` PowerTools
/// divergences allowlisted in the schema oracle.
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

/// Schema ranks for `w:tblPr` children (PtOpenXmlUtil `Order_tblPr`).
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
