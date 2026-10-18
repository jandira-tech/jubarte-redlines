// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! Reader regressions for the one-step attribution migration.
mod common;
use common::docx::part_string;
use jubarte::markdown::{DocxOptions, markdown_to_docx};

fn package(text: &str) -> Vec<u8> {
    markdown_to_docx(text, &DocxOptions::default()).unwrap().docx
}

#[test]
fn tag_and_re_apply_to_the_old_side_only() {
    let bytes = package("{~~old~>new~~}{r4+5 Bob (2026-10-18T10:00:00Z) re r3 Alice (2026-10-17T10:00:00Z)}");
    let xml = part_string(&bytes, "word/document.xml").unwrap();
    assert!(xml.contains("w:id=\"4\""), "{xml}");
    assert!(xml.contains("w:author=\"Bob\""), "{xml}");
    assert!(xml.contains("old</w:delText>"), "{xml}");
    let old = xml.find("<w:del ").unwrap();
    let parent = xml[..old].rfind("<w:ins ").unwrap();
    assert!(xml[parent..old].contains("w:id=\"3\""), "{xml}");
    let new = xml.find("new</w:t>").unwrap();
    assert!(xml[old..new].contains("</w:del></w:ins><w:ins w:id=\"5\""), "new is outside parent: {xml}");
}

#[test]
fn inserted_paragraph_mark_is_not_text() {
    let bytes = package("Body¶{r7 Alice (2026-10-18T10:00:00Z)}");
    let xml = part_string(&bytes, "word/document.xml").unwrap();
    assert!(!xml.contains('¶'), "{xml}");
    assert!(xml.contains("<w:pPr><w:rPr><w:ins w:id=\"7\""), "{xml}");
}

#[test]
fn old_attribution_wrapper_is_a_comment() {
    let bytes = package("{++new++}{>>r7 @AL<<}");
    let xml = part_string(&bytes, "word/document.xml").unwrap();
    assert!(xml.contains("commentReference"), "{xml}");
    assert!(part_string(&bytes, "word/comments.xml").unwrap().contains("r7 @AL"));
}

#[test]
fn escaped_tags_are_literal_after_a_mark_or_pilcrow() {
    let bytes = package(r"{++new++}\{r3 @C} ¶\{r4 @B}");
    let xml = part_string(&bytes, "word/document.xml").unwrap();
    assert!(xml.contains("{r3 @C}"), "{xml}");
    assert!(xml.contains("¶{r4 @B}"), "{xml}");
}
