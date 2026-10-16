// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-FileCopyrightText: 2025 tensorbee
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Parsing and writing of `.rels` relationship parts.
//!
//! Port of rdocx-opc 0.2.0's `relationship.rs` (MIT; preserved notice in
//! `LICENSES/LicenseRef-rdocx-opc-MIT.txt`). The parsing semantics are
//! deliberately unchanged, quirks included: only self-closing
//! `<Relationship/>` elements are read (explicit `</Relationship>` closes are
//! folded in beforehand by [`crate::opc::PartFs::open`]), and attribute
//! values are taken as raw bytes, with XML entities still encoded (the
//! package layer decodes them once on the way in).

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::{Reader, Writer};

use super::error::OpcError;

/// A single relationship entry.
#[derive(Debug, Clone, PartialEq)]
pub struct Relationship {
    /// The relationship's `Id` (e.g. `rId7`).
    pub id: String,
    /// The relationship type URI.
    pub rel_type: String,
    /// The target part (relative) or URI (external).
    pub target: String,
    /// `TargetMode="External"` when set.
    pub target_mode: Option<String>,
}

/// A collection of relationships parsed from a `.rels` part.
///
/// The derived [`Default`] starts `add` numbering at `rId0`; Word numbers
/// from `rId1`, so construction goes through [`Relationships::new`].
#[derive(Debug, Clone, Default)]
pub struct Relationships {
    /// The relationships, in file order.
    pub items: Vec<Relationship>,
    /// Next suffix for `add`'s `rIdN` ids.
    next_id: u32,
}

impl Relationships {
    /// An empty collection that numbers new ids from `rId1`, as Word does.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
        }
    }

    /// Parse from XML bytes.
    ///
    /// # Errors
    ///
    /// [`OpcError::InvalidRelationship`] for a `Relationship` element
    /// missing its `Id`, `Type` or `Target`; the quick-xml errors for
    /// malformed XML or attributes.
    pub fn from_xml(xml: &[u8]) -> Result<Self, OpcError> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);

        let mut items = Vec::new();
        let mut max_id: u32 = 0;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == "Relationship" => {
                    let mut id = None;
                    let mut rel_type = None;
                    let mut target = None;
                    let mut target_mode = None;

                    for attr in e.attributes() {
                        let attr = attr?;
                        match attr.key.as_ref() {
                            "Id" => {
                                let val = attr.value.to_string();
                                // Extract numeric suffix for next_id tracking
                                if let Some(num_str) = val.strip_prefix("rId")
                                    && let Ok(n) = num_str.parse::<u32>()
                                {
                                    max_id = max_id.max(n);
                                }
                                id = Some(val);
                            }
                            "Type" => rel_type = Some(attr.value.to_string()),
                            "Target" => target = Some(attr.value.to_string()),
                            "TargetMode" => target_mode = Some(attr.value.to_string()),
                            _ => {}
                        }
                    }

                    match (id, rel_type, target) {
                        (Some(id), Some(rel_type), Some(target)) => {
                            items.push(Relationship {
                                id,
                                rel_type,
                                target,
                                target_mode,
                            });
                        }
                        _ => return Err(OpcError::InvalidRelationship),
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(Relationships {
            items,
            next_id: max_id.saturating_add(1),
        })
    }

    /// Serialize to XML bytes (two-space indent, `Id`/`Type`/`Target`
    /// attributes in that order, `TargetMode` last when set).
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

        let mut rels_start = BytesStart::new("Relationships");
        rels_start.push_attribute((
            "xmlns",
            "http://schemas.openxmlformats.org/package/2006/relationships",
        ));
        writer.write_event(Event::Start(rels_start))?;

        for rel in &self.items {
            let mut elem = BytesStart::new("Relationship");
            elem.push_attribute(("Id", rel.id.as_str()));
            elem.push_attribute(("Type", rel.rel_type.as_str()));
            elem.push_attribute(("Target", rel.target.as_str()));
            if let Some(ref mode) = rel.target_mode {
                elem.push_attribute(("TargetMode", mode.as_str()));
            }
            writer.write_event(Event::Empty(elem))?;
        }

        writer.write_event(Event::End(BytesEnd::new("Relationships")))?;

        Ok(writer.into_inner())
    }

    /// Find a relationship by its ID.
    #[must_use]
    pub fn get_by_id(&self, id: &str) -> Option<&Relationship> {
        self.items.iter().find(|r| r.id == id)
    }

    /// Find the first relationship matching a given type.
    #[must_use]
    pub fn get_by_type(&self, rel_type: &str) -> Option<&Relationship> {
        self.items.iter().find(|r| r.rel_type == rel_type)
    }

    /// Find all relationships matching a given type.
    #[must_use]
    pub fn get_all_by_type(&self, rel_type: &str) -> Vec<&Relationship> {
        self.items
            .iter()
            .filter(|r| r.rel_type == rel_type)
            .collect()
    }

    /// Add a new relationship and return its generated `rIdN` ID.
    pub fn add(&mut self, rel_type: &str, target: &str) -> String {
        let id = format!("rId{}", self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.items.push(Relationship {
            id: id.clone(),
            rel_type: rel_type.to_string(),
            target: target.to_string(),
            target_mode: None,
        });
        id
    }

    /// Add a relationship with a specific ID, replacing any relationship
    /// already under it. The `next_id` counter is updated to avoid future
    /// collisions.
    pub fn add_with_id(&mut self, id: &str, rel_type: &str, target: &str) {
        self.items.retain(|r| r.id != id);
        self.items.push(Relationship {
            id: id.to_string(),
            rel_type: rel_type.to_string(),
            target: target.to_string(),
            target_mode: None,
        });
        if let Some(num) = id.strip_prefix("rId").and_then(|s| s.parse::<u32>().ok())
            && num >= self.next_id
        {
            self.next_id = num.saturating_add(1);
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    const IMAGE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

    #[test]
    fn round_trip_relationships() {
        let mut rels = Relationships::new();
        rels.add(IMAGE, "media/image.png");
        let xml = rels.to_xml().unwrap();
        let mut parsed = Relationships::from_xml(&xml).unwrap();
        assert_eq!(parsed.items, rels.items);
        assert_eq!(parsed.add("hyperlink", "https://a.example/"), "rId2");
    }

    #[test]
    fn derived_default_numbers_from_rid0_new_from_rid1() {
        assert_eq!(Relationships::default().add(IMAGE, "a.png"), "rId0");
        assert_eq!(Relationships::new().add(IMAGE, "a.png"), "rId1");
    }

    #[test]
    fn parse_tracks_max_numeric_suffix() {
        let xml = br#"<Relationships><Relationship Id="rId9" Type="t" Target="a.xml"/><Relationship Id="rId2" Type="t" Target="b.xml"/></Relationships>"#;
        let mut rels = Relationships::from_xml(xml).unwrap();
        assert_eq!(rels.add("t", "c.xml"), "rId10");
        assert_eq!(rels.items.len(), 3);
    }

    #[test]
    fn missing_required_attribute_is_invalid() {
        assert!(matches!(
            Relationships::from_xml(b"<Relationships><Relationship Id='rId1'/></Relationships>"),
            Err(OpcError::InvalidRelationship)
        ));
    }

    #[test]
    fn explicit_close_is_not_parsed_by_the_upstream_semantics() {
        // Kept on purpose: PartFs folds these closes before re-reading.
        let xml =
            br#"<Relationships><Relationship Id="rId1" Type="t" Target="a.xml"></Relationship></Relationships>"#;
        assert!(Relationships::from_xml(xml).unwrap().items.is_empty());
    }
}
