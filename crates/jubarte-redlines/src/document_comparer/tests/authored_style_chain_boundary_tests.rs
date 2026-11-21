// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::collections::{BTreeMap, HashMap};
fn parse(content: &str) -> (Dom, NodeId, HashMap<String, NodeId>) {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&format!(
        "<w:styles xmlns:w='{}'>{content}</w:styles>",
        W::URI
    ));
    let root = dom.root(document).unwrap();
    let ids = dom
        .elements(root, Some(&W::name("style")))
        .into_iter()
        .map(|node| {
            (
                dom.attribute(node, &W::style_id()).unwrap().to_owned(),
                node,
            )
        })
        .collect();
    (dom, root, ids)
}
fn leaf(local: &str, value: &str) -> String {
    format!(
        "{}\u{1}{}={value}",
        W::name(local).clark(),
        W::val().clark()
    )
}
fn attrs(values: &[(&str, &str)]) -> String {
    let mut values = values
        .iter()
        .map(|(name, value)| format!("{}={value}", W::name(name).clark()))
        .collect::<Vec<_>>();
    values.sort();
    values.join("\u{1}")
}
#[test]
fn inherited_style_font_alternatives_and_missing_blocks_keep_exact_declarations() {
    let defaults = "<w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:asciiTheme='minorHAnsi' w:hAnsiTheme='minorHAnsi'/><w:sz w:val='22'/></w:rPr></w:rPrDefault></w:docDefaults>";
    for has_parent_block in [false, true] {
        let parent = if has_parent_block {
            "<w:rPr><w:rFonts w:ascii='ParentFont'/><w:i/></w:rPr>"
        } else {
            ""
        };
        let (dom, root, ids) = parse(&format!(
            "{defaults}<w:style w:type='paragraph' w:styleId='Base'><w:name w:val='Base'/>{parent}</w:style><w:style w:type='paragraph' w:styleId='Child'><w:name w:val='Child'/><w:basedOn w:val='Base'/><w:rPr><w:rFonts w:ascii='AuthoredFont'/><w:color w:val='123456'/><w:rPrChange w:id='7' w:author='Old owner' w:date='2001-02-03T04:05:06Z'><w:rPr><w:b/></w:rPr></w:rPrChange></w:rPr></w:style>"
        ));
        let before = dom.serialize_element(root);
        let mut expected = BTreeMap::from([
            (
                W::name("rFonts").clark(),
                attrs(&[("ascii", "AuthoredFont"), ("hAnsiTheme", "minorHAnsi")]),
            ),
            (W::name("sz").clark(), leaf("sz", "22")),
            (W::name("color").clark(), leaf("color", "123456")),
        ]);
        if has_parent_block {
            expected.insert(W::name("i").clark(), W::name("i").clark());
        }
        assert_eq!(
            effective_chain_props(&dom, root, &ids, Some(ids["Child"]), "rPr", "rPrDefault"),
            expected
        );
        assert_eq!(
            effective_chain_props(&dom, root, &ids, None, "rPr", "rPrDefault"),
            BTreeMap::from([
                (
                    W::name("rFonts").clark(),
                    attrs(&[("asciiTheme", "minorHAnsi"), ("hAnsiTheme", "minorHAnsi")])
                ),
                (W::name("sz").clark(), leaf("sz", "22"))
            ])
        );
        assert_eq!(
            dom.serialize_element(root),
            before,
            "resolving formatting may not mutate old history or source declarations"
        );
    }
}
#[test]
fn based_on_cycles_stop_after_unique_authored_owners_with_nearest_properties_winning() {
    for cycle in [false, true] {
        let base = if cycle {
            "<w:basedOn w:val='Leaf'/>"
        } else {
            ""
        };
        let (dom, root, ids) = parse(&format!(
            "<w:style w:type='paragraph' w:styleId='Leaf'><w:name w:val='Leaf'/><w:basedOn w:val='Base'/><w:rPr><w:color w:val='123456'/></w:rPr></w:style><w:style w:type='paragraph' w:styleId='Base'><w:name w:val='Base'/>{base}<w:rPr><w:i/><w:color w:val='ABCDEF'/></w:rPr></w:style>"
        ));
        let before = dom.serialize_element(root);
        assert_eq!(
            effective_chain_props(&dom, root, &ids, Some(ids["Leaf"]), "rPr", "rPrDefault"),
            BTreeMap::from([
                (W::name("i").clark(), W::name("i").clark()),
                (W::name("color").clark(), leaf("color", "123456"))
            ])
        );
        assert_eq!(dom.serialize_element(root), before);
    }
}
#[test]
fn bounded_style_walk_reads_exactly_the_nearest_thirty_two_owners() {
    for count in [31usize, 32, 33] {
        let mut content = String::new();
        for i in 0..count {
            let base = if i + 1 < count {
                format!("<w:basedOn w:val='Owner{}'/>", i + 1)
            } else {
                String::new()
            };
            let props = if i == 0 {
                "<w:color w:val='123456'/>"
            } else if i == 31 {
                "<w:i/>"
            } else if i == 32 {
                "<w:b/>"
            } else {
                ""
            };
            content.push_str(&format!("<w:style w:type='paragraph' w:styleId='Owner{i}'><w:name w:val='Owner {i}'/>{base}<w:rPr>{props}</w:rPr></w:style>"));
        }
        let (dom, root, ids) = parse(&content);
        let before = dom.serialize_element(root);
        let mut expected = BTreeMap::from([(W::name("color").clark(), leaf("color", "123456"))]);
        if count >= 32 {
            expected.insert(W::name("i").clark(), W::name("i").clark());
        }
        assert_eq!(
            effective_chain_props(&dom, root, &ids, Some(ids["Owner0"]), "rPr", "rPrDefault"),
            expected,
            "chain count {count}"
        );
        assert_eq!(dom.serialize_element(root), before);
    }
}
