// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::edit`: a plan of uniquely anchored operations is applied to a
//! copy, the comparer produces the redline, and every operation gets a report
//! line. The scenarios mirror what an agent hand-rolled over `document.xml`
//! (redline.py): replace a phrase, insert after an anchor, delete a paragraph,
//! insert a bold-headed paragraph, comment on inserted text.

mod common;

use common::docx::{Part, docx, docx_with, para, part_string, run};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan, apply_plan_json, preview_plan};
use jubarte::inspect::{paragraphs, source_sha256, summary};

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn plan(source: &[u8], operations: &str) -> EditPlan {
    let json = format!(
        r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","date":"2026-09-25T12:00:00Z","operations":{operations}}}"#,
        source_sha256(source)
    );
    EditPlan::from_json(&json).unwrap()
}

const SIGNATURE: &str =
    "The individual signing above also signs in his or her individual capacity.";

#[test]
fn replace_unique_text_yields_clean_copy_and_tracked_redline() {
    let source = docx(&(para("Heading") + &para(SIGNATURE)));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"op-1","kind":"replace","paragraph":{"index":1},"find":"his or her","replacement":"an"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean)[1],
        "The individual signing above also signs in an individual capacity."
    );
    // The redline is a tracked-changes document whose accepted view is the clean copy.
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), texts(&result.clean));
    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(redline_xml.contains("<w:del "), "deletion tracked");
    assert!(redline_xml.contains("<w:ins "), "insertion tracked");
    assert!(redline_xml.contains(r#"w:author="Claude""#));
    assert_word_valid_package(&result.redline);
    assert_word_valid_package(&result.clean);
    // Report.
    let report = &result.report;
    assert!(report.ok);
    assert_eq!(report.operations.len(), 1);
    let op = &report.operations[0];
    assert_eq!(
        (op.id.as_str(), op.status.as_str(), op.matches),
        ("op-1", "ok", 1)
    );
    assert_eq!(op.paragraph.as_deref(), Some("body:p:1"));
    assert_eq!(
        op.context.as_deref(),
        Some("above also signs in {his or her→an} individual capacity")
    );
    assert!(report.revisions.inserted >= 1 && report.revisions.deleted >= 1);
    assert_eq!(report.source_sha256, source_sha256(&source));
    // Inputs are never mutated.
    assert_eq!(source_sha256(&source), report.source_sha256);
}

#[test]
fn insert_after_anchor_inherits_the_anchor_runs_formatting() {
    let body = format!(
        "<w:p>{}{}{}</w:p>",
        run("(a) ", false, false, None),
        run("Confidentiality. ", true, false, None),
        run(
            "You may disclose it only to your attorneys, retained experts, and process servers.",
            false,
            true,
            None
        )
    );
    let source = docx(&body);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert","paragraph":{"starts_with":"(a) Confidentiality."},"after":"retained experts, ","text":"court reporters, "}]"#,
        ),
    )
    .unwrap();
    let p = &paragraphs(&result.clean).unwrap()[0];
    assert_eq!(
        p.text,
        "(a) Confidentiality. You may disclose it only to your attorneys, retained experts, court reporters, and process servers."
    );
    // Still three formatting spans: the insertion joined the italic run.
    assert_eq!(p.runs.len(), 3);
    assert!(p.runs[2].italic);
    assert_eq!(
        result.report.operations[0].id, "op-1",
        "ids are assigned when omitted"
    );
    assert_eq!(
        result.report.operations[0].context.as_deref(),
        Some(", retained experts, {+court reporters, }and process servers.")
    );
}

#[test]
fn insert_at_paragraph_end_and_start_and_delete_text() {
    let source = docx(&para("You will hold the Information in confidence."));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert","paragraph":{"index":0},"position":"end","text":" You remain responsible for every act."},
                {"kind":"insert","paragraph":{"index":0},"position":"start","text":"(a) "},
                {"kind":"delete","paragraph":{"index":0},"find":"the Information "}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean)[0],
        "(a) You will hold in confidence. You remain responsible for every act."
    );
    assert_eq!(
        result.report.operations[2].context.as_deref(),
        Some("You will hold {-the Information }in confidence.")
    );
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), texts(&result.clean));
}

#[test]
fn ambiguous_anchor_fails_the_whole_plan_and_reports_every_operation() {
    let source = docx(&para("aaa and retained experts and retained experts"));
    let err = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"first","kind":"replace","paragraph":{"index":0},"find":"aaa","replacement":"b"},
                {"id":"second","kind":"replace","paragraph":{"index":0},"find":"retained experts","replacement":"x"},
                {"id":"third","kind":"replace","paragraph":{"index":0},"find":"aa","replacement":"c"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "AMBIGUOUS_ANCHOR");
    assert_eq!(err.operation.as_deref(), Some("second"));
    let statuses: Vec<(&str, &str, usize)> = err
        .outcomes
        .iter()
        .map(|o| (o.id.as_str(), o.status.as_str(), o.matches))
        .collect();
    // Overlapping candidates count too: "aaa" holds two occurrences of "aa".
    assert_eq!(
        statuses,
        [
            ("first", "ok", 1),
            ("second", "failed", 2),
            ("third", "failed", 2)
        ]
    );
}

#[test]
fn paragraphs_inserted_after_one_anchor_keep_plan_order() {
    let source = docx(&(para("Heading") + &para("Tail")));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert_paragraph","paragraph":{"index":0},"position":"after","runs":[{"text":"First"}]},
                {"kind":"insert_paragraph","paragraph":{"index":0},"position":"after","runs":[{"text":"Second"}]},
                {"kind":"insert_paragraph","paragraph":{"index":1},"position":"before","runs":[{"text":"Third"}]},
                {"kind":"insert_paragraph","paragraph":{"index":1},"position":"before","runs":[{"text":"Fourth"}]}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean),
        ["Heading", "First", "Second", "Third", "Fourth", "Tail"]
    );
}

#[test]
fn comment_text_must_be_nonempty_xml_safe_text() {
    let source = docx(&para("Sections 1(g), 2(e), 3 survive."));
    for ops in [
        r#"[{"kind":"comment","paragraph":{"index":0},"text":"note\u0001"}]"#,
        r#"[{"kind":"replace","paragraph":{"index":0},"find":"3","replacement":"4","comment":"bad\u0008"}]"#,
        r#"[{"kind":"insert","paragraph":{"index":0},"after":"1(g), ","text":"2(c), ","comment":"\u001f"}]"#,
        r#"[{"kind":"insert_paragraph","paragraph":{"index":0},"position":"after","runs":[{"text":"New"}],"comment":"x￿"}]"#,
        r#"[{"kind":"replace","paragraph":{"index":0},"find":"3","replacement":"4","comment":"  "}]"#,
    ] {
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert_eq!(err.code, "INVALID_EDIT", "{ops}");
    }
    // A line break splits the comment into paragraphs and stays allowed.
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"comment","paragraph":{"index":0},"text":"first line\nsecond line"}]"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&result.redline);
}

#[test]
fn missing_anchor_and_bad_selectors_are_reported_with_codes() {
    let source = docx(&(para("one") + &para("two") + &para("two again")));
    for (ops, code) in [
        (
            r#"[{"kind":"delete","paragraph":{"index":0},"find":"zzz"}]"#,
            "ANCHOR_NOT_FOUND",
        ),
        (
            r#"[{"kind":"delete","paragraph":{"index":9},"find":"one"}]"#,
            "ANCHOR_NOT_FOUND",
        ),
        (
            r#"[{"kind":"delete","paragraph":{"id":"body:p:7"},"find":"one"}]"#,
            "ANCHOR_NOT_FOUND",
        ),
        (
            r#"[{"kind":"delete","paragraph":{"starts_with":"two"},"find":"two"}]"#,
            "AMBIGUOUS_ANCHOR",
        ),
        (
            r#"[{"kind":"delete","paragraph":{"contains":"nowhere"},"find":"two"}]"#,
            "ANCHOR_NOT_FOUND",
        ),
        (
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"","replacement":"x"}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"one","replacement":"a\u0007b"}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"split_paragraph","paragraph":{"index":0}}]"#,
            "INVALID_PLAN",
        ),
    ] {
        let json = format!(r#"{{"schema_version":1,"author":"Claude","operations":{ops}}}"#);
        let err = apply_plan_json(&source, &json).unwrap_err();
        assert_eq!(err.code, code, "{ops}");
    }
    let err = apply_plan_json(
        &source,
        r#"{"schema_version":2,"author":"a","operations":[]}"#,
    )
    .unwrap_err();
    assert_eq!(err.code, "UNSUPPORTED_SCHEMA");
    let err = apply_plan_json(&source, "{not json").unwrap_err();
    assert_eq!(err.code, "INVALID_PLAN");
}

#[test]
fn stale_source_hash_is_refused_before_any_work() {
    let source = docx(&para("text"));
    let json = r#"{"schema_version":1,"source_sha256":"0000000000000000000000000000000000000000000000000000000000000000","author":"Claude","operations":[{"kind":"delete","paragraph":{"index":0},"find":"text"}]}"#;
    let err = apply_plan_json(&source, json).unwrap_err();
    assert_eq!(err.code, "STALE_SOURCE");
    assert!(err.outcomes.is_empty());
}

#[test]
fn unguarded_plan_is_applied_and_flagged_in_the_report() {
    let source = docx(&para("text"));
    let json = r#"{"schema_version":1,"author":"Claude","operations":[{"kind":"replace","paragraph":{"index":0},"find":"text","replacement":"words"}]}"#;
    let result = apply_plan_json(&source, json).unwrap();
    assert!(!result.report.guarded);
    assert_eq!(texts(&result.clean)[0], "words");
}

#[test]
fn existing_revisions_are_refused_by_default_and_flattened_on_request() {
    let body = r#"<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r><w:del w:id="1" w:author="a" w:date="2020-01-01T00:00:00Z"><w:r><w:delText>gone</w:delText></w:r></w:del><w:ins w:id="2" w:author="a" w:date="2020-01-01T00:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#;
    let source = docx(body);
    let ops = r#"[{"kind":"replace","paragraph":{"index":0},"find":"keep","replacement":"hold"}]"#;
    let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
    assert_eq!(err.code, "EXISTING_REVISIONS");

    let json = format!(
        r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","existing_revisions":"accept","operations":{ops}}}"#,
        source_sha256(&source)
    );
    let result = apply_plan_json(&source, &json).unwrap();
    assert_eq!(texts(&result.clean)[0], "hold new");
    assert_ne!(result.report.base_sha256, result.report.source_sha256);
    assert_eq!(summary(&result.clean).unwrap().revisions, 0);

    let json = json.replace(r#""accept""#, r#""reject""#);
    let result = apply_plan_json(&source, &json).unwrap();
    assert_eq!(texts(&result.clean)[0], "hold gone");
}

#[test]
fn overlapping_edits_and_edits_on_a_deleted_paragraph_are_refused() {
    let source = docx(&(para("retained experts and process servers") + &para("second")));
    let err = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"experts and","replacement":"x"},
                {"kind":"delete","paragraph":{"index":0},"find":"and process"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "OVERLAPPING_EDITS");
    let err = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"delete_paragraph","paragraph":{"index":1}},
                {"kind":"replace","paragraph":{"index":1},"find":"second","replacement":"x"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "OVERLAPPING_EDITS");
}

#[test]
fn two_inserts_at_one_position_keep_plan_order() {
    let source = docx(&para("Sections 1(g), 2(e), 3 survive."));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert","paragraph":{"index":0},"after":"1(g), ","text":"2(c), "},
                {"kind":"insert","paragraph":{"index":0},"after":"1(g), ","text":"2(d), "}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean)[0],
        "Sections 1(g), 2(c), 2(d), 2(e), 3 survive."
    );
}

#[test]
fn delete_paragraph_vanishes_from_clean_and_is_tracked_in_redline() {
    let source = docx(
        &(para("(c) Third.")
            + &para("(d) Onward Disclosure. You will not disclose.")
            + &para("5. Confidentiality")),
    );
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"delete_paragraph","paragraph":{"starts_with":"(d) Onward Disclosure."}}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), ["(c) Third.", "5. Confidentiality"]);
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), texts(&result.clean));
    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(redline_xml.contains("<w:delText"), "deleted runs tracked");
    assert_eq!(result.report.paragraphs.from, 3);
    assert_eq!(result.report.paragraphs.to, 2);
    assert_word_valid_package(&result.redline);
}

#[test]
fn delete_paragraph_refuses_section_and_table_cell_last_paragraphs() {
    let with_section = format!(
        "{}<w:p><w:pPr><w:sectPr><w:type w:val=\"nextPage\"/></w:sectPr></w:pPr><w:r><w:t>sect</w:t></w:r></w:p>{}",
        para("a"),
        para("b")
    );
    let source = docx(&with_section);
    let err = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"delete_paragraph","paragraph":{"index":1}}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "UNSUPPORTED_STRUCTURE");
    let table = format!(
        "{}<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
        para("a"),
        para("only cell paragraph")
    );
    let source = docx(&table);
    let err = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"delete_paragraph","paragraph":{"index":1}}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "UNSUPPORTED_STRUCTURE");
}

/// New runs start from the anchor's body run, not its bold lead-in: only the
/// run asking for bold is bold.
#[test]
fn insert_paragraph_copies_anchor_properties_and_sets_run_formatting() {
    let body = format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Sub"/><w:ind w:left="720" w:hanging="360"/><w:sectPr/></w:pPr>{}{}</w:p>"#,
        run("(f) ", true, false, None),
        run(
            "Notice of Inability to Comply. You will notify us.",
            false,
            false,
            None
        )
    );
    let source = docx(&body);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert_paragraph","paragraph":{"starts_with":"(f) Notice"},"position":"after","runs":[{"text":"(g) "},{"text":"Automated Tools. ","bold":true},{"text":"You will not upload the Information to any AI service."}]}]"#,
        ),
    )
    .unwrap();
    let paras = paragraphs(&result.clean).unwrap();
    assert_eq!(paras.len(), 2);
    let new = &paras[1];
    assert_eq!(
        new.text,
        "(g) Automated Tools. You will not upload the Information to any AI service."
    );
    assert_eq!(new.style.as_deref(), Some("Sub"));
    assert_eq!(
        new.runs.iter().map(|r| r.bold).collect::<Vec<_>>(),
        [false, true, false]
    );
    let clean_xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert_eq!(
        clean_xml.matches("<w:sectPr").count(),
        2,
        "the anchor's own sectPr is not cloned"
    );
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), texts(&result.clean));
    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(redline_xml.contains("Automated Tools."));
    assert!(redline_xml.contains("<w:ins "));
    assert_word_valid_package(&result.redline);
}

#[test]
fn comments_on_source_text_and_on_inserted_text_survive_compare() {
    let source = docx(
        &(para("Sections 1(g), 2(e), 3 survive termination.")
            + &para("Notices go by email to legal@acme.example in our case.")),
    );
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"survival","kind":"comment","paragraph":{"index":0},"find":"Sections 1(g), ","text":"Added 2(c) so the deletion duty survives."},
                {"id":"courier","kind":"replace","paragraph":{"index":1},"find":"email","replacement":"email or courier","comment":"Courier only for indemnity notices."},
                {"id":"para","kind":"comment","paragraph":{"index":1},"text":"Whole paragraph note."}]"#,
        ),
    )
    .unwrap();
    assert_eq!(result.report.comments_added, 3);
    let ids: Vec<Option<u32>> = result
        .report
        .operations
        .iter()
        .map(|o| o.comment_id)
        .collect();
    assert_eq!(ids, [Some(0), Some(1), Some(2)]);
    for bytes in [&result.clean, &result.redline] {
        assert_eq!(summary(bytes).unwrap().comments, 3);
        let xml = part_string(bytes, "word/document.xml").unwrap();
        for id in 0..3 {
            assert!(
                xml.contains(&format!(r#"<w:commentRangeStart w:id="{id}""#)),
                "start {id}"
            );
            assert!(
                xml.contains(&format!(r#"<w:commentRangeEnd w:id="{id}""#)),
                "end {id}"
            );
            assert!(
                xml.contains(&format!(r#"<w:commentReference w:id="{id}""#)),
                "ref {id}"
            );
        }
        let comments = part_string(bytes, "word/comments.xml").unwrap();
        assert!(comments.contains("Courier only for indemnity notices."));
        assert!(comments.contains(r#"w:author="Claude""#));
        assert_word_valid_package(bytes);
    }
    // The comment on the replacement anchors the inserted run in the redline.
    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    let start = redline_xml
        .find(r#"<w:commentRangeStart w:id="1""#)
        .unwrap();
    let end = redline_xml.find(r#"<w:commentRangeEnd w:id="1""#).unwrap();
    let anchored = &redline_xml[start..end];
    assert!(anchored.contains("<w:ins "), "{anchored}");
    assert!(
        anchored.contains("email or courier") || anchored.contains("or courier"),
        "{anchored}"
    );
    // The source-text comment anchors exactly its text in the clean copy.
    let clean_xml = part_string(&result.clean, "word/document.xml").unwrap();
    let start = clean_xml.find(r#"<w:commentRangeStart w:id="0""#).unwrap();
    let end = clean_xml.find(r#"<w:commentRangeEnd w:id="0""#).unwrap();
    assert!(clean_xml[start..end].contains("Sections 1(g), "));
    assert!(!clean_xml[start..end].contains("2(e)"));
}

#[test]
fn edits_crossing_tabs_hyperlinks_or_fields_are_unsupported_structure() {
    let body = r#"<w:p><w:r><w:t>Name:</w:t><w:tab/><w:t>Arthur</w:t></w:r></w:p><w:p><w:hyperlink r:id="rId9"><w:r><w:t>arthur.law</w:t></w:r></w:hyperlink><w:r><w:t xml:space="preserve"> site</w:t></w:r></w:p>"#;
    let source = docx(body);
    for ops in [
        r#"[{"kind":"replace","paragraph":{"index":0},"find":"Name:\tArthur","replacement":"x"}]"#,
        r#"[{"kind":"replace","paragraph":{"index":1},"find":"arthur.law","replacement":"x"}]"#,
    ] {
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert_eq!(err.code, "UNSUPPORTED_STRUCTURE", "{ops}");
    }
    // Editing beside the tab is fine.
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"Arthur","replacement":"Souza"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean)[0], "Name:\tSouza");
}

#[test]
fn preview_reports_without_producing_documents() {
    let source = docx(&para("alpha beta"));
    let report = preview_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"beta","replacement":"gamma"}]"#,
        ),
    )
    .unwrap();
    assert!(report.ok);
    assert_eq!(
        report.operations[0].context.as_deref(),
        Some("alpha {beta→gamma}")
    );
    assert_eq!(report.revisions.total, 0, "no compare ran");
}

#[test]
fn macro_packages_are_refused() {
    let mut source = docx(&para("x"));
    // Append a vbaProject part by rebuilding the zip with the extra entry.
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(source.clone())).unwrap();
    let mut out = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).unwrap();
        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut f, &mut data).unwrap();
        out.start_file(
            f.name().to_string(),
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        std::io::Write::write_all(&mut out, &data).unwrap();
    }
    out.start_file(
        "word/vbaProject.bin",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    std::io::Write::write_all(&mut out, b"\x01").unwrap();
    source = out.finish().unwrap().into_inner();
    let err = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"delete","paragraph":{"index":0},"find":"x"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "UNSUPPORTED_PACKAGE");
}

#[test]
fn admission_refusals_keep_their_code_through_inspect_and_edit() {
    // A second copy of the main part under another case: one OPC part name.
    let mut out = zip::ZipWriter::new_append(std::io::Cursor::new(docx(&para("x")))).unwrap();
    out.start_file(
        "Word/Document.xml",
        zip::write::SimpleFileOptions::default(),
    )
    .unwrap();
    std::io::Write::write_all(&mut out, b"<w:document/>").unwrap();
    let source = out.finish().unwrap().into_inner();

    let refused = paragraphs(&source).unwrap_err();
    assert!(
        matches!(
            &refused,
            jubarte::inspect::InspectError::Admission(a) if a.code() == "DUPLICATE_PART"
        ),
        "{refused}"
    );
    let err = preview_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"delete","paragraph":{"index":0},"find":"x"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(err.code, "DUPLICATE_PART");
}

#[test]
fn report_serializes_to_json_lines_an_agent_can_log() {
    let source = docx(&para("alpha beta"));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"beta","replacement":"gamma"}]"#,
        ),
    )
    .unwrap();
    let jsonl = result.report.to_jsonl();
    let lines: Vec<serde_json::Value> = jsonl
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert_eq!(lines[0]["ev"], "load");
    assert_eq!(lines[0]["paras"], 1);
    assert_eq!(lines[1]["ev"], "op");
    assert_eq!(lines[1]["status"], "ok");
    assert_eq!(lines[1]["at"], "body:p:0");
    assert_eq!(lines.last().unwrap()["ev"], "summary");
    assert_eq!(lines.last().unwrap()["status"], "ok");
}

#[test]
fn unicode_replacement_across_runs_preserves_surrounding_text_and_formatting() {
    let source = docx(&format!(
        "<w:p>{}{}{}</w:p>",
        run("Pré 😀 ca", true, false, None),
        run("fé fin", false, true, None),
        run(" 尾", false, false, None),
    ));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[
        {"kind":"replace","paragraph":{"index":0},"find":"café","replacement":"茶 & <tea>"},
        {"kind":"insert","paragraph":{"index":0},"before":"尾","text":"新"}
    ]"#,
        ),
    )
    .unwrap();
    let clean = paragraphs(&result.clean).unwrap();
    assert_eq!(clean[0].text, "Pré 😀 茶 & <tea> fin 新尾");
    let slices: Vec<_> = clean[0]
        .runs
        .iter()
        .map(|span| {
            let text: String = clean[0]
                .text
                .chars()
                .skip(span.start)
                .take(span.end - span.start)
                .collect();
            (text, span.bold, span.italic)
        })
        .collect();
    assert_eq!(
        slices,
        [
            ("Pré 😀 茶 & <tea>".to_string(), true, false),
            (" fin".to_string(), false, true),
            (" 新尾".to_string(), false, false),
        ]
    );
    assert_eq!(
        texts(&accept_revisions(&result.redline).unwrap()),
        texts(&result.clean)
    );
    assert_eq!(
        texts(&jubarte::document_comparer::reject_revisions(&result.redline).unwrap()),
        texts(&source)
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}

#[test]
fn operations_resolve_against_the_source_even_when_prior_replacements_change_length() {
    let source = docx(&para("alpha beta gamma"));
    let result = apply_plan(&source, &plan(&source, r#"[
        {"kind":"replace","paragraph":{"index":0},"find":"alpha","replacement":"a much longer prefix"},
        {"kind":"replace","paragraph":{"index":0},"find":"gamma","replacement":"終"},
        {"kind":"delete","paragraph":{"index":0},"find":"beta "}
    ]"#)).unwrap();
    assert_eq!(texts(&result.clean), ["a much longer prefix 終"]);
    let invalid = plan(
        &source,
        r#"[
        {"id":"create","kind":"replace","paragraph":{"index":0},"find":"alpha","replacement":"new anchor"},
        {"id":"reuse","kind":"delete","paragraph":{"index":0},"find":"new anchor"}
    ]"#,
    );
    let error = apply_plan(&source, &invalid).unwrap_err();
    assert_eq!(error.code, "ANCHOR_NOT_FOUND");
    assert_eq!(error.operation.as_deref(), Some("reuse"));
    assert_eq!(error.outcomes[0].status, "ok");
    assert_eq!(error.outcomes[1].matches, 0);
}

#[test]
fn adjacent_replacements_are_allowed_but_an_insertion_inside_a_replacement_is_refused() {
    let source = docx(&para("abcdef"));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[
        {"kind":"replace","paragraph":{"index":0},"find":"abc","replacement":"X"},
        {"kind":"replace","paragraph":{"index":0},"find":"def","replacement":"Y"}
    ]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), ["XY"]);
    let error = preview_plan(
        &source,
        &plan(
            &source,
            r#"[
        {"kind":"replace","paragraph":{"index":0},"find":"abc","replacement":"X"},
        {"id":"inside","kind":"insert","paragraph":{"index":0},"after":"a","text":"!"}
    ]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(error.code, "OVERLAPPING_EDITS");
    assert_eq!(error.operation.as_deref(), Some("inside"));
}

#[test]
fn simple_field_and_content_control_anchors_are_refused_by_preview_and_apply() {
    for inner in [
        r#"<w:fldSimple w:instr="PAGE"><w:r><w:t>target</w:t></w:r></w:fldSimple>"#,
        r#"<w:sdt><w:sdtContent><w:r><w:t>target</w:t></w:r></w:sdtContent></w:sdt>"#,
    ] {
        let source = docx(&format!("<w:p>{inner}</w:p>"));
        let edit = plan(
            &source,
            r#"[{"id":"opaque","kind":"replace","paragraph":{"index":0},"find":"target","replacement":"new"}]"#,
        );
        for error in [
            preview_plan(&source, &edit).unwrap_err(),
            apply_plan(&source, &edit).unwrap_err(),
        ] {
            assert_eq!(error.code, "UNSUPPORTED_STRUCTURE", "{inner}");
            assert_eq!(error.operation.as_deref(), Some("opaque"));
        }
    }
}

#[test]
fn insertion_requires_one_position_and_nonempty_plain_text() {
    let source = docx(&para("anchor"));
    for extra in [
        serde_json::json!({"text":"x"}),
        serde_json::json!({"after":"anchor","before":"anchor","text":"x"}),
        serde_json::json!({"position":"start","after":"anchor","text":"x"}),
        serde_json::json!({"position":"end","text":""}),
        serde_json::json!({"position":"end","text":"a\nb"}),
        serde_json::json!({"position":"end","text":"a\tb"}),
    ] {
        let mut op = extra;
        op["kind"] = serde_json::json!("insert");
        op["paragraph"] = serde_json::json!({"index":0});
        let ops = serde_json::json!([op]).to_string();
        let edit = plan(&source, &ops);
        assert_eq!(
            preview_plan(&source, &edit).unwrap_err().code,
            "INVALID_EDIT",
            "{ops}"
        );
    }
}

#[test]
fn new_comments_preserve_existing_comments_and_allocate_after_the_highest_id() {
    let comments = format!(
        r#"<w:comments xmlns:w="{}"><w:comment w:id="7" w:author="Original" w:date="2020-01-01T00:00:00Z">{}</w:comment></w:comments>"#,
        common::docx::W_NS,
        para("Existing note"),
    );
    let source = common::docx::docx_with(
        &para("Contract text"),
        &[common::docx::Part {
            name: "word/reviewer-notes.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    );
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[
        {"kind":"comment","paragraph":{"index":0},"find":"Contract","text":"New note"}
    ]"#,
        ),
    )
    .unwrap();
    assert_eq!(result.report.comments_added, 1);
    assert_eq!(result.report.operations[0].comment_id, Some(8));
    assert_eq!(summary(&result.clean).unwrap().comments, 2);
    let xml = part_string(&result.clean, "word/reviewer-notes.xml").unwrap();
    let mut dom = jubarte::xmllinq::Dom::new();
    let document = jubarte::xmllinq::parse::parse_xdocument(&mut dom, &xml);
    let root = dom.root(document).unwrap();
    let nodes = dom.descendants(root, Some(&jubarte::namespaces::W::name("comment")));
    let ids: Vec<_> = nodes
        .iter()
        .map(|&node| dom.attribute(node, &jubarte::namespaces::W::id()).unwrap())
        .collect();
    assert_eq!(ids, ["7", "8"]);
    assert!(xml.contains("Existing note") && xml.contains("New note"));
    assert_eq!(texts(&result.clean), texts(&source));
}

#[test]
fn edits_crossing_complex_field_boundaries_are_refused() {
    // fldChar markers are siblings of the result runs, unlike fldSimple.
    // Replacing across them must not silently rewrite a generated field result.
    let source = docx(
        r#"<w:p><w:r><w:t xml:space="preserve">Before </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>3</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t xml:space="preserve"> after</w:t></w:r></w:p>"#,
    );
    let edit = plan(
        &source,
        r#"[{"id":"field","kind":"replace","paragraph":{"index":0},"find":"Before 3 after","replacement":"New text"}]"#,
    );
    let preview = preview_plan(&source, &edit);
    let applied = apply_plan(&source, &edit);
    assert!(
        preview.is_err() && applied.is_err(),
        "both preview and apply must refuse edits crossing a complex field; preview refused: {}, apply refused: {}",
        preview.is_err(),
        applied.is_err()
    );
    for error in [preview.unwrap_err(), applied.unwrap_err()] {
        assert_eq!(error.code, "UNSUPPORTED_STRUCTURE");
        assert_eq!(error.operation.as_deref(), Some("field"));
    }
}

#[test]
fn edits_crossing_an_empty_complex_field_are_refused_but_text_beside_it_stays_editable() {
    // A field with no result leaves no text of its own, so the words on
    // either side meet in the projection; the markers still sit between them.
    let source = docx(
        r#"<w:p><w:r><w:t xml:space="preserve">Before </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t xml:space="preserve"> after</w:t></w:r></w:p>"#,
    );
    let across = plan(
        &source,
        r#"[{"id":"field","kind":"replace","paragraph":{"index":0},"find":"Before  after","replacement":"New text"}]"#,
    );
    for error in [
        preview_plan(&source, &across).unwrap_err(),
        apply_plan(&source, &across).unwrap_err(),
    ] {
        assert_eq!(error.code, "UNSUPPORTED_STRUCTURE");
        assert_eq!(error.operation.as_deref(), Some("field"));
    }
    let beside = plan(
        &source,
        r#"[{"id":"word","kind":"replace","paragraph":{"index":0},"find":"Before","replacement":"Prior"}]"#,
    );
    let result = apply_plan(&source, &beside).unwrap();
    assert!(texts(&result.clean).iter().any(|t| t.contains("Prior")));
}

/// Visible text between `commentRangeStart` and `commentRangeEnd` of `id`.
fn commented_text(xml: &str, id: u32) -> String {
    let start = xml
        .find(&format!(r#"<w:commentRangeStart w:id="{id}""#))
        .expect("range start");
    let end = xml
        .find(&format!(r#"<w:commentRangeEnd w:id="{id}""#))
        .expect("range end");
    let mut text = String::new();
    let mut in_tag = false;
    for c in xml[start..end].chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => text.push(c),
            _ => {}
        }
    }
    text
}

#[test]
fn deletions_are_checked_together_so_a_cell_keeps_a_closing_paragraph() {
    // Two paragraphs in one cell: each deletion alone is fine, both empty it.
    let table = format!(
        "<w:tbl><w:tr><w:tc>{}{}</w:tc></w:tr></w:tbl>{}",
        para("cell one"),
        para("cell two"),
        para("after")
    );
    let source = docx(&table);
    let both = r#"[{"kind":"delete_paragraph","paragraph":{"index":0}},
                   {"kind":"delete_paragraph","paragraph":{"index":1}}]"#;
    let err = apply_plan(&source, &plan(&source, both)).unwrap_err();
    assert_eq!(err.code, "OVERLAPPING_EDITS");
    let one = r#"[{"kind":"delete_paragraph","paragraph":{"index":1}}]"#;
    assert!(apply_plan(&source, &plan(&source, one)).is_ok());

    // The same paragraph twice would report two deletions of one paragraph.
    let twice = r#"[{"kind":"delete_paragraph","paragraph":{"index":2}},
                    {"kind":"delete_paragraph","paragraph":{"index":2}}]"#;
    let source2 = docx(&(para("keep") + &para("x") + &para("gone")));
    assert_eq!(
        apply_plan(&source2, &plan(&source2, twice))
            .unwrap_err()
            .code,
        "OVERLAPPING_EDITS"
    );

    // A cell ending in a nested table after its last paragraph goes: Word
    // requires the cell to close with a paragraph.
    let nested = format!(
        "<w:tbl><w:tr><w:tc>{}<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>{}</w:tc></w:tr></w:tbl>{}",
        para("outer first"),
        para("inner"),
        para("outer last"),
        para("after")
    );
    let source3 = docx(&nested);
    let last = r#"[{"kind":"delete_paragraph","paragraph":{"index":2}}]"#;
    assert_eq!(
        apply_plan(&source3, &plan(&source3, last))
            .unwrap_err()
            .code,
        "OVERLAPPING_EDITS"
    );
    let first = r#"[{"kind":"delete_paragraph","paragraph":{"index":0}}]"#;
    let kept = apply_plan(&source3, &plan(&source3, first)).unwrap();
    assert_word_valid_package(&kept.clean);
}

#[test]
fn a_comment_may_contain_or_touch_an_edit_but_not_cut_through_one() {
    let source = docx(&para("retained experts and process servers"));
    let cut = r#"[{"kind":"comment","paragraph":{"index":0},"find":"experts and","text":"n"},
                  {"kind":"replace","paragraph":{"index":0},"find":"and process","replacement":"x"}]"#;
    assert_eq!(
        apply_plan(&source, &plan(&source, cut)).unwrap_err().code,
        "OVERLAPPING_EDITS"
    );
    let contains = r#"[{"kind":"comment","paragraph":{"index":0},"find":"experts and process","text":"n"},
                       {"kind":"replace","paragraph":{"index":0},"find":"and","replacement":"or"}]"#;
    let result = apply_plan(&source, &plan(&source, contains)).unwrap();
    let xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert_eq!(commented_text(&xml, 0), "experts or process");
    let touches = r#"[{"kind":"comment","paragraph":{"index":0},"find":"experts","text":"n"},
                      {"kind":"replace","paragraph":{"index":0},"find":" and","replacement":","}]"#;
    let result = apply_plan(&source, &plan(&source, touches)).unwrap();
    let xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert_eq!(commented_text(&xml, 0), "experts");
}

#[test]
fn a_comment_on_the_first_of_two_inserts_at_one_point_anchors_its_own_text() {
    let source = docx(&para("Sections 1(g), 2(e), 3 survive."));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert","paragraph":{"index":0},"after":"1(g), ","text":"2(c), ","comment":"first"},
                {"kind":"insert","paragraph":{"index":0},"after":"1(g), ","text":"2(d), ","comment":"second"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean)[0],
        "Sections 1(g), 2(c), 2(d), 2(e), 3 survive."
    );
    let xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert_eq!(commented_text(&xml, 0), "2(c), ");
    assert_eq!(commented_text(&xml, 1), "2(d), ");
}

#[test]
fn comment_ids_at_the_top_of_the_range_refuse_new_comments_only() {
    let comments = format!(
        r#"<w:comments xmlns:w="{}"><w:comment w:id="4294967295" w:author="O" w:date="2020-01-01T00:00:00Z">{}</w:comment></w:comments>"#,
        common::docx::W_NS,
        para("Existing"),
    );
    let source = common::docx::docx_with(
        &para("Contract text"),
        &[common::docx::Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    );
    let note = r#"[{"kind":"comment","paragraph":{"index":0},"find":"Contract","text":"n"}]"#;
    assert_eq!(
        preview_plan(&source, &plan(&source, note))
            .unwrap_err()
            .code,
        "INVALID_DOCUMENT"
    );
    let plain =
        r#"[{"kind":"replace","paragraph":{"index":0},"find":"text","replacement":"terms"}]"#;
    assert!(apply_plan(&source, &plan(&source, plain)).is_ok());
}

/// The span of `paragraph`'s run formatting that covers char `at`.
fn span_at(bytes: &[u8], paragraph: usize, at: usize) -> jubarte::inspect::Span {
    paragraphs(bytes).unwrap()[paragraph]
        .runs
        .iter()
        .find(|s| s.start <= at && at < s.end)
        .cloned()
        .unwrap_or_else(|| panic!("no span covers char {at}"))
}

#[test]
fn insert_and_replace_format_only_their_new_text() {
    let source = docx(&para("The fee is ten dollars per month."));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"ten","replacement":"twenty","format":{"bold":true}},
                {"kind":"insert","paragraph":{"index":0},"after":"fee","text":" (net)","format":{"italic":true,"highlight":"yellow"}}]"#,
        ),
    )
    .unwrap();
    let text = &texts(&result.clean)[0];
    assert_eq!(text, "The fee (net) is twenty dollars per month.");
    let at = |needle: &str| text.find(needle).unwrap();
    let twenty = span_at(&result.clean, 0, at("twenty"));
    assert!(twenty.bold && !twenty.italic, "{twenty:?}");
    assert_eq!(&text[twenty.start..twenty.end], "twenty");
    let net = span_at(&result.clean, 0, at("(net)"));
    assert!(net.italic && !net.bold, "{net:?}");
    assert_eq!(net.highlight.as_deref(), Some("yellow"));
    assert_eq!(&text[net.start..net.end], " (net)");
    for plain in ["The fee", " is ", " dollars"] {
        let s = span_at(&result.clean, 0, at(plain));
        assert!(
            !s.bold && !s.italic && s.highlight.is_none(),
            "{plain}: {s:?}"
        );
    }
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), texts(&result.clean));
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_eq!(texts(&rejected), texts(&source));
    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(redline_xml.contains("<w:b />") || redline_xml.contains("<w:b/>"));
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}

#[test]
fn a_format_needs_a_word_highlight_colour_and_text() {
    let source = docx(&para("Plain words here."));
    for ops in [
        r#"[{"kind":"insert","paragraph":{"index":0},"after":"Plain","text":"!","format":{"highlight":"yelow"}}]"#,
        r#"[{"kind":"replace","paragraph":{"index":0},"find":"words","replacement":"","format":{"bold":true}}]"#,
        r#"[{"kind":"insert_paragraph","paragraph":{"index":0},"runs":[{"text":"x","highlight":"neon"}]}]"#,
    ] {
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert_eq!(err.code, "INVALID_EDIT", "{ops}");
    }
    let json = r#"{"schema_version":1,"author":"a","operations":[{"kind":"insert","paragraph":{"index":0},"after":"Plain","text":"!","format":{"bold":true,"size":12}}]}"#;
    assert_eq!(EditPlan::from_json(json).unwrap_err().code, "INVALID_PLAN");
}

const STYLES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:rPr><w:b/></w:rPr></w:style><w:style w:type="character" w:styleId="Strong"><w:name w:val="Strong"/></w:style></w:styles>"#;

fn styled_docx(body: &str) -> Vec<u8> {
    docx_with(
        body,
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: STYLES_XML,
        }],
    )
}

#[test]
fn format_paragraph_sets_style_and_alignment_and_the_redline_keeps_the_old_ones() {
    let source = styled_docx(&(para("Master Services Agreement") + &para("Body text.")));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"format_paragraph","paragraph":{"index":0},"style":"heading 1","alignment":"center"}]"#,
        ),
    )
    .unwrap();
    let paras = paragraphs(&result.clean).unwrap();
    assert_eq!(
        paras[0].style.as_deref(),
        Some("Heading1"),
        "name resolves to id"
    );
    assert_eq!(paras[1].style, None);
    let clean_xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert!(clean_xml.contains(r#"<w:jc w:val="center""#), "{clean_xml}");
    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(
        redline_xml.contains("<w:pPrChange"),
        "old properties recorded"
    );
    assert_eq!(result.report.revisions.format_changed, 1);
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_eq!(paragraphs(&rejected).unwrap()[0].style, None);
    assert!(
        !part_string(&rejected, "word/document.xml")
            .unwrap()
            .contains("<w:jc")
    );
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(
        paragraphs(&accepted).unwrap()[0].style.as_deref(),
        Some("Heading1")
    );
    assert_eq!(
        result.report.operations[0].context.as_deref(),
        Some("{¶ style=Heading1 alignment=center} Master Services Agreement")
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}

#[test]
fn format_paragraph_refuses_unknown_styles_and_empty_formats() {
    let source = styled_docx(&(para("Title") + &para("Body")));
    for (ops, code) in [
        (
            r#"[{"kind":"format_paragraph","paragraph":{"index":0},"style":"Heading 9"}]"#,
            "UNKNOWN_STYLE",
        ),
        (
            r#"[{"kind":"format_paragraph","paragraph":{"index":0},"style":"Strong"}]"#,
            "UNKNOWN_STYLE",
        ),
        (
            r#"[{"kind":"insert_paragraph","paragraph":{"index":0},"style":"Nope","runs":[{"text":"x"}]}]"#,
            "UNKNOWN_STYLE",
        ),
        (
            r#"[{"kind":"format_paragraph","paragraph":{"index":0}}]"#,
            "INVALID_EDIT",
        ),
        (
            r#"[{"kind":"format_paragraph","paragraph":{"index":0},"alignment":"center"},
             {"kind":"format_paragraph","paragraph":{"index":0},"style":"Heading1"}]"#,
            "OVERLAPPING_EDITS",
        ),
    ] {
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert_eq!(err.code, code, "{ops}");
    }
    let err = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"format_paragraph","paragraph":{"index":0},"style":"Heading 9"}]"#,
        ),
    )
    .unwrap_err();
    assert!(
        err.message.contains("Heading1"),
        "lists the defined styles: {}",
        err.message
    );
}

#[test]
fn merge_paragraphs_joins_the_next_paragraph_and_the_redline_deletes_the_mark() {
    let source = docx(
        &(r#"<w:p><w:pPr><w:keepNext/></w:pPr><w:r><w:t>1. The Supplier shall deliver the Goods.</w:t></w:r></w:p>"#
            .to_string()
            + &para("Delivery is DDP to the Buyer's site.")
            + &para("2. Price.")),
    );
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0},"separator":" "}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean),
        [
            "1. The Supplier shall deliver the Goods. Delivery is DDP to the Buyer's site.",
            "2. Price."
        ]
    );
    assert_eq!(result.report.paragraphs.from, 3);
    assert_eq!(result.report.paragraphs.to, 2);
    assert_eq!(
        result.report.operations[0].context.as_deref(),
        Some("l deliver the Goods.{¶→ }Delivery is DDP to t…")
    );
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), texts(&result.clean));
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_eq!(texts(&rejected), texts(&source));
    // The second paragraph's mark survives, as when Word accepts a deleted
    // mark: the first's keepNext goes, and no property change is recorded.
    let clean_xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert!(!clean_xml.contains("keepNext"), "{clean_xml}");
    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    assert!(!redline_xml.contains("pPrChange"), "{redline_xml}");
    assert_eq!(result.report.revisions.format_changed, 0);
    assert!(
        !part_string(&accepted, "word/document.xml")
            .unwrap()
            .contains("keepNext")
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}

#[test]
fn merging_into_a_section_break_keeps_the_break() {
    let body = format!(
        "{}<w:p><w:pPr><w:sectPr/></w:pPr><w:r><w:t>end of section</w:t></w:r></w:p>{}",
        para("start"),
        para("next section")
    );
    let source = docx(&body);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0},"separator":" "}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean),
        ["start end of section", "next section"]
    );
    let clean_xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert_eq!(clean_xml.matches("<w:sectPr").count(), 2, "{clean_xml}");
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_eq!(texts(&rejected), texts(&source));
    assert_word_valid_package(&result.redline);
}

#[test]
fn merges_chain_and_carry_range_markup_to_the_join() {
    let body = format!(
        r#"{}<w:bookmarkStart w:id="7" w:name="clause"/>{}<w:bookmarkEnd w:id="7"/>{}{}"#,
        para("a"),
        para("b"),
        para("c"),
        para("d")
    );
    let source = docx(&body);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0},"separator":"-"},
                {"kind":"merge_paragraphs","paragraph":{"index":1},"separator":"+"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), ["a-b+c", "d"]);
    let clean_xml = part_string(&result.clean, "word/document.xml").unwrap();
    let merged = &clean_xml[clean_xml.find("<w:p>").unwrap()..clean_xml.find("</w:p>").unwrap()];
    assert!(
        merged.contains("bookmarkStart") && merged.contains("bookmarkEnd"),
        "the bookmark moves into the merged paragraph: {merged}"
    );
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_eq!(texts(&rejected), ["a", "b", "c", "d"]);
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
}

#[test]
fn merge_paragraphs_refuses_what_it_cannot_join() {
    let table = format!(
        "{}<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>{}",
        para("before table"),
        para("cell"),
        para("after")
    );
    let section = format!(
        "{}<w:p><w:pPr><w:sectPr/></w:pPr><w:r><w:t>end of section</w:t></w:r></w:p>{}",
        para("start"),
        para("next section")
    );
    let three = para("one") + &para("two") + &para("three");
    for (body, ops, code) in [
        (
            three.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":2}}]"#,
            "UNSUPPORTED_STRUCTURE",
        ),
        (
            table.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0}}]"#,
            "UNSUPPORTED_STRUCTURE",
        ),
        (
            table.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":1}}]"#,
            "UNSUPPORTED_STRUCTURE",
        ),
        (
            section.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":1}}]"#,
            "UNSUPPORTED_STRUCTURE",
        ),
        (
            three.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0},"separator":"\t"}]"#,
            "INVALID_EDIT",
        ),
        (
            three.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0}},{"kind":"delete_paragraph","paragraph":{"index":1}}]"#,
            "OVERLAPPING_EDITS",
        ),
        (
            three.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0}},{"kind":"merge_paragraphs","paragraph":{"index":0}}]"#,
            "OVERLAPPING_EDITS",
        ),
        (
            three.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0}},{"kind":"insert_paragraph","paragraph":{"index":0},"runs":[{"text":"x"}]}]"#,
            "OVERLAPPING_EDITS",
        ),
        (
            three.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0}},{"kind":"format_paragraph","paragraph":{"index":0},"alignment":"right"}]"#,
            "OVERLAPPING_EDITS",
        ),
        (
            three.as_str(),
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0}},{"kind":"insert_paragraph","paragraph":{"index":1},"position":"before","runs":[{"text":"x"}]}]"#,
            "OVERLAPPING_EDITS",
        ),
    ] {
        let source = docx(body);
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert_eq!(err.code, code, "{ops}");
    }
    // Text edits on either paragraph, formatting the surviving one and
    // insertions outside the pair are fine.
    let source = docx(&three);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0},"separator":" "},
                {"kind":"replace","paragraph":{"index":1},"find":"two","replacement":"TWO","format":{"bold":true}},
                {"kind":"format_paragraph","paragraph":{"index":1},"alignment":"right"},
                {"kind":"insert_paragraph","paragraph":{"index":0},"position":"before","runs":[{"text":"zero"}]},
                {"kind":"insert_paragraph","paragraph":{"index":1},"position":"after","runs":[{"text":"two and a half"}]}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean),
        ["zero", "one TWO", "two and a half", "three"]
    );
    assert_word_valid_package(&result.redline);
}

#[test]
fn format_paragraph_sets_spacing_in_points_and_multiples() {
    let body = format!(
        r#"<w:p><w:pPr><w:spacing w:before="100" w:beforeAutospacing="1" w:after="200"/></w:pPr><w:r><w:t>Drafting note: confirm the notice address.</w:t></w:r></w:p>{}"#,
        para("Body.")
    );
    let source = docx(&body);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"format_paragraph","paragraph":{"starts_with":"Drafting note"},"line_spacing":1.15,"space_before":6,"space_after":0}]"#,
        ),
    )
    .unwrap();
    let clean_xml = part_string(&result.clean, "word/document.xml").unwrap();
    let spacing = &clean_xml[clean_xml.find("<w:spacing").unwrap()..];
    let spacing = &spacing[..spacing.find("/>").unwrap()];
    for attr in [
        r#"w:before="120""#,
        r#"w:after="0""#,
        r#"w:line="276""#,
        r#"w:lineRule="auto""#,
    ] {
        assert!(spacing.contains(attr), "{attr} in {spacing}");
    }
    assert!(
        !spacing.contains("Autospacing"),
        "autospacing would override: {spacing}"
    );
    assert_eq!(
        result.report.operations[0].context.as_deref(),
        Some(
            "{¶ line_spacing=1.15 space_before=6pt space_after=0pt} Drafting note: confirm the notice addres…"
        )
    );
    let rejected = reject_revisions(&result.redline).unwrap();
    let rejected_xml = part_string(&rejected, "word/document.xml").unwrap();
    assert!(
        rejected_xml.contains(r#"w:beforeAutospacing="1""#),
        "reject restores the old spacing"
    );
    assert_word_valid_package(&result.redline);
    for bad in [
        r#""line_spacing":0"#,
        r#""line_spacing":11"#,
        r#""space_after":-1"#,
    ] {
        let json = format!(
            r#"{{"schema_version":1,"author":"a","operations":[{{"kind":"format_paragraph","paragraph":{{"index":0}},{bad}}}]}}"#
        );
        assert_eq!(
            EditPlan::from_json(&json).unwrap_err().code,
            "INVALID_PLAN",
            "{bad}"
        );
    }
    let json = r#"{"schema_version":1,"author":"a","operations":[{"kind":"format_paragraph","paragraph":{"index":0},"line_spacing":1.15,"space_after":7.5}]}"#;
    let parsed = EditPlan::from_json(json).unwrap();
    assert_eq!(EditPlan::from_json(&parsed.to_json()).unwrap(), parsed);
}
