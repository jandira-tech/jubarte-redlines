// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Namespace + name constants ported from `PtOpenXmlUtil.ts` (the subset the
//! comparer references) — M1.7.
//!
//! Each namespace is exposed as a unit struct with `ns()` (the `XNamespace`) and
//! `name(local)` (an `XName` in that namespace). The hottest element/attribute
//! names are predefined; anything else is reachable via `name("local")`.

use crate::xmllinq::{XName, XNamespace};

macro_rules! ns_struct {
    ($name:ident, $uri:literal) => {
        /// Open XML namespace marker for `$uri`.
        pub struct $name;
        impl $name {
            /// The namespace.
            pub fn ns() -> XNamespace {
                XNamespace::get($uri)
            }
            /// An `XName` for `local` in this namespace.
            pub fn name(local: &str) -> XName {
                XName::get(local, $uri)
            }
            /// The namespace URI.
            pub const URI: &'static str = $uri;
        }
    };
}

ns_struct!(
    W,
    "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
);
ns_struct!(
    R,
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
);
ns_struct!(
    M,
    "http://schemas.openxmlformats.org/officeDocument/2006/math"
);
ns_struct!(
    MC,
    "http://schemas.openxmlformats.org/markup-compatibility/2006"
);
ns_struct!(O, "urn:schemas-microsoft-com:office:office");
ns_struct!(VML, "urn:schemas-microsoft-com:vml");
ns_struct!(A, "http://schemas.openxmlformats.org/drawingml/2006/main");
ns_struct!(C, "http://schemas.openxmlformats.org/drawingml/2006/chart");
ns_struct!(W14, "http://schemas.microsoft.com/office/word/2010/wordml");
ns_struct!(W15, "http://schemas.microsoft.com/office/word/2012/wordml");
ns_struct!(
    WP,
    "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
);
// PowerTools correlation namespace (`PtOpenXml.pt`). NOTE: the actual URI is
// `http://powertools.codeplex.com/2011` (verified in PtOpenXmlUtil.ts:3846) —
// NOT the value the implementation plan claimed.
ns_struct!(PT, "http://powertools.codeplex.com/2011");
// PowerTools "Insert" namespace (`PtOpenXml.ptOpenXml`).
ns_struct!(
    PTInsert,
    "http://powertools.codeplex.com/documentbuilder/2011/insert"
);
ns_struct!(W10, "urn:schemas-microsoft-com:office:word");
ns_struct!(A14, "http://schemas.microsoft.com/office/drawing/2010/main");
ns_struct!(
    WP14,
    "http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing"
);
ns_struct!(
    DGM,
    "http://schemas.openxmlformats.org/drawingml/2006/diagram"
);
ns_struct!(WNE, "http://schemas.microsoft.com/office/word/2006/wordml");

// ── hottest WML names (extend as needed) ──────────────────────────────────────
// NAME-01: cache the hottest XNames in process-wide OnceLocks so every compare
// does not re-enter the interning table for the same syntactic names.
macro_rules! cached_xname {
    ($ns_uri:expr, $method:ident, $local:literal) => {
        /// Cached `XName` for the local name `$local`.
        pub fn $method() -> XName {
            static N: std::sync::OnceLock<XName> = std::sync::OnceLock::new();
            N.get_or_init(|| XName::get($local, $ns_uri)).clone()
        }
    };
}

impl W {
    cached_xname!(W::URI, document, "document");
    cached_xname!(W::URI, body, "body");
    cached_xname!(W::URI, p, "p");
    cached_xname!(W::URI, r, "r");
    cached_xname!(W::URI, t, "t");
    cached_xname!(W::URI, p_pr, "pPr");
    cached_xname!(W::URI, r_pr, "rPr");
    cached_xname!(W::URI, footnote, "footnote");
    cached_xname!(W::URI, endnote, "endnote");
    cached_xname!(W::URI, id, "id");
    cached_xname!(W::URI, ins, "ins");
    cached_xname!(W::URI, del, "del");
    cached_xname!(W::URI, author, "author");
    cached_xname!(W::URI, date, "date");
    cached_xname!(W::URI, val, "val");
    // NAME-01b: table / field / bookmark locals hit on every hash-clone walk.
    cached_xname!(W::URI, tbl, "tbl");
    cached_xname!(W::URI, tr, "tr");
    cached_xname!(W::URI, tc, "tc");
    cached_xname!(W::URI, tc_pr, "tcPr");
    cached_xname!(W::URI, tbl_pr, "tblPr");
    cached_xname!(W::URI, tr_pr, "trPr");
    cached_xname!(W::URI, grid_span, "gridSpan");
    cached_xname!(W::URI, bookmark_start, "bookmarkStart");
    cached_xname!(W::URI, bookmark_end, "bookmarkEnd");
    cached_xname!(W::URI, del_text, "delText");
    cached_xname!(W::URI, sect_pr, "sectPr");
    cached_xname!(W::URI, drawing, "drawing");
    cached_xname!(W::URI, object, "object");
    cached_xname!(W::URI, pict, "pict");
    cached_xname!(W::URI, txbx_content, "txbxContent");
    cached_xname!(W::URI, move_from, "moveFrom");
    cached_xname!(W::URI, move_to, "moveTo");
    cached_xname!(W::URI, move_from_range_start, "moveFromRangeStart");
    cached_xname!(W::URI, move_to_range_start, "moveToRangeStart");
    // NAME-01c: accept / RP pipeline locals (hit every has_rev document).
    cached_xname!(W::URI, move_from_range_end, "moveFromRangeEnd");
    cached_xname!(W::URI, move_to_range_end, "moveToRangeEnd");
    cached_xname!(W::URI, sdt, "sdt");
    cached_xname!(W::URI, sdt_content, "sdtContent");
    // NAME-01d: hottest remaining finalize/lcs locals (sectPr/pPrChange/
    // spacing/… were still interning per call — thread-local pool hash each).
    cached_xname!(W::URI, spacing_el, "spacing");
    cached_xname!(W::URI, p_style, "pStyle");
    cached_xname!(W::URI, jc_el, "jc");

    cached_xname!(W::URI, sdt_pr, "sdtPr");
    cached_xname!(W::URI, fld_char, "fldChar");
    cached_xname!(W::URI, instr_text, "instrText");
    cached_xname!(W::URI, del_instr_text, "delInstrText");
    cached_xname!(W::URI, num_pr, "numPr");
    cached_xname!(W::URI, cell_del, "cellDel");
    cached_xname!(W::URI, cell_ins, "cellIns");
    cached_xname!(W::URI, cell_merge, "cellMerge");
    cached_xname!(W::URI, hyperlink, "hyperlink");
    cached_xname!(W::URI, smart_tag, "smartTag");
    cached_xname!(W::URI, r_pr_change, "rPrChange");
    cached_xname!(W::URI, p_pr_change, "pPrChange");
    cached_xname!(W::URI, numbering_change, "numberingChange");
    cached_xname!(W::URI, v_merge, "vMerge");
    cached_xname!(W::URI, hdr, "hdr");
    cached_xname!(W::URI, ftr, "ftr");
    cached_xname!(W::URI, fld_simple, "fldSimple");
}

// ── PowerTools correlation attribute names ────────────────────────────────────
impl PT {
    cached_xname!(PT::URI, unid, "Unid");
    cached_xname!(PT::URI, sha1_hash, "SHA1Hash");
    cached_xname!(PT::URI, correlated_sha1_hash, "CorrelatedSHA1Hash");
    cached_xname!(PT::URI, structure_sha1_hash, "StructureSHA1Hash");
    cached_xname!(PT::URI, status, "Status");
    // NAME-01c: accept strip / annotate attrs.
    cached_xname!(PT::URI, unique_id, "UniqueId");
    cached_xname!(PT::URI, run_ids, "RunIds");
}
