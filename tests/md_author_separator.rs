// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A plain-view author that itself holds ` re ` and a revision id stays one
//! author. A deletion nested in an insertion still names both people.

mod common;

use common::docx::{docx, part_string};
use jubarte::markdown::{DocxOptions, MarkdownOptions, docx_to_markdown, markdown_to_docx};

fn plain(docx: &[u8]) -> String {
    docx_to_markdown(docx, &MarkdownOptions::default())
        .unwrap()
        .markdown
}

fn written(markdown: &str) -> Vec<u8> {
    markdown_to_docx(markdown, &DocxOptions::default())
        .unwrap()
        .docx
}

/// One deletion by `Bo re r1 Kim`, and no insertion. The plain view's note
/// must come back as that same author.
#[test]
fn a_plain_author_holding_re_and_an_id_stays_one_deletion() {
    let source = docx(
        r#"<w:p><w:del w:id="3" w:author="Bo re r1 Kim" w:date="2026-09-30T09:00:00Z"><w:r><w:delText>kept</w:delText></w:r></w:del></w:p>"#,
    );
    let markdown = plain(&source);
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert!(
        document.contains(r#"w:author="Bo re r1 Kim""#),
        "{markdown}: {document}"
    );
    assert!(!document.contains("<w:ins"), "{markdown}: {document}");
}

/// The same author inside another person's insertion still reads as a nest.
#[test]
fn a_nested_author_holding_re_and_an_id_stays_nested() {
    let source = docx(
        r#"<w:p><w:ins w:id="1" w:author="Ana Lima" w:date="2026-09-29T14:05:00Z"><w:del w:id="3" w:author="Bo re r1 Kim" w:date="2026-09-30T09:00:00Z"><w:r><w:delText>two</w:delText></w:r></w:del></w:ins></w:p>"#,
    );
    let markdown = plain(&source);
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert!(
        document.contains(r#"w:author="Bo re r1 Kim""#)
            && document.contains(r#"w:author="Ana Lima""#),
        "{markdown}: {document}"
    );
    assert!(markdown.contains(" re "), "{markdown}");
}

/// Without a date the note has no date group, and the author still comes
/// back with its space, not with the `%20` the writer spelled.
#[test]
fn a_dateless_author_holding_re_and_an_id_keeps_its_space() {
    let source = docx(
        r#"<w:p><w:del w:id="3" w:author="Bo re r1 Kim"><w:r><w:delText>kept</w:delText></w:r></w:del></w:p>"#,
    );
    let markdown = plain(&source);
    let document = part_string(&written(&markdown), "word/document.xml").unwrap();
    assert!(
        document.contains(r#"w:author="Bo re r1 Kim""#),
        "{markdown}: {document}"
    );
    assert!(!document.contains("%20"), "{markdown}: {document}");
}
