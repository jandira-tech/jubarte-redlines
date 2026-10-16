// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A header's or footer's fields through docx → view → docx: a bare
//! `{PAGE}` (a field with no result) must not take literal text that
//! follows it, nor the `{/}` of a field it sits in, and text that only
//! looks like a field stays text (pi on #502: F1, F2, P1, P2).

mod common;

use common::docx::{Part, docx_with_sect};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

const W: &str = common::docx::W_NS;

fn view(docx: &[u8]) -> String {
    let options = ReadOptions {
        page_markers: false,
        track_changes: TrackChanges::All,
        ..ReadOptions::default()
    };
    read(docx, &options)
        .expect("the docx reads")
        .markdown
        .lines()
        .filter(|l| !l.starts_with("sha256: ") && !l.starts_with("body: "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A docx whose default footer's one paragraph holds `inner`.
fn with_footer(inner: &str) -> Vec<u8> {
    let footer = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:ftr xmlns:w="{W}"><w:p>{inner}</w:p></w:ftr>"#
    );
    docx_with_sect(
        "<w:p><w:r><w:t>Body.</w:t></w:r></w:p>",
        &[Part {
            name: "word/footer1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
            xml: &footer,
        }],
        r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

/// The view of `docx`, the docx it writes, its footer, and that docx's
/// view.
fn round_trip(docx: &[u8]) -> (String, String, String) {
    let first = view(docx);
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let footer = common::docx::part_string(&again, "word/footer1.xml").expect("footer1.xml");
    let second = view(&again);
    (first, footer, second)
}

/// The view's footer line.
fn footer_line(view: &str) -> &str {
    view.lines()
        .find(|l| l.contains("id: footer1"))
        .unwrap_or_else(|| panic!("no footer line: {view}"))
}

#[test]
fn literal_field_end_after_a_bare_field_stays_text() {
    // F1: `{PAGE}` then the typed text `{/}`; the writer took the text
    // as the field's end and dropped it.
    let (first, footer, second) = round_trip(&with_footer(
        r#"<w:fldSimple w:instr=" PAGE "/><w:r><w:t>{/}</w:t></w:r>"#,
    ));
    assert!(footer.contains("{/}</w:t>"), "{first}\n{footer}");
    assert_eq!(second, first);
}

#[test]
fn a_field_inserted_in_a_fields_result_stays_in_it() {
    // F2: NUMPAGES inserted inside PAGE's result; its bare form left the
    // outer `{/}` unmatched, which came back as visible text.
    let (first, footer, second) = round_trip(&with_footer(
        r#"<w:fldSimple w:instr=" PAGE "><w:ins w:id="7" w:author="Ann Counsel" w:date="2026-10-01T09:00:00Z"><w:fldSimple w:instr=" NUMPAGES "/></w:ins></w:fldSimple>"#,
    ));
    assert!(!footer.contains("{/}"), "{first}\n{footer}");
    let page = footer.find(r#"w:instr=" PAGE ""#).expect("PAGE");
    let ins = footer.find("<w:ins ").expect("the insertion");
    let page_end = footer.find("</w:fldSimple>").expect("PAGE's end");
    assert!(page < ins && ins < page_end, "{footer}");
    assert_eq!(second, first);
}

#[test]
fn a_bare_field_nested_in_a_result_keeps_its_place() {
    let (first, footer, second) = round_trip(&with_footer(
        r#"<w:fldSimple w:instr=" PAGE "><w:fldSimple w:instr=" NUMPAGES "/></w:fldSimple><w:r><w:t xml:space="preserve"> of</w:t></w:r>"#,
    ));
    let page = footer.find(r#"w:instr=" PAGE ""#).expect("PAGE");
    let numpages = footer.find(r#"w:instr=" NUMPAGES ""#).expect("NUMPAGES");
    let page_end = footer.find("</w:fldSimple>").expect("PAGE's end");
    assert!(page < numpages && numpages < page_end, "{footer}");
    assert!(footer.contains(" of</w:t>"), "{footer}");
    assert_eq!(second, first);
}

#[test]
fn a_placeholder_in_capitals_stays_text() {
    // P1: `{SELLER}` typed in a footer is a placeholder, not a field.
    let (first, footer, second) = round_trip(&with_footer(
        r#"<w:r><w:t>Signed: {SELLER} and {1999}</w:t></w:r>"#,
    ));
    assert!(!footer.contains("fldSimple"), "{first}\n{footer}");
    assert!(footer.contains("{SELLER}"), "{footer}");
    assert!(footer.contains("{1999}"), "{footer}");
    assert_eq!(second, first);
}

#[test]
fn a_field_code_typed_in_a_result_stays_text() {
    // P2: the result's literal `{DATE}` is escaped; the writer counted it
    // as a nested field and lost the result's end.
    let (first, footer, second) = round_trip(&with_footer(
        r#"<w:fldSimple w:instr=" PAGE "><w:r><w:t>{DATE} x</w:t></w:r></w:fldSimple>"#,
    ));
    assert!(
        footer_line(&first).contains(r"{PAGE}\\{DATE} x{/}"),
        "{first}"
    );
    assert!(
        footer.contains("{DATE} x</w:t></w:r></w:fldSimple>"),
        "{footer}"
    );
    assert_eq!(second, first);
}

#[test]
fn a_bare_field_alone_stays_bare() {
    let (first, footer, second) = round_trip(&with_footer(
        r#"<w:r><w:t xml:space="preserve">Page </w:t></w:r><w:fldSimple w:instr=" PAGE "/>"#,
    ));
    assert!(footer_line(&first).contains("Page {PAGE}"), "{first}");
    assert!(
        footer.contains(r#"<w:fldSimple w:instr=" PAGE "/>"#),
        "{footer}"
    );
    assert_eq!(second, first);
}
