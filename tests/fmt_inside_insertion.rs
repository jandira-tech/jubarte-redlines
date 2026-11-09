// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A formatting change on a run inside a tracked insertion names that
//! insertion on the paragraph's fmt tag, and the tag still reads after
//! the view is written back.

mod common;

use jubarte::changes::{ChangeKind, list_logical_changes};
use jubarte::markdown::{DocxOptions, ReadOptions, TrackChanges, markdown_to_docx, read};

fn view(docx: &[u8]) -> String {
    read(
        docx,
        &ReadOptions {
            page_markers: false,
            track_changes: TrackChanges::All,
            ..ReadOptions::default()
        },
    )
    .expect("the docx reads")
    .markdown
}

fn fmt_line(markdown: &str) -> String {
    markdown
        .lines()
        .find(|line| line.contains("fmt "))
        .unwrap_or("")
        .to_string()
}

fn inserted_bold() -> Vec<u8> {
    common::docx::docx(
        r#"<w:p><w:ins w:id="96" w:author="Rita Cruz" w:date="2026-01-01T00:00:00Z"><w:r><w:t xml:space="preserve">the </w:t></w:r><w:r><w:rPr><w:b/><w:rPrChange w:id="95" w:author="Tom Lee" w:date="2026-01-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr><w:t>word</w:t></w:r><w:r><w:t xml:space="preserve"> end</w:t></w:r></w:ins></w:p>"#,
    )
}

#[test]
fn a_formatting_change_inside_an_insertion_names_that_parent() {
    let source = inserted_bold();
    let changes = list_logical_changes(&source).expect("changes list");
    let formatting = changes
        .iter()
        .find(|change| change.kind == ChangeKind::Formatting)
        .expect("a formatting change");
    assert_eq!(formatting.inside.as_deref(), Some("r96"));
    let first = view(&source);
    let fmt = fmt_line(&first);
    assert!(
        fmt.contains("r95") && fmt.contains("re r96"),
        "fmt tag names no parent:\n{first}"
    );
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let document = common::docx::part_string(&again, "word/document.xml").unwrap();
    let ins = document.find("<w:ins ").expect("the insertion remains");
    let change = document
        .find("<w:rPrChange ")
        .expect("the formatting change remains");
    let end = document
        .find("</w:ins>")
        .unwrap_or_else(|| panic!("the insertion closes:\n{document}"));
    assert!(ins < change && change < end, "{document}");
    let second = view(&again);
    let fmt_again = fmt_line(&second);
    assert!(
        fmt_again.contains("r95") && fmt_again.contains("re r96"),
        "round trip dropped the parent:\n{second}"
    );
}

/// coderabbit review 5816561921: the table-cell `fmt … in pN` entry names
/// the insertion too, and keeps it through a round trip.
#[test]
fn a_cell_formatting_change_inside_an_insertion_names_that_parent() {
    let source = common::docx::docx(
        r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t xml:space="preserve">Kept </w:t></w:r><w:ins w:id="96" w:author="Rita Cruz" w:date="2026-01-01T00:00:00Z"><w:r><w:t xml:space="preserve">the </w:t></w:r><w:r><w:rPr><w:b/><w:rPrChange w:id="95" w:author="Tom Lee" w:date="2026-01-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr><w:t>word</w:t></w:r></w:ins></w:p></w:tc></w:tr></w:tbl><w:p><w:r><w:t>After</w:t></w:r></w:p>"#,
    );
    let first = view(&source);
    let fmt = fmt_line(&first);
    assert!(
        fmt.contains("r95") && fmt.contains("re r96") && fmt.contains(" in p0"),
        "cell fmt entry names no parent:\n{first}"
    );
    let again = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    common::validity::assert_word_valid_package(&again);
    let document = common::docx::part_string(&again, "word/document.xml").unwrap();
    let ins = document.find("<w:ins ").expect("the insertion remains");
    let change = document
        .find("<w:rPrChange ")
        .unwrap_or_else(|| panic!("the formatting change is gone:\n{document}"));
    let end = document
        .find("</w:ins>")
        .unwrap_or_else(|| panic!("the insertion closes:\n{document}"));
    assert!(ins < change && change < end, "{document}");
    let second = view(&again);
    let fmt_again = fmt_line(&second);
    assert!(
        fmt_again.contains("r95") && fmt_again.contains("re r96"),
        "round trip dropped the parent:\n{second}"
    );
}
