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

//! Resource admission for untrusted DOCX input: bound the ZIP container
//! before anything inflates it without limits.
//!
//! [`admit`] reads the central directory, refuses names that are not safe
//! part names, duplicates, encryption and compression other than stored or
//! deflate, then inflates every entry through a counting reader capped at the
//! remaining budget (central-directory sizes are untrusted), checks each XML
//! part's nesting depth, and confirms the package is a WordprocessingML
//! document by its relationships and content types. Nothing is extracted to
//! disk, no macro runs and no external reference is resolved.
//!
//! The agent-facing entry points ([`crate::inspect`], [`crate::edit`] and the
//! bindings over them) admit with [`InputLimits::default`]. The redline
//! comparer ([`crate::document_comparer`]) and [`crate::WmlDocument`] admit
//! with the roomier [`InputLimits::compare`], which
//! [`crate::comparer::WmlComparerSettings::input_limits`] overrides.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt;
use std::io::{Cursor, Read};

use quick_xml::Reader;
use quick_xml::events::Event;
use zip::CompressionMethod;

/// Resource budgets. The defaults are a product policy, not a proof against
/// every denial of service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputLimits {
    /// Size of the ZIP file itself.
    pub max_compressed_bytes: u64,
    /// ZIP entries, directories included.
    pub max_entries: usize,
    /// Inflated size of any one entry.
    pub max_part_bytes: u64,
    /// Inflated size of all entries together.
    pub max_uncompressed_bytes: u64,
    /// Element nesting in any XML part.
    pub max_xml_depth: usize,
    /// Attributes per element, including namespace declarations.
    pub max_element_attributes: usize,
    /// UTF-8 bytes in a qualified attribute name.
    pub max_attribute_name_bytes: usize,
    /// Bytes in a raw or decoded attribute value.
    pub max_attribute_value_bytes: usize,
    /// Bytes in a complete start tag, including syntax and delimiters.
    pub max_start_tag_bytes: usize,
    /// XML element start tags in all the package's XML parts together.
    /// Memory follows the element count, not the bytes: a few hundred KB of
    /// zipped one-run paragraphs is millions of DOM nodes and several GB
    /// once parsed and laid out (#2014).
    pub max_xml_elements: u64,
    /// Characters of text plus elements in one side of a comparison: its
    /// body and notes together, or one header or footer. `compare` turns
    /// every character into an atom of 2-4 KB, so 1 MiB of text peaked at
    /// 2.1-3.9 GB and B1's 66 MiB of spaces passed 6 GB (word_exp
    /// cmp_scale, 2026-10-31). Only the comparer reads it.
    pub max_story_units: u64,
}

impl Default for InputLimits {
    fn default() -> Self {
        const MIB: u64 = 1024 * 1024;
        Self {
            max_compressed_bytes: 64 * MIB,
            max_entries: 10_000,
            max_part_bytes: 64 * MIB,
            max_uncompressed_bytes: 256 * MIB,
            max_xml_depth: 256,
            max_element_attributes: crate::xmllinq::parse::MAX_ELEMENT_ATTRIBUTES,
            max_attribute_name_bytes: crate::xmllinq::parse::MAX_ATTRIBUTE_NAME_BYTES,
            max_attribute_value_bytes: crate::xmllinq::parse::MAX_ATTRIBUTE_VALUE_BYTES,
            max_start_tag_bytes: crate::xmllinq::parse::MAX_TAG_ATTRIBUTE_BYTES,
            // The largest of 2,434 real documents (the bench corpora and
            // tests/) holds 365,343 elements: 2,000,000 refuses none of them
            // and bounds `read` near 3 GB: E5, just under it, peaked at
            // 2.2 GB with page markers and 2.9 GB without (#2014).
            max_xml_elements: 2_000_000,
            max_story_units: 2_000_000,
        }
    }
}

impl InputLimits {
    /// The redline comparer's budget: the same entry and depth caps as
    /// [`Self::default`], with room for the embedded media legal corpora
    /// carry (512 MiB per file and per part, 2 GiB inflated in all). Hosts
    /// that know their documents set
    /// [`crate::comparer::WmlComparerSettings::input_limits`] tighter.
    #[must_use]
    pub const fn compare() -> Self {
        const MIB: u64 = 1024 * 1024;
        Self {
            max_compressed_bytes: 512 * MIB,
            max_entries: 10_000,
            max_part_bytes: 512 * MIB,
            max_uncompressed_bytes: 2048 * MIB,
            max_xml_depth: 256,
            max_element_attributes: crate::xmllinq::parse::MAX_ELEMENT_ATTRIBUTES,
            max_attribute_name_bytes: crate::xmllinq::parse::MAX_ATTRIBUTE_NAME_BYTES,
            max_attribute_value_bytes: crate::xmllinq::parse::MAX_ATTRIBUTE_VALUE_BYTES,
            max_start_tag_bytes: crate::xmllinq::parse::MAX_TAG_ATTRIBUTE_BYTES,
            max_xml_elements: 4_000_000,
            // The largest of 6,645 real documents holds 925,884 (a redline;
            // the largest original 557,534).
            max_story_units: 2_000_000,
        }
    }
}

/// Field-by-field overrides of an [`InputLimits`] budget: the shape the
/// Python and WASM bindings take as a JSON object. Every key is optional and
/// keeps the base value when absent; an unknown key is refused, so a typo
/// cannot silently leave a default in force.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputLimitOverrides {
    /// Replaces [`InputLimits::max_compressed_bytes`].
    pub max_compressed_bytes: Option<u64>,
    /// Replaces [`InputLimits::max_entries`].
    pub max_entries: Option<usize>,
    /// Replaces [`InputLimits::max_part_bytes`].
    pub max_part_bytes: Option<u64>,
    /// Replaces [`InputLimits::max_uncompressed_bytes`].
    pub max_uncompressed_bytes: Option<u64>,
    /// Replaces [`InputLimits::max_xml_depth`].
    pub max_xml_depth: Option<usize>,
    /// Replaces [`InputLimits::max_element_attributes`].
    pub max_element_attributes: Option<usize>,
    /// Replaces [`InputLimits::max_attribute_name_bytes`].
    pub max_attribute_name_bytes: Option<usize>,
    /// Replaces [`InputLimits::max_attribute_value_bytes`].
    pub max_attribute_value_bytes: Option<usize>,
    /// Replaces [`InputLimits::max_start_tag_bytes`].
    pub max_start_tag_bytes: Option<usize>,
    /// Replaces [`InputLimits::max_xml_elements`].
    pub max_xml_elements: Option<u64>,
    /// Replaces [`InputLimits::max_story_units`].
    pub max_story_units: Option<u64>,
}

impl InputLimitOverrides {
    /// Parse a JSON object such as `{"max_part_bytes": 67108864}`.
    ///
    /// # Errors
    ///
    /// `invalid input limits: ...` for malformed JSON, an unknown key, or a
    /// value that is not a non-negative integer in range.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("invalid input limits: {e}"))
    }

    /// `base` with every given key replaced.
    #[must_use]
    pub fn apply(self, base: InputLimits) -> InputLimits {
        InputLimits {
            max_compressed_bytes: self
                .max_compressed_bytes
                .unwrap_or(base.max_compressed_bytes),
            max_entries: self.max_entries.unwrap_or(base.max_entries),
            max_part_bytes: self.max_part_bytes.unwrap_or(base.max_part_bytes),
            max_uncompressed_bytes: self
                .max_uncompressed_bytes
                .unwrap_or(base.max_uncompressed_bytes),
            max_xml_depth: self.max_xml_depth.unwrap_or(base.max_xml_depth),
            max_element_attributes: self
                .max_element_attributes
                .unwrap_or(base.max_element_attributes),
            max_attribute_name_bytes: self
                .max_attribute_name_bytes
                .unwrap_or(base.max_attribute_name_bytes),
            max_attribute_value_bytes: self
                .max_attribute_value_bytes
                .unwrap_or(base.max_attribute_value_bytes),
            max_start_tag_bytes: self.max_start_tag_bytes.unwrap_or(base.max_start_tag_bytes),
            max_xml_elements: self.max_xml_elements.unwrap_or(base.max_xml_elements),
            max_story_units: self.max_story_units.unwrap_or(base.max_story_units),
        }
    }
}

/// Why a package was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionErrorKind {
    /// A budget in [`InputLimits`] was exceeded (`INPUT_LIMIT`).
    InputLimit,
    /// Two entries name the same part (`DUPLICATE_PART`).
    DuplicatePart,
    /// Not a WordprocessingML package, an unsafe part name, encryption or an
    /// unsupported compression method (`UNSUPPORTED_PACKAGE`).
    UnsupportedPackage,
    /// Not a readable ZIP, or an entry fails its CRC (`INVALID_PACKAGE`).
    InvalidPackage,
    /// A ZIP local file header disagrees with the central directory
    /// (`ZIP_HEADER_MISMATCH`): a dual-header ("zip cloaking",
    /// GHSA-m665-v7pm-7grj) package whose two headers describe different
    /// entries. Every reader here trusts the central directory; a tool
    /// scanning local headers would see another package, so admission
    /// refuses the file instead of picking a side.
    ZipHeaderMismatch,
    /// An XML part is malformed or nests too deep (`INVALID_XML`).
    InvalidXml,
    /// A Word 97-2003 `.doc`, or an encrypted document of any Word version:
    /// an OLE compound file, which only Word reads (`LEGACY_DOC`).
    LegacyDocument,
}

impl AdmissionErrorKind {
    /// The stable error code agents and bindings see.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputLimit => "INPUT_LIMIT",
            Self::DuplicatePart => "DUPLICATE_PART",
            Self::UnsupportedPackage => "UNSUPPORTED_PACKAGE",
            Self::InvalidPackage => "INVALID_PACKAGE",
            Self::ZipHeaderMismatch => "ZIP_HEADER_MISMATCH",
            Self::InvalidXml => "INVALID_XML",
            Self::LegacyDocument => "LEGACY_DOC",
        }
    }

    /// Every kind, in code order.
    pub const ALL: [Self; 7] = [
        Self::InputLimit,
        Self::DuplicatePart,
        Self::UnsupportedPackage,
        Self::InvalidPackage,
        Self::ZipHeaderMismatch,
        Self::InvalidXml,
        Self::LegacyDocument,
    ];
}

/// A refusal's message with its code first (`CODE: context: detail`): the
/// comparer's opaque `I/O error: ` wrapper is dropped, any other context
/// before the code (`document B: `) stays after it. `None` when no
/// `: `-separated segment of `message` is an admission code. The CLI and
/// the bindings print refusals through it, so a code reads the same
/// whatever wrapped it.
#[must_use]
pub fn code_first(message: &str) -> Option<String> {
    let segments: Vec<&str> = message.split(": ").collect();
    let at = segments.iter().position(|segment| {
        AdmissionErrorKind::ALL
            .iter()
            .any(|kind| kind.code() == *segment)
    })?;
    let (before, from) = segments.split_at(at);
    let mut parts = from.iter().take(1).copied().collect::<Vec<_>>();
    parts.extend(
        before
            .iter()
            .copied()
            .filter(|segment| *segment != "I/O error" && !segment.is_empty()),
    );
    parts.extend(from.iter().skip(1).copied());
    Some(parts.join(": "))
}

/// The first bytes of an OLE compound file: a Word 97-2003 `.doc`, or a
/// password-encrypted document of any Word version.
const OLE_MAGIC: &[u8] = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1";

/// Refuse, by its signature, a file that is a document but not a `.docx`
/// package: an OLE compound file (`LEGACY_DOC`, with the save-as hint) or
/// RTF (`UNSUPPORTED_PACKAGE`). [`admit`] runs it before any budget, and
/// the CLI runs it as it reads a file, so the two give one message.
///
/// # Errors
///
/// An [`AdmissionError`] naming what the file is instead.
pub fn sniff(bytes: &[u8]) -> Result<(), AdmissionError> {
    if bytes.starts_with(OLE_MAGIC) {
        return Err(AdmissionError::new(
            AdmissionErrorKind::LegacyDocument,
            "a Word 97-2003 (.doc) or encrypted document; open it in Word and save it as .docx without a password",
        ));
    }
    if bytes.starts_with(b"{\\rtf") {
        return Err(AdmissionError::new(
            AdmissionErrorKind::UnsupportedPackage,
            "an RTF file, not a .docx package",
        ));
    }
    Ok(())
}

/// [`sniff`], and a Word input's ZIP signature: what a file must be before
/// it is read as a `.docx`. The CLI runs it on every Word input, so a file
/// that is not a package reads the same whichever task met it.
///
/// # Errors
///
/// An [`AdmissionError`] naming what the file is instead (`INVALID_PACKAGE`
/// for no end-of-central-directory record).
pub fn sniff_package(bytes: &[u8]) -> Result<(), AdmissionError> {
    sniff(bytes)?;
    declared_entry_count(bytes).map(|_| ())
}

/// A refused package: the kind and what exactly was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionError {
    /// The error class.
    pub kind: AdmissionErrorKind,
    /// Human-readable detail, naming the entry when there is one.
    pub message: String,
}

impl AdmissionError {
    pub(crate) fn new(kind: AdmissionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// The stable error code (`INPUT_LIMIT`, …).
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.kind.code()
    }
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.message)
    }
}

impl std::error::Error for AdmissionError {}

/// Facts about an admitted package.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedPackage {
    /// ZIP entries, directories included.
    pub entries: usize,
    /// Inflated bytes of all entries.
    pub inflated_bytes: u64,
    /// The main document part (`word/document.xml` in most files).
    pub main_part: String,
}

const MAIN_CONTENT_TYPES: &[&str] = &[
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml",
    "application/vnd.ms-word.document.macroEnabled.main+xml",
    "application/vnd.ms-word.template.macroEnabledTemplate.main+xml",
];

const OFFICE_DOCUMENT_REL: &[&str] = &[
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
    "http://purl.oclc.org/ooxml/officeDocument/relationships/officeDocument",
];

/// `INPUT_LIMIT` for an input past the compressed-size budget: one wording
/// shared by [`admit`] and the bounded package constructor in
/// [`crate::opc`], so a refusal reads the same wherever it fires (#645).
pub(crate) fn input_over_compressed(size: u64, limits: &InputLimits) -> AdmissionError {
    AdmissionError::new(
        AdmissionErrorKind::InputLimit,
        format!(
            "input is {size} bytes; the limit is {}",
            limits.max_compressed_bytes
        ),
    )
}

/// `INPUT_LIMIT` for a declared entry count past the entry budget.
pub(crate) fn input_over_entries(declared: u64, limits: &InputLimits) -> AdmissionError {
    AdmissionError::new(
        AdmissionErrorKind::InputLimit,
        format!(
            "{declared} ZIP entries; the limit is {}",
            limits.max_entries
        ),
    )
}

/// `name` as a message may print it: each control character spelled as
/// its escape (`\u{1b}`), everything else as the archive spells it, so a
/// crafted entry name cannot write terminal sequences while an ordinary
/// one (quotes, non-ASCII) reads unchanged.
pub(crate) fn shown(name: &str) -> Cow<'_, str> {
    if name.chars().any(char::is_control) {
        name.chars()
            .map(|c| {
                if c.is_control() {
                    c.escape_default().to_string()
                } else {
                    c.to_string()
                }
            })
            .collect::<String>()
            .into()
    } else {
        name.into()
    }
}

/// `INPUT_LIMIT` for an entry whose declared inflated size already passes
/// the per-part budget: refused before anything inflates.
pub(crate) fn part_declared_over(
    name: &str,
    declared: u64,
    limits: &InputLimits,
) -> AdmissionError {
    AdmissionError::new(
        AdmissionErrorKind::InputLimit,
        format!(
            "{} declares {declared} inflated bytes; the part limit is {}",
            shown(name),
            limits.max_part_bytes
        ),
    )
}

/// `INPUT_LIMIT` for an entry that inflated past `cap`, the smaller of the
/// per-part budget and the package budget left.
pub(crate) fn part_inflated_over(name: &str, cap: u64, limits: &InputLimits) -> AdmissionError {
    let which = if cap == limits.max_part_bytes {
        format!("the part limit is {}", limits.max_part_bytes)
    } else {
        format!("the package limit is {}", limits.max_uncompressed_bytes)
    };
    AdmissionError::new(
        AdmissionErrorKind::InputLimit,
        format!("{} inflates past {cap} bytes; {which}", shown(name)),
    )
}

/// Check the input size and EOCD/ZIP64 entry count without building an index.
/// Returns the declared count so admission can also detect lost duplicates.
pub(crate) fn preindex_budget(bytes: &[u8], limits: InputLimits) -> Result<u64, AdmissionError> {
    if bytes.len() as u64 > limits.max_compressed_bytes {
        return Err(input_over_compressed(bytes.len() as u64, &limits));
    }
    let declared = declared_entry_count(bytes)?;
    if declared > limits.max_entries as u64 {
        return Err(input_over_entries(declared, &limits));
    }
    Ok(declared)
}

/// Tally actual inflation, with the same refusal wherever a reader stops.
pub(crate) fn tally_inflated(
    total: &mut u64,
    read: u64,
    name: &str,
    cap: u64,
    limits: &InputLimits,
) -> Result<(), AdmissionError> {
    if read > cap {
        return Err(part_inflated_over(name, cap, limits));
    }
    *total = total
        .checked_add(read)
        .filter(|sum| *sum <= limits.max_uncompressed_bytes)
        .ok_or_else(|| part_inflated_over(name, cap, limits))?;
    Ok(())
}

/// Count one part's XML element start tags into `total`, the way the
/// tolerant reader sees them, and refuse a markup declaration (a DOCTYPE).
///
/// Shared by [`admit`] and the package open every command goes through
/// (`opc::read_package`), so the cap holds on the tolerant `read` path
/// too, where a part is never checked for well-formedness (#2014). Every
/// part is scanned, whatever its name: one whose first byte (after a
/// byte-order mark and white space) is not `<` is not XML and costs a few
/// bytes to rule out, so media never charges the budget.
///
/// The tolerant parser makes an element of every `<` that does not open an
/// end tag, a comment, a CDATA section, a processing instruction or a
/// declaration, whatever follows it (`<0>` and `< x>` included), so this
/// counts each such `<` and skips the rest whole.
///
/// A markup declaration (`<!DOCTYPE`, `<!ENTITY`, … in any case, anywhere
/// in the part) is refused because Word refuses every part that carries a DOCTYPE,
/// even a bare `<!DOCTYPE w:document>` with no internal subset (#2015,
/// probed 2026-10-31), and because an internal subset is where entity
/// bombs and external entities live.
/// Bytes read from a part before deciding whether it is markup: room for
/// a byte-order mark and leading white space.
const XML_SNIFF_BYTES: u64 = 4096;

/// `rest` (what follows a `<`) opens a markup declaration: `!DOCTYPE`,
/// `!ENTITY`, `!ELEMENT`, `!ATTLIST` or `!NOTATION`, in any case. Another
/// `<!` inside an element is an element to the tolerant parser, and counts.
fn declaration(rest: &[u8]) -> bool {
    ["!DOCTYPE", "!ENTITY", "!ELEMENT", "!ATTLIST", "!NOTATION"]
        .iter()
        .any(|keyword| {
            rest.get(..keyword.len())
                .is_some_and(|head| head.eq_ignore_ascii_case(keyword.as_bytes()))
        })
}

/// The part's name says XML: `.xml` or `.rels`, in any case.
fn has_xml_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.ends_with(".xml") || lower.ends_with(".rels")
}

/// `head` opens with `<` after an optional byte-order mark and white
/// space, in UTF-8 or UTF-16: the parts [`tally_xml_elements`] scans.
fn looks_like_markup(head: &[u8]) -> bool {
    match head {
        [0xFF, 0xFE, rest @ ..] | [0xFE, 0xFF, rest @ ..] => {
            rest.iter()
                .copied()
                .find(|b| *b != 0 && !b.is_ascii_whitespace())
                == Some(b'<')
        }
        _ => {
            head.strip_prefix(b"\xEF\xBB\xBF")
                .unwrap_or(head)
                .trim_ascii_start()
                .first()
                == Some(&b'<')
        }
    }
}

pub(crate) fn tally_xml_elements(
    total: &mut u64,
    name: &str,
    data: &[u8],
    limits: &InputLimits,
) -> Result<(), AdmissionError> {
    scan_markup(total, name, data, limits, true)
}

/// [`tally_xml_elements`] without the DTD refusal, for `jubarte debug`,
/// which reports a refusable package instead of refusing it: a declaration
/// is skipped and the elements after it are still counted.
pub(crate) fn count_xml_elements(
    total: &mut u64,
    name: &str,
    data: &[u8],
    limits: &InputLimits,
) -> Result<(), AdmissionError> {
    scan_markup(total, name, data, limits, false)
}

fn scan_markup(
    total: &mut u64,
    name: &str,
    data: &[u8],
    limits: &InputLimits,
    refuse_dtd: bool,
) -> Result<(), AdmissionError> {
    // UTF-16 (a byte-order mark, as SharePoint writes `customXml`) is
    // transcoded; anything else is scanned as bytes, `<` being ASCII in
    // UTF-8 and every single-byte encoding Word reads.
    let utf16 = matches!(data, [0xFF, 0xFE, ..] | [0xFE, 0xFF, ..]);
    let decoded;
    let bytes = if utf16 {
        decoded = part_text(data);
        decoded.as_bytes()
    } else {
        data.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(data)
    };
    let bytes = bytes.trim_ascii_start();
    // Only an XML part carries Word's DTD rule: an altChunk's HTML may
    // open with `<!DOCTYPE html>`, which Word's HTML import reads.
    let xml = refuse_dtd && has_xml_name(name);
    if bytes.first() != Some(&b'<') {
        return Ok(());
    }
    // The index just past the first `end` at or after `from`; the end of
    // the part when it never comes.
    let past = |from: usize, end: &[u8]| {
        bytes
            .get(from..)
            .and_then(|tail| tail.windows(end.len()).position(|w| w == end))
            .map_or(bytes.len(), |at| {
                from.saturating_add(at).saturating_add(end.len())
            })
    };
    let mut at = 0usize;
    while let Some(offset) = bytes
        .get(at..)
        .and_then(|rest| rest.iter().position(|&b| b == b'<'))
    {
        let open = at.saturating_add(offset);
        let rest = bytes.get(open.saturating_add(1)..).unwrap_or_default();
        at = open.saturating_add(1);
        match rest.first() {
            Some(b'/') => {}
            Some(b'?') => at = past(open.saturating_add(2), b"?>"),
            Some(b'!') if rest.starts_with(b"!--") => at = past(open.saturating_add(4), b"-->"),
            Some(b'!') if rest.starts_with(b"![CDATA[") => {
                at = past(open.saturating_add(9), b"]]>");
            }
            Some(b'!') if declaration(rest) && xml => {
                return Err(AdmissionError::new(
                    AdmissionErrorKind::InvalidXml,
                    format!("{}: DTD declarations are unsupported", shown(name)),
                ));
            }
            Some(b'!') if declaration(rest) => at = past(open.saturating_add(2), b">"),
            _ => {
                *total = total.saturating_add(1);
                if *total > limits.max_xml_elements {
                    return Err(AdmissionError::new(
                        AdmissionErrorKind::InputLimit,
                        format!(
                            "{} brings the package past {} XML elements, the element limit",
                            shown(name),
                            limits.max_xml_elements
                        ),
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Admit `bytes` as a DOCX within `limits`, returning facts about it on
/// success and a typed refusal on the first budget or rule the package
/// breaks.
///
/// # Errors
///
/// An [`AdmissionError`] naming the first budget or rule the package
/// breaks; the kind's [`AdmissionErrorKind::code`] is the stable string
/// the CLI and bindings print (`INPUT_LIMIT`, `DUPLICATE_PART`,
/// `UNSUPPORTED_PACKAGE`, `INVALID_PACKAGE`, `ZIP_HEADER_MISMATCH`,
/// `INVALID_XML`, `LEGACY_DOC`).
pub fn admit(bytes: &[u8], limits: InputLimits) -> Result<AdmittedPackage, AdmissionError> {
    use AdmissionErrorKind as K;

    sniff(bytes)?;

    preindex_budget(bytes, limits)?;
    // The shared open (`crate::opc::open_archive`) runs this same check, but
    // as a plain zip error; admission opens unchecked so the refusal keeps
    // its typed kind.
    let mut archive = crate::opc::open_archive_unchecked(Cursor::new(bytes))
        .map_err(|e| AdmissionError::new(K::InvalidPackage, format!("not a ZIP file: {e}")))?;
    // GHSA-m665-v7pm-7grj (zip <= 9.0.4) and ZIP-17 (#644): a local header
    // may disagree with the central directory — a scanner trusting local
    // headers would see a different package — and an entry's extent may
    // overlap another's or reach into the central directory. `ZipArchive`
    // reads names and content through the central directory, so the two
    // records must agree (`zip_defects`). zip 9.0.5's own
    // `validate_header_consistency` is stricter than the format: it also
    // refuses a local time or a central-only Unicode Path field, both of
    // which real Word-opened files carry. `zip_defects` also walks the raw
    // central directory, so two records naming one part — an exact
    // duplicate the index would collapse, or an ASCII-case alias — refuse
    // here with `DUPLICATE_PART` (ZIP-15/#600).
    if let Some(defect) = crate::opc::zip_defects(&archive, &mut Cursor::new(bytes))
        .into_iter()
        .next()
    {
        return Err(defect.refusal());
    }

    let mut names: Vec<String> = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let entry = archive
            .by_index_raw(i)
            .map_err(|e| AdmissionError::new(K::InvalidPackage, e.to_string()))?;
        // zip 8's name decode: bit 11 means UTF-8, otherwise CP437, and a
        // 0x7075 extra field overrides the header name (see `zip_entry_name`).
        let name = zip_entry_name(
            entry.name_raw(),
            entry.flags().as_u16(),
            unicode_path_fields(entry.extra_data_fields().cloned()),
        )
        .map_err(|e| AdmissionError::new(K::InvalidPackage, e.to_string()))?;
        check_part_name(&name)?;
        names.push(name.clone());
        if entry.encrypted() {
            return Err(AdmissionError::new(
                K::UnsupportedPackage,
                format!("{} is encrypted", shown(&name)),
            ));
        }
        if !matches!(
            entry.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(AdmissionError::new(
                K::UnsupportedPackage,
                format!(
                    "{} uses unsupported compression {}",
                    shown(&name),
                    entry.compression()
                ),
            ));
        }
        if entry.size() > limits.max_part_bytes {
            return Err(part_declared_over(&name, entry.size(), &limits));
        }
    }

    let mut total = 0u64;
    let mut elements = 0u64;
    let mut kept: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AdmissionError::new(K::InvalidPackage, e.to_string()))?;
        // A directory entry's payload is validated through the same bounded
        // CRC stream and charged to the budgets like any entry's (#644);
        // it just contributes no part to the kept map.
        let name = zip_entry_name(
            entry.name_raw(),
            entry.flags().as_u16(),
            unicode_path_fields(entry.extra_data_fields().cloned()),
        )
        .map_err(|e| AdmissionError::new(K::InvalidPackage, e.to_string()))?;
        let remaining = limits.max_uncompressed_bytes.saturating_sub(total);
        let cap = limits.max_part_bytes.min(remaining);
        let lower = name.to_ascii_lowercase();
        let is_xml = has_xml_name(&name);
        let mut limited = (&mut entry).take(cap.saturating_add(1));
        let read = if is_xml {
            let mut buf = Vec::new();
            limited.read_to_end(&mut buf).map_err(|e| {
                AdmissionError::new(K::InvalidPackage, format!("{}: {e}", shown(&name)))
            })?;
            let n = buf.len() as u64;
            if n <= cap {
                check_xml_depth(&name, &buf, limits)?;
                tally_xml_elements(&mut elements, &name, &buf, &limits)?;
                if lower == "[content_types].xml" || lower == "_rels/.rels" {
                    kept.insert(lower, buf);
                }
            }
            n
        } else {
            // Any other part is charged to the element budget too, as the
            // package open charges it (`opc::read_package`): XML under
            // another name still reaches the tolerant parser. Its first
            // bytes tell; media is streamed past, never held.
            let io = |e: std::io::Error| {
                AdmissionError::new(K::InvalidPackage, format!("{}: {e}", shown(&name)))
            };
            let mut head = Vec::new();
            (&mut limited)
                .take(XML_SNIFF_BYTES)
                .read_to_end(&mut head)
                .map_err(io)?;
            if looks_like_markup(&head) {
                limited.read_to_end(&mut head).map_err(io)?;
                let n = head.len() as u64;
                if n <= cap {
                    tally_xml_elements(&mut elements, &name, &head, &limits)?;
                }
                n
            } else {
                (head.len() as u64)
                    .saturating_add(std::io::copy(&mut limited, &mut std::io::sink()).map_err(io)?)
            }
        };
        tally_inflated(&mut total, read, &name, cap, &limits)?;
    }

    let main_part = main_part(&names, &kept)?;
    // The scan above is by extension; the main part is parsed whatever it is
    // called, so one named otherwise is read once more for its depth.
    let lower = main_part.to_ascii_lowercase();
    if !(lower.ends_with(".xml") || lower.ends_with(".rels")) {
        let mut entry = archive.by_name(&main_part).map_err(|e| {
            AdmissionError::new(K::InvalidPackage, format!("{}: {e}", shown(&main_part)))
        })?;
        let mut buf = Vec::new();
        (&mut entry)
            .take(limits.max_part_bytes.saturating_add(1))
            .read_to_end(&mut buf)
            .map_err(|e| {
                AdmissionError::new(K::InvalidPackage, format!("{}: {e}", shown(&main_part)))
            })?;
        // Its elements were counted in the scan, whatever its name.
        check_xml_depth(&main_part, &buf, limits)?;
    }
    Ok(AdmittedPackage {
        entries: archive.len(),
        inflated_bytes: total,
        main_part,
    })
}

/// The entry count the end-of-central-directory record declares (ZIP64
/// aware), which the reader's name map can undercount.
fn declared_entry_count(bytes: &[u8]) -> Result<u64, AdmissionError> {
    const EOCD: [u8; 4] = [0x50, 0x4b, 0x05, 0x06];
    const LOCATOR: [u8; 4] = [0x50, 0x4b, 0x06, 0x07];
    const EOCD64: [u8; 4] = [0x50, 0x4b, 0x06, 0x06];
    let invalid = |what: &str| {
        AdmissionError::new(
            AdmissionErrorKind::InvalidPackage,
            format!("not a ZIP file: {what}"),
        )
    };
    let floor = bytes.len().saturating_sub(22 + usize::from(u16::MAX));
    let at = (floor..=bytes.len().saturating_sub(22))
        .rev()
        .find(|&i| bytes.get(i..).is_some_and(|tail| tail.starts_with(&EOCD)))
        .ok_or_else(|| invalid("no end-of-central-directory record"))?;
    let total = bytes
        .get(at.saturating_add(10)..at.saturating_add(12))
        .and_then(|field| <[u8; 2]>::try_from(field).ok())
        .map(u16::from_le_bytes)
        .ok_or_else(|| invalid("truncated end-of-central-directory record"))?;
    if total != u16::MAX {
        return Ok(u64::from(total));
    }
    let locator = at
        .checked_sub(20)
        .and_then(|l| bytes.get(l..at))
        .filter(|locator| locator.starts_with(&LOCATOR))
        .ok_or_else(|| invalid("ZIP64 locator missing"))?;
    let offset = locator
        .get(8..16)
        .and_then(|field| <[u8; 8]>::try_from(field).ok())
        .ok_or_else(|| invalid("ZIP64 locator truncated"))?;
    // The offset is attacker-controlled: on 64-bit every u64 fits a usize,
    // so the end of the record is computed checked and read through `get`.
    let record = usize::try_from(u64::from_le_bytes(offset))
        .ok()
        .and_then(|r| r.checked_add(40).map(|end| (r, end)))
        .and_then(|(r, end)| bytes.get(r..end))
        .filter(|record| record.starts_with(&EOCD64))
        .ok_or_else(|| invalid("ZIP64 end record missing"))?;
    let count = record
        .get(32..40)
        .and_then(|field| <[u8; 8]>::try_from(field).ok())
        .ok_or_else(|| invalid("ZIP64 end record truncated"))?;
    Ok(u64::from_le_bytes(count))
}

/// IBM code page 437's high half, byte 0x80–0xFF to its Unicode character
/// (the Unicode Consortium's CP437 mapping, which zip 8 used for flag-clear
/// names; zip 9 keeps its own `cp437` module private, so the table lives
/// beside the only users). Bytes below 0x80 map to themselves.
const CP437_HIGH: [char; 128] = [
    '\u{00C7}', '\u{00FC}', '\u{00E9}', '\u{00E2}', '\u{00E4}', '\u{00E0}', '\u{00E5}', '\u{00E7}',
    '\u{00EA}', '\u{00EB}', '\u{00E8}', '\u{00EF}', '\u{00EE}', '\u{00EC}', '\u{00C4}', '\u{00C5}',
    '\u{00C9}', '\u{00E6}', '\u{00C6}', '\u{00F4}', '\u{00F6}', '\u{00F2}', '\u{00FB}', '\u{00F9}',
    '\u{00FF}', '\u{00D6}', '\u{00DC}', '\u{00A2}', '\u{00A3}', '\u{00A5}', '\u{20A7}', '\u{0192}',
    '\u{00E1}', '\u{00ED}', '\u{00F3}', '\u{00FA}', '\u{00F1}', '\u{00D1}', '\u{00AA}', '\u{00BA}',
    '\u{00BF}', '\u{2310}', '\u{00AC}', '\u{00BD}', '\u{00BC}', '\u{00A1}', '\u{00AB}', '\u{00BB}',
    '\u{2591}', '\u{2592}', '\u{2593}', '\u{2502}', '\u{2524}', '\u{2561}', '\u{2562}', '\u{2556}',
    '\u{2555}', '\u{2563}', '\u{2551}', '\u{2557}', '\u{255D}', '\u{255C}', '\u{255B}', '\u{2510}',
    '\u{2514}', '\u{2534}', '\u{252C}', '\u{251C}', '\u{2500}', '\u{253C}', '\u{255E}', '\u{255F}',
    '\u{255A}', '\u{2554}', '\u{2569}', '\u{2566}', '\u{2560}', '\u{2550}', '\u{256C}', '\u{2567}',
    '\u{2568}', '\u{2564}', '\u{2565}', '\u{2559}', '\u{2558}', '\u{2552}', '\u{2553}', '\u{256B}',
    '\u{256A}', '\u{2518}', '\u{250C}', '\u{2588}', '\u{2584}', '\u{258C}', '\u{2590}', '\u{2580}',
    '\u{03B1}', '\u{00DF}', '\u{0393}', '\u{03C0}', '\u{03A3}', '\u{03C3}', '\u{00B5}', '\u{03C4}',
    '\u{03A6}', '\u{0398}', '\u{03A9}', '\u{03B4}', '\u{221E}', '\u{03C6}', '\u{03B5}', '\u{2229}',
    '\u{2261}', '\u{00B1}', '\u{2265}', '\u{2264}', '\u{2320}', '\u{2321}', '\u{00F7}', '\u{2248}',
    '\u{00B0}', '\u{2219}', '\u{00B7}', '\u{221A}', '\u{207F}', '\u{00B2}', '\u{25A0}', '\u{00A0}',
];

/// A name decoded as IBM code page 437, as zip 8 decoded flag-clear names:
/// every byte maps (the code page is total), so decoding cannot fail.
fn cp437_name(bytes: &[u8]) -> String {
    if bytes.is_ascii() {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    bytes
        .iter()
        .map(|&byte| {
            if byte < 0x80 {
                char::from(byte)
            } else {
                // byte >= 0x80 holds here; the checked form only satisfies
                // the module's no-panics policy.
                byte.checked_sub(0x80)
                    .and_then(|high| CP437_HIGH.get(usize::from(high)).copied())
                    .unwrap_or(char::REPLACEMENT_CHARACTER)
            }
        })
        .collect()
}

/// The Info-ZIP Unicode Path (0x7075) extra fields of an entry, in file order.
pub(crate) fn unicode_path_fields(
    fields: impl Iterator<Item = zip::ExtraField>,
) -> impl Iterator<Item = zip::extra_fields::UnicodeExtraField> {
    fields.filter_map(|field| match field {
        zip::ExtraField::UnicodePath(unicode) => Some(unicode),
        _ => None,
    })
}

/// CRC-32 (IEEE, reflected), as the ZIP format and the Info-ZIP Unicode
/// Path extra field checksum use it.
pub(crate) fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let bit = crc & 1;
            crc >>= 1;
            if bit != 0 {
                crc ^= 0xEDB8_8320;
            }
        }
    }
    !crc
}

/// The same name [`zip_entry_name`] resolves, decoded from a raw central
/// record's fields — the header name bytes, the general-purpose flags and
/// the extra-field block — where no [`zip::read::ZipFile`] exists to ask
/// (the pre-index identity walk of ZIP-15/#600). zip 9 adopts each 0x7075
/// field whose CRC32 matches the ORIGINAL header name and whose content is
/// UTF-8, setting bit 11 (`zip-9.0.5/src/extra_fields/extra_field.rs`
/// 291-300); [`zip_entry_name`] then refuses a field whose CRC matches the
/// adopted name but whose content differs — the shape a non-UTF-8 adopted
/// name produces — and the same refusal is applied here.
pub(crate) fn zip_entry_name_raw(
    raw: &[u8],
    flags: u16,
    extra: &[u8],
) -> Result<String, zip::result::ZipError> {
    // (crc32, content) of each Info-ZIP Unicode Path (0x7075) field:
    // version byte, CRC32 of the header name, UTF-8 content.
    let mut fields: Vec<(u32, &[u8])> = Vec::new();
    let mut rest = extra;
    while let Some((header, head_tail)) = rest.split_at_checked(4) {
        rest = head_tail;
        let tag = u16::from_le_bytes(header.first_chunk::<2>().copied().unwrap_or_default());
        let len = usize::from(u16::from_le_bytes(
            header.last_chunk::<2>().copied().unwrap_or_default(),
        ));
        let Some((body, tail)) = rest.split_at_checked(len) else {
            break;
        };
        rest = tail;
        if tag == 0x7075 && body.len() >= 5 {
            fields.push((
                u32::from_le_bytes(
                    body.get(1..5)
                        .and_then(|b| <[u8; 4]>::try_from(b).ok())
                        .unwrap_or_default(),
                ),
                body.get(5..).unwrap_or_default(),
            ));
        }
    }
    let original_crc = crc32(raw);
    let mut name: Vec<u8> = raw.to_vec();
    let mut utf8 = flags & 0x0800 != 0;
    for &(crc, content) in &fields {
        if crc == original_crc && std::str::from_utf8(content).is_ok() {
            name = content.to_vec();
            utf8 = true;
        }
    }
    for &(crc, content) in &fields {
        if crc == crc32(&name) && content != name.as_slice() {
            return Err(zip::result::ZipError::InvalidArchive(
                "Info-ZIP Unicode Path extra field content is not UTF-8".into(),
            ));
        }
    }
    Ok(if utf8 {
        String::from_utf8_lossy(&name).into_owned()
    } else {
        cp437_name(&name)
    })
}

/// An entry name decoded exactly as the zip 8 reader did, from the raw name
/// bytes, the general-purpose flag field and the Info-ZIP Unicode Path
/// (0x7075) extra fields: bit 11 set means UTF-8 (lossy), clear means CP437,
/// and a Unicode Path field whose CRC32 matches the header name replaces it
/// (`zip-8.6.0/src/read.rs` 523-529 and 705-709). zip 9's `ZipFile::name`
/// instead tries UTF-8 first whatever the flag says
/// (`zip-9.0.4/src/types.rs` 122-130), so [`admit`] and
/// [`crate::opc::PartFs::open`] both decode here and keep one name per entry
/// (#524: no read/write regressions).
///
/// # Errors
///
/// [`zip::result::ZipError`] when a Unicode Path field's CRC matches the
/// visible name but its content is not UTF-8 — the archive zip 8 refused to
/// open (`String::from_utf8` on the adopted content) rather than read under
/// the header name.
///
/// A field matching neither the header name nor the adopted one (a plain CRC
/// mismatch, or a second field chained after an adopted one) is
/// indistinguishable from an adopted field through zip 9's public surface
/// (adoption replaces `name_raw` and sets bit 11,
/// `zip-9.0.4/src/extra_fields/extra_field.rs` 276-286): such an archive is
/// read under the visible name where zip 8 refused it.
pub(crate) fn zip_entry_name(
    raw: &[u8],
    flags: u16,
    unicode_paths: impl Iterator<Item = zip::extra_fields::UnicodeExtraField>,
) -> Result<String, zip::result::ZipError> {
    let raw = raw.to_vec();
    for field in unicode_paths {
        match field.unwrap_valid(&raw) {
            // The field was adopted (the visible name is its content), or the
            // archive repeats it verbatim: nothing to decide.
            Ok(content) if content.as_ref() == raw.as_slice() => {}
            // The CRC matches the visible name and zip 9 still did not adopt
            // the field, so the content is not UTF-8: refuse as zip 8 did.
            Ok(_) => {
                return Err(zip::result::ZipError::InvalidArchive(
                    "Info-ZIP Unicode Path extra field content is not UTF-8".into(),
                ));
            }
            // The CRC matches neither the header nor the adopted name.
            Err(_) => {}
        }
    }
    // Bit 11 (0x0800) is the language-encoding flag: `ZipFlags::LanguageEncoding`
    // in zip-9.0.4/src/format/flags.rs, `is_utf8` in zip-8.6.0/src/read.rs.
    Ok(if flags & 0x0800 != 0 {
        String::from_utf8_lossy(&raw).into_owned()
    } else {
        cp437_name(&raw)
    })
}

/// A ZIP name must be a relative part name: no leading slash or drive, no
/// backslash, no empty, `.` or `..` segment.
fn check_part_name(name: &str) -> Result<(), AdmissionError> {
    let refuse = |why: &str| {
        Err(AdmissionError::new(
            AdmissionErrorKind::UnsupportedPackage,
            format!("unsafe entry name {name:?}: {why}"),
        ))
    };
    if name.is_empty() {
        return refuse("empty");
    }
    if name.starts_with('/') || name.as_bytes().get(1) == Some(&b':') {
        return refuse("absolute path");
    }
    if name.contains('\\') || name.contains('\0') {
        return refuse("backslash or NUL");
    }
    let segments = name.strip_suffix('/').unwrap_or(name);
    if segments
        .split('/')
        .any(|s| s.is_empty() || s == "." || s == "..")
    {
        return refuse("empty, . or .. segment");
    }
    Ok(())
}

/// A part's text for scanning. Word writes UTF-8, but a part may be UTF-16
/// with a byte-order mark (SharePoint's `customXml` items often are) and
/// Word opens it, so it is transcoded; other bytes decode lossily, as the
/// rest of the engine reads them.
fn part_text(xml: &[u8]) -> Cow<'_, str> {
    let utf16 = |rest: &[u8], unit: fn([u8; 2]) -> u16| {
        let units = rest.as_chunks::<2>().0.iter().map(|&pair| unit(pair));
        Cow::Owned(
            char::decode_utf16(units)
                .map(|c| c.unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect(),
        )
    };
    match xml {
        [0xFF, 0xFE, rest @ ..] => utf16(rest, u16::from_le_bytes),
        [0xFE, 0xFF, rest @ ..] => utf16(rest, u16::from_be_bytes),
        _ => String::from_utf8_lossy(xml),
    }
}

/// Well-formed enough to scan, no deeper than `max_depth` elements, and
/// inside the tag/namespace policy (`xmllinq::parse::TagPolicy`) so no part
/// reaching the permissive DOM parser can carry pathological attribute or
/// namespace-declaration widths.
fn check_xml_depth(name: &str, xml: &[u8], limits: InputLimits) -> Result<(), AdmissionError> {
    let invalid = |why: String| {
        AdmissionError::new(
            AdmissionErrorKind::InvalidXml,
            format!("{}: {why}", shown(name)),
        )
    };
    let text = part_text(xml);
    let mut reader = Reader::from_str(&text);
    reader.config_mut().check_end_names = true;
    let max_depth = limits.max_xml_depth;
    let mut policy = crate::xmllinq::parse::TagPolicy::tolerant(limits);
    let mut depth = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(tag)) => {
                policy.open(&tag).map_err(invalid)?;
                depth = depth.saturating_add(1);
                if depth > max_depth {
                    return Err(invalid(format!("XML nests deeper than {max_depth}")));
                }
            }
            Ok(Event::Empty(tag)) => {
                policy.empty(&tag).map_err(invalid)?;
            }
            Ok(Event::End(_)) => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| invalid("end tag with no open element".to_string()))?;
                policy.close();
            }
            Ok(Event::Eof) if depth > 0 => {
                return Err(invalid(format!("{depth} element(s) left unclosed")));
            }
            Ok(Event::Eof) => return Ok(()),
            // Word refuses every part with a DOCTYPE (#2015); entity bombs
            // and external entities live in its internal subset.
            Ok(Event::DocType(_)) => {
                return Err(invalid("DTD declarations are unsupported".to_string()));
            }
            Ok(_) => {}
            Err(e) => return Err(invalid(e.to_string())),
        }
    }
}

/// The main document part from the package relationships (falling back to
/// `word/document.xml`, as the reader does), checked against its declared
/// content type. `names` holds every entry's decoded name: OPC part names
/// ignore ASCII case (M1.12), so a rels target spelled differently than
/// the stored entry still names it, and the stored spelling is what the
/// main part reports (ZIP-15/#600, Word's p3 probe).
fn main_part(names: &[String], kept: &BTreeMap<String, Vec<u8>>) -> Result<String, AdmissionError> {
    use AdmissionErrorKind as K;
    let types = kept
        .get("[content_types].xml")
        .ok_or_else(|| AdmissionError::new(K::UnsupportedPackage, "no [Content_Types].xml"))?;
    let from_rels = kept.get("_rels/.rels").and_then(|rels| {
        elements(rels, "Relationship")
            .into_iter()
            .find_map(|attrs| {
                let is_main = attrs
                    .get("Type")
                    .is_some_and(|t| OFFICE_DOCUMENT_REL.contains(&t.as_str()));
                let external = attrs.get("TargetMode").is_some_and(|m| m == "External");
                (is_main && !external)
                    .then(|| {
                        attrs
                            .get("Target")
                            .map(|t| t.trim_start_matches('/').to_string())
                    })
                    .flatten()
            })
    });
    let main = from_rels.unwrap_or_else(|| "word/document.xml".to_string());
    let Some(main) = names
        .iter()
        .find(|name| name.eq_ignore_ascii_case(&main))
        .cloned()
    else {
        return Err(AdmissionError::new(
            K::UnsupportedPackage,
            format!("main document part {main} is missing"),
        ));
    };
    if let Some(content_type) = content_type_of(types, &main)
        && !MAIN_CONTENT_TYPES.contains(&content_type.as_str())
    {
        return Err(AdmissionError::new(
            K::UnsupportedPackage,
            format!("{main} is {content_type}, not a Word document"),
        ));
    }
    Ok(main)
}

/// The content type `[Content_Types].xml` declares for `part`: its Override,
/// else the Default for its extension.
fn content_type_of(types: &[u8], part: &str) -> Option<String> {
    let wanted = format!("/{part}");
    let overridden = elements(types, "Override").into_iter().find_map(|attrs| {
        attrs
            .get("PartName")
            .filter(|p| p.eq_ignore_ascii_case(&wanted))
            .and_then(|_| attrs.get("ContentType").cloned())
    });
    overridden.or_else(|| {
        let ext = part.rsplit_once('.')?.1;
        elements(types, "Default").into_iter().find_map(|attrs| {
            attrs
                .get("Extension")
                .filter(|e| e.eq_ignore_ascii_case(ext))
                .and_then(|_| attrs.get("ContentType").cloned())
        })
    })
}

/// Attributes of every element with local name `local` (namespace ignored).
fn elements(xml: &[u8], local: &str) -> Vec<BTreeMap<String, String>> {
    let text = part_text(xml);
    let mut reader = Reader::from_str(&text);
    let mut found = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(e) | Event::Empty(e)) if e.local_name().into_inner() == local => {
                let attrs = e
                    .attributes()
                    .flatten()
                    .filter_map(|a| {
                        let key = a.key.local_name().into_inner().to_string();
                        let value = a
                            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                            .ok()?
                            .into_owned();
                        Some((key, value))
                    })
                    .collect();
                found.push(attrs);
            }
            Ok(Event::Eof) | Err(_) => return found,
            Ok(_) => {}
        }
    }
}

/// ZIP fixtures whose entry names carry legacy encodings: raw name bytes
/// with a chosen language-encoding flag (bit 11), which the writer cannot
/// express (it always writes UTF-8 names and sets the flag for non-ASCII
/// ones), plus Info-ZIP Unicode Path extra field (0x7075) payloads. Shared by
/// the admission and [`crate::opc`] tests so both readers are probed with the
/// same archives.
#[cfg(test)]
pub(crate) mod zip_names {
    use std::io::{Cursor, Write};

    use zip::write::FullFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    /// One ZIP entry with a legacy-encoded name.
    #[derive(Clone)]
    pub(crate) struct RawEntry<'a> {
        /// The exact header name bytes.
        pub name: &'a [u8],
        pub data: &'a [u8],
        /// Whether the general-purpose bit 11 (language encoding) is set.
        pub utf8_flag: bool,
        /// A complete 0x7075 payload (version, CRC32 of `name`, content).
        pub unicode_path: Option<Vec<u8>>,
    }

    impl<'a> RawEntry<'a> {
        /// An entry with the given raw name, bit 11 clear, no extra field.
        pub const fn raw(name: &'a [u8], data: &'a [u8]) -> Self {
            Self {
                name,
                data,
                utf8_flag: false,
                unicode_path: None,
            }
        }

        /// Set bit 11 (the name is UTF-8, whatever the bytes say).
        pub const fn utf8(mut self) -> Self {
            self.utf8_flag = true;
            self
        }

        /// Add a 0x7075 field overriding `name` with `content`.
        pub fn unicode_path(mut self, content: &[u8]) -> Self {
            self.unicode_path = Some(unicode_path_field(self.name, content));
            self
        }
    }

    /// The 0x7075 payload for a field that replaces `name` with `content`.
    pub fn unicode_path_field(name: &[u8], content: &[u8]) -> Vec<u8> {
        let mut field = vec![1];
        field.extend_from_slice(&crc32(name).to_le_bytes());
        field.extend_from_slice(content);
        field
    }

    /// A minimal package — `[Content_Types].xml`, `_rels/.rels`,
    /// `word/document.xml` and `word/_rels/document.xml.rels` holding `rels`
    /// — plus `extra` entries. `admit` and `PartFs::open` are probed with the
    /// same bytes.
    pub fn docx_package(rels: &str, extra: &[RawEntry<'_>]) -> Vec<u8> {
        let mut entries = vec![
            RawEntry::raw(
                b"[Content_Types].xml",
                br#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Default Extension="png" ContentType="image/png"/><Default Extension="bin" ContentType="application/octet-stream"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
            ),
            RawEntry::raw(
                b"_rels/.rels",
                br#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
            ),
            RawEntry::raw(b"word/document.xml", b"<w:document/>"),
        ];
        let rels_xml = format!(
            "<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">{rels}</Relationships>"
        );
        entries.push(RawEntry::raw(
            b"word/_rels/document.xml.rels",
            rels_xml.as_bytes(),
        ));
        entries.extend(extra.iter().cloned());
        zip(&entries)
    }

    /// CRC-32 (IEEE, reflected) as the ZIP format and zip's Unicode extra
    /// field checksum use.
    pub fn crc32(bytes: &[u8]) -> u32 {
        let mut crc = !0u32;
        for &byte in bytes {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                let bit = crc & 1;
                crc >>= 1;
                if bit != 0 {
                    crc ^= 0xEDB8_8320;
                }
            }
        }
        !crc
    }

    /// A Stored-method ZIP of `entries`. The writer emits a unique ASCII
    /// placeholder of each name's byte length with bit 11 clear; the
    /// placeholder bytes are then replaced in the local and central headers
    /// (both store the name verbatim), and bit 11 is ORed into both flag
    /// fields. A Stored entry's payload is never compressed, so it cannot
    /// contain a placeholder unless the caller writes one.
    pub fn zip(entries: &[RawEntry<'_>]) -> Vec<u8> {
        let placeholders: Vec<String> = entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let digits = index.to_string();
                assert!(
                    digits.len() <= entry.name.len(),
                    "name {:?} is too short for a unique placeholder",
                    entry.name
                );
                format!("{digits}{}", "x".repeat(entry.name.len() - digits.len()))
            })
            .collect();
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options: FullFileOptions<'_, '_> =
            zip::write::FileOptions::default().compression_method(CompressionMethod::Stored);
        for (entry, placeholder) in entries.iter().zip(&placeholders) {
            assert!(
                !entry
                    .data
                    .windows(placeholder.len())
                    .any(|w| w == placeholder.as_bytes()),
                "payload contains the placeholder {placeholder}"
            );
            let mut options = options.clone();
            if let Some(field) = &entry.unicode_path {
                // Both headers: a central-only field is legal but makes the
                // local and central extra data disagree, which header
                // consistency refuses.
                options
                    .add_extra_field(0x7075, field.as_slice(), true)
                    .expect("write 0x7075 extra field");
            }
            writer
                .start_file(placeholder.as_str(), options)
                .expect("start");
            writer.write_all(entry.data).expect("write");
        }
        let mut bytes = writer.finish().expect("finish").into_inner();
        for (entry, placeholder) in entries.iter().zip(&placeholders) {
            replace_all(&mut bytes, placeholder.as_bytes(), entry.name);
        }
        set_utf8_flags(&mut bytes, entries);
        bytes
    }

    fn replace_all(bytes: &mut Vec<u8>, from: &[u8], to: &[u8]) {
        let mut at = 0;
        while let Some(found) = bytes[at..]
            .windows(from.len())
            .position(|window| window == from)
        {
            let start = at + found;
            let end = start + from.len();
            bytes.splice(start..end, to.iter().copied());
            at = end;
        }
    }

    /// OR bit 11 into the flag field of every header whose name is an entry
    /// with `utf8_flag` set: local headers (flags at +6, name at +30) and
    /// central headers (flags at +8, name at +46).
    fn set_utf8_flags(bytes: &mut [u8], entries: &[RawEntry<'_>]) {
        const LOCAL: [u8; 4] = [0x50, 0x4b, 0x03, 0x04];
        const CENTRAL: [u8; 4] = [0x50, 0x4b, 0x01, 0x02];
        let mut at = 0;
        while bytes.get(at..).is_some_and(|rest| rest.len() >= 4) {
            let local = bytes[at..].starts_with(&LOCAL);
            let central = bytes[at..].starts_with(&CENTRAL);
            if local || central {
                let (flags, name, len) = if local {
                    (at + 6, at + 30, at + 26)
                } else {
                    (at + 8, at + 46, at + 28)
                };
                let name_len = bytes
                    .get(len..len + 2)
                    .map(|field| u16::from_le_bytes([field[0], field[1]]) as usize)
                    .unwrap_or(0);
                if bytes.get(name..name + name_len).is_some_and(|header_name| {
                    entries
                        .iter()
                        .any(|entry| entry.utf8_flag && header_name == entry.name)
                }) && let Some(field) = bytes.get_mut(flags..flags + 2)
                {
                    let flags = u16::from_le_bytes([field[0], field[1]]) | 0x0800;
                    field.copy_from_slice(&flags.to_le_bytes());
                }
                at += 4;
            } else {
                at += 1;
            }
        }
    }
    /// A local file header (`PK\3\4`) for `name`, as a record in a
    /// hand-built package reads: version 20, `flags`, stored method, no
    /// time, and the given CRC-32, sizes and extra bytes.
    pub fn local_header(
        name: &[u8],
        flags: u16,
        crc32: u32,
        compressed_size: u32,
        uncompressed_size: u32,
        extra: &[u8],
    ) -> Vec<u8> {
        let mut header = vec![0x50, 0x4b, 0x03, 0x04];
        header.extend_from_slice(&20u16.to_le_bytes());
        header.extend_from_slice(&flags.to_le_bytes());
        header.extend_from_slice(&0u16.to_le_bytes());
        header.extend_from_slice(&[0; 4]);
        header.extend_from_slice(&crc32.to_le_bytes());
        header.extend_from_slice(&compressed_size.to_le_bytes());
        header.extend_from_slice(&uncompressed_size.to_le_bytes());
        header.extend_from_slice(&u16::try_from(name.len()).unwrap().to_le_bytes());
        header.extend_from_slice(&u16::try_from(extra.len()).unwrap().to_le_bytes());
        header.extend_from_slice(name);
        header.extend_from_slice(extra);
        header
    }

    /// A ZIP64 extended information extra field (0x0001) carrying both
    /// sizes, the shape a local header always writes.
    pub fn zip64_extra(uncompressed_size: u64, compressed_size: u64) -> Vec<u8> {
        let mut field = vec![0x01, 0x00, 16, 0];
        field.extend_from_slice(&uncompressed_size.to_le_bytes());
        field.extend_from_slice(&compressed_size.to_le_bytes());
        field
    }

    /// The shape of a data descriptor (APPNOTE 4.3.9): signed with
    /// `PK\7\8` or bare — both are legal. APPNOTE 4.3.9.2 makes the
    /// sizes 8 bytes when the local header carries a 0x0001 ZIP64
    /// field, whatever the entry's size, so width is the variant's,
    /// not the entry's.
    #[derive(Clone, Copy)]
    pub enum Descriptor {
        /// `PK\7\8` then CRC-32 and the two 4-byte sizes.
        Signed,
        /// CRC-32 and the two 4-byte sizes, no signature.
        Unsigned,
        /// `PK\7\8` then CRC-32 and the two 8-byte sizes.
        SignedZip64,
        /// CRC-32 and the two 8-byte sizes, no signature.
        UnsignedZip64,
    }

    /// An entry for [`assembled`]: raw name bytes, payload, an optional
    /// trailing data descriptor (which sets bit 3 in both headers and
    /// writes zeros — ZIP64 placeholders plus the extra field when
    /// `zip64` — for the local CRC and sizes), and whether the record is
    /// ZIP64-formed.
    pub struct AssembledEntry<'a> {
        /// The exact header name bytes.
        pub name: &'a [u8],
        /// The stored payload.
        pub data: Vec<u8>,
        /// General-purpose bits ORed into both headers.
        pub flags: u16,
        /// The descriptor trailing the payload, if any.
        pub descriptor: Option<Descriptor>,
        /// ZIP64 headers: 0xFFFFFFFF sizes plus the 0x0001 extra field.
        pub zip64: bool,
        /// Local extra-field bytes; `zip64` writes the 0x0001 field instead.
        pub local_extra: &'a [u8],
    }

    impl<'a> AssembledEntry<'a> {
        /// A plain stored entry.
        pub fn plain(name: &'a [u8], data: &[u8]) -> Self {
            Self {
                name,
                data: data.to_vec(),
                flags: 0,
                descriptor: None,
                zip64: false,
                local_extra: b"",
            }
        }

        /// Trail the payload with a data descriptor of the given shape.
        pub fn with_descriptor(mut self, descriptor: Descriptor) -> Self {
            self.descriptor = Some(descriptor);
            self
        }

        /// Force the entry's headers into ZIP64 form.
        pub const fn zip64(mut self) -> Self {
            self.zip64 = true;
            self
        }
    }

    /// A package assembled record by record: `prefix`, each entry's
    /// local header, payload and descriptor in order, the central
    /// records — in `central_order` indices, entry order when `None` —
    /// and the EOCD with `comment`. The shapes no stock writer
    /// expresses: bit-3 descriptors, forced ZIP64 records and a central
    /// order different from the physical one. With a `prefix`, offsets
    /// are relative to the archive start, as prepended-data packages
    /// carry them (APPNOTE 4.3.6's self-extracting stub).
    pub fn assembled(
        prefix: &[u8],
        entries: &[AssembledEntry<'_>],
        central_order: Option<&[usize]>,
        comment: &[u8],
    ) -> Vec<u8> {
        let base = u64::try_from(prefix.len()).unwrap();
        let mut out = prefix.to_vec();
        let mut offsets = Vec::with_capacity(entries.len());
        for entry in entries {
            let crc = crc32(&entry.data);
            let size = u64::try_from(entry.data.len()).unwrap();
            offsets.push(u64::try_from(out.len()).unwrap() - base);
            let flags = entry.flags
                | if entry.descriptor.is_some() {
                    0x0008
                } else {
                    0
                };
            let (local_crc, local_sizes, extra) = if entry.zip64 {
                let local = if entry.descriptor.is_some() { 0 } else { crc };
                (local, (u32::MAX, u32::MAX), zip64_extra(size, size))
            } else if entry.descriptor.is_some() {
                (0, (0, 0), entry.local_extra.to_vec())
            } else {
                (
                    crc,
                    (u32::try_from(size).unwrap(), u32::try_from(size).unwrap()),
                    entry.local_extra.to_vec(),
                )
            };
            out.extend_from_slice(&local_header(
                entry.name,
                flags,
                local_crc,
                local_sizes.0,
                local_sizes.1,
                &extra,
            ));
            out.extend_from_slice(&entry.data);
            if let Some(shape) = entry.descriptor {
                if matches!(shape, Descriptor::Signed | Descriptor::SignedZip64) {
                    out.extend_from_slice(&[0x50, 0x4b, 0x07, 0x08]);
                }
                out.extend_from_slice(&crc.to_le_bytes());
                if matches!(shape, Descriptor::SignedZip64 | Descriptor::UnsignedZip64) {
                    out.extend_from_slice(&size.to_le_bytes());
                    out.extend_from_slice(&size.to_le_bytes());
                } else {
                    out.extend_from_slice(&u32::try_from(size).unwrap().to_le_bytes());
                    out.extend_from_slice(&u32::try_from(size).unwrap().to_le_bytes());
                }
            }
        }
        let central_at = u64::try_from(out.len()).unwrap() - base;
        let order: Vec<usize> = central_order
            .map(<[_]>::to_vec)
            .unwrap_or_else(|| (0..entries.len()).collect());
        for &index in &order {
            let entry = &entries[index];
            let crc = crc32(&entry.data);
            let size = u64::try_from(entry.data.len()).unwrap();
            let flags = entry.flags
                | if entry.descriptor.is_some() {
                    0x0008
                } else {
                    0
                };
            let (compressed, uncompressed, extra) = if entry.zip64 {
                (u32::MAX, u32::MAX, zip64_extra(size, size))
            } else {
                (
                    u32::try_from(size).unwrap(),
                    u32::try_from(size).unwrap(),
                    Vec::new(),
                )
            };
            out.extend_from_slice(&[0x50, 0x4b, 0x01, 0x02]);
            out.extend_from_slice(&45u16.to_le_bytes());
            out.extend_from_slice(&45u16.to_le_bytes());
            out.extend_from_slice(&flags.to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&[0; 4]);
            out.extend_from_slice(&crc.to_le_bytes());
            out.extend_from_slice(&compressed.to_le_bytes());
            out.extend_from_slice(&uncompressed.to_le_bytes());
            out.extend_from_slice(&u16::try_from(entry.name.len()).unwrap().to_le_bytes());
            out.extend_from_slice(&u16::try_from(extra.len()).unwrap().to_le_bytes());
            out.extend_from_slice(&0u16.to_le_bytes());
            out.extend_from_slice(&[0; 8]);
            out.extend_from_slice(&u32::try_from(offsets[index]).unwrap().to_le_bytes());
            out.extend_from_slice(entry.name);
            out.extend_from_slice(&extra);
        }
        let central_size = u64::try_from(out.len()).unwrap() - base - central_at;
        out.extend_from_slice(&[0x50, 0x4b, 0x05, 0x06]);
        out.extend_from_slice(&[0; 4]);
        let count = u16::try_from(entries.len()).unwrap();
        out.extend_from_slice(&count.to_le_bytes());
        out.extend_from_slice(&count.to_le_bytes());
        out.extend_from_slice(&u32::try_from(central_size).unwrap().to_le_bytes());
        out.extend_from_slice(&u32::try_from(central_at).unwrap().to_le_bytes());
        out.extend_from_slice(&u16::try_from(comment.len()).unwrap().to_le_bytes());
        out.extend_from_slice(comment);
        out
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    /// A name prints with its control characters escaped and nothing else
    /// changed: quotes and non-ASCII stay as the archive spells them.
    #[test]
    fn a_shown_name_escapes_control_characters_only() {
        assert_eq!(shown("word/it's \"é\".xml"), "word/it's \"é\".xml");
        assert_eq!(shown("a\u{1b}[31mb\n"), "a\\u{1b}[31mb\\n");
    }

    /// A limit message names the part as the archive spells it: an
    /// apostrophe is not escaped (review of gid=94's fix).
    #[test]
    fn a_limit_message_keeps_an_ordinary_part_name_verbatim() {
        let error = part_declared_over(
            "word/it's.xml",
            10,
            &InputLimits {
                max_part_bytes: 1,
                ..InputLimits::default()
            },
        );
        let text = error.to_string();
        assert!(text.contains("word/it's.xml declares"), "{text}");
    }

    use std::io::Write;

    use zip::write::SimpleFileOptions;

    use super::*;

    const TYPES: &str = r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    const RELS: &str = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    const DOC: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;

    fn zip_of(entries: &[(&str, &[u8])], method: CompressionMethod) -> Vec<u8> {
        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(method);
        for (name, data) in entries {
            out.start_file(*name, options).unwrap();
            out.write_all(data).unwrap();
        }
        out.finish().unwrap().into_inner()
    }

    fn docx_with(extra: &[(&str, &[u8])]) -> Vec<u8> {
        let mut entries: Vec<(&str, &[u8])> = vec![
            ("[Content_Types].xml", TYPES.as_bytes()),
            ("_rels/.rels", RELS.as_bytes()),
            ("word/document.xml", DOC.as_bytes()),
        ];
        entries.extend_from_slice(extra);
        zip_of(&entries, CompressionMethod::Deflated)
    }

    fn kind(bytes: &[u8], limits: InputLimits) -> AdmissionErrorKind {
        admit(bytes, limits).expect_err("refused").kind
    }

    /// `admit` and `PartFs::open` decode entry names with the same rule
    /// (bit 11 / CP437 / the 0x7075 override) and refuse the same decoded
    /// duplicates, so a package one reader accepts the other accepts with the
    /// same part names (#524, #1069 zip9 review rows 1-2).
    #[test]
    fn admission_and_partfs_agree_on_decoded_names_and_duplicates() {
        // Row 1: two raw names, bit 11 set, whose lossy UTF-8 decode collides —
        // one part, refused by both readers.
        let aliased = zip_names::docx_package(
            "",
            &[
                zip_names::RawEntry::raw(b"x\x82.bin", b"FIRST").utf8(),
                zip_names::RawEntry::raw(b"x\x83.bin", b"SECOND").utf8(),
            ],
        );
        assert_eq!(
            kind(&aliased, InputLimits::default()).code(),
            "DUPLICATE_PART"
        );
        assert!(
            matches!(
                crate::opc::PartFs::open(&aliased),
                Err(crate::opc::OpcError::DuplicatePartName(ref n)) if n == "x\u{FFFD}.bin"
            ),
            "the constructor must refuse the same aliases with the typed variant"
        );
        // Two entries of the same name, byte for byte: the same pair of
        // refusals (the p1 shape).
        let exact = zip_names::docx_package(
            "",
            &[
                zip_names::RawEntry::raw(b"x.bin", b"FIRST"),
                zip_names::RawEntry::raw(b"x.bin", b"SECOND"),
            ],
        );
        assert_eq!(
            kind(&exact, InputLimits::default()).code(),
            "DUPLICATE_PART"
        );
        assert!(
            matches!(
                crate::opc::PartFs::open(&exact),
                Err(crate::opc::OpcError::DuplicatePartName(ref n)) if n == "x.bin"
            ),
            "the constructor must refuse the byte-equal duplicate as well"
        );

        // Row 2: a flag-clear CP437 media name and the relationship that
        // names its CP437 spelling — both readers admit the package under the
        // same names, and the media resolves.
        let cp437 = zip_names::docx_package(
            "<Relationship Id=\"rId1\" Type=\"image\" Target=\"media/\u{251C}\u{2310}.png\"/>",
            &[zip_names::RawEntry::raw(
                b"word/media/\xc3\xa9.png",
                b"PNGPAYLOAD",
            )],
        );
        let admitted = admit(&cp437, InputLimits::default()).unwrap();
        assert_eq!(admitted.main_part, "word/document.xml");
        let fs = crate::opc::PartFs::open(&cp437).unwrap();
        assert_eq!(
            fs.main_document_part().as_deref(),
            Some("word/document.xml")
        );
        assert_eq!(
            fs.part_bytes("word/media/\u{251C}\u{2310}.png"),
            Some(b"PNGPAYLOAD".as_slice())
        );
    }

    /// Change only a local name in the synthetic writer's package.
    fn rename_local_header(bytes: &mut [u8], from: &[u8], to: &[u8]) {
        assert_eq!(from.len(), to.len());
        let at = {
            let mut archive = zip::ZipArchive::new(Cursor::new(&*bytes)).unwrap();
            archive
                .by_name(std::str::from_utf8(from).unwrap())
                .unwrap()
                .header_start() as usize
        };
        assert_eq!(&bytes[at..at + 4], b"PK\x03\x04");
        assert_eq!(&bytes[at + 30..at + 30 + from.len()], from);
        bytes[at + 30..at + 30 + from.len()].copy_from_slice(to);
    }

    /// GHSA-m665-v7pm-7grj (zip <= 9.0.4): a local header may name an entry
    /// differently than the central directory, so a scanner trusting local
    /// headers reviews a different package than the one readers open.
    /// Admission refuses the disagreement with `ZIP_HEADER_MISMATCH`.
    #[test]
    fn a_dual_header_package_is_refused() {
        let mut bytes = zip_names::docx_package("", &[]);
        rename_local_header(&mut bytes, b"word/document.xml", b"w0rd/document.xml");
        let err = admit(&bytes, InputLimits::default()).expect_err("refused");
        assert_eq!(err.kind.code(), "ZIP_HEADER_MISMATCH");
        assert_eq!(err.code(), "ZIP_HEADER_MISMATCH");
        assert!(err.message.contains("headers disagree"), "{}", err.message);
    }

    /// Overwrite the 2-byte field at `local_off` / `central_off` in every
    /// local and central header.
    fn patch_u16(bytes: &mut [u8], local_off: usize, central_off: usize, value: u16) {
        patch(bytes, local_off, central_off, &value.to_le_bytes());
    }

    fn patch(bytes: &mut [u8], local_off: usize, central_off: usize, value: &[u8]) {
        let mut i = 0;
        while i + 4 <= bytes.len() {
            let off = match &bytes[i..i + 4] {
                [0x50, 0x4b, 0x03, 0x04] => Some(local_off),
                [0x50, 0x4b, 0x01, 0x02] => Some(central_off),
                _ => None,
            };
            if let Some(off) = off {
                bytes[i + off..i + off + value.len()].copy_from_slice(value);
            }
            i += 1;
        }
    }

    #[test]
    fn a_plain_docx_is_admitted() {
        let bytes = docx_with(&[]);
        let admitted = admit(&bytes, InputLimits::default()).unwrap();
        assert_eq!(admitted.main_part, "word/document.xml");
        assert_eq!(admitted.entries, 3);
        assert_eq!(
            admitted.inflated_bytes,
            (TYPES.len() + RELS.len() + DOC.len()) as u64
        );
    }

    #[test]
    fn budgets_admit_their_exact_boundary() {
        let bytes = docx_with(&[]);
        let total = (TYPES.len() + RELS.len() + DOC.len()) as u64;
        let largest = TYPES.len().max(RELS.len()).max(DOC.len()) as u64;
        let exact = InputLimits {
            max_compressed_bytes: bytes.len() as u64,
            max_entries: 3,
            max_part_bytes: largest,
            max_uncompressed_bytes: total,
            max_xml_depth: 5,
            ..InputLimits::default()
        };
        assert!(admit(&bytes, exact).is_ok());
        for tighter in [
            InputLimits {
                max_compressed_bytes: bytes.len() as u64 - 1,
                ..exact
            },
            InputLimits {
                max_entries: 2,
                ..exact
            },
            InputLimits {
                max_part_bytes: largest - 1,
                ..exact
            },
            InputLimits {
                max_uncompressed_bytes: total - 1,
                ..exact
            },
        ] {
            assert_eq!(kind(&bytes, tighter), AdmissionErrorKind::InputLimit);
        }
        let shallow = InputLimits {
            max_xml_depth: 4,
            ..exact
        };
        assert_eq!(kind(&bytes, shallow), AdmissionErrorKind::InvalidXml);
    }

    #[test]
    fn a_highly_compressed_part_stops_at_the_budget() {
        // 32 MiB of zeros deflate to ~32 KiB; written in chunks, never held.
        let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, data) in [
            ("[Content_Types].xml", TYPES),
            ("_rels/.rels", RELS),
            ("word/document.xml", DOC),
        ] {
            out.start_file(name, options).unwrap();
            out.write_all(data.as_bytes()).unwrap();
        }
        out.start_file("word/media/zeros.bin", options).unwrap();
        let chunk = vec![0u8; 1 << 20];
        for _ in 0..32 {
            out.write_all(&chunk).unwrap();
        }
        let bytes = out.finish().unwrap().into_inner();
        assert!(bytes.len() < 1 << 20);
        let tiny = InputLimits {
            max_part_bytes: 1 << 20,
            ..InputLimits::default()
        };
        let refused = admit(&bytes, tiny).unwrap_err();
        assert_eq!(refused.kind, AdmissionErrorKind::InputLimit);
        assert!(refused.message.contains("zeros.bin"), "{refused}");
        let package = InputLimits {
            max_uncompressed_bytes: 1 << 20,
            ..InputLimits::default()
        };
        assert_eq!(kind(&bytes, package), AdmissionErrorKind::InputLimit);
    }

    #[test]
    fn an_understated_size_is_caught_while_inflating() {
        let big = vec![b'a'; 4096];
        let mut bytes = docx_with(&[("word/media/a.bin", &big)]);
        // Declare 16 inflated bytes for every entry (local +22, central +24).
        patch(&mut bytes, 22, 24, &16u32.to_le_bytes());
        let limits = InputLimits {
            max_part_bytes: 1024,
            ..InputLimits::default()
        };
        let refused = admit(&bytes, limits).unwrap_err();
        assert!(
            matches!(
                refused.kind,
                AdmissionErrorKind::InputLimit | AdmissionErrorKind::InvalidPackage
            ),
            "{refused}"
        );
    }

    #[test]
    fn duplicate_names_are_refused() {
        // Same length names, renamed in place: the reader would keep one.
        let mut bytes = docx_with(&[("word/aaaa.xml", b"<a/>"), ("word/bbbb.xml", b"<b/>")]);
        let at: Vec<usize> = bytes
            .windows(13)
            .enumerate()
            .filter(|(_, w)| *w == b"word/bbbb.xml")
            .map(|(i, _)| i)
            .collect();
        assert_eq!(at.len(), 2, "local and central name");
        for i in at {
            bytes[i..i + 13].copy_from_slice(b"word/aaaa.xml");
        }
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::DuplicatePart
        );
        // Part names ignore case.
        let cased = docx_with(&[("Word/Document.xml", DOC.as_bytes())]);
        assert_eq!(
            kind(&cased, InputLimits::default()),
            AdmissionErrorKind::DuplicatePart
        );
    }

    #[test]
    fn unsafe_names_are_refused() {
        for name in [
            "../evil.xml",
            "word/../../evil.xml",
            "/abs.xml",
            "C:/abs.xml",
            "word\\x.xml",
            "word//x.xml",
            "word/./x.xml",
        ] {
            let bytes = docx_with(&[(name, b"<x/>")]);
            assert_eq!(
                kind(&bytes, InputLimits::default()),
                AdmissionErrorKind::UnsupportedPackage,
                "{name}"
            );
        }
        // A directory entry is fine.
        let mut out = zip::ZipWriter::new_append(Cursor::new(docx_with(&[]))).unwrap();
        out.add_directory("word/media/", SimpleFileOptions::default())
            .unwrap();
        let bytes = out.finish().unwrap().into_inner();
        assert!(admit(&bytes, InputLimits::default()).is_ok());
    }

    #[test]
    fn encryption_and_other_compression_are_refused() {
        let mut encrypted = docx_with(&[]);
        // General-purpose flag bit 0 (local +6, central +8).
        patch_u16(&mut encrypted, 6, 8, 1);
        assert_eq!(
            kind(&encrypted, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
        let mut lzma = docx_with(&[]);
        // Compression method 14 = LZMA (local +8, central +10).
        patch_u16(&mut lzma, 8, 10, 14);
        assert_eq!(
            kind(&lzma, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
        let stored = zip_of(
            &[
                ("[Content_Types].xml", TYPES.as_bytes()),
                ("_rels/.rels", RELS.as_bytes()),
                ("word/document.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Stored,
        );
        assert!(admit(&stored, InputLimits::default()).is_ok());
    }

    #[test]
    fn a_corrupt_entry_is_refused() {
        let mut bytes = zip_of(
            &[
                ("[Content_Types].xml", TYPES.as_bytes()),
                ("_rels/.rels", RELS.as_bytes()),
                ("word/document.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Stored,
        );
        let at = bytes
            .array_windows()
            .position(|w| w == b"Hi")
            .expect("stored text");
        bytes[at] = b'X';
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
        assert_eq!(
            kind(b"not a zip at all", InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
    }

    #[test]
    fn a_package_that_is_not_a_word_document_is_refused() {
        let no_main = zip_of(
            &[
                ("[Content_Types].xml", TYPES.as_bytes()),
                ("_rels/.rels", RELS.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            kind(&no_main, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
        let no_types = zip_of(
            &[
                ("_rels/.rels", RELS.as_bytes()),
                ("word/document.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            kind(&no_types, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
        // A spreadsheet's main part, named like a Word one: the content type decides.
        let sheet = TYPES.replace(
            "wordprocessingml.document.main+xml",
            "spreadsheetml.sheet.main+xml",
        );
        let xlsx = zip_of(
            &[
                ("[Content_Types].xml", sheet.as_bytes()),
                ("_rels/.rels", RELS.as_bytes()),
                ("word/document.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            kind(&xlsx, InputLimits::default()),
            AdmissionErrorKind::UnsupportedPackage
        );
    }

    #[test]
    fn a_main_part_without_an_xml_extension_is_still_depth_checked() {
        // The engine parses whatever part the relationships name as the main
        // document, whatever its extension, so the nesting budget covers it.
        let rels = RELS.replace("word/document.xml", "word/main.dat");
        let types = TYPES.replace("/word/document.xml", "/word/main.dat");
        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
        let bytes = zip_of(
            &[
                ("[Content_Types].xml", types.as_bytes()),
                ("_rels/.rels", rels.as_bytes()),
                ("word/main.dat", deep.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidXml
        );
        let shallow = zip_of(
            &[
                ("[Content_Types].xml", types.as_bytes()),
                ("_rels/.rels", rels.as_bytes()),
                ("word/main.dat", DOC.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert!(admit(&shallow, InputLimits::default()).is_ok());
    }

    #[test]
    fn the_main_part_follows_the_relationship() {
        let rels = RELS.replace("word/document.xml", "/content/main.xml");
        let types = TYPES.replace("/word/document.xml", "/content/main.xml");
        let bytes = zip_of(
            &[
                ("[Content_Types].xml", types.as_bytes()),
                ("_rels/.rels", rels.as_bytes()),
                ("content/main.xml", DOC.as_bytes()),
            ],
            CompressionMethod::Deflated,
        );
        assert_eq!(
            admit(&bytes, InputLimits::default()).unwrap().main_part,
            "content/main.xml"
        );
    }

    #[test]
    fn deep_or_malformed_xml_is_refused() {
        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
        let bytes = docx_with(&[("word/deep.xml", deep.as_bytes())]);
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidXml
        );
        let broken = docx_with(&[("word/broken.xml", b"<a><b attr=\"x></a>")]);
        assert_eq!(
            kind(&broken, InputLimits::default()),
            AdmissionErrorKind::InvalidXml
        );
    }

    #[test]
    fn mismatched_unclosed_and_stray_end_tags_are_refused() {
        for (name, xml) in [
            ("word/mismatch.xml", "<a></b>"),
            ("word/unclosed.xml", "<a><b></b>"),
            ("word/stray.xml", "<a></a></a>"),
        ] {
            let bytes = docx_with(&[(name, xml.as_bytes())]);
            assert_eq!(
                kind(&bytes, InputLimits::default()),
                AdmissionErrorKind::InvalidXml,
                "{xml}"
            );
        }
        let fine = docx_with(&[("word/fine.xml", b"<a><b/><c></c></a>")]);
        assert!(admit(&fine, InputLimits::default()).is_ok());
    }

    /// N01/N02: parts reaching the DOM parser carry bounded tag width and
    /// namespace budgets — the admission scan refuses them before any node
    /// or attribute is constructed, whatever the part is named.
    #[test]
    fn pathological_tags_and_namespaces_are_refused_at_admission() {
        let wide_attrs = format!(
            "<a{}/>",
            (0..1025).map(|i| format!(" a{i}='v'")).collect::<String>()
        );
        let alias = "<a xmlns:p='urn:x' xmlns:q='urn:x' p:b='1' q:b='2'/>";
        let big_uri = format!("<a xmlns:p='{}'/>", "u".repeat(9000));
        let many_live = format!(
            "<a{}/>",
            (0..300)
                .map(|i| format!(" xmlns:p{i}='urn:{i}'"))
                .collect::<String>()
        );
        for (name, xml) in [
            ("word/wide.xml", wide_attrs.as_str()),
            ("word/alias.xml", alias),
            ("word/uri.xml", big_uri.as_str()),
            ("word/rels/live.rels", many_live.as_str()),
        ] {
            let bytes = docx_with(&[(name, xml.as_bytes())]);
            assert_eq!(
                kind(&bytes, InputLimits::default()),
                AdmissionErrorKind::InvalidXml,
                "{xml}"
            );
        }
        // The exact boundaries still admit.
        let at_cap = format!(
            "<a{}/>",
            (0..1024).map(|i| format!(" a{i}='v'")).collect::<String>()
        );
        let bytes = docx_with(&[("word/capped.xml", at_cap.as_bytes())]);
        assert!(admit(&bytes, InputLimits::default()).is_ok());
    }

    #[test]
    fn utf16_and_non_utf8_parts_are_scanned_not_refused() {
        // SharePoint writes customXml items as UTF-16 and Word opens them;
        // a stray Latin-1 byte decodes lossily, as the engine reads it.
        let utf16 = |xml: &str, le: bool| -> Vec<u8> {
            let bom = if le { [0xFF, 0xFE] } else { [0xFE, 0xFF] };
            bom.into_iter()
                .chain(
                    xml.encode_utf16()
                        .flat_map(|u| if le { u.to_le_bytes() } else { u.to_be_bytes() }),
                )
                .collect()
        };
        let item = r#"<?xml version="1.0" encoding="utf-16"?><p:properties xmlns:p="urn:x"><p:a>é</p:a></p:properties>"#;
        for le in [true, false] {
            let bytes = docx_with(&[("customXml/item1.xml", &utf16(item, le))]);
            assert!(admit(&bytes, InputLimits::default()).is_ok(), "le={le}");
        }
        let latin1 = docx_with(&[("word/latin1.xml", b"<a>caf\xe9</a>")]);
        assert!(admit(&latin1, InputLimits::default()).is_ok());
        // The depth budget still applies through the transcoding.
        let deep = format!("{}{}", "<a>".repeat(300), "</a>".repeat(300));
        let deep = docx_with(&[("customXml/item2.xml", &utf16(&deep, true))]);
        assert_eq!(
            kind(&deep, InputLimits::default()),
            AdmissionErrorKind::InvalidXml
        );
    }

    /// Replace the trailing end-of-central-directory record of `zip` by a
    /// ZIP64 locator pointing at `record_offset`, an EOCD declaring 0xFFFF
    /// entries, and `comment` as the archive comment.
    fn with_zip64_tail(zip: &[u8], record_offset: u64, comment: &[u8]) -> Vec<u8> {
        let eocd_at = zip.len() - 22;
        assert_eq!(&zip[eocd_at..eocd_at + 4], &[0x50, 0x4b, 0x05, 0x06]);
        let mut out = zip[..eocd_at].to_vec();
        out.extend_from_slice(&[0x50, 0x4b, 0x06, 0x07]);
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&record_offset.to_le_bytes());
        out.extend_from_slice(&1u32.to_le_bytes());
        let mut eocd = zip[eocd_at..].to_vec();
        eocd[8..10].copy_from_slice(&u16::MAX.to_le_bytes());
        eocd[10..12].copy_from_slice(&u16::MAX.to_le_bytes());
        eocd[20..22].copy_from_slice(&u16::try_from(comment.len()).unwrap().to_le_bytes());
        out.extend_from_slice(&eocd);
        out.extend_from_slice(comment);
        out
    }

    /// A 56-byte ZIP64 end-of-central-directory record declaring `entries`.
    fn eocd64(entries: u64) -> Vec<u8> {
        let mut record = vec![0x50, 0x4b, 0x06, 0x06];
        record.extend_from_slice(&44u64.to_le_bytes());
        record.extend_from_slice(&[45, 0, 45, 0]);
        record.extend_from_slice(&0u32.to_le_bytes());
        record.extend_from_slice(&0u32.to_le_bytes());
        record.extend_from_slice(&entries.to_le_bytes());
        record.extend_from_slice(&entries.to_le_bytes());
        record.extend_from_slice(&0u64.to_le_bytes());
        record.extend_from_slice(&0u64.to_le_bytes());
        assert_eq!(record.len(), 56);
        record
    }

    #[test]
    fn a_zip64_end_record_is_read_at_the_locator_offset() {
        let docx = docx_with(&[]);
        let record_at = docx.len() - 22;
        let mut body = docx[..record_at].to_vec();
        body.extend_from_slice(&eocd64(7));
        body.extend_from_slice(&docx[record_at..]);
        let bytes = with_zip64_tail(&body, record_at as u64, b"");
        assert_eq!(declared_entry_count(&bytes).unwrap(), 7);
    }

    #[test]
    fn a_zip64_locator_offset_past_the_end_is_refused_not_indexed() {
        let docx = docx_with(&[]);
        let past = with_zip64_tail(&docx, docx.len() as u64 + 1000, b"");
        let err = declared_entry_count(&past).expect_err("offset past the end");
        assert_eq!(err.kind, AdmissionErrorKind::InvalidPackage);
        assert_eq!(
            kind(&past, InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
    }

    #[test]
    fn a_zip64_locator_offset_near_usize_max_does_not_overflow() {
        let docx = docx_with(&[]);
        for offset in [
            u64::MAX,
            u64::MAX - 20,
            u64::MAX - 39,
            usize::MAX as u64 - 39,
        ] {
            let huge = with_zip64_tail(&docx, offset, b"");
            let err = declared_entry_count(&huge).expect_err("offset near usize::MAX");
            assert_eq!(err.kind, AdmissionErrorKind::InvalidPackage);
            assert_eq!(
                kind(&huge, InputLimits::default()),
                AdmissionErrorKind::InvalidPackage
            );
        }
    }

    #[test]
    fn a_truncated_zip64_end_record_is_refused() {
        let docx = docx_with(&[]);
        // The record signature sits in the archive comment, 39 bytes before
        // the end: a valid offset whose 40-byte record runs past the buffer.
        let mut comment = vec![0x50, 0x4b, 0x06, 0x06];
        comment.resize(39, 0);
        let probe = with_zip64_tail(&docx, 0, &comment);
        let record_at = (probe.len() - 39) as u64;
        let truncated = with_zip64_tail(&docx, record_at, &comment);
        assert_eq!(truncated.len(), probe.len());
        assert_eq!(
            &truncated[record_at as usize..record_at as usize + 4],
            &[0x50, 0x4b, 0x06, 0x06]
        );
        let err = declared_entry_count(&truncated).expect_err("truncated record");
        assert_eq!(err.kind, AdmissionErrorKind::InvalidPackage);
    }

    #[test]
    fn unbounded_part_and_package_budgets_admit_a_docx() {
        let bytes = docx_with(&[]);
        let unbounded = InputLimits {
            max_part_bytes: u64::MAX,
            max_uncompressed_bytes: u64::MAX,
            ..InputLimits::default()
        };
        let admitted = admit(&bytes, unbounded).unwrap();
        assert_eq!(admitted.main_part, "word/document.xml");
        assert_eq!(
            admitted.inflated_bytes,
            (TYPES.len() + RELS.len() + DOC.len()) as u64
        );
    }

    #[test]
    fn error_codes_are_stable() {
        use AdmissionErrorKind as K;
        let codes: Vec<_> = K::ALL.map(K::code).into();
        assert_eq!(
            codes,
            [
                "INPUT_LIMIT",
                "DUPLICATE_PART",
                "UNSUPPORTED_PACKAGE",
                "INVALID_PACKAGE",
                "ZIP_HEADER_MISMATCH",
                "INVALID_XML",
                "LEGACY_DOC"
            ]
        );
        let e = AdmissionError::new(K::InputLimit, "x");
        assert_eq!(e.to_string(), "INPUT_LIMIT: x");
    }

    #[test]
    fn an_ole_compound_file_is_legacy_doc_with_a_save_as_hint() {
        let mut doc = b"\xD0\xCF\x11\xE0\xA1\xB1\x1A\xE1".to_vec();
        doc.extend_from_slice(&[0u8; 512]);
        let e = admit(&doc, InputLimits::default()).unwrap_err();
        assert_eq!(e.kind, AdmissionErrorKind::LegacyDocument);
        assert_eq!(e.code(), "LEGACY_DOC");
        assert!(e.message.contains("save it as .docx"), "{}", e.message);
        // Before the size budget, and under the compare budget too.
        let tiny = InputLimits {
            max_compressed_bytes: 1,
            ..InputLimits::default()
        };
        assert_eq!(kind(&doc, tiny), AdmissionErrorKind::LegacyDocument);
        assert_eq!(
            kind(&doc, InputLimits::compare()),
            AdmissionErrorKind::LegacyDocument
        );
        assert_eq!(sniff(&doc).unwrap_err().code(), "LEGACY_DOC");
        assert_eq!(sniff(&docx_with(&[])), Ok(()));
    }

    #[test]
    fn rtf_is_unsupported_not_a_zip_error() {
        let e = admit(b"{\\rtf1\\ansi hello}", InputLimits::default()).unwrap_err();
        assert_eq!(e.code(), "UNSUPPORTED_PACKAGE");
        assert!(e.message.contains("RTF"), "{}", e.message);
        // Only the signature: RTF-looking text after it is not RTF.
        assert_eq!(sniff(b"PK\x03\x04{\\rtf"), Ok(()));
    }

    #[test]
    fn code_first_puts_the_code_first_and_keeps_the_context() {
        assert_eq!(
            code_first("INPUT_LIMIT: 10001 ZIP entries").as_deref(),
            Some("INPUT_LIMIT: 10001 ZIP entries")
        );
        // The comparer's opaque wrapper goes; context that names the input
        // stays, after the code.
        assert_eq!(
            code_first("I/O error: INPUT_LIMIT: original document: 10001 ZIP entries").as_deref(),
            Some("INPUT_LIMIT: original document: 10001 ZIP entries")
        );
        assert_eq!(
            code_first("document B: INVALID_PACKAGE: not a ZIP file").as_deref(),
            Some("INVALID_PACKAGE: document B: not a ZIP file")
        );
        // Only `: ` separates; a colon inside the detail stays.
        assert_eq!(
            code_first("I/O error: LEGACY_DOC: at 12:00").as_deref(),
            Some("LEGACY_DOC: at 12:00")
        );
        // Codes are whole segments, never substrings.
        assert_eq!(code_first("INPUT_LIMITS: x"), None);
        assert_eq!(code_first("no code here"), None);
    }

    /// An entry's local-header and central-record offsets, located
    /// through the reader rather than by scanning for signatures.
    fn header_offsets(bytes: &[u8], name: &str) -> (usize, usize) {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        let entry = archive.by_name(name).unwrap();
        (
            usize::try_from(entry.header_start()).unwrap(),
            usize::try_from(entry.central_header_start()).unwrap(),
        )
    }

    /// ZIP-17 item 1 (#644): a local header whose content fields
    /// disagree with the central record describes a different entry to
    /// a scanner trusting local headers — the same class of defect as a
    /// name mismatch (#594). One case per patched field; the central
    /// record keeps the true values, so today's reader admits them all.
    #[test]
    fn zip17_rejects_local_header_disagreement() {
        for (offset, width, value) in [
            (6usize, 2usize, [0x08, 0x00, 0, 0]), // flags: set bit 3 locally
            (8, 2, [0x08, 0x00, 0, 0]),           // method: deflate vs stored
            (14, 4, [0xef, 0xbe, 0xad, 0xde]),    // CRC-32
            (18, 4, [2, 0, 0, 0]),                // compressed size
            (22, 4, [2, 0, 0, 0]),                // uncompressed size
        ] {
            let mut bytes =
                zip_names::docx_package("", &[zip_names::RawEntry::raw(b"x.bin", b"x")]);
            let (lfh, _) = header_offsets(&bytes, "x.bin");
            bytes[lfh + offset..lfh + offset + width].copy_from_slice(&value[..width]);
            let err = admit(&bytes, InputLimits::default())
                .expect_err("a local field that disagrees must refuse");
            assert_eq!(
                err.kind,
                AdmissionErrorKind::ZipHeaderMismatch,
                "offset {offset}: {err}"
            );
        }
    }

    /// Set or clear one flag bit in one header of `name`'s entry.
    fn patch_flag(bytes: &mut [u8], name: &str, bit: u16, local: bool, set: bool) {
        let (lfh, central) = header_offsets(bytes, name);
        let at = if local { lfh + 6 } else { central + 8 };
        let flags = u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        let flags = if set { flags | bit } else { flags & !bit };
        bytes[at..at + 2].copy_from_slice(&flags.to_le_bytes());
    }

    /// #644 fix round: bit 11 only matters when the raw name is not
    /// ASCII — it selects UTF-8 over CP437 decoding, which an ASCII name
    /// decodes the same either way. A real Word-opened input carries the
    /// bit set in the local header only on an ASCII name.
    #[test]
    fn zip17_compares_the_utf8_flag_only_for_non_ascii_names() {
        let mut ascii = zip_names::docx_package("", &[zip_names::RawEntry::raw(b"x.bin", b"x")]);
        patch_flag(&mut ascii, "x.bin", 0x0800, true, true);
        admit(&ascii, InputLimits::default())
            .expect("a one-sided UTF-8 flag on an ASCII name admits");
        for local_only in [true, false] {
            let mut bytes = zip_names::docx_package(
                "",
                &[zip_names::RawEntry::raw("word/m\u{e9}dia.bin".as_bytes(), b"x").utf8()],
            );
            patch_flag(&mut bytes, "word/m\u{e9}dia.bin", 0x0800, local_only, false);
            assert_eq!(
                kind(&bytes, InputLimits::default()),
                AdmissionErrorKind::ZipHeaderMismatch,
                "local_only={local_only}"
            );
        }
    }

    /// ZIP-17 item 2: a central record whose local header lives inside
    /// another entry's payload is refused even though every compared
    /// field — name, method, flags, CRC-32, sizes — matches.
    #[test]
    fn zip17_rejects_nested_entry_with_matching_names() {
        let mut embedded = zip_names::local_header(b"b.bin", 0, zip_names::crc32(b"x"), 1, 1, &[]);
        assert_eq!(embedded.len(), 35);
        embedded.push(b'x');
        let mut bytes = zip_names::docx_package(
            "",
            &[
                zip_names::RawEntry::raw(b"a.bin", &embedded),
                zip_names::RawEntry::raw(b"b.bin", b"x"),
            ],
        );
        // a.bin's payload opens with the embedded header at its data start.
        let (a_lfh, _) = header_offsets(&bytes, "a.bin");
        let embedded_at = a_lfh + 30 + b"a.bin".len();
        assert_eq!(&bytes[embedded_at..embedded_at + 4], b"PK\x03\x04");
        let (_, b_central) = header_offsets(&bytes, "b.bin");
        bytes[b_central + 42..b_central + 46]
            .copy_from_slice(&u32::try_from(embedded_at).unwrap().to_le_bytes());
        // b.bin's original, now-unreferenced local header is merely a gap.
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
    }

    /// ZIP-17 item 2: a local header placed after the central directory —
    /// inside the archive comment — is outside every legal extent.
    #[test]
    fn zip17_rejects_entry_inside_central_directory() {
        let mut bytes = zip_names::docx_package("", &[zip_names::RawEntry::raw(b"c.bin", b"x")]);
        let (_, c_central) = header_offsets(&bytes, "c.bin");
        let comment_at = bytes.len();
        // The EOCD comment starts right after the 22-byte record.
        let comment_len_at = bytes.len() - 2;
        bytes[comment_len_at..comment_len_at + 2].copy_from_slice(&39u16.to_le_bytes());
        let mut comment = zip_names::local_header(b"c.bin", 0, zip_names::crc32(b"x"), 1, 1, &[]);
        comment.extend_from_slice(b"x");
        comment.resize(39, 0);
        bytes.extend_from_slice(&comment);
        bytes[c_central + 42..c_central + 46]
            .copy_from_slice(&u32::try_from(comment_at).unwrap().to_le_bytes());
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
    }

    /// The mandatory parts plus `extra`, assembled record by record.
    fn assembled_docx(extra: &[zip_names::AssembledEntry<'_>]) -> Vec<u8> {
        let mut entries = vec![
            zip_names::AssembledEntry::plain(b"[Content_Types].xml", TYPES.as_bytes()),
            zip_names::AssembledEntry::plain(b"_rels/.rels", RELS.as_bytes()),
            zip_names::AssembledEntry::plain(b"word/document.xml", DOC.as_bytes()),
        ];
        entries.extend(extra.iter().map(|entry| zip_names::AssembledEntry {
            name: entry.name,
            data: entry.data.clone(),
            flags: entry.flags,
            descriptor: entry.descriptor,
            zip64: entry.zip64,
            local_extra: entry.local_extra,
        }));
        zip_names::assembled(b"", &entries, None, b"")
    }

    /// ZIP-17 item 3: a descriptor whose CRC-32 disagrees with the
    /// central record is refused — today the seeking reader never
    /// validates the descriptor it walks past.
    #[test]
    fn zip17_rejects_descriptor_disagreement() {
        let entry = zip_names::AssembledEntry::plain(b"x.bin", b"payload")
            .with_descriptor(zip_names::Descriptor::Signed);
        let mut bytes = assembled_docx(&[entry]);
        let signature = bytes
            .windows(4)
            .position(|w| w == b"PK\x07\x08")
            .expect("signed descriptor");
        bytes[signature + 4] ^= 0xff;
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
    }

    /// ZIP-17 items 2-3 controls: signed and unsigned ZIP32 descriptors,
    /// a small forced-ZIP64 descriptor, a descriptor followed
    /// immediately by the next local header, a package with a prefix
    /// (APPNOTE 4.3.6 allows a self-extracting stub; jubarte admits one)
    /// and one whose central order differs from its physical order all
    /// admit.
    #[test]
    fn zip17_accepts_legitimate_descriptor_and_layout_shapes() {
        for shape in [
            zip_names::Descriptor::Signed,
            zip_names::Descriptor::Unsigned,
        ] {
            // The descriptor entry first: the next local header follows it.
            let entries = vec![
                zip_names::AssembledEntry::plain(b"x.bin", b"payload").with_descriptor(shape),
                zip_names::AssembledEntry::plain(b"[Content_Types].xml", TYPES.as_bytes()),
                zip_names::AssembledEntry::plain(b"_rels/.rels", RELS.as_bytes()),
                zip_names::AssembledEntry::plain(b"word/document.xml", DOC.as_bytes()),
            ];
            let bytes = zip_names::assembled(b"", &entries, None, b"");
            admit(&bytes, InputLimits::default()).expect("descriptor package admits");
            // The same package with a prefix and a reversed central order.
            let shuffled =
                zip_names::assembled(b"JUNKJUNK", &entries, Some(&[3, 2, 1, 0]), b"no comment");
            admit(&shuffled, InputLimits::default()).expect("prefix and reorder admit");
        }
        let zip64 = zip_names::AssembledEntry::plain(b"x.bin", b"payload")
            .with_descriptor(zip_names::Descriptor::SignedZip64)
            .zip64();
        admit(&assembled_docx(&[zip64]), InputLimits::default())
            .expect("a ZIP64 descriptor package admits");
        let unsigned64 = zip_names::AssembledEntry::plain(b"x.bin", b"payload")
            .with_descriptor(zip_names::Descriptor::UnsignedZip64)
            .zip64();
        admit(&assembled_docx(&[unsigned64]), InputLimits::default())
            .expect("an unsigned ZIP64 descriptor package admits");
        // A bit-3 entry without its descriptor refuses.
        let missing = zip_names::AssembledEntry {
            flags: 0x0008,
            ..zip_names::AssembledEntry::plain(b"x.bin", b"payload")
        };
        assert_eq!(
            kind(&assembled_docx(&[missing]), InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
    }

    /// #644 fix round: APPNOTE 4.3.9.2 sizes a descriptor by the local
    /// header's 0x0001 field — present at all, whatever its values — not
    /// by 0xFFFFFFFF placeholders or 4-GiB sizes alone. The prescribed
    /// width wins when both match; a CRC or size that disagrees in both
    /// widths still refuses.
    #[test]
    fn zip17_descriptor_width_follows_the_local_zip64_extra() {
        let extra = zip_names::zip64_extra(0, 0);
        let wide = zip_names::AssembledEntry {
            local_extra: extra.as_slice(),
            ..zip_names::AssembledEntry::plain(b"x.bin", b"payload")
                .with_descriptor(zip_names::Descriptor::SignedZip64)
        };
        admit(
            &assembled_docx(std::slice::from_ref(&wide)),
            InputLimits::default(),
        )
        .expect("a 24-byte descriptor behind a ZIP64 local extra admits");
        // A writer that got the width wrong still admits through the
        // other width: the CRC and both sizes must match either way.
        let narrow = zip_names::AssembledEntry {
            local_extra: extra.as_slice(),
            ..zip_names::AssembledEntry::plain(b"x.bin", b"payload")
                .with_descriptor(zip_names::Descriptor::Signed)
        };
        admit(&assembled_docx(&[narrow]), InputLimits::default())
            .expect("a 16-byte descriptor behind a ZIP64 local extra admits");
        let mut bytes = assembled_docx(std::slice::from_ref(&wide));
        let signature = bytes
            .windows(4)
            .position(|w| w == b"PK\x07\x08")
            .expect("signed descriptor");
        bytes[signature + 4] ^= 0xff;
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
    }

    /// ZIP-17 item 4: a directory entry's payload goes through the same
    /// bounded CRC stream — a corrupt one is INVALID_PACKAGE — and its
    /// actual bytes charge both budgets, not an early skip.
    #[test]
    fn zip17_checks_directory_crc_and_accounting() {
        let directory = zip_names::RawEntry::raw(b"word/media/", b"x");
        let mut bytes = zip_names::docx_package("", std::slice::from_ref(&directory));
        let (lfh, _) = header_offsets(&bytes, "word/media/");
        let payload_at = lfh + 30 + b"word/media/".len();
        assert_eq!(bytes[payload_at], b'x');
        bytes[payload_at] = b'y';
        assert_eq!(
            kind(&bytes, InputLimits::default()),
            AdmissionErrorKind::InvalidPackage
        );
        // Uncorrupted: the directory's byte joins inflated_bytes …
        let admitted = admit(
            &zip_names::docx_package("", std::slice::from_ref(&directory)),
            InputLimits::default(),
        )
        .expect("a valid directory payload admits");
        let ordinary = admit(&zip_names::docx_package("", &[]), InputLimits::default())
            .unwrap()
            .inflated_bytes;
        assert_eq!(admitted.inflated_bytes, ordinary + 1);
        // … and the aggregate budget counts it: exactly the ordinary
        // parts' total leaves no room for the directory's byte.
        let exact = InputLimits {
            max_uncompressed_bytes: ordinary,
            ..InputLimits::default()
        };
        assert_eq!(
            kind(&zip_names::docx_package("", &[directory]), exact),
            AdmissionErrorKind::InputLimit
        );
    }

    /// Word's own `o:gfxdata` VML attribute lives in `word/header*.xml`
    /// and reaches ~366 KiB in the bench corpus. The admission scan runs
    /// the checked gate on every XML part, including headers, so the same
    /// input the unit test builds must also pass `admit` on a real docx.
    #[test]
    fn a_word_gfxdata_header_is_admitted_on_a_real_docx() {
        let gfx_value: String = "A".repeat(400 * 1024);
        let header = format!(
            r#"<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:o="urn:schemas-microsoft-com:office:office"><w:p><w:r><w:pict><v:shape o:gfxdata="{gfx_value}"/></w:pict></w:r></w:p></w:hdr>"#
        );
        let header_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"/>"#;
        let bytes = docx_with(&[
            ("word/header1.xml", header.as_bytes()),
            ("word/_rels/header1.xml.rels", header_rels.as_bytes()),
        ]);
        let admitted = admit(&bytes, InputLimits::default())
            .expect("Word's o:gfxdata header must be admitted");
        assert!(admitted.entries >= 5);
    }

    /// Overwrite `entry`'s central record: `version_made_by` at +4 (host
    /// byte high, spec low) and the external attributes at +38. The local
    /// header has neither field, so nothing else needs patching.
    fn set_central_attributes(bytes: &mut [u8], entry: &str, version_made_by: u16, external: u32) {
        let at = {
            let mut archive = zip::ZipArchive::new(Cursor::new(&*bytes)).unwrap();
            usize::try_from(archive.by_name(entry).unwrap().central_header_start()).unwrap()
        };
        assert_eq!(&bytes[at..at + 4], b"PK\x01\x02");
        bytes[at + 4..at + 6].copy_from_slice(&version_made_by.to_le_bytes());
        bytes[at + 38..at + 42].copy_from_slice(&external.to_le_bytes());
    }

    /// ZIP-16 (#639): an entry recorded as a non-regular Unix type, and a
    /// directory type whose name is not slash-terminated, are refused like
    /// encryption and exotic compression (`UNSUPPORTED_PACKAGE`), with the
    /// message naming the entry and its type.
    #[test]
    fn zip16_refuses_special_entry_types() {
        for (version_made_by, mode, name, detail) in [
            (0x0314u16, 0o120777u32, "x.bin", "symlink"),
            (0x0314, 0o020666, "x.bin", "character device"),
            (0x0314, 0o060600, "x.bin", "block device"),
            (0x0314, 0o010644, "x.bin", "FIFO"),
            (0x1314, 0o140755, "x.bin", "socket"),
            (0x0314, 0o160644, "x.bin", "160000"),
            (0x0314, 0o040755, "x.bin", "directory"),
        ] {
            // The Unix mode is the high 16 bits of the external attributes.
            let external = mode << 16;
            let mut bytes =
                zip_names::docx_package("", &[zip_names::RawEntry::raw(name.as_bytes(), b"x")]);
            set_central_attributes(&mut bytes, name, version_made_by, external);
            let err =
                admit(&bytes, InputLimits::default()).expect_err("a special entry must be refused");
            assert_eq!(
                err.kind,
                AdmissionErrorKind::UnsupportedPackage,
                "{version_made_by:#06x}/{mode:o}: {err}"
            );
            assert!(
                err.message.contains(name) && err.message.contains(detail),
                "{version_made_by:#06x}/{mode:o}: {}",
                err.message
            );
        }
        // The DOS directory attribute on any host, without a trailing slash.
        for version_made_by in [0x0014u16, 0x0a14, 0x0314] {
            let mut bytes =
                zip_names::docx_package("", &[zip_names::RawEntry::raw(b"x.bin", b"x")]);
            set_central_attributes(&mut bytes, "x.bin", version_made_by, 0x10);
            assert_eq!(
                kind(&bytes, InputLimits::default()),
                AdmissionErrorKind::UnsupportedPackage,
                "{version_made_by:#06x}: a DOS directory on a file name must refuse"
            );
        }
    }

    /// ZIP-16 controls (#639): the shapes real writers emit still admit —
    /// an Info-ZIP regular file and directory, an OS X host record, a
    /// DOS-host archive bit, no attributes at all, and the macOS Archive
    /// Utility shape (regular `0o644` files, `0o755` directories that
    /// carry the DOS directory bit).
    #[test]
    fn zip16_admits_recorded_entry_types() {
        for (version_made_by, external, name) in [
            (0x0314u16, 0o100644u32 << 16, "x.bin"), // Info-ZIP file
            (0x0314, 0o040755 << 16, "word/media/"), // Info-ZIP directory
            (0x1314, 0o100600 << 16, "x.bin"),       // an OS X host record
            (0x0014, 0x20, "x.bin"),                 // DOS archive bit
            (0x0014, 0, "x.bin"),                    // attributes 0
            (0x0314, 0o100644 << 16, "x.bin"),       // Archive Utility file
            (0x0314, 0o040755 << 16 | 0x10, "word/media/"), // … and its directory
        ] {
            let mut bytes =
                zip_names::docx_package("", &[zip_names::RawEntry::raw(name.as_bytes(), b"")]);
            set_central_attributes(&mut bytes, name, version_made_by, external);
            admit(&bytes, InputLimits::default())
                .unwrap_or_else(|err| panic!("{version_made_by:#06x}/{external:o} {name}: {err}"));
        }
    }

    /// ZIP-15 (#600): two central records naming one part — byte-equal or
    /// an ASCII-case alias — is `DUPLICATE_PART` on every route, found by
    /// a raw walk of the central directory before any index collapses the
    /// second record. Word refuses each of these packages.
    #[test]
    fn duplicate_zip_identities_refuse_with_both_spellings() {
        for (alias, first, second) in [
            (false, "word/document.xml", "word/document.xml"),
            (true, "word/document.xml", "word/Document.xml"),
            (true, "word/document.xml", "WORD/document.xml"),
            (true, "[Content_Types].xml", "[content_types].xml"),
            (true, "_rels/.rels", "_RELS/.rels"),
        ] {
            let bytes = zip_names::docx_package(
                "",
                &[zip_names::RawEntry::raw(second.as_bytes(), b"SECOND")],
            );
            let err = admit(&bytes, InputLimits::default())
                .expect_err("a second entry of one part must be refused");
            assert_eq!(
                err.kind,
                AdmissionErrorKind::DuplicatePart,
                "{first}/{second}"
            );
            assert!(err.message.contains(first), "{err}");
            assert!(err.message.contains(second), "{err}");
            if !alias {
                assert!(err.message.contains("same name"), "{err}");
            } else {
                assert!(err.message.contains("ignore case"), "{err}");
            }
            // The package constructor refuses the same identity — as the
            // typed `DuplicatePartName` variant Rust callers match on, naming
            // the second entry's spelling (#600).
            assert!(
                matches!(
                    crate::opc::PartFs::open(&bytes),
                    Err(crate::opc::OpcError::DuplicatePartName(ref n)) if n == second
                ),
                "{first}/{second}"
            );
        }
    }

    /// A directory entry is not a part: `word/` beside `word/document.xml`
    /// (probe p8, which Word opens) and two directory spellings of one
    /// folder admit, where two FILE entries of one identity refuse.
    #[test]
    fn directory_entries_are_not_part_identities() {
        for extra in [
            zip_names::RawEntry::raw(b"word/", b""),
            zip_names::RawEntry::raw(b"WORD/", b""),
        ] {
            let bytes = zip_names::docx_package("", &[extra]);
            admit(&bytes, InputLimits::default())
                .expect("directory entries never collide with parts");
        }
        let both = zip_names::docx_package(
            "",
            &[
                zip_names::RawEntry::raw(b"word/", b""),
                zip_names::RawEntry::raw(b"WORD/", b""),
            ],
        );
        admit(&both, InputLimits::default()).expect("two spellings of one directory");
    }

    /// The message a text surface prints for the constructor's typed
    /// `DuplicatePartName`: the admission refusal for the same package —
    /// the `DUPLICATE_PART` code and both spellings — while any other
    /// error's text passes through unchanged (#600).
    #[test]
    fn a_duplicate_refusal_reads_as_the_admission_detail() {
        let bytes = zip_names::docx_package(
            "",
            &[zip_names::RawEntry::raw(b"word/Document.xml", b"SECOND")],
        );
        let err = crate::opc::PartFs::open(&bytes)
            .err()
            .expect("the constructor refuses the case alias");
        let message = crate::opc::refusal_message(&bytes, &err);
        assert!(message.starts_with("DUPLICATE_PART: "), "{message}");
        assert!(message.contains("'word/document.xml'"), "{message}");
        assert!(message.contains("'word/Document.xml'"), "{message}");
        let exact = zip_names::docx_package(
            "",
            &[zip_names::RawEntry::raw(b"word/document.xml", b"SECOND")],
        );
        let err = crate::opc::PartFs::open(&exact)
            .err()
            .expect("the constructor refuses the byte-equal duplicate");
        let message = crate::opc::refusal_message(&exact, &err);
        assert!(message.starts_with("DUPLICATE_PART: "), "{message}");
        assert!(
            message.contains("two ZIP entries have the same name 'word/document.xml'"),
            "{message}"
        );
        // Any other `OpcError` keeps its own text.
        let err = crate::opc::PartFs::open(b"not a zip")
            .err()
            .expect("not a zip is refused");
        let message = crate::opc::refusal_message(b"not a zip", &err);
        assert_eq!(message, err.to_string());
    }

    /// One case-variant name is the part (ECMA-376 Part 2 §6.2.2.3
    /// M1.12): Word opens `word/Document.xml`, `[content_types].xml` and
    /// `_RELS/.RELS` as the only spellings of the reserved and main parts
    /// (probes p3, p4, p5), so admission resolves them by identity and
    /// reports the stored spelling as the main part.
    #[test]
    fn a_case_variant_name_is_the_part() {
        let bytes = zip_names::zip(&[
            zip_names::RawEntry::raw(b"[Content_Types].xml", TYPES.as_bytes()),
            zip_names::RawEntry::raw(b"_rels/.rels", RELS.as_bytes()),
            zip_names::RawEntry::raw(b"word/Document.xml", DOC.as_bytes()),
        ]);
        let admitted = admit(&bytes, InputLimits::default())
            .expect("the case-variant main part is the part the rels target names");
        assert_eq!(admitted.main_part, "word/Document.xml");

        let lower_types = zip_names::zip(&[
            zip_names::RawEntry::raw(b"[content_types].xml", TYPES.as_bytes()),
            zip_names::RawEntry::raw(b"_rels/.rels", RELS.as_bytes()),
            zip_names::RawEntry::raw(b"word/document.xml", DOC.as_bytes()),
        ]);
        admit(&lower_types, InputLimits::default())
            .expect("the case-variant [Content_Types].xml is the content types");

        let upper_rels = zip_names::zip(&[
            zip_names::RawEntry::raw(b"[Content_Types].xml", TYPES.as_bytes()),
            zip_names::RawEntry::raw(b"_RELS/.RELS", RELS.as_bytes()),
            zip_names::RawEntry::raw(b"word/document.xml", DOC.as_bytes()),
        ]);
        admit(&upper_rels, InputLimits::default())
            .expect("the case-variant _RELS/.RELS is the package relationships");
    }
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
pub(crate) mod limit_test_support {
    use super::InputLimits;
    use std::io::{Cursor, Write};
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    pub fn package() -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (name, data) in [
            ("[Content_Types].xml", b"<Types/>" as &[u8]),
            ("word/media/a.bin", &[b'a'; 80]),
            ("word/media/b.bin", &[b'b'; 80]),
        ] {
            writer.start_file(name, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    pub fn limits() -> InputLimits {
        InputLimits {
            max_part_bytes: 128,
            max_uncompressed_bytes: 168,
            max_entries: 3,
            ..InputLimits::compare()
        }
    }

    pub fn compressed() -> InputLimits {
        InputLimits {
            max_compressed_bytes: package().len() as u64 - 1,
            ..limits()
        }
    }
    pub fn count() -> InputLimits {
        InputLimits {
            max_entries: 2,
            ..limits()
        }
    }
    pub fn part() -> InputLimits {
        InputLimits {
            max_part_bytes: 64,
            ..limits()
        }
    }
    pub fn total() -> InputLimits {
        InputLimits {
            max_uncompressed_bytes: 160,
            ..limits()
        }
    }
}
