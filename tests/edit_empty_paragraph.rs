// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Text inserted into an empty paragraph, as Word types it (plan 13): a new
//! run formatted as the paragraph mark, without the mark's own revisions,
//! in our insertion. The paragraph's properties keep their content, through
//! the comparer and under `keep` alike.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::{assert_word_valid_package, check_word_valid_package};
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, EditResult, apply_plan};
use jubarte::inspect::paragraphs;

const MARK_PPR: &str =
    r#"<w:pPr><w:jc w:val="center"/><w:rPr><w:b/><w:sz w:val="28"/></w:rPr></w:pPr>"#;

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

/// The XML of the body's paragraph `n`, `<w:p` to its `</w:p>`.
fn paragraph_xml(bytes: &[u8], n: usize) -> String {
    let xml = part_string(bytes, "word/document.xml").expect("word/document.xml");
    let starts: Vec<usize> = xml
        .match_indices("<w:p")
        .map(|(i, _)| i)
        .filter(|&i| matches!(xml.as_bytes()[i + 4], b'>' | b' ' | b'/'))
        .collect();
    let start = starts[n];
    let end = xml[start..]
        .find("</w:p>")
        .map_or(xml.len(), |e| start + e + 6);
    xml[start..end].to_string()
}

/// `xml` as the serializer writes it: `<x />`, and no `xml:space` on a run's
/// text.
fn normalized(xml: &str) -> String {
    xml.replace("/>", " />")
        .replace("  />", " />")
        .replace(r#" xml:space="preserve""#, "")
}

fn ppr(paragraph: &str) -> String {
    let paragraph = normalized(paragraph);
    paragraph.find("<w:pPr>").map_or(String::new(), |s| {
        paragraph[s..paragraph.find("</w:pPr>").unwrap() + 8].to_string()
    })
}

fn plan(mode: &str, operation: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-19T12:00:00Z"{mode},"operations":[{operation}]}}"#
    ))
    .unwrap()
}

const MODES: [&str; 2] = ["", r#","existing_revisions":"keep""#];

/// Insert `text` into paragraph `p` of `source` in both modes, and check
/// the redline, Accept All, Reject All and the clean copy.
fn check(source: &[u8], p: usize, expected_rpr: &str) {
    check_in(&MODES, source, p, expected_rpr);
}

fn check_in(modes: &[&str], source: &[u8], p: usize, expected_rpr: &str) {
    let before = texts(source);
    for &mode in modes {
        for position in ["start", "end"] {
            let op = format!(
                r#"{{"kind":"insert","paragraph":"p{p}","position":"{position}","text":"New words."}}"#
            );
            let result = apply_plan(source, &plan(mode, &op)).unwrap();
            let mut after = before.clone();
            after[p] = "New words.".into();
            assert_eq!(texts(&result.clean), after, "{mode} {position}");
            assert_eq!(
                texts(&accept_revisions(&result.redline).unwrap()),
                after,
                "{mode} {position}"
            );
            // Reject All gives the source back with its own revisions
            // rejected too.
            assert_eq!(
                texts(&reject_revisions(&result.redline).unwrap()),
                texts(&reject_revisions(source).unwrap()),
                "{mode} {position}"
            );
            let red = normalized(&paragraph_xml(&result.redline, p));
            assert_eq!(ppr(&red), ppr(&paragraph_xml(source, p)), "{mode}: {red}");
            let ins = red
                .find("<w:ins ")
                .unwrap_or_else(|| panic!("{mode}: {red}"));
            let run = &red[ins..];
            let run = &run[run.find("<w:r>").unwrap_or_else(|| panic!("{mode}: {red}"))..];
            assert!(
                run.starts_with(&format!("<w:r>{expected_rpr}<w:t>New words.</w:t></w:r>")),
                "{mode}: {red}"
            );
            assert_word_valid_package(&result.redline);
            assert_word_valid_package(&result.clean);
        }
    }
}

#[test]
fn insert_into_an_empty_paragraph_creates_a_run_from_the_marks_rpr() {
    let source = docx(&format!(
        "{}<w:p>{MARK_PPR}</w:p>{}",
        para("Before."),
        para("After.")
    ));
    check(&source, 1, r#"<w:rPr><w:b /><w:sz w:val="28" /></w:rPr>"#);
}

/// The mark's own revision (`w:ins` in its `w:rPr`) is not copied into the
/// new run: its id must stay unique.
#[test]
fn insert_into_an_empty_paragraph_leaves_the_marks_revision_out_of_the_run() {
    let source = docx(&format!(
        r#"{}<w:p><w:pPr><w:rPr><w:ins w:id="9" w:author="Other" w:date="2026-09-01T00:00:00Z"/><w:i/></w:rPr></w:pPr></w:p>{}"#,
        para("Before."),
        para("After.")
    ));
    // A source with a revision is edited under keep, or settled first.
    check_in(&MODES[1..], &source, 1, "<w:rPr><w:i /></w:rPr>");
}

/// A mark set as math (`w:oMath`) keeps it, and the new run takes the rest
/// of its formatting: the validator refuses `w:oMath` in a run's `w:rPr`.
#[test]
fn insert_into_an_empty_paragraph_leaves_the_marks_math_flag_out_of_the_run() {
    let source = docx(&format!(
        "{}<w:p><w:pPr><w:rPr><w:b/><w:bCs/><w:specVanish/><w:oMath/></w:rPr></w:pPr></w:p>{}",
        para("Before."),
        para("After.")
    ));
    check(
        &source,
        1,
        "<w:rPr><w:b /><w:bCs /><w:specVanish /></w:rPr>",
    );
}

#[test]
fn insert_into_an_empty_paragraph_without_mark_rpr() {
    let source = docx(&format!("{}<w:p/>{}", para("Before."), para("After.")));
    check(&source, 1, "");
}

#[test]
fn insert_into_an_empty_paragraph_in_a_table_cell() {
    let cell = |inner: &str| {
        format!("<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc>")
    };
    let source = docx(&format!(
        "{}<w:tbl><w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl>{}",
        para("Before."),
        cell(&para("Full.")),
        cell(&format!("<w:p>{MARK_PPR}</w:p>")),
        para("After.")
    ));
    check(&source, 2, r#"<w:rPr><w:b /><w:sz w:val="28" /></w:rPr>"#);
}

/// The plan's acceptance line: `edit FILE -p p1 --content "text"` on an
/// empty p1 writes the text in our insertion.
#[test]
fn content_on_an_empty_paragraph_writes_the_text() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.docx");
    std::fs::write(
        &input,
        docx(&format!(
            "{}<w:p>{MARK_PPR}</w:p>{}",
            para("Before."),
            para("After.")
        )),
    )
    .unwrap();
    let output = dir.path().join("out.docx");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("edit")
        .arg(&input)
        .args([
            "-p",
            "p1",
            "--content",
            "text",
            "--author",
            "Ann Counsel",
            "-o",
        ])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let redline = std::fs::read(&output).unwrap();
    let red = paragraph_xml(&redline, 1);
    assert!(
        red.contains(
            r#"<w:r><w:rPr><w:b /><w:sz w:val="28" /></w:rPr><w:t>text</w:t></w:r></w:ins>"#
        ),
        "{red}"
    );
    assert_eq!(texts(&accept_revisions(&redline).unwrap())[1], "text");
}

/// A v2 plan's `rewrite` and edge `insert` on an empty paragraph write the
/// text the same way.
#[test]
fn a_v2_plan_writes_into_an_empty_paragraph() {
    use jubarte::edit::{
        for_edit::ForEditRequest,
        template::{TemplateRequest, generate},
        v2::PlanV2,
        v2_apply,
    };
    use serde_json::{Value, json};
    let source = docx(&format!(
        "{}<w:p>{MARK_PPR}</w:p>{}",
        para("Before."),
        para("After.")
    ));
    let generated = generate(
        &source,
        &TemplateRequest {
            view: ForEditRequest::default(),
            author: "Ann Counsel".into(),
            date: "2026-10-19T12:00:00Z".into(),
        },
    )
    .unwrap();
    let template: Value = serde_json::from_str(&generated.plan.to_json()).unwrap();
    let empty = template["operations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|op| op["text"] == "")
        .unwrap_or_else(|| panic!("no operation for the empty paragraph: {template}"))
        .clone();
    let rewrite = {
        let mut op = empty.clone();
        op["text"] = json!("New words.");
        op
    };
    let insert = {
        let mut op = empty.clone();
        op.as_object_mut().unwrap().remove("before_sha256");
        op["kind"] = json!("insert");
        op["edge"] = json!("start");
        op["text"] = json!("New words.");
        op
    };
    for op in [rewrite, insert] {
        let mut q = template.clone();
        q["operations"] = json!([op]);
        let result = v2_apply::apply_plan(&source, &PlanV2::from_json(&q.to_string()).unwrap())
            .unwrap_or_else(|e| panic!("{op}: {e:?}"));
        assert_eq!(
            texts(&result.clean),
            ["Before.", "New words.", "After."],
            "{op}"
        );
        assert_eq!(
            texts(&accept_revisions(&result.redline).unwrap()),
            ["Before.", "New words.", "After."],
            "{op}"
        );
        assert_eq!(
            texts(&reject_revisions(&result.redline).unwrap()),
            texts(&source)
        );
        let red = normalized(&paragraph_xml(&result.redline, 1));
        assert_eq!(ppr(&red), ppr(&paragraph_xml(&source, 1)), "{red}");
        assert!(
            red.contains(r#"<w:r><w:rPr><w:b /><w:sz w:val="28" /></w:rPr><w:t>New words.</w:t></w:r></w:ins>"#),
            "{op}: {red}"
        );
        assert_word_valid_package(&result.redline);
    }
    // Inserts at one edge go in array order (#1303). Both edges of an empty
    // paragraph are one source point, so mixing them is the design's
    // ambiguous boundary ownership, and refuses.
    let at = |edge: &str, text: &str| {
        let mut op = empty.clone();
        op.as_object_mut().unwrap().remove("before_sha256");
        op["kind"] = json!("insert");
        op["edge"] = json!(edge);
        op["text"] = json!(text);
        op
    };
    let apply = |operations: Value| {
        let mut q = template.clone();
        q["operations"] = operations;
        v2_apply::apply_plan(&source, &PlanV2::from_json(&q.to_string()).unwrap())
    };
    for (operations, read) in [
        (json!([at("end", "C "), at("end", "D")]), "C D"),
        (json!([at("start", "A "), at("start", "B")]), "A B"),
    ] {
        let result = apply(operations.clone()).unwrap_or_else(|e| panic!("{operations}: {e:?}"));
        assert_eq!(texts(&result.clean)[1], read, "{operations}");
        assert_eq!(
            texts(&accept_revisions(&result.redline).unwrap())[1],
            read,
            "{operations}"
        );
        assert_word_valid_package(&result.redline);
    }
    let mixed = apply(json!([at("end", " C"), at("start", "A ")])).unwrap_err();
    assert_eq!(mixed.code, "CONFLICT", "{mixed:?}");
}

/// An empty paragraph in a header or a footer takes the text the same way.
#[test]
fn insert_into_an_empty_header_or_footer_paragraph() {
    for (fixture, location, part) in [
        (
            "tests/fixtures/nested_table/hf_a.docx",
            "header1.p1",
            "word/header1.xml",
        ),
        (
            "tests/fixtures/agent-view/received.docx",
            "footer1.p1",
            "word/footer1.xml",
        ),
    ] {
        let source = std::fs::read(fixture).unwrap();
        let op = format!(
            r#"{{"kind":"insert","paragraph":"{location}","position":"start","text":"New words."}}"#
        );
        let result = apply_plan(&source, &plan(r#","existing_revisions":"keep""#, &op))
            .unwrap_or_else(|e| panic!("{location}: {e:?}"));
        let red = part_string(&result.redline, part).unwrap();
        let at = red
            .find("New words.")
            .unwrap_or_else(|| panic!("{location}: {red}"));
        let ins = red[..at].rfind("<w:ins ").unwrap();
        assert!(!red[ins..at].contains("</w:ins>"), "{location}: {red}");
        let accepted = part_string(&accept_revisions(&result.redline).unwrap(), part).unwrap();
        assert!(accepted.contains("New words."), "{location}: {accepted}");
        let rejected = part_string(&reject_revisions(&result.redline).unwrap(), part).unwrap();
        assert!(!rejected.contains("New words."), "{location}: {rejected}");
        // The fixture's own findings (received.docx's commentsExtensible
        // misses two durableIds), and no new one.
        let findings = |bytes: &[u8]| {
            let mut errors = check_word_valid_package(bytes).errors;
            errors.sort();
            errors
        };
        assert_eq!(findings(&result.redline), findings(&source), "{location}");
    }
}

/// A paragraph with no text may still hold a field, a bookmark's or a
/// comment's markers: text inserted at its start goes before them, at its
/// end after them, so it joins no range it was not typed in (#1156).
#[test]
fn insert_at_either_edge_goes_beside_what_the_paragraph_holds() {
    let field = r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#;
    let bookmark = r#"<w:bookmarkStart w:id="0" w:name="mark"/><w:bookmarkEnd w:id="0"/>"#;
    let permission = r#"<w:permStart w:id="5" w:edGrp="everyone"/><w:permEnd w:id="5"/>"#;
    for (holds, first, last) in [
        (
            field,
            r#"<w:fldChar w:fldCharType="begin""#,
            r#"<w:fldChar w:fldCharType="end""#,
        ),
        (bookmark, "<w:bookmarkStart", "<w:bookmarkEnd"),
        (permission, "<w:permStart", "<w:permEnd"),
    ] {
        let source = docx(&format!(
            "{}<w:p>{MARK_PPR}{holds}</w:p>{}",
            para("Before."),
            para("After.")
        ));
        // A rewrite starts where the paragraph does, as `start` does.
        let ops = [("start", true), ("end", false), ("rewrite", true)];
        for &mode in &MODES {
            for (position, goes_first) in ops {
                let op = if position == "rewrite" {
                    r#"{"kind":"rewrite","paragraph":"p1","text":"New words."}"#.to_string()
                } else {
                    format!(
                        r#"{{"kind":"insert","paragraph":"p1","position":"{position}","text":"New words."}}"#
                    )
                };
                let result = apply_plan(&source, &plan(mode, &op)).unwrap();
                let red = normalized(&paragraph_xml(&result.redline, 1));
                let ins = red.find("<w:ins ").unwrap_or_else(|| panic!("{red}"));
                let (first, last) = (red.find(first).unwrap(), red.find(last).unwrap());
                if goes_first {
                    assert!(ins < first, "{mode} {position}: {red}");
                } else {
                    assert!(last < ins, "{mode} {position}: {red}");
                }
                assert_eq!(ppr(&red), ppr(&paragraph_xml(&source, 1)), "{mode}: {red}");
                assert_eq!(
                    texts(&accept_revisions(&result.redline).unwrap())[1],
                    texts(&result.clean)[1],
                    "{mode} {position}"
                );
                assert_word_valid_package(&result.redline);
                assert_word_valid_package(&result.clean);
            }
        }
    }
}

/// The text of the `w:t`s in `xml`, in order.
fn run_texts(xml: &str) -> String {
    xml.split("<w:t>")
        .skip(1)
        .map(|t| &t[..t.find("</w:t>").unwrap()])
        .collect()
}

/// Several inserts into one empty paragraph go to the edges they name, each
/// edge in plan order, with what the paragraph holds between them (#1303).
#[test]
fn several_inserts_into_an_empty_paragraph_keep_their_edges() {
    let field = r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#;
    let bookmark = r#"<w:bookmarkStart w:id="0" w:name="mark"/><w:bookmarkEnd w:id="0"/>"#;
    let insert = |position: &str, text: &str| {
        format!(r#"{{"kind":"insert","paragraph":"p1","position":"{position}","text":"{text}"}}"#)
    };
    let ops = [
        insert("end", " C"),
        insert("start", "A "),
        insert("end", " D"),
    ]
    .join(",");
    for (holds, first, last) in [
        ("", "", ""),
        (
            field,
            r#"<w:fldChar w:fldCharType="begin""#,
            r#"<w:fldChar w:fldCharType="end""#,
        ),
        (bookmark, "<w:bookmarkStart", "<w:bookmarkEnd"),
    ] {
        let source = docx(&format!(
            "{}<w:p>{MARK_PPR}{holds}</w:p>{}",
            para("Before."),
            para("After.")
        ));
        for &mode in &MODES {
            let result = apply_plan(&source, &plan(mode, &ops)).unwrap();
            let red = normalized(&paragraph_xml(&result.redline, 1));
            assert_eq!(texts(&result.clean)[1], "A  C D", "{mode} {holds}: {red}");
            assert_eq!(
                texts(&accept_revisions(&result.redline).unwrap())[1],
                "A  C D",
                "{mode} {holds}: {red}"
            );
            assert_eq!(
                texts(&reject_revisions(&result.redline).unwrap()),
                texts(&source),
                "{mode} {holds}"
            );
            if !holds.is_empty() {
                let (first, last) = (red.find(first).unwrap(), red.find(last).unwrap());
                assert_eq!(run_texts(&red[..first]), "A ", "{mode}: {red}");
                assert_eq!(run_texts(&red[last..]), " C D", "{mode}: {red}");
            }
            assert_eq!(ppr(&red), ppr(&paragraph_xml(&source, 1)), "{mode}: {red}");
            assert_word_valid_package(&result.redline);
            assert_word_valid_package(&result.clean);
        }
    }
}

/// A `format` and a comment on one of several inserts into an empty
/// paragraph cover that insert's own words (#1303).
#[test]
fn a_format_and_a_comment_follow_their_insert_into_an_empty_paragraph() {
    let source = docx(&format!(
        "{}<w:p>{MARK_PPR}</w:p>{}",
        para("Before."),
        para("After.")
    ));
    let ops = [
        r#"{"kind":"insert","paragraph":"p1","position":"end","text":" C","format":{"bold":false},"comment":"Why C?"}"#,
        r#"{"kind":"insert","paragraph":"p1","position":"start","text":"A "}"#,
    ]
    .join(",");
    for &mode in &MODES {
        let result = apply_plan(&source, &plan(mode, &ops)).unwrap();
        let red = normalized(&paragraph_xml(&result.redline, 1));
        assert_eq!(
            texts(&accept_revisions(&result.redline).unwrap())[1],
            "A  C",
            "{mode}: {red}"
        );
        let unbold = red
            .find(r#"<w:b w:val="0" />"#)
            .unwrap_or_else(|| panic!("{mode}: {red}"));
        assert_eq!(
            run_texts(&red[unbold..]).trim_start(),
            "C",
            "{mode}: only C loses the bold: {red}"
        );
        let open = red
            .find("<w:commentRangeStart")
            .unwrap_or_else(|| panic!("{mode}: {red}"));
        let close = red.find("<w:commentRangeEnd").unwrap();
        assert_eq!(run_texts(&red[open..close]), " C", "{mode}: {red}");
        assert_word_valid_package(&result.redline);
    }
}

/// `format` on the new words is applied over the mark's formatting.
#[test]
fn format_on_an_insert_into_an_empty_paragraph_applies_to_the_new_run() {
    let source = docx(&format!(
        "{}<w:p>{MARK_PPR}</w:p>{}",
        para("Before."),
        para("After.")
    ));
    for &mode in &MODES {
        let op = r#"{"kind":"insert","paragraph":"p1","position":"end","text":"Plain.","format":{"bold":false}}"#;
        let result = apply_plan(&source, &plan(mode, op)).unwrap();
        let red = normalized(&paragraph_xml(&result.redline, 1));
        let run = &red[red.find("<w:ins ").unwrap()..];
        let rpr = &run[run.find("<w:rPr>").unwrap()..run.find("</w:rPr>").unwrap()];
        assert!(rpr.contains(r#"<w:b w:val="0" />"#), "{mode}: {red}");
        assert!(rpr.contains(r#"<w:sz w:val="28" />"#), "{mode}: {red}");
        assert_word_valid_package(&result.redline);
    }
}

/// In editing mode the text lands as a plain run formatted as the mark.
#[test]
fn editing_mode_writes_a_plain_run_into_an_empty_paragraph() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("in.docx");
    std::fs::write(
        &input,
        docx(&format!(
            "{}<w:p>{MARK_PPR}</w:p>{}",
            para("Before."),
            para("After.")
        )),
    )
    .unwrap();
    let output = dir.path().join("out.docx");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("edit")
        .arg(&input)
        .args(["-p", "p1", "--content", "text", "--editing-mode", "-o"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let edited = std::fs::read(&output).unwrap();
    let red = normalized(&paragraph_xml(&edited, 1));
    assert!(!red.contains("<w:ins "), "{red}");
    assert!(
        red.contains(r#"<w:r><w:rPr><w:b /><w:sz w:val="28" /></w:rPr><w:t>text</w:t></w:r>"#),
        "{red}"
    );
    assert_word_valid_package(&edited);
}

/// A table between two paragraphs whose second cell holds `cell`; the
/// first cell holds "Full.". Body paragraphs: p0 "Before.", p1 "Full.",
/// p2 the second cell's first paragraph, then the cell's others and
/// "After.".
fn table_with_cell(cell: &str) -> Vec<u8> {
    let tc = |inner: &str| {
        format!("<w:tc><w:tcPr><w:tcW w:w=\"2000\" w:type=\"dxa\"/></w:tcPr>{inner}</w:tc>")
    };
    docx(&format!(
        "{}<w:tbl><w:tblPr><w:tblW w:w=\"4000\" w:type=\"dxa\"/></w:tblPr><w:tblGrid><w:gridCol w:w=\"2000\"/><w:gridCol w:w=\"2000\"/></w:tblGrid><w:tr>{}{}</w:tr></w:tbl>{}",
        para("Before."),
        tc(&para("Full.")),
        tc(cell),
        para("After.")
    ))
}

fn plan_of(mode: &str, operations: &[String]) -> EditPlan {
    plan(mode, &operations.join(","))
}

fn delete(p: &str) -> String {
    format!(r#"{{"kind":"delete_paragraph","paragraph":"{p}"}}"#)
}

fn insert(p: &str, side: &str, text: &str) -> String {
    format!(
        r#"{{"kind":"insert_paragraph","paragraph":"{p}","position":"{side}","runs":[{{"text":"{text}"}}]}}"#
    )
}

/// Every `w:tc` in `part` of `bytes` ends with a paragraph.
fn assert_cells_end_in_a_paragraph(bytes: &[u8], what: &str) {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    for (at, _) in xml.match_indices("</w:tc>") {
        assert!(
            xml[..at].ends_with("</w:p>") || xml[..at].ends_with("<w:p />"),
            "{what}: a cell does not end in a paragraph: {xml}"
        );
    }
}

/// The clean copy, the redline, and the redline accepted and rejected:
/// each Word-valid, every cell ending in a paragraph. Accept All reads as
/// the clean copy, Reject All as the source.
fn check_outputs(source: &[u8], plan: &EditPlan, expected: &[&str], what: &str) -> EditResult {
    let result = apply_plan(source, plan).unwrap_or_else(|e| panic!("{what}: {e:?}"));
    let accepted = accept_revisions(&result.redline).unwrap();
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_eq!(texts(&result.clean), expected, "{what}: clean");
    assert_eq!(texts(&accepted), expected, "{what}: Accept All");
    assert_eq!(
        texts(&rejected),
        texts(&reject_revisions(source).unwrap()),
        "{what}: Reject All"
    );
    for (bytes, output) in [
        (&result.clean, "clean"),
        (&result.redline, "redline"),
        (&accepted, "Accept All"),
        (&rejected, "Reject All"),
    ] {
        let report = check_word_valid_package(bytes);
        assert!(report.ok(), "{what}: {output}: {:?}", report.errors);
        assert_cells_end_in_a_paragraph(bytes, &format!("{what}: {output}"));
    }
    result
}

/// Each `w:p` in `xml`, which nests no paragraph.
fn paragraphs_in(xml: &str) -> Vec<String> {
    xml.match_indices("<w:p")
        .map(|(i, _)| i)
        .filter(|&i| matches!(xml.as_bytes()[i + 4], b'>' | b' '))
        .map(|i| {
            let rest = &xml[i..];
            let open = rest.find('>').unwrap();
            let end = if rest.as_bytes()[open - 1] == b'/' {
                open + 1
            } else {
                rest.find("</w:p>").unwrap() + 6
            };
            rest[..end].to_string()
        })
        .collect()
}

/// The paragraphs of the table's second cell.
fn cell_paragraphs(bytes: &[u8]) -> Vec<String> {
    let xml = normalized(&part_string(bytes, "word/document.xml").unwrap());
    let start = xml.rfind("<w:tc>").unwrap();
    let end = start + xml[start..].find("</w:tc>").unwrap();
    paragraphs_in(&xml[start..end])
}

/// The paragraph mark's revision: `+` inserted, `-` deleted, or none.
fn mark(paragraph: &str) -> &'static str {
    let ppr = ppr(paragraph);
    if ppr.contains("<w:ins ") {
        "+"
    } else if ppr.contains("<w:del ") {
        "-"
    } else {
        ""
    }
}

/// The paragraph's words in document order, one entry per stretch: `+`
/// before inserted words, `-` before deleted ones, nothing before the rest.
fn words(paragraph: &str) -> Vec<String> {
    let paragraph = normalized(paragraph);
    let body = paragraph
        .find("</w:pPr>")
        .map_or(paragraph.as_str(), |e| &paragraph[e + 8..]);
    let token =
        regex::Regex::new(r"<w:(ins|del) |</w:(ins|del)>|<w:(t|delText)>([^<]*)</w:").unwrap();
    let mut side = "";
    let mut out: Vec<(&str, String)> = Vec::new();
    for c in token.captures_iter(body) {
        if let Some(open) = c.get(1) {
            side = if open.as_str() == "ins" { "+" } else { "-" };
        } else if c.get(2).is_some() {
            side = "";
        } else {
            match out.last_mut() {
                Some((s, text)) if *s == side => text.push_str(&c[4]),
                _ => out.push((side, c[4].to_string())),
            }
        }
    }
    out.into_iter().map(|(s, t)| format!("{s}{t}")).collect()
}

/// Each paragraph as its mark's revision and its words.
fn shape(paragraphs: &[String]) -> Vec<(&'static str, Vec<String>)> {
    paragraphs.iter().map(|p| (mark(p), words(p))).collect()
}

fn expect(expected: &[(&'static str, &[&str])]) -> Vec<(&'static str, Vec<String>)> {
    expected
        .iter()
        .map(|(m, w)| (*m, w.iter().map(|w| w.to_string()).collect()))
        .collect()
}

/// `paragraphs` hold `expected` (see [`shape`]) when `mode` replays the
/// edits, as Word's redline of the same change does. Without `keep`, the
/// comparer writes the redline, words aligned its own way (a deletion
/// before its insertion): there the paragraphs are only counted. No plan
/// here changes a paragraph's properties, so no paragraph records a change
/// of them.
fn assert_shape(
    mode: &str,
    paragraphs: &[String],
    expected: &[(&'static str, &[&str])],
    what: &str,
) {
    if mode == MODES[1] {
        assert_eq!(
            shape(paragraphs),
            expect(expected),
            "{what}: {paragraphs:?}"
        );
        assert!(
            paragraphs.iter().all(|p| !p.contains("<w:pPrChange")),
            "{what}: {paragraphs:?}"
        );
    } else {
        assert_eq!(paragraphs.len(), expected.len(), "{what}: {paragraphs:?}");
    }
}

/// The table's second cell holds `expected` (see [`assert_shape`]).
fn assert_cell(mode: &str, bytes: &[u8], expected: &[(&'static str, &[&str])], what: &str) {
    assert_shape(mode, &cell_paragraphs(bytes), expected, what);
}

/// The body's paragraphs hold `expected` (see [`assert_shape`]).
fn assert_body(mode: &str, bytes: &[u8], expected: &[(&'static str, &[&str])], what: &str) {
    let body: Vec<String> = (0..texts(bytes).len())
        .map(|n| paragraph_xml(bytes, n))
        .collect();
    assert_shape(mode, &body, expected, what);
}

/// Task 5 (#1668), as Word redlines it: a plan that replaces a cell's only
/// paragraph keeps that paragraph and its mark, and writes the new words in
/// it, before the deleted ones. Before or after, in either operation order,
/// in both modes.
#[test]
fn a_plan_replaces_a_cells_only_paragraph() {
    let source = table_with_cell(&para("Old."));
    for mode in MODES {
        for side in ["before", "after"] {
            for (order, operations) in [
                ("delete first", [delete("p2"), insert("p2", side, "New.")]),
                ("insert first", [insert("p2", side, "New."), delete("p2")]),
            ] {
                let what = format!("{mode} {side} {order}");
                let result = check_outputs(
                    &source,
                    &plan_of(mode, &operations),
                    &["Before.", "Full.", "New.", "After."],
                    &what,
                );
                assert_cell(mode, &result.redline, &[("", &["+New.", "-Old."])], &what);
                assert_cell(MODES[1], &result.clean, &[("", &["New."])], &what);
            }
        }
    }
}

/// Several new paragraphs on the deleted one, on one side or both, keep
/// plan order on each side. As in Word, the last one's words go in the old
/// paragraph, and the others come before it with inserted marks.
#[test]
fn several_paragraphs_replace_a_cells_only_paragraph() {
    let source = table_with_cell(&para("Old."));
    for mode in MODES {
        for (what, operations) in [
            (
                "two after",
                [
                    insert("p2", "after", "One."),
                    delete("p2"),
                    insert("p2", "after", "Two."),
                ],
            ),
            (
                "one each side",
                [
                    insert("p2", "before", "One."),
                    insert("p2", "after", "Two."),
                    delete("p2"),
                ],
            ),
            (
                "two before",
                [
                    delete("p2"),
                    insert("p2", "before", "One."),
                    insert("p2", "before", "Two."),
                ],
            ),
        ] {
            let what = format!("{mode} {what}");
            let result = check_outputs(
                &source,
                &plan_of(mode, &operations),
                &["Before.", "Full.", "One.", "Two.", "After."],
                &what,
            );
            assert_cell(
                mode,
                &result.redline,
                &[("+", &["+One."]), ("", &["+Two.", "-Old."])],
                &what,
            );
        }
    }
}

/// A cell's last paragraph replaced while an earlier one stays: Word keeps
/// the last mark here too.
#[test]
fn a_plan_replaces_a_cells_last_paragraph() {
    let source = table_with_cell(&format!("{}{}", para("Kept."), para("Old.")));
    for mode in MODES {
        let result = check_outputs(
            &source,
            &plan_of(mode, &[delete("p3"), insert("p3", "after", "New.")]),
            &["Before.", "Full.", "Kept.", "New.", "After."],
            mode,
        );
        assert_cell(
            mode,
            &result.redline,
            &[("", &["Kept."]), ("", &["+New.", "-Old."])],
            mode,
        );
    }
}

/// A new paragraph whose properties differ from the replaced one's puts
/// them on the kept mark as a tracked change: the old ones in its
/// `w:pPrChange`, back on Reject All.
#[test]
fn a_replacement_with_other_properties_tracks_them_on_the_kept_mark() {
    let source = table_with_cell(
        r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>Old.</w:t></w:r></w:p>"#,
    );
    let like = r#"{"kind":"insert_paragraph","paragraph":"p2","position":"after","like":"p0","runs":[{"text":"New."}]}"#;
    let centred = |bytes: &[u8]| cell_paragraphs(bytes).iter().any(|p| p.contains("<w:jc "));
    for mode in MODES {
        let result = check_outputs(
            &source,
            &plan_of(mode, &[delete("p2"), like.to_string()]),
            &["Before.", "Full.", "New.", "After."],
            mode,
        );
        assert!(!centred(&result.clean), "{mode}");
        assert!(
            !centred(&accept_revisions(&result.redline).unwrap()),
            "{mode}"
        );
        assert!(
            centred(&reject_revisions(&result.redline).unwrap()),
            "{mode}"
        );
        if mode == MODES[1] {
            let cell = cell_paragraphs(&result.redline);
            assert_eq!(
                shape(&cell),
                expect(&[("", &["+New.", "-Old."])]),
                "{cell:?}"
            );
            assert!(
                cell[0].starts_with("<w:p><w:pPr><w:pPrChange ")
                    && cell[0].contains(
                        r#"<w:pPr><w:jc w:val="center" /></w:pPr></w:pPrChange></w:pPr>"#
                    ),
                "{cell:?}"
            );
        }
    }
}

/// A plan that deletes every paragraph of a cell empties the last one, as
/// Word does: its words are deleted, its mark stays. The marks before it go
/// with their words.
#[test]
fn deleting_a_cells_every_paragraph_empties_the_last() {
    let two = table_with_cell(&format!("{}{}", para("One."), para("Two.")));
    let one = table_with_cell(&para("Old."));
    for mode in MODES {
        for (source, operations, expected, cell) in [
            (
                &two,
                vec![delete("p2"), delete("p3")],
                &["Before.", "Full.", "", "After."][..],
                &[("-", &["-One."][..]), ("", &["-Two."][..])][..],
            ),
            // A paragraph that lands in another container keeps no cell.
            (
                &two,
                vec![delete("p2"), delete("p3"), insert("p4", "before", "New.")],
                &["Before.", "Full.", "", "New.", "After."][..],
                &[("-", &["-One."][..]), ("", &["-Two."][..])][..],
            ),
            (
                &one,
                vec![delete("p2")],
                &["Before.", "Full.", "", "After."][..],
                &[("", &["-Old."][..])][..],
            ),
        ] {
            let what = format!("{mode} {operations:?}");
            let result = check_outputs(source, &plan_of(mode, &operations), expected, &what);
            assert_cell(mode, &result.redline, cell, &what);
            assert_cell(MODES[1], &result.clean, &[("", &[])], &what);
        }
    }
}

/// The body's every paragraph deleted: the same as a cell's.
#[test]
fn deleting_the_bodys_every_paragraph_empties_the_last() {
    let source = docx(&format!("{}{}", para("Alpha."), para("Beta.")));
    for mode in MODES {
        let result = check_outputs(
            &source,
            &plan_of(mode, &[delete("p0"), delete("p1")]),
            &[""],
            mode,
        );
        assert_body(
            mode,
            &result.redline,
            &[("-", &["-Alpha."]), ("", &["-Beta."])],
            mode,
        );
        assert_body(MODES[1], &result.clean, &[("", &[])], mode);
    }
}

/// What still fails, and fails whole, with the code it failed with before:
/// a table anchored on a deleted paragraph.
#[test]
fn a_table_on_a_deleted_paragraph_is_still_refused() {
    let table = |p: &str| {
        format!(r#"{{"kind":"insert_table","paragraph":"{p}","position":"after","rows":[["x"]]}}"#)
    };
    let source = table_with_cell(&format!("{}{}", para("One."), para("Two.")));
    for mode in MODES {
        let operations = [delete("p0"), table("p0")];
        let Err(error) = apply_plan(&source, &plan_of(mode, &operations)) else {
            panic!("{mode}: published")
        };
        assert_eq!(
            (error.code.as_str(), error.message.as_str()),
            (
                "OVERLAPPING_EDITS",
                "anchors a new table on a deleted paragraph"
            ),
            "{mode}"
        );
    }
}

/// The same replacement in the body, and in a header and a footer whose
/// only paragraph it is; and a header's only paragraph deleted.
#[test]
fn a_plan_replaces_a_storys_only_paragraph() {
    const HDR: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml";
    const FTR: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml";
    const HDR_REL: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header";
    const FTR_REL: &str =
        "http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer";
    let story = |root: &str, text: &str| {
        format!(
            r#"<w:{root} xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{}</w:{root}>"#,
            para(text)
        )
    };
    let header = story("hdr", "Head.");
    let footer = story("ftr", "Foot.");
    let source = common::docx::docx_with_sect(
        &para("Old."),
        &[
            common::docx::Part {
                name: "word/header1.xml",
                content_type: HDR,
                rel_type: HDR_REL,
                xml: &header,
            },
            common::docx::Part {
                name: "word/footer1.xml",
                content_type: FTR,
                rel_type: FTR_REL,
                xml: &footer,
            },
        ],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/><w:footerReference w:type="default" r:id="rIdX1"/>"#,
    );
    for mode in MODES {
        let result = check_outputs(
            &source,
            &plan_of(mode, &[delete("p0"), insert("p0", "after", "New.")]),
            &["New."],
            &format!("{mode} body"),
        );
        assert_body(mode, &result.redline, &[("", &["+New.", "-Old."])], mode);
        for (part, old, operations, redline, clean) in [
            (
                "word/header1.xml",
                "Head.",
                vec![insert("header1.p0", "before", "New."), delete("header1.p0")],
                &["+New.", "-Head."][..],
                "New.",
            ),
            (
                "word/footer1.xml",
                "Foot.",
                vec![insert("footer1.p0", "before", "New."), delete("footer1.p0")],
                &["+New.", "-Foot."][..],
                "New.",
            ),
            (
                "word/header1.xml",
                "Head.",
                vec![delete("header1.p0")],
                &["-Head."][..],
                "",
            ),
        ] {
            let what = format!("{mode} {operations:?}");
            let result = apply_plan(&source, &plan_of(mode, &operations))
                .unwrap_or_else(|e| panic!("{what}: {e:?}"));
            let story =
                |bytes: &[u8]| paragraphs_in(&normalized(&part_string(bytes, part).unwrap()));
            // The story's text: its `w:t`, not its deleted `w:delText`.
            let text = |bytes: &[u8]| {
                story(bytes)
                    .iter()
                    .flat_map(|p| words(p))
                    .filter(|w| !w.starts_with('-'))
                    .collect::<String>()
            };
            let accepted = accept_revisions(&result.redline).unwrap();
            let rejected = reject_revisions(&result.redline).unwrap();
            assert_eq!(text(&result.clean), clean, "{what}");
            assert_eq!(text(&accepted), clean, "{what}");
            assert_eq!(text(&rejected), old, "{what}");
            assert_shape(mode, &story(&result.redline), &[("", redline)], &what);
            for bytes in [&result.clean, &result.redline, &accepted, &rejected] {
                let report = check_word_valid_package(bytes);
                assert!(report.ok(), "{what}: {:?}", report.errors);
                assert_eq!(story(bytes).len(), 1, "{what}");
            }
        }
    }
}

/// The same replacement as a typed plan, tracked and untracked.
#[test]
fn a_typed_plan_replaces_a_cells_only_paragraph() {
    use jubarte::edit::for_edit::{ForEditRequest, for_edit};
    use serde_json::json;
    let source = table_with_cell(&para("Old."));
    let view = for_edit(&source, &ForEditRequest::default()).unwrap();
    let row = view
        .paragraphs
        .iter()
        .find(|row| row.original.text.as_deref() == Some("Old."))
        .expect("a for-edit row for the cell's paragraph");
    let target = json!({"short_id": row.target.short_id, "long_id": row.target.long_id});
    for track in ["on", "off"] {
        for side in ["before", "after"] {
            let plan = json!({
                "schema_version": 2,
                "source": {"sha256": jubarte::inspect::source_sha256(&source)},
                "policy": {"track": track, "existing_revisions": "keep", "text_view": "final"},
                "resolve_revisions": {"accept": [], "reject": []},
                "author": "Ann Counsel",
                "date": "2026-10-19T12:00:00Z",
                "scope": {"kind": "all"},
                "operations": [
                    {"kind": "delete_paragraph", "target": target},
                    {"kind": "insert_paragraph", "target": target, "position": side, "text": "New."},
                ],
            });
            let parsed = jubarte::edit::v2::PlanV2::from_json(&plan.to_string()).unwrap();
            let prepared = jubarte::edit::prepare_v2(
                &source,
                &parsed,
                &jubarte::edit::PrepareOptions {
                    require_change: true,
                    exact_text: true,
                    role: jubarte::edit::PrimaryRole::Clean,
                    syntax: jubarte::edit::PlanSyntax::V1,
                    skip_failed: false,
                },
            )
            .unwrap_or_else(|e| panic!("{track} {side}: {e:?}"));
            let result = prepared.result;
            let what = format!("{track} {side}");
            let expected = ["Before.", "Full.", "New.", "After."];
            assert_eq!(texts(&result.clean), expected, "{what}");
            for bytes in [&result.clean, &result.redline] {
                let report = check_word_valid_package(bytes);
                assert!(report.ok(), "{what}: {:?}", report.errors);
                assert_cells_end_in_a_paragraph(bytes, &what);
            }
            assert_cell(MODES[1], &result.clean, &[("", &["New."])], &what);
            if track == "on" {
                assert_cell(
                    MODES[1],
                    &result.redline,
                    &[("", &["+New.", "-Old."])],
                    &what,
                );
                let accepted = accept_revisions(&result.redline).unwrap();
                assert_eq!(texts(&accepted), expected, "{what}");
                let rejected = reject_revisions(&result.redline).unwrap();
                assert_eq!(texts(&rejected), texts(&source), "{what}");
            }
        }
    }
}
