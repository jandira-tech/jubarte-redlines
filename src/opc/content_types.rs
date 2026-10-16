// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2025 tensorbee
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Parsing and writing of `[Content_Types].xml`.
//!
//! Port of rdocx-opc 0.2.0's `content_types.rs` (MIT; preserved notice in
//! `LICENSES/LicenseRef-rdocx-opc-MIT.txt`), trimmed to the surface the
//! package layer uses. Parsing semantics are unchanged: only self-closing
//! `Default`/`Override` elements are read, and attribute values stay
//! raw (the package layer decodes XML entities once on the way in).

use std::collections::HashMap;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use super::error::OpcError;

/// Parsed `[Content_Types].xml`: Default entries by extension and Override
/// entries by part name.
#[derive(Debug, Clone, Default)]
pub struct ContentTypes {
    /// `Default` entries, keyed by extension (e.g. `png`).
    pub defaults: HashMap<String, String>,
    /// `Override` entries, keyed by part name (e.g. `/word/document.xml`).
    pub overrides: HashMap<String, String>,
}

impl ContentTypes {
    /// Parse from XML bytes.
    ///
    /// # Errors
    ///
    /// [`OpcError::InvalidContentTypes`] for a `Default` or `Override`
    /// element missing a required attribute; the quick-xml errors for
    /// malformed XML or attributes.
    pub fn from_xml(xml: &[u8]) -> Result<Self, OpcError> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);

        let mut defaults = HashMap::new();
        let mut overrides = HashMap::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                    "Default" => {
                        let mut ext = None;
                        let mut ct = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                "Extension" => ext = Some(attr.value.to_string()),
                                "ContentType" => ct = Some(attr.value.to_string()),
                                _ => {}
                            }
                        }
                        match (ext, ct) {
                            (Some(e), Some(c)) => {
                                defaults.insert(e, c);
                            }
                            _ => return Err(OpcError::InvalidContentTypes),
                        }
                    }
                    "Override" => {
                        let mut pn = None;
                        let mut ct = None;
                        for attr in e.attributes() {
                            let attr = attr?;
                            match attr.key.as_ref() {
                                "PartName" => pn = Some(attr.value.to_string()),
                                "ContentType" => ct = Some(attr.value.to_string()),
                                _ => {}
                            }
                        }
                        match (pn, ct) {
                            (Some(p), Some(c)) => {
                                overrides.insert(p, c);
                            }
                            _ => return Err(OpcError::InvalidContentTypes),
                        }
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(ContentTypes {
            defaults,
            overrides,
        })
    }

    /// Serialize to XML bytes: defaults then overrides, each sorted by key,
    /// so the same types write the same bytes.
    ///
    /// # Errors
    ///
    /// [`OpcError::Io`] when the underlying writer fails.
    pub fn to_xml(&self) -> Result<Vec<u8>, OpcError> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut types_start = BytesStart::new("Types");
        types_start.push_attribute((
            "xmlns",
            "http://schemas.openxmlformats.org/package/2006/content-types",
        ));
        writer.write_event(Event::Start(types_start))?;

        // Write defaults sorted for deterministic output
        let mut sorted_defaults: Vec<_> = self.defaults.iter().collect();
        sorted_defaults.sort_by_key(|(k, _)| (*k).clone());
        for (ext, ct) in sorted_defaults {
            let mut elem = BytesStart::new("Default");
            elem.push_attribute(("Extension", ext.as_str()));
            elem.push_attribute(("ContentType", ct.as_str()));
            writer.write_event(Event::Empty(elem))?;
        }

        // Write overrides sorted for deterministic output
        let mut sorted_overrides: Vec<_> = self.overrides.iter().collect();
        sorted_overrides.sort_by_key(|(k, _)| (*k).clone());
        for (pn, ct) in sorted_overrides {
            let mut elem = BytesStart::new("Override");
            elem.push_attribute(("PartName", pn.as_str()));
            elem.push_attribute(("ContentType", ct.as_str()));
            writer.write_event(Event::Empty(elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("Types")))?;

        Ok(writer.into_inner())
    }

    /// Look up the content type for a part name: an Override wins, then the
    /// Default for the part's extension.
    #[must_use]
    pub fn content_type_for(&self, part_name: &str) -> Option<&str> {
        // Check overrides first
        if let Some(ct) = self.overrides.get(part_name) {
            return Some(ct.as_str());
        }
        // Fall back to defaults by extension
        part_name
            .rsplit_once('.')
            .and_then(|(_, ext)| self.defaults.get(ext).map(String::as_str))
    }

    /// Add a Default content type for an extension (e.g. `png` →
    /// `image/png`). An existing Default for the extension is kept.
    pub fn add_default(&mut self, extension: &str, content_type: &str) {
        self.defaults
            .entry(extension.to_string())
            .or_insert_with(|| content_type.to_string());
    }

    /// Add an Override content type for a specific part name, replacing any
    /// Override already under it.
    pub fn add_override(&mut self, part_name: &str, content_type: &str) {
        self.overrides
            .insert(part_name.to_string(), content_type.to_string());
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    fn docx_types() -> ContentTypes {
        let mut ct = ContentTypes::default();
        ct.add_default(
            "rels",
            "application/vnd.openxmlformats-package.relationships+xml",
        );
        ct.add_override(
            "/word/document.xml",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        );
        ct
    }

    #[test]
    fn round_trip_content_types() {
        let ct = docx_types();
        let parsed = ContentTypes::from_xml(&ct.to_xml().unwrap()).unwrap();
        assert_eq!(parsed.defaults, ct.defaults);
        assert_eq!(parsed.overrides, ct.overrides);
        assert_eq!(
            parsed.content_type_for("/word/document.xml"),
            Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
            )
        );
        assert_eq!(
            parsed.content_type_for("/word/_rels/document.xml.rels"),
            Some("application/vnd.openxmlformats-package.relationships+xml")
        );
        assert_eq!(parsed.content_type_for("/word/nothing.bin"), None);
    }

    #[test]
    fn add_default_keeps_first_add_override_replaces() {
        let mut ct = docx_types();
        ct.add_default("rels", "text/plain");
        ct.add_override("/word/document.xml", "text/xml");
        assert_eq!(
            ct.defaults.get("rels").map(String::as_str),
            Some("application/vnd.openxmlformats-package.relationships+xml")
        );
        assert_eq!(
            ct.overrides.get("/word/document.xml").map(String::as_str),
            Some("text/xml")
        );
    }

    #[test]
    fn missing_required_attribute_is_invalid() {
        assert!(matches!(
            ContentTypes::from_xml(b"<Types><Default Extension='png'/></Types>"),
            Err(OpcError::InvalidContentTypes)
        ));
        assert!(matches!(
            ContentTypes::from_xml(b"<Types><Override ContentType='text/xml'/></Types>"),
            Err(OpcError::InvalidContentTypes)
        ));
    }

    #[test]
    fn to_xml_writes_sorted_entries_twice_identically() {
        let mut ct = docx_types();
        ct.add_default("png", "image/png");
        ct.add_default("xml", "application/xml");
        assert_eq!(ct.to_xml().unwrap(), ct.to_xml().unwrap());
    }
}
