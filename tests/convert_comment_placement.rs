// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Where a converted PDF puts its comments (`PdfOptions::comments`) and
//! which pages it keeps (`PdfOptions::changed_only`).

mod common;

use std::process::Command;

use common::docx::{Part, docx_with, docx_with_sect, docx_with_sect_pr};
use jubarte::convert::{
    CommentPlacement, ConvertError, PdfOptions, RenderRequest, Rendered, RevisionPalette,
    RevisionStyle, docx_to_pdf_with, pdf_page_count, render,
};

const COMMENTS_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml";
const COMMENTS_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments";

const PAGE_BREAK: &str = r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#;

/// Four pages: Alpha (plain), Bravo (an insertion and comment 1), Charlie
/// (plain text under comment 2), Delta (a deletion).
fn four_pages(tracked: bool) -> Vec<u8> {
    let insertion = if tracked {
        r#"<w:ins w:id="10" w:author="Rev" w:date="2026-01-01T00:00:00Z"><w:r><w:t xml:space="preserve"> inserted words</w:t></w:r></w:ins>"#
    } else {
        r#"<w:r><w:t xml:space="preserve"> inserted words</w:t></w:r>"#
    };
    let deletion = if tracked {
        r#"<w:del w:id="11" w:author="Rev" w:date="2026-01-01T00:00:00Z"><w:r><w:delText xml:space="preserve"> gone words</w:delText></w:r></w:del>"#
    } else {
        ""
    };
    let body = [
        r#"<w:p><w:r><w:t>Alpha page text</w:t></w:r></w:p>"#.to_string(),
        PAGE_BREAK.to_string(),
        format!(
            r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>Bravo page text</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r>{insertion}</w:p>"#
        ),
        PAGE_BREAK.to_string(),
        r#"<w:p><w:commentRangeStart w:id="2"/><w:r><w:t>Charlie page text</w:t></w:r><w:commentRangeEnd w:id="2"/><w:r><w:commentReference w:id="2"/></w:r></w:p>"#.to_string(),
        PAGE_BREAK.to_string(),
        format!(r#"<w:p><w:r><w:t>Delta page text</w:t></w:r>{deletion}</w:p>"#),
    ]
    .concat();
    let comments = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="1" w:author="Jane Roe" w:initials="JR" w:date="2026-01-01T00:00:00Z"><w:p><w:r><w:t>First note here</w:t></w:r></w:p></w:comment><w:comment w:id="2" w:author="Jane Roe" w:initials="JR" w:date="2026-01-01T00:00:00Z"><w:p><w:r><w:t>Second note here</w:t></w:r></w:p></w:comment></w:comments>"#;
    docx_with(
        &body,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: comments,
        }],
    )
}

fn rendered(docx: &[u8], options: PdfOptions) -> Rendered {
    render(
        docx,
        options,
        RenderRequest {
            pdf: true,
            ..RenderRequest::default()
        },
    )
    .expect("the document renders")
}

fn texts(r: &Rendered) -> Vec<String> {
    r.report.pages.iter().map(|p| p.text.clone()).collect()
}

fn pdf_text(r: &Rendered) -> String {
    String::from_utf8_lossy(r.pdf.as_deref().expect("a PDF")).into_owned()
}

#[test]
fn margin_placement_is_the_default_and_paints_balloons_beside_the_text() {
    assert_eq!(PdfOptions::default().comments, CommentPlacement::Margin);
    let r = rendered(&four_pages(true), PdfOptions::default());
    let pages = texts(&r);
    assert_eq!(pages.len(), 4, "{pages:?}");
    assert!(pages[1].contains("Commented [JR1]"), "{:?}", pages[1]);
    assert!(pages[2].contains("Commented [JR2]"), "{:?}", pages[2]);
    let pdf = pdf_text(&r);
    assert!(
        pdf.contains("0.949 0.949 0.949 rg"),
        "the markup pane is painted"
    );
}

#[test]
fn end_placement_marks_the_anchor_and_lists_the_comments_on_a_last_page() {
    let options = PdfOptions {
        comments: CommentPlacement::End,
        ..PdfOptions::default()
    };
    let r = rendered(&four_pages(true), options);
    let pages = texts(&r);
    assert_eq!(
        pages.len(),
        5,
        "the four pages and one comments page: {pages:?}"
    );
    for page in &pages[..4] {
        assert!(
            !page.contains("Commented ["),
            "no balloon on a body page: {page:?}"
        );
        assert!(
            !page.contains("note here"),
            "no comment text on a body page: {page:?}"
        );
    }
    assert!(
        pages[1].contains("[JR1]"),
        "the anchor keeps its marker: {:?}",
        pages[1]
    );
    assert!(
        pages[2].contains("[JR2]"),
        "the anchor keeps its marker: {:?}",
        pages[2]
    );
    let last = &pages[4];
    assert!(last.contains("Comments"), "{last:?}");
    let first = last.find("First note here").expect("comment 1 listed");
    let second = last.find("Second note here").expect("comment 2 listed");
    assert!(first < second, "listed in document order: {last:?}");
    assert!(
        last.contains("[JR1]") && last.contains("Jane Roe"),
        "{last:?}"
    );
    assert!(
        last.contains("page 2") && last.contains("page 3"),
        "{last:?}"
    );
    let pdf = pdf_text(&r);
    assert!(
        !pdf.contains("0.949 0.949 0.949 rg"),
        "no markup pane: the page stays clean"
    );
    assert!(!pdf.contains("/Subtype /Text"), "no sticky notes either");
}

#[test]
fn end_placement_keeps_the_commented_range_highlighted() {
    let tint_fills = |r: &Rendered| {
        pdf_text(r)
            .lines()
            .filter(|l| l.contains(" rg ") && l.ends_with(" re f") && !l.starts_with("0.949"))
            .count()
    };
    let margin = rendered(&four_pages(true), PdfOptions::default());
    let end = rendered(
        &four_pages(true),
        PdfOptions {
            comments: CommentPlacement::End,
            ..PdfOptions::default()
        },
    );
    assert!(tint_fills(&end) > 0, "the commented text keeps its tint");
    assert!(tint_fills(&margin) > 0);
}

#[test]
fn changed_only_keeps_the_pages_with_tracked_changes() {
    let options = PdfOptions {
        changed_only: true,
        ..PdfOptions::default()
    };
    let r = rendered(&four_pages(true), options);
    let pages = texts(&r);
    assert_eq!(pages.len(), 2, "{pages:?}");
    assert!(pages[0].contains("Bravo page text"), "{pages:?}");
    assert!(pages[1].contains("Delta page text"), "{pages:?}");
    assert_eq!(r.report.page_count, 2);
}

#[test]
fn changed_only_with_end_comments_lists_the_kept_pages_comments() {
    let options = PdfOptions {
        changed_only: true,
        comments: CommentPlacement::End,
        ..PdfOptions::default()
    };
    let pages = texts(&rendered(&four_pages(true), options));
    assert_eq!(
        pages.len(),
        3,
        "Bravo, Delta and the comments page: {pages:?}"
    );
    let last = &pages[2];
    assert!(last.contains("First note here"), "{last:?}");
    assert!(
        !last.contains("Second note here"),
        "Charlie's page was dropped: {last:?}"
    );
    assert!(
        last.contains("page 2"),
        "the page number is the document's: {last:?}"
    );
}

#[test]
fn changed_only_without_changes_keeps_the_first_page() {
    let options = PdfOptions {
        changed_only: true,
        ..PdfOptions::default()
    };
    let pages = texts(&rendered(&four_pages(false), options));
    assert_eq!(pages.len(), 1, "{pages:?}");
    assert!(pages[0].contains("Alpha page text"), "{pages:?}");
}

#[test]
fn cli_convert_takes_move_comments_and_changed_only() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let input = dir.join("four.docx");
    std::fs::write(&input, four_pages(true)).unwrap();
    let output = dir.join("four.pdf");
    let status = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args([
            "convert",
            "--force",
            "--move-comments",
            "--changed-only",
            "-o",
        ])
        .arg(&output)
        .arg(&input)
        .status()
        .unwrap();
    assert!(status.success());
    let pdf = std::fs::read(&output).unwrap();
    assert_eq!(jubarte::convert::pdf_page_count(&pdf), 3);
}

#[test]
fn end_placement_folds_a_reply_into_its_threads_marker() {
    let docx = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/corpus/fresh_docx_fixtures_and_redlines/docx_lots_of_comments_addition_redline.docx"
    ))
    .unwrap();
    let options = PdfOptions {
        comments: CommentPlacement::End,
        ..PdfOptions::default()
    };
    let pages = texts(&rendered(&docx, options));
    let (body, listing) = pages.split_at(pages.len() - 1);
    let body = body.concat();
    assert!(body.contains("[AS2]"), "the thread's marker is in the text");
    assert!(!body.contains("R2]"), "no reply marker in the text");
    let listing = &listing[0];
    let parent = listing.find("[AS2] ").expect("the thread is listed");
    let reply = listing.find("[AS2R2] ").expect("its reply is listed");
    assert!(parent < reply, "the reply follows its thread: {listing:?}");
}

/// `n` paragraphs, each under a comment `words` words long.
fn many_comments(n: usize, words: usize) -> Vec<u8> {
    let body: String = (1..=n)
        .map(|i| {
            format!(
                r#"<w:p><w:commentRangeStart w:id="{i}"/><w:r><w:t>Paragraph {i}</w:t></w:r><w:commentRangeEnd w:id="{i}"/><w:r><w:commentReference w:id="{i}"/></w:r></w:p>"#
            )
        })
        .collect();
    let text = vec!["wordy"; words].join(" ");
    let comments: String = (1..=n)
        .map(|i| {
            format!(
                r#"<w:comment w:id="{i}" w:author="Ann Lee" w:initials="AL"><w:p><w:r><w:t>Note {i} {text}</w:t></w:r></w:p></w:comment>"#
            )
        })
        .collect();
    let comments = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{comments}</w:comments>"#
    );
    docx_with(
        &body,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &comments,
        }],
    )
}

#[test]
fn a_long_comment_list_wraps_and_runs_onto_more_pages() {
    let options = PdfOptions {
        comments: CommentPlacement::End,
        ..PdfOptions::default()
    };
    let pages = texts(&rendered(&many_comments(40, 60), options));
    let start = pages
        .iter()
        .position(|p| p.starts_with("Comments"))
        .expect("a comments page");
    let listing = &pages[start..];
    assert!(
        listing.len() >= 2,
        "forty long comments take more than one page: {}",
        pages.len()
    );
    let all = listing.concat();
    for i in [1, 20, 40] {
        assert!(
            all.contains(&format!("[AL{i}] Ann Lee, page ")),
            "comment {i} listed"
        );
    }
    let note_lines = listing[0]
        .lines()
        .filter(|l| l.starts_with("wordy") || l.starts_with("Note 1 "))
        .count();
    assert!(
        note_lines > 2,
        "a sixty-word comment wraps: {:?}",
        listing[0]
    );
    for page in listing {
        assert!(
            page.lines().count() < 70,
            "each listing page stays inside its margins"
        );
    }
}

#[test]
fn end_placement_without_comments_adds_no_page() {
    let options = PdfOptions {
        comments: CommentPlacement::End,
        ..PdfOptions::default()
    };
    let docx = docx_with(r#"<w:p><w:r><w:t>Plain</w:t></w:r></w:p>"#, &[]);
    assert_eq!(texts(&rendered(&docx, options)).len(), 1);
}

#[test]
fn the_page_flags_are_refused_where_nothing_is_laid_out() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old.docx");
    let new = dir.path().join("new.docx");
    let old_bytes = four_pages(false);
    let new_bytes = four_pages(true);
    std::fs::write(&old, &old_bytes).unwrap();
    std::fs::write(&new, &new_bytes).unwrap();
    for flag in ["--move-comments", "--changed-only"] {
        for command in ["convert", "diff"] {
            for format in ["docx", "md"] {
                for explicit_format in [false, true] {
                    let output = dir.path().join(format!("output.{format}"));
                    let sentinel = b"existing output must survive refusal";
                    std::fs::write(&output, sentinel).unwrap();
                    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jubarte"));
                    cmd.args([command, "--force", flag, "-o"]).arg(&output);
                    if explicit_format {
                        cmd.args(["-t", format]);
                    }
                    if command == "diff" {
                        cmd.arg(&old);
                    }
                    let result = cmd.arg(&new).output().unwrap();
                    assert!(!result.status.success(), "{command} {flag} {format}");
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    assert!(
                        stderr.contains(&format!("{flag} applies to PDF or PNG output only")),
                        "{stderr}"
                    );
                    assert_eq!(std::fs::read(&output).unwrap(), sentinel);
                }
            }
        }
    }
    assert_eq!(std::fs::read(old).unwrap(), old_bytes);
    assert_eq!(std::fs::read(new).unwrap(), new_bytes);
}

const COMMENT_XML: &str = r#"<w:comment w:id="1" w:author="Jane Roe" w:initials="JR"><w:p><w:r><w:t>Justified note</w:t></w:r></w:p></w:comment>"#;

fn comments_part(inner: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{inner}</w:comments>"#
    )
}

fn with_comments(body: &str, inner: &str) -> Vec<u8> {
    let xml = comments_part(inner);
    docx_with(
        body,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &xml,
        }],
    )
}

#[test]
fn a_comment_inside_a_justified_line_keeps_its_balloon_and_its_listing() {
    let words = "stretch ".repeat(40);
    let body = format!(
        r#"<w:p><w:pPr><w:jc w:val="both"/></w:pPr><w:r><w:t xml:space="preserve">Lead words </w:t></w:r><w:commentRangeStart w:id="1"/><w:r><w:t xml:space="preserve">commented span </w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r><w:r><w:t xml:space="preserve">{words}</w:t></w:r></w:p>"#
    );
    let docx = with_comments(&body, COMMENT_XML);
    let margin = texts(&rendered(&docx, PdfOptions::default())).concat();
    assert!(margin.contains("Commented [JR1]"), "{margin:?}");
    let end = texts(&rendered(
        &docx,
        PdfOptions {
            comments: CommentPlacement::End,
            ..PdfOptions::default()
        },
    ));
    assert!(end.last().unwrap().contains("Justified note"), "{end:?}");
}

#[test]
fn a_marker_after_an_unpainted_effect_run_still_carries_its_comment() {
    let body = r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t xml:space="preserve">Plain </w:t></w:r><w:r><w:rPr><w14:reflection xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"/></w:rPr><w:t>Echo</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p>"#;
    let docx = with_comments(body, COMMENT_XML);
    let pages = texts(&rendered(
        &docx,
        PdfOptions {
            comments: CommentPlacement::End,
            ..PdfOptions::default()
        },
    ));
    assert_eq!(pages.len(), 2, "{pages:?}");
    assert!(pages[0].contains("[JR1]"), "{pages:?}");
    assert!(pages[1].contains("Justified note"), "{pages:?}");
}

#[test]
fn an_insertion_running_over_a_page_break_keeps_both_pages() {
    let filler = "Filler line words. ".repeat(12);
    let mut body = String::new();
    for _ in 0..30 {
        body.push_str(&format!(r#"<w:p><w:r><w:t>{filler}</w:t></w:r></w:p>"#));
    }
    let inserted = "Inserted words run long. ".repeat(200);
    body.push_str(&format!(
        r#"<w:p><w:ins w:id="5" w:author="Rev" w:date="2026-01-01T00:00:00Z"><w:r><w:t>{inserted}</w:t></w:r></w:ins></w:p>"#
    ));
    body.push_str(PAGE_BREAK);
    body.push_str(r#"<w:p><w:r><w:t>Untouched tail page</w:t></w:r></w:p>"#);
    let docx = docx_with(&body, &[]);
    let all = texts(&rendered(&docx, PdfOptions::default()));
    let changed = texts(&rendered(
        &docx,
        PdfOptions {
            changed_only: true,
            ..PdfOptions::default()
        },
    ));
    let spanned: Vec<&String> = all
        .iter()
        .filter(|p| p.contains("Inserted words"))
        .collect();
    assert!(
        spanned.len() >= 2,
        "the insertion spans pages: {}",
        all.len()
    );
    assert_eq!(
        changed.len(),
        spanned.len(),
        "every page it spans, and no other"
    );
    assert!(changed.iter().all(|p| p.contains("Inserted words")));
}

#[test]
fn an_overlong_word_in_a_comment_is_broken_to_the_line() {
    let word = "x".repeat(400);
    let body = r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>Anchor</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p>"#;
    let inner = format!(
        r#"<w:comment w:id="1" w:author="Jane Roe" w:initials="JR"><w:p><w:r><w:t>{word}</w:t></w:r></w:p></w:comment>"#
    );
    let pages = texts(&rendered(
        &with_comments(body, &inner),
        PdfOptions {
            comments: CommentPlacement::End,
            ..PdfOptions::default()
        },
    ));
    let listing = pages.last().unwrap();
    let pieces: Vec<&str> = listing.lines().filter(|l| l.starts_with('x')).collect();
    assert!(pieces.len() >= 3, "{listing:?}");
    assert_eq!(pieces.concat(), word, "nothing lost in the breaks");
}

#[test]
fn comments_are_listed_in_document_order_across_columns() {
    let filler: String = (0..12)
        .map(|_| r#"<w:p><w:r><w:t>Column one filler line</w:t></w:r></w:p>"#)
        .collect();
    let body = format!(
        r#"{filler}<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>Low in column one</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p><w:p><w:r><w:br w:type="column"/></w:r></w:p><w:p><w:commentRangeStart w:id="2"/><w:r><w:t>Top of column two</w:t></w:r><w:commentRangeEnd w:id="2"/><w:r><w:commentReference w:id="2"/></w:r></w:p>"#
    );
    let inner = r#"<w:comment w:id="1" w:author="Jane Roe" w:initials="JR"><w:p><w:r><w:t>Column one note</w:t></w:r></w:p></w:comment><w:comment w:id="2" w:author="Jane Roe" w:initials="JR"><w:p><w:r><w:t>Column two note</w:t></w:r></w:p></w:comment>"#;
    let xml = comments_part(inner);
    let docx = docx_with_sect_pr(
        &body,
        &[Part {
            name: "word/comments.xml",
            content_type: COMMENTS_CT,
            rel_type: COMMENTS_REL,
            xml: &xml,
        }],
        r#"<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/><w:cols w:num="2" w:space="720"/></w:sectPr>"#,
    );
    let pages = texts(&rendered(
        &docx,
        PdfOptions {
            comments: CommentPlacement::End,
            ..PdfOptions::default()
        },
    ));
    assert_eq!(pages.len(), 2, "{pages:?}");
    let listing = &pages[1];
    let one = listing.find("Column one note").unwrap();
    let two = listing.find("Column two note").unwrap();
    assert!(one < two, "{listing:?}");
}

#[test]
fn changed_only_ignores_revisions_in_headers_and_footers() {
    for (kind, tag) in [("header", "hdr"), ("footer", "ftr")] {
        for body_changed in [false, true] {
            let chrome = format!(
                r#"<w:{tag} xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:ins w:id="1" w:author="Rev"><w:r><w:t>Repeated revision</w:t></w:r></w:ins></w:p></w:{tag}>"#
            );
            let second = if body_changed {
                r#"<w:ins w:id="2" w:author="Rev"><w:r><w:t>Second page</w:t></w:r></w:ins>"#
            } else {
                r#"<w:r><w:t>Second page</w:t></w:r>"#
            };
            let body = format!(
                r#"<w:p><w:r><w:t>First page</w:t></w:r></w:p>{PAGE_BREAK}<w:p>{second}</w:p>"#
            );
            let docx = docx_with_sect(
                &body,
                &[Part {
                    name: &format!("word/{kind}.xml"),
                    content_type: &format!(
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml"
                    ),
                    rel_type: &format!(
                        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}"
                    ),
                    xml: &chrome,
                }],
                &format!(r#"<w:{kind}Reference w:type="default" r:id="rIdX0"/>"#),
            );
            let all = texts(&rendered(&docx, PdfOptions::default()));
            assert_eq!(all.len(), 2);
            assert!(all.iter().all(|p| p.contains("Repeated revision")));
            let kept = texts(&rendered(
                &docx,
                PdfOptions {
                    changed_only: true,
                    ..PdfOptions::default()
                },
            ));
            let expected = if body_changed { &all[1] } else { &all[0] };
            assert_eq!(
                kept,
                vec![expected.clone()],
                "{kind}, body_changed={body_changed}"
            );
        }
    }
}

#[test]
fn changed_only_preserves_page_and_numpages_fields() {
    let footer = r#"<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t xml:space="preserve">Page </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>99</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t xml:space="preserve"> of </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>NUMPAGES</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>99</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p></w:ftr>"#;
    let body = format!(
        r#"<w:p><w:r><w:t>First</w:t></w:r></w:p>{PAGE_BREAK}<w:p><w:commentRangeStart w:id="1"/><w:ins w:id="3" w:author="Rev"><w:r><w:t>Middle</w:t></w:r></w:ins><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p>{PAGE_BREAK}<w:p><w:r><w:t>Last</w:t></w:r></w:p>"#
    );
    let comments = comments_part(COMMENT_XML);
    let docx = docx_with_sect(
        &body,
        &[
            Part {
                name: "word/footer.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer",
                xml: footer,
            },
            Part {
                name: "word/comments.xml",
                content_type: COMMENTS_CT,
                rel_type: COMMENTS_REL,
                xml: &comments,
            },
        ],
        r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
    );
    for comments in [CommentPlacement::Margin, CommentPlacement::End] {
        let r = rendered(
            &docx,
            PdfOptions {
                comments,
                changed_only: true,
                ..PdfOptions::default()
            },
        );
        let pages = texts(&r);
        let expected_count = if comments == CommentPlacement::End {
            2
        } else {
            1
        };
        assert_eq!(pages.len(), expected_count, "{pages:?}");
        assert!(pages[0].contains("Page 2 of 3"), "{pages:?}");
        assert!(
            !pages[0].contains("99"),
            "cached field results must be replaced"
        );
        if comments == CommentPlacement::End {
            assert!(pages[1].contains("Jane Roe, page 2"), "{pages:?}");
            assert!(
                !pages[1].contains("Page 2 of 3"),
                "no document footer on the listing"
            );
        }
    }
}

#[test]
fn changed_only_keeps_revised_table_cells() {
    let body = format!(
        r#"<w:p><w:r><w:t>Untouched first</w:t></w:r></w:p>{PAGE_BREAK}<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="8640"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="8640" w:type="dxa"/></w:tcPr><w:p><w:ins w:id="1" w:author="Rev"><w:r><w:t>Revised cell</w:t></w:r></w:ins></w:p></w:tc></w:tr></w:tbl>{PAGE_BREAK}<w:p><w:r><w:t>Untouched last</w:t></w:r></w:p>"#
    );
    let pages = texts(&rendered(
        &docx_with(&body, &[]),
        PdfOptions {
            changed_only: true,
            ..PdfOptions::default()
        },
    ));
    assert_eq!(pages.len(), 1, "{pages:?}");
    assert!(pages[0].contains("Revised cell"), "{pages:?}");
    assert!(!pages[0].contains("Untouched"), "{pages:?}");
}

#[test]
fn unchanged_fallback_does_not_list_comments_from_discarded_pages() {
    let pages = texts(&rendered(
        &four_pages(false),
        PdfOptions {
            comments: CommentPlacement::End,
            changed_only: true,
            ..PdfOptions::default()
        },
    ));
    assert_eq!(
        pages.len(),
        1,
        "no empty listing after the fallback page: {pages:?}"
    );
    assert!(pages[0].contains("Alpha page text"));
    assert!(!pages[0].contains("[JR"));
}

#[test]
fn unchanged_fallback_lists_comments_anchored_on_the_first_page() {
    let body = format!(
        r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>First anchor</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p>{PAGE_BREAK}<w:p><w:r><w:t>Unchanged tail</w:t></w:r></w:p>"#
    );
    let pages = texts(&rendered(
        &with_comments(&body, COMMENT_XML),
        PdfOptions {
            comments: CommentPlacement::End,
            changed_only: true,
            ..PdfOptions::default()
        },
    ));
    assert_eq!(pages.len(), 2, "{pages:?}");
    assert!(pages[0].contains("[JR1]"));
    assert!(pages[1].contains("Jane Roe, page 1"));
    assert!(pages[1].contains("Justified note"));
    assert!(!pages.concat().contains("Unchanged tail"));
}

#[test]
fn png_selection_indexes_the_filtered_pages_and_comment_listing() {
    let docx = four_pages(true);
    let options = PdfOptions {
        comments: CommentPlacement::End,
        changed_only: true,
        ..PdfOptions::default()
    };
    let all = render(
        &docx,
        options,
        RenderRequest {
            pdf: true,
            png_dpi: Some(12.0),
            pages: None,
        },
    )
    .unwrap();
    assert_eq!(all.pngs.len(), 3);
    assert_eq!(pdf_page_count(all.pdf.as_ref().unwrap()), 3);
    assert_eq!(
        all.report.pages.iter().map(|p| p.index).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert!(all.report.pages[0].text.contains("Bravo page text"));
    assert!(all.report.pages[1].text.contains("Delta page text"));
    assert!(all.report.pages[2].text.starts_with("Comments"));
    let selected = render(
        &docx,
        options,
        RenderRequest {
            pdf: true,
            png_dpi: Some(12.0),
            pages: Some(vec![2, 0, 2]),
        },
    )
    .unwrap();
    assert_eq!(
        selected.pngs,
        vec![all.pngs[0].clone(), all.pngs[2].clone()]
    );
    assert_eq!(
        selected.report, all.report,
        "PNG selection must not narrow the report"
    );
    assert_eq!(
        selected.pdf, all.pdf,
        "PNG selection must not narrow the PDF"
    );
}

#[test]
fn png_page_bounds_use_the_output_count_after_filtering_and_listing() {
    for (comments, count) in [(CommentPlacement::Margin, 2), (CommentPlacement::End, 3)] {
        let err = render(
            &four_pages(true),
            PdfOptions {
                comments,
                changed_only: true,
                ..PdfOptions::default()
            },
            RenderRequest {
                pdf: false,
                png_dpi: Some(12.0),
                pages: Some(vec![count]),
            },
        )
        .unwrap_err();
        assert!(
            matches!(err, ConvertError::PageOutOfRange { requested, page_count }
            if requested == count && page_count == count),
            "{err:?}"
        );
        assert_eq!(
            err.to_string(),
            format!(
                "page {} is out of range: the output has {count} pages",
                count + 1
            )
        );
    }
}

#[test]
fn pdf_entry_points_apply_both_options_in_every_revision_style() {
    for revisions in [
        RevisionStyle::Conventional,
        RevisionStyle::Word,
        RevisionStyle::Custom(RevisionPalette::parse("inserted=#123456:plain").unwrap()),
    ] {
        let options = PdfOptions {
            revisions,
            comments: CommentPlacement::End,
            changed_only: true,
            ..PdfOptions::default()
        };
        let docx = four_pages(true);
        let r = rendered(&docx, options);
        let pages = texts(&r);
        assert_eq!(pages.len(), 3, "{revisions:?}: {pages:?}");
        assert!(pages[0].contains("[JR1]"), "{revisions:?}: {pages:?}");
        assert!(pages[2].contains("First note here"));
        assert!(!pages.concat().contains("Second note here"));
        assert_eq!(docx_to_pdf_with(&docx, options).unwrap(), r.pdf.unwrap());
    }
}

#[test]
fn unanchored_comments_do_not_create_a_listing() {
    let docx = with_comments(r#"<w:p><w:r><w:t>No anchor</w:t></w:r></w:p>"#, COMMENT_XML);
    let pages = texts(&rendered(
        &docx,
        PdfOptions {
            comments: CommentPlacement::End,
            ..PdfOptions::default()
        },
    ));
    assert_eq!(pages.len(), 1, "{pages:?}");
    assert!(!pages[0].contains("[JR1]"));
    assert!(!pages[0].contains("Justified note"));
}

#[test]
fn comment_listing_wraps_unicode_without_losing_characters() {
    let word = "éΩЖ".repeat(100);
    let body = r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>Anchor</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p>"#;
    let inner = format!(
        r#"<w:comment w:id="1" w:author="Jane Roe" w:initials="JR"><w:p><w:r><w:t>{word}</w:t></w:r></w:p></w:comment>"#
    );
    let pages = texts(&rendered(
        &with_comments(body, &inner),
        PdfOptions {
            comments: CommentPlacement::End,
            ..PdfOptions::default()
        },
    ));
    assert_eq!(pages.len(), 2);
    let pieces: Vec<&str> = pages[1]
        .lines()
        .filter(|line| line.chars().next().is_some_and(|ch| "éΩЖ".contains(ch)))
        .collect();
    assert!(pieces.len() > 1, "{pages:?}");
    assert_eq!(pieces.concat(), word);
}

#[test]
fn cli_convert_png_selects_the_comment_listing_after_changed_pages() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.docx");
    let output = dir.path().join("selected.png");
    let source = four_pages(true);
    std::fs::write(&input, &source).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args([
            "convert",
            "-t",
            "png",
            "--move-comments",
            "--changed-only",
            "--pages",
            "3",
            "--dpi",
            "12",
            "-o",
        ])
        .arg(&output)
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let expected = render(
        &source,
        PdfOptions {
            comments: CommentPlacement::End,
            changed_only: true,
            ..PdfOptions::default()
        },
        RenderRequest {
            pdf: false,
            png_dpi: Some(12.0),
            pages: Some(vec![2]),
        },
    )
    .unwrap();
    assert!(expected.report.pages[2].text.contains("First note here"));
    assert_eq!(
        std::fs::read(dir.path().join("selected-page-03.png")).unwrap(),
        expected.pngs[0]
    );
    assert!(!dir.path().join("selected-page-01.png").exists());
    assert!(!dir.path().join("selected-page-02.png").exists());
}

#[test]
fn cli_diff_passes_page_options_to_pdf_rendering() {
    let dir = tempfile::tempdir().unwrap();
    let old = dir.path().join("old.docx");
    let new = dir.path().join("new.docx");
    let redline = dir.path().join("redline.docx");
    let pdf = dir.path().join("redline.pdf");
    let body = |word: &str| {
        format!(
            r#"<w:p><w:r><w:t>Alpha page text</w:t></w:r></w:p>{PAGE_BREAK}<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>The middle has {word} wording.</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p>{PAGE_BREAK}<w:p><w:r><w:t>Unchanged last page</w:t></w:r></w:p>"#
        )
    };
    std::fs::write(&old, with_comments(&body("original"), COMMENT_XML)).unwrap();
    std::fs::write(&new, with_comments(&body("replacement"), COMMENT_XML)).unwrap();
    let baseline = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["diff", "-o"])
        .arg(&redline)
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert!(
        baseline.status.success(),
        "{}",
        String::from_utf8_lossy(&baseline.stderr)
    );
    let result = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["diff", "--move-comments", "--changed-only", "-o"])
        .arg(&pdf)
        .arg(&old)
        .arg(&new)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let docx = std::fs::read(redline).unwrap();
    let expected = rendered(
        &docx,
        PdfOptions {
            comments: CommentPlacement::End,
            changed_only: true,
            ..PdfOptions::default()
        },
    );
    let pages = texts(&expected);
    assert!(pages.last().unwrap().starts_with("Comments"), "{pages:?}");
    assert!(
        !pages.iter().any(|p| p.contains("Alpha page text")),
        "{pages:?}"
    );
    assert!(
        std::fs::read(pdf).unwrap() == expected.pdf.unwrap(),
        "diff must render its redline with both requested page options"
    );
}
