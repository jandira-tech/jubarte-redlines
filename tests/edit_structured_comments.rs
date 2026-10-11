// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A comment edits nothing, so it may sit on any paragraph Word comments:
//! one holding a tab, a symbol, a hyperlink, a field, a content control or
//! another author's tracked change. A whole-paragraph comment and an
//! anchored one both apply, and the package stays Word-valid.

mod common;

use common::docx::{docx, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeFilter, accept_changes, list_changes, reject_changes};
use jubarte::edit::{EditPlan, EditResult, apply_plan};
use jubarte::inspect::paragraphs;

const TAB: &str =
    r#"<w:p><w:r><w:t>Largeco, Inc.</w:t><w:tab/><w:t>Agentco, INC.</w:t></w:r></w:p>"#;
const SYMBOL: &str = r#"<w:p><w:r><w:t xml:space="preserve">Box </w:t></w:r><w:r><w:sym w:font="Wingdings" w:char="F0FC"/></w:r><w:r><w:t xml:space="preserve"> ticked</w:t></w:r></w:p>"#;
const LINK: &str = r#"<w:p><w:r><w:t xml:space="preserve">See </w:t></w:r><w:hyperlink w:anchor="terms"><w:r><w:t>the terms</w:t></w:r></w:hyperlink><w:r><w:t xml:space="preserve"> below.</w:t></w:r></w:p>"#;
const SIMPLE_FIELD: &str = r#"<w:p><w:r><w:t xml:space="preserve">Page </w:t></w:r><w:fldSimple w:instr=" PAGE "><w:r><w:t>3</w:t></w:r></w:fldSimple><w:r><w:t xml:space="preserve"> of the deed.</w:t></w:r></w:p>"#;
const COMPLEX_FIELD: &str = r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TITLE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>Master Agreement</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
const CONTROL: &str = r#"<w:p><w:r><w:t xml:space="preserve">Signed by </w:t></w:r><w:sdt><w:sdtPr><w:alias w:val="Signer"/></w:sdtPr><w:sdtContent><w:r><w:t>Ann Counsel</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t>.</w:t></w:r></w:p>"#;
const THEIRS: &str = r#"<w:p><w:r><w:t xml:space="preserve">Payment within </w:t></w:r><w:ins w:id="7" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins><w:del w:id="8" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:delText>30</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>"#;
const ALL_THEIRS: &str = r#"<w:p><w:ins w:id="9" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>Inserted clause.</w:t></w:r></w:ins></w:p>"#;

fn source() -> Vec<u8> {
    docx(
        &[
            TAB,
            SYMBOL,
            LINK,
            SIMPLE_FIELD,
            COMPLEX_FIELD,
            CONTROL,
            THEIRS,
            ALL_THEIRS,
        ]
        .concat(),
    )
}

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z","existing_revisions":"keep","operations":{operations}}}"#
    ))
    .unwrap()
}

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn mine() -> ChangeFilter {
    ChangeFilter {
        authors: Some(vec!["Ann Counsel".to_string()]),
        ..ChangeFilter::default()
    }
}

/// Every comment of `notes` is in the redline once, with one start, one
/// end and one reference each; the text is untouched, their revisions are
/// still theirs, and both packages are Word-valid. Rejecting every change
/// takes away the `gone_on_reject` comments whose text was all theirs, as
/// Word does (a comment whose reference is deleted goes).
fn check(out: &EditResult, notes: &[&str], gone_on_reject: usize) {
    let source = source();
    assert_eq!(texts(&out.clean), texts(&source));
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        texts(&source)
    );
    assert_eq!(
        list_changes(&out.redline).unwrap().len(),
        3,
        "their three changes stay"
    );
    let comments = part_string(&out.redline, "word/comments.xml").unwrap();
    let body = part_string(&out.redline, "word/document.xml").unwrap();
    for note in notes {
        assert_eq!(comments.matches(note).count(), 1, "{note}: {comments}");
    }
    let markers = |body: &str, n: usize| {
        for marker in [
            "<w:commentRangeStart",
            "<w:commentRangeEnd",
            "<w:commentReference",
        ] {
            assert_eq!(body.matches(marker).count(), n, "{marker}: {body}");
        }
    };
    markers(&body, notes.len());
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    // Accepting or rejecting every change, theirs too, keeps each comment
    // whole: no marker sits where a revision takes it away.
    for (settled, n) in [
        (
            accept_changes(&out.redline, &ChangeFilter::default()).unwrap(),
            notes.len(),
        ),
        (
            reject_changes(&out.redline, &ChangeFilter::default()).unwrap(),
            notes.len() - gone_on_reject,
        ),
    ] {
        markers(&part_string(&settled, "word/document.xml").unwrap(), n);
        let comments = part_string(&settled, "word/comments.xml").unwrap();
        assert_eq!(comments.matches("<w:comment ").count(), n, "{comments}");
        assert_word_valid_package(&settled);
    }
}

#[test]
fn a_whole_paragraph_comment_applies_to_any_paragraph() {
    let notes: Vec<String> = (0..8).map(|i| format!("Note on p{i}.")).collect();
    let ops: Vec<String> = notes
        .iter()
        .enumerate()
        .map(|(i, n)| format!(r#"{{"kind":"comment","paragraph":"p{i}","text":"{n}"}}"#))
        .collect();
    let out = apply_plan(&source(), &plan(&format!("[{}]", ops.join(",")))).unwrap();
    for op in &out.report.operations {
        assert_eq!(op.status, "ok", "{op:?}");
    }
    let notes: Vec<&str> = notes.iter().map(String::as_str).collect();
    // p7 is wholly their insertion.
    check(&out, &notes, 1);
}

#[test]
fn an_anchored_comment_may_cross_or_sit_inside_structure() {
    let out = apply_plan(
        &source(),
        &plan(
            r#"[{"kind":"comment","paragraph":"p0","find":"Inc.\tAgentco","text":"Across the tab."},
                {"kind":"comment","paragraph":"p1","find":"Box ￼ ticked","text":"Across the symbol."},
                {"kind":"comment","paragraph":"p2","find":"the terms","text":"Inside the link."},
                {"kind":"comment","paragraph":"p3","find":"Page 3 of","text":"Across the field."},
                {"kind":"comment","paragraph":"p5","find":"Counsel.","text":"Out of the control."},
                {"kind":"comment","paragraph":"p6","find":"45 days","text":"Over their insertion."}]"#,
        ),
    )
    .unwrap();
    check(
        &out,
        &[
            "Across the tab.",
            "Across the symbol.",
            "Inside the link.",
            "Across the field.",
            "Out of the control.",
            "Over their insertion.",
        ],
        0,
    );
    let contexts: Vec<_> = out
        .report
        .operations
        .iter()
        .map(|op| op.context.clone().unwrap_or_default())
        .collect();
    assert!(contexts[2].contains("{#the terms}"), "{contexts:?}");
    assert!(contexts[5].contains("{#45 days}"), "{contexts:?}");
}

#[test]
fn a_deleted_paragraph_may_carry_a_comment_whatever_it_holds() {
    let out = apply_plan(
        &source(),
        &plan(
            r#"[{"kind":"delete_paragraph","paragraph":"p2","comment":"Drop the link."},
                {"kind":"delete_paragraph","paragraph":"p1","comment":"Drop the box."}]"#,
        ),
    )
    .unwrap();
    for op in &out.report.operations {
        assert_eq!(op.status, "ok", "{op:?}");
    }
    let comments = part_string(&out.redline, "word/comments.xml").unwrap();
    assert!(
        comments.contains("Drop the link.") && comments.contains("Drop the box."),
        "{comments}"
    );
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        texts(&source())
    );
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
}

#[test]
fn a_comment_ending_inside_their_insertion_survives_rejecting_it() {
    let out = apply_plan(
        &source(),
        &plan(
            r#"[{"kind":"comment","paragraph":"p6","find":"Payment within 45","text":"Ends inside their insertion."}]"#,
        ),
    )
    .unwrap();
    check(&out, &["Ends inside their insertion."], 0);
    // The reference run is no part of their insertion.
    let body = part_string(&out.redline, "word/document.xml").unwrap();
    let ins = body.find("<w:ins ").unwrap();
    let ins_end = ins + body[ins..].find("</w:ins>").unwrap();
    assert!(!body[ins..ins_end].contains("commentReference"), "{body}");
}
