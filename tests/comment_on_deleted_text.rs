// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A comment on text that goes. In the redline it stays on the deleted
//! text, inside the paragraph, as Word writes it; in the clean copy it goes
//! when its reference went with the text, as Accept All takes it (rule R1).
//! Every output is Word-valid: the redline that once put the comment's
//! reference run straight into `w:body` made Word load the document empty
//! (2026-10-10).

mod common;

use common::docx::{docx, para, part_string, replace_entry};
use common::validity::assert_word_valid_package;
use jubarte::comments::list_comments;
use jubarte::document_comparer::{accept_revisions, compare_documents};
use jubarte::edit::{EditPlan, apply_plan};

fn plan(author: &str, operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"{author}","operations":{operations}}}"#
    ))
    .unwrap()
}

/// "The cap is 10." / "Kept.", with Ann's comment on "cap".
fn commented() -> Vec<u8> {
    apply_plan(
        &docx(&(para("The cap is 10.") + &para("Kept."))),
        &plan(
            "Ann",
            r#"[{"kind":"comment","paragraph":"p0","find":"cap","text":"Too low"}]"#,
        ),
    )
    .unwrap()
    .clean
}

fn body(docx: &[u8]) -> String {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let start = xml.find("<w:body>").unwrap();
    xml[start..].to_string()
}

/// The comment's reference sits in a paragraph, inside the deletion.
fn reference_in_deleted_text(redline: &[u8]) -> bool {
    let body = body(redline);
    let reference = body.find("<w:commentReference").unwrap();
    let del = body[..reference].rfind("<w:del ").unwrap();
    !body[..reference].ends_with("<w:body>")
        && !body[del..reference].contains("</w:del>")
        && body[..reference].rfind("<w:p>").is_some_and(|p| p < del)
}

#[test]
fn a_deleted_paragraph_or_whole_text_takes_the_comment_only_on_accept() {
    for op in [
        r#"{"kind":"delete_paragraph","paragraph":"p0"}"#,
        r#"{"kind":"delete","paragraph":"p0","find":"The cap is 10."}"#,
        r#"{"kind":"replace","paragraph":"p0","find":"The cap is","replacement":"It is"}"#,
    ] {
        let out = apply_plan(&commented(), &plan("Bob", &format!("[{op}]"))).unwrap();
        assert_word_valid_package(&out.clean);
        assert_word_valid_package(&out.redline);
        assert!(list_comments(&out.clean).unwrap().is_empty(), "{op}");
        assert!(
            part_string(&out.clean, "word/comments.xml").is_none(),
            "{op}: no comment part is left behind"
        );
        let shown = list_comments(&out.redline).unwrap();
        assert_eq!(shown.len(), 1, "{op}");
        assert_eq!(shown[0].text, "Too low");
        assert!(
            reference_in_deleted_text(&out.redline),
            "{op}: {}",
            body(&out.redline)
        );
        // Accepting the redline gives the clean copy's comments.
        let accepted = accept_revisions(&out.redline).unwrap();
        assert!(list_comments(&accepted).unwrap().is_empty(), "{op}");
    }
}

#[test]
fn deleting_only_the_commented_word_keeps_the_comment() {
    let out = apply_plan(
        &commented(),
        &plan(
            "Bob",
            r#"[{"kind":"delete","paragraph":"p0","find":"cap"}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    assert_eq!(list_comments(&out.clean).unwrap().len(), 1);
    assert_eq!(list_comments(&out.redline).unwrap().len(), 1);
    let accepted = accept_revisions(&out.redline).unwrap();
    assert_eq!(list_comments(&accepted).unwrap().len(), 1);
}

/// B still defines A's comment but lost its anchors with the paragraph:
/// A's anchor carries it, as Word's Compare does.
#[test]
fn compare_carries_a_comment_the_revision_defines_without_anchors() {
    let a = commented();
    let b = replace_entry(
        &a,
        "word/document.xml",
        part_string(&a, "word/document.xml")
            .unwrap()
            .replace(
                &body(&a)[..body(&a).find("</w:p>").unwrap() + 6],
                "<w:body>",
            )
            .as_bytes(),
    );
    assert!(!body(&b).contains("commentReference"));
    let redline = compare_documents(&a, &b, "Redline").unwrap();
    assert_word_valid_package(&redline);
    assert_eq!(list_comments(&redline).unwrap().len(), 1);
    assert!(reference_in_deleted_text(&redline), "{}", body(&redline));
}

/// A revision whose only paragraph text went, leaving the comment as a
/// point at the start of the emptied paragraph: the point stays inside it.
#[test]
fn compare_keeps_a_point_comment_at_a_paragraph_start_inside_the_paragraph() {
    let a = commented();
    let emptied = body(&a);
    let first_p = &emptied[..emptied.find("</w:p>").unwrap() + 6];
    let point = r#"<w:body><w:p><w:commentRangeStart w:id="0"/><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#;
    let b = replace_entry(
        &a,
        "word/document.xml",
        part_string(&a, "word/document.xml")
            .unwrap()
            .replace(first_p, point)
            .as_bytes(),
    );
    let redline = compare_documents(&a, &b, "Redline").unwrap();
    assert_word_valid_package(&redline);
    let body = body(&redline);
    assert!(
        !body.starts_with("<w:body><w:commentRange") && !body.starts_with("<w:body><w:r>"),
        "{body}"
    );
    assert_eq!(list_comments(&redline).unwrap().len(), 1);
}
