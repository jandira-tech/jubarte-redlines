// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A comment's paragraph style survives the round trip docx → view → docx:
//! the `comments:` line names it only when it is not Word's
//! `annotation text` (`style Normal` for every comment, `c6 style Normal`
//! for one), and the writer stamps CommentText only where the line is
//! silent.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};
use std::io::Read as _;

const COMMENTS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// A styles part: Normal, Word's CommentText, and a custom `My Note`.
fn styles_xml() -> String {
    let w = common::docx::W_NS;
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{w}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="CommentText"><w:name w:val="annotation text"/><w:basedOn w:val="Normal"/><w:rPr><w:sz w:val="20"/></w:rPr></w:style><w:style w:type="paragraph" w:customStyle="1" w:styleId="My Note"><w:name w:val="My Note"/><w:basedOn w:val="Normal"/><w:rPr><w:i/></w:rPr></w:style></w:styles>"#
    )
}

/// A docx whose body paragraph `i` carries comment `i`, one per entry of
/// `styles` (`None`: its paragraph has no `w:pStyle`).
fn docx(styles: &[Option<&str>]) -> Vec<u8> {
    let (w, r) = (common::docx::W_NS, common::docx::R_NS);
    let mut body = String::new();
    let mut comments = String::new();
    for (i, style) in styles.iter().enumerate() {
        body.push_str(&format!(
            r#"<w:p><w:commentRangeStart w:id="{i}"/><w:r><w:t>Clause {i}.</w:t></w:r><w:commentRangeEnd w:id="{i}"/><w:r><w:commentReference w:id="{i}"/></w:r></w:p>"#
        ));
        let ppr = style
            .map(|s| format!(r#"<w:pPr><w:pStyle w:val="{s}"/></w:pPr>"#))
            .unwrap_or_default();
        comments.push_str(&format!(
            r#"<w:comment w:id="{i}" w:author="Ann" w:date="2026-01-01T00:00:00Z" w:initials="A"><w:p>{ppr}<w:r><w:annotationRef/></w:r><w:r><w:t>Note {i}</w:t></w:r></w:p></w:comment>"#
        ));
    }
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="{w}" xmlns:r="{r}">{comments}</w:comments>"#
    );
    let styles = styles_xml();
    let comments_rel = format!("{RELS}/comments");
    let styles_rel = format!("{RELS}/styles");
    let parts = [
        common::docx::Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: &comments_rel,
            xml: &comments,
        },
        common::docx::Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: &styles_rel,
            xml: &styles,
        },
    ];
    common::docx::docx_with_sect(&body, &parts, "")
}

/// The view of `docx`, the docx it writes, and that docx's view; both
/// views without their `sha256:` line, which names the bytes.
fn round_trip(docx: &[u8]) -> (String, Vec<u8>, String) {
    let options = ReadOptions {
        page_markers: false,
        ..ReadOptions::default()
    };
    let without_sha = |view: String| {
        view.lines()
            .filter(|l| !l.starts_with("sha256: "))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let view = read(docx, &options).expect("the docx reads").markdown;
    let again = markdown_to_docx(&view, &DocxOptions::default())
        .expect("the view reads")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let view_again = read(&again, &options).expect("the docx reads").markdown;
    (without_sha(view), again, without_sha(view_again))
}

fn part(docx: &[u8], name: &str) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name(name).unwrap().read_to_string(&mut xml).unwrap();
    xml
}

/// Each comment's first paragraph's `w:pStyle`, `-` for none.
fn comment_styles(docx: &[u8]) -> Vec<String> {
    part(docx, "word/comments.xml")
        .split("<w:comment ")
        .skip(1)
        .map(|c| {
            c.split("<w:pStyle w:val=\"")
                .nth(1)
                .and_then(|s| s.split('"').next())
                .filter(|_| {
                    c.find("<w:pStyle")
                        .is_some_and(|at| at < c.find("</w:p>").unwrap())
                })
                .unwrap_or("-")
                .to_string()
        })
        .collect()
}

/// The view's `comments:` line.
fn comments_line(view: &str) -> &str {
    view.lines()
        .find(|l| l.starts_with("comments:"))
        .unwrap_or_else(|| panic!("no comments line: {view}"))
}

#[test]
fn comments_in_word_style_print_no_clause() {
    let docx = docx(&[Some("CommentText"), Some("CommentText")]);
    let (view, again, view_again) = round_trip(&docx);
    assert!(!comments_line(&view).contains("style"), "{view}");
    assert_eq!(comment_styles(&again), ["CommentText", "CommentText"]);
    assert_eq!(view_again, view);
}

#[test]
fn comments_without_a_style_keep_none() {
    // ac6cd8d92f, file_170, docx_lots_of_comments…: comment paragraphs
    // with no style came back in `annotation text`.
    let docx = docx(&[None, None, None]);
    let (view, again, view_again) = round_trip(&docx);
    let line = comments_line(&view);
    assert!(line.contains(", style Normal"), "{line}");
    assert_eq!(comment_styles(&again), ["-", "-", "-"]);
    assert_eq!(view_again, view);
}

#[test]
fn one_comment_in_another_style_is_an_exception() {
    // received.docx: one comment without a style among CommentText ones.
    let docx = docx(&[None, Some("CommentText"), Some("CommentText")]);
    let (view, again, view_again) = round_trip(&docx);
    let line = comments_line(&view);
    assert!(line.contains(", c0 style Normal"), "{line}");
    assert!(!line.contains(", style "), "{line}");
    assert_eq!(comment_styles(&again), ["-", "CommentText", "CommentText"]);
    assert_eq!(view_again, view);
}

#[test]
fn a_comment_style_id_with_a_space_is_quoted_and_defined() {
    let docx = docx(&[Some("My Note"), Some("My Note")]);
    let (view, again, view_again) = round_trip(&docx);
    let line = comments_line(&view);
    assert!(line.contains(r#", style "My Note""#), "{line}");
    assert_eq!(comment_styles(&again), ["My Note", "My Note"]);
    let styles = part(&again, "word/styles.xml");
    let definition = styles
        .split("<w:style ")
        .find(|s| s.contains("w:styleId=\"My Note\""))
        .unwrap_or_else(|| panic!("no My Note: {styles}"));
    assert!(definition.contains("<w:i/>"), "{view}\n{definition}");
    assert_eq!(view_again, view);
}

#[test]
fn normal_is_described_once_when_no_style_is_marked_default() {
    // ids-comments (all.docx): no `w:default` style, unstyled comments.
    let docx = docx(&[None, None]);
    let styles = common::docx::part_string(&docx, "word/styles.xml")
        .unwrap()
        .replace(r#" w:default="1""#, "");
    let docx = common::docx::replace_entry(&docx, "word/styles.xml", styles.as_bytes());
    let (view, again, view_again) = round_trip(&docx);
    assert_eq!(view.matches("\n  Normal:").count(), 1, "{view}");
    assert_eq!(comment_styles(&again), ["-", "-"]);
    assert_eq!(view_again, view);
}
