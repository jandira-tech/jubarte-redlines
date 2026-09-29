// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! jubarte's redlines, accepted, end where Word's redlines of the same pair
//! end once Word accepts them (`_to_improve_accepted_changes`, 2026-09-28:
//! Word's redline and Word's Accept All of it against jubarte's redline and
//! Word's Accept All of that). Each rule was read off Word's redlines and is
//! pinned here with a synthetic pair.

mod common;

use common::docx::{Part, docx_with_sect, part_string};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, compare_documents};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const HEADER: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

/// The top-level paragraphs of `part` as (text, pPr xml).
fn story(pkg: &[u8], part: &str) -> Vec<(String, String)> {
    let xml = part_string(pkg, part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.elements(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            let text = dom
                .descendants(p, Some(&W::t()))
                .into_iter()
                .map(|t| dom.value(t))
                .collect();
            let ppr = dom
                .element(p, &W::p_pr())
                .map(|n| dom.serialize_element(n))
                .unwrap_or_default();
            (text, ppr)
        })
        .collect()
}

/// A header the revised document drops (it has no headers at all): Word
/// deletes the header's content and every paragraph mark but the last. The
/// last paragraph stays, its properties reset to the blank paragraph the
/// revised side implies and the old ones recorded in a `pPrChange`/
/// `rPrChange` (25f1d6ecc3, 5b878b9756, 5f0fed8e2a, 9c8876c6d7, ac6cd8d92f,
/// c46dee2ac8, f00b7032aa, ff42b4a7a3, 73105518ef, d8b0c2ae01). Accepted,
/// the header keeps one empty paragraph, as Word's accept of Word's redline
/// does; a deleted last mark left `<w:hdr/>`.
#[test]
fn a_dropped_header_keeps_its_last_paragraph_mark() {
    let header = format!(
        r#"<w:hdr xmlns:w="{w}"><w:p><w:r><w:t>Company</w:t></w:r></w:p><w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="2" w:space="1" w:color="E2E8F0"/></w:pBdr><w:jc w:val="center"/><w:rPr><w:b/></w:rPr></w:pPr><w:r><w:rPr><w:b/></w:rPr><w:t>Charter</w:t></w:r></w:p></w:hdr>"#,
        w = common::docx::W_NS
    );
    let base = docx_with_sect(
        r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#,
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let next = docx_with_sect(r#"<w:p><w:r><w:t>Body text</w:t></w:r></w:p>"#, &[], "");
    let redline = compare_documents(&base, &next, "Redline").unwrap();
    assert_word_valid_package(&redline);

    let paras = story(&redline, "word/header1.xml");
    assert_eq!(paras.len(), 2, "{paras:?}");
    let (first, last) = (&paras[0].1, &paras[1].1);
    assert!(first.contains("<w:del "), "first mark deleted: {first}");
    let (live, change) = last.split_once("<w:pPrChange").expect("pPrChange");
    assert!(!live.contains("<w:del "), "last mark stays: {last}");
    assert!(!live.contains("pBdr") && !live.contains("<w:jc"), "{last}");
    assert!(
        change.contains("pBdr") && change.contains("<w:jc"),
        "{last}"
    );
    assert!(
        live.contains("<w:rPrChange"),
        "mark formatting recorded: {last}"
    );

    let accepted = accept_revisions(&redline).unwrap();
    assert_word_valid_package(&accepted);
    let paras = story(&accepted, "word/header1.xml");
    assert_eq!(paras.len(), 1, "{paras:?}");
    assert_eq!(paras[0].0, "");
    assert!(!paras[0].1.contains("pBdr"), "{:?}", paras[0]);
}
