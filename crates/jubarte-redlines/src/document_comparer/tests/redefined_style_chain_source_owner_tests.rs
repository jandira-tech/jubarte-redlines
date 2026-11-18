// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn stylesheet(
    count: usize,
    cycle: bool,
    revised: bool,
    default: Option<&str>,
    leaf_metrics: &str,
) -> String {
    let defaults = if revised { "DefaultB" } else { "DefaultA" };
    let inherited = if revised { "InheritedB" } else { "InheritedA" };
    let history = if revised {
        ""
    } else {
        "<w:pPr><w:pPrChange w:id='91' w:author='Authored owner' w:date='2000-01-01T00:00:00Z'><w:pPr><w:keepNext/><w:spacing w:after='180'/></w:pPr></w:pPrChange></w:pPr>"
    };
    let mut xml = format!(
        "<w:styles xmlns:w='{}'><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii='{defaults}' w:hAnsi='SharedFont'/><w:sz w:val='22'/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type='paragraph' w:styleId='Leaf'{}><w:name w:val='Authored clause'/><w:basedOn w:val='Owner1'/>{history}{leaf_metrics}</w:style>",
        W::URI,
        default
            .map(|v| format!(" w:default='{v}'"))
            .unwrap_or_default()
    );
    for index in 1..count {
        let based = if index + 1 < count {
            format!("<w:basedOn w:val='Owner{}'/>", index + 1)
        } else if cycle {
            "<w:basedOn w:val='Leaf'/>".to_owned()
        } else {
            String::new()
        };
        let metrics = if index + 1 == count {
            format!(
                "<w:rPr><w:rFonts w:ascii='{inherited}'/><w:i/><w:color w:val='123456'/></w:rPr>"
            )
        } else {
            String::new()
        };
        xml.push_str(&format!("<w:style w:type='paragraph' w:styleId='Owner{index}'><w:name w:val='Parent{index}'/>{based}{metrics}</w:style>"));
    }
    xml.push_str("</w:styles>");
    xml
}

fn root(dom: &mut Dom, xml: &str) -> NodeId {
    let document = dom.parse_xdocument(xml);
    dom.root(document).unwrap()
}

#[test]
fn redefined_style_metrics_bound_cycles_depth_and_default_owners_without_source_history_loss() {
    for (count, cycle, default) in [
        (11, false, None),
        (12, false, None),
        (13, false, None),
        (3, true, None),
        (3, false, Some("1")),
        (3, false, Some("true")),
        (3, false, Some("0")),
    ] {
        let mut dom = Dom::new();
        let output = root(&mut dom, &stylesheet(count, cycle, false, default, ""));
        let revised = root(&mut dom, &stylesheet(count, cycle, true, default, ""));
        let revised_before = dom.serialize_element(revised);
        let excluded_default = matches!(default, Some("1" | "true"));
        assert_eq!(
            resolve_redefined_style_metrics(&mut dom, output, revised),
            !excluded_default,
            "depth={count} cycle={cycle} default={default:?}"
        );
        // The twelve-owner walk includes Leaf. A thirteenth owner's
        // authored declaration therefore remains outside the lookup;
        // the actual revised docDefaults supply the effective font.
        let font = if count > 12 { "DefaultB" } else { "InheritedB" };
        let leaf_metrics = if excluded_default {
            String::new()
        } else {
            format!("<w:rPr><w:rFonts w:ascii='{font}'/></w:rPr>")
        };
        let expected = root(
            &mut dom,
            &stylesheet(count, cycle, false, default, &leaf_metrics),
        );
        assert_eq!(
            dom.serialize_element(output),
            dom.serialize_element(expected),
            "all original parents, exact authored history and revised effective font delta must survive"
        );
        assert_eq!(
            dom.serialize_element(revised),
            revised_before,
            "revised source stylesheet must remain byte-stable"
        );
        if !cycle {
            let once = dom.serialize_element(output);
            assert!(!resolve_redefined_style_metrics(&mut dom, output, revised));
            assert_eq!(
                dom.serialize_element(output),
                once,
                "no duplicate history or repeated metric writes"
            );
        }
    }
}
