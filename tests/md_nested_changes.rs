// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only
//! A change inside a change (`w:ins` holding a `w:del`, and back): the view
//! splits the outer change around the inner one, each piece keeping the
//! outer change's id, author and date, so the writer nests the marks again
//! and both directions are fixed points (issue #517).

mod common;

use common::docx::{docx, part_string};
use common::validity::check_word_valid_package;
use jubarte::markdown::{
    DocxOptions, MarkdownOptions, ReadOptions, docx_to_markdown, markdown_to_docx, read,
};

/// `w:ins` by Ana holding a `w:del` by Ben between two runs.
fn del_in_ins() -> Vec<u8> {
    docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:r><w:t xml:space="preserve">one </w:t></w:r>\
           <w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">two</w:delText></w:r></w:del>\
           <w:r><w:t xml:space="preserve"> three</w:t></w:r></w:ins></w:p>"#,
    )
}

/// `w:del` by Ben holding a `w:ins` by Ana between two runs.
fn ins_in_del() -> Vec<u8> {
    docx(
        r#"<w:p><w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">\
           <w:r><w:delText xml:space="preserve">one </w:delText></w:r>\
           <w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">\
           <w:r><w:t xml:space="preserve">two</w:t></w:r></w:ins>\
           <w:r><w:delText xml:space="preserve"> three</w:delText></w:r></w:del></w:p>"#,
    )
}

/// The plain view (`--commonmark`) of `docx`.
fn plain(docx: &[u8]) -> String {
    docx_to_markdown(docx, &MarkdownOptions::default())
        .unwrap()
        .markdown
}

/// The agent view (`jubarte read`) of `docx`, without page markers.
fn agent(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .unwrap()
    .markdown
}

/// `markdown` written to Word and read back, each way.
fn written(markdown: &str) -> Vec<u8> {
    markdown_to_docx(markdown, &DocxOptions::default())
        .unwrap()
        .docx
}

#[test]
fn a_deletion_inside_an_insertion_splits_the_insertion() {
    assert_eq!(
        plain(&del_in_ins()),
        "{++one ++}{>>r1 Ana Lima (2026-09-29T14:05:00Z)<<}\
         {--two--}{>>r3 Ben Ode (2026-09-30T09:00:00Z)<<}\
         {++ three++}{>>r1 Ana Lima (2026-09-29T14:05:00Z)<<}\n"
    );
    let view = agent(&del_in_ins());
    assert!(
        view.contains("{++one ++}{>>r1 @AL<<}{--two--}{>>r3 @BO<<}{++ three++}{>>r1 @AL<<}"),
        "{view}"
    );
}

#[test]
fn an_insertion_inside_a_deletion_splits_the_deletion() {
    assert_eq!(
        plain(&ins_in_del()),
        "{--one --}{>>r3 Ben Ode (2026-09-30T09:00:00Z)<<}\
         {++two++}{>>r1 Ana Lima (2026-09-29T14:05:00Z)<<}\
         {-- three--}{>>r3 Ben Ode (2026-09-30T09:00:00Z)<<}\n"
    );
}

#[test]
fn the_split_writes_the_nested_marks_back() {
    let document = part_string(&written(&plain(&del_in_ins())), "word/document.xml").unwrap();
    // One `w:ins`, one `w:del` inside it, each with its id, author and date.
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    let ins = document.split("<w:ins ").nth(1).unwrap();
    assert!(
        ins.starts_with(r#"w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z""#),
        "{document}"
    );
    assert!(
        ins.contains(r#"<w:del w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z">"#),
        "{document}"
    );
    // The note was an attribution: no comment and no literal delimiters.
    assert!(!document.contains("commentReference"), "{document}");
    assert!(!document.contains("{--"), "{document}");
}

#[test]
fn a_deleted_insertion_writes_the_nested_marks_back() {
    let document = part_string(&written(&plain(&ins_in_del())), "word/document.xml").unwrap();
    assert_eq!(document.matches("<w:ins ").count(), 1, "{document}");
    assert_eq!(document.matches("<w:del ").count(), 1, "{document}");
    let del = document.split("<w:del ").nth(1).unwrap();
    assert!(
        del.starts_with(r#"w:id="3" w:author="Ben Ode" w:date="2026-09-30T09:00:00Z""#),
        "{document}"
    );
    assert!(
        del.contains(r#"<w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z">"#),
        "{document}"
    );
}

#[test]
fn nested_changes_round_trip_in_both_views() {
    for source in [del_in_ins(), ins_in_del()] {
        let markdown = plain(&source);
        assert_eq!(plain(&written(&markdown)), markdown, "{markdown}");
        let docx = written(&markdown);
        assert!(
            check_word_valid_package(&docx).ok(),
            "{markdown}: {}",
            check_word_valid_package(&docx).errors.join("\n")
        );
    }
}

#[test]
fn nested_changes_keep_both_authors_and_dates() {
    use jubarte::changes::{ChangeKind, list_changes};
    for (source, kinds) in [
        (del_in_ins(), [ChangeKind::Insertion, ChangeKind::Deletion]),
        (ins_in_del(), [ChangeKind::Deletion, ChangeKind::Insertion]),
    ] {
        let changes = list_changes(&written(&plain(&source))).unwrap();
        assert_eq!(
            changes.iter().map(|c| c.kind).collect::<Vec<_>>(),
            kinds,
            "{changes:?}"
        );
        let by_author = |name: &str| {
            changes
                .iter()
                .find(|c| c.author.as_deref() == Some(name))
                .unwrap_or_else(|| panic!("no change by {name}: {changes:?}"))
                .date
                .clone()
        };
        assert_eq!(
            by_author("Ana Lima").as_deref(),
            Some("2026-09-29T14:05:00Z")
        );
        assert_eq!(
            by_author("Ben Ode").as_deref(),
            Some("2026-09-30T09:00:00Z")
        );
    }
}
