// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Exercise the public parser directly, without PartFs's admission preflight.
//! The dependency advisory gate guards the patched parser version; these
//! deterministic inputs protect duplicate checks and the existing public API.

use jubarte::opc::{OpcError, Relationship, Relationships};

const NS: &str = "http://schemas.openxmlformats.org/package/2006/relationships";

fn relationship_xml(extra: &str) -> Vec<u8> {
    format!(
        r#"<Relationships xmlns="{NS}"><Relationship Id="rId7" Type="image" Target="media/image.png"{extra}/></Relationships>"#
    )
    .into_bytes()
}

#[test]
fn public_parser_handles_many_distinct_attributes_without_package_preflight() {
    // Larger than the admission cap by design: the public parser is safe
    // through the patched dependency, rather than a PartFs-only guard.
    let extra: String = (0..16_384).map(|i| format!(r#" a{i}="x""#)).collect();
    let mut rels = Relationships::from_xml(&relationship_xml(&extra)).unwrap();
    assert_eq!(
        rels.items,
        [Relationship {
            id: "rId7".into(),
            rel_type: "image".into(),
            target: "media/image.png".into(),
            target_mode: None,
        }]
    );
    assert_eq!(rels.add("hyperlink", "https://example.com/"), "rId8");
    let reparsed = Relationships::from_xml(&rels.to_xml().unwrap()).unwrap();
    assert_eq!(reparsed.items, rels.items);
}

#[test]
fn public_parser_retains_duplicate_checks_after_many_distinct_attributes() {
    let mut extra: String = (0..16_384).map(|i| format!(r#" a{i}="x""#)).collect();
    extra.push_str(r#" a0="again""#);
    assert!(matches!(
        Relationships::from_xml(&relationship_xml(&extra)),
        Err(OpcError::XmlAttr(_))
    ));
}

#[test]
fn public_parser_refuses_duplicate_required_attributes() {
    assert!(matches!(
        Relationships::from_xml(&relationship_xml(r#" Id="rId8""#)),
        Err(OpcError::XmlAttr(_))
    ));
    assert!(matches!(
        Relationships::from_xml(b"<Relationships><Relationship Id='rId1'/></Relationships>"),
        Err(OpcError::InvalidRelationship)
    ));
}

#[test]
fn public_relationship_collection_preserves_lookup_and_edit_contracts() {
    let mut rels = Relationships::new();
    assert_eq!(rels.add("image", "first.png"), "rId1");
    rels.add_with_id("rId9", "image", "second.png");
    rels.add_with_id("rId9", "image", "replacement.png");
    assert_eq!(rels.get_by_id("rId9").unwrap().target, "replacement.png");
    assert_eq!(rels.get_by_type("image").unwrap().id, "rId1");
    assert_eq!(rels.get_all_by_type("image").len(), 2);
    assert_eq!(rels.add("hyperlink", "https://example.com/"), "rId10");
    let parsed = Relationships::from_xml(&rels.to_xml().unwrap()).unwrap();
    assert_eq!(parsed.items, rels.items);
    assert!(parsed.get_by_id("missing").is_none());
}
