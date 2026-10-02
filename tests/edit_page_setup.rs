// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `page_setup`: page size, orientation and margins of the last section or
//! of every section. The copy's `w:pgSz`/`w:pgMar` change in schema order;
//! the redline keeps the old geometry in `w:sectPrChange`.

mod common;

use common::docx::{docx, docx_with_sect_pr, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::convert::{PdfOptions, docx_render_report};
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

/// The `attr` values of the first `element` in `xml` after `from`.
fn attr(xml: &str, from: usize, element: &str, attr: &str) -> Option<String> {
    let at = from + xml[from..].find(&format!("<{element} "))?;
    let tag = &xml[at..at + xml[at..].find('>')?];
    let key = format!(" {attr}=\"");
    let start = tag.find(&key)? + key.len();
    Some(tag[start..start + tag[start..].find('"')?].to_string())
}

/// The body's final `w:sectPr` (the last one in document.xml).
fn final_sect(xml: &str) -> usize {
    xml.rfind("<w:sectPr>")
        .or_else(|| xml.rfind("<w:sectPr "))
        .unwrap()
}

const SENTENCE: &str = "The parties agree that this clause governs every notice, payment, and obligation described in the schedule. ";

fn long_document() -> Vec<u8> {
    docx(
        &(0..60)
            .map(|_| para(&SENTENCE.repeat(3)))
            .collect::<String>(),
    )
}

#[test]
fn page_setup_rewrites_the_last_section_and_the_redline_records_the_old_geometry() {
    let source = long_document();
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"page_setup","page":"a4","orientation":"landscape","margins_dxa":{"top":720,"right":1080,"bottom":720,"left":1080}}]"#,
        ),
    )
    .unwrap();
    let clean = part_string(&result.clean, "word/document.xml").unwrap();
    let at = final_sect(&clean);
    assert_eq!(attr(&clean, at, "w:pgSz", "w:w").as_deref(), Some("16838"));
    assert_eq!(attr(&clean, at, "w:pgSz", "w:h").as_deref(), Some("11906"));
    assert_eq!(
        attr(&clean, at, "w:pgSz", "w:orient").as_deref(),
        Some("landscape")
    );
    for (side, value) in [
        ("w:top", "720"),
        ("w:right", "1080"),
        ("w:bottom", "720"),
        ("w:left", "1080"),
        ("w:header", "720"),
        ("w:footer", "720"),
        ("w:gutter", "0"),
    ] {
        assert_eq!(
            attr(&clean, at, "w:pgMar", side).as_deref(),
            Some(value),
            "{side}"
        );
    }
    assert!(clean[at..].find("<w:pgSz").unwrap() < clean[at..].find("<w:pgMar").unwrap());
    assert_eq!(texts(&result.clean), texts(&source));

    let redline = part_string(&result.redline, "word/document.xml").unwrap();
    // The live final section is the one holding the change record.
    let record = redline.rfind("<w:sectPrChange").expect("w:sectPrChange");
    let live = &redline[final_sect(&redline[..record])..];
    let change = live.find("<w:sectPrChange").unwrap();
    assert_eq!(attr(live, 0, "w:pgSz", "w:w").as_deref(), Some("16838"));
    assert_eq!(
        attr(live, change, "w:pgSz", "w:w").as_deref(),
        Some("12240")
    );
    assert_eq!(
        attr(live, change, "w:pgSz", "w:h").as_deref(),
        Some("15840")
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
    let rejected = part_string(
        &reject_revisions(&result.redline).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    assert_eq!(
        attr(&rejected, final_sect(&rejected), "w:pgSz", "w:w").as_deref(),
        Some("12240")
    );
    let accepted = part_string(
        &accept_revisions(&result.redline).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    assert_eq!(
        attr(&accepted, final_sect(&accepted), "w:pgSz", "w:w").as_deref(),
        Some("16838")
    );

    // A shorter, wider page holds fewer lines: the copy runs longer.
    let pages = |bytes: &[u8]| {
        docx_render_report(bytes, PdfOptions::default())
            .unwrap()
            .page_count
    };
    let (before, after) = (pages(&source), pages(&result.clean));
    assert!(before >= 2, "{before}");
    assert_ne!(before, after);

    let op = &result.report.operations[0];
    assert_eq!((op.kind.as_str(), op.status.as_str()), ("page_setup", "ok"));
    assert_eq!(op.paragraph, None);
}

#[test]
fn page_setup_custom_size_portrait_and_every_section() {
    let mid = r#"<w:p><w:pPr><w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="708" w:footer="708" w:gutter="0"/></w:sectPr></w:pPr><w:r><w:t>Section one.</w:t></w:r></w:p>"#;
    let source = docx_with_sect_pr(
        &(mid.to_string() + &para("Section two.")),
        &[],
        r#"<w:sectPr><w:pgSz w:w="15840" w:h="12240" w:orient="landscape"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="720" w:footer="720" w:gutter="0"/><w:cols w:space="720"/></w:sectPr>"#,
    );
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"page_setup","section":"all","page":{"width_dxa":12000,"height_dxa":16000},"orientation":"portrait","margins_dxa":{"left":1800}}]"#,
        ),
    )
    .unwrap();
    let clean = part_string(&result.clean, "word/document.xml").unwrap();
    let starts: Vec<usize> = clean.match_indices("<w:sectPr").map(|(at, _)| at).collect();
    assert_eq!(starts.len(), 2, "{clean}");
    for &at in &starts {
        assert_eq!(attr(&clean, at, "w:pgSz", "w:w").as_deref(), Some("12000"));
        assert_eq!(attr(&clean, at, "w:pgSz", "w:h").as_deref(), Some("16000"));
        assert_eq!(attr(&clean, at, "w:pgSz", "w:orient"), None);
        assert_eq!(
            attr(&clean, at, "w:pgMar", "w:left").as_deref(),
            Some("1800")
        );
        assert_eq!(
            attr(&clean, at, "w:pgMar", "w:right").as_deref(),
            Some("1440")
        );
    }
    // The other children stay, after pgSz/pgMar.
    assert!(
        clean[starts[1]..].find("<w:pgMar").unwrap() < clean[starts[1]..].find("<w:cols").unwrap()
    );
    assert_eq!(
        attr(&clean, starts[0], "w:pgMar", "w:header").as_deref(),
        Some("708")
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
    assert_eq!(result.report.operations[0].matches, 2);
    // Every changed section records its old geometry, mid-document ones
    // included; rejecting restores both, accepting keeps the new ones.
    let redline = part_string(&result.redline, "word/document.xml").unwrap();
    assert_eq!(redline.matches("<w:sectPrChange").count(), 2, "{redline}");
    let first = redline.find("<w:sectPr").unwrap();
    let mid = &redline[first..first + redline[first..].find("</w:pPr>").unwrap()];
    let record = mid.find("<w:sectPrChange").expect("mid-document record");
    assert_eq!(attr(mid, record, "w:pgSz", "w:w").as_deref(), Some("11906"));
    assert_eq!(
        attr(mid, record, "w:pgMar", "w:header").as_deref(),
        Some("708")
    );
    assert!(!mid[record..].contains("headerReference"));
    let rejected = part_string(
        &reject_revisions(&result.redline).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    let widths: Vec<String> = rejected
        .match_indices("<w:pgSz ")
        .map(|(at, _)| attr(&rejected, at, "w:pgSz", "w:w").unwrap())
        .collect();
    assert_eq!(widths, ["11906", "15840"]);
    let accepted = part_string(
        &accept_revisions(&result.redline).unwrap(),
        "word/document.xml",
    )
    .unwrap();
    let widths: Vec<String> = accepted
        .match_indices("<w:pgSz ")
        .map(|(at, _)| attr(&accepted, at, "w:pgSz", "w:w").unwrap())
        .collect();
    assert_eq!(widths, ["12000", "12000"]);
}

#[test]
fn page_setup_landscape_alone_swaps_the_current_size_and_creates_a_missing_section() {
    let source = docx_with_sect_pr(&para("Only paragraph."), &[], "");
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"page_setup","orientation":"landscape"}]"#,
        ),
    )
    .unwrap();
    let clean = part_string(&result.clean, "word/document.xml").unwrap();
    let at = final_sect(&clean);
    // No section: Word's default Letter, one-inch margins, turned.
    assert_eq!(attr(&clean, at, "w:pgSz", "w:w").as_deref(), Some("15840"));
    assert_eq!(attr(&clean, at, "w:pgSz", "w:h").as_deref(), Some("12240"));
    assert_eq!(
        attr(&clean, at, "w:pgSz", "w:orient").as_deref(),
        Some("landscape")
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}

#[test]
fn page_setup_refusals() {
    let source = docx(&para("Body."));
    for (ops, code) in [
        (r#"[{"kind":"page_setup"}]"#, "INVALID_EDIT"),
        (r#"[{"kind":"page_setup","page":"legal"}]"#, "INVALID_PLAN"),
        (
            r#"[{"kind":"page_setup","section":"first","page":"a4"}]"#,
            "INVALID_PLAN",
        ),
        (
            r#"[{"kind":"page_setup","page":{"width_dxa":0,"height_dxa":15840}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"page_setup","margins_dxa":{"left":6000,"right":6300}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"page_setup","margins_dxa":{"top":9000,"bottom":7000}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"page_setup","margins_dxa":{"left":-5}}]"#,
            "INVALID_PLAN",
        ),
        (
            r#"[{"kind":"page_setup","margins_dxa":{"inside":5}}]"#,
            "INVALID_PLAN",
        ),
        (
            r#"[{"kind":"page_setup","paragraph":{"index":0},"page":"a4"}]"#,
            "INVALID_PLAN",
        ),
        (
            r#"[{"kind":"page_setup","page":"a4"},{"kind":"page_setup","orientation":"landscape"}]"#,
            "OVERLAPPING_EDITS",
        ),
    ] {
        let code_seen = match EditPlan::from_json(&format!(
            r#"{{"schema_version":1,"source_sha256":"{}","author":"a","operations":{ops}}}"#,
            source_sha256(&source)
        )) {
            Err(e) => e.code,
            Ok(plan) => apply_plan(&source, &plan).unwrap_err().code,
        };
        assert_eq!(code_seen, code, "{ops}");
    }
}
