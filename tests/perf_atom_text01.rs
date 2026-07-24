// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! ATOM-TEXT-01 — direct text accessor for single-child leaves.
//!
//! `value_str` must match `value` byte-for-byte (Unicode / nested / empty),
//! and the single-text-child path must be borrowable (Cow::Borrowed).

use std::borrow::Cow;

use jubarte::xmllinq::Dom;

fn parse_root(xml: &str) -> (Dom, jubarte::xmllinq::NodeId) {
    let mut d = Dom::new();
    let doc = d.parse_xdocument(xml);
    let root = d.root(doc).expect("root");
    (d, root)
}

#[test]
fn atom_text01_value_str_matches_value_simple_t() {
    let (d, root) = parse_root(
        r#"<w:t xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">Z</w:t>"#,
    );
    assert_eq!(d.value(root), "Z");
    assert_eq!(d.value_str(root).as_ref(), "Z");
    assert!(
        matches!(d.value_str(root), Cow::Borrowed(_)),
        "single text child must borrow"
    );
}

#[test]
fn atom_text01_value_str_matches_value_unicode() {
    let (d, root) = parse_root(r#"<t>café 日本語 😀</t>"#);
    let v = d.value(root);
    assert_eq!(d.value_str(root).as_ref(), v);
    assert!(matches!(d.value_str(root), Cow::Borrowed(_)));
}

#[test]
fn atom_text01_nested_elements_still_concat() {
    // Nested element children force the owned path (not a single direct Text).
    let (d, root) = parse_root(r#"<r><t>a</t><t>b</t></r>"#);
    assert_eq!(d.value(root), "ab");
    assert_eq!(d.value_str(root).as_ref(), "ab");
    assert!(
        matches!(d.value_str(root), Cow::Owned(_)),
        "multi-child must own concatenated text"
    );
}

#[test]
fn atom_text01_empty_element() {
    let (d, root) = parse_root(r#"<t/>"#);
    assert_eq!(d.value(root), "");
    assert_eq!(d.value_str(root).as_ref(), "");
}

#[test]
fn atom_text01_whitespace_and_entities() {
    let (d, root) = parse_root(r#"<t xml:space="preserve"> a&amp;b </t>"#);
    assert_eq!(d.value(root), " a&b ");
    assert_eq!(d.value_str(root).as_ref(), " a&b ");
    assert!(matches!(d.value_str(root), Cow::Borrowed(_)));
}

#[test]
fn atom_text01_extract_block_equivalent_via_push() {
    // Mirrors moves::extract_text_from_atom_block push path.
    let (d, root) = parse_root(r#"<p><t>H</t><t>i</t></p>"#);
    let mut via_value = String::new();
    let mut via_str = String::new();
    // walk children of p
    for i in 0..d.child_count(root) {
        let c = d.child_at(root, i);
        if d.is_element(c) {
            via_value.push_str(&d.value(c));
            via_str.push_str(&d.value_str(c));
        }
    }
    assert_eq!(via_value, via_str);
    assert_eq!(via_str, "Hi");
}
