// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2025 tensorbee
//
// SPDX-License-Identifier: AGPL-3.0-only

//! OPC package — the zip-backed parts, relationships and content types of
//! an OOXML package.
//!
//! Port of rdocx-opc 0.2.0's `package.rs` (MIT; preserved notice in
//! `LICENSES/LicenseRef-rdocx-opc-MIT.txt`), trimmed to the surface
//! [`super::PartFs`] wraps and adapted to zip 9 (`ZipFile::name` now
//! returns a result). Parts and part-level relationships are keyed with a
//! leading slash (`/word/document.xml`).

use std::collections::HashMap;
use std::io::{Read, Seek};

use zip::read::ZipArchive;

use super::content_types::ContentTypes;
use super::error::OpcError;
use super::relationship::Relationships;

/// The `officeDocument` package relationship's type URI: the one pointing at
/// the main document part.
const DOCUMENT_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument";

/// An in-memory representation of an OPC package (ZIP archive).
#[derive(Debug, Clone)]
pub struct OpcPackage {
    /// Content types from `[Content_Types].xml`.
    pub content_types: ContentTypes,
    /// Package-level relationships from `_rels/.rels`.
    pub package_rels: Relationships,
    /// Part-level relationships keyed by the part they belong to
    /// (e.g. `/word/document.xml`).
    pub part_rels: HashMap<String, Relationships>,
    /// All parts keyed by their URI (e.g. `/word/document.xml`); the
    /// content-types and relationships parts are not included.
    pub parts: HashMap<String, Vec<u8>>,
}

impl OpcPackage {
    /// Open an OPC package from a reader holding zip bytes.
    ///
    /// # Errors
    ///
    /// [`OpcError::PartNotFound`] when `[Content_Types].xml` is missing;
    /// the zip and quick-xml errors for anything unreadable or unparseable.
    pub fn from_reader<R: Read + Seek>(reader: R) -> Result<Self, OpcError> {
        let mut archive = ZipArchive::new(reader)?;
        let mut raw_parts: HashMap<String, Vec<u8>> = HashMap::new();

        // Read all entries from the ZIP
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            if entry.is_dir() {
                continue;
            }
            let name = entry.name()?.to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;
            raw_parts.insert(name, data);
        }

        // Parse [Content_Types].xml
        let ct_xml = raw_parts
            .get("[Content_Types].xml")
            .ok_or_else(|| OpcError::PartNotFound("[Content_Types].xml".into()))?;
        let content_types = ContentTypes::from_xml(ct_xml)?;

        // Parse package-level relationships: _rels/.rels
        let package_rels = if let Some(rels_xml) = raw_parts.get("_rels/.rels") {
            Relationships::from_xml(rels_xml)?
        } else {
            Relationships::new()
        };

        // Parse all part-level .rels files
        let mut part_rels = HashMap::new();
        let rels_entries: Vec<String> = raw_parts
            .keys()
            .filter(|k| k.ends_with(".rels") && *k != "_rels/.rels")
            .cloned()
            .collect();

        for rels_path in rels_entries {
            if let Some(xml_data) = raw_parts.get(&rels_path) {
                let rels = Relationships::from_xml(xml_data)?;
                // Convert rels path to the part name it belongs to.
                // e.g. "word/_rels/document.xml.rels" → "/word/document.xml"
                let part_name = rels_path_to_part_name(&rels_path);
                part_rels.insert(part_name, rels);
            }
        }

        // Build parts map with leading "/" normalized
        let mut parts = HashMap::new();
        for (name, data) in &raw_parts {
            if name == "[Content_Types].xml" || name == "_rels/.rels" || name.ends_with(".rels") {
                continue;
            }
            let normalized = if name.starts_with('/') {
                name.clone()
            } else {
                format!("/{name}")
            };
            parts.insert(normalized, data.clone());
        }

        Ok(OpcPackage {
            content_types,
            package_rels,
            part_rels,
            parts,
        })
    }

    /// Get raw bytes of a part by its URI.
    #[must_use]
    pub fn get_part(&self, part_name: &str) -> Option<&[u8]> {
        self.parts.get(part_name).map(Vec::as_slice)
    }

    /// Set (or replace) a part's raw bytes.
    pub fn set_part(&mut self, part_name: &str, data: Vec<u8>) {
        self.parts.insert(part_name.to_string(), data);
    }

    /// Get the relationships of a specific part.
    #[must_use]
    pub fn get_part_rels(&self, part_name: &str) -> Option<&Relationships> {
        self.part_rels.get(part_name)
    }

    /// Get or create the relationships of a specific part. A created
    /// collection is the derived `Default` (rId0 numbering), not `new`.
    pub fn get_or_create_part_rels(&mut self, part_name: &str) -> &mut Relationships {
        self.part_rels.entry(part_name.to_string()).or_default()
    }

    /// Resolve the target URI of a relationship relative to its source part.
    ///
    /// `.` and `..` segments are collapsed, so a target such as
    /// `../media/image1.png` on `/word/charts/chart1.xml` resolves to
    /// `/word/media/image1.png` and matches the key it is stored under.
    #[must_use]
    pub fn resolve_rel_target(source_part: &str, rel_target: &str) -> String {
        let joined = if rel_target.starts_with('/') {
            rel_target.to_string()
        } else {
            // Directory of the source part, including the trailing slash.
            let dir = source_part
                .rfind('/')
                .and_then(|pos| source_part.get(..=pos))
                .unwrap_or("/");
            format!("{dir}{rel_target}")
        };
        normalize_part_name(&joined)
    }

    /// Find the main document part URI (leading slash) through the package
    /// relationships.
    #[must_use]
    pub fn main_document_part(&self) -> Option<String> {
        self.package_rels.get_by_type(DOCUMENT_REL).map(|rel| {
            if rel.target.starts_with('/') {
                rel.target.clone()
            } else {
                format!("/{}", rel.target)
            }
        })
    }
}

/// Collapse `.` and `..` segments in an absolute part name.
///
/// A `..` that would escape the package root is dropped, matching how OPC
/// consumers treat over-long parent traversals.
fn normalize_part_name(path: &str) -> String {
    if !path.contains("./") && !path.ends_with("/.") && !path.ends_with("/..") {
        return path.to_string();
    }

    let mut segments: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            other => segments.push(other),
        }
    }

    let mut out = String::with_capacity(path.len());
    for segment in segments {
        out.push('/');
        out.push_str(segment);
    }
    if out.is_empty() { "/".to_string() } else { out }
}

/// Convert a .rels file path to the part name it belongs to.
/// e.g. "word/_rels/document.xml.rels" → "/word/document.xml"
fn rels_path_to_part_name(rels_path: &str) -> String {
    // Strip the ".rels" suffix once (`trim_end_matches` would strip repeats,
    // turning "a.rels.rels" into "a") and drop only the "_rels" path segment
    // that directly precedes the file name.
    let without_suffix = rels_path.strip_suffix(".rels").unwrap_or(rels_path);
    let path = without_suffix.rsplit_once("_rels/").map_or_else(
        || without_suffix.to_string(),
        |(before, after)| format!("{before}{after}"),
    );
    if path.starts_with('/') {
        path
    } else {
        format!("/{path}")
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn rels_path_conversion() {
        assert_eq!(
            rels_path_to_part_name("word/_rels/document.xml.rels"),
            "/word/document.xml"
        );
        // A part whose own name ends in ".rels" must keep that segment.
        assert_eq!(
            rels_path_to_part_name("word/_rels/odd.rels.rels"),
            "/word/odd.rels"
        );
    }

    #[test]
    fn resolve_relative_target() {
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/document.xml", "styles.xml"),
            "/word/styles.xml"
        );
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/document.xml", "/word/styles.xml"),
            "/word/styles.xml"
        );
        // Charts and headers routinely reference media through a parent dir.
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/charts/chart1.xml", "../media/image1.png"),
            "/word/media/image1.png"
        );
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/document.xml", "./styles.xml"),
            "/word/styles.xml"
        );
        assert_eq!(
            OpcPackage::resolve_rel_target("/word/document.xml", "../../../etc/passwd"),
            "/etc/passwd"
        );
    }
}
