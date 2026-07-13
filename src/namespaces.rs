//! Namespace + name constants ported from `PtOpenXmlUtil.ts` (the subset the
//! comparer references) — M1.7.
//!
//! Each namespace is exposed as a unit struct with `ns()` (the `XNamespace`) and
//! `name(local)` (an `XName` in that namespace). The hottest element/attribute
//! names are predefined; anything else is reachable via `name("local")`.

use crate::xmllinq::{XName, XNamespace};

macro_rules! ns_struct {
    ($name:ident, $uri:literal) => {
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
impl W {
    pub fn document() -> XName {
        Self::name("document")
    }
    pub fn body() -> XName {
        Self::name("body")
    }
    pub fn p() -> XName {
        Self::name("p")
    }
    pub fn r() -> XName {
        Self::name("r")
    }
    pub fn t() -> XName {
        Self::name("t")
    }
    pub fn p_pr() -> XName {
        Self::name("pPr")
    }
    pub fn r_pr() -> XName {
        Self::name("rPr")
    }
    pub fn footnote() -> XName {
        Self::name("footnote")
    }
    pub fn endnote() -> XName {
        Self::name("endnote")
    }
    pub fn id() -> XName {
        Self::name("id")
    }
    pub fn ins() -> XName {
        Self::name("ins")
    }
    pub fn del() -> XName {
        Self::name("del")
    }
    pub fn author() -> XName {
        Self::name("author")
    }
    pub fn date() -> XName {
        Self::name("date")
    }
    pub fn val() -> XName {
        Self::name("val")
    }
}

// ── PowerTools correlation attribute names ────────────────────────────────────
impl PT {
    pub fn unid() -> XName {
        Self::name("Unid")
    }
    pub fn sha1_hash() -> XName {
        Self::name("SHA1Hash")
    }
    pub fn correlated_sha1_hash() -> XName {
        Self::name("CorrelatedSHA1Hash")
    }
    pub fn structure_sha1_hash() -> XName {
        Self::name("StructureSHA1Hash")
    }
    pub fn status() -> XName {
        Self::name("Status")
    }
}
