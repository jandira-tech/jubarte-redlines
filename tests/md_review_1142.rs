// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only
//! Adversarial probes for the revision dialect and its secondary consumers.
mod common;
use jubarte::markdown::{
    DocxOptions, PatchOptions, ReadOptions, TrackChanges, markdown_to_docx, patch_critic, read,
    resolve_critic,
};

fn package(text: &str) -> Vec<u8> {
    markdown_to_docx(text, &DocxOptions::default())
        .unwrap()
        .docx
}
fn document(text: &str) -> String {
    common::docx::part_string(&package(text), "word/document.xml").unwrap()
}
fn story(text: &str) -> String {
    format!(
        "---\nview: tracked\nauthors:\n  A: Alice\nheaders:\n  default: {{id: header1, text: \"{text}\"}}\n---\n\nBody\n"
    )
}
#[test]
fn review_4_all_trailing_story_paragraph_marks_are_metadata() {
    let bytes = package(&story("A¶{header1.r4 @A}¶{header1.r5 @A}"));
    let xml = common::docx::part_string(&bytes, "word/header1.xml").unwrap();
    assert!(!xml.contains('¶'), "literal paragraph metadata: {xml}");
    for id in [4, 5] {
        assert!(xml.contains(&format!("<w:ins w:id=\"{id}\"")), "{xml}");
    }
    let view = read(&bytes, &ReadOptions::default()).unwrap().markdown;
    assert!(view.contains("A¶{header1.r4 @A}¶{header1.r5 @A}"), "{view}");
}
#[test]
fn review_6_crlf_and_lf_keep_the_same_empty_paragraphs() {
    for gap in ["\n\n", "\n\n\n\n", "\n\n\n\n\n\n"] {
        let lf = format!("A¶{{r1 @A}}{gap}B");
        assert_eq!(document(&lf), document(&lf.replace('\n', "\r\n")));
    }
}
#[test]
fn review_inline_crlf_rejection_joins_one_boundary() {
    assert_eq!(
        resolve_critic("A¶{r1 @A}\r\n\r\nB", TrackChanges::Reject),
        "AB"
    );
    assert_eq!(
        resolve_critic("A¶{r1 @A}\r\n\r\n\r\n\r\nB", TrackChanges::Reject),
        "A\r\n\r\nB"
    );
}
#[test]
fn review_inline_rejecting_a_paragraph_mark_removes_the_joined_list_prefix() {
    for (source, expected) in [
        ("- one¶{r1 @A}\n\n- two", "- onetwo"),
        ("1. one¶{r1 @A}\n\n2. two", "1. onetwo"),
        ("# one¶{r1 @A}\n\n## two", "# onetwo"),
    ] {
        assert_eq!(resolve_critic(source, TrackChanges::Reject), expected);
    }
}
#[test]
fn review_inline_patch_records_an_inserted_paragraph_boundary() {
    let patch = patch_critic(
        "A¶{r1 Alice}\n\nB",
        &PatchOptions {
            old_name: "old.md".into(),
            new_name: "new.md".into(),
            owner: jubarte::markdown::Attribution {
                author: "Bob".into(),
                date: String::new(),
            },
        },
    );
    assert_eq!(patch.hunks.len(), 1, "{patch:?}");
    let change = &patch.hunks[0].changes[0];
    assert_eq!((change.old.as_str(), change.new.as_str()), ("", "\n\n"));
    assert_eq!(change.author, "Alice");
}
#[test]
fn review_inline_a_literal_code_closer_does_not_eat_following_text() {
    let source = "`{++x++}`{r1 Alice}";
    for mode in [TrackChanges::Accept, TrackChanges::Reject] {
        assert_eq!(resolve_critic(source, mode), source);
    }
    assert!(document(source).contains("{r1 Alice}"));
}
#[test]
fn review_inline_rejecting_a_parented_substitution_drops_its_old_side() {
    let source = "Before {~~old~>new~~}{r4+5 Bob re r3 Alice} after";
    assert_eq!(
        resolve_critic(source, TrackChanges::Reject),
        "Before  after"
    );
    assert_eq!(
        resolve_critic(source, TrackChanges::Accept),
        "Before new after"
    );
}
#[test]
fn review_ranges_ending_on_a_marked_paragraph_are_balanced() {
    let bytes = package("{==A\n\nB==}{>>c0 @A: note<<}¶{r9 @A}");
    let xml = common::docx::part_string(&bytes, "word/document.xml").unwrap();
    assert_eq!(xml.matches("<w:commentRangeStart ").count(), 1, "{xml}");
    assert_eq!(xml.matches("<w:commentRangeEnd ").count(), 1, "{xml}");
    assert_eq!(xml.matches("<w:commentReference ").count(), 1, "{xml}");
    assert!(xml.contains("<w:ins w:id=\"9\""), "{xml}");
    common::validity::assert_word_valid_package(&bytes);
}

#[test]
fn review_inline_a_missing_date_does_not_share_a_dated_attribution() {
    let xml = r#"<w:p><w:del w:id="1" w:author="Alice" w:date="2026-10-01T09:00:00Z"><w:r><w:delText>old</w:delText></w:r></w:del><w:ins w:id="2" w:author="Alice"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#;
    let source = common::docx::docx(xml);
    let view = read(
        &source,
        &ReadOptions {
            dates: true,
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(!view.contains("{~~"), "{view}");
    assert!(view.contains("{--old--}{r1 @A}{++new++}{r2 @A}"), "{view}");
}

#[test]
fn review_default_view_keeps_pairing_when_per_revision_dates_are_omitted() {
    let xml = r#"<w:p><w:del w:id="1" w:author="Alice" w:date="2026-10-01T09:00:00Z"><w:r><w:delText>old</w:delText></w:r></w:del><w:ins w:id="2" w:author="Alice" w:date="2026-10-02T09:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#;
    let source = common::docx::docx(xml);
    let view = read(
        &source,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(view.contains("{~~old~>new~~}{r1+2 @A}"), "{view}");
}
