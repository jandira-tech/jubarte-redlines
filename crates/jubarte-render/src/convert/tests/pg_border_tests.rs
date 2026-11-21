// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn parse(inner: &str) -> PageBorders {
    let xml = format!(
        r#"<w:pgBorders xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" {inner}</w:pgBorders>"#
    );
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let pb = dom.root(doc).expect("pgBorders");
    parse_pg_borders(&dom, pb)
}

#[test]
fn omitted_offset_from_is_text_and_display_is_all_pages() {
    let b = parse(r#"><w:top w:val="single" w:sz="8"/>"#);
    assert!(!b.from_page);
    assert_eq!(b.display, BorderDisplay::AllPages);
    assert!(!b.back, "zOrder defaults to front");
    let b = parse(
        r#"w:offsetFrom="page" w:display="notFirstPage" w:zOrder="back"><w:top w:val="single"/>"#,
    );
    assert!(b.from_page);
    assert_eq!(b.display, BorderDisplay::NotFirstPage);
    assert!(b.back);
}

#[test]
fn medium_gap_edges_paint_as_one_band() {
    // 00205272: Word draws thinThick- and thickThinMediumGap alike.
    let b = parse(
        r#"><w:top w:val="thinThickMediumGap" w:sz="24"/><w:bottom w:val="thickThinMediumGap" w:sz="24"/>"#,
    );
    assert_eq!(b.top.map(|e| e.line), Some(BorderLine::MediumGap));
    assert_eq!(b.bottom.map(|e| e.line), Some(BorderLine::MediumGap));
}

#[test]
fn edge_styles_and_nil_edges() {
    let b = parse(
        r#"><w:top w:val="double"/><w:left w:val="dotted"/><w:bottom w:val="dashSmallGap"/><w:right w:val="nil"/>"#,
    );
    assert_eq!(b.top.map(|e| e.line), Some(BorderLine::Double));
    assert_eq!(b.left.map(|e| e.line), Some(BorderLine::Dotted));
    assert_eq!(b.bottom.map(|e| e.line), Some(BorderLine::Dashed));
    assert!(b.right.is_none(), "nil paints nothing");
    let none = parse(r#"><w:top w:val="none"/>"#);
    assert!(none.top.is_none());
}
