// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::edit`: a plan of uniquely anchored operations is applied to a
//! copy, the comparer produces the redline, and every operation gets a report
//! line. The scenarios mirror what an agent hand-rolled over `document.xml`
//! (redline.py): replace a phrase, insert after an anchor, delete a paragraph,
//! insert a bold-headed paragraph, comment on inserted text.

mod common;

use common::docx::{docx, para, part_string, run};
use common::validity::assert_word_valid_package;
use jubarte::document_comparer::accept_revisions;
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
            r#"[{"kind":"merge_paragraphs","paragraph":{"index":0}}]"#,
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

#[test]
fn insert_paragraph_copies_anchor_properties_and_sets_run_formatting() {
    let body = format!(
        r#"<w:p><w:pPr><w:pStyle w:val="Sub"/><w:ind w:left="720" w:hanging="360"/><w:sectPr/></w:pPr>{}{}</w:p>"#,
        run("(f) ", false, false, None),
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
    assert!(new.runs.len() == 3 && new.runs[1].bold && !new.runs[0].bold);
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
