// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! M4.F.6 — FixUp*Ids tests.

mod common;

use jubarte::comparer::fixups::{
    fix_up_doc_pr_ids, fix_up_drawing_ids, fix_up_group_ids, fix_up_shape_ids,
    fix_up_shape_type_ids,
};
use jubarte::namespaces::{O, VML, W, WP};
use jubarte::xmllinq::{Dom, XName};

fn id() -> XName {
    XName::get("id", "")
}

#[test]
fn m4_f6_docpr_ids() {
    let mut d = Dom::new();
    let body = d.new_element(W::body());
    for old in ["7", "7", "99"] {
        let dp = d.new_element(WP::name("docPr"));
        d.set_attribute_value(dp, &id(), Some(old));
        d.add(body, dp);
    }
    fix_up_doc_pr_ids(&mut d, body);
    let ids: Vec<_> = d
        .descendants(body, Some(&WP::name("docPr")))
        .iter()
        .map(|&e| d.attribute(e, &id()).unwrap().to_string())
        .collect();
    assert_eq!(ids, vec!["1", "2", "3"]);
}

#[test]
fn m4_f6_shape_ids_relink_ole() {
    let mut d = Dom::new();
    let pict = d.new_element(W::name("pict"));
    let shape = d.new_element(VML::name("shape"));
    d.set_attribute_value(shape, &id(), Some("_x0000_s1026"));
    let ole = d.new_element(O::name("OLEObject"));
    d.set_attribute_value(ole, &XName::get("ShapeID", ""), Some("_x0000_s1026"));
    let reserved = vml(&mut d, "rect", &[("id", "_x0000_s1026")]);
    d.add(pict, reserved);
    d.add(pict, shape);
    d.add(pict, ole);
    fix_up_shape_ids(&mut d, pict);
    let new_shape_id = d.attribute(shape, &id()).unwrap().to_string();
    assert_eq!(new_shape_id, "_x0000_s1026_1");
    assert_eq!(
        d.attribute(ole, &XName::get("ShapeID", "")).unwrap(),
        "_x0000_s1026_1",
        "OLEObject ShapeID relinked"
    );
}

#[test]
fn m4_f6_shapetype_relink() {
    let mut d = Dom::new();
    let pict = d.new_element(W::name("pict"));
    let st = d.new_element(VML::name("shapetype"));
    d.set_attribute_value(st, &id(), Some("t75"));
    let shape = d.new_element(VML::name("shape"));
    d.set_attribute_value(shape, &XName::get("type", ""), Some("#t75"));
    let reserved = vml(&mut d, "rect", &[("id", "t75")]);
    d.add(pict, reserved);
    d.add(pict, st);
    d.add(pict, shape);
    fix_up_shape_type_ids(&mut d, pict);
    assert_eq!(d.attribute(st, &id()).unwrap(), "t75_1");
    assert_eq!(
        d.attribute(shape, &XName::get("type", "")).unwrap(),
        "#t75_1",
        "shape type relinked"
    );
}

/// A `w:pict` holding `children`, added to `parent`.
fn pict_of(d: &mut Dom, parent: jubarte::xmllinq::NodeId, children: &[jubarte::xmllinq::NodeId]) {
    let pict = d.new_element(W::name("pict"));
    for &c in children {
        d.add(pict, c);
    }
    d.add(parent, pict);
}

fn vml(d: &mut Dom, local: &str, attrs: &[(&str, &str)]) -> jubarte::xmllinq::NodeId {
    let e = d.new_element(VML::name(local));
    for &(name, value) in attrs {
        d.set_attribute_value(e, &XName::get(name, ""), Some(value));
    }
    e
}

/// #1872: a shape in a later `w:pict` names the shapetype written beside
/// the first shape; after a second shapetype with the same id, shapes name
/// that one.
#[test]
fn m4_f6_shapetype_relink_reaches_later_shapes() {
    let mut d = Dom::new();
    let body = d.new_element(W::body());
    let st1 = vml(&mut d, "shapetype", &[("id", "_x0000_t3")]);
    let a = vml(&mut d, "shape", &[("type", "#_x0000_t3")]);
    let b = vml(&mut d, "shape", &[("type", "#_x0000_t3")]);
    let st2 = vml(&mut d, "shapetype", &[("id", "_x0000_t3")]);
    let c = vml(&mut d, "shape", &[("type", "_x0000_t3")]);
    pict_of(&mut d, body, &[st1, a]);
    pict_of(&mut d, body, &[b]);
    pict_of(&mut d, body, &[st2, c]);
    fix_up_shape_type_ids(&mut d, body);
    let ty = XName::get("type", "");
    assert_eq!(d.attribute(st1, &id()), Some("_x0000_t3"));
    assert_eq!(d.attribute(st2, &id()), Some("_x0000_t3_1"));
    assert_eq!(d.attribute(a, &ty), Some("#_x0000_t3"));
    assert_eq!(
        d.attribute(b, &ty),
        Some("#_x0000_t3"),
        "a later pict's shape relinked"
    );
    assert_eq!(
        d.attribute(c, &ty),
        Some("_x0000_t3_1"),
        "the bare form, after the second shapetype"
    );
}

/// #1872: an edit's redline renames only ids that repeat, and relinks what
/// named the renamed one.
#[test]
fn m4_f6_preserve_renames_only_repeated_ids() {
    let mut d = Dom::new();
    let body = d.new_element(W::body());
    for old in ["7", "7", "99"] {
        let dp = d.new_element(WP::name("docPr"));
        d.set_attribute_value(dp, &id(), Some(old));
        d.add(body, dp);
    }
    let st1 = vml(&mut d, "shapetype", &[("id", "_x0000_t75")]);
    let s1 = vml(
        &mut d,
        "shape",
        &[("id", "_x0000_i1025"), ("type", "#_x0000_t75")],
    );
    let st2 = vml(&mut d, "shapetype", &[("id", "_x0000_t75")]);
    let s2 = vml(
        &mut d,
        "shape",
        &[("id", "_x0000_i1025"), ("type", "#_x0000_t75")],
    );
    let ole = d.new_element(O::name("OLEObject"));
    d.set_attribute_value(ole, &XName::get("ShapeID", ""), Some("_x0000_i1025"));
    pict_of(&mut d, body, &[st1, s1]);
    pict_of(&mut d, body, &[st2, s2, ole]);
    fix_up_drawing_ids(&mut d, body, true);
    let ids: Vec<_> = d
        .descendants(body, Some(&WP::name("docPr")))
        .iter()
        .map(|&e| d.attribute(e, &id()).unwrap().to_string())
        .collect();
    assert_eq!(ids, ["7", "100", "99"]);
    let ty = XName::get("type", "");
    assert_eq!(d.attribute(st1, &id()), Some("_x0000_t75"));
    assert_eq!(d.attribute(st2, &id()), Some("_x0000_t75_1"));
    assert_eq!(d.attribute(s1, &id()), Some("_x0000_i1025"));
    assert_eq!(d.attribute(s2, &id()), Some("_x0000_i1025_1"));
    assert_eq!(d.attribute(s1, &ty), Some("#_x0000_t75"));
    assert_eq!(d.attribute(s2, &ty), Some("#_x0000_t75_1"));
    assert_eq!(
        d.attribute(ole, &XName::get("ShapeID", "")),
        Some("_x0000_i1025_1"),
        "the renamed shape's OLE object follows it"
    );
}

#[test]
fn m4_f6_group_ids() {
    // `fix_up_group_ids` renumbers `v:group/@id` from 1 (FixUpGroupIds :5986). It is
    // the consolidate-path helper (not wired into compare/produce — see
    // comparer/mod.rs), so cover it directly here to keep it from rotting.
    let mut d = Dom::new();
    let pict = d.new_element(W::name("pict"));
    for old in ["5", "9", "5"] {
        let g = d.new_element(VML::name("group"));
        d.set_attribute_value(g, &id(), Some(old));
        d.add(pict, g);
    }
    fix_up_group_ids(&mut d, pict);
    let ids: Vec<_> = d
        .descendants(pict, Some(&VML::name("group")))
        .iter()
        .map(|&e| d.attribute(e, &id()).unwrap().to_string())
        .collect();
    assert_eq!(ids, vec!["1", "2", "3"]);
}

/// End-of-produce FixUpDocPrIds: drawing-heavy pairs must not emit duplicate
/// wp:docPr/@id after Word-mode merge/wrap (parity S-dup-docpr-id).
#[test]
fn m4_f6_docpr_ids_unique_after_full_compare() {
    use jubarte::document_comparer::compare_documents;
    use std::collections::HashMap;
    use std::io::Read;
    use zip::ZipArchive;

    let corpus = {
        let root = std::env::var("BENCH_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/corpus/neurotic_docx_bench")
            });
        let c = root.join("corpus/word_based/docx_source");
        if !c.is_dir() {
            eprintln!("skip: bench corpus missing");
            return;
        }
        c
    };
    let corpus = corpus.to_str().unwrap();
    let a = std::fs::read(format!("{corpus}/strict01_sdt_controls.docx"))
        .expect("strict01_sdt_controls fixture");
    let b = std::fs::read(format!("{corpus}/strict01.docx")).expect("strict01 fixture");
    let out = compare_documents(&a, &b, "Test").expect("compare");
    let mut zip = ZipArchive::new(std::io::Cursor::new(out)).unwrap();
    let mut f = zip.by_name("word/document.xml").unwrap();
    let mut xml = String::new();
    f.read_to_string(&mut xml).unwrap();
    // Namespace-agnostic id harvest (mirrors parity_ladder S-dup-docpr-id).
    let mut ids = Vec::new();
    let mut rest = xml.as_str();
    while let Some(i) = rest.find("docPr") {
        let slice = &rest[i..];
        if let Some(id_at) = slice.find("id=\"") {
            let after = &slice[id_at + 4..];
            if let Some(end) = after.find('"') {
                ids.push(after[..end].to_string());
            }
        }
        rest = &rest[i + 4..];
    }
    let mut counts: HashMap<String, usize> = HashMap::new();
    for id in &ids {
        *counts.entry(id.clone()).or_default() += 1;
    }
    let dups: Vec<_> = counts
        .iter()
        .filter(|(_, c)| **c > 1)
        .map(|(k, _)| k.clone())
        .collect();
    assert!(
        dups.is_empty(),
        "duplicate wp:docPr/@id after compare: {dups:?} (all={ids:?})"
    );
}

/// #1800: even a single source shape/type pair used to become id="1" twice.
#[test]
fn m4_f6_vml_ids_unique_through_compare() {
    use common::docx::{docx, para, part_string};
    use jubarte::document_comparer::compare_documents;

    let drawing = |text: &str| {
        format!(
            r##"<w:p><w:r><w:pict xmlns:v="urn:schemas-microsoft-com:vml"><v:shapetype id="_x0000_t202" coordsize="21600,21600"/><v:shape id="Text Box 2" type="#_x0000_t202" style="width:100pt;height:30pt"><v:textbox><w:txbxContent><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p>"##
        )
    };
    let a = docx(&drawing("Original box"));
    let b = docx(&drawing("Changed box"));
    let empty = docx(&para("No drawing"));
    for (label, before, after) in [
        ("single source", &empty, &a),
        ("two sources", &a, &b),
        ("identical input", &a, &a),
    ] {
        let result = compare_documents(before, after, "Test").unwrap();
        let xml = part_string(&result, "word/document.xml").unwrap();
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&xml);
        let root = dom.root(doc).unwrap();
        let nodes = dom.descendants(root, None);
        let mut ids = std::collections::HashSet::new();
        let mut types = std::collections::HashSet::new();
        for &node in &nodes {
            if dom
                .name(node)
                .is_some_and(|n| n.namespace_name() == VML::URI)
                && let Some(value) = dom.attribute(node, &id())
            {
                assert!(
                    ids.insert(value.to_string()),
                    "{label}: duplicate VML id {value:?}: {xml}"
                );
                if dom.name_is(node, &VML::shapetype()) {
                    types.insert(value.to_string());
                }
            }
        }
        assert!(!types.is_empty(), "{label}: missing shapetype: {xml}");
        for node in dom.descendants(root, Some(&VML::shape())) {
            let named = dom.attribute(node, &XName::get("type", "")).unwrap();
            assert!(
                types.contains(named.trim_start_matches('#')),
                "{label}: missing shapetype for {named}: {xml}"
            );
        }
        if label != "two sources" {
            assert!(ids.contains("_x0000_t202"), "source shapetype id preserved");
            assert!(ids.contains("Text Box 2"), "source shape id preserved");
        }
    }
}

/// All VML kinds share an id space; existing suffixes must be reserved before
/// allocating new ids. References before and after repeated definitions bind
/// to the first following or latest preceding definition, respectively.
#[test]
fn m4_f6_vml_collision_space_and_references() {
    for preserve in [false, true] {
        let mut d = Dom::new();
        let body = d.new_element(W::body());
        let reserved = vml(&mut d, "rect", &[("id", "shared_1")]);
        let early = vml(&mut d, "shape", &[("id", "early"), ("type", "#shared")]);
        let rect = vml(&mut d, "rect", &[("id", "shared")]);
        let t1 = vml(&mut d, "shapetype", &[("id", "shared")]);
        let s1 = vml(&mut d, "shape", &[("id", "shared"), ("type", "#shared")]);
        let t2 = vml(&mut d, "shapetype", &[("id", "shared")]);
        let s2 = vml(&mut d, "shape", &[("id", "shared"), ("type", "shared")]);
        let oval = vml(&mut d, "oval", &[("id", "shared"), ("type", "#shared")]);
        let group = vml(&mut d, "group", &[("id", "shared")]);
        let ole = d.new_element(O::ole_object());
        d.set_attribute_value(ole, &XName::get("ShapeID", ""), Some("shared"));
        pict_of(&mut d, body, &[reserved, early, rect, t1, s1]);
        pict_of(&mut d, body, &[t2, s2, ole, oval, group]);
        fix_up_drawing_ids(&mut d, body, preserve);
        let nodes = [reserved, early, rect, t1, s1, t2, s2, oval, group];
        let ids: std::collections::HashSet<_> = nodes
            .iter()
            .map(|&n| d.attribute(n, &id()).unwrap())
            .collect();
        assert_eq!(
            ids.len(),
            nodes.len(),
            "all VML kinds share one id space: {ids:?}"
        );
        assert_eq!(d.attribute(reserved, &id()), Some("shared_1"));
        assert_eq!(d.attribute(early, &id()), Some("early"));
        for (node, target, hash) in [
            (early, t1, "#"),
            (s1, t1, "#"),
            (s2, t2, ""),
            (oval, t2, "#"),
        ] {
            assert_eq!(
                d.attribute(node, &XName::get("type", "")).unwrap(),
                format!("{hash}{}", d.attribute(target, &id()).unwrap())
            );
        }
        assert_eq!(
            d.attribute(ole, &XName::get("ShapeID", "")),
            d.attribute(s2, &id())
        );
        let once = d.serialize_element(body);
        fix_up_drawing_ids(&mut d, body, preserve);
        assert_eq!(d.serialize_element(body), once, "fixup is idempotent");
    }
}
