// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

// Untrusted bytes reach this module: an out-of-range index or an integer overflow
// is an abort in the Python and WASM consumers, so both are refused here
// (test fixtures are exempt).
#![cfg_attr(
    not(test),
    deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)
)]

//! ISO/IEC 29500 **Strict** → **Transitional** namespace normalization (M8).
//!
//! Microsoft Word can save a `.docx` in the ISO *Strict* schema, whose XML
//! namespaces live under `http://purl.oclc.org/ooxml/…` rather than the
//! *Transitional* `http://schemas.openxmlformats.org/…` URIs this comparer (and
//! the OpenXML SDK it ports) expects. A Strict `<w:body>` is therefore invisible
//! to `W::body()`, which used to panic with "original has no body".
//!
//! The fix mirrors what the OpenXML SDK does before PowerTools' `WmlComparer`
//! runs: rewrite every Strict URI to its Transitional twin at load time. The
//! mapping is a 1:1 string substitution, transcribed verbatim from the SDK's
//! `OpenXmlNamespaceResolver` (main).
//!
//! Two contexts, disambiguated by part extension:
//!   - **`.xml` parts** (element/attribute namespaces) → `NAMESPACE_TABLE`.
//!   - **`.rels` parts** (relationship `Type=`) → `RELATIONSHIP_TABLE`.
//!
//! Replacements run LONGEST-KEY-FIRST so a key like
//! `…/relationships/customProperties` is rewritten before its prefix
//! `…/relationships/customProperty` (which would otherwise corrupt the plural
//! form). Transitional URIs contain no `purl.oclc.org`, so sequential
//! `str::replace` cannot double-translate.
//!
//! **Zero-churn guarantee**: if no part of the package actually contains the
//! Strict marker (`purl.oclc.org/ooxml`), the input is returned byte-for-byte
//! unchanged — Transitional documents round-trip exactly, preserving golden/parity.

use std::io::{Cursor, Read, Write};
use std::sync::LazyLock;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::admission::InputLimits;

/// Substring present in every ISO Strict URI. Its absence in an entry means the
/// entry is already Transitional (or binary) and needs no rewriting.
const STRICT_MARKER: &str = "purl.oclc.org/ooxml";

/// NAMESPACE table — applied to `.xml` parts (element/attribute namespaces).
///
/// Transcribed from the OpenXML SDK strict→transitional map. All strict keys
/// share the prefix `http://purl.oclc.org/ooxml/`; the three IRREGULAR entries
/// (`custom-properties`, `extended-properties`, and the `descriptions.*` host
/// switch to `descriptions.openxmlformats.org`) are flagged inline.
const NAMESPACE_TABLE: &[(&str, &str)] = &[
    // descriptions.* → descriptions.openxmlformats.org (host switches away from
    // schemas.openxmlformats.org, and the path segment becomes singular
    // "description").
    (
        "http://purl.oclc.org/ooxml/descriptions/base",
        "http://descriptions.openxmlformats.org/description/base",
    ),
    (
        "http://purl.oclc.org/ooxml/descriptions/full",
        "http://descriptions.openxmlformats.org/description/full",
    ),
    // drawingml/* → schemas.openxmlformats.org/drawingml/2006/*
    (
        "http://purl.oclc.org/ooxml/drawingml/chart",
        "http://schemas.openxmlformats.org/drawingml/2006/chart",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/chartDrawing",
        "http://schemas.openxmlformats.org/drawingml/2006/chartDrawing",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/compatibility",
        "http://schemas.openxmlformats.org/drawingml/2006/compatibility",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/diagram",
        "http://schemas.openxmlformats.org/drawingml/2006/diagram",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/lockedCanvas",
        "http://schemas.openxmlformats.org/drawingml/2006/lockedCanvas",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/main",
        "http://schemas.openxmlformats.org/drawingml/2006/main",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/picture",
        "http://schemas.openxmlformats.org/drawingml/2006/picture",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/spreadsheetDrawing",
        "http://schemas.openxmlformats.org/drawingml/2006/spreadsheetDrawing",
    ),
    (
        "http://purl.oclc.org/ooxml/drawingml/wordprocessingDrawing",
        "http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing",
    ),
    // officeDocument/* → schemas.openxmlformats.org/officeDocument/2006/*
    (
        "http://purl.oclc.org/ooxml/officeDocument/bibliography",
        "http://schemas.openxmlformats.org/officeDocument/2006/bibliography",
    ),
    // IRREGULAR: customProperties → custom-properties (hyphenated).
    (
        "http://purl.oclc.org/ooxml/officeDocument/customProperties",
        "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/customXml",
        "http://schemas.openxmlformats.org/officeDocument/2006/customXml",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/customXmlDataProps",
        "http://schemas.openxmlformats.org/officeDocument/2006/customXmlDataProps",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/docPropsVTypes",
        "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes",
    ),
    // IRREGULAR: extendedProperties → extended-properties (hyphenated).
    (
        "http://purl.oclc.org/ooxml/officeDocument/extendedProperties",
        "http://schemas.openxmlformats.org/officeDocument/2006/extended-properties",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/math",
        "http://schemas.openxmlformats.org/officeDocument/2006/math",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships",
    ),
    // ns-context only (ISO bug workaround): a Strict xmlns that points at the
    // relationships/customXml URI maps to the plain customXml namespace.
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/customXml",
        "http://schemas.openxmlformats.org/officeDocument/2006/customXml",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/sharedTypes",
        "http://schemas.openxmlformats.org/officeDocument/2006/sharedTypes",
    ),
    (
        "http://purl.oclc.org/ooxml/presentationml/main",
        "http://schemas.openxmlformats.org/presentationml/2006/main",
    ),
    (
        "http://purl.oclc.org/ooxml/schemaLibrary/main",
        "http://schemas.openxmlformats.org/schemaLibrary/2006/main",
    ),
    (
        "http://purl.oclc.org/ooxml/spreadsheetml/main",
        "http://schemas.openxmlformats.org/spreadsheetml/2006/main",
    ),
    (
        "http://purl.oclc.org/ooxml/wordprocessingml/main",
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    ),
];

/// RELATIONSHIP table — applied to `.rels` parts (relationship `Type=` values).
///
/// All regular entries share the pattern
/// `…/officeDocument/relationships/<x>` →
/// `…/officeDocument/2006/relationships/<x>`. The three IRREGULAR entries are
/// flagged inline (two hyphenate the suffix; one reparents `metadata/thumbnail`
/// under `/package/` instead of `/officeDocument/`).
const RELATIONSHIP_TABLE: &[(&str, &str)] = &[
    // Regular relationship Types (alphabetical by suffix <x>).
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/aFChunk",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/aFChunk",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/attachedTemplate",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/attachedTemplate",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/audio",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/audio",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/calcChain",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/calcChain",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/chart",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chart",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/chartsheet",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chartsheet",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/chartUserShapes",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/chartUserShapes",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/commentAuthors",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/commentAuthors",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/comments",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/connections",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/connections",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/control",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/control",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/customProperty",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customProperty",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/customXml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXml",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/customXmlProps",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/customXmlProps",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/diagramColors",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramColors",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/diagramData",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramData",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/diagramLayout",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramLayout",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/diagramQuickStyle",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/diagramQuickStyle",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/dialogsheet",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/dialogsheet",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/drawing",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/drawing",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/endnotes",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/endnotes",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/externalLink",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLink",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/externalLinkPath",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/externalLinkPath",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/font",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/font",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/fontTable",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/fontTable",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/footer",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/footnotes",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footnotes",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/frame",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/frame",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/glossaryDocument",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/glossaryDocument",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/handoutMaster",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/handoutMaster",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/header",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/htmlPubSaveAs",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/htmlPubSaveAs",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/hyperlink",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/image",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/mailMergeHeaderSource",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/mailMergeHeaderSource",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/mailMergeRecipientData",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/mailMergeRecipientData",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/mailMergeSource",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/mailMergeSource",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/movie",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/movie",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/notesMaster",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesMaster",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/notesSlide",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/numbering",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/oleObject",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/oleObject",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/package",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/package",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/pivotCacheDefinition",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheDefinition",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/pivotCacheRecords",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheRecords",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/pivotTable",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotTable",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/presProps",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/presProps",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/printerSettings",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/printerSettings",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/queryTable",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/queryTable",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/revisionHeaders",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/revisionHeaders",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/revisionLog",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/revisionLog",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/settings",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/settings",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/sharedStrings",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/sheetMetadata",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/sheetMetadata",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/slide",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/slideLayout",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/slideMaster",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/slideUpdateInfo",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideUpdateInfo",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/slideUpdateUrl",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideUpdateUrl",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/styles",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/subDocument",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/subDocument",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/table",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/table",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/tableSingleCells",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/tableSingleCells",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/tableStyles",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/tableStyles",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/tags",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/tags",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/theme",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/themeOverride",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/themeOverride",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/transform",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/transform",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/usernames",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/usernames",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/video",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/video",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/viewProps",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/viewProps",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/volatileDependencies",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/volatileDependencies",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/webSettings",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/worksheet",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/xmlMaps",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/xmlMaps",
    ),
    // IRREGULAR rels — hyphenate the suffix.
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/customProperties",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/custom-properties",
    ),
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/extendedProperties",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties",
    ),
    // IRREGULAR rel — metadata/thumbnail reparents under /package/, not /officeDocument/.
    (
        "http://purl.oclc.org/ooxml/officeDocument/relationships/metadata/thumbnail",
        "http://schemas.openxmlformats.org/package/2006/relationships/metadata/thumbnail",
    ),
];

/// `NAMESPACE_TABLE` sorted by descending strict-key length, built once.
static NAMESPACE_SORTED: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut v = NAMESPACE_TABLE.to_vec();
    v.sort_by_key(|e| std::cmp::Reverse(e.0.len()));
    v
});

/// `RELATIONSHIP_TABLE` sorted by descending strict-key length, built once.
static RELATIONSHIP_SORTED: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut v = RELATIONSHIP_TABLE.to_vec();
    v.sort_by_key(|e| std::cmp::Reverse(e.0.len()));
    v
});

/// Where in a part a Strict URI may be rewritten. `.xml` parts carry them
/// in namespace declarations; `.rels` parts carry them in relationship
/// `Type` attribute values. Anywhere else the bytes are the document's
/// own — `w:t`/`w:instrText`/`w:delText` prose, hyperlink `Target`s,
/// comments — and a document that *quotes* a Strict URI keeps it
/// verbatim (#531).
#[derive(Clone, Copy)]
enum UriSite {
    /// `xmlns="…"` / `xmlns:p="…"` attribute values of an `.xml` part, and
    /// every other attribute whose whole value is a Strict namespace URI
    /// (`a:graphicData/@uri`).
    NamespaceDeclaration,
    /// `Type="…"` attribute values of a `.rels` part.
    RelationshipType,
}

impl UriSite {
    /// Whether an attribute named `attr` carrying `value` is a URI of this
    /// site. Beside the declarations, any attribute whose value is wholly
    /// a Strict namespace URI carries one: `a:graphicData/@uri` names the
    /// grammar of the graphic it wraps.
    fn carries(self, attr: &str, value: &str, table: &[(&str, &str)]) -> bool {
        match self {
            UriSite::NamespaceDeclaration => {
                attr == "xmlns"
                    || attr.starts_with("xmlns:")
                    || table.iter().any(|&(from, _)| value == from)
            }
            UriSite::RelationshipType => attr == "Type",
        }
    }
}

/// Apply a (strict → transitional) table to the URI-bearing attribute
/// values of `text` — namespace declarations, a `.rels` part's `Type`, and
/// attributes whose whole value is a Strict namespace URI — leaving every
/// other byte: element and attribute names, text content, comments, as
/// the file spells them. `text` is assumed to already contain
/// `STRICT_MARKER` (callers gate on that for the fast path).
fn translate(text: &str, table: &[(&str, &str)], site: UriSite) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(lt) = rest.find('<') {
        let (before, from_lt) = rest.split_at(lt);
        out.push_str(before);
        if let Some((verbatim, after)) = split_opaque_markup(from_lt) {
            // Comments, CDATA, the XML declaration: no attributes inside.
            out.push_str(verbatim);
            rest = after;
        } else if let Some((tag, after)) = split_tag(from_lt) {
            rewrite_tag(tag, table, site, &mut out);
            rest = after;
        } else {
            // Unterminated tag (malformed part): keep the rest verbatim.
            out.push_str(from_lt);
            return out;
        }
    }
    out.push_str(rest);
    out
}

/// Comments (`<!--…-->`), CDATA (`<![CDATA[…]]>`), processing instructions
/// (`<?…?>`) and the DOCTYPE (`<!…>`): returned with the text after them.
fn split_opaque_markup(s: &str) -> Option<(&str, &str)> {
    for (open, close) in [("<!--", "-->"), ("<![CDATA[", "]]>"), ("<?", "?>")] {
        if let Some(body) = s.strip_prefix(open) {
            let end = body
                .find(close)
                .map(|i| open.len().saturating_add(i).saturating_add(close.len()))?;
            return Some(s.split_at(end));
        }
    }
    if s.starts_with("<!") {
        s.find('>').map(|i| s.split_at(i.saturating_add(1)))
    } else {
        None
    }
}

/// A start or end tag: from its `<` to the first `>` outside quoted
/// attribute values (a `>` inside a value is legal XML).
fn split_tag(s: &str) -> Option<(&str, &str)> {
    let mut quote: Option<char> = None;
    for (i, c) in s.char_indices().skip(1) {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None => match c {
                '"' | '\'' => quote = Some(c),
                '>' => return Some(s.split_at(i.saturating_add(1))),
                _ => {}
            },
        }
    }
    None
}

/// Copy `tag` to `out`, rewriting only the values of the attributes that
/// carry URIs of `site`.
fn rewrite_tag(tag: &str, table: &[(&str, &str)], site: UriSite, out: &mut String) {
    let mut rest = tag;
    while let Some(open) = rest.find(['"', '\'']) {
        let (before, from_open) = rest.split_at(open);
        let quote = from_open.chars().next().expect("find hit a quote");
        let Some(after_open) = from_open.strip_prefix(quote) else {
            break;
        };
        let Some(rel) = after_open.find(quote) else {
            // Unterminated value: keep the rest verbatim.
            out.push_str(rest);
            return;
        };
        let (value, from_close) = after_open.split_at(rel);
        out.push_str(before);
        out.push(quote);
        if site.carries(attr_name(before), value, table) {
            out.push_str(&replace_all(value, table));
        } else {
            out.push_str(value);
        }
        out.push(quote);
        rest = from_close.strip_prefix(quote).unwrap_or(from_close);
    }
    out.push_str(rest);
}

/// The attribute name ending `before` — the token after the last
/// whitespace, `=` stripped: `… <w:p xmlns:w=` → `xmlns:w`.
fn attr_name(before: &str) -> &str {
    let name = before.trim_end();
    let name = name.strip_suffix('=').unwrap_or(name).trim_end();
    match name.rsplit_once(char::is_whitespace) {
        Some((_, n)) => n,
        None => name.strip_prefix('<').unwrap_or(name),
    }
}

/// The old whole-string substitution, now scoped to one attribute value.
fn replace_all(text: &str, table: &[(&str, &str)]) -> String {
    let mut out = text.to_string();
    for &(from, to) in table {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    out
}

/// The Strict wordprocessingml main namespace — the URI that makes a main
/// part Strict rather than Transitional.
const STRICT_MAIN: &str = "http://purl.oclc.org/ooxml/wordprocessingml/main";

/// Whether the package's main document part (`word/document.xml`) is ISO
/// Strict — decided by the namespace its root element declares, the same
/// fact Word reads the dialect from. A Transitional package that merely
/// *quotes* a Strict URI in its text is not a Strict package (#531).
pub fn main_part_is_strict(bytes: &[u8]) -> bool {
    let Ok(mut archive) = crate::opc::open_archive(Cursor::new(bytes)) else {
        return false;
    };
    let Ok(mut part) = archive.by_name("word/document.xml") else {
        return false;
    };
    // The root element's opening tag follows whatever preamble the part
    // carries — a comment, a DOCTYPE — however long: read until it
    // appears, bounded by the part's own size, not by a window a long
    // preamble can push the root past (#561). Only the tail since the
    // last `<` can still hold the tag, so the head stays small.
    let mut head: Vec<u8> = Vec::new();
    let mut chunk = vec![0u8; 16 * 1024];
    loop {
        let Ok(read) = part.read(&mut chunk) else {
            return false;
        };
        if read == 0 {
            break;
        }
        head.extend(chunk.iter().copied().take(read));
        let window = String::from_utf8_lossy(&head);
        if let Some((start, _)) = root_open(&window)
            && window.split_at(start).1.contains('>')
        {
            break;
        }
        match head.iter().rposition(|&b| b == b'<') {
            Some(lt) => {
                head.drain(..lt);
            }
            None => head.clear(),
        }
    }
    let head = String::from_utf8_lossy(&head);
    let Some((start, _)) = root_open(&head) else {
        return false;
    };
    let decl = head.split_at(start).1;
    let decl = match decl.find('>') {
        Some(end) => decl.split_at(end).0,
        // Truncated by the end of the part: the declarations it holds
        // still decide.
        None => decl,
    };
    decl.contains(STRICT_MAIN)
}

/// Where `head` opens its root element: the first `<` an element name
/// follows (a comment, PI or DOCTYPE `<!`/`<?` opens none).
fn root_open(head: &str) -> Option<(usize, char)> {
    head.char_indices().find(|&(i, c)| {
        c == '<'
            && head
                .get(i.saturating_add(1)..)
                .is_some_and(|rest| rest.starts_with(char::is_alphabetic))
    })
}

/// Normalize an ISO/IEC 29500 **Strict** `.docx` to **Transitional**.
///
/// Walks every zip entry; rewrites Strict URIs in `.rels` parts using
/// `RELATIONSHIP_TABLE` and in `.xml` parts (including `[Content_Types].xml`)
/// using `NAMESPACE_TABLE`; binary parts pass through untouched.
///
/// **Zero-churn**: if no entry contains `STRICT_MARKER` (i.e. the package is
/// already Transitional), the original `bytes` are returned unchanged — the zip
/// is never rebuilt. Only when at least one URI was rewritten is a new zip
/// assembled (every entry under its original name, Deflated).
///
/// Entries inflate under [`InputLimits::compare`]; see
/// [`strict_to_transitional_docx_within`] for the budget semantics.
///
/// # Errors
///
/// A [`crate::opc::Zip32Limit`] (`OUTPUT_LIMIT`) when the rewritten package
/// could not stay inside ZIP32 — unreachable under the compare budget, which
/// caps the output far below the boundary, but refused rather than written.
pub fn strict_to_transitional_docx(bytes: &[u8]) -> Result<Vec<u8>, crate::opc::Zip32Limit> {
    strict_to_transitional_docx_within(bytes, InputLimits::compare())
}

/// [`strict_to_transitional_docx`] with an explicit inflation budget.
///
/// Every entry is inflated through a reader capped at the smaller of
/// `limits.max_part_bytes` and the package budget left
/// (`limits.max_uncompressed_bytes`), because the sizes the central directory
/// declares are not trusted and are never used to size an allocation. An
/// entry that inflates past its cap ends the scan and `bytes` come back
/// unchanged, the same answer an unreadable archive gets; a caller that wants
/// a typed refusal admits the package first with [`crate::admission::admit`].
///
/// # Errors
///
/// A [`crate::opc::Zip32Limit`] (`OUTPUT_LIMIT`) when the entries about to be
/// written cannot fit a ZIP32 package (ZIP-10/#604): jubarte writes no ZIP64,
/// so the plan refuses before `ZipWriter::new` instead of handing back the
/// untranslated Strict bytes, which a caller could mistake for output.
pub fn strict_to_transitional_docx_within(
    bytes: &[u8],
    limits: InputLimits,
) -> Result<Vec<u8>, crate::opc::Zip32Limit> {
    // Not a readable zip — leave untouched (PartFs::open reports the real
    // error downstream).
    let Ok(mut archive) = crate::opc::open_archive(Cursor::new(bytes)) else {
        return Ok(bytes.to_vec());
    };

    let n = archive.len();
    // Drain the archive into (name, bytes) so we can rebuild afterward. Track
    // whether ANY entry actually changed to preserve the zero-churn fast path.
    let mut entries: Vec<(String, Vec<u8>)> = Vec::with_capacity(n);
    let mut any_changed = false;
    let mut total: u64 = 0;
    for i in 0..n {
        let Ok(mut f) = archive.by_index(i) else {
            return Ok(bytes.to_vec());
        };
        if f.is_dir() {
            continue;
        }
        let Ok(name) = f.name() else {
            return Ok(bytes.to_vec());
        };
        let name = name.into_owned();
        let remaining = limits.max_uncompressed_bytes.saturating_sub(total);
        let cap = limits.max_part_bytes.min(remaining);
        let mut buf = Vec::new();
        if (&mut f)
            .take(cap.saturating_add(1))
            .read_to_end(&mut buf)
            .is_err()
            || buf.len() as u64 > cap
        {
            return Ok(bytes.to_vec());
        }
        total = total.saturating_add(buf.len() as u64);
        entries.push((name, buf));
    }

    // The XML parts are those named `.xml` in any case, and those the
    // content types call XML: a main part may be `word/main.dat`.
    let typed = crate::opc::typed_as_xml(
        entries
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("[Content_Types].xml"))
            .map(|(_, data)| data.as_slice()),
    );
    for (name, buf) in &mut entries {
        let rels = crate::opc::ends_with_ignore_case(name, ".rels");
        if !rels && !crate::opc::ends_with_ignore_case(name, ".xml") && !typed(name) {
            continue;
        }
        // Fast path: only text parts carrying the Strict marker can change.
        let text = String::from_utf8_lossy(buf);
        if text.contains(STRICT_MARKER) {
            let (table, site) = if rels {
                (&*RELATIONSHIP_SORTED, UriSite::RelationshipType)
            } else {
                (&*NAMESPACE_SORTED, UriSite::NamespaceDeclaration)
            };
            let translated = translate(&text, table, site);
            if translated != text.as_ref() {
                any_changed = true;
                *buf = translated.into_bytes();
            }
        }
    }

    if !any_changed {
        // Already Transitional: byte-identical round-trip.
        return Ok(bytes.to_vec());
    }

    // jubarte writes ZIP32 only (ZIP-10/#604): a rebuild that would need
    // ZIP64 is a typed refusal, not the untranslated input back.
    let plan: Vec<(&str, u64)> = entries
        .iter()
        .map(|(name, data)| (name.as_str(), data.len() as u64))
        .collect();
    crate::opc::zip32_plan(&plan)?;

    // Rebuild the package with translated parts, dated as `PartFs::to_zip`
    // dates its entries (no wall-clock time).
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(zip::DateTime::default());
    for (name, data) in &entries {
        if writer.start_file(name, options).is_err() {
            return Ok(bytes.to_vec());
        }
        if writer.write_all(data).is_err() {
            return Ok(bytes.to_vec());
        }
    }
    match writer.finish() {
        Ok(c) => Ok(c.into_inner()),
        Err(_) => Ok(bytes.to_vec()),
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;
    use zip::ZipArchive;

    /// A Strict document whose *text* quotes a Strict URI: the namespace
    /// declaration is rewritten, the `w:t` keeps the bytes the file's own
    /// text carries (#531 review defect 2 — a whole-part `str::replace`
    /// rewrote the document's prose as well).
    #[test]
    fn text_quoting_a_strict_uri_keeps_it() {
        let strict = concat!(
            r#"<?xml version="1.0"?>"#,
            r#"<w:document "#,
            r#"xmlns:w="http://purl.oclc.org/ooxml/wordprocessingml/main">"#,
            r#"<w:body><w:p><w:r>"#,
            r#"<w:t>See http://purl.oclc.org/ooxml/wordprocessingml/main for the strict namespace.</w:t>"#,
            r#"</w:r></w:p></w:body></w:document>"#,
        );
        let out = translate(strict, &NAMESPACE_SORTED, UriSite::NamespaceDeclaration);
        assert!(
            out.contains(
                r#"xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main""#
            ),
            "the namespace declaration is translated: {out}"
        );
        assert!(
            out.contains(
                "See http://purl.oclc.org/ooxml/wordprocessingml/main for the strict namespace."
            ),
            "the document's own text stays verbatim: {out}"
        );
    }

    /// A Strict `.rels` whose hyperlink `Target` is itself a Strict URI: the
    /// relationship `Type` is translated, the target the author linked to
    /// stays verbatim.
    #[test]
    fn relationship_targets_keep_their_strict_uri() {
        let strict_rels = concat!(
            r#"<?xml version="1.0"?>"#,
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
            r#"<Relationship Id="rId1" "#,
            r#"Type="http://purl.oclc.org/ooxml/officeDocument/relationships/hyperlink" "#,
            r#"Target="http://purl.oclc.org/ooxml/wordprocessingml/main" "#,
            r#"TargetMode="External"/>"#,
            r#"</Relationships>"#,
        );
        let out = translate(strict_rels, &RELATIONSHIP_SORTED, UriSite::RelationshipType);
        assert!(
            out.contains(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink"
            ),
            "the relationship type is translated: {out}"
        );
        assert!(
            out.contains(r#"Target="http://purl.oclc.org/ooxml/wordprocessingml/main""#),
            "the hyperlink target stays verbatim: {out}"
        );
    }

    /// A Transitional package whose text merely mentions a Strict URI is
    /// handed back byte-for-byte: the marker in prose no longer counts as
    /// a Strict package (#531 review defect 2).
    #[test]
    fn transitional_package_quoting_a_strict_uri_stays_untouched() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer.start_file("word/document.xml", options).unwrap();
        writer
            .write_all(
                concat!(
                    "<?xml version=\"1.0\"?>\n",
                    "<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">",
                    "<w:body><w:p><w:r><w:t>See http://purl.oclc.org/ooxml/wordprocessingml/main</w:t></w:r></w:p></w:body>",
                    "</w:document>",
                )
                .as_bytes(),
            )
            .unwrap();
        let pkg = writer.finish().unwrap().into_inner();

        assert_eq!(
            strict_to_transitional_docx(&pkg).unwrap(),
            pkg,
            "a Transitional package quoting a Strict URI must round-trip byte-identical"
        );
    }

    /// A Strict package whose `w:instrText` quotes a Strict URI keeps the
    /// instruction's bytes while its namespaces are translated end to end.
    #[test]
    fn strict_package_keeps_field_text_quoting_a_strict_uri() {
        let strict_doc = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<w:document xmlns:w=\"http://purl.oclc.org/ooxml/wordprocessingml/main\">",
            "<w:body><w:p><w:r><w:instrText> REF http://purl.oclc.org/ooxml/wordprocessingml/main \\h </w:instrText></w:r></w:p></w:body>",
            "</w:document>",
        );
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer.start_file("word/document.xml", options).unwrap();
        writer.write_all(strict_doc.as_bytes()).unwrap();
        let strict_pkg = writer.finish().unwrap().into_inner();

        let out = strict_to_transitional_docx(&strict_pkg).unwrap();
        let mut reread = ZipArchive::new(Cursor::new(&out)).unwrap();
        let doc = {
            let mut f = reread.by_name("word/document.xml").unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            s
        };
        assert!(
            doc.contains(" REF http://purl.oclc.org/ooxml/wordprocessingml/main \\h "),
            "the field instruction stays verbatim: {doc}"
        );
        assert!(
            doc.contains("http://schemas.openxmlformats.org/wordprocessingml/2006/main"),
            "the namespaces are translated: {doc}"
        );
    }

    /// A Strict `document.xml` snippet: the `w`, `r`, and `m` namespaces flip
    /// to their Transitional URIs, and no `purl.oclc.org` survives.
    #[test]
    fn translates_strict_document_xml_namespaces() {
        let strict = concat!(
            r#"<?xml version="1.0"?>"#,
            r#"<w:document "#,
            r#"xmlns:w="http://purl.oclc.org/ooxml/wordprocessingml/main" "#,
            r#"xmlns:r="http://purl.oclc.org/ooxml/officeDocument/relationships" "#,
            r#"xmlns:m="http://purl.oclc.org/ooxml/officeDocument/math">"#,
            r#"<w:body/></w:document>"#,
        );
        let out = translate(strict, &NAMESPACE_SORTED, UriSite::NamespaceDeclaration);
        assert!(
            out.contains("http://schemas.openxmlformats.org/wordprocessingml/2006/main"),
            "w namespace not transitional: {out}"
        );
        assert!(
            out.contains("http://schemas.openxmlformats.org/officeDocument/2006/relationships"),
            "r namespace not transitional: {out}"
        );
        assert!(
            out.contains("http://schemas.openxmlformats.org/officeDocument/2006/math"),
            "m namespace not transitional: {out}"
        );
        assert!(!out.contains(STRICT_MARKER), "strict marker leaked: {out}");
    }

    /// A Strict `a:graphicData/@uri` — an attribute whose whole value is a
    /// namespace URI, not a declaration — is translated with the
    /// namespaces, while `mc:Ignorable`, which names prefixes rather than
    /// URIs, stays as the file spells it (#561 review defect 1).
    #[test]
    fn graphic_data_uri_attributes_are_translated() {
        let strict = concat!(
            r#"<?xml version="1.0"?>"#,
            r#"<w:document "#,
            r#"xmlns:w="http://purl.oclc.org/ooxml/wordprocessingml/main" "#,
            r#"xmlns:a="http://purl.oclc.org/ooxml/drawingml/main" "#,
            r#"xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" "#,
            r#"mc:Ignorable="w14">"#,
            r#"<w:r><w:drawing><a:graphic>"#,
            r#"<a:graphicData uri="http://purl.oclc.org/ooxml/drawingml/chart"/>"#,
            r#"<a:graphicData uri="http://purl.oclc.org/ooxml/drawingml/picture"/>"#,
            r#"<a:graphicData uri="http://purl.oclc.org/ooxml/drawingml/diagram"/>"#,
            r#"</a:graphic></w:drawing></w:r>"#,
            r#"</w:document>"#,
        );
        let out = translate(strict, &NAMESPACE_SORTED, UriSite::NamespaceDeclaration);
        assert!(
            out.contains(r#"uri="http://schemas.openxmlformats.org/drawingml/2006/chart""#),
            "the chart uri is translated: {out}"
        );
        assert!(
            out.contains(r#"uri="http://schemas.openxmlformats.org/drawingml/2006/picture""#),
            "the picture uri is translated: {out}"
        );
        assert!(
            out.contains(r#"uri="http://schemas.openxmlformats.org/drawingml/2006/diagram""#),
            "the diagram uri is translated: {out}"
        );
        assert!(
            out.contains(r#"mc:Ignorable="w14""#),
            "Ignorable names prefixes, not URIs: {out}"
        );
        assert!(!out.contains(STRICT_MARKER), "strict marker leaked: {out}");
    }

    /// A Strict `.rels` snippet: the IRREGULAR `extendedProperties` rel becomes
    /// `extended-properties` (longest-key-first beats the `customProperty`/
    /// `extendedProperty` prefixes), and a regular `styles` rel becomes the
    /// Transitional styles rel.
    #[test]
    fn translates_strict_rels_including_irregular() {
        let strict_rels = concat!(
            r#"<?xml version="1.0"?>"#,
            r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
            r#"<Relationship Id="rId1" "#,
            r#"Type="http://purl.oclc.org/ooxml/officeDocument/relationships/extendedProperties" "#,
            r#"Target="docProps/app.xml"/>"#,
            r#"<Relationship Id="rId2" "#,
            r#"Type="http://purl.oclc.org/ooxml/officeDocument/relationships/styles" "#,
            r#"Target="word/styles.xml"/>"#,
            r#"</Relationships>"#,
        );
        let out = translate(strict_rels, &RELATIONSHIP_SORTED, UriSite::RelationshipType);
        assert!(
            out.contains("http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties"),
            "extendedProperties not hyphenated: {out}"
        );
        assert!(
            out.contains(
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles"
            ),
            "regular styles rel not transitional: {out}"
        );
        assert!(!out.contains(STRICT_MARKER), "strict marker leaked: {out}");
    }

    /// A Strict package whose media part inflates past the part budget is
    /// handed back untouched instead of being inflated without bound; the
    /// same package passes under the comparer's default budget.
    #[test]
    fn oversized_part_is_left_untouched_under_a_small_budget() {
        let strict_doc = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<w:document xmlns:w=\"http://purl.oclc.org/ooxml/wordprocessingml/main\">",
            "<w:body/></w:document>",
        );
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer.start_file("word/document.xml", options).unwrap();
        writer.write_all(strict_doc.as_bytes()).unwrap();
        writer.start_file("word/media/blob.bin", options).unwrap();
        writer.write_all(&vec![0u8; 3 * 1024 * 1024]).unwrap();
        let strict_pkg = writer.finish().unwrap().into_inner();

        let small = crate::admission::InputLimits {
            max_part_bytes: 1024 * 1024,
            ..crate::admission::InputLimits::compare()
        };
        assert_eq!(
            strict_to_transitional_docx_within(&strict_pkg, small).unwrap(),
            strict_pkg,
            "a part past the budget stops the rewrite and returns the input"
        );
        let total_only = crate::admission::InputLimits {
            max_uncompressed_bytes: 2 * 1024 * 1024,
            ..crate::admission::InputLimits::compare()
        };
        assert_eq!(
            strict_to_transitional_docx_within(&strict_pkg, total_only).unwrap(),
            strict_pkg,
            "the package budget is honoured as well"
        );
        let out = strict_to_transitional_docx(&strict_pkg).unwrap();
        assert_ne!(
            out, strict_pkg,
            "within the default budget the Strict package is rewritten"
        );
    }

    /// A preamble longer than the old 64 KiB window no longer hides the
    /// Strict root: the dialect is read from the root's own opening tag,
    /// wherever in the part it lies (#561 review defect 2).
    #[test]
    fn a_long_preamble_does_not_hide_a_strict_root() {
        let comment = format!("<!--{}-->", "x".repeat(100 * 1024));
        let package = |uri: &str| {
            let doc = format!(
                "<?xml version=\"1.0\"?>\n{comment}<w:document xmlns:w=\"{uri}\"><w:body/></w:document>",
            );
            let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
            let options =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            writer.start_file("word/document.xml", options).unwrap();
            writer.write_all(doc.as_bytes()).unwrap();
            writer.finish().unwrap().into_inner()
        };
        assert!(main_part_is_strict(&package(
            "http://purl.oclc.org/ooxml/wordprocessingml/main"
        )));
        assert!(!main_part_is_strict(&package(
            "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
        )));
    }

    /// A Transitional package is returned byte-for-byte unchanged (zero-churn).
    #[test]
    fn transitional_package_returned_byte_identical() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer.start_file("[Content_Types].xml", options).unwrap();
        writer
            .write_all(
                b"<?xml version=\"1.0\"?>\n\
<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"/>",
            )
            .unwrap();
        writer.start_file("word/document.xml", options).unwrap();
        writer
            .write_all(
                b"<?xml version=\"1.0\"?>\n\
<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"/>",
            )
            .unwrap();
        let pkg = writer.finish().unwrap().into_inner();

        let out = strict_to_transitional_docx(&pkg).unwrap();
        assert_eq!(
            out, pkg,
            "transitional package must round-trip byte-identical"
        );
    }

    /// A Strict package is actually rewritten (the zero-churn fast path does NOT
    /// fire): every text part ends up Transitional.
    #[test]
    fn strict_package_is_rewritten_to_transitional() {
        let strict_doc = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<w:document xmlns:w=\"http://purl.oclc.org/ooxml/wordprocessingml/main\">",
            "<w:body/></w:document>",
        );
        let strict_rels = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
            "<Relationship Id=\"rId1\" ",
            "Type=\"http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument\" ",
            "Target=\"word/document.xml\"/>",
            "</Relationships>",
        );
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer.start_file("_rels/.rels", options).unwrap();
        writer.write_all(strict_rels.as_bytes()).unwrap();
        writer.start_file("word/document.xml", options).unwrap();
        writer.write_all(strict_doc.as_bytes()).unwrap();
        let strict_pkg = writer.finish().unwrap().into_inner();

        let out = strict_to_transitional_docx(&strict_pkg).unwrap();
        assert_ne!(out, strict_pkg, "strict package must be rewritten");
        assert!(
            out.windows(4).any(|w| w == b"PK\x03\x04"),
            "output is still a zip"
        );

        let mut reread = ZipArchive::new(Cursor::new(out)).unwrap();
        let doc = {
            let mut f = reread.by_name("word/document.xml").unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            s
        };
        assert!(doc.contains("http://schemas.openxmlformats.org/wordprocessingml/2006/main"));
        assert!(!doc.contains(STRICT_MARKER));
        let rels = {
            let mut f = reread.by_name("_rels/.rels").unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();
            s
        };
        assert!(rels.contains(
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
        ));
        assert!(!rels.contains(STRICT_MARKER));
        for i in 0..reread.len() {
            let entry = reread.by_index(i).unwrap();
            assert_eq!(
                entry.last_modified(),
                Some(zip::DateTime::default()),
                "{} carries no wall-clock time",
                entry.name().expect("decode ZIP fixture name")
            );
        }
    }

    /// A non-ZIP byte array is returned byte-identical because it cannot be parsed.
    #[test]
    fn invalid_zip_is_returned_untouched() {
        let invalid_bytes = b"not a zip file";
        let out = strict_to_transitional_docx(invalid_bytes).unwrap();
        assert_eq!(
            out, invalid_bytes,
            "an invalid zip should be returned untouched"
        );
    }

    /// Directory entries are skipped and not included in the rebuilt archive,
    /// while valid strict parts are translated.
    #[test]
    fn directory_entries_are_skipped() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

        // Add a directory entry
        writer.add_directory("word/", options).unwrap();

        let strict_doc = concat!(
            "<?xml version=\"1.0\"?>\n",
            "<w:document xmlns:w=\"http://purl.oclc.org/ooxml/wordprocessingml/main\">",
            "<w:body/></w:document>",
        );
        writer.start_file("word/document.xml", options).unwrap();
        writer.write_all(strict_doc.as_bytes()).unwrap();

        let strict_pkg = writer.finish().unwrap().into_inner();

        let out = strict_to_transitional_docx(&strict_pkg).unwrap();

        assert_ne!(
            out, strict_pkg,
            "the valid strict part should cause a rewrite"
        );

        let mut reread = ZipArchive::new(Cursor::new(out)).unwrap();

        // The directory should have been skipped, leaving only the file.
        assert_eq!(reread.len(), 1, "directory entry should have been skipped");
        let file = reread.by_index(0).unwrap();
        assert_eq!(file.name().unwrap(), "word/document.xml");
    }
}
