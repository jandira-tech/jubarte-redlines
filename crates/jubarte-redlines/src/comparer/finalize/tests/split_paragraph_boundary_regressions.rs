// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

#[test]
fn mixed_inserted_boundary_joins_original_text_and_closing_layout_on_reject() {
    for gap in [false, true] {
        let mut dom = Dom::new();
        let blank = if gap {
            "<w:p><w:pPr><w:rPr><w:ins w:id=\"12\"/></w:rPr></w:pPr></w:p>"
        } else {
            ""
        };
        let doc = dom.parse_xdocument(&format!(
                r#"<w:document xmlns:w="{}"><w:body><w:p><w:pPr><w:rPr><w:ins w:id="11"/></w:rPr></w:pPr><w:ins><w:r><w:t>New</w:t></w:r></w:ins><w:del><w:r><w:delText>Old</w:delText></w:r></w:del></w:p>{blank}<w:p><w:pPr><w:pPrChange w:id="13"><w:pPr><w:bidi/><w:contextualSpacing/><w:outlineLvl w:val="1"/></w:pPr></w:pPrChange></w:pPr><w:r><w:t xml:space="preserve"> tail</w:t></w:r></w:p></w:body></w:document>"#,
                W::URI
            ));
        let root = dom.root(doc).unwrap();
        let body = dom.element(root, &W::body()).unwrap();
        let first = dom.elements(body, Some(&W::p()))[0];
        let first_ppr = dom.element(first, &W::p_pr()).unwrap();
        let boundary = dom.serialize_element(first_ppr);
        let closing = *dom.elements(body, Some(&W::p())).last().unwrap();
        let change = dom
            .element(dom.element(closing, &W::p_pr()).unwrap(), &W::p_pr_change())
            .unwrap();
        let saved = dom.element(change, &W::p_pr()).unwrap();
        let expected = formatchg::normalize_para_properties(&mut dom, saved);
        strip_unrecorded_word_defaults(&mut dom, root, None);
        assert_eq!(dom.element(first, &W::p_pr()), Some(first_ppr));
        assert_eq!(dom.serialize_element(first_ppr), boundary);
        let accepted_input = dom.clone_subtree(root);
        let accepted =
            crate::revision_processor::accept_revisions_document(&mut dom, accepted_input);
        let abody = dom.element(accepted, &W::body()).unwrap();
        assert_eq!(
            dom.elements(abody, Some(&W::p())).len(),
            if gap { 3 } else { 2 }
        );
        let accepted_text: String = dom
            .descendants(abody, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect();
        assert_eq!(accepted_text, "New tail");
        let rejected = crate::revision_processor::reject_revisions_document(&mut dom, root);
        let rbody = dom.element(rejected, &W::body()).unwrap();
        let paragraphs = dom.elements(rbody, Some(&W::p()));
        assert_eq!(paragraphs.len(), 1);
        let restored = dom.element(paragraphs[0], &W::p_pr()).unwrap();
        assert_eq!(
            formatchg::normalize_para_properties(&mut dom, restored),
            expected
        );
        let rejected_text: String = dom
            .descendants(rbody, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect();
        assert_eq!(rejected_text, "Old tail");
    }
}
