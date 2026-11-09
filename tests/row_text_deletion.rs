// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A deletion that fills a table cell stays a text deletion when the view
//! is written back. A row the package marks deleted stays a row deletion.

mod common;

use jubarte::changes::list_changes;
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

fn cell(inner: &str) -> String {
    format!("<w:tc><w:tcPr><w:tcW w:w=\"9360\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc>")
}

fn table(row: &str) -> Vec<u8> {
    let body = format!("<w:tbl><w:tblGrid><w:gridCol w:w=\"9360\"/></w:tblGrid>{row}</w:tbl>");
    common::docx::docx(&body)
}

fn described(docx: &[u8]) -> String {
    list_changes(docx)
        .expect("the changes list")
        .into_iter()
        .map(|change| {
            format!(
                "{} {:?} {} {:?}",
                change.id, change.kind, change.target, change.text
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn written(docx: &[u8]) -> (String, Vec<u8>) {
    let view = view_of(docx);
    let package = markdown_to_docx(&view, &DocxOptions::default())
        .expect("the view converts")
        .docx;
    common::validity::assert_word_valid_package(&package);
    (view, package)
}

#[test]
fn a_deletion_that_fills_a_row_stays_on_the_text() {
    let source = table(&format!(
        "<w:tr>{}</w:tr>",
        cell(
            "<w:p><w:del w:id=\"1\" w:author=\"A\" w:date=\"2026-10-01T09:00:00Z\"><w:r><w:delText>y1</w:delText></w:r></w:del></w:p>"
        )
    ));
    let (view, package) = written(&source);
    let got = described(&package);
    assert!(
        got.contains("Deletion text \"y1\""),
        "the written deletion is the cell text:\n{got}\nview:\n{view}"
    );
    assert!(
        !got.contains("table_row"),
        "the row itself is not the deletion:\n{got}\nview:\n{view}"
    );
    let document = common::docx::part_string(&package, "word/document.xml").expect("document");
    assert!(
        document.contains("<w:delText"),
        "the cell text stays deleted:\n{document}"
    );
}

#[test]
fn a_deleted_row_stays_a_row_deletion() {
    let source = table(&format!(
        "<w:tr><w:trPr><w:del w:id=\"1\" w:author=\"A\" w:date=\"2026-10-01T09:00:00Z\"/></w:trPr>{}</w:tr>",
        cell("<w:p><w:r><w:t>y1</w:t></w:r></w:p>")
    ));
    let (view, package) = written(&source);
    let got = described(&package);
    assert!(
        got.contains("Deletion table_row"),
        "the written deletion is the row:\n{got}\nview:\n{view}"
    );
}

/// A plain (`--commonmark`) view prints no table lines. A row Word deleted
/// (the row's mark and its runs') still comes back a row deletion there.
#[test]
fn a_plain_view_keeps_a_deleted_row() {
    let source = table(&format!(
        "<w:tr><w:trPr><w:del w:id=\"1\" w:author=\"A\" w:date=\"2026-10-01T09:00:00Z\"/></w:trPr>{}</w:tr><w:tr>{}</w:tr>",
        cell(
            "<w:p><w:del w:id=\"2\" w:author=\"A\" w:date=\"2026-10-01T09:00:00Z\"><w:r><w:delText>y1</w:delText></w:r></w:del></w:p>"
        ),
        cell("<w:p><w:r><w:t>kept</w:t></w:r></w:p>")
    ));
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.docx"), &source).unwrap();
    let run = |args: &[&str]| {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(args)
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    run(&[
        "-q",
        "convert",
        "a.docx",
        "-o",
        "a.md",
        "--commonmark",
        "--no-page-markers",
    ]);
    run(&["-q", "convert", "a.md", "-o", "b.docx"]);
    let view = std::fs::read_to_string(dir.path().join("a.md")).unwrap();
    let package = std::fs::read(dir.path().join("b.docx")).unwrap();
    let got = described(&package);
    assert!(
        got.contains("Deletion table_row"),
        "the plain view lost the row deletion:\n{got}\nview:\n{view}"
    );
}
