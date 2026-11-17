// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "2001-02-03T04:05:06Z";

fn paragraph(kind: &str, text: &str, properties: &str, mark: &str) -> String {
    let leaf = if kind == "del" { "delText" } else { "t" };
    format!(
        "<w:p><w:pPr>{properties}<w:rPr>{mark}<w:{kind} w:id='17' w:author='Prior editor' w:date='{DATE}'/></w:rPr></w:pPr><w:{kind} w:id='18' w:author='Prior editor' w:date='{DATE}'><w:r><w:rPr><w:color w:val='234567'/></w:rPr><w:{leaf}>{text}</w:{leaf}></w:r></w:{kind}></w:p>"
    )
}
fn source(old: &str, new: &str) -> (Dom, NodeId, NodeId) {
    let fragment = format!(
        "{}{}{}{}",
        paragraph("ins", "Introduction", "", "").repeat(2),
        paragraph(
            "ins",
            "1. What this is",
            "<w:pBdr><w:bottom w:val='single'/></w:pBdr><w:spacing w:before='300' w:after='80'/><w:jc w:val='center'/>",
            new
        ),
        paragraph(
            "del",
            "Old Demo",
            "<w:spacing w:line='276' w:lineRule='auto'/><w:ind w:left='720'/><w:jc w:val='both'/>",
            old
        ),
        paragraph("ins", "Revised body", "", "").repeat(4)
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}'><w:body>{fragment}<w:sectPr/></w:body></w:document>",
        W::URI
    ));
    let root = dom.root(doc).unwrap();
    let heading = dom.descendants(root, Some(&W::p()))[2];
    (dom, root, heading)
}
fn projection(dom: &mut Dom, root: NodeId, accept: bool) -> Vec<(String, String)> {
    let copy = dom.clone_subtree(root);
    let copy = if accept {
        crate::revision_processor::accept_revisions_document(dom, copy)
    } else {
        crate::revision_processor::reject_revisions_document(dom, copy)
    };
    for n in dom.descendants(copy, Some(&W::r_pr())) {
        if dom.elements(n, None).is_empty() && !dom.has_attributes(n) {
            dom.remove(n);
        }
    }
    dom.descendants(copy, Some(&W::p()))
        .into_iter()
        .map(|p| {
            let properties = dom
                .element(p, &W::p_pr())
                .map(|n| dom.serialize_element(n))
                .unwrap_or_default();
            (dom.value(p), properties)
        })
        .collect()
}
#[test]
fn title_fold_retains_complete_pilcrow_formats_and_configured_history() {
    for (old, new) in [
        (
            "<w:rFonts w:ascii='Arial' w:hAnsi='Arial'/><w:b/><w:sz w:val='24'/><w:u w:val='single'/>",
            "<w:rFonts w:ascii='Calibri' w:hAnsi='Calibri'/><w:i/><w:color w:val='456789'/><w:sz w:val='28'/>",
        ),
        ("", "<w:b/><w:sz w:val='28'/>"),
    ] {
        for tracking in [false, true] {
            let (mut dom, root, heading) = source(old, new);
            let accepted = projection(&mut dom, root, true);
            let rejected = projection(&mut dom, root, false);
            let before = dom.serialize_element(root);
            let settings = WmlComparerSettings {
                detect_format_changes: tracking,
                author_for_revisions: "Configured editor".into(),
                date_time_for_revisions: "2002-03-04T05:06:07Z".into(),
                ..WmlComparerSettings::default()
            };
            let mut id = 701;
            fold_midstream_demo_title_into_numbered_heading_with_settings(
                &mut dom, root, &settings, &mut id,
            );
            assert_eq!(
                projection(&mut dom, root, true),
                accepted,
                "accepted mark {old} -> {new}"
            );
            assert_eq!(
                projection(&mut dom, root, false),
                rejected,
                "rejected mark {old} -> {new}"
            );
            if !tracking {
                assert_eq!(
                    dom.serialize_element(root),
                    before,
                    "different marks retain separate source paragraphs without format tracking"
                );
                assert_eq!(id, 701);
                assert!(dom.descendants(root, Some(&W::r_pr_change())).is_empty());
                continue;
            }
            assert_eq!(id, 703);
            let ppr = dom.element(heading, &W::p_pr()).unwrap();
            let names: Vec<_> = dom
                .elements(ppr, None)
                .into_iter()
                .map(|n| dom.name(n).unwrap().local_name().to_string())
                .collect();
            assert_eq!(names, ["pBdr", "spacing", "jc", "rPr", "pPrChange"]);
            let mark = dom.element(ppr, &W::r_pr()).unwrap();
            let history = dom.element(mark, &W::r_pr_change()).unwrap();
            assert_eq!(dom.elements(mark, None).last().copied(), Some(history));
            let saved = dom.element(history, &W::r_pr()).unwrap();
            assert!(dom.element(saved, &W::ins()).is_none());
            assert!(dom.element(saved, &W::del()).is_none());
            let pchange = dom.element(ppr, &W::p_pr_change()).unwrap();
            let psaved = dom.element(pchange, &W::p_pr()).unwrap();
            assert!(dom.element(psaved, &W::r_pr()).is_none());
            assert!(dom.element(psaved, &W::sect_pr()).is_none());
            for (change, expected_id) in [(pchange, "701"), (history, "702")] {
                assert_eq!(dom.attribute(change, &W::id()), Some(expected_id));
                assert_eq!(
                    dom.attribute(change, &W::author()),
                    Some("Configured editor")
                );
                assert_eq!(
                    dom.attribute(change, &W::date()),
                    Some("2002-03-04T05:06:07Z")
                );
            }
        }
    }
}
fn effect_projection(dom: &mut Dom, root: NodeId, accept: bool) -> Vec<(String, String)> {
    fn canonical(dom: &Dom, n: NodeId) -> String {
        let mut attrs = dom
            .attributes_iter(n)
            .filter(|(name, _)| !dom.is_namespace_declaration(name))
            .map(|(name, value)| {
                (
                    name.namespace_name().to_string(),
                    name.local_name().to_string(),
                    value,
                )
            })
            .collect::<Vec<_>>();
        attrs.sort();
        let children = dom
            .nodes(n)
            .into_iter()
            .map(|n| canonical(dom, n))
            .collect::<Vec<_>>();
        format!(
            "{:?}:{attrs:?}:{:?}:{children:?}",
            dom.name(n),
            dom.text_value(n)
        )
    }
    projection(dom, root, accept)
        .into_iter()
        .map(|(text, properties)| {
            if properties.is_empty() {
                return (text, properties);
            }
            let mut parsed = Dom::new();
            let doc = parsed.parse_xdocument(&properties);
            let root = parsed.root(doc).unwrap();
            (text, canonical(&parsed, root))
        })
        .collect()
}

#[test]
fn nested_mark_effect_changes_keep_both_source_projections_and_disabled_tracking() {
    let fill = |color: &str| {
        format!(
            "<w14:textFill xmlns:w14='{}'><w14:solidFill><w14:srgbClr w14:val='{color}'/></w14:solidFill></w14:textFill>",
            crate::namespaces::W14::URI
        )
    };
    let old = fill("FF0000");
    let new = fill("0000FF");
    for tracking in [false, true] {
        let (mut dom, root, heading) = source(&old, &new);
        let accepted = effect_projection(&mut dom, root, true);
        let rejected = effect_projection(&mut dom, root, false);
        let before = dom.serialize_element(root);
        let settings = WmlComparerSettings {
            detect_format_changes: tracking,
            author_for_revisions: "Effect editor".into(),
            date_time_for_revisions: "2002-03-04T05:06:07Z".into(),
            ..WmlComparerSettings::default()
        };
        let mut id = 701;
        fold_midstream_demo_title_into_numbered_heading_with_settings(
            &mut dom, root, &settings, &mut id,
        );
        assert_eq!(
            effect_projection(&mut dom, root, true),
            accepted,
            "nested effect accepted, tracking={tracking}"
        );
        assert_eq!(
            effect_projection(&mut dom, root, false),
            rejected,
            "nested effect rejected, tracking={tracking}"
        );
        if !tracking {
            assert_eq!(dom.serialize_element(root), before);
            assert_eq!(id, 701);
        } else {
            assert_eq!(id, 703);
            let mark = dom
                .element(dom.element(heading, &W::p_pr()).unwrap(), &W::r_pr())
                .unwrap();
            let change = dom.element(mark, &W::r_pr_change()).unwrap();
            assert_eq!(dom.attribute(change, &W::author()), Some("Effect editor"));
            assert_eq!(
                dom.attribute(change, &W::date()),
                Some("2002-03-04T05:06:07Z")
            );
            let nested = crate::namespaces::W14::name("srgbClr");
            let old_color = dom.descendants(change, Some(&nested))[0];
            assert_eq!(
                dom.attribute(old_color, &crate::namespaces::W14::name("val")),
                Some("FF0000")
            );
            let live_fill = dom
                .element(mark, &crate::namespaces::W14::name("textFill"))
                .unwrap();
            let new_color = dom.descendants(live_fill, Some(&nested))[0];
            assert_eq!(
                dom.attribute(new_color, &crate::namespaces::W14::name("val")),
                Some("0000FF")
            );
        }
    }
}

#[test]
fn closing_mark_history_preserves_nested_effects_and_ignores_only_existing_normalization_noise() {
    let uri = crate::namespaces::W14::URI;
    for same in [false, true] {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!("<w:pPr xmlns:w='{}' xmlns:w14='{uri}' xmlns:pt='{}'><w:rPr><w:b/><w14:textFill><w14:solidFill><w14:srgbClr w14:val='FF0000'/></w14:solidFill></w14:textFill></w:rPr></w:pPr>", W::URI, PT::URI));
        let old = dom.root(doc).unwrap();
        let color = if same { "FF0000" } else { "0000FF" };
        let doc = dom.parse_xdocument(&format!("<w:pPr xmlns:w='{}' xmlns:w14='{uri}' xmlns:pt='{}'><w:rPr><w14:textFill w:rsidRPr='01234567'><w14:solidFill><w14:srgbClr w14:val='{color}'/></w14:solidFill></w14:textFill><w:b/><pt:Diagnostic/></w:rPr></w:pPr>", W::URI, PT::URI));
        let new = dom.root(doc).unwrap();
        let mut id = 901;
        let before = dom.serialize_element(old);
        let disabled = WmlComparerSettings {
            detect_format_changes: false,
            ..WmlComparerSettings::default()
        };
        closing_mark_takes_revised_mark_formatting(&mut dom, old, new, &disabled, &mut id);
        assert_eq!(dom.serialize_element(old), before);
        assert_eq!(id, 901);
        let settings = WmlComparerSettings::default();
        closing_mark_takes_revised_mark_formatting(&mut dom, old, new, &settings, &mut id);
        let mark = dom.element(old, &W::r_pr()).unwrap();
        assert_eq!(dom.element(mark, &W::r_pr_change()).is_some(), !same);
        assert_eq!(id, if same { 901 } else { 902 });
        let fill = dom
            .element(mark, &crate::namespaces::W14::name("textFill"))
            .unwrap();
        let leaf = dom.descendants(fill, Some(&crate::namespaces::W14::name("srgbClr")))[0];
        assert_eq!(
            dom.attribute(leaf, &crate::namespaces::W14::name("val")),
            Some(color)
        );
    }
}

#[test]
fn sharing_a_label_with_nested_effects_obeys_disabled_format_tracking() {
    let uri = crate::namespaces::W14::URI;
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!("<w:document xmlns:w='{}' xmlns:w14='{uri}'><w:body><w:p><w:del w:id='1'><w:r><w:rPr><w14:textFill><w14:solidFill><w14:srgbClr w14:val='FF0000'/></w14:solidFill></w14:textFill></w:rPr><w:delText>Lvl 1 a</w:delText></w:r></w:del><w:ins w:id='2'><w:r><w:rPr><w14:textFill><w14:solidFill><w14:srgbClr w14:val='0000FF'/></w14:solidFill></w14:textFill></w:rPr><w:t>a</w:t></w:r></w:ins></w:p></w:body></w:document>", W::URI));
    let root = dom.root(doc).unwrap();
    let p = dom.descendants(root, Some(&W::p()))[0];
    let before = dom.serialize_element(root);
    let settings = WmlComparerSettings {
        detect_format_changes: false,
        ..WmlComparerSettings::default()
    };
    mesh_short_label_shared_eq_with_settings(&mut dom, p, &settings);
    assert_eq!(dom.serialize_element(root), before);
    assert!(dom.descendants(root, Some(&W::r_pr_change())).is_empty());
}

#[test]
fn title_fold_preserves_existing_mark_history_and_section_break_ownership() {
    for section in [false, true] {
        let (mut dom, root, _) = source("<w:b/>", "<w:i/>");
        let title = dom.descendants(root, Some(&W::p()))[3];
        let properties = dom.element(title, &W::p_pr()).unwrap();
        if section {
            let doc = dom.parse_xdocument(&format!(
                "<w:sectPr xmlns:w='{}'><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>",
                W::URI
            ));
            let part = dom.root(doc).unwrap();
            dom.add(properties, part);
        } else {
            let doc = dom.parse_xdocument(&format!("<w:rPrChange xmlns:w='{}' w:id='99' w:author='Historical editor' w:date='{DATE}'><w:rPr><w:sz w:val='22'/></w:rPr></w:rPrChange>", W::URI));
            let history = dom.root(doc).unwrap();
            let mark = dom.element(properties, &W::r_pr()).unwrap();
            dom.add(mark, history);
        }
        let before = dom.serialize_element(root);
        let settings = WmlComparerSettings {
            author_for_revisions: "Configured editor".into(),
            date_time_for_revisions: "2002-03-04T05:06:07Z".into(),
            ..WmlComparerSettings::default()
        };
        let mut id = 701;
        fold_midstream_demo_title_into_numbered_heading_with_settings(
            &mut dom, root, &settings, &mut id,
        );
        assert_eq!(dom.serialize_element(root), before);
        assert_eq!(id, 701);
    }
}
#[test]
fn authored_title_snapshot_survives_demo_default_layout_normalization() {
    let (mut dom, root, heading) = source("<w:b/>", "<w:i/>");
    let accepted = projection(&mut dom, root, true);
    let rejected = projection(&mut dom, root, false);
    let source_properties = capture_deleted_demo_title_properties(&mut dom, root);
    assert_eq!(source_properties.len(), 1);
    strip_redundant_demo_default_spacing(&mut dom, root, (Some("276"), None));
    let title = dom.descendants(root, Some(&W::p()))[3];
    let live = dom.element(title, &W::p_pr()).unwrap();
    assert!(
        dom.element(live, &W::spacing_el()).is_none(),
        "Word live normalization ran before fold"
    );
    let settings = WmlComparerSettings {
        author_for_revisions: "Configured editor".into(),
        date_time_for_revisions: "2002-03-04T05:06:07Z".into(),
        ..WmlComparerSettings::default()
    };
    let mut id = 701;
    fold_midstream_demo_title_into_numbered_heading_with_source_properties(
        &mut dom,
        root,
        &settings,
        &mut id,
        &source_properties,
    );
    assert_eq!(projection(&mut dom, root, true), accepted);
    assert_eq!(projection(&mut dom, root, false), rejected);
    let ppr = dom.element(heading, &W::p_pr()).unwrap();
    let history = dom.element(ppr, &W::p_pr_change()).unwrap();
    let saved = dom.element(history, &W::p_pr()).unwrap();
    let spacing = dom.element(saved, &W::spacing_el()).unwrap();
    assert_eq!(dom.attribute(spacing, &W::name("line")), Some("276"));
    assert_eq!(
        dom.attribute(history, &W::author()),
        Some("Configured editor")
    );
    assert_eq!(
        dom.attribute(history, &W::date()),
        Some("2002-03-04T05:06:07Z")
    );
}
