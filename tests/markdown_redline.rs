// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Word against Markdown: `apply_markdown` edits the Word document to read
//! as the Markdown, keeping what Markdown cannot say, and `redline`
//! compares Word and Markdown in either order.

mod common;

use common::docx::{docx, para, part_string, run};
use common::validity::assert_word_valid_package;
use jubarte::comparer::{WmlComparerRevisionType, WmlComparerSettings};
use jubarte::document_comparer::{accept_revisions, get_revisions, reject_revisions};
use jubarte::inspect;
use jubarte::markdown::{RedlineOptions, Source, apply_markdown, redline};
use jubarte::opc::PartFs;

fn texts(docx: &[u8]) -> Vec<String> {
    inspect::paragraphs(docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

/// A contract with what Markdown cannot say: empty paragraphs, a bold
/// lead-in, a hyperlink, typed numbering, a list and a table.
fn contract() -> Vec<u8> {
    let numbered = |text: &str| {
        format!(
            r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:p>"#
        )
    };
    let cell = |text: &str| {
        format!(
            r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr>{}</w:tc>"#,
            para(text)
        )
    };
    let body = [
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Sale Agreement</w:t></w:r></w:p>"#.to_string(),
        "<w:p/>".to_string(),
        format!(
            "<w:p>{}{}</w:p>",
            run("Parties. ", true, false, None),
            run("The Seller sells the goods to the Buyer.", false, false, None)
        ),
        "<w:p/>".to_string(),
        para("1.\tPrice. The price is ten thousand dollars."),
        para("2.\tDelivery. Delivery is within thirty days."),
        numbered("Invoices are due monthly."),
        numbered("Late payments bear interest."),
        r#"<w:p><w:r><w:t xml:space="preserve">See </w:t></w:r><w:hyperlink r:id="rId9"><w:r><w:t>the terms site</w:t></w:r></w:hyperlink><w:r><w:t xml:space="preserve"> for details.</w:t></w:r></w:p>"#.to_string(),
        format!("<w:tbl><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}{}</w:tr><w:tr>{}{}</w:tr></w:tbl>", cell("Item"), cell("Price"), cell("Goods"), cell("10000")),
        para("Signed by both parties."),
    ]
    .concat();
    // The link's relationship, which the plain test package lacks.
    let mut package = PartFs::open(&docx(&body)).unwrap();
    let id = package.add_document_relationship_external(
        "word/document.xml",
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
        "https://example.com",
    );
    let xml = package
        .part_string("word/document.xml")
        .unwrap()
        .replace("rId9", &id);
    package.set_part("word/document.xml", xml.into_bytes());
    let bytes = package.to_zip().unwrap();
    assert_word_valid_package(&bytes);
    bytes
}

/// The contract as Markdown, as a converter would write it.
const CONTRACT_MD: &str = "# Sale Agreement\n\n\
**Parties.** The Seller sells the goods to the Buyer.\n\n\
1. Price. The price is ten thousand dollars.\n\n\
2. Delivery. Delivery is within thirty days.\n\n\
- Invoices are due monthly.\n\
- Late payments bear interest.\n\n\
See [the terms site](https://example.com) for details.\n\n\
| Item | Price |\n|---|---|\n| Goods | 10000 |\n\n\
Signed by both parties.\n";

#[test]
fn unchanged_markdown_changes_nothing() {
    let source = contract();
    let patched = apply_markdown(&source, CONTRACT_MD).unwrap();
    assert!(patched.warnings.is_empty(), "{:?}", patched.warnings);
    assert_eq!(texts(&patched.docx), texts(&source));
}

#[test]
fn edits_land_on_the_document_and_keep_what_markdown_cannot_say() {
    let source = contract();
    let edited = CONTRACT_MD
        .replace("ten thousand", "twelve thousand")
        .replace("\n\n2. Delivery. Delivery is within thirty days.", "")
        .replace(
            "- Late payments bear interest.",
            "- Late payments bear interest.\n- Disputes go to arbitration.",
        )
        .replace("| Goods | 10000 |", "| Goods | 12000 |")
        .replace(
            "Signed by both parties.",
            "Governing law is New York.\n\nSigned by both parties.",
        );
    let patched = apply_markdown(&source, &edited).unwrap();
    assert!(patched.warnings.is_empty(), "{:?}", patched.warnings);
    assert_word_valid_package(&patched.docx);
    assert_eq!(
        texts(&patched.docx),
        [
            "Sale Agreement",
            "",
            "Parties. The Seller sells the goods to the Buyer.",
            "",
            "1.\tPrice. The price is twelve thousand dollars.",
            "Invoices are due monthly.",
            "Late payments bear interest.",
            "Disputes go to arbitration.",
            "See the terms site for details.",
            "Item",
            "Price",
            "Goods",
            "12000",
            "Governing law is New York.",
            "Signed by both parties.",
        ]
    );
    let paragraphs = inspect::paragraphs(&patched.docx).unwrap();
    // The bold lead-in, the list item's numbering and the link survive.
    assert!(paragraphs[2].runs.iter().any(|r| r.bold && r.start == 0));
    assert!(
        paragraphs[7].numbered,
        "a new item takes its neighbour's numbering"
    );
    assert!(!paragraphs[13].numbered, "a new paragraph does not");
    assert!(
        part_string(&patched.docx, "word/document.xml")
            .unwrap()
            .contains("<w:hyperlink")
    );
}

#[test]
fn a_redline_between_word_and_markdown_shows_only_the_edits() {
    let source = contract();
    let edited = CONTRACT_MD.replace("thirty days", "forty-five days");
    for (original, modified) in [
        (Source::Docx(&source), Source::Markdown(&edited)),
        (Source::Markdown(&edited), Source::Docx(&source)),
    ] {
        let docx = redline(original, modified, &RedlineOptions::default()).unwrap();
        assert_word_valid_package(&docx);
        let revisions = get_revisions(&docx, &WmlComparerSettings::default()).unwrap();
        let changed: Vec<String> = revisions
            .iter()
            .filter(|r| {
                matches!(
                    r.revision_type,
                    WmlComparerRevisionType::Inserted | WmlComparerRevisionType::Deleted
                )
            })
            .map(|r| r.text.clone().unwrap_or_default())
            .collect();
        assert!(
            changed
                .iter()
                .all(|t| t.contains("thirty") || t.contains("forty")),
            "{changed:?}"
        );
        assert!(!changed.is_empty());
        let (old, new) = match original {
            Source::Docx(_) => (texts(&source), texts(&accept_revisions(&docx).unwrap())),
            Source::Markdown(_) => (texts(&reject_revisions(&docx).unwrap()), texts(&source)),
        };
        assert_eq!(old.len(), new.len());
    }
}

#[test]
fn an_edit_inside_a_link_replaces_the_paragraph() {
    let source = contract();
    let edited = CONTRACT_MD.replace("[the terms site]", "[the new terms site]");
    let patched = apply_markdown(&source, &edited).unwrap();
    assert!(patched.warnings.is_empty(), "{:?}", patched.warnings);
    assert!(texts(&patched.docx).contains(&"See the new terms site for details.".to_string()));
}

#[test]
fn footnotes_are_reported_not_applied() {
    let source = contract();
    let edited = format!("{CONTRACT_MD}\n[^1]: A note.\n");
    let patched = apply_markdown(&source, &edited).unwrap();
    assert_eq!(
        patched.warnings,
        ["footnote text is not applied: only the body is edited"]
    );
}
