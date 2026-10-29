// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1898: two comments that say the same thing (same author, date and text)
//! are two comments. The comparer dropped one with an empty range when
//! another with a range said the same, in `compare` (both modes) and in the
//! redline of `add` and `edit`: `1672057675`'s c0 and its reply c1, both on
//! the start of a paragraph, vanished from its redline, though the clean
//! copy kept them, because c4 and c5 say the same over later text.

mod common;

use common::docx::{Part, W_NS, docx_with, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::comparer::WmlComparerSettings;
use jubarte::document_comparer::{
    accept_revisions, compare_documents_with_settings, reject_revisions,
};
use jubarte::edit::{EditPlan, apply_plan};

fn modes() -> [(&'static str, WmlComparerSettings); 2] {
    [
        ("word", WmlComparerSettings::default()),
        ("powertools", WmlComparerSettings::powertools_faithful()),
    ]
}

/// Comment `id` saying `text`, by one author at one time: two such comments
/// share their definition.
fn comment(id: u32, text: &str) -> String {
    format!(
        r#"<w:comment w:id="{id}" w:author="Arthur Souza Rodrigues" w:date="2026-07-04T23:17:00Z" w:initials="ASR"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#
    )
}

/// A paragraph opening with comment `id`'s empty range and its reference,
/// then `text`, as `1672057675` holds c0.
fn point(id: u32, text: &str) -> String {
    format!(
        r#"<w:p><w:commentRangeStart w:id="{id}"/><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r><w:r><w:t>{text}</w:t></w:r></w:p>"#
    )
}

/// A paragraph whose `text` is comment `id`'s range.
fn ranged(id: u32, text: &str) -> String {
    format!(
        r#"<w:p><w:commentRangeStart w:id="{id}"/><w:r><w:t>{text}</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r></w:p>"#
    )
}

fn with_comments(body: &str, comments: &[String]) -> Vec<u8> {
    let xml = format!(
        r#"<w:comments xmlns:w="{W_NS}">{}</w:comments>"#,
        comments.concat()
    );
    docx_with(
        body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &xml,
        }],
    )
}

/// "Title", c0 at the start of "Body text.", c4 over "Sign-off text.", and
/// `tail`; c0 and c4 both say "Horrible.".
fn twins(tail: &str) -> Vec<u8> {
    with_comments(
        &format!(
            "{}{}{}{tail}",
            para("Title"),
            point(0, "Body text."),
            ranged(4, "Sign-off text.")
        ),
        &[comment(0, "Horrible."), comment(4, "Horrible.")],
    )
}

/// The comment ids `docx` defines and the ids its body references, sorted.
fn comments(docx: &[u8]) -> (Vec<String>, Vec<String>) {
    let ids = |xml: &str, tag: &str| {
        let mut out: Vec<String> = xml
            .match_indices(tag)
            .map(|(at, _)| {
                let rest = &xml[at + tag.len()..];
                let rest = &rest[rest.find("w:id=\"").unwrap() + 6..];
                rest[..rest.find('"').unwrap()].to_string()
            })
            .collect();
        out.sort();
        out
    };
    let defined = part_string(docx, "word/comments.xml")
        .map(|x| ids(&x, "<w:comment "))
        .unwrap_or_default();
    let referenced = ids(
        &part_string(docx, "word/document.xml").unwrap(),
        "<w:commentReference ",
    );
    (defined, referenced)
}

/// Both comments, each defined once and referenced once.
fn assert_both(docx: &[u8], view: &str) {
    let (defined, referenced) = comments(docx);
    assert_eq!(defined.len(), 2, "{view}: {defined:?}");
    assert_eq!(defined, referenced, "{view}");
}

/// The issue's minimal pair: a paragraph added at the end leaves both
/// comments in the redline, Accept All and Reject All.
#[test]
fn a_redline_keeps_both_comments_that_say_the_same() {
    let original = twins("");
    let revised = twins(&para("Probe paragraph."));
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        assert_both(&redline, &format!("{mode}, redline"));
        assert_both(
            &accept_revisions(&redline).unwrap(),
            &format!("{mode}, accept"),
        );
        assert_both(
            &reject_revisions(&redline).unwrap(),
            &format!("{mode}, reject"),
        );
    }
}

/// An `add` keeps both in the clean copy, the redline and its Accept All.
#[test]
fn an_add_keeps_both_comments_that_say_the_same() {
    let edited = apply_plan(
        &twins(""),
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":"p2","runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    assert_both(&edited.clean, "clean");
    assert_word_valid_package(&edited.redline);
    assert_both(&edited.redline, "redline");
    assert_both(&accept_revisions(&edited.redline).unwrap(), "accept");
}

/// A comment the revision removed is treated alike whether or not another
/// comment says the same: a twin is no reason to drop it, nor to keep it.
#[test]
fn a_removed_comment_fares_as_it_would_without_a_twin() {
    let revised = with_comments(
        &format!(
            "{}{}{}",
            para("Title"),
            para("Body text."),
            ranged(4, "Sign-off text.")
        ),
        &[comment(4, "Horrible.")],
    );
    let lone = with_comments(
        &format!(
            "{}{}{}",
            para("Title"),
            point(0, "Body text."),
            ranged(4, "Sign-off text.")
        ),
        &[comment(0, "Dreadful."), comment(4, "Horrible.")],
    );
    for (mode, settings) in modes() {
        let twin = compare_documents_with_settings(&twins(""), &revised, &settings).unwrap();
        let alone = compare_documents_with_settings(&lone, &revised, &settings).unwrap();
        assert_word_valid_package(&twin);
        let (twin_ids, twin_refs) = comments(&twin);
        let (alone_ids, alone_refs) = comments(&alone);
        assert_eq!(
            twin_ids.len(),
            alone_ids.len(),
            "{mode}: {twin_ids:?} vs {alone_ids:?}"
        );
        assert_eq!(twin_refs.len(), alone_refs.len(), "{mode}");
        assert_eq!(twin_ids, twin_refs, "{mode}");
    }
}

/// The issue's document: an `add` on its first paragraph keeps all six
/// comments, c0 and its reply c1 among them, each defined and referenced.
#[test]
fn the_issues_document_keeps_its_six_comments_after_an_add() {
    let source = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/corpus/neurotic_docx_bench/corpus/word/with_comments_clean/docx/1672057675_485599b4e9_rejected_tracking.docx"
    ))
    .unwrap();
    let edited = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let want: Vec<String> = (0..6).map(|i| i.to_string()).collect();
    for (view, docx) in [("clean", &edited.clean), ("redline", &edited.redline)] {
        let (defined, referenced) = comments(docx);
        assert_eq!(defined, want, "{view}");
        assert_eq!(referenced, want, "{view}");
    }
}
