// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `DocxOptions::page`: the page a Markdown document is written on when no
//! reference document gives one (US Letter by default, or A4).

mod common;

use common::docx::part_string;
use common::validity::assert_word_valid_package;
use jubarte::markdown::{DocxOptions, PageSize, markdown_to_docx};

const LETTER: &str = r#"<w:pgSz w:w="12240" w:h="15840"/>"#;
const A4: &str = r#"<w:pgSz w:w="11906" w:h="16838"/>"#;
const ONE_INCH: &str = r#"<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/>"#;

fn document_xml(page: PageSize) -> (String, Vec<String>) {
    let written = markdown_to_docx(
        "# T\n\ntext\n",
        &DocxOptions {
            page,
            ..DocxOptions::default()
        },
    )
    .unwrap();
    assert_word_valid_package(&written.docx);
    (
        part_string(&written.docx, "word/document.xml").unwrap(),
        written.warnings,
    )
}

#[test]
fn a4_writes_the_a4_section_and_letter_stays_default() {
    let (a4, warnings) = document_xml(PageSize::A4);
    assert!(a4.contains(A4), "{a4}");
    assert!(!a4.contains(LETTER), "{a4}");
    assert!(warnings.is_empty(), "{warnings:?}");

    let (letter, warnings) = document_xml(PageSize::Letter);
    assert!(letter.contains(LETTER), "{letter}");
    assert!(!letter.contains(A4), "{letter}");
    assert!(warnings.is_empty(), "{warnings:?}");

    let (default, _) = document_xml(PageSize::default());
    assert_eq!(default, letter, "the default page is US Letter");
}

#[test]
fn both_pages_keep_one_inch_margins() {
    // Word's own A4 template uses 2 cm margins; jubarte keeps the Letter
    // margins so a document only changes page size when the user asks.
    for page in [PageSize::Letter, PageSize::A4] {
        let (xml, _) = document_xml(page);
        assert!(xml.contains(ONE_INCH), "{page:?}: {xml}");
    }
}

#[test]
fn a_reference_documents_section_wins_over_page_with_a_warning() {
    let reference = std::fs::read("tests/fixtures/redline-inpi/original-new.docx").unwrap();
    let reference_xml = part_string(&reference, "word/document.xml").unwrap();
    let reference_width = reference_xml
        .find("<w:pgSz ")
        .map(|at| &reference_xml[at..at + reference_xml[at..].find("/>").unwrap() + 2])
        .expect("the fixture has a page size");
    assert!(!reference_width.contains("11906"), "{reference_width}");

    let written = markdown_to_docx(
        "Body.\n",
        &DocxOptions {
            reference: Some(&reference),
            page: PageSize::A4,
            ..DocxOptions::default()
        },
    )
    .unwrap();
    let xml = part_string(&written.docx, "word/document.xml").unwrap();
    assert!(xml.contains(reference_width), "{xml}");
    assert!(!xml.contains(A4), "{xml}");
    assert_eq!(
        written.warnings,
        ["page size a4 ignored: the reference document's page setup is used"]
    );

    // The default page with a reference raises no warning: nothing was asked.
    let written = markdown_to_docx(
        "Body.\n",
        &DocxOptions {
            reference: Some(&reference),
            ..DocxOptions::default()
        },
    )
    .unwrap();
    assert!(written.warnings.is_empty(), "{:?}", written.warnings);
}

#[test]
fn page_size_parses_its_cli_names() {
    assert_eq!(PageSize::parse("letter"), Some(PageSize::Letter));
    assert_eq!(PageSize::parse("a4"), Some(PageSize::A4));
    assert_eq!(PageSize::parse("A4"), None);
    assert_eq!(PageSize::parse("legal"), None);
    assert_eq!(PageSize::Letter.as_str(), "letter");
    assert_eq!(PageSize::A4.as_str(), "a4");
    assert_eq!(PageSize::default(), PageSize::Letter);
    assert_eq!(
        serde_json::from_str::<PageSize>("\"a4\"").unwrap(),
        PageSize::A4
    );
    assert_eq!(serde_json::to_string(&PageSize::Letter).unwrap(), "\"letter\"");
}
