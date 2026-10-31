// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Edits use Word's typing order; Compare keeps its own insertion-first order.
mod common;

use common::docx::{Part, R_NS, W_NS, docx, docx_with_sect, para, part_string, run};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, compare_documents, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;
use serde_json::json;
use std::path::Path;
use std::process::Command;

const OLD: &str = "Pay within 30 days.";
const NEW: &str = "Pay within 45 days.";
const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

fn command(dir: &Path, args: &[&str]) -> String {
    let out = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn revisions(bytes: &[u8], part: &str) -> Vec<(String, u64, String)> {
    let xml = part_string(bytes, part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    dom.descendants(dom.root(doc).unwrap(), None)
        .into_iter()
        .filter(|&n| dom.name_is(n, &W::del()) || dom.name_is(n, &W::ins()))
        .filter(|&n| !dom.descendants(n, Some(&W::r())).is_empty())
        .map(|n| {
            (
                dom.name(n).unwrap().local_name().to_string(),
                dom.attribute(n, &W::id()).unwrap().parse().unwrap(),
                dom.value(n),
            )
        })
        .collect()
}

fn text(bytes: &[u8], part: &str) -> String {
    let xml = part_string(bytes, part).unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    dom.descendants(dom.root(doc).unwrap(), Some(&W::t()))
        .into_iter()
        .map(|n| dom.value(n))
        .collect()
}

fn typing_order(bytes: &[u8], part: &str) {
    let revs = revisions(bytes, part);
    assert_eq!(
        revs.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        ["del", "ins"],
        "{revs:?}"
    );
    assert!(
        revs[0].1 < revs[1].1,
        "deletion id precedes insertion id: {revs:?}"
    );
    assert_eq!(revs[0].2, "30");
    assert_eq!(revs[1].2, "45");
}

fn resolved_text(source: &[u8], edited: &[u8], part: &str, accepted: &str, rejected: Option<&str>) {
    assert_eq!(text(&accept_revisions(edited).unwrap(), part), accepted);
    let expected_reject = rejected.map_or_else(|| text(source, part), String::from);
    assert_eq!(
        text(&reject_revisions(edited).unwrap(), part),
        expected_reject
    );
    assert_word_valid_package(edited);
}

fn resolved(source: &[u8], edited: &[u8], part: &str) {
    resolved_text(source, edited, part, NEW, None);
}

fn anchored(source: &[u8], at: &str) -> (Vec<u8>, String) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), source).unwrap();
    command(
        dir.path(),
        &[
            "edit",
            "source.docx",
            "-p",
            at,
            "--anchor",
            "30",
            "--content",
            "45",
            "-a",
            "Carol Diaz",
            "-d",
            "2026-10-18T10:00:00Z",
            "-o",
            "out.docx",
        ],
    );
    let bytes = std::fs::read(dir.path().join("out.docx")).unwrap();
    let view = command(dir.path(), &["out.docx", "--no-page-markers"]);
    (bytes, view)
}

fn replace(source: &[u8], whole: bool) -> Vec<u8> {
    let plan = EditPlan::from_json(&json!({"schema_version":1,"author":"Carol Diaz","date":"2026-10-18T10:00:00Z","operations":[{"kind":"replace","paragraph":"p0","find":"30","replacement":"45","whole":whole}]}).to_string()).unwrap();
    apply_plan(source, &plan).unwrap().redline
}

// A received Word Compare redline is insertion first. Editing another
// paragraph must leave its pairs, their order and their ids as they were.
#[test]
fn an_edit_keeps_an_existing_insertion_first_pair_untouched() {
    let source = docx(&format!(
        r#"{}<w:p><w:r><w:t xml:space="preserve">Rate </w:t></w:r><w:ins w:id="7" w:author="Opposing" w:date="2026-01-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins><w:del w:id="9" w:author="Opposing" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>30</w:delText></w:r></w:del></w:p>"#,
        para(OLD)
    ));
    let (out, _) = anchored(&source, "p0");
    let found = revisions(&out, "word/document.xml");
    assert_eq!(found.len(), 4, "{found:?}");
    assert_eq!(
        (found[0].0.as_str(), found[1].0.as_str()),
        ("del", "ins"),
        "{found:?}"
    );
    assert!(found[0].1 < found[1].1, "{found:?}");
    assert_eq!(
        found[2..],
        [
            ("ins".to_string(), 7, "45".to_string()),
            ("del".to_string(), 9, "30".to_string())
        ],
        "the received pair keeps its order and ids"
    );
}

#[test]
fn anchored_body_xml_uses_typing_order() {
    let source = docx(&para(OLD));
    let (out, _) = anchored(&source, "p0");
    resolved(&source, &out, "word/document.xml");
    typing_order(&out, "word/document.xml");
}

#[test]
fn anchored_body_view_is_one_substitution_with_document_ids() {
    let (out, view) = anchored(&docx(&para(OLD)), "p0");
    let revs = revisions(&out, "word/document.xml");
    assert_eq!(revs.iter().map(|r| r.1).collect::<Vec<_>>(), [1, 2]);
    assert!(
        view.contains("Pay within {~~30~>45~~}{r1+2 @CD} days."),
        "{view}"
    );
}

#[test]
fn plan_replace_uses_typing_order() {
    let source = docx(&para(OLD));
    let out = replace(&source, false);
    resolved(&source, &out, "word/document.xml");
    typing_order(&out, "word/document.xml");
    let view = jubarte::markdown::docx_to_markdown(
        &out,
        &jubarte::markdown::MarkdownOptions {
            ids: true,
            page_markers: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        view.markdown
            .contains("Pay within {~~30~>45~~}{r1+2 @CD} days."),
        "{}",
        view.markdown
    );
}

#[test]
fn whole_plan_replace_uses_typing_order() {
    let out = replace(&docx(&para(OLD)), true);
    typing_order(&out, "word/document.xml");
}

fn story(kind: &str) -> (Vec<u8>, String, String) {
    if kind == "table" {
        return (
            docx(&format!(
                "<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w=\"4000\"/></w:tblGrid><w:tr><w:tc><w:tcPr/>{}</w:tc></w:tr></w:tbl>{}",
                para(OLD),
                para("")
            )),
            "t0.row0.cell0".into(),
            "word/document.xml".into(),
        );
    }
    let (root, content, refs, at) = match kind {
        "header" => (
            "hdr",
            para(OLD),
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
            "header1",
        ),
        "footer" => (
            "ftr",
            para(OLD),
            r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
            "footer1",
        ),
        "footnotes" => (
            "footnotes",
            format!(
                r#"<w:footnote w:type="separator" w:id="-1"><w:p><w:r><w:separator/></w:r></w:p></w:footnote><w:footnote w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:footnote><w:footnote w:id="1">{}</w:footnote>"#,
                para(OLD)
            ),
            "",
            "footnotes.p0",
        ),
        _ => unreachable!(),
    };
    let part = if kind == "header" {
        "word/header1.xml"
    } else if kind == "footer" {
        "word/footer1.xml"
    } else {
        "word/footnotes.xml"
    };
    let ct = format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml");
    let rel = format!("{R_NS}/{kind}");
    let xml = format!(r#"<w:{root} xmlns:w="{W_NS}">{content}</w:{root}>"#);
    let body = if kind == "footnotes" {
        r#"<w:p><w:r><w:t>Terms</w:t></w:r><w:r><w:footnoteReference w:id="1"/></w:r></w:p>"#
            .to_string()
    } else {
        para("Terms")
    };
    (
        docx_with_sect(
            &body,
            &[Part {
                name: part,
                content_type: &ct,
                rel_type: &rel,
                xml: &xml,
            }],
            refs,
        ),
        at.into(),
        part.into(),
    )
}

fn anchored_story(kind: &str) {
    let (source, at, part) = story(kind);
    let (out, view) = anchored(&source, &at);
    resolved(&source, &out, &part);
    typing_order(&out, &part);
    assert!(view.contains("{~~30~>45~~}"), "{view}");
}

#[test]
fn anchored_table_cell_uses_typing_order() {
    anchored_story("table");
}
#[test]
fn anchored_header_uses_typing_order() {
    anchored_story("header");
}
#[test]
fn anchored_footer_uses_typing_order() {
    anchored_story("footer");
}
#[test]
fn anchored_footnote_uses_typing_order() {
    anchored_story("footnotes");
}

#[test]
fn paragraph_rewrite_uses_typing_order() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), docx(&para(OLD))).unwrap();
    command(
        dir.path(),
        &[
            "edit",
            "source.docx",
            "-p",
            "p0",
            "--content",
            NEW,
            "-a",
            "Carol Diaz",
            "-d",
            "2026-10-18T10:00:00Z",
            "-o",
            "out.docx",
        ],
    );
    typing_order(
        &std::fs::read(dir.path().join("out.docx")).unwrap(),
        "word/document.xml",
    );
}

#[test]
fn replacement_across_formatted_runs_deletes_before_inserting() {
    let source = docx(&format!(
        "<w:p>{}{}{}{}</w:p>",
        run("Pay within ", false, false, None),
        run("3", true, false, None),
        run("0", false, true, None),
        run(" days.", false, false, None)
    ));
    let out = replace(&source, false);
    resolved(&source, &out, "word/document.xml");
    let revs = revisions(&out, "word/document.xml");
    let first_ins = revs.iter().position(|r| r.0 == "ins").unwrap();
    assert!(
        revs[..first_ins].iter().any(|r| r.0 == "del"),
        "all deleted pieces precede insertions: {revs:?}"
    );
    assert!(revs[first_ins..].iter().all(|r| r.0 == "ins"), "{revs:?}");
    assert!(revs.windows(2).all(|w| w[0].1 < w[1].1), "{revs:?}");
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(
        xml.contains("<w:b") && xml.contains("<w:i"),
        "deleted formatting preserved: {xml}"
    );
}

#[test]
fn compare_keeps_word_compare_insertion_before_deletion() {
    let source = docx(&para(OLD));
    let out = compare_documents(&source, &docx(&para(NEW)), "Carol Diaz").unwrap();
    let revs = revisions(&out, "word/document.xml");
    assert_eq!(
        revs.iter().map(|r| r.0.as_str()).collect::<Vec<_>>(),
        ["ins", "del"]
    );
    assert_eq!(
        revs.iter().map(|r| r.2.as_str()).collect::<Vec<_>>(),
        ["45", "30"]
    );
    resolved(&source, &out, "word/document.xml");
}

#[test]
fn filling_a_control_uses_typing_order() {
    let source = docx(
        r#"<w:p><w:r><w:t xml:space="preserve">Pay within </w:t></w:r><w:sdt><w:sdtPr><w:tag w:val="days"/><w:id w:val="101"/><w:text/></w:sdtPr><w:sdtContent><w:r><w:t>30</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>"#,
    );
    let plan = EditPlan::from_json(&json!({"schema_version":1,"author":"Carol Diaz","date":"2026-10-18T10:00:00Z","operations":[{"kind":"fill_control","control":{"tag":"days"},"text":"45"}]}).to_string()).unwrap();
    let out = apply_plan(&source, &plan).unwrap().redline;
    // #1703: the fill's insertion lands beside the control it emptied,
    // and either way around the emptied control shows Word's five-space
    // placeholder.
    resolved_text(
        &source,
        &out,
        "word/document.xml",
        "Pay within      45 days.",
        Some("Pay within 30      days."),
    );
    typing_order(&out, "word/document.xml");
}

#[test]
fn rewriting_an_entire_sentence_deletes_before_inserting() {
    let source = docx(&para(OLD));
    let plan = EditPlan::from_json(&json!({"schema_version":1,"author":"Carol Diaz","date":"2026-10-18T10:00:00Z","operations":[{"kind":"rewrite","paragraph":"p0","text":"Bob refuses."}]}).to_string()).unwrap();
    let out = apply_plan(&source, &plan).unwrap().redline;
    let revs = revisions(&out, "word/document.xml");
    assert_eq!(
        revs,
        [
            ("del".to_string(), 1, OLD.to_string()),
            ("ins".to_string(), 2, "Bob refuses.".to_string())
        ]
    );
    assert_eq!(
        text(&accept_revisions(&out).unwrap(), "word/document.xml"),
        "Bob refuses."
    );
    assert_eq!(
        text(&reject_revisions(&out).unwrap(), "word/document.xml"),
        OLD
    );
    assert_word_valid_package(&out);
}
