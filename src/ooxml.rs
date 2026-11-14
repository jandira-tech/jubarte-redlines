// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Shared OOXML schema element orders, named colour sets and element-name
//! groups needed across multiple engine layers.

use crate::namespaces::{W, WNE};
use crate::xmllinq::XName;

/// `CT_SectPr` child order.
const SECT_ORDER: &[&str] = &[
    "headerReference",
    "footerReference",
    "footnotePr",
    "endnotePr",
    "type",
    "pgSz",
    "pgMar",
    "paperSrc",
    "pgBorders",
    "lnNumType",
    "pgNumType",
    "cols",
    "formProt",
    "vAlign",
    "noEndnote",
    "titlePg",
    "textDirection",
    "bidi",
    "rtlGutter",
    "docGrid",
    "printerSettings",
    "sectPrChange",
];

/// `local`'s position in `SECT_ORDER`; unknown names sort last.
pub(crate) fn sect_rank(local: &str) -> usize {
    SECT_ORDER
        .iter()
        .position(|&n| n == local)
        .unwrap_or(SECT_ORDER.len())
}

/// `ST_HighlightColor`.
pub(crate) const HIGHLIGHTS: &[&str] = &[
    "black",
    "blue",
    "cyan",
    "green",
    "magenta",
    "red",
    "yellow",
    "white",
    "darkBlue",
    "darkCyan",
    "darkGreen",
    "darkMagenta",
    "darkRed",
    "darkYellow",
    "darkGray",
    "lightGray",
    "none",
];

/// The two names a text box's content goes by: `w:txbxContent`, and the
/// `wne:txbxContent` a Strict package's shapes hold (the agent view skips
/// both by local name, so every id walk must too).
pub(crate) fn text_box_names() -> [XName; 2] {
    [W::txbx_content(), WNE::name("txbxContent")]
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn sect_rank_orders_schema_children_and_unknown_last() {
        assert!(sect_rank("headerReference") < sect_rank("pgSz"));
        assert!(sect_rank("pgSz") < sect_rank("pgMar"));
        assert!(sect_rank("docGrid") < sect_rank("sectPrChange"));
        assert_eq!(sect_rank("customUnknown"), SECT_ORDER.len());
    }

    #[test]
    fn text_box_names_cover_transitional_and_strict() {
        let names = text_box_names();
        assert_eq!(names[0], W::txbx_content());
        assert_eq!(names[1], WNE::name("txbxContent"));
        assert!(HIGHLIGHTS.contains(&"yellow"));
        assert!(HIGHLIGHTS.contains(&"none"));
    }
}
