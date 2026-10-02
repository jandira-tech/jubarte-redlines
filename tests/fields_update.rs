// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `fields::update_fields`: cached field results written back from
//! jubarte's own layout.

mod common;

use common::docx::{Part, docx, docx_with, docx_with_sect, part_string};
use common::validity::assert_word_valid_package;
use jubarte::fields::{BOOKMARK_NOT_DEFINED, NO_TOC_ENTRIES, REFERENCE_NOT_FOUND, update_fields};
use jubarte::inspect::paragraphs;

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";

fn heading(text: &str) -> String {
    format!(r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>"#)
}

const PAGE_BREAK: &str = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;

/// A complex field with an optional cached result.
fn field(code: &str, result: Option<&str>) -> String {
    let result = result.map_or(String::new(), |r| {
        format!(r#"<w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:rPr><w:b/></w:rPr><w:t xml:space="preserve">{r}</w:t></w:r>"#)
    });
    format!(
        r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> {code} </w:instrText></w:r>{result}<w:r><w:fldChar w:fldCharType="end"/></w:r>"#
    )
}

fn texts(docx: &[u8]) -> Vec<String> {
    paragraphs(docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn two_headings_and_an_empty_toc() -> Vec<u8> {
    // A TOC with no cached result, Heading1 "A", a page break, Heading1 "B".
    let body = String::new()
        + r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TOC \o "1-3" \h </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#
        + &heading("A")
        + PAGE_BREAK
        + &heading("B")
        + r#"<w:p><w:r><w:t xml:space="preserve">Pages: </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
    docx(&body)
}

#[test]
fn toc_entries_and_numpages_come_from_the_layout() {
    // `docx()` writes no styles part: the layout's built-in Heading1 and the
    // explicit page break decide the pages.
    let updated = update_fields(&two_headings_and_an_empty_toc()).unwrap();
    assert_word_valid_package(&updated.docx);
    let texts = texts(&updated.docx);
    // The TOC paragraph grew into entries: "A\t1" and "B\t2".
    assert!(texts.iter().any(|t| t == "A\t1"), "{texts:?}");
    assert!(texts.iter().any(|t| t == "B\t2"), "{texts:?}");
    assert!(texts.last().unwrap().ends_with("Pages: 2"), "{texts:?}");
    assert_eq!(updated.page_count, 2);
    assert_eq!(
        updated
            .fields
            .iter()
            .filter(|f| f.kind == "PAGEREF")
            .count(),
        2
    );
    assert_eq!(
        updated
            .fields
            .iter()
            .filter(|f| f.kind == "NUMPAGES")
            .count(),
        1
    );
    let toc = updated.fields.iter().find(|f| f.kind == "TOC").unwrap();
    assert_eq!((toc.old.as_str(), toc.new.as_str()), ("", "A\t1\nB\t2"));
    assert_eq!(toc.paragraph, "body:p:0");
    // Field codes are intact so Word can refresh.
    let xml = part_string(&updated.docx, "word/document.xml").unwrap();
    assert!(xml.contains(r#" TOC \o "1-3" \h "#) && xml.contains(" NUMPAGES "));
    // Each heading carries its `_Toc` bookmark inside the paragraph, and
    // each entry links to it.
    assert_eq!(xml.matches("<w:bookmarkStart").count(), 2, "{xml}");
    assert_eq!(xml.matches("w:anchor=\"_Toc").count(), 2, "{xml}");
    assert!(xml.contains("w:leader=\"dot\""), "{xml}");
    assert!(xml.contains("w:pStyle w:val=\"TOC1\""), "{xml}");
    assert!(!xml.contains("updateFields"));
}

#[test]
fn a_second_refresh_changes_nothing() {
    let once = update_fields(&two_headings_and_an_empty_toc()).unwrap();
    let twice = update_fields(&once.docx).unwrap();
    assert_eq!(texts(&once.docx), texts(&twice.docx));
    assert_eq!(
        part_string(&once.docx, "word/document.xml"),
        part_string(&twice.docx, "word/document.xml")
    );
}

#[test]
fn a_stale_toc_spanning_paragraphs_is_rebuilt() {
    let body = String::new()
        + r#"<w:p><w:pPr><w:pStyle w:val="TOC1"/></w:pPr><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TOC \o "1-1" </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>Old one</w:t></w:r></w:p>"#
        + r#"<w:p><w:pPr><w:pStyle w:val="TOC1"/></w:pPr><w:r><w:t>Old two</w:t></w:r></w:p>"#
        + r#"<w:p><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#
        + &heading("Alpha")
        + r#"<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Below the levels</w:t></w:r></w:p>"#;
    let updated = update_fields(&docx(&body)).unwrap();
    assert_word_valid_package(&updated.docx);
    let texts = texts(&updated.docx);
    assert_eq!(texts[0], "Alpha\t1", "{texts:?}");
    assert_eq!(texts[1], "", "the end mark keeps its paragraph: {texts:?}");
    assert!(!texts.iter().any(|t| t.starts_with("Old")), "{texts:?}");
    let toc = updated.fields.iter().find(|f| f.kind == "TOC").unwrap();
    assert_eq!(toc.old, "Old one\nOld two");
    // No `\h`: no hyperlink.
    let xml = part_string(&updated.docx, "word/document.xml").unwrap();
    assert!(!xml.contains("w:hyperlink"), "{xml}");
}

#[test]
fn a_toc_without_headings_says_so() {
    let body = format!("<w:p>{}</w:p>", field(r#"TOC \o "1-3""#, None));
    let updated = update_fields(&docx(&body)).unwrap();
    assert_word_valid_package(&updated.docx);
    assert_eq!(texts(&updated.docx), [NO_TOC_ENTRIES]);
}

#[test]
fn toc_switches_it_does_not_implement_keep_the_cached_toc() {
    let body = format!(
        "<w:p>{}</w:p>{}",
        field(r#"TOC \t "Custom,1""#, Some("kept")),
        heading("A")
    );
    let updated = update_fields(&docx(&body)).unwrap();
    assert_eq!(texts(&updated.docx), ["kept", "A"]);
    assert!(updated.fields.is_empty());
}

#[test]
fn toc_styles_are_added_to_an_existing_styles_part() {
    let styles = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Titre2"><w:name w:val="heading 2"/><w:basedOn w:val="Normal"/></w:style></w:styles>"#;
    let body = format!(
        "<w:p>{}</w:p>{}<w:p><w:pPr><w:pStyle w:val=\"Titre2\"/></w:pPr><w:r><w:t>Sub</w:t></w:r></w:p>",
        field(r#"TOC \o "1-3" \h"#, None),
        heading("Top")
    );
    let updated = update_fields(&docx_with(
        &body,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: styles,
        }],
    ))
    .unwrap();
    assert_word_valid_package(&updated.docx);
    let texts = texts(&updated.docx);
    assert_eq!(&texts[..2], ["Top\t1", "Sub\t1"], "{texts:?}");
    let styles = part_string(&updated.docx, "word/styles.xml").unwrap();
    for id in ["TOC1", "TOC2", "TOCHeading"] {
        assert!(
            styles.contains(&format!("w:styleId=\"{id}\"")),
            "{id}: {styles}"
        );
    }
    assert!(!styles.contains("w:styleId=\"TOC3\""));
    let xml = part_string(&updated.docx, "word/document.xml").unwrap();
    assert!(xml.contains("w:pStyle w:val=\"TOC2\""), "{xml}");
}

#[test]
fn pagerefs_distinguish_undefined_bookmarks_from_unpaged_ones() {
    let body = String::new()
        + r#"<w:p><w:bookmarkStart w:id="1" w:name="target"/><w:r><w:t>Target</w:t></w:r><w:bookmarkEnd w:id="1"/></w:p>"#
        + PAGE_BREAK
        + &format!("<w:p>{}</w:p>", field("PAGEREF target \\h", Some("9")))
        + &format!("<w:p>{}</w:p>", field("PAGEREF missing", Some("9")))
        // Defined, but outside any paragraph: the layout cannot page it.
        + r#"<w:bookmarkStart w:id="2" w:name="loose"/><w:bookmarkEnd w:id="2"/>"#
        + &format!("<w:p>{}</w:p>", field("PAGEREF loose", Some("cached")))
        // `\p` (above/below) is not implemented: cached result stays.
        + &format!("<w:p>{}</w:p>", field("PAGEREF target \\p", Some("above")));
    let updated = update_fields(&docx(&body)).unwrap();
    assert_word_valid_package(&updated.docx);
    let texts = texts(&updated.docx);
    assert_eq!(
        &texts[2..],
        ["1", BOOKMARK_NOT_DEFINED, "cached", "above"],
        "{texts:?}"
    );
    let first = &updated.fields[0];
    assert_eq!(
        (
            first.kind.as_str(),
            first.code.as_str(),
            first.old.as_str(),
            first.new.as_str(),
            first.paragraph.as_str()
        ),
        ("PAGEREF", "PAGEREF target \\h", "9", "1", "body:p:2")
    );
    // The result run keeps the first result run's formatting.
    let xml = part_string(&updated.docx, "word/document.xml").unwrap();
    assert!(
        xml.contains("<w:rPr><w:b /></w:rPr><w:t xml:space=\"preserve\">1</w:t>"),
        "{xml}"
    );
}

#[test]
fn refs_copy_the_bookmarked_text() {
    let body = String::new()
        + r#"<w:p><w:r><w:t xml:space="preserve">See </w:t></w:r><w:bookmarkStart w:id="1" w:name="clause"/><w:r><w:t>Clause 4</w:t></w:r><w:bookmarkEnd w:id="1"/></w:p>"#
        + &format!("<w:p>{}</w:p>", field("REF clause \\h", Some("old")))
        + &format!("<w:p>{}</w:p>", field("REF nowhere", None))
        + &format!("<w:p>{}</w:p>", field("REF clause \\n", Some("4.")));
    let updated = update_fields(&docx(&body)).unwrap();
    assert_word_valid_package(&updated.docx);
    assert_eq!(
        &texts(&updated.docx)[1..],
        ["Clause 4", REFERENCE_NOT_FOUND, "4."]
    );
}

#[test]
fn seq_fields_count_per_identifier() {
    let body = String::new()
        + &format!(
            "<w:p><w:r><w:t xml:space=\"preserve\">Figure </w:t></w:r>{}</w:p>",
            field("SEQ Figure \\* ARABIC", Some("7"))
        )
        + &format!("<w:p>{}</w:p>", field("SEQ Table", Some("7")))
        + &format!(
            "<w:p><w:r><w:t xml:space=\"preserve\">Figure </w:t></w:r>{}</w:p>",
            field("SEQ Figure", Some("7"))
        )
        + r#"<w:p><w:fldSimple w:instr=" SEQ Figure "><w:r><w:t>7</w:t></w:r></w:fldSimple></w:p>"#;
    let updated = update_fields(&docx(&body)).unwrap();
    assert_word_valid_package(&updated.docx);
    assert_eq!(texts(&updated.docx), ["Figure 1", "1", "Figure 2", "3"]);
}

#[test]
fn numpages_in_a_footer_and_in_a_simple_field() {
    let footer = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t xml:space="preserve">of </w:t></w:r><w:fldSimple w:instr=" NUMPAGES "><w:r><w:t>1</w:t></w:r></w:fldSimple></w:p></w:ftr>"#;
    let body = String::new()
        + "<w:p><w:r><w:t>One</w:t></w:r></w:p>"
        + PAGE_BREAK
        + "<w:p><w:r><w:t>Two</w:t></w:r></w:p>"
        + PAGE_BREAK
        + &format!("<w:p>{}</w:p>", field("NUMPAGES \\* roman", Some("i")));
    let updated = update_fields(&docx_with_sect(
        &body,
        &[Part {
            name: "word/footer1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
            xml: footer,
        }],
        r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
    ))
    .unwrap();
    assert_word_valid_package(&updated.docx);
    assert_eq!(updated.page_count, 3);
    let footer = part_string(&updated.docx, "word/footer1.xml").unwrap();
    assert!(footer.contains(">3</w:t>"), "{footer}");
    let numpages = updated
        .fields
        .iter()
        .find(|f| f.kind == "NUMPAGES")
        .unwrap();
    assert_eq!(numpages.paragraph, "footer1:p:0");
    // A number format it does not write keeps the cached result.
    assert_eq!(texts(&updated.docx).last().unwrap(), "i");
}

#[test]
fn a_package_without_fields_comes_back_with_the_same_text() {
    let source = docx("<w:p><w:r><w:t>Plain</w:t></w:r></w:p>");
    let updated = update_fields(&source).unwrap();
    assert!(updated.fields.is_empty());
    assert_eq!(texts(&updated.docx), ["Plain"]);
    assert_eq!(
        part_string(&updated.docx, "word/document.xml"),
        part_string(&source, "word/document.xml")
    );
}

#[test]
fn a_package_it_cannot_read_is_an_error() {
    let err = update_fields(b"not a zip").unwrap_err();
    assert!(err.to_string().contains("DOCX"), "{err}");
}

#[test]
fn field_marks_sharing_one_run_are_split_and_refreshed() {
    // Generators (python-docx scripts, Google Docs exports) put a whole
    // field in one run.
    let body = String::new()
        + r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/><w:instrText xml:space="preserve">TOC \o "1-3" \h \z \u</w:instrText><w:fldChar w:fldCharType="separate"/><w:fldChar w:fldCharType="end"/></w:r></w:p>"#
        + &heading("Only")
        + r#"<w:p><w:r><w:rPr><w:i/></w:rPr><w:fldChar w:fldCharType="begin"/><w:instrText> NUMPAGES </w:instrText><w:fldChar w:fldCharType="separate"/><w:t>9</w:t><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
    let updated = update_fields(&docx(&body)).unwrap();
    assert_word_valid_package(&updated.docx);
    assert_eq!(texts(&updated.docx), ["Only\t1", "Only", "1"]);
    let xml = part_string(&updated.docx, "word/document.xml").unwrap();
    // Each split run keeps the original run's formatting.
    assert!(
        xml.contains(r#"<w:r><w:rPr><w:i /></w:rPr><w:instrText>"#),
        "{xml}"
    );
}
