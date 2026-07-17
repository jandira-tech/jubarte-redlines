// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

mod common;

use jubarte::opc::PartFs;
use jubarte::xmllinq::{Dom, XName, XNamespace};

fn w(local: &str) -> XName {
    XName::get(
        local,
        "http://schemas.openxmlformats.org/wordprocessingml/2006/main",
    )
}

/// Build `<w:p><w:r><w:t>Hi</w:t></w:r></w:p>` and exercise query + mutation.
#[test]
fn arena_build_query_mutate() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "Hi");
    d.add(r, t);
    d.add(p, r);

    // descendants(W::t) finds the single <w:t> and its value is "Hi"
    let ts = d.descendants(p, Some(&w("t")));
    assert_eq!(ts.len(), 1);
    assert_eq!(d.value(ts[0]), "Hi");

    // value of the whole paragraph is also "Hi"
    assert_eq!(d.value(p), "Hi");

    // set an attribute, read it back
    d.set_attribute_value(r, &w("rsidR"), Some("00AB12"));
    assert_eq!(d.attribute(r, &w("rsidR")), Some("00AB12"));

    // setting to None removes it
    d.set_attribute_value(r, &w("rsidR"), None);
    assert_eq!(d.attribute(r, &w("rsidR")), None);

    // ancestors of <w:t> are <w:r>, <w:p>
    let anc = d.ancestors(ts[0], None);
    assert_eq!(anc, vec![r, p]);

    // remove <w:r> → no more <w:t> descendants
    d.remove(r);
    assert_eq!(d.descendants(p, Some(&w("t"))).len(), 0);
    assert_eq!(d.parent(r), None);
}

/// `clone_subtree` deep-copies; mutating the clone leaves the original intact,
/// and Add of an already-parented node clones it (LINQ-to-XML semantics).
#[test]
fn arena_clone_and_add_semantics() {
    let mut d = Dom::new();
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    d.add_text(r, "x");
    d.add(p, r);

    let clone = d.clone_subtree(r);
    assert_eq!(d.parent(clone), None);
    assert_eq!(d.value(clone), "x");
    // independent: mutate clone
    d.set_value(clone, "y");
    assert_eq!(d.value(clone), "y");
    assert_eq!(d.value(r), "x");

    // Add an already-parented node → it is cloned, original keeps its parent
    let p2 = d.new_element(w("p"));
    d.add(p2, r);
    assert_eq!(d.parent(r), Some(p)); // original untouched
    assert_eq!(d.elements(p2, None).len(), 1);
}

/// Annotations are typed and retrievable / removable by type.
#[test]
fn arena_annotations() {
    #[derive(PartialEq, Debug)]
    struct Sha1Hash(String);
    let mut d = Dom::new();
    let e = d.new_element(w("p"));
    d.add_annotation(e, Sha1Hash("abc".into()));
    assert_eq!(d.annotation::<Sha1Hash>(e), Some(&Sha1Hash("abc".into())));
    d.remove_annotations::<Sha1Hash>(e);
    assert_eq!(d.annotation::<Sha1Hash>(e), None);
}

// ── M1.3 parse ──────────────────────────────────────────────────────────────

#[test]
fn parse_resolves_namespaces_and_entities() {
    let mut d = Dom::new();
    let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>A &amp; B &#65; <![CDATA[raw<>]]></w:t></w:r></w:p></w:body></w:document>"#;
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    assert_eq!(d.name(root).unwrap(), w("document"));
    // body/p/r/t resolve in the w namespace
    let ts = d.descendants(root, Some(&w("t")));
    assert_eq!(ts.len(), 1);
    // entity + char ref + CDATA decode correctly
    assert_eq!(d.value(ts[0]), "A & B A raw<>");
}

#[test]
fn parse_default_namespace_applies_to_unprefixed_elements() {
    let mut d = Dom::new();
    let xml = r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels"/></Types>"#;
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    let ct = "http://schemas.openxmlformats.org/package/2006/content-types";
    assert_eq!(d.name(root).unwrap(), XName::get("Types", ct));
    let def = d.elements(root, None);
    assert_eq!(d.name(def[0]).unwrap(), XName::get("Default", ct));
    // attribute is in NO namespace (unprefixed attrs don't inherit default ns)
    assert_eq!(
        d.attribute(def[0], &XName::get("Extension", "")),
        Some("rels")
    );
}

// ── M1.4 serialize ────────────────────────────────────────────────────────────

#[test]
fn serialize_roundtrip_preserves_structure() {
    let mut d = Dom::new();
    let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" mc:Ignorable="w14"><w:body><w:p><w:r><w:t>Hi</w:t></w:r></w:p></w:body></w:document>"#;
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).unwrap();
    let s = d.serialize_element(root);
    // re-parse the serialized output; structure must be preserved
    let mut d2 = Dom::new();
    let doc2 = d2.parse_xdocument(&s);
    let root2 = d2.root(doc2).unwrap();
    assert_eq!(d2.name(root2).unwrap(), w("document"));
    assert_eq!(d2.descendants(root2, Some(&w("t"))).len(), 1);
    assert_eq!(d2.value(root2), "Hi");
    // mc:Ignorable preserved with its w-prefixed value intact
    let mc_ign = XName::get(
        "Ignorable",
        "http://schemas.openxmlformats.org/markup-compatibility/2006",
    );
    assert_eq!(d2.attribute(root2, &mc_ign), Some("w14"));
    // w prefix kept (not reassigned to nsN)
    assert!(s.contains("xmlns:w=\""), "serialized: {s}");
    assert!(s.contains("<w:document"), "serialized: {s}");
}

#[test]
fn serialize_escapes_and_self_closes() {
    let mut d = Dom::new();
    let doc =
        d.parse_xdocument(r#"<root xmlns="urn:x"><a v="x&amp;&quot;y"/><b>1&lt;2</b></root>"#);
    let root = d.root(doc).unwrap();
    let s = d.serialize_element(root);
    assert!(s.contains("v=\"x&amp;&quot;y\""), "serialized: {s}");
    assert!(s.contains("1&lt;2"), "serialized: {s}");
    assert!(s.contains("/>"), "self-closing empty element: {s}");
}

// ── M1.5 OPC layer (rdocx-opc adapter) ────────────────────────────────────────

const ORIGINAL: &[u8] = include_bytes!("fixtures/redline/original.docx");

#[test]
fn opc_open_read_resolve_content_type() {
    let pkg = PartFs::open(ORIGINAL).expect("open docx");
    // byte-level part read
    assert!(pkg.part_bytes("word/document.xml").is_some());
    assert!(
        pkg.part_string("word/document.xml")
            .unwrap()
            .contains("<w:document")
    );
    // relative target resolution
    assert_eq!(
        pkg.resolve_rel_target("word/document.xml", "footnotes.xml"),
        "word/footnotes.xml"
    );
    // content type for the main document part
    let ct = pkg.content_type_for("word/document.xml").unwrap();
    assert!(
        ct.contains("wordprocessingml.document.main+xml"),
        "content type was {ct}"
    );
    // main document part discoverable
    assert_eq!(
        pkg.main_document_part().as_deref(),
        Some("word/document.xml")
    );
}

#[test]
fn opc_set_part_and_repackage_roundtrips() {
    let mut pkg = PartFs::open(ORIGINAL).expect("open docx");
    let names_before = pkg.parts();
    assert!(names_before.iter().any(|n| n == "word/document.xml"));

    // replace a part, repackage, reopen — the change persists
    pkg.set_part("word/document.xml", b"<w:document/>".to_vec());
    let zip = pkg.to_zip().expect("write zip");
    let pkg2 = PartFs::open(&zip).expect("reopen");
    assert_eq!(
        pkg2.part_string("word/document.xml").as_deref(),
        Some("<w:document/>")
    );
    // part set is preserved across the round-trip
    assert_eq!(pkg2.parts(), names_before);
}

// ── M1.8 foundation round-trip gate (real Word document.xml) ───────────────────

/// Parse the real `word/document.xml`, serialize it back, and assert the part is
/// structurally unchanged. This proves the DOM parse↔serialize pair is faithful
/// on a genuine Word document before any algorithm is built on it.
#[test]
fn foundation_roundtrip_document_xml_structural() {
    let pkg = PartFs::open(ORIGINAL).unwrap();
    let xml = pkg.part_string("word/document.xml").unwrap();
    let mut d = Dom::new();
    let doc = d.parse_xdocument(&xml);
    let root = d.root(doc).unwrap();
    let out = d.serialize_element(root);

    // Compare just the document.xml part canonically (other parts are regenerated
    // by the OPC layer and compared separately by the full-package gate).
    common::assert_xml_structurally_eq(&out, &xml, "word/document.xml");
}

// ── M1.6 util: sha1 + group_adjacent ──────────────────────────────────────────

#[test]
fn sha1_hex_known_vector() {
    use jubarte::util::sha1_hex;
    assert_eq!(sha1_hex("abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    assert_eq!(sha1_hex(""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");
}

#[test]
fn group_adjacent_runs() {
    use jubarte::util::group_adjacent;
    let groups = group_adjacent([1, 1, 2, 2, 1], |x| *x);
    let shapes: Vec<(i32, Vec<i32>)> = groups;
    assert_eq!(shapes, vec![(1, vec![1, 1]), (2, vec![2, 2]), (1, vec![1])]);
}

// ── M1.7 namespaces + UnidHelper + WmlDocument ────────────────────────────────

#[test]
fn namespace_constants_match_source() {
    use jubarte::namespaces::{PT, W};
    assert_eq!(
        W::p().clark(),
        "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}p"
    );
    // Faithful to PtOpenXmlUtil.ts:3846-3848 (NOT the plan's claimed URI).
    assert_eq!(
        PT::unid().clark(),
        "{http://powertools.codeplex.com/2011}Unid"
    );
}

#[test]
fn unid_assigns_to_all_descendants() {
    use jubarte::namespaces::PT;
    use jubarte::unid::assign_to_all_elements;
    let mut d = Dom::new();
    let body = d.new_element(w("body"));
    let p = d.new_element(w("p"));
    let r = d.new_element(w("r"));
    let t = d.new_element(w("t"));
    d.add_text(t, "Hi");
    d.add(r, t);
    d.add(p, r);
    d.add(body, p);

    assign_to_all_elements(&mut d, body);
    // every descendant got a Unid
    for desc in d.descendants(body, None) {
        assert!(
            d.attribute(desc, &PT::unid()).is_some(),
            "descendant missing Unid"
        );
    }
    // unids are distinct across two siblings-of-different-content elements
    let unid_p = d.attribute(p, &PT::unid());
    let unid_r = d.attribute(r, &PT::unid());
    assert!(unid_p.is_some() && unid_r.is_some() && unid_p != unid_r);
}

#[test]
fn wml_document_parses_main_document() {
    use jubarte::WmlDocument;
    let mut wml = WmlDocument::from_bytes(ORIGINAL).expect("open");
    assert_eq!(wml.main_document_part_name(), "word/document.xml");
    let root = wml.main_document_root().expect("main root");
    assert_eq!(wml.dom().name(root).unwrap(), w("document"));
    // body present with paragraphs
    let body = wml.dom().element(root, &w("body")).expect("body");
    assert!(!wml.dom().descendants(body, Some(&w("p"))).is_empty());
}

#[test]
fn xname_clark_notation_roundtrip() {
    let w = XNamespace::get("http://schemas.openxmlformats.org/wordprocessingml/2006/main");
    let n = w.name("p");
    assert_eq!(
        n.clark(),
        "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}p"
    );
    assert_eq!(XName::from_clark(&n.clark()), n);
}

#[test]
fn xname_bare_local_has_empty_namespace() {
    let n = XName::from_clark("foo");
    assert_eq!(n.local_name(), "foo");
    assert_eq!(n.namespace_name(), "");
    assert_eq!(n.clark(), "foo");
    assert_eq!(n, XName::get("foo", ""));
}

#[test]
fn xnamespace_well_known_singletons() {
    assert_eq!(XNamespace::none().namespace_name(), "");
    assert_eq!(
        XNamespace::xmlns().namespace_name(),
        "http://www.w3.org/2000/xmlns/"
    );
    assert_eq!(
        XNamespace::xml().namespace_name(),
        "http://www.w3.org/XML/1998/namespace"
    );
}
