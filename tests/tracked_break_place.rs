// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A page break between the runs of a tracked change stays between those
//! runs when the view is written back, in the mark that held it.

mod common;

use jubarte::changes::list_logical_changes;
use jubarte::markdown::{DocxOptions, ReadOptions, markdown_to_docx, read};

const DATE: &str = "2026-10-01T09:00:00Z";

fn source(inner: &str) -> Vec<u8> {
    common::docx::docx(&format!("<w:p>{inner}</w:p>"))
}

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
    .lines()
    .filter(|line| !line.starts_with("sha256: ") && !line.starts_with("body: "))
    .collect::<Vec<_>>()
    .join("\n")
}

fn written(docx: &[u8]) -> (String, Vec<u8>, String) {
    let first = view_of(docx);
    let copy = markdown_to_docx(&first, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    let second = view_of(&copy);
    (first, copy, second)
}

fn described(docx: &[u8]) -> Vec<String> {
    list_logical_changes(docx)
        .expect("the changes list")
        .into_iter()
        .map(|change| format!("{} {:?} {:?}", change.id, change.kind, change.text))
        .collect()
}

fn break_sits_between(xml: &str, before: &str, after: &str) -> bool {
    let left = xml.find(before);
    let right = xml.find(after);
    let br = xml.find("<w:br w:type=\"page\"/>");
    matches!((left, br, right), (Some(left), Some(br), Some(right)) if left < br && br < right)
}

#[test]
fn a_page_break_inside_one_insertion_stays_between_its_runs() {
    let docx = source(&format!(
        r#"<w:r><w:t xml:space="preserve">K </w:t></w:r><w:ins w:id="7" w:author="B" w:date="{DATE}"><w:r><w:t>a</w:t></w:r><w:r><w:br w:type="page"/></w:r><w:r><w:t>b</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> T.</w:t></w:r>"#
    ));
    let (first, copy, second) = written(&docx);
    let xml = common::docx::part_string(&copy, "word/document.xml").expect("document");
    assert!(
        break_sits_between(&xml, ">a<", ">b<"),
        "the break stays between a and b:\n{xml}\nview:\n{first}\nagain:\n{second}"
    );
    assert_eq!(xml.matches("<w:ins ").count(), 1, "{xml}");
    assert_eq!(described(&copy), described(&docx));
    assert_eq!(second, first);
    common::validity::assert_word_valid_package(&copy);
}

#[test]
fn a_page_break_between_two_marks_of_one_change_keeps_those_marks() {
    let docx = source(&format!(
        r#"<w:r><w:t xml:space="preserve">K </w:t></w:r><w:ins w:id="7" w:author="B" w:date="{DATE}"><w:r><w:t>a</w:t></w:r><w:r><w:br w:type="page"/></w:r></w:ins><w:ins w:id="8" w:author="B" w:date="{DATE}"><w:r><w:t>b</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> T.</w:t></w:r>"#
    ));
    let (first, copy, second) = written(&docx);
    let xml = common::docx::part_string(&copy, "word/document.xml").expect("document");
    assert!(
        break_sits_between(&xml, ">a<", ">b<"),
        "the break stays between the two marks:\n{xml}\nview:\n{first}\nagain:\n{second}"
    );
    assert!(
        xml.contains(r#"w:id="7""#) && xml.contains(r#"w:id="8""#),
        "{xml}"
    );
    assert_eq!(xml.matches("<w:ins ").count(), 2, "{xml}");
    assert_eq!(described(&copy), described(&docx));
    assert_eq!(second, first);
    common::validity::assert_word_valid_package(&copy);
}
