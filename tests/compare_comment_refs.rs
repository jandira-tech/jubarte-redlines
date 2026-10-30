// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! #1897: a comment's reference keeps its own run properties and its own
//! place. The comparer put every reference back as a bare `rStyle
//! CommentReference` run right after its range's end, in `compare` (both
//! modes) and in the redline of `add` and `edit`: `r5_01`'s reference, at
//! the start of the paragraph after its range and sized 10.5pt, came back
//! at the end of the range's paragraph with no size, so Accept All of an
//! `add` was not its clean copy.

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

/// The size `r5_01` gives its reference run.
const SIZED: &str = r#"<w:rStyle w:val="CommentReference"/><w:sz w:val="21"/><w:szCs w:val="22"/>"#;

fn comment(id: u32, text: &str) -> String {
    format!(
        r#"<w:comment w:id="{id}" w:author="Reviewer" w:date="2026-07-04T23:17:00Z" w:initials="R"><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:comment>"#
    )
}

/// Comment `id`'s reference run, with `rpr` as its properties (none when
/// empty).
fn reference(id: u32, rpr: &str) -> String {
    let rpr = if rpr.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{rpr}</w:rPr>")
    };
    format!(r#"<w:r>{rpr}<w:commentReference w:id="{id}"/></w:r>"#)
}

fn text(t: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{t}</w:t></w:r>"#)
}

/// A paragraph that is comment `id`'s whole range, its reference elsewhere.
fn ranged(id: u32, t: &str) -> String {
    format!(
        r#"<w:p><w:commentRangeStart w:id="{id}"/>{}<w:commentRangeEnd w:id="{id}"/></w:p>"#,
        text(t)
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

/// Where comment `id`'s reference sits in `docx`'s body: the text of its
/// paragraph, the text before it there, and its run's `w:rPr` content.
fn place(docx: &[u8], id: u32) -> (String, String, String) {
    let xml = part_string(docx, "word/document.xml")
        .unwrap()
        .replace(" />", "/>");
    let texts = |s: &str| -> String {
        s.split("<w:t")
            .skip(1)
            .filter(|t| t.starts_with('>') || t.starts_with(' '))
            .filter_map(|t| {
                let t = &t[t.find('>')? + 1..];
                Some(t[..t.find('<')?].to_string())
            })
            .collect()
    };
    let marker = format!(r#"<w:commentReference w:id="{id}"/>"#);
    let at = xml
        .find(&marker)
        .unwrap_or_else(|| panic!("no reference {id}"));
    let p_start = xml[..at]
        .rfind("<w:p>")
        .into_iter()
        .chain(xml[..at].rfind("<w:p "))
        .max()
        .unwrap();
    let p_end = at + xml[at..].find("</w:p>").unwrap();
    let run = xml[..at]
        .rfind("<w:r>")
        .into_iter()
        .chain(xml[..at].rfind("<w:r "))
        .max()
        .unwrap();
    let rpr = xml[run..at]
        .find("<w:rPr>")
        .map(|s| {
            let body = &xml[run + s + 7..at];
            body[..body.find("</w:rPr>").unwrap()].to_string()
        })
        .unwrap_or_default();
    (texts(&xml[p_start..p_end]), texts(&xml[p_start..run]), rpr)
}

/// `r5_01`: the range is the first paragraph, the sized reference opens the
/// second; `tail` follows.
fn ref_opens_next(tail: &str) -> Vec<u8> {
    with_comments(
        &format!(
            "{}<w:p>{}{}</w:p>{tail}",
            ranged(4, "First paragraph, commented."),
            reference(4, SIZED),
            text("Second paragraph.")
        ),
        &[comment(4, "Look.")],
    )
}

/// `r5_06`: the sized reference closes the second paragraph.
fn ref_closes_next(tail: &str) -> Vec<u8> {
    with_comments(
        &format!(
            "{}<w:p>{}{}</w:p>{tail}",
            ranged(4, "First paragraph, commented."),
            text("Second paragraph."),
            reference(4, SIZED)
        ),
        &[comment(4, "Look.")],
    )
}

/// `r5_08`: the range ends between the paragraphs, at body level.
fn end_in_body(tail: &str) -> Vec<u8> {
    with_comments(
        &format!(
            r#"<w:p><w:commentRangeStart w:id="4"/>{}</w:p><w:commentRangeEnd w:id="4"/><w:p>{}{}</w:p>{tail}"#,
            text("First paragraph, commented."),
            text("Second paragraph."),
            reference(4, SIZED)
        ),
        &[comment(4, "Look.")],
    )
}

fn probe() -> String {
    para("Probe paragraph.")
}

fn assert_place(docx: &[u8], view: &str, before: &str) {
    let (paragraph, got_before, rpr) = place(docx, 4);
    assert_eq!(paragraph, "Second paragraph.", "{view}");
    assert_eq!(got_before, before, "{view}");
    assert_eq!(rpr, SIZED, "{view}");
}

/// A source shape: its name, its builder (given a tail), and the text
/// before the reference in its paragraph.
type Shape = (&'static str, fn(&str) -> Vec<u8>, &'static str);

/// Each shape keeps its reference's place and size in the redline, Accept
/// All and Reject All, in both modes.
#[test]
fn a_redline_keeps_a_references_place_and_size() {
    let shapes: [Shape; 3] = [
        ("opens the next paragraph", ref_opens_next, ""),
        (
            "closes the next paragraph",
            ref_closes_next,
            "Second paragraph.",
        ),
        ("range ends in the body", end_in_body, "Second paragraph."),
    ];
    for (shape, make, before) in shapes {
        for (mode, settings) in modes() {
            let redline =
                compare_documents_with_settings(&make(""), &make(&probe()), &settings).unwrap();
            assert_word_valid_package(&redline);
            let view = format!("{shape}, {mode}");
            assert_place(&redline, &format!("{view}, redline"), before);
            assert_place(
                &accept_revisions(&redline).unwrap(),
                &format!("{view}, accept"),
                before,
            );
            assert_place(
                &reject_revisions(&redline).unwrap(),
                &format!("{view}, reject"),
                before,
            );
        }
    }
}

/// An `add` of a first paragraph: (clean copy, redline).
fn add_probe(source: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let edited = apply_plan(
        source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Probe","operations":[
            {"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":"Probe paragraph."}]}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    (edited.clean, edited.redline)
}

/// An `add`'s Accept All gives back the clean copy's reference: its place
/// and its run properties, a bare run included.
#[test]
fn an_adds_accept_all_keeps_the_clean_copys_references() {
    let bare = with_comments(
        &format!(
            "{}<w:p>{}{}</w:p>",
            ranged(4, "First paragraph, commented."),
            reference(4, ""),
            text("Second paragraph.")
        ),
        &[comment(4, "Look.")],
    );
    for (shape, source) in [
        ("opens", ref_opens_next("")),
        ("closes", ref_closes_next("")),
        ("body end", end_in_body("")),
        ("bare", bare),
    ] {
        let (clean, redline) = add_probe(&source);
        assert_word_valid_package(&redline);
        let accepted = accept_revisions(&redline).unwrap();
        assert_eq!(place(&accepted, 4), place(&clean, 4), "{shape}");
    }
}

/// A point comment (a reference with no range) keeps its place and size.
#[test]
fn a_point_comment_keeps_its_place_and_size() {
    let source = |tail: &str| {
        with_comments(
            &format!(
                "{}<w:p>{}{}{}</w:p>{tail}",
                para("First paragraph."),
                text("Second "),
                reference(4, SIZED),
                text("paragraph.")
            ),
            &[comment(4, "Look.")],
        )
    };
    for (mode, settings) in modes() {
        let redline =
            compare_documents_with_settings(&source(""), &source(&probe()), &settings).unwrap();
        assert_word_valid_package(&redline);
        assert_place(&redline, mode, "Second ");
    }
}

/// A comment the revision removed keeps the original's reference run.
#[test]
fn a_removed_comments_reference_keeps_the_originals_properties() {
    let kept = format!(
        r#"<w:p><w:commentRangeStart w:id="1"/>{}<w:commentRangeEnd w:id="1"/>{}</w:p>"#,
        text("Kept."),
        reference(1, "")
    );
    let original = with_comments(
        &format!(
            r#"{kept}<w:p><w:commentRangeStart w:id="4"/>{}<w:commentRangeEnd w:id="4"/>{}</w:p>"#,
            text("Second paragraph."),
            reference(4, SIZED)
        ),
        &[comment(1, "Stays."), comment(4, "Goes.")],
    );
    let revised = with_comments(
        &format!("{kept}{}", para("Second paragraph.")),
        &[comment(1, "Stays.")],
    );
    for (mode, settings) in modes() {
        let redline = compare_documents_with_settings(&original, &revised, &settings).unwrap();
        assert_word_valid_package(&redline);
        assert_place(&redline, mode, "Second paragraph.");
    }
}
