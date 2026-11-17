// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

fn projection(dom: &Dom, node: NodeId, accepting: bool) -> String {
    let excluded = if accepting { W::del() } else { W::ins() };
    if dom.name_is(node, &excluded) {
        return String::new();
    }
    if dom.name_is(node, &W::t()) || dom.name_is(node, &W::del_text()) {
        return dom.value(node);
    }
    dom.elements(node, None)
        .into_iter()
        .map(|child| projection(dom, child, accepting))
        .collect()
}

#[test]
fn shared_label_fragments_preserve_both_revision_projections() {
    for (first, last, label) in [("Lvl 1 A", "B", "AB"), ("Lvl 1 1", "2", "12")] {
        let mut dom = Dom::new();
        let doc=dom.parse_xdocument(&format!(r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:del><w:r><w:delText xml:space="preserve">{first}</w:delText></w:r><w:r><w:delText>{last}</w:delText></w:r></w:del><w:ins><w:r><w:t>{label}</w:t></w:r></w:ins></w:p>"#));
        let p = dom.root(doc).unwrap();
        let accepted = projection(&dom, p, true);
        let rejected = projection(&dom, p, false);
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(projection(&dom, p, true), accepted);
        assert_eq!(projection(&dom, p, false), rejected);
        assert!(dom.elements(p, Some(&W::ins())).is_empty());
        assert_eq!(
            dom.descendants(p, Some(&W::t()))
                .iter()
                .map(|&t| dom.value(t))
                .collect::<String>(),
            label
        );
    }
}

#[test]
fn unsupported_fragment_tail_cannot_partially_erase_a_deletion() {
    let mut dom = Dom::new();
    let doc=dom.parse_xdocument(r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:del><w:r><w:delText xml:space="preserve">Lvl 1 A</w:delText></w:r><w:r><w:delText>B</w:delText></w:r><w:r><w:delText>!</w:delText></w:r></w:del><w:ins><w:r><w:t>AB</w:t></w:r></w:ins></w:p>"#);
    let p = dom.root(doc).unwrap();
    let before = dom.serialize_element(p);
    mesh_short_label_shared_eq(&mut dom, p);
    assert_eq!(dom.serialize_element(p), before);
    assert_eq!(projection(&dom, p, true), "AB");
    assert_eq!(projection(&dom, p, false), "Lvl 1 AB!");
}
#[test]
fn label_mesh_preserves_punctuation_styles_and_nontext_payloads() {
    for (deleted, inserted) in [
        (
            "<w:r><w:delText>Lvl 1 AB!</w:delText></w:r>",
            "<w:r><w:t>AB</w:t></w:r>",
        ),
        (
            "<w:r><w:delText>Lvl 1 AB</w:delText></w:r>",
            "<w:r><w:t>AB</w:t><w:tab/></w:r>",
        ),
        (
            "<w:r><w:delText>Lvl 1 AB</w:delText></w:r>",
            "<w:r><w:t>AB</w:t><w:br/></w:r>",
        ),
        (
            "<w:r><w:delText>Lvl 1 AB</w:delText></w:r>",
            "<w:r><w:t>AB</w:t><w:footnoteReference w:id=\"2\"/></w:r>",
        ),
    ] {
        let mut dom = Dom::new();
        let xml = format!(
            r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:del>{deleted}</w:del><w:ins>{inserted}</w:ins></w:p>"#
        );
        let doc = dom.parse_xdocument(&xml);
        let p = dom.root(doc).unwrap();
        let before = dom.serialize_element(p);
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(dom.serialize_element(p), before);
    }
}
#[test]
fn interleaved_revision_blocks_preserve_insertion_order() {
    for revisions in [
        "<w:ins><w:r><w:t>AB!</w:t></w:r></w:ins><w:del><w:r><w:delText>Lvl 1 </w:delText></w:r></w:del><w:ins><w:r><w:t>?</w:t></w:r></w:ins><w:del><w:r><w:delText>AB</w:delText></w:r></w:del>",
        "<w:del><w:r><w:delText>Lvl 1 </w:delText></w:r></w:del><w:ins><w:r><w:t>AB!</w:t></w:r></w:ins><w:del><w:r><w:delText>AB</w:delText></w:r></w:del>",
    ] {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{revisions}</w:p>"#));
        let p = dom.root(doc).unwrap();
        let accepted = projection(&dom, p, true);
        let rejected = projection(&dom, p, false);
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(projection(&dom, p, true), accepted);
        assert_eq!(projection(&dom, p, false), rejected);
    }
}
#[test]
fn shared_label_mesh_keeps_nested_revision_hyperlink_and_control_ownership() {
    for payload in [
        r#"<w:del><w:r><w:delText>Lvl 1 </w:delText></w:r></w:del><w:ins><w:r><w:t>AB</w:t></w:r></w:ins><w:hyperlink w:anchor="target"><w:del><w:r><w:delText>AB</w:delText></w:r></w:del></w:hyperlink>"#,
        r#"<w:del><w:r><w:delText>Lvl 1 AB</w:delText></w:r></w:del><w:ins><w:r><w:t/></w:r></w:ins><w:hyperlink w:anchor="target"><w:ins><w:r><w:t>AB</w:t></w:r></w:ins></w:hyperlink>"#,
        r#"<w:del><w:r><w:delText>Lvl 1 </w:delText></w:r></w:del><w:ins><w:r><w:t>AB</w:t></w:r></w:ins><w:sdt><w:sdtPr><w:tag w:val="source-control"/></w:sdtPr><w:sdtContent><w:del><w:r><w:delText>AB</w:delText></w:r></w:del></w:sdtContent></w:sdt>"#,
    ] {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(r#"<w:p xmlns:w="{}">{payload}</w:p>"#, W::URI));
        let p = dom.root(doc).unwrap();
        let before = dom.serialize_element(p);
        let accepted = projection(&dom, p, true);
        let rejected = projection(&dom, p, false);
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(
            dom.serialize_element(p),
            before,
            "wrapper ownership must survive"
        );
        assert_eq!(projection(&dom, p, true), accepted);
        assert_eq!(projection(&dom, p, false), rejected);
    }
}

#[test]
fn short_label_mesh_keeps_unrelated_empty_run_properties() {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(r#"<w:p xmlns:w="{}"><w:r w:rsidR="12345678"><w:rPr><w:b/></w:rPr></w:r><w:del><w:r><w:delText>Lvl 1 AB</w:delText></w:r></w:del><w:ins><w:r><w:t>AB</w:t></w:r></w:ins></w:p>"#, W::URI));
    let p = dom.root(doc).unwrap();
    let retained = dom.elements(p, Some(&W::r()))[0];
    let properties = dom.serialize_element(retained);
    mesh_short_label_shared_eq(&mut dom, p);
    assert_eq!(dom.parent(retained), Some(p));
    assert_eq!(dom.serialize_element(retained), properties);
    assert_eq!(projection(&dom, p, true), "AB");
    assert_eq!(projection(&dom, p, false), "Lvl 1 AB");
    assert!(dom.elements(p, Some(&W::ins())).is_empty());
    assert_eq!(dom.elements(p, Some(&W::r())).len(), 2);
}

#[test]
fn paragraph_mark_revision_does_not_block_plain_shared_label_meshing() {
    for revision in ["ins", "del"] {
        let mut dom = Dom::new();
        let doc = dom.parse_xdocument(&format!(r#"<w:p xmlns:w="{}"><w:pPr><w:rPr><w:{revision} w:id="7"/></w:rPr></w:pPr><w:del><w:r><w:delText>Lvl 1 AB</w:delText></w:r></w:del><w:ins><w:r><w:t>AB</w:t></w:r></w:ins></w:p>"#, W::URI));
        let p = dom.root(doc).unwrap();
        let ppr = dom.element(p, &W::p_pr()).unwrap();
        let mark = dom.serialize_element(ppr);
        mesh_short_label_shared_eq(&mut dom, p);
        assert_eq!(projection(&dom, p, true), "AB");
        assert_eq!(projection(&dom, p, false), "Lvl 1 AB");
        assert_eq!(dom.serialize_element(ppr), mark);
        assert!(dom.elements(p, Some(&W::ins())).is_empty());
        assert_eq!(dom.elements(p, Some(&W::r())).len(), 1);
    }
}
