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
use jubarte::edit::{EditPlan, apply_plan};
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
