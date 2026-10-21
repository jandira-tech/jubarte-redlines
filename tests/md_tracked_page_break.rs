// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A page break inside a tracked insertion or deletion keeps its change
//! through docx → view → docx: the id line names it (`page-break-del r5
//! @SW`), where `rev r5 @SW` said only that some change had no text.
//! ids-fields.md (791ca61933.docx) lost a deleted page break this way.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

const SW: &str = r#"w:author="S Wheeler" w:date="2026-09-25T20:04:00Z""#;

fn view(docx: &[u8], track_changes: TrackChanges) -> String {
    let options = ReadOptions {
        page_markers: false,
        track_changes,
        ..ReadOptions::default()
    };
    read(docx, &options)
        .expect("the docx reads")
        .markdown
        .lines()
        // The page count is measured, not written (as md_fixed_point).
        .filter(|l| !l.starts_with("sha256: ") && !l.starts_with("body: "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The tracked view of `body`, the docx it writes, and that docx's view.
fn round_trip(body: &str) -> (String, Vec<u8>, String) {
    let first = view(&common::docx::docx(body), TrackChanges::All);
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let second = view(&again, TrackChanges::All);
    (first, again, second)
}

fn document(docx: &[u8]) -> String {
    common::docx::part_string(docx, "word/document.xml").expect("document.xml")
}

/// A paragraph whose mark is deleted (r4) and whose only content is a
/// page break in `wrapper` (`del` or `ins`, r5), then a plain one.
fn break_paragraph(wrapper: &str, extra: &str) -> String {
    format!(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="4" {SW}/></w:rPr></w:pPr><w:{wrapper} w:id="5" {SW}>{extra}<w:r><w:br w:type="page"/></w:r></w:{wrapper}></w:p><w:p><w:r><w:t>After.</w:t></w:r></w:p>"#
    )
}

#[test]
fn a_deleted_page_break_names_its_deletion() {
    let (first, again, second) = round_trip(&break_paragraph("del", ""));
    assert!(
        first.contains("<!-- p0 page-break-del r5 @SW, break-del r4 @SW -->"),
        "{first}"
    );
    assert!(!first.contains("rev r5"), "{first}");
    let xml = document(&again);
    assert!(
        xml.contains(r#"<w:del w:id="5""#)
            && xml.contains(r#"<w:br w:type="page"/></w:r></w:del>"#),
        "{xml}"
    );
    assert_eq!(second, first);
}

#[test]
fn an_inserted_page_break_names_its_insertion() {
    let (first, again, second) = round_trip(&break_paragraph("ins", ""));
    assert!(first.contains("page-break-ins r5 @SW"), "{first}");
    assert!(!first.contains("rev r5"), "{first}");
    let xml = document(&again);
    assert!(
        xml.contains(r#"<w:ins w:id="5""#)
            && xml.contains(r#"<w:br w:type="page"/></w:r></w:ins>"#),
        "{xml}"
    );
    assert_eq!(second, first);
}

#[test]
fn a_page_break_deleted_with_text_shares_the_texts_deletion() {
    let extra = r#"<w:r><w:delText>Gone.</w:delText></w:r>"#;
    let (first, again, second) = round_trip(&break_paragraph("del", extra));
    assert!(first.contains("page-break-del r5 @SW"), "{first}");
    assert!(first.contains("{--Gone.--}{>>r5 @SW<<}"), "{first}");
    let xml = document(&again);
    assert_eq!(xml.matches(r#"<w:del w:id="5""#).count(), 1, "{xml}");
    assert_eq!(second, first);
}

#[test]
fn an_untracked_page_break_stays_plain() {
    let body = r#"<w:p><w:r><w:t>Before.</w:t></w:r><w:r><w:br w:type="page"/></w:r></w:p><w:p><w:r><w:t>After.</w:t></w:r></w:p>"#;
    let (first, _, second) = round_trip(body);
    assert!(first.contains("<!-- p0 page-break -->"), "{first}");
    assert_eq!(second, first);
}

#[test]
fn resolved_views_name_no_tracked_page_break() {
    let docx = common::docx::docx(&break_paragraph("del", ""));
    for track_changes in [TrackChanges::Accept, TrackChanges::Reject] {
        let view = view(&docx, track_changes);
        assert!(!view.contains("page-break-del"), "{view}");
        assert!(!view.contains("page-break-ins"), "{view}");
    }
}
