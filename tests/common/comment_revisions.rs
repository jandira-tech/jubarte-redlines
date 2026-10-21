// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Synthetic packages for issue #1223: tracked changes inside comment bodies
//! (`word/comments.xml`), which Word's Accept All / Reject All resolves.

use crate::common::docx::{Part, W_NS, docx_with};

/// The anchors of comment `id` in a body paragraph: range start, `text`,
/// range end and reference, as Word writes them.
pub fn anchor(id: u32, text: &str) -> String {
    format!(
        r#"<w:commentRangeStart w:id="{id}"/><w:r><w:t>{text}</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r>"#
    )
}

/// One `w:comment` with the given attributes and body XML (paragraphs).
pub fn comment(id: u32, author: &str, body: &str) -> String {
    format!(r#"<w:comment w:id="{id}" w:author="{author}">{body}</w:comment>"#)
}

/// A DOCX: `body` under `word/document.xml`, `comments` as the `w:comment`
/// elements of `word/comments.xml`. Every comment listed must be anchored in
/// `body` (see [`anchor`]); an unanchored one is pruned with its revisions.
pub fn docx(body: &str, comments: &[String]) -> Vec<u8> {
    let w14 = "http://schemas.microsoft.com/office/word/2010/wordml";
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}" xmlns:w14="{w14}">{}</w:comments>"#,
        comments.concat()
    );
    docx_with(
        body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    )
}
