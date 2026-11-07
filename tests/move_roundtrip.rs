// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! A move beside another change stays a move when the view is written back.

mod common;

use common::docx::{docx, part_string};
use jubarte::changes::{ChangeKind, list_changes};
use jubarte::markdown::{DocxOptions, MarkdownOptions, docx_to_markdown, markdown_to_docx};

const BOB: &str = "Bob";
const DATE: &str = "2026-10-01T10:00:00Z";

fn moved_beside_an_insertion() -> Vec<u8> {
    let range = |tag: &str, id: u32| {
        format!(r#"<w:{tag} w:id="{id}" w:author="{BOB}" w:date="{DATE}" w:name="move1"/>"#)
    };
    let mark = format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="6" w:author="{BOB}" w:date="{DATE}"/></w:rPr></w:pPr>"#
    );
    docx(&format!(
        "{mark}\
         <w:r><w:t xml:space=\"preserve\">Pay </w:t></w:r>\
         {start}\
         <w:moveFrom w:id=\"3\" w:author=\"{BOB}\" w:date=\"{DATE}\"><w:r><w:t xml:space=\"preserve\">General</w:t></w:r></w:moveFrom>\
         <w:moveFromRangeEnd w:id=\"20\"/>\
         <w:ins w:id=\"5\" w:author=\"{BOB}\" w:date=\"{DATE}\"><w:r><w:t xml:space=\"preserve\">CONSENT</w:t></w:r></w:ins>\
         </w:p>\
         <w:p><w:r><w:t xml:space=\"preserve\">Then </w:t></w:r>\
         {to}\
         <w:moveTo w:id=\"4\" w:author=\"{BOB}\" w:date=\"{DATE}\"><w:r><w:t xml:space=\"preserve\">General</w:t></w:r></w:moveTo>\
         <w:moveToRangeEnd w:id=\"21\"/>\
         <w:r><w:t xml:space=\"preserve\"> end.</w:t></w:r></w:p>",
        start = range("moveFromRangeStart", 20),
        to = range("moveToRangeStart", 21),
    ))
}

fn kinds(doc: &[u8]) -> Vec<(String, ChangeKind)> {
    list_changes(doc)
        .expect("changes lists")
        .into_iter()
        .map(|change| (change.id, change.kind))
        .collect()
}

#[test]
fn a_move_beside_a_change_keeps_both_move_elements() {
    let doc = moved_beside_an_insertion();
    let source = kinds(&doc);
    assert!(
        source.contains(&("r3".to_string(), ChangeKind::Move))
            && source.contains(&("r4".to_string(), ChangeKind::Move)),
        "the source lists the move: {source:?}"
    );
    let view = docx_to_markdown(
        &doc,
        &MarkdownOptions {
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .expect("the view reads")
    .markdown;
    let copy = markdown_to_docx(&view, &DocxOptions::default())
        .expect("the view writes")
        .docx;
    let xml = part_string(&copy, "word/document.xml").expect("document.xml");
    assert!(xml.contains("<w:moveFrom "), "moveFrom survives:\n{xml}");
    assert!(xml.contains("<w:moveTo "), "moveTo survives:\n{xml}");
    assert!(
        xml.matches(r#"w:name="move1""#).count() >= 2,
        "both range names survive:\n{xml}"
    );
    let written = kinds(&copy);
    assert!(
        written.contains(&("r3".to_string(), ChangeKind::Move))
            && written.contains(&("r4".to_string(), ChangeKind::Move)),
        "changes lists two moves: {written:?}\n{view}"
    );
}
