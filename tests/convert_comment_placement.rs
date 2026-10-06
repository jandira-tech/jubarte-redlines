// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Where a converted PDF puts its comments (`PdfOptions::comments`) and
//! which pages it keeps (`PdfOptions::changed_only`).

mod common;

use std::process::Command;

use common::docx::{Part, docx_with};
use jubarte::convert::{CommentPlacement, PdfOptions, RenderRequest, Rendered, render};

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
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("comment_placement_cli");
    std::fs::create_dir_all(&dir).unwrap();
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
