// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `format_run`: change the run formatting of one occurrence of existing
//! text. The clean copy splits the run at the range's edges and formats only
//! the range; the redline records the old formatting as `w:rPrChange`.

mod common;

use common::docx::{docx, para, part_string, run};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeKind, list_changes};
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::{paragraphs, source_sha256};

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

fn span_at(bytes: &[u8], at: usize) -> jubarte::inspect::Span {
    paragraphs(bytes).unwrap()[0]
        .runs
        .iter()
        .find(|s| s.start <= at && at < s.end)
        .cloned()
        .unwrap_or_else(|| panic!("no span covers char {at}"))
}

const SENTENCE: &str = "The fee is ten dollars per month.";

#[test]
fn format_run_splits_the_run_and_formats_only_the_range() {
    let source = docx(&para(SENTENCE));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"ten dollars","format":{"bold":true,"underline":true,"highlight":"yellow","font":"Arial","size_pt":11,"color":"FF0000","strike":true,"caps":true}}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), vec![SENTENCE.to_string()]);
    let at = SENTENCE.find("ten").unwrap();
    let span = span_at(&result.clean, at);
    assert!(span.bold && span.underline && !span.italic, "{span:?}");
    assert_eq!(span.highlight.as_deref(), Some("yellow"));
    assert_eq!(&SENTENCE[span.start..span.end], "ten dollars");
    for plain in ["The fee", " per month"] {
        let s = span_at(&result.clean, SENTENCE.find(plain).unwrap());
        assert!(!s.bold && !s.underline && s.highlight.is_none(), "{s:?}");
    }
    // `Span` carries only bold/italic/underline/highlight: check the rest in XML.
    let clean_xml = part_string(&result.clean, "word/document.xml").unwrap();
    for needle in [
        r#"w:ascii="Arial""#,
        r#"w:hAnsi="Arial""#,
        r#"<w:sz w:val="22""#,
        r#"<w:szCs w:val="22""#,
        r#"<w:color w:val="FF0000""#,
        "<w:strike",
        "<w:caps",
    ] {
        assert!(clean_xml.contains(needle), "{needle} in {clean_xml}");
    }
    // Schema order inside the formatted run's w:rPr: rFonts, b, caps,
    // strike, color, sz, highlight, u.
    let fonts = clean_xml.find("<w:rFonts").unwrap();
    let rpr_start = clean_xml[..fonts].rfind("<w:rPr").unwrap();
    let rpr = &clean_xml[rpr_start..fonts + clean_xml[fonts..].find("</w:rPr>").unwrap()];
    let order: Vec<usize> = [
        "rFonts",
        "b",
        "caps",
        "strike",
        "color",
        "sz",
        "highlight",
        "u",
    ]
    .iter()
    .map(|n| {
        [" ", "/", ">"]
            .iter()
            .find_map(|end| rpr.find(&format!("<w:{n}{end}")))
            .unwrap_or_else(|| panic!("{n} in {rpr}"))
    })
    .collect();
    assert!(
        order.array_windows().all(|[first, second]| first < second),
        "{order:?}"
    );

    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(redline_xml.contains("<w:rPrChange"), "{redline_xml}");
    let changes = list_changes(&result.redline).unwrap();
    let formatting: Vec<_> = changes
        .iter()
        .filter(|c| c.kind == ChangeKind::Formatting)
        .collect();
    assert_eq!(formatting.len(), 1, "{changes:?}");
    assert_eq!(formatting[0].text, "ten dollars");
    assert!(changes.iter().all(|c| c.kind == ChangeKind::Formatting));
    assert_eq!(
        texts(&accept_revisions(&result.redline).unwrap()),
        texts(&result.clean)
    );
    assert_eq!(
        texts(&reject_revisions(&result.redline).unwrap()),
        texts(&source)
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
    let op = &result.report.operations[0];
    assert_eq!((op.kind.as_str(), op.status.as_str()), ("format_run", "ok"));
}

#[test]
fn format_run_clears_formatting_across_runs() {
    let body = format!(
        r#"<w:p>{}{}</w:p>"#,
        run("Bold ", true, false, None),
        run("tail", true, true, None)
    );
    let source = docx(&body);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"ld ta","format":{"bold":false}}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), vec!["Bold tail".to_string()]);
    assert!(span_at(&result.clean, 0).bold);
    let cleared = span_at(&result.clean, 2);
    assert!(!cleared.bold, "{cleared:?}");
    assert!(span_at(&result.clean, 7).bold && span_at(&result.clean, 7).italic);
    assert!(!span_at(&result.clean, 6).bold && span_at(&result.clean, 6).italic);
    assert_word_valid_package(&result.redline);
}

#[test]
fn format_run_occurrence_picks_one_of_several() {
    let source = docx(&para("one and one and one"));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"one","occurrence":2,"format":{"italic":true}}]"#,
        ),
    )
    .unwrap();
    assert!(!span_at(&result.clean, 0).italic);
    let second = span_at(&result.clean, 8);
    assert!(
        second.italic && second.start == 8 && second.end == 11,
        "{second:?}"
    );
    assert!(!span_at(&result.clean, 16).italic);
    assert_eq!(result.report.operations[0].matches, 3);
}

#[test]
fn format_run_refusals() {
    let source = docx(&para("one and one"));
    for (ops, code) in [
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"one","format":{"bold":true}}]"#,
            "AMBIGUOUS_ANCHOR",
        ),
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"one","occurrence":3,"format":{"bold":true}}]"#,
            "AMBIGUOUS_ANCHOR",
        ),
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"one","occurrence":0,"format":{"bold":true}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"two","format":{"bold":true}}]"#,
            "ANCHOR_NOT_FOUND",
        ),
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"and","format":{}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"and","format":{"color":"red"}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"and","format":{"size_pt":0}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"and","format":{"font":""}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"format_run","paragraph":{"index":0},"find":"and","format":{"bold":true}},
                {"kind":"replace","paragraph":{"index":0},"find":"nd o","replacement":"x"}]"#,
            "OVERLAPPING_EDITS",
        ),
    ] {
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert_eq!(err.code, code, "{ops}");
    }
    let json = r#"{"schema_version":1,"author":"a","operations":[{"kind":"format_run","paragraph":{"index":0},"find":"and","format":{"bold":true},"text":"x"}]}"#;
    assert_eq!(EditPlan::from_json(json).unwrap_err().code, "INVALID_PLAN");
}

#[test]
fn format_run_beside_a_text_edit_in_the_same_paragraph() {
    let source = docx(&para(SENTENCE));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"The fee","replacement":"A charge"},
                {"kind":"format_run","paragraph":{"index":0},"find":"month","format":{"bold":true}}]"#,
        ),
    )
    .unwrap();
    let text = &texts(&result.clean)[0];
    assert_eq!(text, "A charge is ten dollars per month.");
    let span = span_at(&result.clean, text.find("month").unwrap());
    assert!(span.bold);
    assert_eq!(&text[span.start..span.end], "month");
    assert_word_valid_package(&result.redline);
}
