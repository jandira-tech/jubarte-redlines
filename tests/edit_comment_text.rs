// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Editing the text a comment covers (Task 23): the comment keeps covering
//! both sides of a tracked replacement, so Accept All leaves it on the new
//! text and Reject All on the old.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::comments::list_comments;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};

fn plan(json: &str) -> EditPlan {
    EditPlan::from_json(json).unwrap()
}

/// "The fee is ten dollars per hour." with comment 0 on exactly "ten
/// dollars".
fn commented() -> Vec<u8> {
    let source = docx(&para("The fee is ten dollars per hour."));
    apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
            {"kind":"comment","paragraph":"body:p:0","find":"ten dollars","text":"Check the rate"}]}"#,
        ),
    )
    .unwrap()
    .clean
}

/// The anchor text of the only comment in `docx`.
fn covered(docx: &[u8]) -> String {
    let comments = list_comments(docx).unwrap();
    assert_eq!(comments.len(), 1, "{comments:?}");
    comments[0].anchor_text.clone()
}

/// The comment's range markers and the revisions between them, in document
/// order: `start`, `end`, `ref`, `del:TEXT`, `ins:TEXT`.
fn shape(docx: &[u8]) -> Vec<String> {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let mut out = Vec::new();
    let mut rest = xml.as_str();
    while let Some(at) = rest.find('<') {
        rest = &rest[at..];
        let tag_end = rest.find('>').unwrap();
        let tag = &rest[..=tag_end];
        let name = tag
            .trim_start_matches(['<', '/'])
            .split([' ', '>', '/'])
            .next()
            .unwrap();
        let text_of = |close: &str| {
            let body = &rest[..rest.find(close).unwrap()];
            body.split("<w:t")
                .skip(1)
                .chain(body.split("<w:delText").skip(1))
                .filter_map(|t| t.split_once('>').map(|(_, v)| v.split('<').next().unwrap()))
                .collect::<String>()
        };
        match name {
            "w:commentRangeStart" => out.push("start".into()),
            "w:commentRangeEnd" => out.push("end".into()),
            "w:commentReference" => out.push("ref".into()),
            "w:del" if !tag.starts_with("</") => out.push(format!("del:{}", text_of("</w:del>"))),
            "w:ins" if !tag.starts_with("</") => out.push(format!("ins:{}", text_of("</w:ins>"))),
            _ => {}
        }
        rest = &rest[tag_end + 1..];
    }
    out
}

#[test]
fn replacing_the_whole_commented_text_keeps_the_deletion_inside_the_comment() {
    let source = commented();
    assert_eq!(covered(&source), "ten dollars");
    let edited = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Bob","date":"2026-10-12T00:00:00Z","operations":[
            {"kind":"replace","paragraph":"body:p:0","find":"ten dollars","replacement":"twelve euros"}]}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&edited.redline);
    assert_eq!(
        shape(&edited.redline),
        ["start", "del:ten dollars", "ins:twelve euros", "end", "ref"],
        "Word's order: the deletion, then the insertion, both inside the range"
    );
    let accepted = accept_revisions(&edited.redline).unwrap();
    let rejected = reject_revisions(&edited.redline).unwrap();
    assert_word_valid_package(&accepted);
    assert_word_valid_package(&rejected);
    assert_eq!(covered(&accepted), "twelve euros");
    assert_eq!(covered(&rejected), "ten dollars");
}
