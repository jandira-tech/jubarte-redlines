// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `insert_footnote`: a footnote reference right after one occurrence of an
//! anchor, and the note itself in the footnotes part, which is created with
//! Word's separator notes when the source has none.

mod common;

use common::docx::{Part, R_NS, W_NS, docx, docx_with, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeKind, list_changes};
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::{paragraphs, source_sha256, stories, summary};

fn plan(source: &[u8], operations: &str) -> EditPlan {
    let json = format!(
        r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","date":"2026-10-02T12:00:00Z","operations":{operations}}}"#,
        source_sha256(source)
    );
    EditPlan::from_json(&json).unwrap()
}

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn note_texts(bytes: &[u8]) -> Vec<String> {
    stories(bytes)
        .unwrap()
        .into_iter()
        .filter(|s| s.kind == "footnotes")
        .flat_map(|s| s.paragraphs.into_iter().map(|p| p.text))
        .collect()
}

const NOTE: &str = "See the 2024 master agreement.";

#[test]
fn insert_footnote_creates_the_notes_part_and_references_it_after_the_anchor() {
    let source = docx(&(para("Heading") + &para("The parties agree to the terms.")));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            &format!(
                r#"[{{"kind":"insert_footnote","paragraph":{{"index":1}},"after":"agree","text":"{NOTE}"}}]"#
            ),
        ),
    )
    .unwrap();
    assert_eq!(summary(&result.clean).unwrap().footnotes, 1);
    assert_eq!(note_texts(&result.clean), vec![format!(" {NOTE}")]);
    assert_eq!(
        texts(&result.clean),
        vec!["Heading", "The parties agree to the terms."]
    );
    let body = part_string(&result.clean, "word/document.xml").unwrap();
    let reference = body.find("<w:footnoteReference").expect("reference run");
    let agree = body.find("agree").unwrap();
    let rest = body.find(" to the terms.").unwrap();
    assert!(agree < reference && reference < rest, "{body}");
    let notes = part_string(&result.clean, "word/footnotes.xml").unwrap();
    for needle in [
        r#"w:type="separator""#,
        r#"w:type="continuationSeparator""#,
        "<w:footnoteRef",
    ] {
        assert!(notes.contains(needle), "{needle} in {notes}");
    }
    assert!(
        part_string(&result.clean, "[Content_Types].xml")
            .unwrap()
            .contains("footnotes+xml")
    );
    assert!(
        part_string(&result.clean, "word/_rels/document.xml.rels")
            .unwrap()
            .contains("/footnotes")
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
    // The redline inserts the reference; accepting it gives the clean copy.
    let changes = list_changes(&result.redline).unwrap();
    assert!(
        changes.iter().any(|c| c.kind == ChangeKind::Insertion),
        "{changes:?}"
    );
    let redline_body = part_string(&result.redline, "word/document.xml").unwrap();
    let ins = redline_body.find("<w:ins ").expect("insertion");
    let ins_end = ins + redline_body[ins..].find("</w:ins>").unwrap();
    assert!(
        redline_body[ins..ins_end].contains("<w:footnoteReference"),
        "{redline_body}"
    );
    assert_eq!(summary(&result.redline).unwrap().footnotes, 1);
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), texts(&result.clean));
    assert_eq!(summary(&accepted).unwrap().footnotes, 1);
    let rejected = reject_revisions(&result.redline).unwrap();
    assert!(
        !part_string(&rejected, "word/document.xml")
            .unwrap()
            .contains("<w:footnoteReference")
    );
    let op = &result.report.operations[0];
    assert_eq!(
        (op.kind.as_str(), op.status.as_str()),
        ("insert_footnote", "ok")
    );
}

fn with_notes() -> Vec<u8> {
    let notes = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:footnotes xmlns:w="{W_NS}"><w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="3"><w:p><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:footnoteRef/></w:r><w:r><w:t xml:space="preserve"> Old note.</w:t></w:r></w:p></w:footnote></w:footnotes>"#
    );
    let body = r#"<w:p><w:r><w:t>First claim.</w:t></w:r><w:r><w:rPr><w:vertAlign w:val="superscript"/></w:rPr><w:footnoteReference w:id="3"/></w:r><w:r><w:t xml:space="preserve"> Second claim and third claim.</w:t></w:r></w:p>"#;
    let rel = format!("{R_NS}/footnotes");
    docx_with(
        body,
        &[Part {
            name: "word/footnotes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.footnotes+xml",
            rel_type: &rel,
            xml: &notes,
        }],
    )
}

#[test]
fn insert_footnote_appends_to_an_existing_notes_part_after_the_highest_id() {
    let source = with_notes();
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert_footnote","paragraph":{"index":0},"after":"claim","occurrence":3,"text":"Third."},
                {"kind":"insert_footnote","paragraph":{"index":0},"after":"Second claim","text":"Second."}]"#,
        ),
    )
    .unwrap();
    assert_eq!(summary(&result.clean).unwrap().footnotes, 3);
    assert_eq!(
        note_texts(&result.clean),
        vec![" Old note.", " Third.", " Second."]
    );
    let notes = part_string(&result.clean, "word/footnotes.xml").unwrap();
    assert!(notes.contains(r#"w:id="4""#) && notes.contains(r#"w:id="5""#));
    let body = part_string(&result.clean, "word/document.xml").unwrap();
    let ids: Vec<&str> = body
        .match_indices("<w:footnoteReference w:id=\"")
        .map(|(at, m)| {
            let rest = &body[at + m.len()..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect();
    assert_eq!(ids, ["3", "5", "4"], "{body}");
    assert_eq!(
        texts(&result.clean),
        vec!["First claim. Second claim and third claim."]
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
    assert_eq!(summary(&result.redline).unwrap().footnotes, 3);
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(summary(&accepted).unwrap().footnotes, 3);
}

#[test]
fn insert_footnote_refusals() {
    let source = with_notes();
    for (ops, code) in [
        (
            r#"[{"kind":"insert_footnote","paragraph":{"index":0},"after":"claim","text":"x"}]"#,
            "AMBIGUOUS_ANCHOR",
        ),
        (
            r#"[{"kind":"insert_footnote","paragraph":{"index":0},"after":"missing","text":"x"}]"#,
            "ANCHOR_NOT_FOUND",
        ),
        (
            r#"[{"kind":"insert_footnote","paragraph":{"index":0},"after":"First","text":""}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"insert_footnote","paragraph":{"index":0},"after":"First","text":"a\u0007b"}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"insert_footnote","paragraph":"footnotes:p:0","after":"Old","text":"x"}]"#,
            "UNSUPPORTED_STRUCTURE",
        ),
        (
            r#"[{"kind":"insert_footnote","paragraph":{"index":0},"after":"First","text":"x"},
                {"kind":"replace","paragraph":{"index":0},"find":"First claim","replacement":"A claim"}]"#,
            "OVERLAPPING_EDITS",
        ),
    ] {
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert_eq!(err.code, code, "{ops}");
    }
    let json = r#"{"schema_version":1,"author":"a","operations":[{"kind":"insert_footnote","paragraph":{"index":0},"after":"x","text":"y","before":"z"}]}"#;
    assert_eq!(EditPlan::from_json(json).unwrap_err().code, "INVALID_PLAN");
}

#[test]
fn insert_footnote_beside_a_text_edit_lands_after_the_new_text() {
    let source = docx(&para("The fee is due."));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert","paragraph":{"index":0},"after":"fee","text":" (net)"},
                {"kind":"insert_footnote","paragraph":{"index":0},"after":"fee","text":"Net of tax."}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), vec!["The fee (net) is due."]);
    let body = part_string(&result.clean, "word/document.xml").unwrap();
    let net = body.find("(net)").unwrap();
    let reference = body.find("<w:footnoteReference").unwrap();
    let due = body.find(" is due.").unwrap();
    assert!(net < reference && reference < due, "{body}");
    assert_word_valid_package(&result.redline);
}
