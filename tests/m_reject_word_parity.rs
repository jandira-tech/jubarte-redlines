// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Reject All as Microsoft Word does it. Each rule was read off Word's own
//! Reject All of Word's redlines (the bench's `rejected_tracking` corpus,
//! 2026-09-29) and is pinned here with a synthetic package.

mod common;

use common::docx::{docx, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::reject_revisions;
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const REV: &str = r#"w:author="a" w:date="2026-01-01T00:00:00Z""#;

/// Body paragraphs of `document.xml` (tables included, in order), each as
/// its visible text.
fn paragraphs(pkg: &[u8]) -> Vec<String> {
    let xml = part_string(pkg, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            dom.descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect::<String>()
        })
        .collect()
}

/// A move range is a pair of markers, not a container: Word ends a moveTo
/// range inside the first cell of the table after the moved heading (its
/// redline of 30f20e787b). Rejecting the move removes the moved heading and
/// its mark, and the table keeps its tblPr, tblGrid and surviving rows.
#[test]
fn reject_keeps_a_table_a_move_range_ends_inside() {
    let body = format!(
        r#"<w:p><w:r><w:t>Before</w:t></w:r></w:p>
<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:rPr><w:moveTo w:id="1" {REV}/></w:rPr></w:pPr><w:moveToRangeStart w:id="2" {REV} w:name="move1"/><w:moveTo w:id="3" {REV}><w:r><w:t>Heading</w:t></w:r></w:moveTo></w:p>
<w:tbl><w:tblPr><w:tblW w:type="auto" w:w="0"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid>
<w:tr><w:trPr><w:ins w:id="4" {REV}/></w:trPr><w:tc><w:tcPr><w:tcW w:type="dxa" w:w="2000"/></w:tcPr><w:moveToRangeEnd w:id="2"/><w:p><w:pPr><w:rPr><w:ins w:id="5" {REV}/></w:rPr></w:pPr><w:ins w:id="6" {REV}><w:r><w:t>New row</w:t></w:r></w:ins></w:p></w:tc></w:tr>
<w:tr><w:tc><w:tcPr><w:tcW w:type="dxa" w:w="2000"/></w:tcPr><w:p><w:r><w:t>Kept row</w:t></w:r></w:p></w:tc></w:tr>
</w:tbl>
<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:rPr><w:moveFrom w:id="7" {REV}/></w:rPr></w:pPr><w:moveFromRangeStart w:id="8" {REV} w:name="move1"/><w:moveFrom w:id="9" {REV}><w:r><w:t>Heading</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="8"/></w:p>
<w:p/>"#
    );
    let rejected = reject_revisions(&docx(&body)).unwrap();
    assert_word_valid_package(&rejected);
    let xml = part_string(&rejected, "word/document.xml").unwrap();
    for kept in ["<w:tblPr>", "<w:tblGrid>", "<w:tcW"] {
        assert!(xml.contains(kept), "{kept} lost:\n{xml}");
    }
    for mark in ["w:moveFrom", "w:moveTo", "w:ins ", "w:del "] {
        assert!(!xml.contains(mark), "{mark} left:\n{xml}");
    }
    assert_eq!(paragraphs(&rejected), ["Before", "Kept row", "Heading", ""]);
}
