// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn already_revised_live_closing_layout_still_records_original_properties() {
    let mut dom = Dom::new();
    let original_doc = dom.parse_xdocument(&format!(
        r#"<w:pPr xmlns:w="{}"><w:bidi/><w:contextualSpacing/><w:outlineLvl w:val="1"/></w:pPr>"#,
        W::URI
    ));
    let original = dom.root(original_doc).unwrap();
    let revised = dom.new_element(W::p_pr());
    let doc = dom.parse_xdocument(&format!(
            r#"<w:document xmlns:w="{}"><w:body><w:p><w:pPr/><w:r><w:t>original closing text</w:t></w:r></w:p></w:body></w:document>"#,
            W::URI
        ));
    let root = dom.root(doc).unwrap();
    let p = dom.descendants(root, Some(&W::p()))[0];
    let expected = formatchg::normalize_para_properties(&mut dom, original);
    let mut id = 100;
    closing_mark_takes_revised_properties(
        &mut dom,
        root,
        Some(original),
        Some(revised),
        (None, None),
        &WmlComparerSettings::default(),
        &mut id,
    );
    assert_eq!(id, 101);
    let ppr = dom.element(p, &W::p_pr()).unwrap();
    assert!(formatchg::normalize_para_properties(&mut dom, ppr).is_empty());
    let change = dom.element(ppr, &W::p_pr_change()).unwrap();
    assert_eq!(dom.attribute(change, &W::id()), Some("100"));
    let saved = dom.element(change, &W::p_pr()).unwrap();
    assert_eq!(
        formatchg::normalize_para_properties(&mut dom, saved),
        expected
    );
    let once = dom.serialize_element(root);
    closing_mark_takes_revised_properties(
        &mut dom,
        root,
        Some(original),
        Some(revised),
        (None, None),
        &WmlComparerSettings::default(),
        &mut id,
    );
    assert_eq!(id, 101);
    assert_eq!(dom.serialize_element(root), once);
    let accepted_input = dom.clone_subtree(root);
    let accepted = crate::revision_processor::accept_revisions_document(&mut dom, accepted_input);
    let ap = dom.descendants(accepted, Some(&W::p()))[0];
    let appr = dom.element(ap, &W::p_pr()).unwrap();
    assert!(formatchg::normalize_para_properties(&mut dom, appr).is_empty());
    let rejected = crate::revision_processor::reject_revisions_document(&mut dom, root);
    let rp = dom.descendants(rejected, Some(&W::p()))[0];
    let rppr = dom.element(rp, &W::p_pr()).unwrap();
    assert_eq!(
        formatchg::normalize_para_properties(&mut dom, rppr),
        expected
    );
    assert_eq!(
        dom.value(dom.descendants(rp, Some(&W::t()))[0]),
        "original closing text"
    );
}
