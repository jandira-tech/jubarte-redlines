// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Text edits must not borrow another paragraph's formatting history (#1149).

mod common;

use common::docx::{Part, W_NS, docx_with, para, part_string};
use common::validity::{assert_word_valid_package, check_word_valid_package};
use jubarte::changes::{ChangeKind, list_changes};
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

const LIST_PROPERTIES: &str = r#"<w:pStyle w:val="ListParagraph"/><w:numPr><w:ilvl w:val="1"/><w:numId w:val="1"/></w:numPr>"#;

fn clean_document(properties: &str) -> Vec<u8> {
    let properties = if properties.is_empty() {
        String::new()
    } else {
        format!("<w:pPr>{properties}</w:pPr>")
    };
    let body = format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Agreement</w:t></w:r></w:p><w:p>{properties}<w:r><w:t>Alice shall pay Bob within sixty days of receipt.</w:t></w:r></w:p>{}"#,
        para("Carol Diaz")
    );
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:styleId="ListParagraph"><w:name w:val="List Paragraph"/></w:style></w:styles>"#
    );
    let numbering = format!(
        r#"<w:numbering xmlns:w="{W_NS}"><w:abstractNum w:abstractNumId="0"><w:multiLevelType w:val="multilevel"/><w:lvl w:ilvl="1"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%2."/></w:lvl></w:abstractNum><w:num w:numId="1"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#
    );
    docx_with(
        &body,
        &[
            Part {
                name: "word/styles.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                xml: &styles,
            },
            Part {
                name: "word/numbering.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering",
                xml: &numbering,
            },
        ],
    )
}

fn paragraph_properties(bytes: &[u8], index: usize) -> Option<String> {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&part_string(bytes, "word/document.xml").unwrap());
    let p = dom.descendants(dom.root(doc).unwrap(), Some(&W::p()))[index];
    dom.element(p, &W::p_pr())
        .map(|ppr| dom.serialize_element(ppr))
}

fn assert_text_revisions_only(bytes: &[u8]) {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    assert!(
        !xml.contains("<w:pPrChange") && !xml.contains("<w:rPrChange"),
        "text-only changes must not invent formatting history: {xml}"
    );
    let changes = list_changes(bytes).unwrap();
    assert!(!changes.is_empty());
    assert!(
        changes
            .iter()
            .all(|c| matches!(c.kind, ChangeKind::Insertion | ChangeKind::Deletion)),
        "changes must list only insertions and deletions: {changes:?}"
    );
}

fn check_word_edit(properties: &str) {
    let source = clean_document(properties);
    assert!(list_changes(&source).unwrap().is_empty());
    let plan = EditPlan::from_json(
        r#"{"schema_version":1,"author":"Alice","date":"2026-01-02T03:04:05Z","operations":[{"kind":"replace","paragraph":"p1","find":"sixty","replacement":"thirty"}]}"#,
    ).unwrap();
    let out = apply_plan(&source, &plan).unwrap();
    assert_text_revisions_only(&out.redline);
    assert_word_valid_package(&out.redline);
    let changes = list_changes(&out.redline).unwrap();
    assert_eq!(
        changes
            .iter()
            .map(|c| (c.kind, c.text.as_str()))
            .collect::<Vec<_>>(),
        [
            (ChangeKind::Insertion, "thirty"),
            (ChangeKind::Deletion, "sixty")
        ]
    );
    let rejected = reject_revisions(&out.redline).unwrap();
    assert_eq!(
        paragraph_properties(&rejected, 1),
        paragraph_properties(&source, 1)
    );
    assert_eq!(
        jubarte::inspect::paragraphs(&rejected).unwrap()[1].text,
        "Alice shall pay Bob within sixty days of receipt."
    );
}

#[test]
fn word_edit_preserves_style_and_numbering_without_formatting_revisions() {
    check_word_edit(LIST_PROPERTIES);
}

#[test]
fn word_edit_without_paragraph_properties_has_only_text_revisions() {
    check_word_edit("");
}

#[test]
fn received_rejected_vs_accepted_has_only_real_text_revisions() {
    let received = include_bytes!("fixtures/agent-view/received.docx");
    let original = reject_revisions(received).unwrap();
    let revised = accept_revisions(received).unwrap();
    assert_eq!(paragraph_properties(&original, 3), None);
    assert_eq!(paragraph_properties(&revised, 3), None);
    let out = compare_documents(&original, &revised, "Alice").unwrap();
    assert_text_revisions_only(&out);
    // Accept/reject of this fixture already carries two commentsExtensible
    // metadata warnings. Compare must introduce no additional finding.
    let mut inherited = check_word_valid_package(&original).errors;
    let mut actual = check_word_valid_package(&out).errors;
    inherited.sort();
    actual.sort();
    assert_eq!(actual, inherited);
}
