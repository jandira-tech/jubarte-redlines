// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

#[test]
fn parked_layout_restores_properties_that_remain_live_when_rejected() {
    for mixed in [false, true] {
        let mut dom = Dom::new();
        let inserted = if mixed {
            "<w:ins><w:r><w:t>new</w:t></w:r></w:ins>"
        } else {
            ""
        };
        let doc = dom.parse_xdocument(&format!(
                r#"<w:document xmlns:w="{}"><w:body><w:p><w:pPr><w:keepNext/><w:keepLines/><w:contextualSpacing/><w:outlineLvl w:val="1"/><w:spacing w:before="120" w:after="240"/><w:ind w:left="720" w:hanging="360"/><w:jc w:val="both"/><w:rPr><w:del w:id="7"/></w:rPr></w:pPr>{inserted}<w:del><w:r><w:delText>old</w:delText></w:r></w:del></w:p></w:body></w:document>"#,
                W::URI
            ));
        let root = dom.root(doc).unwrap();
        let p = dom.descendants(root, Some(&W::p()))[0];
        let ppr = dom.element(p, &W::p_pr()).unwrap();
        let expected = formatchg::normalize_para_properties(&mut dom, ppr);
        let mut id = 100;
        last_pure_del_spacing_to_pprchange(
            &mut dom,
            root,
            &WmlComparerSettings::default(),
            &mut id,
        );
        assert_eq!(id, 101);
        assert!(dom.element(ppr, &W::spacing_el()).is_none());
        for property in ["keepNext", "keepLines", "contextualSpacing", "outlineLvl"] {
            assert!(dom.element(ppr, &W::name(property)).is_some());
        }
        assert_eq!(dom.element(ppr, &W::name("ind")).is_some(), mixed);
        assert_eq!(dom.element(ppr, &W::jc_el()).is_some(), mixed);
        let change = dom.element(ppr, &W::p_pr_change()).unwrap();
        assert_eq!(dom.attribute(change, &W::id()), Some("100"));
        let saved = dom.element(change, &W::p_pr()).unwrap();
        assert_eq!(
            formatchg::normalize_para_properties(&mut dom, saved),
            expected
        );
        assert!(dom.element(saved, &W::r_pr()).is_none());
        let rejected = crate::revision_processor::reject_revisions_document(&mut dom, root);
        let p = dom.descendants(rejected, Some(&W::p()))[0];
        let restored = dom.element(p, &W::p_pr()).unwrap();
        assert_eq!(
            formatchg::normalize_para_properties(&mut dom, restored),
            expected
        );
        assert!(dom.element(restored, &W::p_pr_change()).is_none());
        let text: String = dom
            .descendants(p, Some(&W::t()))
            .into_iter()
            .map(|t| dom.value(t))
            .collect();
        assert_eq!(text, "old");
    }
}
