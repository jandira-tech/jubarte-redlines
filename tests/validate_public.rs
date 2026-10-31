// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::validate`: Ring-1 invariants as data, with repair and the
//! tracked-edit audit.

mod common;

use common::docx::{Part, docx, docx_with_sect, para, part_string};
use jubarte::document_comparer::compare_documents;
use jubarte::inspect::markdown;
use jubarte::validate::{audit_tracked, repair, validate};

#[test]
fn text_inside_a_deletion_is_word_fatal_and_repairable() {
    let body = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    let f = findings
        .iter()
        .find(|f| f.code == "TEXT_INSIDE_DELETION")
        .expect("finding");
    assert!(f.word_fatal && f.repairable);
    assert_eq!(f.part, "word/document.xml");
    assert!(f.location.starts_with("w:body[0]/w:p[0]"), "{}", f.location);

    let fixed = repair(&docx(body)).unwrap();
    assert_eq!(fixed.repaired.len(), 1, "{:?}", fixed.repaired);
    assert!(fixed.remaining.is_empty(), "{:?}", fixed.remaining);
    assert!(validate(&fixed.docx).unwrap().is_empty());
    assert!(
        part_string(&fixed.docx, "word/document.xml")
            .unwrap()
            .contains("<w:delText>gone</w:delText>")
    );
}

#[test]
fn an_unbound_mc_requires_prefix_is_word_fatal_and_repairable() {
    let body = r#"<w:p><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:t>x</w:t></mc:Choice><mc:Fallback><w:t>x</w:t></mc:Fallback></mc:AlternateContent></w:r></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.code == "MC_UNBOUND_PREFIX" && f.word_fatal && f.message.contains("wps")),
        "{findings:?}"
    );
    let fixed = repair(&docx(body)).unwrap();
    assert!(fixed.repaired.iter().any(|f| f.code == "MC_UNBOUND_PREFIX"));
    assert!(
        validate(&fixed.docx).unwrap().is_empty(),
        "{:?}",
        fixed.remaining
    );
}

#[test]
fn a_clean_compare_output_validates_empty() {
    let redline =
        compare_documents(&docx(&para("one two")), &docx(&para("one three")), "R").unwrap();
    assert_eq!(validate(&redline).unwrap(), Vec::new());
}

#[test]
fn findings_print_in_one_order_every_run() {
    // received.docx has two comment-metadata findings at one location; they
    // come from hash sets, so only a total sort keeps their order.
    let docx = std::fs::read("tests/fixtures/agent-view/received.docx").unwrap();
    let findings = validate(&docx).unwrap();
    let key = |f: &jubarte::validate::Finding| {
        (
            f.part.clone(),
            f.location.clone(),
            f.code.clone(),
            f.message.clone(),
        )
    };
    let keys: Vec<_> = findings.iter().map(key).collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
    assert!(
        keys.windows(2)
            .any(|w| (&w[0].0, &w[0].1) == (&w[1].0, &w[1].1)),
        "the fixture still has findings that tie on part and location"
    );
}

#[test]
fn findings_are_json_with_stable_field_names() {
    let body = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    let json: serde_json::Value = serde_json::to_value(&findings).unwrap();
    let first = &json[0];
    let keys: Vec<&str> = first
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    // serde_json's map sorts keys.
    assert_eq!(
        keys,
        [
            "code",
            "location",
            "message",
            "part",
            "ref",
            "repairable",
            "set",
            "severity",
            "word_fatal"
        ]
    );
    assert_eq!(
        (&first["set"], &first["severity"]),
        (&"word".into(), &"error".into())
    );
}

#[test]
fn an_unreadable_package_is_an_error_not_a_finding() {
    assert!(validate(b"not a zip").is_err());
    assert!(repair(b"not a zip").is_err());
}

#[test]
fn the_audit_names_an_untracked_edit_by_paragraph() {
    let original = docx(&(para("Fee is 10.") + &para("Term is 2 years.")));
    // A hand edit: the second paragraph changed with no w:ins/w:del at all.
    let edited = docx(&(para("Fee is 10.") + &para("Term is 3 years.")));
    let findings = audit_tracked(&original, &edited, "Reviewer").unwrap();
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert_eq!(findings[0].code, "UNTRACKED_EDIT");
    // The location keeps the long id; the printed message names the short one.
    assert_eq!(findings[0].location, "p1");
    assert_eq!(findings[0].r#ref, "p1");
    assert!(
        findings[0].message.starts_with("p1 differs") && !findings[0].message.contains(":p:"),
        "{}",
        findings[0].message
    );
    // The same edit done by the engine is fully tracked.
    let tracked = compare_documents(&original, &edited, "Reviewer").unwrap();
    assert!(
        audit_tracked(&original, &tracked, "Reviewer")
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        markdown(&tracked)
            .unwrap()
            .lines()
            .filter(|l| l.starts_with("[p"))
            .count(),
        2
    );
}

#[test]
fn the_audit_reports_another_authors_changes() {
    let original = docx(&para("Fee is 10."));
    let edited = docx(&para("Fee is 12."));
    let tracked = compare_documents(&original, &edited, "Someone Else").unwrap();
    let findings = audit_tracked(&original, &tracked, "Reviewer").unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.code == "FOREIGN_AUTHOR" && f.message.contains("Someone Else")),
        "{findings:?}"
    );
}

// ── Repair probes: each repairable code is fixed and nothing else breaks ──

const DEL_OPEN: &str = r#"<w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z">"#;

fn codes(findings: &[jubarte::validate::Finding]) -> Vec<&str> {
    findings.iter().map(|f| f.code.as_str()).collect()
}

fn assert_repairs(body: &str, code: &str) -> Vec<u8> {
    let input = docx(body);
    let findings = validate(&input).unwrap();
    let f = findings
        .iter()
        .find(|f| f.code == code)
        .unwrap_or_else(|| panic!("{code} not found in {findings:?}"));
    assert!(f.repairable, "{f:?}");
    let fixed = repair(&input).unwrap();
    assert!(
        fixed.repaired.iter().any(|f| f.code == code),
        "repaired {:?}, remaining {:?}",
        codes(&fixed.repaired),
        codes(&fixed.remaining)
    );
    assert!(
        !fixed.remaining.iter().any(|f| f.code == code),
        "{code} remains: {:?}",
        fixed.remaining
    );
    assert_eq!(validate(&fixed.docx).unwrap(), fixed.remaining);
    fixed.docx
}

#[test]
fn deleted_text_outside_a_deletion_is_revived() {
    let out = assert_repairs(
        "<w:p><w:r><w:delText>x</w:delText></w:r></w:p>",
        "DELTEXT_OUTSIDE_DELETION",
    );
    assert!(
        part_string(&out, "word/document.xml")
            .unwrap()
            .contains("<w:t>x</w:t>")
    );
}

#[test]
fn deleted_text_under_a_move_source_becomes_live_text() {
    let body = r#"<w:p><w:moveFrom w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:moveFrom></w:p>"#;
    let out = assert_repairs(body, "MOVEFROM_WITH_DELTEXT");
    assert!(
        part_string(&out, "word/document.xml")
            .unwrap()
            .contains("<w:t>x</w:t>")
    );
}

/// An insertion inside a move source does not shelter `w:delText`: Word
/// refuses `w:moveFrom > w:ins > w:r > w:delText` (probe M1, 2026-10-18) and
/// opens it once the text is `w:t` (probe M2).
#[test]
fn deleted_text_of_an_insertion_inside_a_move_source_becomes_live_text() {
    let body = r#"<w:p><w:moveFrom w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:ins w:id="2" w:author="B" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:ins></w:moveFrom></w:p>"#;
    let out = assert_repairs(body, "MOVEFROM_WITH_DELTEXT");
    assert!(
        part_string(&out, "word/document.xml")
            .unwrap()
            .contains("<w:t>x</w:t>")
    );
}

/// A deletion inside a move source owns its text: Word opens
/// `w:moveFrom > w:ins > w:del > w:r > w:delText` (probe M5, 2026-10-18) and
/// refuses it with `w:t` (probe M6), so neither check nor repair touches it.
#[test]
fn deleted_text_of_a_deletion_inside_a_move_source_stays_deleted() {
    let body = r#"<w:p><w:moveFrom w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:ins w:id="2" w:author="B" w:date="2026-01-01T00:00:00Z"><w:del w:id="3" w:author="C" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>x</w:delText></w:r></w:del></w:ins></w:moveFrom></w:p>"#;
    let input = docx(body);
    let findings = validate(&input).unwrap();
    assert!(
        !findings
            .iter()
            .any(|f| f.code.contains("DELTEXT") || f.code.contains("DELETION")),
        "{findings:?}"
    );
    let fixed = repair(&input).unwrap();
    assert!(
        part_string(&fixed.docx, "word/document.xml")
            .unwrap()
            .contains("<w:delText>x</w:delText>")
    );
}

#[test]
fn a_field_code_inside_a_deletion_is_a_lead_with_a_repair() {
    let body = format!(
        r#"<w:p>{DEL_OPEN}<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:del></w:p>"#
    );
    let findings = validate(&docx(&body)).unwrap();
    let f = findings
        .iter()
        .find(|f| f.code == "INSTR_TEXT_INSIDE_DELETION")
        .expect("finding");
    assert!(!f.word_fatal && f.repairable);
    let out = assert_repairs(&body, "INSTR_TEXT_INSIDE_DELETION");
    assert!(
        part_string(&out, "word/document.xml")
            .unwrap()
            .contains("<w:delInstrText>PAGE</w:delInstrText>")
    );
}

#[test]
fn a_bookmark_in_a_text_control_is_dropped_with_its_end() {
    let body = r#"<w:p><w:sdt><w:sdtPr><w:text/></w:sdtPr><w:sdtContent><w:bookmarkStart w:id="7" w:name="_x"/><w:r><w:t>x</w:t></w:r></w:sdtContent></w:sdt><w:bookmarkEnd w:id="7"/></w:p>"#;
    let out = assert_repairs(body, "BOOKMARK_IN_SINGLE_VALUE_CONTROL");
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(!xml.contains("bookmark"), "{xml}");
    assert!(xml.contains("<w:t>x</w:t>"));
}

#[test]
fn a_dangling_relationship_attribute_is_dropped() {
    let body = r#"<w:p><w:hyperlink r:id="rId999"><w:r><w:t>x</w:t></w:r></w:hyperlink></w:p>"#;
    let out = assert_repairs(body, "DANGLING_RELATIONSHIP");
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(!xml.contains("rId999"), "{xml}");
    assert!(xml.contains("<w:t>x</w:t>"));
}

#[test]
fn a_duplicate_revision_id_is_renumbered_and_a_move_range_keeps_its_end() {
    let body = r#"<w:p><w:ins w:id="5" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>a</w:t></w:r></w:ins><w:ins w:id="5" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>b</w:t></w:r></w:ins><w:moveFromRangeStart w:id="5" w:name="m1"/><w:moveFrom w:id="9" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>c</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="5"/><w:moveFromRangeStart w:id="5" w:name="m2"/><w:r><w:t>d</w:t></w:r><w:moveFromRangeEnd w:id="5"/></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    // The second insertion, range start and range end: OpenXmlValidator
    // reports the same three (`Sem_UniqueAttributeValue`).
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.code == "DUPLICATE_REVISION_ID")
            .count(),
        3,
        "{findings:?}"
    );
    assert!(findings.iter().all(|f| !f.word_fatal), "{findings:?}");
    let out = assert_repairs(body, "DUPLICATE_REVISION_ID");
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(xml.contains(r#"<w:moveFromRangeStart w:id="11""#), "{xml}");
    assert!(xml.contains(r#"<w:moveFromRangeEnd w:id="11""#), "{xml}");
    assert!(xml.contains(r#"<w:ins w:id="10""#), "{xml}");
    // The first range may share the insertion's id; only the repeated
    // range start and its matching end change.
    assert!(xml.contains(r#"<w:moveFromRangeStart w:id="5""#), "{xml}");
    assert!(xml.contains(r#"<w:moveFromRangeEnd w:id="5""#), "{xml}");
}

#[test]
fn a_paragraph_id_outside_words_range_is_masked_into_it() {
    let body = r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="FFFFFFFF" w14:textId="12345678"><w:r><w:t>x</w:t></w:r></w:p>"#;
    let out = assert_repairs(body, "PARA_ID_OUT_OF_RANGE");
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(xml.contains(r#"w14:paraId="7FFFFFFF""#), "{xml}");
    assert!(
        !xml.contains("FFFFFFFF\"") || xml.contains("7FFFFFFF"),
        "{xml}"
    );
}

#[test]
fn a_cell_without_a_last_paragraph_gets_one() {
    let body = r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid><w:tr><w:tc><w:tcPr/></w:tc></w:tr></w:tbl><w:p/>"#;
    let out = assert_repairs(body, "CELL_WITHOUT_PARAGRAPH");
    let xml = part_string(&out, "word/document.xml")
        .unwrap()
        .replace(" />", "/>");
    assert!(xml.contains("<w:tcPr/><w:p/></w:tc>"), "{xml}");
}

/// A cell-level content control: the cell's last child is a `w:sdt` (here two,
/// nested) whose content ends in the paragraph. Word writes and opens this
/// shape (file_114/115 and file_196/197 redlines of the 0.12.0 sample, Word
/// 16.115, 2026-10-10); appending a paragraph after it adds a blank line.
#[test]
fn a_cell_ending_in_a_content_control_paragraph_is_sound() {
    let body = r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid><w:tr><w:tc><w:tcPr/><w:sdt><w:sdtPr/><w:sdtContent><w:p><w:r><w:t>one</w:t></w:r></w:p></w:sdtContent></w:sdt></w:tc><w:tc><w:tcPr/><w:sdt><w:sdtPr/><w:sdtContent><w:sdt><w:sdtPr/><w:sdtContent><w:p><w:r><w:t>two</w:t></w:r></w:p></w:sdtContent></w:sdt></w:sdtContent></w:sdt></w:tc></w:tr></w:tbl><w:p/>"#;
    let findings = validate(&docx(body)).unwrap();
    assert!(
        !codes(&findings).contains(&"CELL_WITHOUT_PARAGRAPH"),
        "{findings:?}"
    );
}

/// A content control that ends in a table still leaves the cell without a
/// closing paragraph.
#[test]
fn a_cell_ending_in_a_content_control_table_still_needs_a_paragraph() {
    let body = r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="100"/></w:tblGrid><w:tr><w:tc><w:tcPr/><w:sdt><w:sdtPr/><w:sdtContent><w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="50"/></w:tblGrid><w:tr><w:tc><w:p/></w:tc></w:tr></w:tbl></w:sdtContent></w:sdt></w:tc></w:tr></w:tbl><w:p/>"#;
    let findings = validate(&docx(body)).unwrap();
    assert!(
        codes(&findings).contains(&"CELL_WITHOUT_PARAGRAPH"),
        "{findings:?}"
    );
    let out = assert_repairs(body, "CELL_WITHOUT_PARAGRAPH");
    let xml = part_string(&out, "word/document.xml")
        .unwrap()
        .replace(" />", "/>");
    assert!(xml.contains("</w:sdt><w:p/></w:tc>"), "{xml}");
}

#[test]
fn an_orphan_comment_anchor_is_dropped_with_its_reference_run() {
    let body = r#"<w:p><w:commentRangeStart w:id="3"/><w:r><w:t>x</w:t></w:r><w:commentRangeEnd w:id="3"/><w:r><w:rPr><w:rStyle w:val="CommentReference"/></w:rPr><w:commentReference w:id="3"/></w:r></w:p>"#;
    let out = assert_repairs(body, "COMMENT_ANCHOR_ORPHAN");
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(!xml.contains("comment"), "{xml}");
    assert!(xml.contains("<w:t>x</w:t>"));
}

#[test]
fn triage_findings_carry_their_codes_and_are_not_repaired() {
    let body = format!(
        r#"<w:sectPr/><w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r></w:p><w:p>{DEL_OPEN}<w:r><w:fldChar w:fldCharType="begin"/></w:r></w:del><w:r><w:instrText>PAGE</w:instrText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p><w:p><w:r><w:instrText/></w:r></w:p>"#
    );
    let input = docx(&body);
    let findings = validate(&input).unwrap();
    let found = codes(&findings);
    for code in [
        "SECTPR_NOT_LAST",
        "FIELD_UNBALANCED",
        "FIELD_SPLIT_DELETION",
    ] {
        assert!(found.contains(&code), "{code} missing in {found:?}");
    }
    assert!(findings.iter().all(|f| f.part == "word/document.xml"));
    let fixed = repair(&input).unwrap();
    assert!(fixed.repaired.is_empty(), "{:?}", fixed.repaired);
    assert_eq!(codes(&fixed.remaining).len(), findings.len());
    // Findings, but none repairable: the package is not rewritten either.
    assert_eq!(fixed.docx, input);
    // The triage checks alone, as `jubarte debug` sees them.
    let triage = jubarte::debug::findings(&input).unwrap();
    assert!(
        triage
            .iter()
            .any(|f| f.code == "SECTPR_NOT_LAST" && f.word_fatal && !f.repairable)
    );
    assert!(triage.iter().any(|f| f.message.contains("field-unclosed")));
}

#[test]
fn ring1_alone_runs_on_an_opened_package() {
    let pkg = jubarte::opc::PartFs::open(&docx(&para("x"))).unwrap();
    assert!(jubarte::validate::ring1(&pkg).is_empty());
    let body = "<w:p><w:r><w:delText>x</w:delText></w:r></w:p>";
    let pkg = jubarte::opc::PartFs::open(&docx(body)).unwrap();
    assert_eq!(
        codes(&jubarte::validate::ring1(&pkg)),
        ["DELTEXT_OUTSIDE_DELETION"]
    );
}

#[test]
fn validate_error_displays_its_cause() {
    let err = validate(b"not a zip").unwrap_err();
    assert!(!err.to_string().is_empty());
    let err = audit_tracked(b"junk", b"junk", "A").unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[test]
fn a_dangling_relationship_is_reported_once() {
    let body = r#"<w:p><w:hyperlink r:id="rId999"><w:r><w:t>x</w:t></w:r></w:hyperlink></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.code == "DANGLING_RELATIONSHIP")
            .count(),
        1,
        "{findings:?}"
    );
    let fixed = repair(&docx(body)).unwrap();
    assert_eq!(codes(&fixed.repaired), ["DANGLING_RELATIONSHIP"]);
}

#[test]
fn a_header_reference_without_its_relationship_is_removed_whole() {
    let input = docx_with_sect(
        &para("x"),
        &[],
        r#"<w:headerReference w:type="default" r:id="rId777"/>"#,
    );
    let findings = validate(&input).unwrap();
    assert!(
        findings.iter().any(|f| f.code == "DANGLING_RELATIONSHIP"),
        "{findings:?}"
    );
    let fixed = repair(&input).unwrap();
    let xml = part_string(&fixed.docx, "word/document.xml").unwrap();
    assert!(!xml.contains("headerReference"), "{xml}");
    assert!(fixed.remaining.is_empty(), "{:?}", fixed.remaining);
}

#[test]
fn a_masked_paragraph_id_is_never_zero_nor_a_duplicate() {
    let body = r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="80000000"><w:r><w:t>a</w:t></w:r></w:p><w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="00000001"><w:r><w:t>b</w:t></w:r></w:p><w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="80000001"><w:r><w:t>c</w:t></w:r></w:p><w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="00000000"><w:r><w:t>d</w:t></w:r></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.code == "PARA_ID_OUT_OF_RANGE")
            .count(),
        3,
        "{findings:?}"
    );
    let fixed = repair(&docx(body)).unwrap();
    assert!(fixed.remaining.is_empty(), "{:?}", fixed.remaining);
    let xml = part_string(&fixed.docx, "word/document.xml").unwrap();
    let ids: Vec<&str> = xml
        .match_indices("w14:paraId=\"")
        .map(|(i, m)| &xml[i + m.len()..i + m.len() + 8])
        .collect();
    assert_eq!(ids.len(), 4, "{xml}");
    let distinct: std::collections::HashSet<&str> = ids.iter().copied().collect();
    assert_eq!(distinct.len(), 4, "{ids:?}");
    assert!(!ids.contains(&"00000000"), "{ids:?}");
    assert!(
        ids.iter()
            .all(|id| u32::from_str_radix(id, 16).unwrap() < 0x8000_0000),
        "{ids:?}"
    );
}

const HEADER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
const HEADER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";

fn header_docx(header_text: &str) -> Vec<u8> {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>{header_text}</w:t></w:r></w:p></w:hdr>"#
    );
    docx_with_sect(
        &para("Body stays."),
        &[Part {
            name: "word/header1.xml",
            content_type: HEADER_CT,
            rel_type: HEADER_REL,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

#[test]
fn the_audit_sees_an_untracked_header_edit_and_names_its_part() {
    let original = header_docx("Draft");
    let edited = header_docx("Final");
    assert!(validate(&edited).unwrap().is_empty());
    let findings = audit_tracked(&original, &edited, "Legal").unwrap();
    assert_eq!(codes(&findings), ["UNTRACKED_EDIT"], "{findings:?}");
    assert_eq!(findings[0].part, "word/header1.xml");
    assert!(
        findings[0].location == "header1",
        "{}",
        findings[0].location
    );
    assert!(
        findings[0]
            .message
            .starts_with(&format!("{} differs", findings[0].r#ref))
            && !findings[0].message.contains(":p:"),
        "{}",
        findings[0].message
    );
    assert!(
        audit_tracked(&original, &original, "Legal")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_package_with_nothing_to_repair_comes_back_byte_for_byte() {
    // Nothing repairable: repair writes nothing, so a caller can compare
    // bytes to know nothing changed.
    let clean = docx(&para("nothing wrong"));
    let fixed = repair(&clean).unwrap();
    assert!(fixed.repaired.is_empty() && fixed.remaining.is_empty());
    assert_eq!(fixed.docx, clean);
}

// Foreign history is compared after rejecting the reviewing author's edits.
const FOREIGN_HISTORY: &str = r#"<w:p><w:ins w:id="1" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>alpha beta gamma</w:t></w:r></w:ins><w:del w:id="2" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> tail</w:t></w:r></w:p>"#;

fn foreign_plan(original: &[u8], author: &str, operation: &serde_json::Value) -> Vec<u8> {
    let plan = jubarte::edit::EditPlan::from_json(
        &serde_json::json!({
            "schema_version": 1,
            "author": author,
            "date": "2026-02-01T00:00:00Z",
            "existing_revisions": "keep",
            "operations": [operation]
        })
        .to_string(),
    )
    .unwrap();
    jubarte::edit::apply_plan(original, &plan).unwrap().redline
}

fn foreign_messages(original: &[u8], edited: &[u8]) -> Vec<String> {
    audit_tracked(original, edited, "Bob")
        .unwrap()
        .into_iter()
        .filter(|f| f.code == "FOREIGN_AUTHOR")
        .map(|f| {
            assert_eq!((f.set.as_str(), f.severity.as_str()), ("edits", "error"));
            f.message
        })
        .collect()
}

#[test]
fn foreign_history_validated_against_itself_is_clean() {
    let original = docx(FOREIGN_HISTORY);
    let findings = audit_tracked(&original, &original, "Bob").unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn foreign_history_split_by_bobs_tracked_replace_is_clean() {
    let original = docx(FOREIGN_HISTORY);
    let edited = foreign_plan(
        &original,
        "Bob",
        &serde_json::json!({
            "kind": "replace", "paragraph": "p0", "find": "beta", "replacement": "new"
        }),
    );
    let listed = jubarte::changes::list_changes(&edited).unwrap();
    assert_eq!(
        listed
            .iter()
            .filter(|c| c.author.as_deref() == Some("Alice")
                && c.kind == jubarte::changes::ChangeKind::Insertion)
            .count(),
        2
    );
    let findings = audit_tracked(&original, &edited, "Bob").unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

#[test]
fn foreign_history_a_plan_under_alices_name_reports_only_the_new_revision() {
    let original = docx(FOREIGN_HISTORY);
    let edited = foreign_plan(
        &original,
        "Alice",
        &serde_json::json!({
            "kind": "insert", "paragraph": "p0", "after": "tail", "text": " extra"
        }),
    );
    let listed = jubarte::changes::list_changes(&edited).unwrap();
    let added = listed
        .iter()
        .find(|c| c.date.as_deref() == Some("2026-02-01T00:00:00Z"))
        .unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        [format!("{} by Alice is new", added.id)]
    );
}

#[test]
fn foreign_history_rejecting_alices_insertion_reports_the_original_id_gone() {
    let original = docx(FOREIGN_HISTORY);
    let edited =
        jubarte::changes::reject_changes(&original, &jubarte::changes::ChangeFilter::ids(["r1"]))
            .unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        ["Alice's r1 in the original is gone"]
    );
}

#[test]
fn foreign_history_carols_new_revision_reports_only_carol() {
    let original = docx(FOREIGN_HISTORY);
    let edited = foreign_plan(
        &original,
        "Carol Diaz",
        &serde_json::json!({
            "kind": "insert", "paragraph": "p0", "after": "tail", "text": " extra"
        }),
    );
    let listed = jubarte::changes::list_changes(&edited).unwrap();
    let added = listed
        .iter()
        .find(|c| c.author.as_deref() == Some("Carol Diaz"))
        .unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        [format!("{} by Carol Diaz is new", added.id)]
    );
}

#[test]
fn foreign_history_ids_do_not_define_identity() {
    let original = docx(FOREIGN_HISTORY);
    let edited = docx(
        &FOREIGN_HISTORY
            .replace("id=\"1\"", "id=\"91\"")
            .replace("id=\"2\"", "id=\"92\""),
    );
    assert_eq!(audit_tracked(&original, &edited, "Bob").unwrap(), []);
}

#[test]
fn foreign_history_changed_deletion_text_is_one_foreign_revision() {
    let original = docx(FOREIGN_HISTORY);
    let edited = docx(&FOREIGN_HISTORY.replace(">old<", ">different<"));
    assert_eq!(
        foreign_messages(&original, &edited),
        ["r2 by Alice differs from the original"]
    );
}

#[test]
fn foreign_history_is_compared_per_paragraph() {
    let original = docx(&(FOREIGN_HISTORY.to_string() + &para("tail")));
    let edited = docx(&(para("tail") + FOREIGN_HISTORY));
    // One finding per logical change: `changes` lists Alice's two marks as
    // the change `r1+2` (#1893).
    assert_eq!(
        foreign_messages(&original, &edited),
        [
            "r1+2 by Alice is new",
            "Alice's r1+2 in the original is gone"
        ]
    );
}

fn foreign_story(story: &str, body: &str) -> Vec<u8> {
    let (root, content) = match story {
        "header1" => ("hdr", body.to_string()),
        "footer1" => ("ftr", body.to_string()),
        "footnotes" => (
            "footnotes",
            format!(r#"<w:footnote w:id="1">{body}</w:footnote>"#),
        ),
        "endnotes" => (
            "endnotes",
            format!(r#"<w:endnote w:id="1">{body}</w:endnote>"#),
        ),
        _ => unreachable!(),
    };
    let xml = format!(
        r#"<w:{root} xmlns:w="{}">{content}</w:{root}>"#,
        common::docx::W_NS
    );
    let kind = match story {
        "header1" => "header",
        "footer1" => "footer",
        _ => story,
    };
    let content_type =
        format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml");
    let rel_type = format!("{}/{kind}", common::docx::R_NS);
    common::docx::docx_with(
        &para("body"),
        &[Part {
            name: &format!("word/{story}.xml"),
            content_type: &content_type,
            rel_type: &rel_type,
            xml: &xml,
        }],
    )
}

#[test]
fn foreign_history_in_headers_footers_and_notes_is_preserved() {
    let findings: Vec<_> = ["header1", "footer1", "footnotes", "endnotes"]
        .into_iter()
        .map(|story| {
            let original = foreign_story(story, FOREIGN_HISTORY);
            (story, audit_tracked(&original, &original, "Bob").unwrap())
        })
        .collect();
    assert!(findings.iter().all(|(_, f)| f.is_empty()), "{findings:?}");
}

#[test]
fn foreign_history_removed_from_a_story_names_its_original_id_and_part() {
    let findings: Vec<_> = ["header1", "footer1", "footnotes", "endnotes"]
        .into_iter()
        .map(|story| {
            let original = foreign_story(story, FOREIGN_HISTORY);
            let without_insertion =
                format!("<w:p>{}", FOREIGN_HISTORY.split_once("</w:ins>").unwrap().1);
            let edited = foreign_story(story, &without_insertion);
            let foreign: Vec<_> = audit_tracked(&original, &edited, "Bob")
                .unwrap()
                .into_iter()
                .filter(|f| f.code == "FOREIGN_AUTHOR")
                .collect();
            (story, foreign)
        })
        .collect();
    for (story, foreign) in findings {
        assert_eq!(foreign.len(), 1, "{story}: {foreign:?}");
        assert_eq!(
            foreign[0].message,
            format!("Alice's {story}.r1 in the original is gone")
        );
        assert_eq!(foreign[0].part, format!("word/{story}.xml"));
    }
}

const FOREIGN_OTHER_KINDS: &str = r#"<w:p><w:pPr><w:rPr><w:ins w:id="10" w:author="Alice"/></w:rPr></w:pPr><w:moveFrom w:id="11" w:author="Alice"><w:r><w:t>moved</w:t></w:r></w:moveFrom><w:moveTo w:id="12" w:author="Alice"><w:r><w:t>moved</w:t></w:r></w:moveTo><w:r><w:rPr><w:b/><w:rPrChange w:id="13" w:author="Alice"><w:rPr/></w:rPrChange></w:rPr><w:t>formatted</w:t></w:r></w:p>"#;

#[test]
fn foreign_history_moves_formatting_and_paragraph_marks_are_clean() {
    let original = docx(FOREIGN_OTHER_KINDS);
    assert_eq!(audit_tracked(&original, &original, "Bob").unwrap(), []);
}

#[test]
fn foreign_history_removed_move_format_or_paragraph_mark_is_reported() {
    let original = docx(FOREIGN_OTHER_KINDS);
    let results: Vec<_> = ["r10", "r11", "r12", "r13"]
        .into_iter()
        .map(|id| {
            let edited = jubarte::changes::accept_changes(
                &original,
                &jubarte::changes::ChangeFilter::ids([id]),
            )
            .unwrap();
            foreign_messages(&original, &edited)
        })
        .collect();
    assert_eq!(
        results,
        ["r10", "r11", "r12", "r13"]
            .map(|id| vec![format!("Alice's {id} in the original is gone")])
    );
}

#[test]
fn foreign_history_without_an_author_is_retained_and_named_when_removed() {
    let original = docx(&FOREIGN_HISTORY.replace(r#" w:author="Alice""#, ""));
    assert_eq!(audit_tracked(&original, &original, "Bob").unwrap(), []);
    let edited =
        jubarte::changes::reject_changes(&original, &jubarte::changes::ChangeFilter::ids(["r1"]))
            .unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        ["<no author>'s r1 in the original is gone"]
    );
}

#[test]
fn foreign_history_different_dates_are_not_coalesced() {
    let original = docx(
        r#"<w:p><w:ins w:id="1" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>alpha</w:t></w:r></w:ins><w:ins w:id="2" w:author="Alice" w:date="2026-01-02T00:00:00Z"><w:r><w:t>beta</w:t></w:r></w:ins></w:p>"#,
    );
    let edited = docx(
        r#"<w:p><w:ins w:id="91" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>alphabeta</w:t></w:r></w:ins></w:p>"#,
    );
    assert_eq!(
        foreign_messages(&original, &edited),
        [
            "r91 by Alice differs from the original",
            "Alice's r2 in the original is gone"
        ]
    );
}

#[test]
fn foreign_history_exact_matches_are_consumed_before_changed_text() {
    let body = FOREIGN_HISTORY.replace(">old<", ">separator<");
    let original = docx(&body.replace("</w:p>", r#"<w:ins w:id="3" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>second</w:t></w:r></w:ins></w:p>"#));
    let edited = docx(
        &body
            .replace("alpha beta gamma", "second")
            .replace(r#"id="1""#, r#"id="91""#),
    );
    assert_eq!(
        foreign_messages(&original, &edited),
        ["Alice's r1 in the original is gone"]
    );
}

fn foreign_carrier(kind: &str, id: usize, author: &str, text: &str) -> String {
    let tag = if kind == "del" { "delText" } else { "t" };
    format!(
        r#"<w:{kind} w:id="{id}" w:author="{author}" w:date="2026-01-01T00:00:00Z"><w:r><w:{tag} xml:space="preserve">{text}</w:{tag}></w:r></w:{kind}>"#
    )
}

#[test]
fn foreign_separator_settlement_reports_only_r2() {
    let original = docx(&format!(
        "<w:p>{}{}{}</w:p>",
        foreign_carrier("ins", 1, "Alice", "Hello "),
        foreign_carrier("del", 2, "Alice", "old"),
        foreign_carrier("ins", 3, "Alice", "World")
    ));
    let edited =
        jubarte::changes::accept_changes(&original, &jubarte::changes::ChangeFilter::ids(["r2"]))
            .unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        ["Alice's r2 in the original is gone"]
    );
}

#[test]
fn foreign_visible_separator_prevents_deletion_redistribution() {
    let make = |left, right| {
        docx(&format!(
            "<w:p>{}<w:r><w:t> X </w:t></w:r>{}</w:p>",
            foreign_carrier("del", 1, "Alice", left),
            foreign_carrier("del", 3, "Alice", right)
        ))
    };
    assert!(!foreign_messages(&make("penalty", "bonus"), &make("penaltyb", "onus")).is_empty());
}

#[test]
fn foreign_visible_separator_prevents_move_redistribution() {
    let make = |left, right| {
        docx(&format!(
            "<w:p>{}<w:r><w:t> X </w:t></w:r>{}</w:p>",
            foreign_carrier("moveFrom", 1, "Alice", left),
            foreign_carrier("moveFrom", 3, "Alice", right)
        ))
    };
    assert!(!foreign_messages(&make("penalty", "bonus"), &make("penaltyb", "onus")).is_empty());
}

#[test]
fn foreign_deletion_swap_preserves_order() {
    let make = |left, right| {
        docx(&format!(
            "<w:p>{}{}{}</w:p>",
            foreign_carrier("del", 1, "Alice", left),
            foreign_carrier("ins", 2, "Carol Diaz", "new rate"),
            foreign_carrier("del", 3, "Alice", right)
        ))
    };
    assert!(!foreign_messages(&make("penalty", "bonus"), &make("bonus", "penalty")).is_empty());
}

fn foreign_owner_revisions(part: &str, owners: &str) -> Vec<u8> {
    let (root, kind) = if part == "styles" {
        ("styles", "styles")
    } else {
        ("numbering", "numbering")
    };
    let xml = format!(
        r#"<w:{root} xmlns:w="{}">{owners}</w:{root}>"#,
        common::docx::W_NS
    );
    common::docx::docx_with(
        &para("body"),
        &[Part {
            name: &format!("word/{part}.xml"),
            content_type: &format!(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml"
            ),
            rel_type: &format!("{}/{kind}", common::docx::R_NS),
            xml: &xml,
        }],
    )
}

fn foreign_style(id: usize, name: &str) -> String {
    format!(
        r#"<w:style w:type="paragraph" w:styleId="{name}"><w:name w:val="{name}"/><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:rPr/></w:rPrChange></w:rPr></w:style>"#
    )
}

#[test]
fn foreign_style_x_owner_survives_y_settlement() {
    let original =
        foreign_owner_revisions("styles", &(foreign_style(1, "X") + &foreign_style(2, "Y")));
    let edited = jubarte::changes::accept_changes(
        &original,
        &jubarte::changes::ChangeFilter::ids(["styles.r2"]),
    )
    .unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        ["Alice's styles.r2 in the original is gone"]
    );
}

#[test]
fn foreign_style_y_owner_survives_x_settlement() {
    let original =
        foreign_owner_revisions("styles", &(foreign_style(1, "X") + &foreign_style(2, "Y")));
    let edited = jubarte::changes::accept_changes(
        &original,
        &jubarte::changes::ChangeFilter::ids(["styles.r1"]),
    )
    .unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        ["Alice's styles.r1 in the original is gone"]
    );
}

#[test]
fn foreign_numbering_part_is_outside_change_listing() {
    let owner = |id| {
        format!(
            r#"<w:abstractNum w:abstractNumId="{id}"><w:lvl w:ilvl="0"><w:rPr><w:b/><w:rPrChange w:id="{id}" w:author="Alice"><w:rPr/></w:rPrChange></w:rPr></w:lvl></w:abstractNum>"#
        )
    };
    let original = foreign_owner_revisions("numbering", &(owner(1) + &owner(2)));
    assert!(
        jubarte::changes::list_changes(&original)
            .unwrap()
            .is_empty()
    );
    assert!(foreign_messages(&original, &original).is_empty());
}

#[test]
fn foreign_paragraph_inserted_above_history_only_reports_structure() {
    let body = format!(
        "{}<w:p>{}</w:p>",
        para("a"),
        foreign_carrier("ins", 1, "Alice", "x")
    );
    let findings = audit_tracked(&docx(&body), &docx(&(para("new") + &body)), "Bob").unwrap();
    assert!(
        findings.iter().all(|f| f.code != "FOREIGN_AUTHOR"),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|f| f.code == "UNTRACKED_EDIT"
                && f.message.contains("2 paragraphs before, 3 after")),
        "{findings:?}"
    );
}

#[test]
fn foreign_insertion_moved_to_another_paragraph_is_reported() {
    let revision = foreign_carrier("ins", 1, "Alice", "x");
    let original = docx(&format!(
        "<w:p><w:r><w:t>a</w:t></w:r>{revision}</w:p>{}",
        para("b")
    ));
    let edited = docx(&format!(
        "{}<w:p><w:r><w:t>b</w:t></w:r>{revision}</w:p>",
        para("a")
    ));
    assert!(!foreign_messages(&original, &edited).is_empty());
}

#[test]
fn foreign_changed_and_new_messages_are_distinct_and_do_not_quote_text() {
    let original = docx(FOREIGN_HISTORY);
    let changed = docx(&FOREIGN_HISTORY.replace(">old<", ">confidential<"));
    assert_eq!(
        foreign_messages(&original, &changed),
        ["r2 by Alice differs from the original"]
    );
    let added = docx(&FOREIGN_HISTORY.replace(
        "</w:p>",
        &(foreign_carrier("ins", 8, "Carol Diaz", "confidential") + "</w:p>"),
    ));
    assert_eq!(
        foreign_messages(&original, &added),
        ["r8 by Carol Diaz is new"]
    );
}

#[test]
#[cfg_attr(
    miri,
    ignore = "wall-clock performance bound is for native execution, not Miri interpretation"
)]
fn foreign_reverse_ten_thousand_revisions_is_fast() {
    let make = |reverse: bool| {
        let mut body = String::from("<w:p>");
        for n in 0..10_000 {
            let id = if reverse { 10_000 - n } else { n + 1 };
            let date = format!(
                "2026-01-01T{:02}:{:02}:{:02}Z",
                id / 3600,
                id / 60 % 60,
                id % 60
            );
            body.push_str(
                &foreign_carrier("del", id, "Alice", &format!("text {id}"))
                    .replace("2026-01-01T00:00:00Z", &date),
            );
        }
        body.push_str("</w:p>");
        docx(&body)
    };
    let original = make(false);
    let edited = make(true);
    let start = std::time::Instant::now();
    let _findings = foreign_messages(&original, &edited);
    let elapsed = start.elapsed();
    eprintln!("10,000 reversed revisions took {elapsed:?}");
    // Quadratic matching took 200 s here; linear takes about 2 s, and up to
    // 5 s while other builds share the machine. A wide bound stays
    // deterministic under load and still fails the quadratic shape.
    assert!(
        elapsed < std::time::Duration::from_secs(60),
        "10,000 reversed revisions took {elapsed:?}"
    );
}

#[test]
fn foreign_change_json_keeps_its_exact_public_shape() {
    let changes = jubarte::changes::list_changes(&docx(&format!(
        "<w:p>{}</w:p>",
        foreign_carrier("del", 1, "Alice", "old")
    )))
    .unwrap();
    assert_eq!(
        serde_json::to_string(&changes[0]).unwrap(),
        r#"{"id":"r1","ref":"r1","kind":"deletion","target":"text","author":"Alice","date":"2026-01-01T00:00:00Z","text":"old","paragraph":"p0"}"#
    );
}

#[test]
fn foreign_new_revision_message_is_explicit() {
    let original = docx(FOREIGN_HISTORY);
    let edited = docx(&FOREIGN_HISTORY.replace(
        "</w:p>",
        &(foreign_carrier("ins", 8, "Carol Diaz", "confidential") + "</w:p>"),
    ));
    assert_eq!(
        foreign_messages(&original, &edited),
        ["r8 by Carol Diaz is new"]
    );
}

#[test]
fn foreign_accepted_insertion_separator_reports_only_it() {
    let original = docx(&format!(
        "<w:p>{}{}{}</w:p>",
        foreign_carrier("del", 1, "Alice", "penalty"),
        foreign_carrier("ins", 2, "Carol Diaz", "new rate"),
        foreign_carrier("del", 3, "Alice", "bonus")
    ));
    let edited =
        jubarte::changes::accept_changes(&original, &jubarte::changes::ChangeFilter::ids(["r2"]))
            .unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        ["Carol Diaz's r2 in the original is gone"]
    );
}

#[test]
fn foreign_story_paragraph_alignment_includes_deleted_text() {
    for story in ["header1", "footer1", "footnotes", "endnotes"] {
        let body = format!(
            "{}<w:p>{}</w:p>",
            para("a"),
            foreign_carrier("del", 1, "Alice", "deleted")
        );
        let original = foreign_story(story, &body);
        let edited = foreign_story(story, &(para("new") + &body));
        let findings = audit_tracked(&original, &edited, "Bob").unwrap();
        assert!(
            findings.iter().all(|f| f.code != "FOREIGN_AUTHOR"),
            "{story}: {findings:?}"
        );
        assert!(findings.iter().any(|f| f.code == "UNTRACKED_EDIT"));
    }
}

#[test]
fn foreign_whole_paragraph_relocation_is_reported_in_both_directions() {
    for (original, edited) in [
        (
            FOREIGN_HISTORY.to_string() + &para("tail"),
            para("tail") + FOREIGN_HISTORY,
        ),
        (
            para("tail") + FOREIGN_HISTORY,
            FOREIGN_HISTORY.to_string() + &para("tail"),
        ),
    ] {
        assert!(!foreign_messages(&docx(&original), &docx(&edited)).is_empty());
    }
}

#[test]
fn foreign_empty_runs_do_not_separate_adjacent_splits() {
    let original = docx(&format!(
        "<w:p>{}</w:p>",
        foreign_carrier("ins", 1, "Alice", "Hello World")
    ));
    let edited = docx(&format!(
        "<w:p>{}<w:r><w:t/></w:r>{}</w:p>",
        foreign_carrier("ins", 1, "Alice", "Hello "),
        foreign_carrier("ins", 3, "Alice", "World")
    ));
    assert!(foreign_messages(&original, &edited).is_empty());
}

fn foreign_sandwich(separator: &str, left: &str, right: &str) -> Vec<u8> {
    docx(&format!(
        "<w:p>{}{separator}{}</w:p>",
        foreign_carrier("ins", 1, "Alice", left),
        foreign_carrier("ins", 3, "Alice", right)
    ))
}

#[test]
fn foreign_settled_move_does_not_name_untouched_neighbors() {
    let separator = format!(
        r#"<w:moveFromRangeStart w:id="20" w:name="move"/>{}<w:moveFromRangeEnd w:id="20"/>"#,
        foreign_carrier("moveFrom", 2, "Alice", "X")
    );
    let original = foreign_sandwich(&separator, "A", "B");
    let edited = foreign_sandwich("", "A", "B");
    assert_eq!(
        foreign_messages(&original, &edited),
        ["Alice's r2 in the original is gone"]
    );
}

#[test]
fn foreign_removed_empty_move_markers_are_transparent() {
    for side in ["From", "To"] {
        let markers = format!(
            r#"<w:move{side}RangeStart w:id="20" w:name="move"/><w:move{side}RangeEnd w:id="20"/>"#
        );
        assert_eq!(
            foreign_messages(
                &foreign_sandwich(&markers, "A", "B"),
                &foreign_sandwich("", "A", "B")
            ),
            Vec::<String>::new()
        );
    }
}

#[test]
fn foreign_retained_empty_move_markers_keep_boundaries() {
    let markers =
        r#"<w:moveFromRangeStart w:id="20" w:name="move"/><w:moveFromRangeEnd w:id="20"/>"#;
    let original = foreign_sandwich(markers, "A", "BC");
    assert_eq!(foreign_messages(&original, &original), Vec::<String>::new());
    assert_eq!(
        foreign_messages(&original, &foreign_sandwich(markers, "AB", "C")),
        // One finding per logical change: the two carriers are the change
        // `r1+3` (#1893).
        ["r1+3 by Alice differs from the original"]
    );
}

#[test]
fn foreign_rejected_reviewer_carrier_is_transparent() {
    let original = foreign_sandwich(&foreign_carrier("ins", 2, "Bob", "X"), "A", "B");
    assert_eq!(foreign_messages(&original, &original), Vec::<String>::new());
}

#[test]
fn foreign_settled_separator_names_the_changed_carrier() {
    let original = foreign_sandwich(
        &foreign_carrier("del", 2, "Alice", "old"),
        "Hello ",
        "World",
    );
    let edited = foreign_sandwich("", "Hello ", "Changed");
    assert_eq!(
        foreign_messages(&original, &edited),
        [
            "r3 by Alice differs from the original",
            "Alice's r2 in the original is gone"
        ]
    );
}

#[test]
fn foreign_settled_adjacent_carol_carriers_are_one_gone_change() {
    let original = docx(&format!(
        "<w:p>{}{}{}{}</w:p>",
        foreign_carrier("del", 1, "Alice", "left"),
        foreign_carrier("ins", 2, "Carol Diaz", "one"),
        foreign_carrier("ins", 3, "Carol Diaz", "two"),
        foreign_carrier("del", 4, "Alice", "right")
    ));
    let edited = jubarte::changes::accept_changes(
        &original,
        &jubarte::changes::ChangeFilter::ids(["r2", "r3"]),
    )
    .unwrap();
    // One finding per logical change: the two marks are the change
    // `r2+3` (#1893).
    assert_eq!(
        foreign_messages(&original, &edited),
        ["Carol Diaz's r2+3 in the original is gone"]
    );
}

#[test]
fn foreign_untracked_paragraph_merge_reports_new_gone_and_structure() {
    let revision = foreign_carrier("ins", 1, "Alice", "history");
    let original = docx(&format!("{}<w:p>{revision}</w:p>", para("lead")));
    let edited = docx(&format!("<w:p><w:r><w:t>lead</w:t></w:r>{revision}</w:p>"));
    let findings = audit_tracked(&original, &edited, "Bob").unwrap();
    assert_eq!(
        foreign_messages(&original, &edited),
        ["r1 by Alice is new", "Alice's r1 in the original is gone"]
    );
    assert_eq!(
        findings.iter().map(|f| f.code.as_str()).collect::<Vec<_>>(),
        ["FOREIGN_AUTHOR", "FOREIGN_AUTHOR", "UNTRACKED_EDIT"]
    );
}

#[test]
fn foreign_removed_markers_allow_a_single_merged_carrier() {
    let markers =
        r#"<w:moveFromRangeStart w:id="20" w:name="move"/><w:moveFromRangeEnd w:id="20"/>"#;
    let original = foreign_sandwich(markers, "A", "B");
    let merged = docx(&format!(
        "<w:p>{}</w:p>",
        foreign_carrier("ins", 1, "Alice", "AB")
    ));
    assert_eq!(foreign_messages(&original, &merged), Vec::<String>::new());
}

/// #1214: Word opens this shape cleanly but Reject All duplicates moved text.
#[test]
fn unequal_move_range_counts_warn_without_being_word_fatal() {
    let ranges = |side: &str, count: usize, name: &str, offset: usize| {
        (0..count).map(|i| {
            let id = offset + i * 2;
            format!(r#"<w:{side}RangeStart w:id="{id}" w:name="{name}"/><w:{side} w:id="{}" w:author="Alice" w:date="2026-10-18T12:00:00Z"><w:r><w:t>Alice</w:t></w:r></w:{side}><w:{side}RangeEnd w:id="{id}"/>"#, id + 1)
        }).collect::<String>()
    };
    for (from, to) in [(2, 1), (1, 2), (1, 0), (0, 1), (1, 1), (2, 2), (0, 0)] {
        let body = format!(
            "<w:p>{}{}</w:p>",
            ranges("moveFrom", from, "move1", 1),
            ranges("moveTo", to, "move1", 20)
        );
        let findings = validate(&docx(&body)).unwrap();
        let warnings: Vec<_> = findings
            .iter()
            .filter(|f| f.code == "MOVE_RANGE_COUNT_MISMATCH")
            .collect();
        assert_eq!(
            warnings.len(),
            usize::from(from != to),
            "{from} source ranges, {to} destination ranges: {findings:?}"
        );
        if let Some(f) = warnings.first() {
            assert_eq!(f.severity, "warning");
            assert_eq!(f.set, "word");
            assert!(!f.word_fatal && !f.repairable);
            assert_eq!(f.part, "word/document.xml");
            assert!(f.message.contains("move1"));
        }
    }
    // Equal totals across different names must not hide either unmatched move.
    let body = format!(
        "<w:p>{}{}</w:p>",
        ranges("moveFrom", 1, "move1", 1),
        ranges("moveTo", 1, "move2", 20)
    );
    assert_eq!(
        validate(&docx(&body))
            .unwrap()
            .iter()
            .filter(|f| f.code == "MOVE_RANGE_COUNT_MISMATCH")
            .count(),
        2
    );
}
