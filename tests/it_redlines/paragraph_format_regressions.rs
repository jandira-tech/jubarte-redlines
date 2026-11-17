// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! Property comparison ignores scratch metadata while preserving real layout edits.

use jubarte::comparer::formatchg::get_changed_property_names;
use jubarte::comparer::{WmlComparerSettings, compare_bodies_faithful};
use jubarte::namespaces::{PT, W};
use jubarte::revision_processor::{accept_revisions_document, reject_revisions_document};
use jubarte::xmllinq::{Dom, NodeId};

fn properties(dom: &mut Dom, inner: &str) -> NodeId {
    let doc = dom.parse_xdocument(&format!(
        r#"<w:pPr xmlns:w="{}" xmlns:pt="{}">{inner}</w:pPr>"#,
        W::URI,
        PT::URI
    ));
    dom.root(doc).unwrap()
}

#[test]
fn scratch_attributes_and_attribute_order_do_not_invent_property_changes() {
    let mut dom = Dom::new();
    let old = properties(
        &mut dom,
        r#"<w:spacing pt:Unid="old" w:after="120" w:before="240" w:rsidR="111"/>"#,
    );
    let new = properties(
        &mut dom,
        r#"<w:spacing w:before="240" w:after="120" pt:Unid="new" w:rsidR="222"/>"#,
    );
    assert!(get_changed_property_names(&mut dom, Some(old), Some(new)).is_empty());
}

#[test]
fn scratch_metadata_does_not_hide_a_real_property_change() {
    let mut dom = Dom::new();
    let old = properties(&mut dom, r#"<w:spacing pt:Unid="same" w:after="120"/>"#);
    let new = properties(&mut dom, r#"<w:spacing pt:Unid="same" w:after="240"/>"#);
    assert_eq!(
        get_changed_property_names(&mut dom, Some(old), Some(new)),
        ["characterSpacing"]
    );
}

fn document(dom: &mut Dom, props: &str) -> (NodeId, NodeId) {
    let doc = dom.parse_xdocument(&format!(r#"<w:document xmlns:w="{}"><w:body><w:p><w:pPr>{props}</w:pPr><w:r><w:t>Unchanged paragraph text.</w:t></w:r></w:p></w:body></w:document>"#, W::URI));
    let root = dom.root(doc).unwrap();
    (root, dom.element(root, &W::body()).unwrap())
}

#[test]
fn added_layout_properties_are_tracked_and_removed_on_rejection() {
    for (local, xml) in [
        ("outlineLvl", r#"<w:outlineLvl w:val="2"/>"#),
        ("keepNext", "<w:keepNext/>"),
        ("ind", r#"<w:ind w:left="720"/>"#),
        ("pageBreakBefore", "<w:pageBreakBefore/>"),
    ] {
        let mut dom = Dom::new();
        let (a, ab) = document(&mut dom, "");
        let (b, bb) = document(&mut dom, xml);
        let out = compare_bodies_faithful(&mut dom, a, b, ab, bb, &WmlComparerSettings::default());
        let changes = dom.descendants(out, Some(&W::name("pPrChange")));
        assert_eq!(changes.len(), 1, "{local}: {}", dom.serialize_element(out));
        let old = dom.element(changes[0], &W::p_pr()).unwrap();
        assert!(
            dom.elements(old, None).is_empty(),
            "{local}: previous layout was empty"
        );
        let accepted_input = dom.clone_subtree(out);
        let accepted = accept_revisions_document(&mut dom, accepted_input);
        assert_eq!(
            dom.descendants(accepted, Some(&W::name(local))).len(),
            1,
            "{local}"
        );
        assert!(
            dom.descendants(accepted, Some(&W::name("pPrChange")))
                .is_empty()
        );
        let rejected = reject_revisions_document(&mut dom, out);
        assert!(
            dom.descendants(rejected, Some(&W::name(local))).is_empty(),
            "{local}"
        );
        let text: String = dom
            .descendants(rejected, Some(&W::t()))
            .iter()
            .map(|&t| dom.value(t))
            .collect();
        assert_eq!(text, "Unchanged paragraph text.");
    }
}

#[test]
fn unchanged_layout_does_not_create_a_format_revision() {
    let mut dom = Dom::new();
    let (a, ab) = document(
        &mut dom,
        r#"<w:outlineLvl w:val="2"/><w:ind w:left="720"/>"#,
    );
    let (b, bb) = document(
        &mut dom,
        r#"<w:outlineLvl w:val="2"/><w:ind w:left="720"/>"#,
    );
    let out = compare_bodies_faithful(&mut dom, a, b, ab, bb, &WmlComparerSettings::default());
    assert!(dom.descendants(out, Some(&W::name("pPrChange"))).is_empty());
    assert!(dom.descendants(out, Some(&W::name("rPrChange"))).is_empty());
}
