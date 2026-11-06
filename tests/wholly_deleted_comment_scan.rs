// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
// SPDX-License-Identifier: AGPL-3.0-only

//! The wholly-deleted comment scan agrees with the characters accept removes.

mod common;

use jubarte::changes::{ChangeFilter, accept_changes, list_changes};
use jubarte::comments::list_comments;

const REV: &str = r#"w:author="Carol Diaz" w:date="2026-10-18T00:00:00Z""#;
const MATH: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

fn package(body: &str) -> Vec<u8> {
    let w = common::docx::W_NS;
    let w14 = "http://schemas.microsoft.com/office/word/2010/wordml";
    let comments = format!(
        r#"<w:comments xmlns:w="{w}" xmlns:w14="{w14}"><w:comment w:id="5" w:author="Alice"><w:p w14:paraId="00000001"><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    common::docx::docx_with(
        body,
        &[common::docx::Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    )
}

fn comment_ids(docx: &[u8]) -> Vec<u32> {
    list_comments(docx)
        .expect("comments list")
        .into_iter()
        .map(|comment| comment.id)
        .collect()
}

fn change_ids(docx: &[u8]) -> Vec<String> {
    list_changes(docx)
        .expect("changes list")
        .into_iter()
        .map(|change| change.id)
        .collect()
}

#[test]
fn a_custom_xml_move_from_range_removes_its_comment() {
    let source = package(&format!(
        r#"<w:p><w:commentRangeStart w:id="5"/><w:customXmlMoveFromRangeStart w:id="4"/><w:r><w:t>gone</w:t></w:r><w:customXmlMoveFromRangeEnd w:id="4"/><w:commentRangeEnd w:id="5"/><w:r><w:commentReference w:id="5"/></w:r><w:del w:id="7" {REV}><w:r><w:delText>other</w:delText></w:r></w:del></w:p>"#
    ));
    assert!(
        comment_ids(&source).contains(&5),
        "the comment must be anchored before accept"
    );
    let accepted = accept_changes(&source, &ChangeFilter::default()).expect("accept");
    let left = comment_ids(&accepted);
    assert!(
        !left.contains(&5),
        "comment survived a custom xml move-from range: {left:?}"
    );
}

#[test]
fn an_unselected_deletion_keeps_its_comment() {
    let source = package(&format!(
        r#"<w:p><w:commentRangeStart w:id="5"/><w:del w:id="7" {REV}><w:r><w:delText>selected</w:delText></w:r></w:del><w:del w:id="8" w:author="Bob" w:date="2026-10-18T00:00:00Z"><w:r><w:delText>kept</w:delText></w:r></w:del><w:commentRangeEnd w:id="5"/><w:r><w:commentReference w:id="5"/></w:r></w:p>"#
    ));
    let before = change_ids(&source);
    assert!(
        before.iter().any(|id| id == "r7") && before.iter().any(|id| id == "r8"),
        "the range must list both deletions before accept: {before:?}"
    );
    let accepted = accept_changes(&source, &ChangeFilter::ids(["r7"])).expect("accept r7");
    let left = comment_ids(&accepted);
    assert!(
        left.contains(&5),
        "comment removed while an unselected deletion remains: {left:?}"
    );
    let after = change_ids(&accepted);
    assert!(
        after.iter().any(|id| id == "r8"),
        "unselected deletion r8 is gone: {after:?}"
    );
}

#[test]
fn a_deleted_field_instruction_removes_its_comment() {
    let source = package(&format!(
        r#"<w:p><w:commentRangeStart w:id="5"/><w:del w:id="1" {REV}><w:r><w:fldChar w:fldCharType="begin"/></w:r></w:del><w:r><w:instrText xml:space="preserve"> DATE </w:instrText></w:r><w:del w:id="2" {REV}><w:r><w:fldChar w:fldCharType="end"/></w:r></w:del><w:commentRangeEnd w:id="5"/><w:r><w:commentReference w:id="5"/></w:r></w:p>"#
    ));
    assert!(
        comment_ids(&source).contains(&5),
        "the comment must be anchored before accept"
    );
    let accepted = accept_changes(&source, &ChangeFilter::default()).expect("accept");
    let left = comment_ids(&accepted);
    assert!(
        !left.contains(&5),
        "comment survived a deleted field: {left:?}"
    );
}

#[test]
fn a_deleted_fraction_removes_its_comment() {
    let source = package(&format!(
        r#"<w:p><w:commentRangeStart w:id="5"/><m:oMath xmlns:m="{MATH}"><m:f><m:fPr><m:ctrlPr><w:del w:id="3" {REV}/></m:ctrlPr></m:fPr><m:num><m:r><m:t>x</m:t></m:r></m:num><m:den><m:r><m:t>y</m:t></m:r></m:den></m:f></m:oMath><w:commentRangeEnd w:id="5"/><w:r><w:commentReference w:id="5"/></w:r></w:p>"#
    ));
    assert!(
        comment_ids(&source).contains(&5),
        "the comment must be anchored before accept"
    );
    let accepted = accept_changes(&source, &ChangeFilter::default()).expect("accept");
    let left = comment_ids(&accepted);
    assert!(
        !left.contains(&5),
        "comment survived a deleted fraction: {left:?}"
    );
}
