// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `list`: paragraphs become a bulleted, numbered or lettered list in the
//! clean copy; the redline records each paragraph's old properties.

mod common;

use common::docx::{Part, docx, docx_with, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeKind, list_changes};
use jubarte::debug::{Check, Options, report};
use jubarte::document_comparer::reject_revisions;
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;

const NUMBERING_CT: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml";
const NUMBERING_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering";

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"A","date":"2026-10-02T12:00:00Z","operations":{operations}}}"#
    ))
    .unwrap()
}

fn numbering(bytes: &[u8]) -> String {
    let options = Options {
        checks: vec![Check::Numbering],
        limit: 100,
        ..Options::default()
    };
    report(bytes, None, &options).unwrap()
}

fn three() -> Vec<u8> {
    docx(&(para("Intro.") + &para("Apples.") + &para("Pears.") + &para("Outro.")))
}

#[test]
fn paragraphs_become_a_tracked_bulleted_list() {
    let source = three();
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"list","paragraphs":["body:p:1","body:p:2"]}]"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let clean = paragraphs(&out.clean).unwrap();
    let flags: Vec<_> = clean
        .iter()
        .map(|p| (p.numbered, p.style.as_deref()))
        .collect();
    assert_eq!(
        flags,
        [
            (false, None),
            (true, Some("ListParagraph")),
            (true, Some("ListParagraph")),
            (false, None),
        ]
    );
    let listed = numbering(&out.clean);
    assert!(listed.contains("lvl 0 start=1 bullet"), "{listed}");
    let styles = part_string(&out.clean, "word/styles.xml").expect("styles part");
    assert!(styles.contains(r#"w:styleId="ListParagraph""#), "{styles}");
    // The redline numbers the same paragraphs and keeps the old properties.
    assert!(numbering(&out.redline).contains("bullet"));
    let formatting = list_changes(&out.redline)
        .unwrap()
        .into_iter()
        .filter(|c| c.kind == ChangeKind::Formatting && c.id.starts_with("body:"))
        .map(|c| c.text)
        .collect::<Vec<_>>();
    // The comparer also tracks the ListParagraph definition it finds only
    // in the clean copy (styles:rev:N); the body has one change per item.
    assert_eq!(
        formatting,
        ["Apples.", "Pears."],
        "one paragraph property change per item"
    );
    let rejected = paragraphs(&reject_revisions(&out.redline).unwrap()).unwrap();
    assert!(rejected.iter().all(|p| !p.numbered));
    let op = &out.report.operations[0];
    assert_eq!(op.kind, "list");
    assert_eq!(op.paragraph.as_deref(), Some("body:p:1, body:p:2"));
    assert_eq!(op.matches, 2);
}

#[test]
fn decimal_and_lettered_lists_at_a_level() {
    let source = three();
    let out = apply_plan(
        &source,
        &plan(
            r#"[{"kind":"list","paragraphs":["body:p:1"],"kind_of_list":"decimal"},
                {"kind":"list","paragraphs":[{"starts_with":"Pears"}],"kind_of_list":"lower_letter","level":1}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let listed = numbering(&out.clean);
    assert!(
        listed.contains(r#"lvl 0 start=1 decimal "%1.""#),
        "{listed}"
    );
    assert!(
        listed.contains(r#"lvl 1 start=1 lowerLetter "%2.""#),
        "{listed}"
    );
    let xml = part_string(&out.clean, "word/document.xml").unwrap();
    assert!(xml.contains(r#"<w:ilvl w:val="1""#), "{xml}");
    // Two lists, two numbering instances.
    let num_ids: std::collections::BTreeSet<&str> = xml
        .match_indices(r#"<w:numId w:val=""#)
        .map(|(at, m)| {
            let rest = &xml[at + m.len()..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect();
    assert_eq!(num_ids.len(), 2, "{xml}");
}

#[test]
fn an_existing_numbering_part_keeps_its_lists_and_ids() {
    let existing = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="4"><w:multiLevelType w:val="hybridMultilevel"/><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="upperRoman"/><w:lvlText w:val="%1."/><w:lvlJc w:val="left"/></w:lvl></w:abstractNum><w:num w:numId="7"><w:abstractNumId w:val="4"/></w:num><w:numIdMacAtCleanup w:val="7"/></w:numbering>"#;
    let body = para("Intro.")
        + r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="7"/></w:numPr></w:pPr><w:r><w:t>Old item.</w:t></w:r></w:p>"#
        + &para("New item.")
        + &para("Outro.");
    let source = docx_with(
        &body,
        &[Part {
            name: "word/numbering.xml",
            content_type: NUMBERING_CT,
            rel_type: NUMBERING_REL,
            xml: existing,
        }],
    );
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"list","paragraphs":["body:p:2"],"kind_of_list":"decimal"}]"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let listed = numbering(&out.clean);
    assert!(
        listed.contains("num 7 lvl 0 start=1 upperRoman"),
        "{listed}"
    );
    assert!(listed.contains("num 8 lvl 0 start=1 decimal"), "{listed}");
    let part = part_string(&out.clean, "word/numbering.xml").unwrap();
    assert!(part.contains(r#"w:abstractNumId="5""#), "{part}");
    // Every abstractNum precedes every num.
    let last_abstract = part.rfind("<w:abstractNum ").unwrap();
    let first_num = part.find("<w:num ").unwrap();
    assert!(last_abstract < first_num, "{part}");
    let last_num = part.rfind("<w:num ").unwrap();
    assert!(
        last_num < part.find("<w:numIdMacAtCleanup").unwrap(),
        "{part}"
    );
}

#[test]
fn restart_false_continues_the_list_before() {
    let existing = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:numbering xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:abstractNum w:abstractNumId="0"><w:lvl w:ilvl="0"><w:start w:val="1"/><w:numFmt w:val="decimal"/><w:lvlText w:val="%1."/><w:lvlJc w:val="left"/></w:lvl></w:abstractNum><w:num w:numId="3"><w:abstractNumId w:val="0"/></w:num></w:numbering>"#;
    let body = r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="3"/></w:numPr></w:pPr><w:r><w:t>First.</w:t></w:r></w:p>"#
        .to_string()
        + &para("Between.")
        + &para("Second.");
    let source = docx_with(
        &body,
        &[Part {
            name: "word/numbering.xml",
            content_type: NUMBERING_CT,
            rel_type: NUMBERING_REL,
            xml: existing,
        }],
    );
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"list","paragraphs":["body:p:2"],"restart":false}]"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    let xml = part_string(&out.clean, "word/document.xml").unwrap();
    assert_eq!(xml.matches(r#"<w:numId w:val="3""#).count(), 2, "{xml}");
    let part = part_string(&out.clean, "word/numbering.xml").unwrap();
    assert_eq!(part.matches("<w:num ").count(), 1, "no new list: {part}");
    assert!(
        out.report.operations[0]
            .context
            .as_deref()
            .unwrap()
            .contains("continues")
    );
}

#[test]
fn restart_false_without_a_list_before_is_refused() {
    let (code, _) = refusal(
        &three(),
        r#"[{"kind":"list","paragraphs":["body:p:1"],"restart":false}]"#,
    );
    assert_eq!(code, "UNSUPPORTED_STRUCTURE");
}

#[test]
fn a_styled_paragraph_keeps_its_style() {
    let body =
        r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title.</w:t></w:r></w:p>"#
            .to_string()
            + &para("Body.");
    let source = docx(&body);
    let out = apply_plan(
        &source,
        &plan(r#"[{"kind":"list","paragraphs":["body:p:0"],"kind_of_list":"decimal"}]"#),
    )
    .unwrap();
    assert_word_valid_package(&out.clean);
    let first = &paragraphs(&out.clean).unwrap()[0];
    assert!(first.numbered);
    assert_eq!(first.style.as_deref(), Some("Heading1"));
    // No paragraph took ListParagraph, so the style is not added.
    let styles = part_string(&out.clean, "word/styles.xml").unwrap_or_default();
    assert!(!styles.contains("ListParagraph"), "{styles}");
}

fn refusal(source: &[u8], operations: &str) -> (String, String) {
    let error = apply_plan(source, &plan(operations)).unwrap_err();
    (error.code, error.message)
}

#[test]
fn bad_lists_are_refused() {
    let source = three();
    for (ops, code) in [
        (r#"[{"kind":"list","paragraphs":[]}]"#, "INVALID_EDIT"),
        (
            r#"[{"kind":"list","paragraphs":["body:p:1"],"level":9}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"list","paragraphs":["body:p:1",{"index":1}]}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"list","paragraphs":["body:p:9"]}]"#,
            "ANCHOR_NOT_FOUND",
        ),
        (
            r#"[{"kind":"list","paragraphs":["body:p:1"]},{"kind":"delete_paragraph","paragraph":"body:p:1"}]"#,
            "OVERLAPPING_EDITS",
        ),
        (
            r#"[{"kind":"list","paragraphs":["body:p:1"]},{"kind":"list","paragraphs":["body:p:1"]}]"#,
            "OVERLAPPING_EDITS",
        ),
        (
            r#"[{"kind":"list","paragraphs":["body:p:1"]},{"kind":"format_paragraph","paragraph":"body:p:1","alignment":"center"}]"#,
            "OVERLAPPING_EDITS",
        ),
        (
            r#"[{"kind":"list","paragraphs":["body:p:1"]},{"kind":"merge_paragraphs","paragraph":"body:p:1"}]"#,
            "OVERLAPPING_EDITS",
        ),
    ] {
        let (got, message) = refusal(&source, ops);
        assert_eq!(got, code, "{ops}: {message}");
    }
}

#[test]
fn list_keys_are_checked() {
    for ops in [
        r#"[{"kind":"list","paragraph":"body:p:1","paragraphs":["body:p:1"]}]"#,
        r#"[{"kind":"list","paragraphs":["body:p:1"],"style":"decimal"}]"#,
        r#"[{"kind":"list","paragraphs":["body:p:1"],"kind_of_list":"roman"}]"#,
    ] {
        let error = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"A","operations":{ops}}}"#
        ))
        .unwrap_err();
        assert_eq!(error.code, "INVALID_PLAN", "{ops}");
    }
}
