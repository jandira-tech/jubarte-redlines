// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

#[test]
fn locally_declared_namespace_metadata_never_becomes_a_paragraph_property_delta() {
    for own_spacing in [false, true] {
        let mut dom = Dom::new();
        let source_doc = dom.parse_xdocument(&format!("<w:styles xmlns:w='{}'><w:docDefaults><w:pPrDefault><w:pPr><w:ind xmlns:source='urn:authored:source' w:left='300' w:right='120'/><w:spacing xmlns:source='urn:authored:spacing' w:beforeLines='2' w:afterLines='1'/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type='paragraph' w:styleId='Normal'><w:name w:val='Normal'/></w:style></w:styles>", W::URI));
        let revised_doc = dom.parse_xdocument(&format!("<w:styles xmlns:w='{}'><w:style w:type='paragraph' w:styleId='Normal'><w:name w:val='Normal'/>{}</w:style></w:styles>", W::URI, if own_spacing { "<w:pPr><w:spacing xmlns:revision='urn:authored:revision' w:beforeLines='1'/></w:pPr>" } else { "" }));
        let source = dom.root(source_doc).unwrap();
        let revised = dom.root(revised_doc).unwrap();
        let before = (
            dom.serialize_element(source),
            dom.serialize_element(revised),
        );
        let revised_normal = find_normal_style(&dom, revised);
        let delta = doc_default_ppr_delta(&mut dom, source, revised, revised_normal);
        assert_eq!(delta.elements.len(), 1);
        let (name, neutralizer) = &delta.elements[0];
        assert_eq!(name, "ind");
        let mut attributes = dom.attributes(*neutralizer);
        attributes.sort_by_key(|a| a.0.clark());
        assert_eq!(
            attributes,
            vec![
                (W::name("left"), "0".to_owned()),
                (W::name("right"), "0".to_owned())
            ]
        );
        assert!(dom.nodes(*neutralizer).is_empty());
        assert_eq!(
            delta.spacing,
            vec![
                (
                    "beforeLines".to_owned(),
                    if own_spacing { "1" } else { "0" }.to_owned()
                ),
                ("afterLines".to_owned(), "0".to_owned())
            ]
        );
        assert_eq!(
            (
                dom.serialize_element(source),
                dom.serialize_element(revised)
            ),
            before,
            "all authored namespace declarations and source property values remain unchanged"
        );
    }
}
