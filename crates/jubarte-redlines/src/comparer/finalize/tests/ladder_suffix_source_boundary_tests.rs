// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;
const DATE: &str = "2026-01-02T03:04:05Z";
const FORMAT: &str = "<w:rPr><w:b/><w:color w:val='123456'/></w:rPr>";
const OTHER: &str = "<w:rPr><w:i/><w:color w:val='654321'/></w:rPr>";
const PROPS: &str = "<w:pPr><w:pStyle w:val='BodyText'/><w:spacing w:after='80'/></w:pPr>";
fn xml(body: &str) -> String {
    format!(
        "<w:document xmlns:w='{}'><w:body>{body}<w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",
        W::URI
    )
}
fn package(body: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml(body));
    let root = dom.root(document).expect("document");
    (dom, root)
}
fn carrier(paragraph: &str, cell: bool) -> String {
    if cell {
        format!(
            "<w:tbl><w:tblPr><w:tblW w:w='1800' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1800'/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w='1800' w:type='dxa'/></w:tcPr>{paragraph}</w:tc></w:tr></w:tbl>"
        )
    } else {
        paragraph.to_owned()
    }
}
#[test]
fn public_ladder_restores_suffix_author_at_unicode_and_run_boundaries() {
    // Each expected prefix/suffix run is authored literally, independently
    // of the splitting algorithm. XML space is needed only on actual
    // leading/trailing whitespace, including after a new split boundary.
    for (name, prefix, suffix, combined, prefix_runs, suffix_runs) in [
        (
            "Unicode within one leaf",
            "Préfix ",
            "尾",
            format!("<w:r>{FORMAT}<w:t>Préfix 尾</w:t></w:r>"),
            format!("<w:r>{FORMAT}<w:t xml:space='preserve'>Préfix </w:t></w:r>"),
            format!("<w:r>{FORMAT}<w:t>尾</w:t></w:r>"),
        ),
        (
            "multiple suffix leaves in same source run",
            "Préfix ",
            "尾 second",
            format!(
                "<w:r>{FORMAT}<w:t xml:space='preserve'>Préfix </w:t><w:t>尾</w:t><w:t xml:space='preserve'> second</w:t></w:r>"
            ),
            format!("<w:r>{FORMAT}<w:t xml:space='preserve'>Préfix </w:t></w:r>"),
            format!("<w:r>{FORMAT}<w:t>尾</w:t><w:t xml:space='preserve'> second</w:t></w:r>"),
        ),
        (
            "different owned run properties",
            "Préfix ",
            "尾",
            format!(
                "<w:r>{FORMAT}<w:t xml:space='preserve'>Préfix </w:t></w:r><w:r>{OTHER}<w:t>尾</w:t></w:r>"
            ),
            format!("<w:r>{FORMAT}<w:t xml:space='preserve'>Préfix </w:t></w:r>"),
            format!("<w:r>{OTHER}<w:t>尾</w:t></w:r>"),
        ),
        (
            "multiple suffix leaves in an independently moved run",
            "Préfix ",
            "尾 second",
            format!(
                "<w:r>{FORMAT}<w:t xml:space='preserve'>Préfix </w:t></w:r><w:r>{OTHER}<w:t>尾</w:t><w:t xml:space='preserve'> second</w:t></w:r>"
            ),
            format!("<w:r>{FORMAT}<w:t xml:space='preserve'>Préfix </w:t></w:r>"),
            format!("<w:r>{OTHER}<w:t>尾</w:t><w:t xml:space='preserve'> second</w:t></w:r>"),
        ),
        (
            "combining-character boundary without whitespace",
            "é\u{300}",
            "Tail",
            format!("<w:r>{FORMAT}<w:t>é\u{300}Tail</w:t></w:r>"),
            format!("<w:r>{FORMAT}<w:t>é\u{300}</w:t></w:r>"),
            format!("<w:r>{FORMAT}<w:t>Tail</w:t></w:r>"),
        ),
    ] {
        for source_has_date in [false, true] {
            for cell in [false, true] {
                let source_date = if source_has_date {
                    " w:date='2025-12-01T02:03:04Z'"
                } else {
                    ""
                };
                let expected_date = if source_has_date {
                    "2025-12-01T02:03:04Z"
                } else {
                    DATE
                };
                let input_p = format!(
                    "<w:p>{PROPS}<w:ins w:id='9' w:author='Comparison owner' w:date='{DATE}'>{combined}</w:ins></w:p>"
                );
                let expected_p = format!(
                    "<w:p>{PROPS}<w:ins w:id='9' w:author='Comparison owner' w:date='{DATE}'>{prefix_runs}</w:ins><w:ins w:id='10' w:author='Authored suffix owner' w:date='{expected_date}'>{suffix_runs}</w:ins></w:p>"
                );
                let old_p = format!(
                    "<w:p>{PROPS}<w:r>{FORMAT}<w:t xml:space='preserve'>{prefix}</w:t></w:r><w:r>{OTHER}<w:t>Original tail</w:t></w:r></w:p>"
                );
                let new_p = format!(
                    "<w:p>{PROPS}<w:r>{FORMAT}<w:t xml:space='preserve'>{prefix}</w:t></w:r><w:ins w:id='2' w:author='Authored suffix owner'{source_date}><w:r>{OTHER}<w:t>{suffix}</w:t></w:r></w:ins></w:p>"
                );
                let input = carrier(&input_p, cell);
                let expected = carrier(&expected_p, cell);
                let original_xml = xml(&carrier(&old_p, cell));
                let revised_xml = xml(&carrier(&new_p, cell));
                let settings = WmlComparerSettings {
                    author_for_revisions: "Configured ladder editor".into(),
                    date_time_for_revisions: DATE.into(),
                    ..WmlComparerSettings::default()
                };
                let sources = LadderSources {
                    original_xml: &original_xml,
                    revised_xml: &revised_xml,
                    settings: &settings,
                };
                let (mut dom, root) = package(&input);
                let (expected_dom, expected_root) = package(&expected);
                align_remaining_ladder_rungs(&mut dom, root, &sources);
                let coordinate = format!("{name}/source-date={source_has_date}/cell={cell}");
                assert_eq!(
                    dom.serialize_element(root),
                    expected_dom.serialize_element(expected_root),
                    "{coordinate}: complete text, format, provenance and geometry"
                );
                let once = dom.serialize_element(root);
                align_remaining_ladder_rungs(&mut dom, root, &sources);
                assert_eq!(dom.serialize_element(root), once, "{coordinate}: repeat");
            }
        }
    }
}
