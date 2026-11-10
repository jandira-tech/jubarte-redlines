// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A copied list definition keeps a unique durable id, and a num both
//! documents share over the same definition stays the source's num.

mod common;

use common::docx::{Part, docx_with};
use jubarte::comparer::footnotes::copy_missing_numbering;
use jubarte::namespaces::W;
use jubarte::opc::PartFs;
use jubarte::validate::ring1;
use jubarte::xmllinq::Dom;

const CID: &str = "http://schemas.microsoft.com/office/word/2016/wordml/cid";

fn abstract_num(format: &str) -> String {
    format!(
        r#"<w:abstractNum w:abstractNumId="0"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="{format}"/><w:lvlText w:val="%1."/></w:lvl></w:abstractNum>"#
    )
}

fn num(id: &str, durable: &str) -> String {
    format!(
        r#"<w:num w:numId="{id}" w16cid:durableId="{durable}"><w:abstractNumId w:val="0"/></w:num>"#
    )
}

fn numbering(inner: &str) -> String {
    format!(
        r#"<w:numbering xmlns:w="{}" xmlns:w16cid="{CID}">{inner}</w:numbering>"#,
        W::URI
    )
}

fn roots(
    to_inner: &str,
    from_inner: &str,
) -> (Dom, jubarte::xmllinq::NodeId, jubarte::xmllinq::NodeId) {
    let mut dom = Dom::new();
    let to_doc = dom.parse_xdocument(&numbering(to_inner));
    let from_doc = dom.parse_xdocument(&numbering(from_inner));
    let to = dom.root(to_doc).expect("destination numbering");
    let from = dom.root(from_doc).expect("source numbering");
    (dom, to, from)
}

#[test]
fn compare_keeps_shared_numbering_by_identity() {
    let (mut dom, to, from) = roots(
        &format!("{}{}", abstract_num("decimal"), num("1", "100")),
        &format!("{}{}", abstract_num("decimal"), num("1", "100")),
    );
    let (remap, _) = copy_missing_numbering(&mut dom, to, from);
    assert!(
        remap.is_empty(),
        "the same num over the same definition stays put: {remap:?}"
    );
    let xml = dom.serialize_element(to);
    assert_eq!(xml.matches("<w:num ").count(), 1, "{xml}");
}

/// A num whose definition changed is a list edit, not a shared num: Word
/// moves the unchanged items onto the revised list with a `pPrChange`
/// (m_list_definition_change), so the comparer copies it under a fresh
/// id with its own durable id (cursor 4501780818).
#[test]
fn a_changed_definition_under_the_same_ids_is_copied() {
    let (mut dom, to, from) = roots(
        &format!("{}{}", abstract_num("decimal"), num("1", "100")),
        &format!("{}{}", abstract_num("bullet"), num("1", "100")),
    );
    let (remap, _) = copy_missing_numbering(&mut dom, to, from);
    assert_eq!(remap.get("1").map(String::as_str), Some("2"), "{remap:?}");
    let xml = dom.serialize_element(to);
    assert_eq!(xml.matches("<w:num ").count(), 2, "{xml}");
    assert_eq!(xml.matches("durableId=\"100\"").count(), 1, "{xml}");
}

#[test]
fn cloned_num_gets_a_fresh_durable_id() {
    let (mut dom, to, from) = roots(
        &format!("{}{}", abstract_num("decimal"), num("1", "100")),
        &format!("{}{}", abstract_num("decimal"), num("2", "100")),
    );
    let _ = copy_missing_numbering(&mut dom, to, from);
    let xml = dom.serialize_element(to);
    assert_eq!(xml.matches("<w:num ").count(), 2, "{xml}");
    assert_eq!(
        xml.matches("durableId=\"100\"").count(),
        1,
        "the copy must not repeat the destination's durable id:\n{xml}"
    );
}

#[test]
fn ring1_flags_duplicate_durable_ids() {
    let numbering = numbering(&format!(
        "{}{}{}",
        abstract_num("decimal"),
        num("1", "100"),
        num("2", "100")
    ));
    let bytes = docx_with(
        "<w:p><w:r><w:t>Hi</w:t></w:r></w:p>",
        &[Part {
            name: "word/numbering.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
            xml: &numbering,
        }],
    );
    let pkg = PartFs::open(&bytes).expect("open");
    let findings = ring1(&pkg);
    assert!(
        findings
            .iter()
            .any(|f| f.code == "DUPLICATE_DURABLE_ID" && !f.word_fatal),
        "{findings:?}"
    );
}
