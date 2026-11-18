// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::deeper_boundary_fixture::{self as fixture, Part, docx, docx_with, para};
use super::*;

fn plan(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Boundary","date":"2026-01-01T00:00:00Z","operations":{operations}}}"#
    )).unwrap()
}

#[test]
fn anchor_candidate_literal_success_skips_normalized_searches() {
    for find in ["Liability Cap", "**Liability Cap**"] {
        let source = docx(&para("Liability Cap"));
        let plan = plan(
            &serde_json::json!([{
                "kind":"replace", "paragraph":"p0", "find":find, "replacement":"Cap"
            }])
            .to_string(),
        );
        let mut tx = Transaction::start(&source, &plan).unwrap();
        anchor::READING_CALLS.with(|calls| calls.set(0));
        tx.resolve().unwrap();
        assert_eq!(
            anchor::READING_CALLS.with(std::cell::Cell::get),
            0,
            "literal/plain success must not allocate normalized offset maps"
        );
    }
}

#[test]
fn anchor_candidate_repeated_failures_scan_once_per_anchor() {
    let source = docx(&format!("{}{}", para("Other."), para("Liability Cap")));
    let operations = serde_json::json!([
        {"kind":"replace", "paragraph":"p0", "find":"Liability Cap", "replacement":"Cap"},
        {"kind":"replace", "paragraph":"p0", "find":"Liability Cap", "replacement":"Cap"},
        {"kind":"replace", "paragraph":"p0", "find":"absent", "replacement":"Cap"},
        {"kind":"replace", "paragraph":"p0", "find":"absent", "replacement":"Cap"}
    ]);
    let plan = plan(&operations.to_string());
    let mut tx = Transaction::start(&source, &plan).unwrap();
    CANDIDATE_SCANS.with(|calls| calls.set(0));
    let error = tx.resolve().unwrap_err();
    assert_eq!(error.outcomes.len(), 4);
    assert_eq!(error.outcomes[0].candidates, error.outcomes[1].candidates);
    assert!(error.outcomes[2].candidates.is_empty());
    assert_eq!(
        CANDIDATE_SCANS.with(std::cell::Cell::get),
        2,
        "repeated failures must reuse both found and empty candidate results"
    );
}

fn dom(xml: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&format!(
        r#"<root xmlns:w="{}" xmlns:x="urn:foreign">{xml}</root>"#,
        W::URI
    ));
    let root = dom.root(document).unwrap();
    (dom, root)
}

fn names(dom: &Dom, parent: NodeId) -> Vec<String> {
    dom.elements(parent, None)
        .iter()
        .map(|&n| dom.name(n).unwrap().local_name().to_string())
        .collect()
}

fn outcome(id: &str) -> EditOutcome {
    EditOutcome {
        id: id.into(),
        kind: "boundary".into(),
        status: "ok".into(),
        paragraph: None,
        matches: 1,
        context: None,
        comment_id: None,
        code: None,
        message: None,
        anchor_given: None,
        anchor_read_as: None,
        near: None,
        candidates: Vec::new(),
    }
}

// Baseline gaps: 113, 916, 933, 936, 942, 945, 958.
/// A retry that holds clears the refusal its first side stored, so a
/// later failure of the same operation names no revision that did not
/// block it (#1308 review).
#[test]
fn a_retried_insertion_point_leaves_no_refusal() {
    let source = docx(concat!(
        r#"<w:p><w:r><w:t xml:space="preserve">Alpha </w:t></w:r>"#,
        r#"<w:moveTo w:id="9" w:author="Bob" w:date="2026-01-01T00:00:00Z"><w:r><w:t>Beta</w:t></w:r></w:moveTo>"#,
        r#"<w:r><w:t>Gamma</w:t></w:r></w:p>"#,
    ));
    let plan = EditPlan::from_json(
        r#"{"schema_version":1,"author":"Carol","date":"2026-02-02T00:00:00Z","existing_revisions":"keep","operations":[]}"#,
    )
    .unwrap();
    let tx = Transaction::start(&source, &plan).unwrap();
    let projection = &tx.projections[0];
    assert_eq!(projection.text, "Alpha BetaGamma");
    let mut attach_before = true;
    assert_eq!(
        tx.check_insert_either(projection, 10, &mut attach_before),
        Ok(())
    );
    assert!(!attach_before);
    assert!(tx.revision_refusal.borrow().is_none());
}

#[test]
fn report_optional_fields_and_resolution_truth_table() {
    let source = docx(&format!(
        "<w:p>{}</w:p>",
        fixture::run("source", false, false, None)
    ));
    let plan = plan("[]");
    let tx = Transaction::start(&source, &plan).unwrap();
    for (accepted, rejected, empty) in [
        (vec![], vec![], true),
        (vec!["a"], vec![], false),
        (vec![], vec!["r"], false),
        (vec!["a"], vec!["r"], false),
    ] {
        let resolved = ResolvedRevisions {
            accepted: accepted.into_iter().map(String::from).collect(),
            rejected: rejected.into_iter().map(String::from).collect(),
        };
        assert_eq!(resolved.is_empty(), empty);
        let mut report = tx.report(false);
        report.resolved_revisions = resolved.clone();
        let mut failed = outcome("bad");
        failed.status = "failed".into();
        failed.code = Some("INVALID_EDIT".into());
        failed.message = Some("boundary refusal".into());
        report.operations = vec![failed];
        let rows: Vec<serde_json::Value> = report
            .to_jsonl()
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
        assert_eq!(rows.len(), 3);
        let expected_resolved = serde_json::to_value(&resolved).unwrap();
        assert_eq!(
            rows[0].get("resolved_revisions"),
            if empty {
                None
            } else {
                Some(&expected_resolved)
            }
        );
        assert_eq!(
            rows[1],
            serde_json::json!({
                "ev":"op", "i":1, "id":"bad", "op":"boundary", "status":"failed",
                "matches":1, "code":"INVALID_EDIT", "message":"boundary refusal"
            })
        );
        assert_eq!(rows[2]["status"], "failed");
        assert_eq!(rows[2]["ops"], serde_json::json!({"ok":0,"failed":1}));
    }
}

// 702's two disjuncts, including values JSON cannot represent.
#[test]
fn half_points_finite_range_and_rounding_boundaries() {
    use serde::de::value::{Error, F64Deserializer};
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1638.01] {
        let result = HalfPoints::deserialize(F64Deserializer::<Error>::new(value));
        assert_eq!(
            result.unwrap_err().to_string(),
            format!("size_pt {value} is outside 0..=1638")
        );
    }
    for (value, expected) in [(-0.0, 0), (0.24, 0), (0.25, 1), (0.75, 2), (1638.0, 3276)] {
        assert_eq!(
            HalfPoints::deserialize(F64Deserializer::<Error>::new(value)).unwrap(),
            HalfPoints(expected)
        );
    }
}

// 1055, 1069, 1073: no selection, empty filters, reject-only selection.
#[test]
fn revision_selection_empty_and_reject_only_preserve_unselected_change() {
    let source = docx(
        r#"<w:p><w:ins w:id="1" w:author="A"><w:r><w:t>one</w:t></w:r></w:ins><w:ins w:id="2" w:author="B"><w:r><w:t>two</w:t></w:r></w:ins></w:p>"#,
    );
    for selection in [
        ResolveRevisions::default(),
        ResolveRevisions {
            accept: Some(ChangeFilter::ids(Vec::<String>::new())),
            reject: Some(ChangeFilter::ids(Vec::<String>::new())),
            reject_visible: false,
        },
    ] {
        assert_eq!(
            resolve_selected(
                &source,
                Some(&selection),
                RemovedAnchorComments::Remove,
                PlanSyntax::V1
            )
            .unwrap(),
            (None, ResolvedRevisions::default(), AnchorChanges::default())
        );
    }
    let selection = ResolveRevisions {
        accept: None,
        reject: Some(ChangeFilter::ids(["r1"])),
        reject_visible: false,
    };
    let (bytes, resolved, _) = resolve_selected(
        &source,
        Some(&selection),
        RemovedAnchorComments::Remove,
        PlanSyntax::V1,
    )
    .unwrap();
    assert_eq!(
        resolved,
        ResolvedRevisions {
            accepted: vec![],
            rejected: vec!["r1".into()]
        }
    );
    let bytes = bytes.unwrap();
    let changes = crate::changes::list_changes(&bytes).unwrap();
    assert_eq!(
        changes.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        ["r2"]
    );
    assert_eq!(crate::inspect::paragraphs(&bytes).unwrap()[0].text, "two");
}

// 1652, 1669, 1677; validation precedes package parsing.
#[test]
fn admission_boundaries_and_signature_prefix_are_exact() {
    let mut bad = plan("[]");
    bad.schema_version = SCHEMA_VERSION + 1;
    let e = preview_plan(b"", &bad).unwrap_err();
    assert_eq!(
        (e.code.as_str(), e.message.as_str()),
        ("UNSUPPORTED_SCHEMA", "schema_version 2 is not supported")
    );
    bad.schema_version = SCHEMA_VERSION;
    bad.author = " \t\n".into();
    let e = preview_plan(b"", &bad).unwrap_err();
    assert_eq!(
        (e.code.as_str(), e.message.as_str()),
        ("INVALID_PLAN", "author must be nonempty")
    );
    let source = docx(&para("source"));
    for (name, refused) in [
        ("_xmlsignatures/sig.xml", true),
        ("word/vbaProject.bin", true),
        ("word/not-vbaProject.bin.xml", false),
    ] {
        let mut pkg = crate::opc::PartFs::open(&source).unwrap();
        pkg.set_part(name, b"<probe/>".to_vec());
        let bytes = pkg.to_zip().unwrap();
        let result = preview_plan(&bytes, &plan("[]"));
        if refused {
            let e = result.unwrap_err();
            assert_eq!(
                (e.code.as_str(), e.message.as_str()),
                ("UNSUPPORTED_PACKAGE", "signed or macro-bearing package")
            );
        } else {
            assert_eq!(
                result.unwrap().paragraphs,
                ParagraphDelta { from: 1, to: 1 }
            );
        }
    }
}

// 1376, 1389, 4519: invalid byte input is handled without panic.
#[test]
fn invalid_package_has_no_revisions_or_kept_comment_text() {
    assert_eq!(revision_counts(b"invalid zip"), RevisionCounts::default());
    assert!(!comment_holds_kept_text(b"invalid zip", 0));
}

// 2259, 2312, 2339, 2503, 2703, 3173.
#[test]
fn resolution_empty_text_and_combined_comment_anchors() {
    let source = docx("<w:p/><w:p><w:r><w:t>tail</w:t></w:r></w:p>");
    for (ops, message) in [
        (
            r#"[{"kind":"comment","paragraph":"p0","find":"x","through":"p1","text":"n"}]"#,
            "find and through cannot be combined: through comments on whole paragraphs",
        ),
        (
            r#"[{"kind":"insert_paragraph","paragraph":"p0","runs":[]}]"#,
            "runs must carry text",
        ),
        (
            r#"[{"kind":"insert_paragraph","paragraph":"p0","runs":[{"text":""},{"text":""}]}]"#,
            "runs must carry text",
        ),
        (
            r#"[{"kind":"insert_toc","paragraph":"p0","title":""}]"#,
            "title must carry text",
        ),
        (
            r#"[{"kind":"comment","paragraph":{"contains":""},"text":"n"}]"#,
            "paragraph selector text must be nonempty",
        ),
    ] {
        let e = preview_plan(&source, &plan(ops)).unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            ("INVALID_EDIT", message),
            "{ops}"
        );
        assert_eq!(e.outcomes[0].message.as_deref(), Some(message));
    }
    for ops in [
        r#"[{"kind":"comment","paragraph":"p0","text":"empty anchor"}]"#,
        r#"[{"kind":"delete_paragraph","paragraph":"p0","comment":"empty deletion"}]"#,
    ] {
        let p = plan(ops);
        let mut tx = Transaction::start(&source, &p).unwrap();
        tx.resolve().unwrap();
        assert_eq!(tx.outcomes[0].matches, 1);
        tx.apply().unwrap();
        if ops.contains("delete_paragraph") {
            assert_eq!(tx.comments, Vec::<(u32, String)>::new());
            assert_eq!(tx.deletion_comments, [(0, 0)]);
            assert_eq!(tx.outcomes[0].comment_id, Some(0));
            assert_eq!(
                tx.opened.dom.elements(tx.opened.body, Some(&W::p())).len(),
                1
            );
        } else {
            assert_eq!(tx.comments, [(0, "empty anchor".into())]);
            assert_eq!(
                tx.opened.dom.elements(tx.opened.body, Some(&W::p())).len(),
                2
            );
        }
    }
}

// Body-only operations with a real in-memory header story: 2432, 2688, 2987.
#[test]
fn header_refuses_body_only_picture_toc_and_list() {
    let header = format!(r#"<w:hdr xmlns:w="{}">{}</w:hdr>"#, W::URI, para("header"));
    let source = fixture::docx_with_sect(
        &para("body"),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    for (ops, message) in [
        (
            r#"[{"kind":"insert_image","paragraph":"header1","image_base64":""}]"#,
            "pictures can be inserted in the body only",
        ),
        (
            r#"[{"kind":"insert_toc","paragraph":"header1"}]"#,
            "a table of contents goes in the body",
        ),
        (
            r#"[{"kind":"list","paragraphs":["header1"],"kind_of_list":"bullet"}]"#,
            "paragraphs[0]: list is supported in the body only, not header1",
        ),
    ] {
        let e = preview_plan(&source, &plan(ops)).unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            ("UNSUPPORTED_STRUCTURE", message)
        );
    }
}

// 3109, 3114: a related style part can disappear/become rootless.
#[test]
fn paragraph_styles_missing_and_rootless_parts() {
    let styles = format!(
        r#"<w:styles xmlns:w="{}"><w:style w:styleId="S" w:type="paragraph"><w:name w:val="Style"/></w:style></w:styles>"#,
        W::URI
    );
    let source = docx_with(
        &para("body"),
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: &styles,
        }],
    );
    let p = plan("[]");
    let mut tx = Transaction::start(&source, &p).unwrap();
    assert_eq!(tx.paragraph_styles(), [("S".into(), "Style".into())]);
    tx.opened
        .pkg
        .set_part("word/styles.xml", b"<?xml version=\"1.0\"?>".to_vec());
    assert_eq!(tx.paragraph_styles(), Vec::<(String, String)>::new());
    tx.opened.pkg.remove_part("word/styles.xml");
    assert_eq!(tx.paragraph_styles(), Vec::<(String, String)>::new());
}

// 3336, 3342, 3368: end coverage, field-boundary equality, and glyph attachment.
#[test]
fn projection_range_and_insert_boundary_truth_tables() {
    let source = docx(&para("abc"));
    let p = plan("[]");
    let tx = Transaction::start(&source, &p).unwrap();
    let mut projection = tx.projections[0].clone();
    assert_eq!(
        tx.check_range(&projection, 0, 4),
        Err("the text is not addressable".into())
    );
    projection.field_marks = vec![0, 3];
    assert_eq!(tx.check_range(&projection, 0, 3), Ok(()));
    projection.field_marks = vec![1];
    assert_eq!(
        tx.check_range(&projection, 0, 3),
        Err(
            "the text crosses a field; edit the text on either side of it, or comment on it".into()
        )
    );
    // Text may go beside a tab, never beside a symbol.
    let (d, root) = dom(
        r#"<w:p><w:r><w:tab/></w:r><w:r><w:sym w:font="Wingdings" w:char="F0FC"/></w:r></w:p>"#,
    );
    let paragraph = d.element(root, &W::p()).unwrap();
    let glyphs = project_paragraph(&d, paragraph);
    for before in [false, true] {
        assert_eq!(tx.check_insert_position(&glyphs, 0, before), Ok(()));
        assert_eq!(
            tx.check_insert_position(&glyphs, 4, before),
            Err("the insertion point touches a symbol".into())
        );
    }
    assert_eq!(tx.check_range(&glyphs, 0, 1), Ok(()));
    assert_eq!(
        tx.check_range(&glyphs, 0, 4),
        Err("the text crosses a symbol".into())
    );
    assert_eq!(
        tx.check_insert_position(&projection, 4, false),
        Err("no run holds the insertion point".into())
    );
}

fn text_range(start: usize, end: usize) -> Resolved {
    Resolved::Text {
        para: 0,
        start,
        end,
        replacement: "X".into(),
        attach_before: false,
        comment: None,
        format: None,
    }
}

fn conflicts(source: &[u8], resolved: Vec<Resolved>) -> Result<(), EditError> {
    let mut p = plan("[]");
    // Conflict checks still consult the original operation metadata (in
    // particular deleted-paragraph comments), so keep it alongside each
    // synthetic resolution rather than constructing an empty plan.
    let operations = {
        let seed = Transaction::start(source, &p).unwrap();
        let selector = |index| Selector::Index { index, story: None };
        let text = |para: usize, start: usize, end: usize| {
            seed.projections[para]
                .text
                .chars()
                .skip(start)
                .take(end - start)
                .collect::<String>()
        };
        resolved
            .iter()
            .enumerate()
            .map(|(i, r)| {
                let kind = match r {
                    Resolved::Text {
                        para,
                        start,
                        end,
                        replacement,
                        comment,
                        format,
                        attach_before,
                    } => {
                        if start == end {
                            OperationKind::Insert {
                                paragraph: selector(*para),
                                after: (!*attach_before && *start > 0)
                                    .then(|| text(*para, 0, *start)),
                                before: (*attach_before).then(|| {
                                    text(
                                        *para,
                                        *start,
                                        seed.projections[*para].text.chars().count(),
                                    )
                                }),
                                position: (!*attach_before && *start == 0).then_some(Edge::Start),
                                text: replacement.clone(),
                                format: format.clone(),
                                comment: comment.clone(),
                                occurrence: None,
                            }
                        } else {
                            OperationKind::Replace {
                                paragraph: selector(*para),
                                find: text(*para, *start, *end),
                                replacement: replacement.clone(),
                                format: format.clone(),
                                comment: comment.clone(),
                                whole: false,
                                occurrence: None,
                            }
                        }
                    }
                    Resolved::DeleteParagraph { para } => OperationKind::DeleteParagraph {
                        paragraph: selector(*para),
                        comment: None,
                    },
                    Resolved::CommentRange {
                        para,
                        start,
                        end,
                        text: note,
                    } => OperationKind::Comment {
                        paragraph: selector(*para),
                        find: Some(text(*para, *start, *end)),
                        text: note.clone(),
                        through: None,
                        occurrence: None,
                    },
                    Resolved::CommentSpan { para, last, text } => OperationKind::Comment {
                        paragraph: selector(*para),
                        find: None,
                        text: text.clone(),
                        through: Some(selector(*last)),
                        occurrence: None,
                    },
                    Resolved::InsertParagraph {
                        anchor,
                        side,
                        runs,
                        like,
                        style,
                        comment,
                        toc: None,
                        keep_mark: false,
                    } => OperationKind::InsertParagraph {
                        paragraph: selector(*anchor),
                        position: *side,
                        runs: runs.clone(),
                        like: Some(selector(*like)),
                        style: style.clone(),
                        comment: comment.clone(),
                    },
                    Resolved::InsertTable {
                        anchor,
                        side,
                        rows,
                        header_row,
                        widths,
                        style,
                        ..
                    } => OperationKind::InsertTable {
                        paragraph: selector(*anchor),
                        position: *side,
                        rows: rows.clone(),
                        header_row: *header_row,
                        widths_dxa: Some(widths.clone()),
                        style: Some(style.clone()),
                    },
                    Resolved::FormatRun {
                        para,
                        start,
                        end,
                        format,
                    } => OperationKind::FormatRun {
                        paragraph: selector(*para),
                        find: text(*para, *start, *end),
                        format: format.clone(),
                        occurrence: None,
                    },
                    Resolved::InsertFootnote {
                        para,
                        at,
                        text: note,
                    } => OperationKind::InsertFootnote {
                        paragraph: selector(*para),
                        after: text(*para, 0, *at),
                        text: note.clone(),
                        occurrence: None,
                    },
                    Resolved::FormatParagraph {
                        para,
                        style,
                        alignment,
                        spacing,
                    } => OperationKind::FormatParagraph {
                        paragraph: selector(*para),
                        style: style.clone(),
                        alignment: *alignment,
                        line_spacing: spacing.line,
                        space_before: spacing.before,
                        space_after: spacing.after,
                    },
                    Resolved::MergeParagraphs {
                        para, separator, ..
                    } => OperationKind::MergeParagraphs {
                        paragraph: selector(*para),
                        separator: Some(separator.clone()),
                    },
                    Resolved::Thread { op, .. } => match op {
                        ThreadOp::Reply { parent, text } => OperationKind::ReplyComment {
                            comment_id: *parent,
                            text: text.clone(),
                        },
                        ThreadOp::Resolve { id, done } => OperationKind::ResolveComment {
                            comment_id: *id,
                            done: *done,
                        },
                        ThreadOp::Edit { id, text } => OperationKind::EditComment {
                            comment_id: *id,
                            text: text.clone(),
                        },
                        ThreadOp::Delete { id } => OperationKind::DeleteComment { comment_id: *id },
                    },
                    _ => panic!("conflict fixture needs matching operation metadata"),
                };
                Operation {
                    id: Some(format!("case-{i}")),
                    group: None,
                    kind,
                }
            })
            .collect()
    };
    p.operations = operations;
    let mut tx = Transaction::start(source, &p).unwrap();
    tx.outcomes = (0..resolved.len())
        .map(|i| outcome(&format!("case-{i}")))
        .collect();
    tx.resolved = resolved.into_iter().enumerate().collect();
    tx.check_conflicts()
}

// 3460/3461 and 3477: same-start replacements vs insertions, and either comment edge.
#[test]
fn overlapping_range_and_comment_edge_truth_tables() {
    let source = docx(&para("abcdefgh"));
    for (left, right, culprit) in [
        ((1, 4), (1, 2), Some("case-0")),
        ((1, 1), (1, 4), None),
        ((1, 4), (4, 4), None),
        ((1, 4), (2, 2), Some("case-1")),
        ((1, 4), (4, 7), None),
        ((1, 1), (1, 1), None),
    ] {
        let result = conflicts(
            &source,
            vec![text_range(left.0, left.1), text_range(right.0, right.1)],
        );
        if let Some(culprit) = culprit {
            let e = result.unwrap_err();
            assert_eq!(
                (e.code.as_str(), e.message.as_str()),
                ("OVERLAPPING_EDITS", "overlaps an earlier edit's text range")
            );
            assert_eq!(e.operation.as_deref(), Some(culprit));
        } else {
            assert_eq!(result, Ok(()));
        }
    }
    for (start, end, conflict) in [
        (2, 5, true),
        (0, 2, true),
        (0, 4, false),
        (1, 4, false),
        (4, 7, false),
        (1, 1, false),
    ] {
        let result = conflicts(
            &source,
            vec![
                text_range(1, 4),
                Resolved::CommentRange {
                    para: 0,
                    start,
                    end,
                    text: "note".into(),
                },
            ],
        );
        if conflict {
            let e = result.unwrap_err();
            assert_eq!(e.message, "comment range cuts through an edited range");
            assert_eq!(e.operation.as_deref(), Some("case-1"));
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}

// All deletion conflict arms use explicit functional error assertions, not a call sweep.
#[test]
fn deleted_paragraph_rejects_each_anchor_owner() {
    let source = docx(&(para("one") + &para("two") + &para("three")));
    let cases = vec![
        (
            Resolved::CommentRange {
                para: 0,
                start: 0,
                end: 1,
                text: "n".into(),
            },
            "comments on a deleted paragraph",
        ),
        (
            Resolved::InsertTable {
                anchor: 0,
                side: Side::After,
                rows: vec![vec!["cell".into()]],
                header_row: false,
                widths: vec![100],
                style: "TableGrid".into(),
                add_style: false,
            },
            "anchors a new table on a deleted paragraph",
        ),
        (
            Resolved::FormatRun {
                para: 0,
                start: 0,
                end: 1,
                format: RunFormat {
                    bold: Some(true),
                    ..RunFormat::default()
                },
            },
            "formats text of a deleted paragraph",
        ),
        (
            Resolved::InsertFootnote {
                para: 0,
                at: 1,
                text: "n".into(),
            },
            "adds a footnote to a deleted paragraph",
        ),
        (
            Resolved::CommentSpan {
                para: 0,
                last: 1,
                text: "n".into(),
            },
            "comments on a deleted paragraph",
        ),
        (
            Resolved::FormatParagraph {
                para: 0,
                style: None,
                alignment: Some(Alignment::Left),
                spacing: Spacing::default(),
            },
            "formats a deleted paragraph",
        ),
        (
            Resolved::MergeParagraphs {
                para: 0,
                next: 1,
                separator: "".into(),
                strike: false,
            },
            "merges a deleted paragraph",
        ),
    ];
    for (r, message) in cases {
        let e = conflicts(&source, vec![Resolved::DeleteParagraph { para: 0 }, r]).unwrap_err();
        assert_eq!(
            (e.code.as_str(), e.message.as_str()),
            ("OVERLAPPING_EDITS", message)
        );
        assert_eq!(e.operation.as_deref(), Some("case-1"));
        assert_eq!(e.outcomes[0].status, "ok");
        assert_eq!(e.outcomes[1].message.as_deref(), Some(message));
    }
    // A reply to a comment in the deleted paragraph is not one: the
    // tracked deletion keeps the reference in struck text (#1159).
    let reply = Resolved::Thread {
        op: ThreadOp::Reply {
            parent: 7,
            text: "n".into(),
        },
        stories: vec![0],
        anchor: Some(0),
    };
    conflicts(&source, vec![Resolved::DeleteParagraph { para: 0 }, reply]).unwrap();
    // Nor is a paragraph inserted beside it, on either side: the deleted
    // paragraph's place is the new one's (#1668).
    for side in [Side::Before, Side::After] {
        let insert = Resolved::InsertParagraph {
            anchor: 0,
            side,
            runs: vec![],
            like: 0,
            style: None,
            comment: None,
            toc: None,
            keep_mark: false,
        };
        conflicts(&source, vec![Resolved::DeleteParagraph { para: 0 }, insert]).unwrap();
    }
    // Nor is deleting every paragraph: the final one is emptied, as
    // Word empties it (#1668).
    conflicts(
        &source,
        vec![
            Resolved::DeleteParagraph { para: 0 },
            Resolved::DeleteParagraph { para: 1 },
            Resolved::DeleteParagraph { para: 2 },
        ],
    )
    .unwrap();
}

// 3586's Before/tail arm, with a table rather than the existing paragraph integration case.
#[test]
fn table_insertion_sides_at_merge_boundaries() {
    let source = docx(&(para("one") + &para("two")));
    for (anchor, side, conflict) in [
        (0, Side::Before, false),
        (0, Side::After, true),
        (1, Side::Before, true),
        (1, Side::After, false),
    ] {
        let result = conflicts(
            &source,
            vec![
                Resolved::MergeParagraphs {
                    para: 0,
                    next: 1,
                    separator: "".into(),
                    strike: false,
                },
                Resolved::InsertTable {
                    anchor,
                    side,
                    rows: vec![vec!["cell".into()]],
                    header_row: false,
                    widths: vec![100],
                    style: "TableGrid".into(),
                    add_style: false,
                },
            ],
        );
        if conflict {
            assert_eq!(
                result.unwrap_err().message,
                "inserts a table between two a merge joins"
            );
        } else {
            assert_eq!(result, Ok(()));
        }
    }
}

// 2804, 2842, 2851, 3667, 3680.
#[test]
fn comment_reference_identity_and_thread_conflicts() {
    let comments = format!(
        r#"<w:comments xmlns:w="{}"><w:comment w:id="7" w:author="A"><w:p><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#,
        W::URI
    );
    let source = docx_with(
        r#"<w:p><w:commentRangeStart/><w:commentRangeStart w:id="7"/><w:r><w:t>body</w:t><w:commentReference w:id="7"/><w:commentReference w:id="7"/></w:r><w:commentRangeEnd w:id="7"/></w:p>"#,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    );
    let p = plan("[]");
    let mut tx = Transaction::start(&source, &p).unwrap();
    let (stories, reference) = tx.comment_markers(&[7], 7);
    assert_eq!(stories, [0]);
    let references = tx
        .opened
        .dom
        .descendants(tx.opened.body, Some(&W::comment_reference()));
    assert_eq!(reference, Some(references[0]));
    for r in references {
        tx.opened.dom.remove(r);
    }
    let (e, _) = *tx
        .resolve_thread(
            "reply",
            &OperationKind::ReplyComment {
                comment_id: 7,
                text: "reply".into(),
            },
        )
        .unwrap_err();
    assert_eq!(
        (e.code.as_str(), e.message.as_str()),
        (
            "UNSUPPORTED_STRUCTURE",
            "comment 7 has no reference in the document to anchor a reply on"
        )
    );
    for ops in [
        vec![
            ThreadOp::Edit {
                id: 7,
                text: "a".into(),
            },
            ThreadOp::Edit {
                id: 7,
                text: "b".into(),
            },
        ],
        vec![
            ThreadOp::Delete { id: 7 },
            ThreadOp::Resolve { id: 7, done: true },
        ],
    ] {
        let edit = matches!(ops[0], ThreadOp::Edit { .. });
        let e = conflicts(
            &source,
            ops.into_iter()
                .map(|op| Resolved::Thread {
                    op,
                    stories: vec![0],
                    anchor: Some(0),
                })
                .collect(),
        )
        .unwrap_err();
        assert_eq!(
            e.message,
            if edit {
                "edits a comment another operation edits"
            } else {
                "acts on a comment another operation deletes"
            }
        );
        assert_eq!(e.operation.as_deref(), Some("case-1"));
    }
}

// 4697, 4713, 4726: insertion affinity and glyphs never turn into text nodes.
#[test]
fn text_edit_glyphs_and_segment_attachment_are_exact() {
    let (mut d, root) = dom("<w:p><w:r><w:t>ab</w:t><w:tab/><w:t>cd</w:t></w:r></w:p>");
    let p = d.element(root, &W::p()).unwrap();
    let projection = project_paragraph(&d, p);
    for (pos, before, index) in [
        (0, false, 0),
        (2, true, 0),
        (2, false, 1),
        (3, true, 1),
        (3, false, 2),
        (5, false, 2),
    ] {
        assert_eq!(
            attach_segment(&projection, pos, before),
            Some(&projection.segments[index])
        );
    }
    assert_eq!(attach_segment(&projection, 6, false), None);
    // Beside a glyph, new text joins its run: before it, or after it
    // when the glyph holds the char before the point.
    apply_text_edit(&mut d, &projection, 2, 2, "+", false);
    assert_eq!(project_paragraph(&d, p).text, "ab+\tcd");
    let projection = project_paragraph(&d, p);
    apply_text_edit(&mut d, &projection, 4, 4, "-", true);
    assert_eq!(project_paragraph(&d, p).text, "ab+\t-cd");
    // A range over the glyph removes it.
    let projection = project_paragraph(&d, p);
    apply_text_edit(&mut d, &projection, 1, 5, "X", false);
    assert_eq!(project_paragraph(&d, p).text, "aXcd");
    assert!(d.descendants(p, Some(&W::name("tab"))).is_empty());
    let texts: Vec<String> = d
        .descendants(p, Some(&W::t()))
        .iter()
        .map(|&t| d.value(t))
        .filter(|text| !text.is_empty())
        .collect();
    assert_eq!(texts, ["aX", "cd"]);
    // A tab or a break in new text is one of the run.
    let projection = project_paragraph(&d, p);
    apply_text_edit(&mut d, &projection, 2, 2, "\tY\n", true);
    assert_eq!(project_paragraph(&d, p).text, "aX\tY\ncd");
    assert_eq!(d.descendants(p, Some(&W::name("tab"))).len(), 1);
    assert_eq!(d.descendants(p, Some(&W::name("br"))).len(), 1);
}

// 4751, 4754, 4771, 4776: both edges, a glyph, and children on either side of t.
#[test]
fn splitting_multichild_run_preserves_order_and_properties() {
    let (mut d, root) =
        dom("<w:p><w:r><w:rPr><w:b/></w:rPr><w:tab/><w:t>abcd</w:t><w:br/></w:r></w:p>");
    let p = d.element(root, &W::p()).unwrap();
    let projection = project_paragraph(&d, p);
    let seg = &projection.segments[1];
    let before = d.serialize_element(p);
    for at in [seg.start, seg.end, seg.end + 1] {
        split_run_at(&mut d, seg, at, &mut RevisionCounter::default());
        assert_eq!(d.serialize_element(p), before);
    }
    let (mut glyph_dom, glyph_root) = dom("<w:p><w:r><w:noBreakHyphen/></w:r></w:p>");
    let glyph_p = glyph_dom.element(glyph_root, &W::p()).unwrap();
    let glyph_projection = project_paragraph(&glyph_dom, glyph_p);
    split_run_at(
        &mut glyph_dom,
        &glyph_projection.segments[0],
        1,
        &mut RevisionCounter::default(),
    );
    assert_eq!(project_paragraph(&glyph_dom, glyph_p).text, "\u{2011}");
    assert_eq!(glyph_dom.elements(glyph_p, Some(&W::r())).len(), 1);
    assert_eq!(d.serialize_element(p), before);
    split_run_at(&mut d, seg, 3, &mut RevisionCounter::default());
    let runs = d.elements(p, Some(&W::r()));
    assert_eq!(runs.len(), 2);
    assert_eq!(names(&d, runs[0]), ["rPr", "tab", "t"]);
    assert_eq!(names(&d, runs[1]), ["rPr", "t", "br"]);
    for r in &runs {
        let rpr = d.element(*r, &W::r_pr()).unwrap();
        assert_eq!(names(&d, rpr), ["b"]);
    }
    assert_eq!(d.value(d.element(runs[0], &W::t()).unwrap()), "ab");
    assert_eq!(d.value(d.element(runs[1], &W::t()).unwrap()), "cd");
    assert_eq!(project_paragraph(&d, p).text, "\tabcd\n");
}

// 4798, 4830, 4840, 4867, 4903, 4957.
#[test]
fn empty_comment_anchors_and_reference_cleanup() {
    let (mut d, root) = dom("<w:p><w:pPr><w:jc w:val=\"left\"/></w:pPr></w:p><w:p/>");
    let ps = d.elements(root, Some(&W::p()));
    anchor_span(&mut d, ps[0], ps[1], 7, &mut RevisionCounter::default());
    assert_eq!(names(&d, ps[0]), ["pPr", "commentRangeStart"]);
    assert_eq!(names(&d, ps[1]), ["commentRangeEnd", "r"]);
    for local in ["commentRangeStart", "commentRangeEnd", "commentReference"] {
        let markers = d.descendants(root, Some(&W::name(local)));
        assert_eq!(markers.len(), 1);
        assert_eq!(d.attribute(markers[0], &W::id()), Some("7"));
    }
    anchor_comment(&mut d, ps[1], 0, 0, 8, &mut RevisionCounter::default());
    assert_eq!(
        names(&d, ps[1]),
        [
            "commentRangeEnd",
            "r",
            "commentRangeStart",
            "commentRangeEnd",
            "r"
        ]
    );
    let before = d.serialize_element(root);
    place_reply_markers(&mut d, &[root], 999, 9);
    assert_eq!(d.serialize_element(root), before);
    let (mut d, root) = dom(
        r#"<w:p><w:r><w:rPr><w:b/></w:rPr><w:commentReference w:id="7"/><w:tab/></w:r><w:r><w:rPr/><w:commentReference w:id="8"/></w:r></w:p>"#,
    );
    let p = d.element(root, &W::p()).unwrap();
    remove_comment_markers(&mut d, &[root], &["7".into(), "8".into()]);
    assert_eq!(d.elements(p, Some(&W::r())).len(), 1);
    let r = d.element(p, &W::r()).unwrap();
    assert_eq!(names(&d, r), ["rPr", "tab"]);
    assert_eq!(project_paragraph(&d, p).text, "\t");
    let open = comment_marker(&mut d, "commentRangeStart", "9");
    let close = comment_marker(&mut d, "commentRangeEnd", "9");
    assert!(!wrap_range(
        &mut d,
        p,
        0,
        0,
        open,
        close,
        &mut RevisionCounter::default()
    ));
    assert_eq!(d.parent(open), None);
    assert_eq!(d.parent(close), None);
    let (mut d, root) = dom("<w:p><w:r><w:t>a</w:t><w:t>b</w:t></w:r></w:p>");
    let p = d.element(root, &W::p()).unwrap();
    let open = comment_marker(&mut d, "commentRangeStart", "10");
    let close = comment_marker(&mut d, "commentRangeEnd", "10");
    assert!(!wrap_range(
        &mut d,
        p,
        1,
        1,
        open,
        close,
        &mut RevisionCounter::default()
    ));
    assert_eq!(project_paragraph(&d, p).text, "ab");
    assert_eq!(d.parent(open), None);
    assert_eq!(d.parent(close), None);
}

// 4975, 4992, 4994, 5009: no-op ranges, empty t, and two segments in one run.
#[test]
fn format_range_deduplicates_runs_and_removes_empty_properties() {
    let (mut d, root) = dom(
        "<w:p><w:r><w:rPr><w:u/></w:rPr><w:t/><w:t>a</w:t><w:t>b</w:t></w:r><w:r><w:t>c</w:t></w:r></w:p>",
    );
    let p = d.element(root, &W::p()).unwrap();
    let format = RunFormat {
        underline: Some(false),
        ..RunFormat::default()
    };
    let original = d.serialize_element(p);
    for (start, end) in [(0, 0), (2, 1)] {
        format_range(
            &mut d,
            p,
            start,
            end,
            &format,
            &mut RevisionCounter::default(),
        );
        assert_eq!(d.serialize_element(p), original);
    }
    format_range(&mut d, p, 0, 2, &format, &mut RevisionCounter::default());
    assert_eq!(d.descendants(p, Some(&W::r_pr())).len(), 0);
    assert_eq!(d.elements(p, Some(&W::r())).len(), 2);
    assert_eq!(project_paragraph(&d, p).text, "abc");
}

// 5032, 5041, 5058, 5063, 5067; only the requested spacing axis changes.
#[test]
fn paragraph_format_partial_spacing_preserves_other_axes() {
    for (spacing, expected) in [
        (
            Spacing {
                before: Some(Points(40)),
                ..Spacing::default()
            },
            [Some("360"), Some("40"), Some("60"), None, Some("1")],
        ),
        (
            Spacing {
                after: Some(Points(80)),
                ..Spacing::default()
            },
            [Some("360"), Some("20"), Some("80"), Some("1"), None],
        ),
        (
            Spacing {
                line: Some(LineSpacing(480)),
                ..Spacing::default()
            },
            [Some("480"), Some("20"), Some("60"), Some("1"), Some("1")],
        ),
    ] {
        let (mut d, root) = dom(
            r#"<w:p><w:pPr><w:pStyle w:val="Old"/><w:spacing w:line="360" w:before="20" w:after="60" w:beforeAutospacing="1" w:afterAutospacing="1"/><w:jc w:val="left"/></w:pPr><w:r><w:t>x</w:t></w:r></w:p>"#,
        );
        let p = d.element(root, &W::p()).unwrap();
        format_paragraph(&mut d, p, Some("New"), Some(Alignment::Justify), spacing);
        let ppr = d.element(p, &W::p_pr()).unwrap();
        assert_eq!(names(&d, ppr), ["pStyle", "spacing", "jc"]);
        assert_eq!(
            d.attribute(d.element(ppr, &W::p_style()).unwrap(), &W::val()),
            Some("New")
        );
        assert_eq!(
            d.attribute(d.element(ppr, &W::jc_el()).unwrap(), &W::val()),
            Some("both")
        );
        let s = d.element(ppr, &W::spacing_el()).unwrap();
        for (local, value) in [
            "line",
            "before",
            "after",
            "beforeAutospacing",
            "afterAutospacing",
        ]
        .into_iter()
        .zip(expected)
        {
            assert_eq!(d.attribute(s, &W::name(local)), value, "{local}");
        }
        assert_eq!(project_paragraph(&d, p).text, "x");
    }
}

// 5125, 5482: higher-ranked and foreign children do not supply the insertion anchor.
#[test]
fn schema_insertion_ignores_foreign_and_later_properties() {
    for paragraph in [false, true] {
        let xml = if paragraph {
            "<w:pPr><x:pStyle/><w:sectPr/></w:pPr>"
        } else {
            "<w:rPr><x:rStyle/><w:sz/></w:rPr>"
        };
        let (mut d, root) = dom(xml);
        let parent = d.elements(root, None)[0];
        let child = d.new_element(if paragraph {
            W::p_style()
        } else {
            W::name("b")
        });
        if paragraph {
            insert_ppr_child(&mut d, parent, child);
        } else {
            insert_rpr_child(&mut d, parent, child);
        }
        assert_eq!(d.elements(parent, None)[0], child);
        assert_eq!(
            names(&d, parent),
            if paragraph {
                vec!["pStyle", "pStyle", "sectPr"]
            } else {
                vec!["b", "rStyle", "sz"]
            }
        );
        assert_eq!(
            d.name(d.elements(parent, None)[1])
                .unwrap()
                .namespace_name(),
            "urn:foreign"
        );
    }
    let (d, root) = dom("<w:bookmarkStart/><x:bookmarkStart/><w:r/>");
    let children = d.elements(root, None);
    assert!(is_range_markup(&d, children[0]));
    assert!(!is_range_markup(&d, children[1]));
    assert!(!is_range_markup(&d, children[2]));
}

// 5142, 5148, 5151-5153: separator inherits clean properties only.
#[test]
fn merge_separator_copies_properties_without_revision_children() {
    for separator in ["", " / "] {
        let (mut d, root) = dom(
            r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange/><w:ins/><w:del/></w:rPr><w:t>head</w:t></w:r></w:p><w:bookmarkStart w:id="1"/><w:p><w:pPr><w:jc w:val="right"/></w:pPr><w:r><w:t>tail</w:t></w:r></w:p>"#,
        );
        let ps = d.elements(root, Some(&W::p()));
        merge_into(&mut d, ps[0], ps[1], separator);
        assert_eq!(d.elements(root, Some(&W::p())), [ps[1]]);
        assert_eq!(
            project_paragraph(&d, ps[1]).text,
            format!("head{separator}tail")
        );
        assert_eq!(d.descendants(ps[1], Some(&W::bookmark_start())).len(), 1);
        let runs = d.elements(ps[1], Some(&W::r()));
        assert_eq!(runs.len(), if separator.is_empty() { 2 } else { 3 });
        if !separator.is_empty() {
            let rpr = d.element(runs[1], &W::r_pr()).unwrap();
            assert_eq!(names(&d, rpr), ["b"]);
            assert_eq!(d.value(d.element(runs[1], &W::t()).unwrap()), separator);
        }
        let ppr = d.element(ps[1], &W::p_pr()).unwrap();
        assert_eq!(
            d.attribute(d.element(ppr, &W::jc_el()).unwrap(), &W::val()),
            Some("right")
        );
    }
}

// 5238, 5244, 5245, 5276, 5285-5287: property cloning and empty run specs.
#[test]
fn new_paragraph_style_and_revision_cleanup_truth_table() {
    for existing_style in [false, true] {
        let style = if existing_style {
            "<w:pStyle w:val=\"Old\"/>"
        } else {
            ""
        };
        let (mut d, root) = dom(&format!(
            "<w:p><w:pPr>{style}<w:keepNext/><w:sectPr/><w:pPrChange/><w:rPr/></w:pPr><w:r><w:rPr><w:i/><w:rPrChange/><w:ins/><w:del/></w:rPr><w:t>anchor</w:t></w:r></w:p>"
        ));
        let anchor = d.element(root, &W::p()).unwrap();
        let original = d.serialize_element(anchor);
        let p = build_paragraph(
            &mut d,
            anchor,
            &[
                RunSpec::default(),
                RunSpec {
                    text: "new".into(),
                    ..RunSpec::default()
                },
            ],
            Some("Chosen"),
        );
        assert_eq!(d.serialize_element(anchor), original);
        assert_eq!(project_paragraph(&d, p).text, "new");
        assert_eq!(d.elements(p, Some(&W::r())).len(), 1);
        let ppr = d.element(p, &W::p_pr()).unwrap();
        assert_eq!(names(&d, ppr), ["pStyle", "keepNext"]);
        assert_eq!(
            d.attribute(d.element(ppr, &W::p_style()).unwrap(), &W::val()),
            Some("Chosen")
        );
        let r = d.element(p, &W::r()).unwrap();
        assert_eq!(names(&d, d.element(r, &W::r_pr()).unwrap()), ["i"]);
        assert_eq!(
            d.attribute(
                d.element(r, &W::t()).unwrap(),
                &XNamespace::xml().name("space")
            ),
            Some("preserve")
        );
    }
}

// 5314, 5317, 5324, 5327, 5336, 5346, 5355.
#[test]
fn replacing_and_clearing_existing_run_properties() {
    for clear in [false, true] {
        let (mut d, root) = dom(
            r#"<w:rPr><w:rFonts w:ascii="Old"/><w:color w:val="000000"/><w:sz w:val="20"/><w:szCs w:val="21"/><w:highlight w:val="yellow"/><w:u w:val="double"/></w:rPr>"#,
        );
        let rpr = d.element(root, &W::r_pr()).unwrap();
        apply_run_format(
            &mut d,
            rpr,
            &RunFormat {
                underline: Some(!clear),
                highlight: Some(if clear { "none" } else { "green" }.into()),
                font: Some("Bundled Name".into()),
                color: Some("aBcD09".into()),
                size_pt: Some(HalfPoints(25)),
                ..RunFormat::default()
            },
        );
        assert_eq!(
            names(&d, rpr),
            if clear {
                vec!["rFonts", "color", "sz", "szCs"]
            } else {
                vec!["rFonts", "color", "sz", "szCs", "highlight", "u"]
            }
        );
        let fonts = d.element(rpr, &W::name("rFonts")).unwrap();
        for script in ["ascii", "hAnsi", "eastAsia", "cs"] {
            assert_eq!(d.attribute(fonts, &W::name(script)), Some("Bundled Name"));
        }
        for size in ["sz", "szCs"] {
            assert_eq!(
                d.attribute(d.element(rpr, &W::name(size)).unwrap(), &W::val()),
                Some("25")
            );
        }
        assert_eq!(
            d.attribute(d.element(rpr, &W::name("color")).unwrap(), &W::val()),
            Some("aBcD09")
        );
        if !clear {
            assert_eq!(
                d.attribute(d.element(rpr, &W::name("u")).unwrap(), &W::val()),
                Some("single")
            );
            assert_eq!(
                d.attribute(d.element(rpr, &W::name("highlight")).unwrap(), &W::val()),
                Some("green")
            );
        }
    }
}

// 5398, 5399, 5406, 5407, 5421. Font names are validated as strings only.
#[test]
fn format_validation_unicode_length_and_color_truth_tables() {
    for (font, valid) in [
        ("é".repeat(31), true),
        ("é".repeat(32), false),
        ("ab\u{0001}cd".into(), false),
    ] {
        let f = RunFormat {
            font: Some(font.clone()),
            ..RunFormat::default()
        };
        assert_eq!(
            check_format(&f, "x"),
            if valid {
                Ok(())
            } else {
                Err(format!(
                    "font {font:?} must be a nonempty name of at most 31 characters"
                ))
            }
        );
    }
    for (color, valid) in [
        ("auto", true),
        ("0aBf19", true),
        ("ABCDEF", true),
        ("12345", false),
        ("12345g", false),
    ] {
        let f = RunFormat {
            color: Some(color.into()),
            ..RunFormat::default()
        };
        assert_eq!(
            check_format(&f, "x"),
            if valid {
                Ok(())
            } else {
                Err(format!(
                    "color {color:?} must be six hex digits (FF0000) or auto"
                ))
            }
        );
    }
    assert_eq!(check_format(&RunFormat::default(), ""), Ok(()));
    assert_eq!(
        check_format(
            &RunFormat {
                bold: Some(false),
                ..RunFormat::default()
            },
            ""
        ),
        Err("format needs nonempty text".into())
    );
}

// 4009: whole deletion skips replacement bookmarks.
#[test]
fn whole_deletion_builds_no_bookmark_and_keeps_surrounding_text() {
    let source = docx(&para("left middle right"));
    let p = plan(
        r#"[{"kind":"replace","paragraph":"p0","find":"middle","replacement":"","whole":true}]"#,
    );
    let mut tx = Transaction::start(&source, &p).unwrap();
    tx.resolve().unwrap();
    tx.apply().unwrap();
    assert_eq!(tx.whole_marks.len(), 0);
    assert_eq!(
        project_paragraph(&tx.opened.dom, tx.paragraph_nodes[0]).text,
        "left  right"
    );
}

// 1402, 1414, 4526: tolerate an unreadable story and ignore foreign revisions.
#[test]
fn revision_and_comment_scans_ignore_rootless_and_foreign_story_parts() {
    let header = format!(
        r#"<w:hdr xmlns:w="{}" xmlns:x="urn:foreign"><w:p><x:ins/><w:r><w:t>header</w:t></w:r></w:p></w:hdr>"#,
        W::URI
    );
    let source = docx_with(
        &para("body"),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
    );
    assert_eq!(revision_counts(&source), RevisionCounts::default());
    let broken = fixture::replace_entry(&source, "word/header1.xml", b"<?xml version=\"1.0\"?>");
    assert_eq!(revision_counts(&broken), RevisionCounts::default());
    assert!(!comment_holds_kept_text(&broken, 7));
    assert_eq!(
        fixture::part_string(&broken, "word/header1.xml").as_deref(),
        Some("<?xml version=\"1.0\"?>")
    );
}
