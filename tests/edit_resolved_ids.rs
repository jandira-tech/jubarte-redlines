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
                    {"kind":"replace","paragraph":"p4","find":"Epsilon","replacement":"Eta"},
                    {"kind":"replace","paragraph":"t1.row0.cell0","find":"Cell","replacement":"Box"},
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
        assert_eq!(at, ["p1", "p2", "p4", "p5"], "{settle}");
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
            moved("p1", Some("p0")),
            moved("p2", None),
            moved("p3", Some("p1")),
            moved("p4", Some("p2")),
            moved("p5", None),
            moved("p6", Some("p3")),
            moved("p7", Some("p4")),
            moved("p8", Some("p5")),
        ]
    );
    // The JSONL `load` line carries the same map.
    let load: serde_json::Value =
        serde_json::from_str(out.report.to_jsonl().lines().next().unwrap()).unwrap();
    assert_eq!(
        load["paragraph_ids"][1],
        serde_json::json!({"from": "p2", "to": null})
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
    assert_eq!(out.report.operations[0].paragraph.as_deref(), Some("p0"));
}

#[test]
fn a_whole_paragraph_edit_of_a_merged_or_removed_paragraph_is_refused_by_name() {
    for (op, says) in [
        (
            r#"{"kind":"rewrite","paragraph":"p1","text":"Bravo"}"#,
            "p1 merged into p0",
        ),
        (
            r#"{"kind":"delete_paragraph","paragraph":"p0"}"#,
            "p0 merged into p0",
        ),
        (
            r#"{"kind":"rewrite","paragraph":"p2","text":"Gamma"}"#,
            "p2 is gone",
        ),
        (
            r#"{"kind":"replace","paragraph":"p2","find":"Gamma","replacement":"G"}"#,
            "p2 is gone",
        ),
        (
            r#"{"kind":"replace","paragraph":"t0.row0.cell0","find":"Gone","replacement":"G"}"#,
            "p5 is gone",
        ),
    ] {
        let e = apply_plan(&source(), &plan(RESOLVE, &format!("[{op}]"))).unwrap_err();
        assert_eq!(e.code, "PARAGRAPH_RESOLVED", "{op}: {e}");
        assert!(e.message.contains(says), "{op}: {e}");
        assert!(e.message.contains("revisions"), "{op}: {e}");
    }
}

#[test]
fn a_control_id_names_the_source_control_after_settling() {
    // CodeRabbit on #488: accepting the deletions before the controls moves
    // `p3.cc0` to `p1.cc0`; the plan's id still names the control its
    // author read.
    let control = |tag: &str, text: &str| {
        format!(
            r#"<w:sdt><w:sdtPr><w:tag w:val="{tag}"/><w:text/></w:sdtPr><w:sdtContent><w:r><w:t>{text}</w:t></w:r></w:sdtContent></w:sdt>"#
        )
    };
    let source = docx(&[
        format!(r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>Alpha</w:t></w:r></w:p>"#),
        "<w:p><w:r><w:t>Beta</w:t></w:r></w:p>".to_string(),
        format!(r#"<w:p><w:pPr><w:rPr><w:del w:id="2" {REV}/></w:rPr></w:pPr><w:del w:id="3" {REV}><w:r><w:delText>Gamma</w:delText></w:r></w:del></w:p>"#),
        format!(r#"<w:p><w:r><w:t xml:space="preserve">Name: </w:t></w:r>{}</w:p>"#, control("Name", "Click")),
        format!(r#"<w:p><w:r><w:t xml:space="preserve">Town: </w:t></w:r>{}</w:p>"#, control("Town", "Pick")),
    ]
    .concat());
    let ids: Vec<String> = jubarte::inspect::controls(&source)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(ids, ["p3.cc0", "p4.cc0"]);
    for settle in [ACCEPT, RESOLVE] {
        let out = apply_plan(
            &source,
            &plan(
                settle,
                r#"[{"kind":"fill_control","control":"p3.cc0","text":"Ada"},
                    {"kind":"fill_control","control":{"id":"p4.cc0"},"text":"Lyon"}]"#,
            ),
        )
        .unwrap();
        let texts: Vec<String> = paragraphs(&out.clean)
            .unwrap()
            .into_iter()
            .map(|p| p.text)
            .collect();
        assert_eq!(texts, ["AlphaBeta", "Name: Ada", "Town: Lyon"], "{settle}");
    }
}

#[test]
fn settled_control_errors_name_the_source_ids() {
    // Two controls share a tag after a deletion moved them: the ambiguity
    // names the ids the author read, and a control whose text settling
    // removes is gone, not silently another control.
    let control = |tag: &str, text: &str| {
        format!(
            r#"<w:sdt><w:sdtPr><w:tag w:val="{tag}"/><w:text/></w:sdtPr><w:sdtContent><w:r><w:t>{text}</w:t></w:r></w:sdtContent></w:sdt>"#
        )
    };
    let source = docx(&[
        format!(r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:del w:id="2" {REV}><w:r><w:delText>Gone</w:delText></w:r></w:del></w:p>"#),
        format!(r#"<w:p><w:r><w:t xml:space="preserve">A: </w:t></w:r>{}</w:p>"#, control("Same", "x")),
        format!(r#"<w:p><w:r><w:t xml:space="preserve">B: </w:t></w:r>{}</w:p>"#, control("Same", "y")),
    ]
    .concat());
    let e = apply_plan(
        &source,
        &plan(
            ACCEPT,
            r#"[{"kind":"fill_control","control":{"tag":"Same"},"text":"z"}]"#,
        ),
    )
    .unwrap_err();
    assert!(e.message.contains("p1.cc0, p2.cc0"), "{e}");
    let e = apply_plan(
        &source,
        &plan(
            ACCEPT,
            r#"[{"kind":"fill_control","control":"p0.cc0","text":"z"}]"#,
        ),
    )
    .unwrap_err();
    assert!(e.message.contains("(first p1.cc0, last p2.cc0)"), "{e}");
}

#[test]
fn controls_in_a_settled_paragraph_keep_their_source_ids() {
    // pi on #488: X sits in a deleted paragraph, Y after it. After
    // accepting, `p2.cc0` still names Y and `p1.cc0` X, and the expired
    // `body:sdt:0` hints X's id, never Y's.
    let source = docx(&[
        "<w:p><w:r><w:t>keep</w:t></w:r></w:p>".to_string(),
        format!(r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:sdt><w:sdtPr><w:tag w:val="X"/><w:text/></w:sdtPr><w:sdtContent><w:del w:id="2" {REV}><w:r><w:delText>xx</w:delText></w:r></w:del></w:sdtContent></w:sdt></w:p>"#),
        r#"<w:p><w:sdt><w:sdtPr><w:tag w:val="Y"/><w:text/></w:sdtPr><w:sdtContent><w:r><w:t>yy</w:t></w:r></w:sdtContent></w:sdt></w:p>"#.to_string(),
    ]
    .concat());
    let out = apply_plan(
        &source,
        &plan(
            ACCEPT,
            r#"[{"kind":"fill_control","control":"p2.cc0","text":"Y!"}]"#,
        ),
    )
    .unwrap();
    let texts: Vec<String> = paragraphs(&out.clean)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(texts, ["keep", "Y!"]);
    // Accepting keeps X's control, emptied, in the paragraph Y's took
    // over: `p1.cc0` still names X.
    let out = apply_plan(
        &source,
        &plan(
            ACCEPT,
            r#"[{"kind":"fill_control","control":"p1.cc0","text":"X!"}]"#,
        ),
    )
    .unwrap();
    let texts: Vec<String> = paragraphs(&out.clean)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect();
    assert_eq!(texts, ["keep", "X!yy"]);
    let e = apply_plan(
        &source,
        &plan(
            ACCEPT,
            r#"[{"kind":"fill_control","control":"body:sdt:0","text":"z"}]"#,
        ),
    )
    .unwrap_err();
    assert!(e.message.contains("the settled spelling is p1.cc0"), "{e}");
}
