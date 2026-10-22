// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Edits preserve boundary bookmarks through tracking and revision resolution,
//! and a new insertion never takes in a range marker it did not create
//! (#1156): opening markers stay with the words to their right, closing
//! markers with the words to their left, body-level markers at body level,
//! and the source position wins, so no range grows over the new text.

mod common;

use common::docx::{Part, docx, docx_with, para, part_string, run};
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

fn assert_no_bookmark_findings(outputs: &[(&str, &[u8])]) {
    let mut diagnostics = Vec::new();
    for (stage, bytes) in outputs {
        let findings: Vec<_> = validate(bytes)
            .unwrap()
            .into_iter()
            .filter(|f| f.code.starts_with("BOOKMARK_"))
            .collect();
        if !findings.is_empty() {
            diagnostics.push(format!(
                "{stage}: {findings:#?}\n{}",
                part_string(bytes, "word/document.xml").unwrap()
            ));
        }
    }
    assert!(diagnostics.is_empty(), "{}", diagnostics.join("\n"));
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
    assert_no_bookmark_findings(&[("output", output)]);
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
        assert_no_bookmark_findings(&[("accepted", &accepted), ("rejected", &rejected)]);
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

fn check_boundary_edit(trailing_start: bool, operation: &str, expected_texts: &[&str]) {
    let body = if trailing_start {
        format!(
            "<w:p><w:r><w:t>foo</w:t></w:r>{START}</w:p><w:p><w:r><w:t>bar</w:t></w:r>{END}</w:p>"
        )
    } else {
        format!(
            "<w:p>{START}<w:r><w:t>foo</w:t></w:r></w:p><w:p>{END}<w:r><w:t>bar</w:t></w:r></w:p>"
        )
    };
    let source = docx(&(body + "<w:p><w:r><w:t>final</w:t></w:r></w:p>"));
    assert_no_bookmark_findings(&[("source", &source)]);
    let plan = EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Editor","date":"2026-10-17T12:00:00Z","existing_revisions":"refuse","operations":[{operation}]}}"#
    )).unwrap();
    let result = apply_plan(&source, &plan).unwrap();
    let accepted = accept_revisions(&result.redline).unwrap();
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_no_bookmark_findings(&[
        ("redline", &result.redline),
        ("accepted", &accepted),
        ("rejected", &rejected),
    ]);
    // Reopen each serialized package and check the named, matched pair even
    // when deleting a paragraph removes one endpoint from the clean copy.
    endpoints(&result.redline);
    endpoints(&accepted);
    endpoints(&rejected);
    assert_eq!(texts(&accepted), expected_texts);
    assert_eq!(texts(&accepted), texts(&result.clean));
    assert_eq!(texts(&rejected), texts(&source));
}

#[test]
fn trailing_bookmark_start_survives_run_insert() {
    check_boundary_edit(
        true,
        r#"{"kind":"insert","paragraph":"p0","after":"foo","text":"X"}"#,
        &["fooX", "bar", "final"],
    );
}

#[test]
fn trailing_bookmark_start_survives_paragraph_deletion() {
    check_boundary_edit(
        true,
        r#"{"kind":"delete_paragraph","paragraph":"p0"}"#,
        &["bar", "final"],
    );
}

#[test]
fn leading_bookmark_end_survives_run_insert() {
    check_boundary_edit(
        false,
        r#"{"kind":"insert","paragraph":"p1","before":"bar","text":"X"}"#,
        &["foo", "Xbar", "final"],
    );
}

#[test]
fn leading_bookmark_end_survives_run_replace() {
    check_boundary_edit(
        false,
        r#"{"kind":"replace","paragraph":"p1","find":"bar","replacement":"baz"}"#,
        &["foo", "baz", "final"],
    );
}

#[test]
fn leading_bookmark_end_survives_paragraph_deletion() {
    check_boundary_edit(
        false,
        r#"{"kind":"delete_paragraph","paragraph":"p1"}"#,
        &["foo", "final"],
    );
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

const EDGE_TEXT: &str = "Clause one.";
const EDGE_ADDED: &str = " Added.";

fn edge_source(shape: &str) -> Vec<u8> {
    let r = run(EDGE_TEXT, false, false, None);
    let body = match shape {
        "body_end" | "comment" => format!("<w:p>{START}{r}</w:p>{END}"),
        "para_end" | "para_start" | "after" | "cli" | "before" => {
            format!("<w:p>{START}{r}{END}</w:p>")
        }
        "span_end" => format!("<w:p>{START}{r}</w:p><w:p>{r}</w:p>{END}"),
        "body_start" => format!("{START}<w:p>{r}</w:p>{END}"),
        _ => unreachable!(),
    };
    docx(&(body + &para("Next.")))
}

/// Parent ownership and source-text coordinates, independently read from the
/// serialized package. Body endpoints also record which paragraph gap owns
/// them. New words are removed only when comparing source coordinates below.
fn edge_endpoints(bytes: &[u8], family: &str) -> Vec<(String, String, String)> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let d = dom.parse_xdocument(&xml);
    let root = dom.root(d).unwrap();
    let body = dom.element(root, &W::body()).unwrap();
    let ps = dom.descendants(body, Some(&W::p()));
    let names = match family {
        "bookmark" => ["bookmarkStart", "bookmarkEnd"],
        "comment" => ["commentRangeStart", "commentRangeEnd"],
        "permission" => ["permStart", "permEnd"],
        _ => unreachable!(),
    };
    let mut ids = Vec::new();
    let locations = names
        .into_iter()
        .map(|name| {
            let markers = dom.descendants(body, Some(&W::name(name)));
            assert_eq!(markers.len(), 1, "{name}: {xml}");
            let marker = markers[0];
            ids.push(dom.attribute(marker, &W::id()).unwrap().to_owned());
            assert!(
                dom.ancestors(marker, Some(&W::ins())).is_empty(),
                "pre-existing {name} swallowed: {xml}"
            );
            let parent = dom.parent(marker).unwrap();
            let owner = if parent == body {
                let gap = dom
                    .elements(body, None)
                    .into_iter()
                    .take_while(|&n| n != marker)
                    .filter(|n| ps.contains(n))
                    .count();
                format!("body:{gap}")
            } else {
                assert!(dom.name_is(parent, &W::p()), "{name} parent changed: {xml}");
                format!("p{}", ps.iter().position(|&p| p == parent).unwrap())
            };
            let mut before = String::new();
            let mut after = String::new();
            let mut seen = false;
            for n in dom.descendant_nodes(parent) {
                if n == marker {
                    seen = true;
                } else if dom.name_is(n, &W::t()) {
                    if seen {
                        after.push_str(&dom.value(n));
                    } else {
                        before.push_str(&dom.value(n));
                    }
                }
            }
            assert!(seen);
            (owner, before, after)
        })
        .collect();
    assert_eq!(ids[0], ids[1], "range pair changed: {xml}");
    locations
}

fn check_edge_output(
    source: &[u8],
    output: &[u8],
    family: &str,
    p: usize,
    start: bool,
    tracked: bool,
) {
    let expected = edge_endpoints(source, family);
    let check_locations = |bytes: &[u8], inserted: bool| {
        let actual = edge_endpoints(bytes, family);
        for (i, ((owner, before, after), old)) in actual.iter().zip(&expected).enumerate() {
            assert_eq!(
                (
                    owner.clone(),
                    before.replace(EDGE_ADDED, ""),
                    after.replace(EDGE_ADDED, "")
                ),
                *old,
                "source endpoint ownership changed"
            );
            if inserted && *owner == format!("p{p}") {
                if start && i == 0 && old.1.is_empty() {
                    assert_eq!(
                        before, EDGE_ADDED,
                        "opening marker must follow the new words"
                    );
                }
                if !start && i == 1 && old.2.is_empty() {
                    assert_eq!(
                        after, EDGE_ADDED,
                        "closing marker must precede the new words"
                    );
                }
            }
        }
    };
    let mut wanted = texts(source);
    wanted[p] = if start {
        format!("{EDGE_ADDED}{}", wanted[p])
    } else {
        format!("{}{EDGE_ADDED}", wanted[p])
    };
    assert_eq!(texts(output), wanted);
    assert_no_bookmark_findings(&[("output", output)]);
    check_locations(output, true);
    assert_word_valid_package(output);
    let xml = part_string(output, "word/document.xml").unwrap();
    assert_eq!(xml.contains("<w:ins "), tracked, "{xml}");
    let accepted = accept_revisions(output).unwrap();
    let rejected = reject_revisions(output).unwrap();
    assert_no_bookmark_findings(&[("accepted", &accepted), ("rejected", &rejected)]);
    check_locations(&accepted, true);
    check_locations(&rejected, !tracked);
    assert_eq!(texts(&accepted), wanted);
    assert_eq!(
        texts(&rejected),
        if tracked { texts(source) } else { wanted }
    );
}

fn save_edge_output(shape: &str, source: &[u8], tracked: &[u8]) {
    if let Some(path) = std::env::var_os("JUBARTE_EDGE_OUTPUT_DIR") {
        let path = std::path::PathBuf::from(path);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join(format!("{shape}.docx")), tracked).unwrap();
        std::fs::write(path.join(format!("{shape}-source.docx")), source).unwrap();
        std::fs::write(path.join(format!("{shape}.txt")), "opens with no prompt; Reject All gives the source text with the bookmark where the source had it\n").unwrap();
    }
}

fn check_run_edge(shape: &str, policy: &str) {
    let source = edge_source(shape);
    let p = usize::from(shape == "span_end");
    let start = matches!(shape, "body_start" | "para_start" | "before");
    let location = match shape {
        "after" => format!(r#""after":"{EDGE_TEXT}""#),
        "before" => format!(r#""before":"{EDGE_TEXT}""#),
        _ => format!(r#""position":"{}""#, if start { "start" } else { "end" }),
    };
    let comment = if shape == "comment" {
        r#", "comment":"Drafts for review.""#
    } else {
        ""
    };
    let plan = EditPlan::from_json(&format!(r#"{{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","existing_revisions":"{policy}","operations":[{{"kind":"insert","paragraph":"p{p}",{location},"text":"{EDGE_ADDED}"{comment}}}]}}"#)).unwrap();
    let result = apply_plan(&source, &plan);
    assert!(result.is_ok(), "edge insert refused: {result:?}");
    let result = result.unwrap();
    for (output, tracked) in [(&result.redline, true), (&result.clean, false)] {
        check_edge_output(&source, output, "bookmark", p, start, tracked);
    }
    if shape == "comment" {
        let comments = jubarte::comments::list_comments(&result.redline).unwrap();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].anchor_text, EDGE_ADDED);
        let xml = part_string(&result.redline, "word/document.xml").unwrap();
        let mut dom = Dom::new();
        let d = dom.parse_xdocument(&xml);
        let root = dom.root(d).unwrap();
        for name in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
            let markers = dom.descendants(root, Some(&W::name(name)));
            assert_eq!(markers.len(), 1, "{xml}");
            assert_eq!(
                dom.ancestors(markers[0], Some(&W::ins())).len(),
                1,
                "new comment belongs to new insertion: {xml}"
            );
        }
    }
    if policy == "refuse" {
        save_edge_output(shape, &source, &result.redline);
    }
}

#[test]
fn run_edge_body_end() {
    check_run_edge("body_end", "refuse");
}
#[test]
fn run_edge_para_end() {
    check_run_edge("para_end", "refuse");
}
#[test]
fn run_edge_span_end() {
    check_run_edge("span_end", "refuse");
}
#[test]
fn run_edge_body_start() {
    check_run_edge("body_start", "refuse");
}
#[test]
fn run_edge_para_start() {
    check_run_edge("para_start", "refuse");
}
#[test]
fn run_edge_after() {
    check_run_edge("after", "refuse");
}
#[test]
fn run_edge_before() {
    check_run_edge("before", "refuse");
}
#[test]
fn run_edge_comment() {
    check_run_edge("comment", "refuse");
}
#[test]
fn run_edge_keep() {
    for shape in [
        "body_end",
        "para_end",
        "span_end",
        "body_start",
        "para_start",
        "after",
        "before",
        "comment",
    ] {
        check_run_edge(shape, "keep");
    }
}

#[test]
fn run_edge_cli() {
    let source = edge_source("cli");
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("source.docx");
    let output = dir.path().join("tracked.docx");
    std::fs::write(&input, &source).unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("edit")
        .arg(&input)
        .args(["-p", "p0", "--anchor", EDGE_TEXT, "--content"])
        .arg(format!("{EDGE_TEXT}{EDGE_ADDED}"))
        .args(["--author", "Alice", "-o"])
        .arg(&output)
        .args(["--force", "-q"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let tracked = std::fs::read(output).unwrap();
    check_edge_output(&source, &tracked, "bookmark", 0, false, true);
    let clean = accept_revisions(&tracked).unwrap();
    check_edge_output(&source, &clean, "bookmark", 0, false, false);
    save_edge_output("cli", &source, &tracked);
}

fn check_existing_edge_range(family: &str, start: bool, policy: &str) {
    let (open, close) = match family {
        "comment" => (
            r#"<w:commentRangeStart w:id="7"/>"#,
            r#"<w:commentRangeEnd w:id="7"/><w:r><w:commentReference w:id="7"/></w:r>"#,
        ),
        "permission" => (
            r#"<w:permStart w:id="7" w:edGrp="everyone"/>"#,
            r#"<w:permEnd w:id="7"/>"#,
        ),
        _ => unreachable!(),
    };
    let body = format!(
        "<w:p>{open}{}{close}</w:p>{}",
        run(EDGE_TEXT, false, false, None),
        para("Next.")
    );
    let source = if family == "comment" {
        common::docx::docx_with(
            &body,
            &[common::docx::Part {
                name: "word/comments.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: r#"<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="7" w:author="Carol Diaz" w:date="2026-10-18T12:00:00Z"><w:p><w:r><w:t>Original comment.</w:t></w:r></w:p></w:comment></w:comments>"#,
            }],
        )
    } else {
        docx(&body)
    };
    let position = if start { "start" } else { "end" };
    let plan = EditPlan::from_json(&format!(r#"{{"schema_version":1,"author":"Alice","date":"2026-10-18T12:00:00Z","existing_revisions":"{policy}","operations":[{{"kind":"insert","paragraph":"p0","position":"{position}","text":"{EDGE_ADDED}"}}]}}"#)).unwrap();
    let result = apply_plan(&source, &plan);
    assert!(result.is_ok(), "edge insert refused: {result:?}");
    let result = result.unwrap();
    for (output, tracked) in [(&result.redline, true), (&result.clean, false)] {
        check_edge_output(&source, output, family, 0, start, tracked);
        if family == "comment" {
            let comments = jubarte::comments::list_comments(output).unwrap();
            assert_eq!(comments.len(), 1);
            assert_eq!(comments[0].anchor_text, EDGE_TEXT);
            let xml = part_string(output, "word/document.xml").unwrap();
            let mut dom = Dom::new();
            let d = dom.parse_xdocument(&xml);
            let root = dom.root(d).unwrap();
            let references = dom.descendants(root, Some(&W::name("commentReference")));
            assert_eq!(references.len(), 1, "{xml}");
            assert!(
                dom.ancestors(references[0], Some(&W::ins())).is_empty(),
                "pre-existing comment reference swallowed: {xml}"
            );
        }
    }
}

#[test]
fn run_edge_existing_comment_end() {
    check_existing_edge_range("comment", false, "refuse");
}
#[test]
fn run_edge_existing_comment_start() {
    check_existing_edge_range("comment", true, "refuse");
}
#[test]
fn run_edge_existing_permission_end() {
    check_existing_edge_range("permission", false, "keep");
}
#[test]
fn run_edge_existing_permission_start() {
    check_existing_edge_range("permission", true, "keep");
}
#[test]
fn run_edge_existing_ranges_keep() {
    for family in ["comment", "permission"] {
        for start in [false, true] {
            check_existing_edge_range(family, start, "keep");
        }
    }
}

/// The range markers, by name, that an insertion must leave beside it.
const MARKERS: [&str; 6] = [
    "bookmarkStart",
    "bookmarkEnd",
    "commentRangeStart",
    "commentRangeEnd",
    "permStart",
    "permEnd",
];
const EDITOR: &str = "Editor";
/// Under `refuse` (the default for a source without revisions) the redline
/// used to come from the comparer; under `keep` it is emitted directly.
const POLICIES: [&str; 2] = ["keep", "refuse"];

fn body_dom(bytes: &[u8]) -> (Dom, NodeId) {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let body = dom.descendants(root, Some(&W::body()))[0];
    (dom, body)
}

fn is_marker(dom: &Dom, node: NodeId) -> Option<String> {
    let name = dom.name(node)?;
    (name.namespace_name() == W::URI && MARKERS.contains(&name.local_name()))
        .then(|| name.local_name().to_string())
}

fn in_editors_insertion(dom: &Dom, node: NodeId) -> bool {
    in_editors(dom, node, &W::ins())
}

fn in_editors(dom: &Dom, node: NodeId, revision: &jubarte::xmllinq::XName) -> bool {
    dom.ancestors(node, Some(revision))
        .iter()
        .any(|&r| dom.attribute(r, &W::author()) == Some(EDITOR))
}

/// Whether `node` is text of the source: a `w:t` the editor did not insert,
/// or a `w:delText` the editor deleted.
fn source_text(dom: &Dom, node: NodeId) -> bool {
    (dom.name_is(node, &W::t()) && !in_editors_insertion(dom, node))
        || (dom.name_is(node, &W::name("delText")) && in_editors(dom, node, &W::del()))
}

/// Each range marker of the body in document order: its name, the index of
/// its paragraph (`None` at body level), and how many characters of the
/// source's text come before it there: the editor's inserted words are not
/// the source's, the words it deleted are. Ids are left out: Reject All
/// renumbers bookmarks.
fn marker_places(bytes: &[u8]) -> Vec<(String, Option<usize>, usize)> {
    let (dom, body) = body_dom(bytes);
    let paragraphs = dom.descendants(body, Some(&W::p()));
    let mut out = Vec::new();
    for node in dom.descendant_nodes(body) {
        let Some(name) = is_marker(&dom, node) else {
            continue;
        };
        let Some(&owner) = dom.ancestors(node, Some(&W::p())).first() else {
            out.push((name, None, 0));
            continue;
        };
        let mut before = 0;
        for n in dom.descendant_nodes(owner) {
            if n == node {
                break;
            }
            if source_text(&dom, n) {
                before += dom.value(n).chars().count();
            }
        }
        let index = paragraphs.iter().position(|&p| p == owner);
        out.push((name, index, before));
    }
    out
}

/// The markers the editor's insertions hold.
fn swallowed(bytes: &[u8]) -> Vec<String> {
    let (dom, body) = body_dom(bytes);
    dom.descendant_nodes(body)
        .into_iter()
        .filter(|&n| is_marker(&dom, n).is_some() && in_editors_insertion(&dom, n))
        .filter_map(|n| is_marker(&dom, n))
        .collect()
}

/// The text each range of the body covers, in the order the ranges open:
/// the range's kind and the `w:t` text between its start and its end. A
/// range that does not close is left out.
fn covered(bytes: &[u8]) -> Vec<(String, String)> {
    let (dom, body) = body_dom(bytes);
    let mut ranges: Vec<(String, String, String, bool)> = Vec::new();
    for node in dom.descendant_nodes(body) {
        if dom.name_is(node, &W::t()) {
            let text = dom.value(node);
            for range in ranges.iter_mut().filter(|r| !r.3) {
                range.2.push_str(&text);
            }
            continue;
        }
        let Some(name) = is_marker(&dom, node) else {
            continue;
        };
        let id = dom
            .attribute(node, &W::id())
            .unwrap_or_default()
            .to_string();
        if let Some(kind) = name.strip_suffix("Start") {
            ranges.push((kind.to_string(), id, String::new(), false));
        } else if let Some(kind) = name.strip_suffix("End")
            && let Some(range) = ranges.iter_mut().find(|r| !r.3 && r.0 == kind && r.1 == id)
        {
            range.3 = true;
        }
    }
    ranges
        .into_iter()
        .filter(|r| r.3)
        .map(|(kind, _, text, _)| (kind, text))
        .collect()
}

/// A comment part for the one comment (`w:id` 0) the comment rows anchor.
fn with_comment(body: &str) -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{}"><w:comment w:id="0" w:author="Ann" w:date="2026-10-01T09:00:00Z" w:initials="A"><w:p><w:r><w:t>Check this.</w:t></w:r></w:p></w:comment></w:comments>"#,
        common::docx::W_NS
    );
    docx_with(
        body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    )
}

/// `operation` on `source`, under both revision policies: the redline's new
/// insertion holds no marker, every marker keeps its place in the source's
/// text, and Accept All and Reject All give the clean copy and the source
/// with the markers where the source had them. No range grows over the new
/// text: each covers the source's words in the clean copy and after Accept
/// All alike.
fn check_beside(source: &[u8], operation: &str, accepted_texts: &[&str]) {
    check_beside_covering(source, operation, accepted_texts, None);
}

/// [`check_beside`] where the one range of `source` holds the paragraph the
/// words go into, so after Accept All (and in the clean copy) it covers
/// `grown`.
fn check_beside_covering(
    source: &[u8],
    operation: &str,
    accepted_texts: &[&str],
    grown: Option<&str>,
) {
    let places = marker_places(source);
    assert!(!places.is_empty());
    let ranges = covered(source);
    assert!(!ranges.is_empty());
    let accepted_ranges = match grown {
        Some(text) => vec![(ranges[0].0.clone(), text.to_string())],
        None => ranges.clone(),
    };
    for policy in POLICIES {
        let plan = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"{EDITOR}","date":"2026-10-19T12:00:00Z","existing_revisions":"{policy}","operations":[{operation}]}}"#
        ))
        .unwrap();
        let result = apply_plan(source, &plan).unwrap();
        let redline = &result.redline;
        let xml = part_string(redline, "word/document.xml").unwrap();
        assert!(xml.contains("<w:ins "), "{policy}: {xml}");
        assert_eq!(swallowed(redline), Vec::<String>::new(), "{policy}: {xml}");
        assert_eq!(marker_places(redline), places, "{policy}: {xml}");
        assert_word_valid_package(redline);
        let accepted = accept_revisions(redline).unwrap();
        let rejected = reject_revisions(redline).unwrap();
        assert_eq!(texts(&accepted), accepted_texts, "{policy}");
        assert_eq!(texts(&accepted), texts(&result.clean), "{policy}");
        assert_eq!(texts(&rejected), texts(source), "{policy}");
        assert_eq!(marker_places(&rejected), places, "{policy}");
        assert_eq!(covered(&result.clean), accepted_ranges, "{policy}: clean");
        assert_eq!(covered(&accepted), accepted_ranges, "{policy}: accepted");
        assert_eq!(covered(&rejected), ranges, "{policy}: rejected");
        assert_no_bookmark_findings(&[
            ("redline", redline),
            ("accepted", &accepted),
            ("rejected", &rejected),
        ]);
    }
}

fn insert(position: &str, anchor: &str, text: &str) -> String {
    insert_in("p0", position, anchor, text)
}

fn insert_in(paragraph: &str, position: &str, anchor: &str, text: &str) -> String {
    format!(
        r#"{{"kind":"insert","paragraph":"{paragraph}","{position}":"{anchor}","text":"{text}"}}"#
    )
}

fn bookmarked(inner: &str) -> Vec<u8> {
    docx(&format!("<w:p>{inner}</w:p>{}", para("final")))
}

fn r(text: &str) -> String {
    run(text, false, false, None)
}

#[test]
fn an_insert_at_a_paragraphs_start_stays_before_its_bookmark() {
    let source = bookmarked(&format!("{START}{}{END}", r("foo")));
    check_beside(&source, &insert("before", "foo", "X"), &["Xfoo", "final"]);
}

#[test]
fn an_insert_at_a_paragraphs_end_stays_after_its_bookmark_end() {
    let source = bookmarked(&format!("{START}{}{END}", r("foo")));
    check_beside(&source, &insert("after", "foo", "X"), &["fooX", "final"]);
}

#[test]
fn an_insert_beside_a_bookmark_start_stays_outside_the_bookmark() {
    let source = bookmarked(&format!("{}{START}{}{END}", r("a "), r("b")));
    check_beside(&source, &insert("after", "a ", "X"), &["a Xb", "final"]);
    check_beside(&source, &insert("before", "b", "X"), &["a Xb", "final"]);
}

#[test]
fn an_insert_beside_a_bookmark_end_stays_outside_the_bookmark() {
    let source = bookmarked(&format!("{START}{}{END}{}", r("a"), r(" b")));
    check_beside(&source, &insert("after", "a", "X"), &["aX b", "final"]);
    check_beside(&source, &insert("before", " b", "X"), &["aX b", "final"]);
}

/// A bookmark at body level holds whole paragraphs: words typed into one
/// are in it, and its markers stay between the paragraphs.
#[test]
fn body_level_markers_stay_at_body_level() {
    let source = docx(&format!(
        "{START}<w:p>{}</w:p>{END}{}",
        r("foo"),
        para("final")
    ));
    let before = insert("before", "foo", "X");
    check_beside_covering(&source, &before, &["Xfoo", "final"], Some("Xfoo"));
    let after = insert("after", "foo", "X");
    check_beside_covering(&source, &after, &["fooX", "final"], Some("fooX"));
}

#[test]
fn an_insert_beside_a_comment_range_stays_outside_it() {
    let source = with_comment(&format!(
        r#"<w:p><w:commentRangeStart w:id="0"/>{}<w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>{}"#,
        r("foo"),
        para("final")
    ));
    check_beside(&source, &insert("before", "foo", "X"), &["Xfoo", "final"]);
    check_beside(&source, &insert("after", "foo", "X"), &["fooX", "final"]);
}

#[test]
fn an_insert_beside_a_permission_range_stays_outside_it() {
    let source = bookmarked(&format!(
        r#"<w:permStart w:id="3" w:edGrp="everyone"/>{}<w:permEnd w:id="3"/>"#,
        r("foo")
    ));
    check_beside(&source, &insert("before", "foo", "X"), &["Xfoo", "final"]);
    check_beside(&source, &insert("after", "foo", "X"), &["fooX", "final"]);
}

#[test]
fn an_insert_after_a_paragraphs_last_words_stays_before_a_bookmark_opening_there() {
    let source = docx(&format!(
        "<w:p>{}{START}</w:p><w:p>{}{END}</w:p>{}",
        r("foo"),
        r("bar"),
        para("final")
    ));
    check_beside(
        &source,
        &insert("after", "foo", "X"),
        &["fooX", "bar", "final"],
    );
}

#[test]
fn an_insert_before_a_paragraphs_first_words_stays_after_a_bookmark_closing_there() {
    let source = docx(&format!(
        "<w:p>{START}{}</w:p><w:p>{END}{}</w:p>{}",
        r("foo"),
        r("bar"),
        para("final")
    ));
    check_beside(
        &source,
        &insert_in("p1", "before", "bar", "X"),
        &["foo", "Xbar", "final"],
    );
}

#[test]
fn a_word_inserted_beside_a_bookmark_stays_outside_it() {
    let source = bookmarked(&format!("{START}{}{END}", r("foo bar")));
    check_beside(
        &source,
        &insert("before", "foo", "New "),
        &["New foo bar", "final"],
    );
    check_beside(
        &source,
        &insert("after", "bar", " new"),
        &["foo bar new", "final"],
    );
}

#[test]
fn an_insert_beside_an_empty_bookmark_leaves_it_whole() {
    let source = bookmarked(&format!("{}{START}{END}{}", r("a"), r("b")));
    check_beside(&source, &insert("after", "a", "X"), &["aXb", "final"]);
    check_beside(&source, &insert("before", "b", "X"), &["aXb", "final"]);
}

#[test]
fn an_insert_between_two_ranges_joins_neither() {
    let source = bookmarked(&format!(
        r#"{START}{}{END}<w:bookmarkStart w:id="8" w:name="Next"/>{}<w:bookmarkEnd w:id="8"/>"#,
        r("foo"),
        r("bar")
    ));
    check_beside(&source, &insert("after", "foo", "X"), &["fooXbar", "final"]);
    check_beside(
        &source,
        &insert("before", "bar", "X"),
        &["fooXbar", "final"],
    );
}

/// A replacement at a range's edge replaces words the range covers: the new
/// words join the range, Reject All gives the source's range back, and no
/// marker is left inside the editor's insertion.
fn check_replace(source: &[u8], find: &str, with: &str, accepted_range: &str) {
    let ranges = covered(source);
    for policy in POLICIES {
        let plan = EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"author":"{EDITOR}","date":"2026-10-19T12:00:00Z","existing_revisions":"{policy}","operations":[{{"kind":"replace","paragraph":"p0","find":"{find}","replacement":"{with}"}}]}}"#
        ))
        .unwrap();
        let result = apply_plan(source, &plan).unwrap();
        let redline = &result.redline;
        let xml = part_string(redline, "word/document.xml").unwrap();
        assert_eq!(swallowed(redline), Vec::<String>::new(), "{policy}: {xml}");
        assert_word_valid_package(redline);
        let accepted = accept_revisions(redline).unwrap();
        let rejected = reject_revisions(redline).unwrap();
        let grown = vec![(ranges[0].0.clone(), accepted_range.to_string())];
        assert_eq!(covered(&result.clean), grown, "{policy}: clean");
        assert_eq!(covered(&accepted), grown, "{policy}: accepted {xml}");
        assert_eq!(covered(&rejected), ranges, "{policy}: rejected {xml}");
        assert_no_bookmark_findings(&[
            ("redline", redline),
            ("accepted", &accepted),
            ("rejected", &rejected),
        ]);
    }
}

#[test]
fn a_replacement_at_a_bookmarks_edge_stays_inside_it() {
    let source = bookmarked(&format!("{START}{}{END}", r("foo bar")));
    check_replace(&source, "foo", "baz", "baz bar");
    check_replace(&source, "bar", "baz", "foo baz");
    check_replace(&source, "foo bar", "baz qux", "baz qux");
}

/// Property: an insert before or after any word of a paragraph that holds a
/// bookmark and a comment range leaves every marker at its offset in the
/// source's text, and every range over the same words. A permission range
/// has its own row above: the comparer, which writes the redline of an edit
/// meeting no marker, still drops every permission range (#1160).
#[test]
fn every_insert_keeps_each_marker_at_its_source_offset() {
    let words = ["one ", "two", " three ", "four", " five"];
    let source = with_comment(&format!(
        r#"<w:p>{}{START}{}{END}{}<w:commentRangeStart w:id="0"/>{}<w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r>{}</w:p>{}"#,
        r(words[0]),
        r(words[1]),
        r(words[2]),
        r(words[3]),
        r(words[4]),
        para("final")
    ));
    let text = words.concat();
    for (index, word) in words.iter().enumerate() {
        let at = words[..index].concat().len();
        for (position, offset) in [("before", at), ("after", at + word.len())] {
            let expected = format!("{}X{}", &text[..offset], &text[offset..]);
            check_beside(&source, &insert(position, word, "X"), &[&expected, "final"]);
        }
    }
}

/// The comparer changes whole words: a marker inside a word an edit
/// touches stays where the source had it all the same. Here the words go
/// inside the bookmark, which opens two letters into "alpha".
#[test]
fn a_marker_inside_the_edited_word_stays_in_place() {
    let source = bookmarked(&format!("{}{START}{}{END}", r("al"), r("pha beta")));
    let texts = ["alphXa beta", "final"];
    let grown = Some("phXa beta");
    check_beside_covering(&source, &insert("after", "alph", "X"), &texts, grown);
    check_beside_covering(&source, &insert("before", "a beta", "X"), &texts, grown);
}

#[test]
fn a_replacement_beside_a_marker_inside_its_word_keeps_the_marker() {
    let source = bookmarked(&format!("{}{START}{}{END}", r("al"), r("pha beta")));
    check_replace(&source, "ha", "HA", "pHA beta");
}

/// A replacement in a word a comment opens inside: the comment's range
/// starts after the replaced letters, so it covers the same text in the
/// redline, the clean copy, and after Accept All or Reject All, and no
/// comment marker is left inside the editor's insertion.
#[test]
fn a_replacement_beside_a_comment_inside_its_word_keeps_the_comment() {
    let source = with_comment(&format!(
        r#"<w:p>{}<w:commentRangeStart w:id="0"/>{}<w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>{}"#,
        r("foo"),
        r("bar"),
        para("final")
    ));
    check_replace(&source, "foo", "FOO", "bar");
}

/// An insert at a paragraph's edge beside a tab (a glyph, not text) that a
/// bookmark holds: the clean copy puts the words where the redline does,
/// outside the bookmark.
#[test]
fn an_insert_at_a_paragraphs_edge_beside_a_glyph_stays_outside_its_bookmark() {
    let tab_last = bookmarked(&format!("{START}<w:r><w:t>foo</w:t><w:tab/></w:r>{END}"));
    let end = r#"{"kind":"insert","paragraph":"p0","position":"end","text":"X"}"#;
    check_beside(&tab_last, end, &["foo\tX", "final"]);
    let tab_first = bookmarked(&format!("{START}<w:r><w:tab/><w:t>foo</w:t></w:r>{END}"));
    let start = r#"{"kind":"insert","paragraph":"p0","position":"start","text":"X"}"#;
    check_beside(&tab_first, start, &["X\tfoo", "final"]);
}

/// `update_fields` refreshes only the clean copy, which an emitted redline
/// would then not accept to: an edit beside a marker, in the paragraph or
/// at body level around it, is compared under it, as before #1156. A
/// candidate the comparer cannot keep valid is refused with the route that
/// keeps the marker; one it can is Word-valid and accepts to the clean copy.
#[test]
fn update_fields_beside_a_marker_is_compared() {
    let in_paragraph = bookmarked(&format!("{START}{}{END}", r("foo bar")));
    let body_level = docx(&format!(
        "{START}<w:p>{}</w:p>{END}{}",
        r("foo bar"),
        para("final")
    ));
    let plan = EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"{EDITOR}","date":"2026-10-19T12:00:00Z","update_fields":true,"operations":[{}]}}"#,
        insert("before", "foo", "New ")
    ))
    .unwrap();
    // The comparer keeps a body-level bookmark outside the paragraph's
    // inline revisions (V2-04, #827), so the compared edit is the source's
    // (#1296 no longer reaches it).
    let result = apply_plan(&body_level, &plan).unwrap();
    assert_word_valid_package(&result.redline);
    assert_word_valid_package(&result.clean);
    assert!(swallowed(&result.redline).is_empty());
    let accepted = accept_revisions(&result.redline).unwrap();
    let rejected = reject_revisions(&result.redline).unwrap();
    // The range holds whole paragraphs: its markers stay between them in
    // every copy, and it covers the paragraph with its new words.
    for copy in [&result.redline, &result.clean, &accepted, &rejected] {
        assert_eq!(marker_places(copy), marker_places(&body_level));
    }
    for copy in [&result.clean, &accepted] {
        assert_eq!(
            covered(copy),
            [("bookmark".to_string(), "New foo bar".to_string())]
        );
    }
    assert_eq!(texts(&accepted), texts(&result.clean));
    assert_eq!(texts(&rejected), texts(&body_level));
    let result = apply_plan(&in_paragraph, &plan).unwrap();
    assert_word_valid_package(&result.redline);
    assert_word_valid_package(&result.clean);
    assert_eq!(texts(&result.clean)[0], "New foo bar");
    assert_eq!(
        texts(&accept_revisions(&result.redline).unwrap()),
        texts(&result.clean)
    );
    assert_eq!(
        texts(&reject_revisions(&result.redline).unwrap()),
        texts(&in_paragraph)
    );
}
