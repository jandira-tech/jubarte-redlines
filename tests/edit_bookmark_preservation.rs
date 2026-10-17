// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Paragraph insertion preserves the source bookmark's endpoint ownership,
//! including an end at the beginning of the following paragraph.

mod common;

use common::docx::{docx, para, part_string, run};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;
use jubarte::namespaces::W;
use jubarte::validate::validate;
use jubarte::xmllinq::{Dom, NodeId};

const START: &str = r#"<w:bookmarkStart w:id="7" w:name="Clause"/>"#;
const END: &str = r#"<w:bookmarkEnd w:id="7"/>"#;
const INSERTED: &str = "An independent inserted paragraph.";

fn source(end: &str) -> Vec<u8> {
    let head = run("The bookmarked clause.", false, false, None);
    let tail = run("The following clause.", false, false, None);
    let body = match end {
        "next_start" => format!("<w:p>{START}{head}</w:p><w:p>{END}{tail}</w:p>"),
        "paragraph_end" => format!("<w:p>{START}{head}{END}</w:p><w:p>{tail}</w:p>"),
        "next_text" => format!("<w:p>{START}{head}</w:p><w:p>{tail}{END}</w:p>"),
        "start_at_end" => format!("<w:p>{head}{START}</w:p><w:p>{tail}{END}</w:p>"),
        "start_in_text" => {
            let before = run("The bookmarked ", false, false, None);
            let after = run("clause.", false, false, None);
            format!("<w:p>{before}{START}{after}</w:p><w:p>{tail}{END}</w:p>")
        }
        _ => unreachable!(),
    };
    docx(&(body + &para("An unrelated final paragraph.")))
}

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

/// Independent endpoint oracle: owner paragraph's text, and the text before
/// and after the endpoint within it. Numeric bookmark ids may be renumbered
/// by Compare, but the name, pair and logical locations must survive.
fn endpoints(bytes: &[u8]) -> [(String, String); 2] {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let starts = dom.descendants(root, Some(&W::bookmark_start()));
    assert_eq!(starts.len(), 1, "{xml}");
    let start = starts[0];
    assert_eq!(dom.attribute(start, &W::name("name")), Some("Clause"));
    let ends = dom.descendants(root, Some(&W::bookmark_end()));
    assert_eq!(ends.len(), 1, "{xml}");
    let end = ends[0];
    assert_eq!(dom.attribute(start, &W::id()), dom.attribute(end, &W::id()));
    let location = |marker: NodeId| {
        let owner = dom.ancestors(marker, Some(&W::p()))[0];
        let mut before = String::new();
        let mut after = String::new();
        let mut seen = false;
        for node in dom.descendant_nodes(owner) {
            if node == marker {
                seen = true;
            } else if dom.name_is(node, &W::t()) {
                if seen {
                    after.push_str(&dom.value(node));
                } else {
                    before.push_str(&dom.value(node));
                }
            }
        }
        assert!(seen);
        (before, after)
    };
    [location(start), location(end)]
}

fn check_insert(end: &str, tracked: bool, policy: &str) {
    let source = source(end);
    let expected = endpoints(&source);
    assert!(
        validate(&source)
            .unwrap()
            .iter()
            .all(|f| !f.code.starts_with("BOOKMARK_"))
    );
    let plan = EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Editor","date":"2026-10-17T12:00:00Z","existing_revisions":"{policy}","operations":[{{"kind":"insert_paragraph","paragraph":"p0","position":"after","runs":[{{"text":"{INSERTED}"}}]}}]}}"#
    )).unwrap();
    let result = apply_plan(&source, &plan).unwrap();
    // Both APIs reopen the serialized ZIP; no in-memory DOM is the oracle.
    let output = if tracked {
        &result.redline
    } else {
        &result.clean
    };
    let findings = validate(output).unwrap();
    assert!(
        findings
            .iter()
            .all(|f| f.code != "BOOKMARK_CROSSES_REVISION"),
        "{findings:#?}\n{}",
        part_string(output, "word/document.xml").unwrap()
    );
    assert_eq!(
        endpoints(output),
        expected,
        "source endpoint ownership changed"
    );
    assert_word_valid_package(output);
    let mut expected_texts = texts(&source);
    expected_texts.insert(1, INSERTED.into());
    assert_eq!(texts(output), expected_texts);
    if tracked {
        let xml = part_string(output, "word/document.xml").unwrap();
        assert!(xml.contains("<w:ins "), "{xml}");
        let accepted = accept_revisions(output).unwrap();
        let rejected = reject_revisions(output).unwrap();
        assert_eq!(texts(&accepted), texts(&result.clean));
        assert_eq!(texts(&rejected), texts(&source));
        assert_eq!(endpoints(&accepted), expected);
        assert_eq!(endpoints(&rejected), expected);
    } else {
        assert!(
            !part_string(output, "word/document.xml")
                .unwrap()
                .contains("<w:ins ")
        );
    }
}

#[test]
fn insert_after_bookmarked_paragraph_preserves_endpoints() {
    check_insert("next_start", true, "refuse");
}

#[test]
fn insert_after_bookmark_ending_at_paragraph_end_preserves_endpoints() {
    check_insert("paragraph_end", true, "refuse");
}

#[test]
fn insert_inside_bookmark_spanning_next_paragraph_preserves_endpoints() {
    check_insert("next_text", true, "refuse");
}

#[test]
fn untracked_insert_after_bookmarked_paragraph_preserves_endpoints() {
    for end in ["next_start", "paragraph_end", "next_text", "start_at_end"] {
        check_insert(end, false, "refuse");
    }
}

#[test]
fn kept_revisions_insert_after_bookmarked_paragraph_preserves_endpoints() {
    for end in ["next_start", "paragraph_end", "next_text", "start_at_end"] {
        check_insert(end, true, "keep");
    }
}

#[test]
fn insert_after_bookmark_start_at_paragraph_end_preserves_endpoints() {
    check_insert("start_at_end", true, "refuse");
}

#[test]
fn insert_after_bookmark_start_inside_paragraph_text_preserves_endpoints() {
    // Text on both sides of the start: neither the leading nor the trailing
    // boundary rule applies, so the start stays between the two runs.
    check_insert("start_in_text", true, "refuse");
    check_insert("start_in_text", false, "refuse");
}

#[test]
fn cli_validate_accepts_insert_after_bookmarked_paragraph() {
    let source = source("next_start");
    let plan = EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Editor","date":"2026-10-17T12:00:00Z","operations":[{{"kind":"insert_paragraph","paragraph":"p0","position":"after","runs":[{{"text":"{INSERTED}"}}]}}]}}"#
    )).unwrap();
    let result = apply_plan(&source, &plan).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bookmarked-insertion.docx");
    std::fs::write(&path, &result.redline).unwrap();
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("validate")
        .arg(&path)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.contains("BOOKMARK_CROSSES_REVISION"),
        "{stdout}\n{stderr}"
    );
    assert!(output.status.success(), "{stdout}\n{stderr}");
}

#[test]
fn validator_still_reports_bookmark_crossing_revision() {
    let source = docx(&format!(
        r#"<w:p>{START}<w:r><w:t>Original text.</w:t></w:r><w:ins w:id="8" w:author="Editor" w:date="2026-10-17T12:00:00Z"><w:r><w:t>Inserted text.</w:t></w:r>{END}</w:ins></w:p>"#
    ));
    assert!(
        validate(&source)
            .unwrap()
            .iter()
            .any(|f| f.code == "BOOKMARK_CROSSES_REVISION")
    );
}
