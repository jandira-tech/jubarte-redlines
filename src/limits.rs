// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

#![cfg_attr(
    not(test),
    deny(clippy::indexing_slicing, clippy::arithmetic_side_effects)
)]

//! Input resource budgets and refusal error types shared by XML parsing,
//! ZIP package admission and the comparer.

use std::fmt;

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
    /// that know their documents set `WmlComparerSettings::input_limits`
    /// tighter.
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

/// A story holds markup the comparer cannot flatten into atoms: a
/// `w:subDoc`, `w:altChunk`, `w:contentPart` or custom XML markup.
/// Carried inside `OpcError::Io`; callers can downcast its `get_ref()`.
#[derive(Debug)]
pub struct UnsupportedContentError {
    /// The story and the element, such as
    /// `original document: Document contains subDoc`.
    pub message: String,
}

impl fmt::Display for UnsupportedContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Self::CODE, self.message)
    }
}

impl std::error::Error for UnsupportedContentError {}

impl UnsupportedContentError {
    /// The stable error code agents and bindings see.
    pub const CODE: &'static str = "UNSUPPORTED_CONTENT";
}

/// A refusal's message with its code first (`CODE: context: detail`): the
/// comparer's opaque `I/O error: ` wrapper is dropped, any other context
/// before the code (`document B: `) stays after it. `None` when no
/// `: `-separated segment of `message` is an admission code or the
/// comparer's `UNSUPPORTED_CONTENT`. The CLI and
/// the bindings print refusals through it, so a code reads the same
/// whatever wrapped it.
#[must_use]
pub fn code_first(message: &str) -> Option<String> {
    let segments: Vec<&str> = message.split(": ").collect();
    let at = segments.iter().position(|segment| {
        *segment == UnsupportedContentError::CODE
            || AdmissionErrorKind::ALL
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

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn code_first_handles_admission_and_unsupported_content_codes() {
        assert_eq!(
            code_first("document B: I/O error: UNSUPPORTED_CONTENT: body: subDoc").as_deref(),
            Some("UNSUPPORTED_CONTENT: document B: body: subDoc")
        );
        assert_eq!(
            code_first("I/O error: INPUT_LIMIT: zip too big").as_deref(),
            Some("INPUT_LIMIT: zip too big")
        );
        assert_eq!(code_first("plain error"), None);
    }

    #[test]
    fn limit_overrides_replace_selected_fields_only() {
        let base = InputLimits::default();
        let patched = InputLimitOverrides::from_json(r#"{"max_part_bytes": 1234}"#)
            .expect("valid overrides")
            .apply(base);
        assert_eq!(patched.max_part_bytes, 1234);
        assert_eq!(patched.max_entries, base.max_entries);
    }
}
