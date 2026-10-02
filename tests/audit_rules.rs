// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte::audit`: one test per rule, its negative case, and the CLI's
//! exit codes.

mod common;

use std::process::Command;

use common::docx::{Part, docx, docx_with, docx_with_sect, para};
use jubarte::audit::{AuditError, RULES, audit, audit_report, audit_report_with};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

const STYLES_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml";
const STYLES_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles";
const FOOTER_CT: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
const FOOTER_REL: &str =
    "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn codes(docx: &[u8], rules: &[&str]) -> Vec<String> {
    audit(docx, rules)
        .unwrap()
        .into_iter()
        .map(|f| f.code)
        .collect()
}

/// A styles part whose `docDefaults` sets the language and whose default
/// paragraph style `Normal` asks for 12pt Calibri.
fn styles_with_lang() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="{W_NS}"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/><w:lang w:val="en-US"/></w:rPr></w:rPrDefault></w:docDefaults><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:sz w:val="24"/></w:rPr></w:style><w:style w:type="paragraph" w:styleId="Titulo1"><w:name w:val="heading 1"/></w:style><w:style w:type="paragraph" w:styleId="Titulo2"><w:name w:val="heading 2"/></w:style></w:styles>"#
    )
}

fn with_styles(body: &str, styles: &str) -> Vec<u8> {
    docx_with(
        body,
        &[Part {
            name: "word/styles.xml",
            content_type: STYLES_CT,
            rel_type: STYLES_REL,
            xml: styles,
        }],
    )
}

fn with_footer(body: &str, footer_inner: &str) -> Vec<u8> {
    let footer = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="{W_NS}">{footer_inner}</w:ftr>"#
    );
    docx_with_sect(
        body,
        &[Part {
            name: "word/footer1.xml",
            content_type: FOOTER_CT,
            rel_type: FOOTER_REL,
            xml: &footer,
        }],
        r#"<w:footerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

fn numpages(result: &str) -> String {
    format!(
        r#"<w:p><w:r><w:t xml:space="preserve">of </w:t></w:r><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> NUMPAGES </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>{result}</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#
    )
}

fn sized_run(text: &str, half_points: u32) -> String {
    format!(r#"<w:r><w:rPr><w:sz w:val="{half_points}"/></w:rPr><w:t>{text}</w:t></w:r>"#)
}

const IMAGE_PREFIX: &str = r#"<w:p><w:r><w:drawing><wp:inline xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing"><wp:extent cx="914400" cy="914400"/>"#;
const IMAGE_SUFFIX: &str = r#"<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture"/></a:graphic></wp:inline></w:drawing></w:r></w:p>"#;

fn table(header_row: bool) -> String {
    let tr_pr = if header_row {
        "<w:trPr><w:tblHeader/></w:trPr>"
    } else {
        ""
    };
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr>{tr_pr}<w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>h</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:tcPr><w:tcW w:w="4000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>d</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
    )
}

// ---- the plan's five tests, verbatim ------------------------------------

#[test]
fn literal_bullet_and_heading_skip() {
    let body = String::new()
        + r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Title</w:t></w:r></w:p>"#
        + r#"<w:p><w:pPr><w:pStyle w:val="Heading3"/></w:pPr><w:r><w:t>Deep</w:t></w:r></w:p>"#
        + &para("• item");
    let found = audit(&docx(&body), &["a11y", "style"]).unwrap();
    assert!(
        found
            .iter()
            .any(|f| f.code == "HEADING_SKIP" && f.location == "body:p:1"),
        "{found:?}"
    );
    assert!(
        found
            .iter()
            .any(|f| f.code == "LITERAL_BULLET" && f.location == "body:p:2"),
        "{found:?}"
    );
}

#[test]
fn a_numbered_paragraph_is_not_a_literal_bullet() {
    let body = r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>- dash in a real list</w:t></w:r></w:p>"#;
    assert!(!codes(&docx(body), &["style"]).contains(&"LITERAL_BULLET".to_string()));
}

#[test]
fn table_without_header_row_and_image_without_descr() {
    let image = format!(r#"{IMAGE_PREFIX}<wp:docPr id="1" name="Pic"/>{IMAGE_SUFFIX}"#);
    let found = codes(&docx(&format!("{}{image}", table(false))), &["a11y"]);
    assert!(
        found.contains(&"TABLE_NO_HEADER_ROW".to_string())
            && found.contains(&"IMAGE_NO_DESCR".to_string()),
        "{found:?}"
    );
}

#[test]
fn spacers_and_missing_lang() {
    let body = para("a") + "<w:p/><w:p/>" + &para("b");
    let found = codes(&docx(&body), &["style", "a11y"]);
    assert!(
        found.contains(&"EMPTY_SPACER_PARAGRAPH".to_string())
            && found.contains(&"MISSING_LANG".to_string()),
        "{found:?}"
    );
}

#[test]
fn an_empty_toc_is_a_stale_cache() {
    let toc = r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TOC \o "1-3" </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
    assert!(codes(&docx(toc), &["STALE_FIELD_CACHE"]).contains(&"STALE_FIELD_CACHE".to_string()));
}

// ---- the rule table ------------------------------------------------------

#[test]
fn rules_table_lists_nine_codes_with_sets_and_severities() {
    let codes: Vec<&str> = RULES.iter().map(|r| r.0).collect();
    assert_eq!(
        codes,
        [
            "HEADING_SKIP",
            "IMAGE_NO_DESCR",
            "TABLE_NO_HEADER_ROW",
            "MISSING_LANG",
            "LITERAL_BULLET",
            "EMPTY_SPACER_PARAGRAPH",
            "DIRECT_FORMATTING_OVERRIDES_STYLE",
            "STALE_FIELD_CACHE",
            "FONT_SUBSTITUTED",
        ]
    );
    assert!(
        RULES
            .iter()
            .all(|r| ["a11y", "style", "structure"].contains(&r.1))
    );
    assert!(
        RULES
            .iter()
            .all(|r| ["error", "warning", "info"].contains(&r.2))
    );
    assert_eq!(
        RULES.iter().find(|r| r.0 == "IMAGE_NO_DESCR").unwrap().2,
        "error"
    );
}

#[test]
fn findings_carry_set_and_severity_from_the_table() {
    let image = format!(r#"{IMAGE_PREFIX}<wp:docPr id="1" name="Pic"/>{IMAGE_SUFFIX}"#);
    let found = audit(&docx(&image), &["IMAGE_NO_DESCR"]).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].rule_set, "a11y");
    assert_eq!(found[0].severity, "error");
    assert_eq!(found[0].location, "body:p:0");
    assert!(!found[0].message.is_empty());
}

#[test]
fn an_unknown_rule_is_an_error() {
    let err = audit(&docx(&para("a")), &["NOT_A_RULE"]).unwrap_err();
    assert!(matches!(err, AuditError::UnknownRule(ref r) if r == "NOT_A_RULE"));
    assert!(err.to_string().contains("NOT_A_RULE"));
}

#[test]
fn a_broken_package_is_an_inspect_error() {
    let err = audit(b"not a zip", &[]).unwrap_err();
    assert!(matches!(err, AuditError::Inspect(_)), "{err:?}");
}

// ---- HEADING_SKIP ---------------------------------------------------------

#[test]
fn heading_skip_fires_when_the_first_heading_is_not_level_one() {
    let body =
        r#"<w:p><w:pPr><w:pStyle w:val="Heading2"/></w:pPr><w:r><w:t>Start</w:t></w:r></w:p>"#;
    let found = audit(&docx(body), &["HEADING_SKIP"]).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location, "body:p:0");
}

#[test]
fn heading_levels_come_from_style_names_and_steps_down_are_fine() {
    // Localized style ids: the level comes from `w:name` "heading N".
    let body = String::new()
        + r#"<w:p><w:pPr><w:pStyle w:val="Titulo1"/></w:pPr><w:r><w:t>A</w:t></w:r></w:p>"#
        + r#"<w:p><w:pPr><w:pStyle w:val="Titulo2"/></w:pPr><w:r><w:t>B</w:t></w:r></w:p>"#
        + r#"<w:p><w:pPr><w:pStyle w:val="Titulo1"/></w:pPr><w:r><w:t>C</w:t></w:r></w:p>"#;
    assert!(codes(&with_styles(&body, &styles_with_lang()), &["HEADING_SKIP"]).is_empty());
}

// ---- IMAGE_NO_DESCR -------------------------------------------------------

#[test]
fn an_image_with_descr_or_marked_decorative_passes() {
    let described =
        format!(r#"{IMAGE_PREFIX}<wp:docPr id="1" name="Pic" descr="A whale"/>{IMAGE_SUFFIX}"#);
    let decorative = format!(
        r#"{IMAGE_PREFIX}<wp:docPr id="2" name="Rule"><a:extLst xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:ext uri="{{C183D7F6-B498-43B3-948B-1728B52AA6E4}}"><adec:decorative xmlns:adec="http://schemas.microsoft.com/office/drawing/2017/decorative" val="1"/></a:ext></a:extLst></wp:docPr>{IMAGE_SUFFIX}"#
    );
    let blank = format!(r#"{IMAGE_PREFIX}<wp:docPr id="3" name="Pic" descr="  "/>{IMAGE_SUFFIX}"#);
    let found = audit(
        &docx(&format!("{described}{decorative}{blank}")),
        &["IMAGE_NO_DESCR"],
    )
    .unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location, "body:p:2");
}

// ---- TABLE_NO_HEADER_ROW --------------------------------------------------

#[test]
fn a_table_with_a_header_row_or_one_row_passes() {
    let one_row = r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>x</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let body = format!("{}{one_row}{}", table(true), para("after"));
    assert!(codes(&docx(&body), &["TABLE_NO_HEADER_ROW"]).is_empty());
}

#[test]
fn table_finding_points_at_the_first_cell_paragraph() {
    let body = format!("{}{}", para("before"), table(false));
    let found = audit(&docx(&body), &["TABLE_NO_HEADER_ROW"]).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location, "body:p:1");
}

// ---- MISSING_LANG ---------------------------------------------------------

#[test]
fn a_language_in_doc_defaults_satisfies_missing_lang() {
    assert!(
        codes(
            &with_styles(&para("a"), &styles_with_lang()),
            &["MISSING_LANG"]
        )
        .is_empty()
    );
}

#[test]
fn a_language_on_normal_satisfies_missing_lang() {
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/><w:rPr><w:lang w:val="pt-BR"/></w:rPr></w:style></w:styles>"#
    );
    assert!(codes(&with_styles(&para("a"), &styles), &["MISSING_LANG"]).is_empty());
}

#[test]
fn normal_found_by_name_when_no_style_is_marked_default() {
    // LibreOffice writes Normal as `style0` without `w:default`.
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:styleId="style0" w:type="paragraph"><w:name w:val="Normal"/><w:rPr><w:sz w:val="24"/><w:lang w:val="en-US"/></w:rPr></w:style></w:styles>"#
    );
    let bytes = with_styles(&format!("<w:p>{}</w:p>", sized_run("a", 24)), &styles);
    assert!(
        codes(
            &bytes,
            &["MISSING_LANG", "DIRECT_FORMATTING_OVERRIDES_STYLE"]
        )
        .is_empty()
    );
}

#[test]
fn a_styles_part_without_language_fires_at_the_styles_part() {
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:default="1" w:styleId="Normal"><w:name w:val="Normal"/></w:style></w:styles>"#
    );
    let found = audit(&with_styles(&para("a"), &styles), &["MISSING_LANG"]).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location, "word/styles.xml");
}

// ---- LITERAL_BULLET -------------------------------------------------------

#[test]
fn literal_bullet_markers_need_a_following_space() {
    let body = para("- a dash item") + &para("*bold claim*") + &para("·\tmiddle dot");
    let found = audit(&docx(&body), &["LITERAL_BULLET"]).unwrap();
    let locations: Vec<&str> = found.iter().map(|f| f.location.as_str()).collect();
    assert_eq!(locations, ["body:p:0", "body:p:2"], "{found:?}");
}

#[test]
fn literal_bullets_in_a_footer_carry_the_story_id() {
    let found = audit(
        &with_footer(&para("body"), &para("• footer item")),
        &["LITERAL_BULLET"],
    )
    .unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].location.ends_with(":p:0") && !found[0].location.starts_with("body"),
        "{found:?}"
    );
}

// ---- EMPTY_SPACER_PARAGRAPH -----------------------------------------------

#[test]
fn one_empty_paragraph_or_empty_cells_are_not_spacers() {
    let cells = r#"<w:tbl><w:tblPr/><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc><w:p/><w:p/></w:tc></w:tr></w:tbl>"#;
    let body = para("a") + "<w:p/>" + &para("b") + cells + &para("c");
    assert!(codes(&docx(&body), &["EMPTY_SPACER_PARAGRAPH"]).is_empty());
}

#[test]
fn spacers_before_a_section_break_are_exempt() {
    let body = para("a")
        + "<w:p/>"
        + r#"<w:p><w:pPr><w:sectPr><w:pgSz w:w="12240" w:h="15840"/></w:sectPr></w:pPr></w:p>"#
        + &para("b");
    assert!(codes(&docx(&body), &["EMPTY_SPACER_PARAGRAPH"]).is_empty());
}

#[test]
fn a_run_of_spacers_is_one_finding_at_its_first_paragraph() {
    let body = para("a") + "<w:p/><w:p/><w:p/>" + &para("b");
    let found = audit(&docx(&body), &["EMPTY_SPACER_PARAGRAPH"]).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location, "body:p:1");
    assert_eq!(found[0].severity, "info");
}

// ---- DIRECT_FORMATTING_OVERRIDES_STYLE ------------------------------------

#[test]
fn most_runs_overriding_normal_size_is_a_finding() {
    let p = format!(
        "<w:p>{}{}{}{}</w:p>",
        sized_run("a", 20),
        sized_run("b", 20),
        sized_run("c", 20),
        sized_run("d", 24)
    );
    let found = audit(
        &with_styles(&p, &styles_with_lang()),
        &["DIRECT_FORMATTING_OVERRIDES_STYLE"],
    )
    .unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location, "word/document.xml");
    assert!(found[0].message.contains("3 of 4"), "{found:?}");
}

#[test]
fn few_overrides_or_a_font_matching_the_style_pass() {
    let p = format!(
        r#"<w:p>{}<w:r><w:rPr><w:rFonts w:ascii="Calibri" w:hAnsi="Calibri"/></w:rPr><w:t>b</w:t></w:r><w:r><w:t>c</w:t></w:r><w:r><w:t>d</w:t></w:r></w:p>"#,
        sized_run("a", 20)
    );
    assert!(
        codes(
            &with_styles(&p, &styles_with_lang()),
            &["DIRECT_FORMATTING_OVERRIDES_STYLE"]
        )
        .is_empty()
    );
}

#[test]
fn a_direct_font_differing_from_normal_counts() {
    let p = r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="Arial" w:hAnsi="Arial"/></w:rPr><w:t>a</w:t></w:r><w:r><w:t>b</w:t></w:r></w:p>"#;
    assert_eq!(
        codes(
            &with_styles(p, &styles_with_lang()),
            &["DIRECT_FORMATTING_OVERRIDES_STYLE"]
        ),
        ["DIRECT_FORMATTING_OVERRIDES_STYLE"]
    );
}

// ---- STALE_FIELD_CACHE ----------------------------------------------------

#[test]
fn a_toc_with_a_result_is_not_stale() {
    let toc = r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TOC \o "1-3" </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>Intro 1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
    let report = audit_report(&docx(toc), &["STALE_FIELD_CACHE"]).unwrap();
    assert!(report.findings.is_empty(), "{report:?}");
    // No NUMPAGES field, so no layout pass was needed.
    assert!(!report.layout);
}

#[test]
fn a_toc_without_separate_and_a_simple_toc_are_stale() {
    let open = r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText>TOC</w:instrText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#;
    let simple = r#"<w:p><w:fldSimple w:instr=" toc \h "/></w:p>"#;
    let found = audit(&docx(&format!("{open}{simple}")), &["STALE_FIELD_CACHE"]).unwrap();
    let locations: Vec<&str> = found.iter().map(|f| f.location.as_str()).collect();
    assert_eq!(locations, ["body:p:0", "body:p:1"], "{found:?}");
}

#[test]
fn a_numpages_cache_that_disagrees_with_the_layout_is_stale() {
    let report = audit_report(
        &with_footer(&para("one page"), &numpages("99")),
        &["STALE_FIELD_CACHE"],
    )
    .unwrap();
    assert!(report.layout);
    assert_eq!(report.findings.len(), 1, "{report:?}");
    assert!(report.findings[0].message.contains("99"), "{report:?}");
    assert!(report.findings[0].location.ends_with(":p:0"), "{report:?}");
}

#[test]
fn a_numpages_cache_that_matches_the_layout_is_current() {
    let report = audit_report(
        &with_footer(&para("one page"), &numpages("1")),
        &["STALE_FIELD_CACHE"],
    )
    .unwrap();
    assert!(report.layout);
    assert!(report.findings.is_empty(), "{report:?}");
}

// ---- FONT_SUBSTITUTED -----------------------------------------------------

#[test]
fn a_font_that_is_not_installed_is_substituted() {
    let p = r#"<w:p><w:r><w:rPr><w:rFonts w:ascii="Nonexistent Jubarte Sans" w:hAnsi="Nonexistent Jubarte Sans"/></w:rPr><w:t>hello</w:t></w:r></w:p>"#;
    let report = audit_report(&docx(p), &["FONT_SUBSTITUTED"]).unwrap();
    assert!(report.layout);
    assert!(
        report.findings.iter().any(|f| f.code == "FONT_SUBSTITUTED"
            && f.severity == "info"
            && f.message.contains("Nonexistent Jubarte Sans")),
        "{report:?}"
    );
}

#[test]
fn selecting_only_paragraph_rules_skips_the_layout_pass() {
    let report = audit_report(&docx(&para("a")), &["a11y", "style"]).unwrap();
    assert!(!report.layout);
    assert!(report.rules.iter().all(|c| c != "FONT_SUBSTITUTED"));
}

#[test]
fn no_rules_selects_every_rule() {
    let report = audit_report(&docx(&para("a")), &[]).unwrap();
    assert_eq!(report.rules.len(), RULES.len());
    assert!(report.layout);
}

// ---- CLI ------------------------------------------------------------------

fn run_cli(bytes: &[u8], args: &[&str]) -> (i32, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("in.docx");
    std::fs::write(&path, bytes).unwrap();
    let out = Command::new(BIN)
        .arg("audit")
        .arg(&path)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn cli_exits_zero_without_findings() {
    let (code, stdout, _) = run_cli(
        &with_styles(&para("clean"), &styles_with_lang()),
        &["--rules", "a11y,style"],
    );
    assert_eq!(code, 0, "{stdout}");
}

#[test]
fn cli_exits_zero_on_warnings_and_two_with_strict() {
    let bytes = docx(&para("• item"));
    let (code, stdout, _) = run_cli(&bytes, &["--rules", "LITERAL_BULLET"]);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("LITERAL_BULLET"), "{stdout}");
    assert!(stdout.contains("body:p:0"), "{stdout}");
    let (code, _, _) = run_cli(&bytes, &["--rules", "LITERAL_BULLET", "--strict"]);
    assert_eq!(code, 2);
}

#[test]
fn cli_exits_two_on_an_error_finding_and_info_never_fails() {
    let image = format!(r#"{IMAGE_PREFIX}<wp:docPr id="1" name="Pic"/>{IMAGE_SUFFIX}"#);
    let (code, _, _) = run_cli(&docx(&image), &["--rules", "a11y"]);
    assert_eq!(code, 2);
    let spacers = para("a") + "<w:p/><w:p/>" + &para("b");
    let (code, _, _) = run_cli(
        &docx(&spacers),
        &["--rules", "EMPTY_SPACER_PARAGRAPH", "--strict"],
    );
    assert_eq!(code, 0);
}

#[test]
fn cli_json_reports_findings_rules_and_layout() {
    let (code, stdout, _) = run_cli(&docx(&para("• item")), &["--json", "--rules", "style"]);
    assert_eq!(code, 0);
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["layout"], false);
    assert_eq!(value["findings"][0]["code"], "LITERAL_BULLET");
    assert_eq!(value["findings"][0]["rule_set"], "style");
    assert_eq!(value["findings"][0]["severity"], "warning");
    assert_eq!(value["findings"][0]["location"], "body:p:0");
    assert!(value["rules"].as_array().unwrap().len() == 3);
}

#[test]
fn cli_unknown_rule_exits_one_with_an_error() {
    let (code, _, stderr) = run_cli(&docx(&para("a")), &["--rules", "BOGUS"]);
    assert_eq!(code, 1);
    assert!(stderr.contains("BOGUS"), "{stderr}");
}

// ---- capabilities ---------------------------------------------------------

#[test]
fn capabilities_list_audit_and_every_rule_code() {
    let manifest = jubarte::capabilities::capabilities("rust");
    assert!(manifest.operations.audit);
    let codes: Vec<&str> = RULES.iter().map(|rule| rule.0).collect();
    assert_eq!(manifest.audit_rules, codes);
    let json: serde_json::Value =
        serde_json::from_str(&jubarte::capabilities::capabilities_json("cli")).unwrap();
    assert_eq!(json["operations"]["audit"], true);
    assert_eq!(json["audit_rules"].as_array().unwrap().len(), RULES.len());
}

// ---- without a layout pass (the slim WASM build) ---------------------------

#[test]
fn without_layout_the_default_selection_leaves_out_font_substitution() {
    let report = audit_report_with(&docx(&para("• item")), &[], None).unwrap();
    assert!(!report.layout);
    assert_eq!(report.rules.len(), RULES.len() - 1);
    assert!(report.rules.iter().all(|code| code != "FONT_SUBSTITUTED"));
    assert!(report.findings.iter().any(|f| f.code == "LITERAL_BULLET"));
}

#[test]
fn without_layout_asking_for_font_substitution_is_an_error() {
    for rules in [&["FONT_SUBSTITUTED"][..], &["structure"][..]] {
        let err = audit_report_with(&docx(&para("a")), rules, None).unwrap_err();
        assert!(matches!(err, AuditError::Layout(_)), "{err:?}");
        assert!(err.to_string().contains("FONT_SUBSTITUTED"), "{err}");
    }
}

#[test]
fn without_layout_stale_fields_check_caches_but_not_page_counts() {
    let numeric = audit_report_with(
        &with_footer(&para("one page"), &numpages("99")),
        &["STALE_FIELD_CACHE"],
        None,
    )
    .unwrap();
    assert!(!numeric.layout);
    assert!(numeric.findings.is_empty(), "{numeric:?}");
    let blank = audit_report_with(
        &with_footer(&para("one page"), &numpages(" ")),
        &["STALE_FIELD_CACHE"],
        None,
    )
    .unwrap();
    assert_eq!(blank.findings.len(), 1, "{blank:?}");
}

#[test]
fn a_supplied_layout_pass_is_used_and_its_failure_reported() {
    let mut calls = 0;
    let mut fail = |_: &[u8]| {
        calls += 1;
        Err(AuditError::Layout("renderer unavailable".into()))
    };
    let err =
        audit_report_with(&docx(&para("a")), &["FONT_SUBSTITUTED"], Some(&mut fail)).unwrap_err();
    assert!(err.to_string().contains("renderer unavailable"), "{err}");
    assert_eq!(calls, 1);
}
