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
fn public_parser_validates_records_that_redeclare_the_default_namespace() {
    // A record-local default-namespace declaration unbinds the OPC namespace
    // the reader wraps records in, and upstream skips a record outside it
    // without validating its attributes. The record is read in the OPC
    // namespace whatever its own declaration says, as the old unprefixed
    // matcher did (#1069: the public parser keeps its old refusal).
    for xmlns in [r#" xmlns="""#, " xmlns='http://example.com/'"] {
        let missing = format!(r#"<Relationships><Relationship{xmlns} Id="rId1"/></Relationships>"#);
        assert!(
            matches!(
                Relationships::from_xml(missing.as_bytes()),
                Err(OpcError::InvalidRelationship)
            ),
            "{missing}"
        );
        let complete = format!(
            r#"<Relationships><Relationship{xmlns} Id="rId7" Type="image" Target="media/a.png"/></Relationships>"#
        );
        let rels = Relationships::from_xml(complete.as_bytes()).unwrap();
        assert_eq!(rels.items.len(), 1, "{complete}");
        assert_eq!(rels.items[0].id, "rId7");
        assert_eq!(rels.items[0].rel_type, "image");
        assert_eq!(rels.items[0].target, "media/a.png");
    }
}

#[test]
fn public_parser_reports_xml_syntax_errors_as_xml_errors() {
    // The record scanner reads with the quick-xml version `OpcError::Xml`
    // carries (oxml-opc 0.14.2 uses 0.41): a syntax error surfaces as
    // `OpcError::Xml`, not as an I/O refusal that a `Xml(_)` match misses.
    assert!(matches!(
        Relationships::from_xml(b"<Relationships><x></y></Relationships>"),
        Err(OpcError::Xml(_))
    ));
}

#[test]
fn public_parser_does_not_repair_mismatched_xml() {
    // `fold_explicit_closes` replaced `></Relationship>` textually, turning
    // `<Bad></Relationship>` into `<Bad/>` before any reader saw it, so the
    // public parser accepted XML whose end tags do not match. The reader
    // itself must stay the validator (#1069 kept the old refusal).
    assert!(matches!(
        Relationships::from_xml(b"<Relationships><Bad></Relationship></Relationships>"),
        Err(OpcError::Xml(_))
    ));
}

#[test]
fn explicit_closes_fold_without_touching_attribute_values() {
    // `>` is legal unescaped in an attribute value, and the folded close may
    // also appear there as entities: the fold happens between events, never
    // as a text replacement, so both targets survive verbatim.
    let xml = br#"<Relationships><Relationship Id="rId1" Type="t" Target="a&gt;&lt;/Relationship&gt;"></Relationship><Relationship Id="rId2" Type="t" Target="b></Relationship>c"></Relationship></Relationships>"#;
    let rels = Relationships::from_xml(xml).unwrap();
    let targets: Vec<&str> = rels.items.iter().map(|r| r.target.as_str()).collect();
    assert_eq!(targets, ["a></Relationship>", "b></Relationship>c"]);
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
