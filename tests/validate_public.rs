// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::validate`: Ring-1 invariants as data, with repair and the
//! tracked-edit audit.

mod common;

use common::docx::{docx, para, part_string};
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
    assert!(f.path.starts_with("w:body[0]/w:p[0]"), "{}", f.path);

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
fn findings_are_json_with_stable_field_names() {
    let body = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    let json: serde_json::Value = serde_json::to_value(&findings).unwrap();
    let first = &json[0];
    for key in [
        "code",
        "part",
        "path",
        "message",
        "word_fatal",
        "repairable",
    ] {
        assert!(first.get(key).is_some(), "{key} missing in {first}");
    }
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
    assert!(
        findings[0].message.contains("body:p:1"),
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
    assert_eq!(markdown(&tracked).unwrap().matches("[body:p:").count(), 2);
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
    let body = r#"<w:p><w:ins w:id="5" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>a</w:t></w:r></w:ins><w:ins w:id="5" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>b</w:t></w:r></w:ins><w:moveFromRangeStart w:id="5" w:name="m1"/><w:moveFrom w:id="9" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>c</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="5"/></w:p>"#;
    let findings = validate(&docx(body)).unwrap();
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.code == "DUPLICATE_REVISION_ID")
            .count(),
        2,
        "{findings:?}"
    );
    assert!(findings.iter().all(|f| !f.word_fatal), "{findings:?}");
    let out = assert_repairs(body, "DUPLICATE_REVISION_ID");
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(xml.contains(r#"<w:moveFromRangeStart w:id="11""#), "{xml}");
    assert!(xml.contains(r#"<w:moveFromRangeEnd w:id="11""#), "{xml}");
    assert!(xml.contains(r#"<w:ins w:id="10""#), "{xml}");
}

#[test]
fn a_paragraph_id_outside_words_range_is_masked_into_it() {
    let body = r#"<w:p xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" w14:paraId="FFFFFFFF" w14:textId="7FFFFFFF"><w:r><w:t>x</w:t></w:r></w:p>"#;
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
