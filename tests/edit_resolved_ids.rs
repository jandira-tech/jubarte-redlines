// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A plan addresses the document its author read. When `resolve_revisions`
//! or `existing_revisions` settles tracked changes first, paragraphs merge
//! and vanish; a positional id still names the paragraph it named in the
//! source, and the report maps the ids that moved.

mod common;

use common::docx::docx;
use common::validity::assert_word_valid_package;
use jubarte::edit::{EditPlan, EditResult, ParagraphMove, apply_plan};
use jubarte::inspect::paragraphs;

const REV: &str = r#"w:author="Other" w:date="2026-09-01T00:00:00Z""#;

/// p0's mark is deleted (it merges with p1), p2 is deleted whole, table 0
/// is deleted whole; p3, p4 and table 1 come through unchanged.
fn source() -> Vec<u8> {
    let cell = |text: &str, deleted: bool| {
        let tr_pr = if deleted {
            format!(r#"<w:trPr><w:del w:id="40" {REV}/></w:trPr>"#)
        } else {
            String::new()
        };
        let run = if deleted {
            format!(r#"<w:del w:id="41" {REV}><w:r><w:delText>{text}</w:delText></w:r></w:del>"#)
        } else {
            format!("<w:r><w:t>{text}</w:t></w:r>")
        };
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr>{tr_pr}<w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p>{run}</w:p></w:tc></w:tr></w:tbl>"#
        )
    };
    docx(&[
        format!(r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>Alpha</w:t></w:r></w:p>"#),
        "<w:p><w:r><w:t>Beta</w:t></w:r></w:p>".to_string(),
        format!(r#"<w:p><w:pPr><w:rPr><w:del w:id="2" {REV}/></w:rPr></w:pPr><w:del w:id="3" {REV}><w:r><w:delText>Gamma</w:delText></w:r></w:del></w:p>"#),
        "<w:p><w:r><w:t>Delta clause.</w:t></w:r></w:p>".to_string(),
        "<w:p><w:r><w:t>Epsilon clause.</w:t></w:r></w:p>".to_string(),
        cell("Gone cell", true),
        "<w:p/>".to_string(),
        cell("Cell text", false),
        "<w:p><w:r><w:t>Zeta.</w:t></w:r></w:p>".to_string(),
    ]
    .concat())
}

fn plan(settle: &str, operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z",{settle},"operations":{operations}}}"#
    ))
    .unwrap()
}

const RESOLVE: &str = r#""resolve_revisions":{"accept":{}}"#;
const ACCEPT: &str = r#""existing_revisions":"accept""#;

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn ok(out: &EditResult) {
    for op in &out.report.operations {
        assert_eq!(op.status, "ok", "{op:?}");
    }
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
}

#[test]
fn ids_name_the_paragraphs_of_the_source_once_revisions_are_settled() {
    for settle in [RESOLVE, ACCEPT] {
        let out = apply_plan(
            &source(),
            &plan(
                settle,
                r#"[{"kind":"rewrite","paragraph":"p3","text":"Delta clause, amended."},
                    {"kind":"replace","paragraph":"body:p:4","find":"Epsilon","replacement":"Eta"},
                    {"kind":"replace","paragraph":"t1.r0.c0","find":"Cell","replacement":"Box"},
                    {"kind":"comment","paragraph":{"index":8},"text":"Last."}]"#,
            ),
        )
        .unwrap();
        ok(&out);
        assert_eq!(
            texts(&out.clean),
            [
                "AlphaBeta",
                "Delta clause, amended.",
                "Eta clause.",
                "",
                "Box text",
                "Zeta."
            ],
            "{settle}"
        );
        // The report names each paragraph as the output has it.
        let at: Vec<_> = out
            .report
            .operations
            .iter()
            .map(|op| op.paragraph.clone().unwrap())
            .collect();
        assert_eq!(
            at,
            ["body:p:1", "body:p:2", "body:p:4", "body:p:5"],
            "{settle}"
        );
    }
}

#[test]
fn the_report_maps_every_id_that_moved() {
    let out = apply_plan(
        &source(),
        &plan(
            RESOLVE,
            r#"[{"kind":"comment","paragraph":"p3","text":"Here."}]"#,
        ),
    )
    .unwrap();
    let moved = |from: &str, to: Option<&str>| ParagraphMove {
        from: from.to_string(),
        to: to.map(str::to_string),
    };
    assert_eq!(
        out.report.paragraph_ids,
        [
            moved("body:p:1", Some("body:p:0")),
            moved("body:p:2", None),
            moved("body:p:3", Some("body:p:1")),
            moved("body:p:4", Some("body:p:2")),
            moved("body:p:5", None),
            moved("body:p:6", Some("body:p:3")),
            moved("body:p:7", Some("body:p:4")),
            moved("body:p:8", Some("body:p:5")),
        ]
    );
    // The JSONL `load` line carries the same map.
    let load: serde_json::Value =
        serde_json::from_str(out.report.to_jsonl().lines().next().unwrap()).unwrap();
    assert_eq!(
        load["paragraph_ids"][1],
        serde_json::json!({"from": "body:p:2", "to": null})
    );
    assert_eq!(load["paragraph_ids"].as_array().unwrap().len(), 8);
    // Nothing settled, nothing moved.
    let unsettled = docx("<w:p><w:r><w:t>Plain.</w:t></w:r></w:p>");
    let out = apply_plan(
        &unsettled,
        &plan(
            RESOLVE,
            r#"[{"kind":"comment","paragraph":"p0","text":"Hi."}]"#,
        ),
    )
    .unwrap();
    assert!(out.report.paragraph_ids.is_empty());
    let json = serde_json::to_string(&out.report).unwrap();
    assert!(!json.contains("paragraph_ids"), "{json}");
}

#[test]
fn an_anchored_edit_follows_a_paragraph_into_the_one_it_merged_with() {
    let out = apply_plan(
        &source(),
        &plan(
            RESOLVE,
            r#"[{"kind":"replace","paragraph":"p1","find":"Beta","replacement":"Bravo"}]"#,
        ),
    )
    .unwrap();
    ok(&out);
    assert_eq!(texts(&out.clean)[0], "AlphaBravo");
    assert_eq!(
        out.report.operations[0].paragraph.as_deref(),
        Some("body:p:0")
    );
}

#[test]
fn a_whole_paragraph_edit_of_a_merged_or_removed_paragraph_is_refused_by_name() {
    for (op, says) in [
        (
            r#"{"kind":"rewrite","paragraph":"p1","text":"Bravo"}"#,
            "body:p:1 merged into body:p:0",
        ),
        (
            r#"{"kind":"delete_paragraph","paragraph":"p0"}"#,
            "body:p:0 merged into body:p:0",
        ),
        (
            r#"{"kind":"rewrite","paragraph":"p2","text":"Gamma"}"#,
            "body:p:2 is gone",
        ),
        (
            r#"{"kind":"replace","paragraph":"p2","find":"Gamma","replacement":"G"}"#,
            "body:p:2 is gone",
        ),
        (
            r#"{"kind":"replace","paragraph":"t0.r0.c0","find":"Gone","replacement":"G"}"#,
            "body:p:5 is gone",
        ),
    ] {
        let e = apply_plan(&source(), &plan(RESOLVE, &format!("[{op}]"))).unwrap_err();
        assert_eq!(e.code, "PARAGRAPH_RESOLVED", "{op}: {e}");
        assert!(e.message.contains(says), "{op}: {e}");
        assert!(e.message.contains("revisions"), "{op}: {e}");
    }
}
