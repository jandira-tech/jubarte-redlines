// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;
const DATE: &str = "2026-01-02T03:04:05Z";
const SPACING: &str = "<w:spacing w:line='240' w:lineRule='auto'/>";
const DEL_MARK: &str =
    "<w:del w:id='12' w:author='Original mark owner' w:date='2025-01-01T00:00:00Z'/>";
const HISTORY: &str = "<w:pPrChange w:id='13' w:author='Original layout owner' w:date='2025-01-01T00:00:00Z'><w:pPr><w:spacing w:after='80'/></w:pPr></w:pPrChange>";
const END: &str = "<w:p><w:pPr><w:pStyle w:val='BodyText'/><w:spacing w:line='240' w:lineRule='auto'/><w:jc w:val='both'/><w:rPr><w:del w:id='22' w:author='Terminal mark owner' w:date='2025-01-01T00:00:00Z'/><w:rFonts w:ascii='Arial'/><w:sz w:val='24'/></w:rPr></w:pPr><w:del w:id='23' w:author='Terminal text owner' w:date='2025-01-01T00:00:00Z'><w:r><w:delText>Terminal source formatting remains authored</w:delText></w:r></w:del></w:p><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr>";
fn package(body: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&format!(
        "<w:document xmlns:w='{}'><w:body>{body}</w:body></w:document>",
        W::URI
    ));
    let root = dom.root(document).expect("document");
    (dom, root)
}
/// M376 removes copied list layout from a middle prose deletion. Optional
/// source jc/rPr/pPr may legitimately be absent; terminal/list/history
/// owners remain exact. This phase's Word carrier contract is literal.
#[test]
fn list_layout_cleanup_handles_absent_optional_properties_without_inventing_owners() {
    let donor = format!(
        "<w:p><w:pPr><w:pStyle w:val='ListParagraph'/><w:numPr><w:ilvl w:val='0'/><w:numId w:val='7'/></w:numPr>{SPACING}<w:rPr><w:ins w:id='1' w:author='Revised list mark owner' w:date='{DATE}'/><w:b/></w:rPr></w:pPr><w:ins w:id='2' w:author='Revised list text owner' w:date='{DATE}'><w:r><w:rPr><w:b/></w:rPr><w:t>Revised authored list item</w:t></w:r></w:ins></w:p>"
    );
    let body = "<w:del w:id='11' w:author='Original prose owner' w:date='2025-01-01T00:00:00Z'><w:r><w:rPr><w:i/><w:color w:val='654321'/></w:rPr><w:delText>Original independent prose</w:delText></w:r></w:del>";
    for (rpr_name, original_rpr, expected_rpr) in [
        ("absent", String::new(), String::new()),
        (
            "source mark only",
            format!("<w:rPr>{DEL_MARK}</w:rPr>"),
            format!("<w:rPr>{DEL_MARK}</w:rPr>"),
        ),
        (
            "mark with copied scalar formats",
            format!("<w:rPr>{DEL_MARK}<w:rFonts w:ascii='Arial'/><w:sz w:val='24'/></w:rPr>"),
            format!("<w:rPr>{DEL_MARK}</w:rPr>"),
        ),
        (
            "formats without a revised paragraph mark",
            "<w:rPr><w:rFonts w:ascii='Arial'/><w:sz w:val='24'/></w:rPr>".into(),
            "<w:rPr/>".into(),
        ),
    ] {
        for jc in [
            None,
            Some("both"),
            Some("distribute"),
            Some("center"),
            Some("right"),
        ] {
            for history in [false, true] {
                let jc_xml = jc
                    .map(|value| format!("<w:jc w:val='{value}'/>"))
                    .unwrap_or_default();
                let saved = if history { HISTORY } else { "" };
                let keep_jc = if matches!(jc, Some("both" | "distribute")) {
                    ""
                } else {
                    &jc_xml
                };
                let input = format!(
                    "{donor}<w:p><w:pPr><w:pStyle w:val='BodyText'/>{SPACING}{jc_xml}{original_rpr}{saved}</w:pPr>{body}</w:p>{END}"
                );
                let expected = format!(
                    "{donor}<w:p><w:pPr><w:pStyle w:val='BodyText'/>{keep_jc}{expected_rpr}{saved}</w:pPr>{body}</w:p>{END}"
                );
                let (mut dom, root) = package(&input);
                let (expected_dom, expected_root) = package(&expected);
                strip_list_layout_from_mid_pure_del(&mut dom, root);
                let coordinate = format!("rPr={rpr_name}/jc={jc:?}/history={history}");
                assert_eq!(
                    dom.serialize_element(root),
                    expected_dom.serialize_element(expected_root),
                    "{coordinate}: complete Word phase source ownership"
                );
                let once = dom.serialize_element(root);
                strip_list_layout_from_mid_pure_del(&mut dom, root);
                assert_eq!(dom.serialize_element(root), once, "{coordinate}: repeat");
            }
        }
    }
    // A middle body-only deletion has no paragraph-property carrier to
    // clean. It must remain absent, rather than acquiring empty formatting.
    let input = format!("{donor}<w:p>{body}</w:p>{END}");
    let (mut dom, root) = package(&input);
    let before = dom.serialize_element(root);
    let nodes = dom.node_count();
    strip_list_layout_from_mid_pure_del(&mut dom, root);
    assert_eq!(dom.serialize_element(root), before);
    assert_eq!(dom.node_count(), nodes);
}
