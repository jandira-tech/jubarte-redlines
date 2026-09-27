// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Early-exit traversal must retain the DOM's namespace and document-order semantics.

use jubarte::xmllinq::{Dom, NodeId, XName};

fn tree() -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(
        r#"<r:hit xmlns:r="urn:right" xmlns:x="urn:other"><group>text<!--skip--><x:hit/><r:hit id="first"><r:hit id="nested"/></r:hit></group><r:hit id="last"/></r:hit>"#,
    );
    let root = dom.root(doc).unwrap();
    (dom, root)
}

#[test]
fn filtered_search_descends_through_nonmatching_parents_and_stops_at_first_match() {
    let (dom, root) = tree();
    let mut visited = Vec::new();
    let found = dom.find_descendant_element(root, Some(&XName::get("hit", "urn:right")), |n| {
        visited.push(n);
        true
    });
    let expected = dom.descendants(root, Some(&XName::get("hit", "urn:right")))[0];
    assert_eq!(found, Some(expected));
    assert_eq!(visited, [expected]);
    assert_eq!(
        dom.attribute(expected, &XName::get("id", "")),
        Some("first")
    );
}

#[test]
fn rejected_parent_does_not_prune_its_matching_children() {
    let (dom, root) = tree();
    let filter = XName::get("hit", "urn:right");
    let expected = dom.descendants(root, Some(&filter));
    let mut visited = Vec::new();
    let found = dom.find_descendant_element(root, Some(&filter), |n| {
        visited.push(n);
        dom.attribute(n, &XName::get("id", "")) == Some("nested")
    });
    assert_eq!(found, Some(expected[1]));
    assert_eq!(visited, expected[..2]);
}

#[test]
fn unsuccessful_search_and_visitor_walk_all_elements_in_preorder() {
    let (dom, root) = tree();
    let expected = dom.descendants(root, None);
    let mut searched = Vec::new();
    assert_eq!(
        dom.find_descendant_element(root, None, |n| {
            searched.push(n);
            false
        }),
        None
    );
    let mut visited = Vec::new();
    dom.for_each_descendant_element(root, None, |n| visited.push(n));
    assert_eq!(searched, expected);
    assert_eq!(visited, expected);
    assert_eq!(expected.len(), 5, "root, text and comments are excluded");
}

#[test]
fn missing_namespace_or_leaf_never_invokes_predicate() {
    let (dom, root) = tree();
    assert_eq!(
        dom.find_descendant_element(root, Some(&XName::get("hit", "urn:missing")), |_| {
            panic!("a nonmatching element reached the predicate")
        }),
        None
    );
    let leaf = *dom.descendants(root, None).last().unwrap();
    assert_eq!(
        dom.find_descendant_element(leaf, None, |_| panic!("leaf has no descendants")),
        None
    );
}
