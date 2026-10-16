// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2025 tensorbee
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Error type for OPC operations.
//!
//! Port of rdocx-opc 0.2.0's `error.rs` (MIT; preserved notice in
//! `LICENSES/LicenseRef-rdocx-opc-MIT.txt`), with the `thiserror` derives
//! written out by hand so the crate needs no new dependency. The variants
//! and their `Display` strings are unchanged: they are part of
//! [`crate::opc`]'s public API.

use std::fmt;

/// Errors opening, parsing or writing an OPC package.
#[derive(Debug)]
pub enum OpcError {
    /// The zip layer failed.
    Zip(zip::result::ZipError),
    /// Reading or writing bytes failed.
    Io(std::io::Error),
    /// The XML was malformed.
    Xml(quick_xml::Error),
    /// An XML attribute was malformed.
    XmlAttr(quick_xml::events::attributes::AttrError),
    /// A required part is missing from the package.
    PartNotFound(String),
    /// `[Content_Types].xml` could not be parsed.
    InvalidContentTypes,
    /// A `.rels` part could not be parsed.
    InvalidRelationship,
    /// A name or value was not UTF-8.
    Utf8(std::str::Utf8Error),
}

impl fmt::Display for OpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip(e) => write!(f, "ZIP error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Xml(e) => write!(f, "XML error: {e}"),
            Self::XmlAttr(e) => write!(f, "XML attribute error: {e}"),
            Self::PartNotFound(part) => write!(f, "part not found: {part}"),
            Self::InvalidContentTypes => write!(f, "invalid content types XML"),
            Self::InvalidRelationship => write!(f, "invalid relationship XML"),
            Self::Utf8(e) => write!(f, "UTF-8 error: {e}"),
        }
    }
}

impl std::error::Error for OpcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Zip(e) => Some(e),
            Self::Io(e) => Some(e),
            Self::Xml(e) => Some(e),
            Self::XmlAttr(e) => Some(e),
            Self::Utf8(e) => Some(e),
            Self::PartNotFound(_) | Self::InvalidContentTypes | Self::InvalidRelationship => None,
        }
    }
}

impl From<zip::result::ZipError> for OpcError {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Zip(e)
    }
}

impl From<std::io::Error> for OpcError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<quick_xml::Error> for OpcError {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e)
    }
}

impl From<quick_xml::events::attributes::AttrError> for OpcError {
    fn from(e: quick_xml::events::attributes::AttrError) -> Self {
        Self::XmlAttr(e)
    }
}

impl From<std::str::Utf8Error> for OpcError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8(e)
    }
}
