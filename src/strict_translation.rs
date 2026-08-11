// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

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
//!   - **`.xml` parts** (element/attribute namespaces) → [`NAMESPACE_TABLE`].
//!   - **`.rels` parts** (relationship `Type=`) → [`RELATIONSHIP_TABLE`].
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
use zip::{CompressionMethod, ZipArchive, ZipWriter};

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

/// [`NAMESPACE_TABLE`] sorted by descending strict-key length, built once.
static NAMESPACE_SORTED: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut v = NAMESPACE_TABLE.to_vec();
    v.sort_by_key(|e| std::cmp::Reverse(e.0.len()));
    v
});

/// [`RELATIONSHIP_TABLE`] sorted by descending strict-key length, built once.
static RELATIONSHIP_SORTED: LazyLock<Vec<(&'static str, &'static str)>> = LazyLock::new(|| {
    let mut v = RELATIONSHIP_TABLE.to_vec();
    v.sort_by_key(|e| std::cmp::Reverse(e.0.len()));
    v
});

/// Apply a (strict → transitional) table to `text` via sequential
/// `str::replace`, longest key first. `text` is assumed to already contain
/// [`STRICT_MARKER`] (callers gate on that for the fast path).
fn translate(text: &str, table: &[(&str, &str)]) -> String {
    let mut out = text.to_string();
    for &(from, to) in table {
        if out.contains(from) {
            out = out.replace(from, to);
        }
    }
    out
}

/// Normalize an ISO/IEC 29500 **Strict** `.docx` to **Transitional**.
///
/// Walks every zip entry; rewrites Strict URIs in `.rels` parts using
/// [`RELATIONSHIP_TABLE`] and in `.xml` parts (including `[Content_Types].xml`)
/// using [`NAMESPACE_TABLE`]; binary parts pass through untouched.
///
/// **Zero-churn**: if no entry contains [`STRICT_MARKER`] (i.e. the package is
/// already Transitional), the original `bytes` are returned unchanged — the zip
/// is never rebuilt. Only when at least one URI was rewritten is a new zip
/// assembled (every entry under its original name, Deflated).
pub fn strict_to_transitional_docx(bytes: &[u8]) -> Vec<u8> {
    // Not a readable zip — leave untouched (PartFs::open reports the real
    // error downstream).
    let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes.to_vec())) else {
        return bytes.to_vec();
    };

    let n = archive.len();
    // Drain the archive into (name, bytes) so we can rebuild afterward. Track
    // whether ANY entry actually changed to preserve the zero-churn fast path.
    let mut entries: Vec<(String, Vec<u8>)> = Vec::with_capacity(n);
    let mut any_changed = false;
    for i in 0..n {
        let Ok(mut f) = archive.by_index(i) else {
            return bytes.to_vec();
        };
        if f.is_dir() {
            continue;
        }
        let name = f.name().to_string();
        let mut buf = Vec::with_capacity(f.size() as usize);
        if f.read_to_end(&mut buf).is_err() {
            return bytes.to_vec();
        }

        // Fast path: only text parts carrying the Strict marker can change.
        if name.ends_with(".rels") || name.ends_with(".xml") {
            let text = String::from_utf8_lossy(&buf);
            if text.contains(STRICT_MARKER) {
                let table = if name.ends_with(".rels") {
                    &*RELATIONSHIP_SORTED
                } else {
                    &*NAMESPACE_SORTED
                };
                let translated = translate(&text, table);
                if translated != text.as_ref() {
                    any_changed = true;
                    buf = translated.into_bytes();
                }
            }
        }

        entries.push((name, buf));
    }

    if !any_changed {
        // Already Transitional: byte-identical round-trip.
        return bytes.to_vec();
    }

    // Rebuild the package with translated parts.
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, data) in &entries {
        if writer.start_file(name, options).is_err() {
            return bytes.to_vec();
        }
        if writer.write_all(data).is_err() {
            return bytes.to_vec();
        }
    }
    match writer.finish() {
        Ok(c) => c.into_inner(),
        Err(_) => bytes.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let out = translate(strict, &NAMESPACE_SORTED);
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
        let out = translate(strict_rels, &RELATIONSHIP_SORTED);
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

        let out = strict_to_transitional_docx(&pkg);
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

        let out = strict_to_transitional_docx(&strict_pkg);
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
    }
}
