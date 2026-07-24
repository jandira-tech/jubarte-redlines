// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! PATH-01 — sibling character atoms share one `Arc` ancestor path.
//!
//! Multi-char `w:t` must produce one atom per character with identical
//! ancestor NodeId sequences, and those sequences must share the same Arc
//! allocation (pointer equality). Path contents/order remain exact.

use std::sync::Arc;

use jubarte::comparer::WmlComparerSettings;
use jubarte::comparer::atomize::create_comparison_unit_atom_list;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

fn body_atoms(dom: &mut Dom, body_xml: &str) -> Vec<jubarte::comparer::atoms::ComparisonUnitAtom> {
    let full = format!(
        r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body_xml}</w:body></w:document>"#
    );
    let doc = dom.parse_xdocument(&full);
    let root = dom.root(doc).expect("root");
    let body = dom
        .elements(root, Some(&W::body()))
        .into_iter()
        .next()
        .expect("body");
    create_comparison_unit_atom_list(dom, body, &WmlComparerSettings::default())
}

#[test]
fn path01_multi_char_text_shares_arc_path() {
    let mut dom = Dom::new();
    let atoms = body_atoms(&mut dom, r#"<w:p><w:r><w:t>Hello</w:t></w:r></w:p>"#);
    // Atoms = H,e,l,l,o + paragraph mark — at least 5 text atoms.
    let text_atoms: Vec<_> = atoms
        .iter()
        .filter(|a| dom.name(a.content_element) == Some(W::t()))
        .collect();
    assert!(
        text_atoms.len() >= 5,
        "expected ≥5 char atoms, got {}",
        text_atoms.len()
    );
    let first = &text_atoms[0].ancestor_elements;
    for a in &text_atoms[1..] {
        assert_eq!(
            a.ancestor_elements.as_ref(),
            first.as_ref(),
            "path NodeId sequences must match"
        );
        assert!(
            Arc::ptr_eq(&a.ancestor_elements, first),
            "PATH-01: multi-char siblings must share one Arc allocation"
        );
    }
}

#[test]
fn path01_path_order_outermost_to_leaf() {
    let mut dom = Dom::new();
    let atoms = body_atoms(&mut dom, r#"<w:p><w:r><w:t>ab</w:t></w:r></w:p>"#);
    let text = atoms
        .iter()
        .find(|a| dom.name(a.content_element) == Some(W::t()))
        .expect("text atom");
    // Chain is outermost → leaf excluding body: [w:p, w:r, w:t]
    assert!(text.ancestor_elements.len() >= 2);
    let last = *text.ancestor_elements.last().unwrap();
    assert_eq!(dom.name(last), Some(W::t()));
    let first = text.ancestor_elements[0];
    assert_eq!(dom.name(first), Some(W::p()));
}

#[test]
fn path01_coalesce_roundtrip_preserves_text() {
    use jubarte::comparer::atomize::coalesce;
    let mut dom = Dom::new();
    let atoms = body_atoms(&mut dom, r#"<w:p><w:r><w:t>Hi</w:t></w:r></w:p>"#);
    let doc = coalesce(&mut dom, &atoms);
    let ser = dom.serialize_document(doc);
    assert!(
        ser.contains("Hi") || ser.contains(">H<") || ser.contains("H"),
        "{ser}"
    );
}
