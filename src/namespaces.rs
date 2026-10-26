// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! WordprocessingML and related namespace / `XName` constants ported from
//! `PtOpenXmlUtil.ts` (the subset the comparer references) — M1.7.
//!
//! Each namespace is exposed as a unit struct with three associated
//! items:
//!
//! - `Ns::ns()` returns the `XNamespace` for the URI;
//! - `Ns::name(local)` returns an `XName` for `local` in that namespace;
//! - `Ns::URI` is the namespace URI string (a `const` for switch arms).
//!
//! The hottest element and attribute names — those hit on most documents
//! during hash-cloning, finalize, and the LCS walks — are predefined
//! on `W` and `PT` as `OnceLock`-cached methods (e.g. `W::p`,
//! `PT::unid`). Anything else is reachable via
//! `Ns::name("local")`, which still benefits from the thread-local
//! intern pool.
//!
//! Used by every code path that touches WML/DML/MC names: the comparer,
//! edit pipeline, accept / reject, inspect, audit, markdown projection,
//! `revision_processor`. CLI: these constants back every comparison the
//! `jubarte` subcommands perform.

use crate::xmllinq::{XName, XNamespace};

macro_rules! ns_struct {
    ($(#[$meta:meta])* $name:ident, $uri:literal) => {
        $(#[$meta])*
        #[doc = ""]
        #[doc = concat!("Marker for the namespace `", $uri, "`. ")]
        #[doc = "All three associated items are generated:"]
        #[doc = " [`Self::ns`] for the [`XNamespace`], [`Self::name`] for an"]
        #[doc = " [`XName`] in it, and [`Self::URI`] as a `&'static str`."]
        pub struct $name;
        impl $name {
            /// The namespace, as an [`XNamespace`] rooted at this URI.
            pub fn ns() -> XNamespace {
                XNamespace::get($uri)
            }
            /// An [`XName`] for `local` in this namespace.
            pub fn name(local: &str) -> XName {
                XName::get(local, $uri)
            }
            /// The namespace URI as a `&'static str`. Use it in `match` arms
            /// and other places that need a string constant.
            pub const URI: &'static str = $uri;
        }
    };
}

ns_struct!(
    /// WordprocessingML (`w:`) — the main namespace for Word body, paragraph,
    /// run, table, and property elements and attributes. The hottest element
    /// and attribute constants are predefined on this struct ([`W::p`],
    /// [`W::r`], [`W::sect_pr`], …) as `OnceLock`-cached methods.
    W,
    "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
);
ns_struct!(
    /// Office relationships namespace (`r:`) — `r:id` references between
    /// parts (`word/_rels/document.xml.rels`).
    R,
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships"
);
ns_struct!(
    /// Office Math (`m:`) — equations embedded in a paragraph.
    M,
    "http://schemas.openxmlformats.org/officeDocument/2006/math"
);
ns_struct!(
    /// Markup compatibility (`mc:`) — `mc:AlternateContent`, `mc:Choice`,
    /// `mc:Fallback`, `mc:Ignorable`, and the prefix-list attributes
    /// (`Ignorable`, `PreserveAttributes`, `PreserveElements`,
    /// `ProcessContent`, `MustUnderstand`, and the unqualified `Requires`
    /// on `mc:Choice`) that the serializer must rewrite when prefixes are
    /// rebound.
    MC,
    "http://schemas.openxmlformats.org/markup-compatibility/2006"
);
ns_struct!(
    /// Microsoft `office:o` (VML / Office compatibility markup).
    O,
    "urn:schemas-microsoft-com:office:office"
);
ns_struct!(
    /// VML (`v:`) — legacy vector markup still used by text boxes, shapes,
    /// and some legacy drawings.
    VML,
    "urn:schemas-microsoft-com:vml"
);
ns_struct!(
    /// DrawingML main (`a:`) — the shared shape / drawing element vocabulary.
    A,
    "http://schemas.openxmlformats.org/drawingml/2006/main"
);
ns_struct!(
    /// DrawingML chart (`c:`) — chart parts referenced from `w:drawing` /
    /// `wp:inline` / `wp:anchor`.
    C,
    "http://schemas.openxmlformats.org/drawingml/2006/chart"
);
ns_struct!(
    /// Microsoft Word 2010 wordml (`w14:`) — paragraph / run extensions
    /// (e.g. `w14:paraId`, `w14:textId`) introduced in Word 2010.
    W14,
    "http://schemas.microsoft.com/office/word/2010/wordml"
);
ns_struct!(
    /// Microsoft Word 2012 wordml (`w15:`) — the Word 2013 extensions
    /// (`w15:commentEx`, `w15:person`, `w15:dataBinding`, …).
    W15,
    "http://schemas.microsoft.com/office/word/2012/wordml"
);
ns_struct!(
    /// DrawingML wordprocessingDrawing (`wp:`) — drawing anchors and inlines
    /// inside a run; the parent namespace for inline and anchor wrapper
    /// elements.
    WP,
    "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"
);
// PowerTools correlation namespace (`PtOpenXml.pt`). NOTE: the actual URI is
// `http://powertools.codeplex.com/2011` (verified in PtOpenXmlUtil.ts:3846) —
// NOT the value the implementation plan claimed. Stamped on every
// comparison unit and read back to keep the LCS / move detection
// deterministic; stripped from the output before the package is written.
ns_struct!(
    /// PowerTools correlation namespace — `pt:Unid`, `pt:SHA1Hash`,
    /// `pt:CorrelatedSHA1Hash`, `pt:StructureSHA1Hash`, `pt:Status`,
    /// `pt:UniqueId`, `pt:RunIds`, and the Word-mode stamps `pt:DefaultLine`,
    /// `pt:HasDefaultTableStyle`. Attributes stamped on every comparison
    /// unit and read back to keep the LCS / move detection deterministic;
    /// stripped from the output before the package is written.
    PT,
    "http://powertools.codeplex.com/2011"
);
// PowerTools "Insert" namespace (`PtOpenXml.ptOpenXml`).
ns_struct!(
    /// PowerTools `ptOpenXml` "Insert" namespace — used by the original
    /// `DocumentBuilder` to carry `dataBinding` / build traces.
    PTInsert,
    "http://powertools.codeplex.com/documentbuilder/2011/insert"
);
ns_struct!(
    /// Microsoft `office:word` (`w10:`) — Word's VML extensions (`w10:wrap`,
    /// `w10:anchorlock`), distinct from `w14` / `w15`.
    W10,
    "urn:schemas-microsoft-com:office:word"
);
ns_struct!(
    /// Microsoft drawing 2010 (`a14:`) — DrawingML shape extensions
    /// (line / fill / effect extras).
    A14,
    "http://schemas.microsoft.com/office/drawing/2010/main"
);
ns_struct!(
    /// Microsoft Word 2010 wordprocessingDrawing (`wp14:`) — drawing
    /// extensions co-namespaced with `wp:` for the 2010 wordprocessing
    /// surface.
    WP14,
    "http://schemas.microsoft.com/office/word/2010/wordprocessingDrawing"
);
ns_struct!(
    /// DrawingML diagram (`dgm:`) — SmartArt diagrams.
    DGM,
    "http://schemas.openxmlformats.org/drawingml/2006/diagram"
);
ns_struct!(
    /// Microsoft `wne:` namespace — Word-specific extension attributes
    /// (notably used by `w:noProof`, `w:dataBinding`, and notification
    /// helpers).
    WNE,
    "http://schemas.microsoft.com/office/word/2006/wordml"
);

// ── hottest WML names (extend as needed) ──────────────────────────────────────
// NAME-01: cache the hottest XNames in process-wide OnceLocks so every compare
// does not re-enter the interning table for the same syntactic names.
macro_rules! cached_xname {
    ($ns_uri:expr, $method:ident, $local:literal) => {
        #[doc = concat!("Cached [`XName`] for the local name `", $local, "`.")]
        #[doc = " Stored in a process-wide `OnceLock` so every compare avoids"]
        #[doc = " re-entering the interning table for the same syntactic name"]
        #[doc = " (NAME-01)."]
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
    // The markers a paragraph may hold and still bring nothing to a join.
    cached_xname!(W::URI, proof_err, "proofErr");
    cached_xname!(W::URI, comment_range_start, "commentRangeStart");
    cached_xname!(W::URI, comment_range_end, "commentRangeEnd");
    cached_xname!(W::URI, comment_reference, "commentReference");
    cached_xname!(W::URI, perm_start, "permStart");
    cached_xname!(W::URI, perm_end, "permEnd");
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
    // The schema-hygiene pass: a table grid's columns and their `w:type`,
    // shading and its `w:fill` (#1719).
    cached_xname!(W::URI, grid_col, "gridCol");
    cached_xname!(W::URI, type_, "type");
    cached_xname!(W::URI, shd, "shd");
    cached_xname!(W::URI, fill, "fill");
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
    cached_xname!(W::URI, custom_xml, "customXml");
    cached_xname!(W::URI, sdt, "sdt");
    cached_xname!(W::URI, sdt_content, "sdtContent");
    // NAME-01d: hottest remaining finalize/lcs locals (sectPr/pPrChange/
    // spacing/… were still interning per call — thread-local pool hash each).
    cached_xname!(W::URI, spacing_el, "spacing");
    cached_xname!(W::URI, p_style, "pStyle");
    cached_xname!(W::URI, r_style, "rStyle");
    cached_xname!(W::URI, jc_el, "jc");
    cached_xname!(W::URI, footnote_ref, "footnoteRef");
    cached_xname!(W::URI, endnote_ref, "endnoteRef");

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
    // The marks Word saves a table with, read per table by the convert fit.
    cached_xname!(W::URI, rsid_r, "rsidR");
    cached_xname!(W::URI, rsid_tr, "rsidTr");
    // A table grid column's width, read per column (`grid_col` is above).
    cached_xname!(W::URI, w, "w");
}

impl W14 {
    cached_xname!(W14::URI, para_id, "paraId");
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
    // Word-mode stamp on the original root: the line pitch the output's
    // default paragraph resolves to (read by the demo-spacing strip).
    cached_xname!(PT::URI, default_line, "DefaultLine");
    // Word-mode stamp on each source root: "1" when that document's styles
    // define a default table style (read by the table-margin synthesis).
    cached_xname!(PT::URI, has_default_table_style, "HasDefaultTableStyle");
}

// ── OMML names the comparer's final-mark pass reads per paragraph ─────────────
impl M {
    cached_xname!(M::URI, r, "r");
    cached_xname!(M::URI, r_pr, "rPr");
    cached_xname!(M::URI, o_math, "oMath");
}
