// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A tracked change inside a comment body stays a change when the view
//! is written back. A brace sequence that is already text stays text.

mod common;

use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

fn view_of(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
}

fn comment_notes(markdown: &str) -> String {
    markdown
        .lines()
        .filter(|line| line.contains("{>>"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn probe() -> Vec<u8> {
    let body = r#"<w:p><w:commentRangeStart w:id="1"/><w:r><w:t>anchor</w:t></w:r><w:commentRangeEnd w:id="1"/><w:r><w:commentReference w:id="1"/></w:r></w:p>"#;
    let comments = r#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="1" w:author="Carla" w:date="2026-10-01T09:00:00Z"><w:p><w:r><w:t xml:space="preserve">see </w:t></w:r><w:ins w:id="5" w:author="Carla" w:date="2026-10-01T09:00:00Z"><w:r><w:t>this</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> not </w:t></w:r><w:del w:id="6" w:author="Carla" w:date="2026-10-01T09:00:00Z"><w:r><w:delText>that</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> {++kept++}</w:t></w:r></w:p></w:comment></w:comments>"#;
    common::docx::docx_with(
        body,
        &[common::docx::Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: comments,
        }],
    )
}

#[test]
fn a_comment_body_change_reads_back_as_a_change() {
    let source = probe();
    let first = view_of(&source);
    let note = comment_notes(&first);
    assert!(
        note.contains("{++this++}"),
        "the insertion is a mark in the first view:\n{first}"
    );
    assert!(
        note.contains("{--that--}"),
        "the deletion is a mark in the first view:\n{first}"
    );
    assert!(
        note.contains("{\\++kept++\\}"),
        "a brace sequence that is text stays escaped:\n{first}"
    );
    let written = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view converts")
        .docx;
    common::validity::assert_word_valid_package(&written);
    let comments = common::docx::part_string(&written, "word/comments.xml").expect("comments part");
    assert!(
        comments.contains("<w:ins "),
        "the written comment keeps the insertion:\n{comments}"
    );
    assert!(
        comments.contains("<w:del "),
        "the written comment keeps the deletion:\n{comments}"
    );
    assert!(
        comments.contains(">this<"),
        "the insertion's text is the word, not an escaped mark:\n{comments}"
    );
    let second = view_of(&written);
    assert_eq!(
        comment_notes(&second),
        note,
        "first:\n{first}\nsecond:\n{second}\ncomments:\n{comments}"
    );
}
