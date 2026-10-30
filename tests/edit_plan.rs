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
use jubarte::namespaces::W;
use jubarte::xmllinq::Dom;

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
    assert_eq!(op.paragraph.as_deref(), Some("p1"));
    assert_eq!(
        op.context.as_deref(),
        Some("above also signs in {his or her→an} individual capacity")
    );
    assert!(report.revisions.replaced >= 1, "{:?}", report.revisions);
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

// A boundary insertion and a touching removal form one substitution (#1922).
fn assert_boundary_substitution(find: &str, replacement: Option<&str>, edge: &str, first: bool) {
    let source = docx(&format!(
        "<w:p>{}{}{}</w:p>",
        run("Alpha ", true, false, None),
        run("middle ", false, true, None),
        run("omega", false, false, None),
    ));
    let mut removal = serde_json::json!({
        "id": "removal", "kind": "delete", "paragraph": {"index": 0}, "find": find
    });
    if let Some(text) = replacement {
        removal["kind"] = "replace".into();
        removal["replacement"] = text.into();
    }
    let insertion = serde_json::json!({
        "id": "insertion", "kind": "insert", "paragraph": {"index": 0},
        "position": edge, "text": "Reserved. "
    });
    let ops = if first {
        vec![insertion, removal]
    } else {
        vec![removal, insertion]
    };
    let expected = if edge == "start" {
        format!(
            "Reserved. {}{}",
            replacement.unwrap_or_default(),
            &"Alpha middle omega"[find.len()..]
        )
    } else {
        format!(
            "{}{}Reserved. ",
            &"Alpha middle omega"[.."Alpha middle omega".len() - find.len()],
            replacement.unwrap_or_default()
        )
    };
    for keep in [false, true] {
        let mut request = plan(&source, &serde_json::to_string(&ops).unwrap());
        if keep {
            request.existing_revisions = jubarte::edit::ExistingRevisions::Keep;
        }
        let prepared = std::panic::catch_unwind(|| {
            jubarte::edit::prepare(&source, &request, &Default::default())
        })
        .expect("a boundary pair must never panic")
        .unwrap();
        let result = prepared.result;
        assert_eq!(texts(&result.clean), std::slice::from_ref(&expected));
        assert_eq!(
            texts(&accept_revisions(&result.redline).unwrap()),
            std::slice::from_ref(&expected)
        );
        assert_eq!(
            texts(&reject_revisions(&result.redline).unwrap()),
            ["Alpha middle omega"]
        );
        let changes = jubarte::changes::list_changes(&result.redline).unwrap();
        for kind in [
            jubarte::changes::ChangeKind::Deletion,
            jubarte::changes::ChangeKind::Insertion,
        ] {
            assert!(
                changes
                    .iter()
                    .any(|c| c.kind == kind && c.author.as_deref() == Some("Claude")),
                "{changes:?}"
            );
        }
        let marked = marked_paragraphs(&result.redline);
        assert!(
            marked[0].find("[-").unwrap() < marked[0].find("{+").unwrap(),
            "{marked:?}"
        );
        assert_word_valid_package(&result.redline);
    }
}

#[test]
fn edit_boundary_delete_start() {
    assert_boundary_substitution("Alpha ", None, "start", false);
}

#[test]
fn edit_boundary_replace_start() {
    assert_boundary_substitution("Alpha ", Some("New "), "start", false);
}

#[test]
fn edit_boundary_whole_multirun_delete_start() {
    assert_boundary_substitution("Alpha middle omega", None, "start", false);
}

#[test]
fn edit_boundary_insert_first() {
    assert_boundary_substitution("Alpha ", None, "start", true);
    assert_boundary_substitution("Alpha ", Some("New "), "start", true);
    assert_boundary_substitution("Alpha middle omega", None, "start", true);
}

#[test]
fn edit_boundary_end() {
    for first in [false, true] {
        assert_boundary_substitution("omega", None, "end", first);
        assert_boundary_substitution("omega", Some("New "), "end", first);
    }
}

#[test]
fn edit_boundary_substitution_keeps_shared_words_deleted_then_inserted() {
    let source = docx(&para("Alpha middle omega"));
    for keep in [false, true] {
        let mut request = plan(
            &source,
            r#"[
            {"id":"removal","kind":"delete","paragraph":{"index":0},"find":"Alpha middle omega"},
            {"id":"prefix","kind":"insert","paragraph":{"index":0},"position":"start","text":"Alpha "},
            {"id":"suffix","kind":"insert","paragraph":{"index":0},"position":"end","text":"omega"}
        ]"#,
        );
        if keep {
            request.existing_revisions = jubarte::edit::ExistingRevisions::Keep;
        }
        let result = apply_plan(&source, &request).unwrap();
        assert_eq!(
            marked_paragraphs(&result.redline),
            ["[-Alpha middle omega-]{+Alpha omega+}"]
        );
        assert_eq!(
            texts(&accept_revisions(&result.redline).unwrap()),
            ["Alpha omega"]
        );
        assert_eq!(
            texts(&reject_revisions(&result.redline).unwrap()),
            ["Alpha middle omega"]
        );
    }
}

/// The body's text by formatting: adjacent runs of one `(bold, italic)`
/// merged, empty runs dropped.
fn formatted_runs(bytes: &[u8]) -> Vec<(String, bool, bool)> {
    let xml = part_string(bytes, "word/document.xml").unwrap();
    let mut out: Vec<(String, bool, bool)> = Vec::new();
    let mut rest = xml.as_str();
    while let Some(at) = rest
        .find("<w:r>")
        .into_iter()
        .chain(rest.find("<w:r "))
        .min()
    {
        let run = &rest[at..];
        let end = run.find("</w:r>").map_or(run.len(), |e| e + "</w:r>".len());
        let (run, tail) = run.split_at(end);
        rest = tail;
        let mut text = String::new();
        let mut t = run;
        while let Some(open) = t.find("<w:t") {
            let after = &t[open..];
            if !(after.starts_with("<w:t>") || after.starts_with("<w:t ")) {
                t = &after[4..];
                continue;
            }
            let body = &after[after.find('>').unwrap() + 1..];
            let close = body.find("</w:t>").unwrap_or(body.len());
            text.push_str(&body[..close]);
            t = &body[close..];
        }
        if text.is_empty() {
            continue;
        }
        let key = (run.contains("<w:b/>"), run.contains("<w:i/>"));
        match out.last_mut() {
            Some(last) if (last.1, last.2) == key => last.0.push_str(&text),
            _ => out.push((text, key.0, key.1)),
        }
    }
    out
}

#[test]
fn edit_boundary_replace_keeps_its_pieces_beside_an_edge_insert() {
    // A replace writes only the words that changed, so an insert at its
    // edge stays a separate mark: no word it kept is re-marked (#1922).
    let source = docx(&para("Alpha middle omega"));
    for (find, replacement, edge, expected, text) in [
        (
            "Alpha middle",
            "Alpha new",
            "start",
            Some("{+Reserved. +}Alpha [-middle-]{+new+} omega"),
            "Reserved. Alpha new omega",
        ),
        (
            "middle omega",
            "new omega",
            "end",
            // Whether " omega" stays outside the piece depends on the route
            // (existing behaviour); the text checks below hold on both.
            None,
            "Alpha new omegaReserved. ",
        ),
    ] {
        for keep in [false, true] {
            let ops = serde_json::json!([
                {"kind":"replace","paragraph":{"index":0},"find":find,"replacement":replacement},
                {"kind":"insert","paragraph":{"index":0},"position":edge,"text":"Reserved. "}
            ]);
            let mut request = plan(&source, &ops.to_string());
            if keep {
                request.existing_revisions = jubarte::edit::ExistingRevisions::Keep;
            }
            let result = apply_plan(&source, &request).unwrap();
            if let Some(expected) = expected {
                assert_eq!(marked_paragraphs(&result.redline), [expected]);
            }
            assert_eq!(texts(&result.clean), [text]);
            assert_eq!(texts(&accept_revisions(&result.redline).unwrap()), [text]);
            assert_eq!(
                texts(&reject_revisions(&result.redline).unwrap()),
                ["Alpha middle omega"]
            );
        }
    }
}

#[test]
fn edit_boundary_replace_keeps_the_runs_of_its_kept_words() {
    // Review of #1922: rebuilding the replace as one inserted run took the
    // first run's bold for every kept word in the redline only.
    let source = docx(&format!(
        "<w:p>{}{}{}</w:p>",
        run("Alpha ", true, false, None),
        run("middle", false, true, None),
        run(" omega", false, false, None)
    ));
    let request = plan(
        &source,
        r#"[
            {"kind":"replace","paragraph":{"index":0},"find":"Alpha middle omega","replacement":"Alpha XXX omega"},
            {"kind":"insert","paragraph":{"index":0},"position":"start","text":"Reserved. "}
        ]"#,
    );
    let result = apply_plan(&source, &request).unwrap();
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), ["Reserved. Alpha XXX omega"]);
    assert_eq!(formatted_runs(&accepted), formatted_runs(&result.clean));
    let runs = formatted_runs(&accepted);
    let last = runs.last().unwrap();
    assert!(last.0.ends_with(" omega") && !last.1, "{runs:?}");
}

#[test]
fn edit_boundary_unchanged_replace_marks_only_the_insert() {
    // Review of #1922: a replace that changes nothing beside an insert at
    // its edge marks the insert alone, formatted or not, and is not refused.
    let source = docx(&para("Alpha middle omega"));
    for format in ["", r#","format":{"bold":true}"#] {
        let request = plan(
            &source,
            &format!(
                r#"[
                {{"kind":"replace","paragraph":{{"index":0}},"find":"Alpha ","replacement":"Alpha "}},
                {{"kind":"insert","paragraph":{{"index":0}},"position":"start","text":"X. "{format}}}
            ]"#
            ),
        );
        let result = apply_plan(&source, &request).unwrap();
        assert_eq!(
            marked_paragraphs(&result.redline),
            ["{+X. +}Alpha middle omega"]
        );
        assert_eq!(texts(&result.clean), ["X. Alpha middle omega"]);
    }
}

#[test]
fn edit_boundary_formatted_replace_beside_a_plain_insert_is_applied() {
    // Review of #1922: only insertions merged into one delete must share a
    // format; a formatted replace beside a plain edge insert is not refused.
    let source = docx(&para("Alpha middle omega"));
    let request = plan(
        &source,
        r#"[
            {"id":"removal","kind":"replace","paragraph":{"index":0},"find":"Alpha","replacement":"Beta","format":{"bold":true}},
            {"id":"insertion","kind":"insert","paragraph":{"index":0},"position":"start","text":"New "}
        ]"#,
    );
    let result = apply_plan(&source, &request).unwrap();
    assert_eq!(texts(&result.clean), ["New Beta middle omega"]);
    assert_eq!(
        texts(&accept_revisions(&result.redline).unwrap()),
        ["New Beta middle omega"]
    );
}

#[test]
fn edit_boundary_differently_formatted_inserts_around_one_delete_are_refused() {
    // One delete of the whole text takes both edge inserts into one inserted
    // run, which cannot carry two formats: refused, naming both inserts.
    let source = docx(&para("Alpha middle omega"));
    let request = plan(
        &source,
        r#"[
            {"id":"removal","kind":"delete","paragraph":{"index":0},"find":"Alpha middle omega"},
            {"id":"prefix","kind":"insert","paragraph":{"index":0},"position":"start","text":"New ","format":{"bold":true}},
            {"id":"insertion","kind":"insert","paragraph":{"index":0},"position":"end","text":"End"}
        ]"#,
    );
    let error = jubarte::edit::prepare(&source, &request, &Default::default()).unwrap_err();
    assert_eq!(error.code, "OVERLAPPING_EDITS");
    assert!(
        error.message.contains("prefix") && error.message.contains("insertion"),
        "{error:?}"
    );
}

#[test]
fn edit_operation_pairs_never_unwind() {
    // Non-ASCII text also exercises byte/character boundary handling.
    let text = "αbcdef";
    let source = docx(&para(text));
    let mut operations = Vec::new();
    for (start, end) in [(0, 3), (3, 7), (0, 7)] {
        for kind in ["delete", "replace"] {
            let mut op = serde_json::json!({"kind": kind, "paragraph": {"index": 0}, "find": &text[start..end]});
            if kind == "replace" {
                op["replacement"] = "X".into();
            }
            operations.push(op);
        }
    }
    for anchor in [
        serde_json::json!({"position": "start"}),
        serde_json::json!({"before": "cdef"}),
        serde_json::json!({"position": "end"}),
    ] {
        let mut op = serde_json::json!({"kind": "insert", "paragraph": {"index": 0}, "text": "Y"});
        op.as_object_mut()
            .unwrap()
            .extend(anchor.as_object().unwrap().clone());
        operations.push(op);
    }
    let mut panics = Vec::new();
    for a in &operations {
        for b in &operations {
            for keep in [false, true] {
                let mut request = plan(&source, &serde_json::to_string(&[a, b]).unwrap());
                if keep {
                    request.existing_revisions = jubarte::edit::ExistingRevisions::Keep;
                }
                let options = jubarte::edit::PrepareOptions {
                    exact_text: true,
                    ..Default::default()
                };
                if std::panic::catch_unwind(|| jubarte::edit::prepare(&source, &request, &options))
                    .is_err()
                {
                    panics.push(format!("{a}, {b}, keep={keep}"));
                }
            }
        }
    }
    assert!(panics.is_empty(), "prepare panicked for: {panics:#?}");
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
        // A comment operation needs text; only an edit's optional comment
        // may be blank (and then means none).
        r#"[{"kind":"comment","paragraph":{"index":0},"text":"  "}]"#,
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
            r#"[{"kind":"delete","paragraph":{"id":"p7"},"find":"one"}]"#,
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

/// `resolve_revisions` accepts and rejects a selection of the tracked
/// changes before the plan edits; what it leaves follows
/// `existing_revisions`.
#[test]
fn a_plan_resolves_selected_changes_before_it_edits() {
    let body = r#"<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r><w:del w:id="1" w:author="a" w:date="2020-01-01T00:00:00Z"><w:r><w:delText>gone</w:delText></w:r></w:del><w:ins w:id="2" w:author="b" w:date="2020-01-01T00:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins></w:p>"#;
    let source = docx(body);
    let ops = r#"[{"kind":"replace","paragraph":{"index":0},"find":"keep","replacement":"hold"}]"#;
    let with = |extra: &str| {
        format!(
            r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude",{extra},"operations":{ops}}}"#,
            source_sha256(&source)
        )
    };

    let json = with(r#""resolve_revisions":{"accept":{"ids":["r1"]},"reject":{"authors":["b"]}}"#);
    let result = apply_plan_json(&source, &json).unwrap();
    assert_eq!(texts(&result.clean)[0].trim_end(), "hold");
    assert_eq!(result.report.resolved_revisions.accepted, ["r1"]);
    assert_eq!(result.report.resolved_revisions.rejected, ["r2"]);
    assert_ne!(result.report.base_sha256, result.report.source_sha256);
    assert_word_valid_package(&result.redline);

    // A change left tracked is refused by default...
    let json = with(r#""resolve_revisions":{"accept":{"ids":["r1"]}}"#);
    let err = apply_plan_json(&source, &json).unwrap_err();
    assert_eq!(err.code, "EXISTING_REVISIONS");
    // ...or flattened as `existing_revisions` says.
    let json =
        with(r#""resolve_revisions":{"accept":{"ids":["r1"]}},"existing_revisions":"accept""#);
    let result = apply_plan_json(&source, &json).unwrap();
    assert_eq!(texts(&result.clean)[0], "hold new");
    assert_eq!(result.report.resolved_revisions.accepted, ["r1"]);

    // One change accepted and rejected at once is a conflict.
    let json = with(r#""resolve_revisions":{"accept":{"authors":["a"]},"reject":{"ids":["r1"]}}"#);
    let err = apply_plan_json(&source, &json).unwrap_err();
    assert_eq!(err.code, "REVISION_CONFLICT");
    // An id the document does not hold is refused.
    let json = with(r#""resolve_revisions":{"accept":{"ids":["r9"]}}"#);
    let err = apply_plan_json(&source, &json).unwrap_err();
    assert_eq!(err.code, "UNKNOWN_CHANGE");

    // The view's short id and the long id select r1; the report
    // keeps the long id.
    for id in ["r1", "r1"] {
        let json = with(&format!(
            r#""resolve_revisions":{{"accept":{{"ids":["{id}"]}},"reject":{{"authors":["b"]}}}}"#
        ));
        let result = apply_plan_json(&source, &json).unwrap();
        assert_eq!(result.report.resolved_revisions.accepted, ["r1"], "{id}");
    }
    // A comment id is not a change id.
    let json = with(r#""resolve_revisions":{"accept":{"ids":["c1"]}}"#);
    let err = apply_plan_json(&source, &json).unwrap_err();
    assert_eq!(err.code, "INVALID_PLAN");
    assert!(
        err.to_string()
            .contains("c1: expected a change id such as r12, r1+2 or header1.r1"),
        "{err}"
    );
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
fn a_deleted_paragraph_carries_its_comment_in_the_redline_only() {
    let source = docx(
        &(para("(c) Third.")
            + &para("(d) Onward Disclosure. You will not disclose.")
            + &para("5. Confidentiality")),
    );
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"keep","kind":"comment","paragraph":{"index":0},"text":"Fine as is."},
                {"id":"drop","kind":"delete_paragraph","paragraph":{"index":1},"comment":"Covered by section 5."}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), ["(c) Third.", "5. Confidentiality"]);
    let report = &result.report;
    assert_eq!(report.comments_added, 2);
    let ids: Vec<Option<u32>> = report.operations.iter().map(|o| o.comment_id).collect();
    assert_eq!(ids, [Some(0), Some(1)]);
    // The clean copy has no paragraph to hold the deletion's comment.
    assert_eq!(summary(&result.clean).unwrap().comments, 1);
    let comments = part_string(&result.clean, "word/comments.xml").unwrap();
    assert!(!comments.contains("Covered by section 5."), "{comments}");
    // The redline anchors it on the deleted text, under the plan's author.
    assert_eq!(summary(&result.redline).unwrap().comments, 2);
    let xml = part_string(&result.redline, "word/document.xml").unwrap();
    let deleted = &xml[xml
        .find(r#"<w:commentRangeStart w:id="1""#)
        .expect("range start")
        ..xml
            .find(r#"<w:commentRangeEnd w:id="1""#)
            .expect("range end")];
    assert!(
        deleted.contains("<w:delText>(d) Onward Disclosure. You will not disclose.</w:delText>"),
        "{deleted}"
    );
    let comments = part_string(&result.redline, "word/comments.xml").unwrap();
    let comment = &comments[comments.find(r#"w:id="1""#).expect("comment 1")..];
    assert!(comment.contains("Covered by section 5."), "{comments}");
    assert!(comment.contains(r#"w:author="Claude""#), "{comments}");
    // Accepting gives the clean text; rejecting brings the paragraph back.
    let accepted = accept_revisions(&result.redline).unwrap();
    assert_eq!(texts(&accepted), texts(&result.clean));
    let rejected = reject_revisions(&result.redline).unwrap();
    assert_eq!(texts(&rejected), texts(&source));
    assert_word_valid_package(&result.redline);
    assert_word_valid_package(&result.clean);
}

#[test]
fn a_deleted_paragraphs_comment_is_refused_when_its_twin_is_deleted_instead() {
    // Paragraphs 2 and 3 read the same: the comparer keeps the commented
    // one and deletes the other, so the comment would sit on text that stays.
    let source = std::fs::read("tests/fixtures/redline/original.docx").unwrap();
    assert_eq!(texts(&source)[2], texts(&source)[3]);
    let error = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"drop","kind":"delete_paragraph","paragraph":{"index":2},"comment":"Duplicate."}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.operation.as_deref()),
        ("UNSUPPORTED_STRUCTURE", Some("drop"))
    );
    assert!(error.message.contains("identical"), "{}", error.message);
    let outcome = &error.outcomes[0];
    assert_eq!(
        (outcome.status.as_str(), outcome.code.as_deref()),
        ("failed", Some("UNSUPPORTED_STRUCTURE"))
    );
    // Without the comment the same deletion goes through.
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"delete_paragraph","paragraph":{"index":2}}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean).len(), texts(&source).len() - 1);
}

#[test]
fn a_deleted_paragraphs_comment_is_checked_like_any_comment() {
    let source = docx(&(para("Keep.") + &para("Drop.")));
    let error = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"drop","kind":"delete_paragraph","paragraph":{"index":1},"comment":"bad\u0001"}]"#,
        ),
    )
    .unwrap_err();
    assert_eq!(
        (error.code.as_str(), error.operation.as_deref()),
        ("INVALID_EDIT", Some("drop"))
    );
}

/// A paragraph that carries a section break is refused. A cell's only
/// paragraph is emptied, not removed: Word never deletes a container's final
/// paragraph mark (#1668).
#[test]
fn delete_paragraph_refuses_a_section_paragraph_and_empties_a_cells_only_one() {
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
    // A second cell keeps its words, so the row stays.
    let table = format!(
        "{}<w:tbl><w:tr><w:tc>{}</w:tc><w:tc>{}</w:tc></w:tr></w:tbl>",
        para("a"),
        para("only cell paragraph"),
        para("beside")
    );
    let source = docx(&table);
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"delete_paragraph","paragraph":{"index":1}}]"#,
        ),
    )
    .unwrap();
    let accepted = accept_revisions(&result.redline).unwrap();
    for (what, output) in [("clean", &result.clean), ("accepted", &accepted)] {
        assert_eq!(texts(output), ["a", "", "beside"], "{what}");
        assert_eq!(
            cell_endings(output),
            [Some(String::new()), Some("beside".to_string())],
            "{what}"
        );
    }
    assert_eq!(
        texts(&reject_revisions(&result.redline).unwrap()),
        ["a", "only cell paragraph", "beside"]
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
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
fn edits_inside_hyperlinks_or_across_symbols_are_unsupported_structure() {
    let body = r#"<w:p><w:r><w:t>Name:</w:t><w:tab/><w:t>Arthur</w:t></w:r></w:p><w:p><w:hyperlink r:id="rId9"><w:r><w:t>arthur.law</w:t></w:r></w:hyperlink><w:r><w:t xml:space="preserve"> site</w:t></w:r></w:p><w:p><w:r><w:t>Box</w:t><w:sym w:font="Wingdings" w:char="F0FC"/><w:t>ticked</w:t></w:r></w:p>"#;
    let source = docx(body);
    for ops in [
        r#"[{"kind":"replace","paragraph":{"index":1},"find":"arthur.law","replacement":"x"}]"#,
        r#"[{"kind":"replace","paragraph":{"index":2},"find":"Box￼ticked","replacement":"x"}]"#,
    ] {
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert_eq!(err.code, "UNSUPPORTED_STRUCTURE", "{ops}");
    }
    // A tab is no obstacle: an edit over it removes it, one beside it keeps it.
    for (find, replacement, want) in [
        ("Name:\\tArthur", "x", "x"),
        ("Arthur", "Souza", "Name:\tSouza"),
    ] {
        let ops = format!(
            r#"[{{"kind":"replace","paragraph":{{"index":0}},"find":"{find}","replacement":"{replacement}"}}]"#
        );
        let result = apply_plan(&source, &plan(&source, &ops)).unwrap();
        assert_eq!(texts(&result.clean)[0], want);
    }
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
    // The counts are applying's: one replacement (plan 11).
    assert_eq!(
        (report.revisions.replaced, report.revisions.total),
        (1, 1),
        "{:?}",
        report.revisions
    );
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
            f.name().expect("decode ZIP fixture name").into_owned(),
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
    assert_eq!(lines[1]["at"], "p0");
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
        serde_json::json!({"position":"end","text":"a\rb"}),
        serde_json::json!({"position":"end","text":"a\u{0}b"}),
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
fn deletions_are_checked_together_so_a_cell_keeps_its_closing_paragraph() {
    // Two paragraphs in one cell: deleting both empties the last one, whose
    // mark Word never deletes (#1668). A second cell keeps the row.
    let table = format!(
        "<w:tbl><w:tr><w:tc>{}{}</w:tc><w:tc>{}</w:tc></w:tr></w:tbl>{}",
        para("cell one"),
        para("cell two"),
        para("beside"),
        para("after")
    );
    let source = docx(&table);
    let both = r#"[{"kind":"delete_paragraph","paragraph":{"index":0}},
                   {"kind":"delete_paragraph","paragraph":{"index":1}}]"#;
    let result = apply_plan(&source, &plan(&source, both)).unwrap();
    let accepted = accept_revisions(&result.redline).unwrap();
    for (what, output) in [("clean", &result.clean), ("accepted", &accepted)] {
        assert_eq!(texts(output), ["", "beside", "after"], "{what}");
        assert_eq!(
            cell_endings(output),
            [Some(String::new()), Some("beside".to_string())],
            "{what}"
        );
    }
    assert_eq!(
        texts(&reject_revisions(&result.redline).unwrap()),
        ["cell one", "cell two", "beside", "after"]
    );
    assert_word_valid_package(&result.clean);
    assert_word_valid_package(&result.redline);
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

    // A cell's last paragraph after a nested table is emptied, not removed:
    // Word requires the cell to close with a paragraph.
    let nested = format!(
        "<w:tbl><w:tr><w:tc>{}<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>{}</w:tc></w:tr></w:tbl>{}",
        para("outer first"),
        para("inner"),
        para("outer last"),
        para("after")
    );
    let source3 = docx(&nested);
    let last = r#"[{"kind":"delete_paragraph","paragraph":{"index":2}}]"#;
    let emptied = apply_plan(&source3, &plan(&source3, last)).unwrap();
    assert_eq!(texts(&emptied.clean), ["outer first", "inner", "", "after"]);
    assert_eq!(
        cell_endings(&emptied.clean),
        [Some(String::new()), Some("inner".to_string())]
    );
    assert_eq!(
        texts(&reject_revisions(&emptied.redline).unwrap()),
        ["outer first", "inner", "outer last", "after"]
    );
    assert_word_valid_package(&emptied.clean);
    assert_word_valid_package(&emptied.redline);
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

/// Each table cell's closing block, in document order: the text of its last
/// paragraph, or `None` when the cell does not end in a paragraph.
fn cell_endings(docx: &[u8]) -> Vec<Option<String>> {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::tc()))
        .into_iter()
        .map(|cell| {
            let last = *dom.elements(cell, None).last()?;
            dom.name_is(last, &W::p()).then(|| {
                dom.descendants(last, Some(&W::t()))
                    .into_iter()
                    .map(|t| dom.value(t))
                    .collect()
            })
        })
        .collect()
}

/// Body paragraphs of a redline as `kept[-deleted-]{+inserted+}`.
fn marked_paragraphs(docx: &[u8]) -> Vec<String> {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    dom.descendants(root, Some(&W::p()))
        .into_iter()
        .map(|p| {
            let mut segs: Vec<(char, String)> = Vec::new();
            for t in dom.descendants(p, None) {
                let kind = if dom.name_is(t, &W::del_text()) {
                    '-'
                } else if dom.name_is(t, &W::t()) {
                    let inserted = dom
                        .ancestors(t, None)
                        .into_iter()
                        .take_while(|&a| a != p)
                        .any(|a| dom.name_is(a, &W::ins()));
                    if inserted { '+' } else { '=' }
                } else {
                    continue;
                };
                match segs.last_mut() {
                    Some((k, s)) if *k == kind => s.push_str(&dom.value(t)),
                    _ => segs.push((kind, dom.value(t))),
                }
            }
            segs.into_iter()
                .map(|(k, s)| match k {
                    '+' => format!("{{+{s}+}}"),
                    '-' => format!("[-{s}-]"),
                    _ => s,
                })
                .collect()
        })
        .collect()
}

/// `whole: true` shows a replacement the way typing over a selection with
/// Track Changes on does: all of `find` deleted, then all of the replacement
/// inserted. Without it the comparer shows Word Compare's word-level diff,
/// which keeps shared words ("within", "days") as unchanged text.
#[test]
fn whole_replace_deletes_the_find_text_then_inserts_the_replacement() {
    let body = [
        "You will notify us within five business days of any breach.",
        "You will promptly notify us of any breach.",
    ];
    let source = docx(&(para("Heading") + &para(body[0]) + &para(body[1])));
    let ops = |whole: &str| {
        format!(
            r#"[{{"kind":"replace","paragraph":{{"index":1}},"find":"within five business days","replacement":"within ten days"{whole}}},
                {{"kind":"replace","paragraph":{{"index":2}},"find":"promptly notify","replacement":"notify"{whole}}}]"#
        )
    };
    let word_level = apply_plan(&source, &plan(&source, &ops(""))).unwrap();
    let marked = marked_paragraphs(&word_level.redline);
    assert!(
        marked[1].contains("within {+ten+}[-five business-] days")
            || marked[1].contains("within [-five business-]{+ten+} days"),
        "default stays word-level: {}",
        marked[1]
    );

    let whole = apply_plan(&source, &plan(&source, &ops(r#","whole":true"#))).unwrap();
    assert_eq!(
        marked_paragraphs(&whole.redline)[1..],
        [
            "You will notify us [-within five business days-]{+within ten days+} of any breach.",
            "You will [-promptly notify-]{+notify+} us of any breach.",
        ]
    );
    // Same edit, same clean text; the redline still round-trips.
    assert_eq!(texts(&whole.clean), texts(&word_level.clean));
    assert_eq!(
        texts(&accept_revisions(&whole.redline).unwrap()),
        texts(&whole.clean)
    );
    assert_eq!(
        texts(&reject_revisions(&whole.redline).unwrap()),
        texts(&source)
    );
    for doc in [&whole.clean, &whole.redline] {
        let xml = part_string(doc, "word/document.xml").unwrap();
        assert!(!xml.contains("_jubarte_whole"), "no helper bookmark left");
        assert_word_valid_package(doc);
    }
    assert!(whole.report.ok);
    assert!(whole.report.operations.iter().all(|o| o.message.is_none()));
    // Each deletion and the insertion after it is one replacement.
    assert_eq!(
        (
            whole.report.revisions.deleted,
            whole.report.revisions.inserted,
            whole.report.revisions.replaced
        ),
        (0, 0, 2)
    );
}

/// A whole replacement across runs keeps each run's formatting on its side
/// of the change and keeps its comment.
#[test]
fn whole_replace_across_runs_keeps_formatting_and_comment() {
    let source = docx(&format!(
        "<w:p>{}{}</w:p>",
        run("Notices. Send notices by ", false, false, None),
        run("email to Alice", true, false, None) + &run(" within two days.", false, false, None)
    ));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"replace","paragraph":{"index":0},"find":"by email to","replacement":"by courier to","whole":true,"comment":"Courier only."}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        marked_paragraphs(&result.redline),
        ["Notices. Send notices [-by email to-]{+by courier to+} Alice within two days."]
    );
    let xml = part_string(&result.redline, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let document = dom.parse_xdocument(&xml);
    let root = dom.root(document).unwrap();
    let mut seen = std::collections::HashSet::new();
    let mut count = 0;
    for node in dom.descendants(root, None) {
        let Some(name) = dom.name(node) else { continue };
        if name.namespace_name() == W::URI && matches!(name.local_name(), "rPrChange" | "pPrChange")
        {
            let id = dom.attribute(node, &W::id()).unwrap();
            assert!(
                seen.insert((name.local_name().to_string(), id.to_string())),
                "duplicate {}:{id} in word/document.xml",
                name.local_name()
            );
            count += 1;
        }
    }
    // Re-typed whole, the span is one deletion and one insertion: neither
    // side carries a formatting change of its own (#1375).
    assert_eq!(
        count, 0,
        "no w:rPrChange on either side of a whole replacement"
    );
    assert_eq!(xml.matches("<w:commentReference ").count(), 1);
    // "email to" was bold in the source; its deleted copy stays bold.
    let del = &xml[xml.find("<w:del ").unwrap()..xml.find("</w:del>").unwrap()];
    assert!(del.contains("<w:b/>") || del.contains("<w:b />"), "{del}");
    assert_word_valid_package(&result.redline);
}

/// Annotation ids are independent of the revision-record allocator.
#[test]
fn whole_replace_ignores_div_and_bookmark_ids() {
    let source = docx_with(
        r#"<w:p><w:bookmarkStart w:id="900000" w:name="Alice"/><w:ins w:id="10" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>Send notices by email to Alice.</w:t></w:r></w:ins><w:bookmarkEnd w:id="900000"/></w:p>"#,
        &[Part {
            name: "word/webSettings.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.webSettings+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/webSettings",
            xml: r#"<w:webSettings xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:divs><w:div w:id="2147483647"/></w:divs></w:webSettings>"#,
        }],
    );
    let mut edit = plan(
        &source,
        r#"[{"kind":"replace","paragraph":{"index":0},"find":"by email to","replacement":"by courier to","whole":true}]"#,
    );
    edit.existing_revisions = jubarte::edit::ExistingRevisions::Accept;
    let result = apply_plan(&source, &edit).unwrap();
    let xml = part_string(&result.redline, "word/document.xml").unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let root = dom.root(doc).unwrap();
    let ids: Vec<u64> = dom
        .descendants(root, None)
        .into_iter()
        .filter_map(|n| {
            let name = dom.name(n)?;
            (name.namespace_name() == W::URI
                && matches!(name.local_name(), "ins" | "del" | "rPrChange" | "pPrChange"))
            .then(|| dom.attribute(n, &W::id()).unwrap().parse().unwrap())
        })
        .collect();
    assert!(ids.len() >= 2, "both replacement sides must be tracked");
    let bound = 10 + ids.len() as u64;
    assert!(
        ids.iter().all(|&id| id <= bound && id <= 2147483647),
        "revision ids {ids:?} exceed input revision max 10 + {} records (bound {bound}) or signed 32-bit range",
        ids.len()
    );
    assert_eq!(
        marked_paragraphs(&result.redline),
        ["Send notices [-by email to-]{+by courier to+} Alice."]
    );
    assert_word_valid_package(&result.redline);
}

/// `whole` is part of the replace wire schema only.
#[test]
fn whole_is_a_replace_field() {
    let source = docx(&para(SIGNATURE));
    let json =
        |op: &str| format!(r#"{{"schema_version":1,"author":"Claude","operations":[{op}]}}"#);
    let bad = EditPlan::from_json(&json(
        r#"{"kind":"delete","paragraph":{"index":0},"find":"his","whole":true}"#,
    ))
    .unwrap_err();
    assert_eq!(bad.code, "INVALID_PLAN");
    let good = EditPlan::from_json(&json(
        r#"{"kind":"replace","paragraph":{"index":0},"find":"his or her","replacement":"an","whole":true}"#,
    ))
    .unwrap();
    assert!(good.to_json().contains(r#""whole": true"#));
    assert!(apply_plan(&source, &good).unwrap().report.ok);
}

#[test]
fn rewrite_changes_only_the_words_that_differ() {
    let source = docx(&format!(
        "<w:p>{}{}{}</w:p>{}",
        run("The ", false, false, None),
        run("Buyer", true, false, None),
        run(" shall pay within thirty days.", false, false, None),
        para("Name:\tArthur"),
    ));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"r","kind":"rewrite","paragraph":{"index":0},"text":"The Buyer shall pay the price within forty-five days."}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean)[0],
        "The Buyer shall pay the price within forty-five days."
    );
    // The bold word keeps its run and formatting; the report names the op.
    let clean = paragraphs(&result.clean).unwrap();
    let bold: Vec<(usize, usize)> = clean[0]
        .runs
        .iter()
        .filter(|r| r.bold)
        .map(|r| (r.start, r.end))
        .collect();
    assert_eq!(bold, [(4, 9)]);
    let op = &result.report.operations[0];
    assert_eq!((op.kind.as_str(), op.status.as_str()), ("rewrite", "ok"));
    assert_eq!(
        texts(&accept_revisions(&result.redline).unwrap()),
        texts(&result.clean)
    );
    assert_eq!(
        texts(&reject_revisions(&result.redline).unwrap()),
        texts(&source)
    );
    assert_word_valid_package(&result.redline);

    // A tab stays; the text may write it as a space.
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"rewrite","paragraph":{"index":1},"text":"Name: Arthur Souza"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean)[1], "Name:\tArthur Souza");

    // Unchanged text is no edit at all.
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"rewrite","paragraph":{"index":1},"text":"Name: Arthur"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean), texts(&source));
    assert_eq!(
        result.report.revisions.inserted + result.report.revisions.deleted,
        0
    );
}

#[test]
fn rewrite_replaces_a_decomposed_word_with_its_accent() {
    // "cafe" + U+0301 (decomposed), as text copied from a Mac file name or a
    // PDF arrives, with the accent in a run of its own: the word and its
    // accent are one change, so the new word takes the old word's formatting
    // whole instead of leaving the old accent behind in the plain run.
    let source = docx(&format!(
        "<w:p>{}{}{}</w:p>",
        run("Le ", false, false, None),
        run("cafe", true, false, None),
        run("\u{301} est chaud.", false, false, None),
    ));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"r","kind":"rewrite","paragraph":{"index":0},"text":"Le the\u0301 est chaud."}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean)[0], "Le the\u{301} est chaud.");
    let op = &result.report.operations[0];
    assert_eq!((op.kind.as_str(), op.status.as_str()), ("rewrite", "ok"));
    let clean = paragraphs(&result.clean).unwrap();
    let bold: Vec<String> = clean[0]
        .runs
        .iter()
        .filter(|r| r.bold)
        .map(|r| {
            clean[0]
                .text
                .chars()
                .skip(r.start)
                .take(r.end - r.start)
                .collect()
        })
        .collect();
    assert_eq!(bold, ["the\u{301}"]);
    let redline_xml = part_string(&result.redline, "word/document.xml").unwrap();
    // The accent is deleted with its word (in a run of its own, as in the
    // source) and the new word is inserted with its accent.
    assert!(redline_xml.contains(">cafe</w:delText>"), "{redline_xml}");
    assert!(
        redline_xml.contains(">\u{301}</w:delText>"),
        "{redline_xml}"
    );
    assert!(redline_xml.contains(">the\u{301}</w:t>"), "{redline_xml}");
    assert_eq!(
        texts(&reject_revisions(&result.redline).unwrap()),
        texts(&source)
    );
    assert_word_valid_package(&result.redline);
}

#[test]
fn rewrite_refuses_what_replace_refuses() {
    let body = r#"<w:p><w:hyperlink r:id="rId9"><w:r><w:t>arthur.law</w:t></w:r></w:hyperlink><w:r><w:t xml:space="preserve"> site</w:t></w:r></w:p><w:p/>"#;
    let source = docx(body);
    for ops in [
        r#"[{"kind":"rewrite","paragraph":{"index":0},"text":"jubarte.law site"}]"#,
        r#"[{"kind":"rewrite","paragraph":{"index":0},"text":"bell\u0007"}]"#,
    ] {
        let err = apply_plan(&source, &plan(&source, ops)).unwrap_err();
        assert!(
            matches!(err.code.as_str(), "UNSUPPORTED_STRUCTURE" | "INVALID_EDIT"),
            "{ops}: {err:?}"
        );
    }
    // Text beside the link can change.
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"rewrite","paragraph":{"index":0},"text":"arthur.law website"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean)[0], "arthur.law website");
    // An empty paragraph takes the text in a new run (#1263).
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"rewrite","paragraph":{"index":1},"text":"into an empty paragraph"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&result.clean)[1], "into an empty paragraph");
    assert_word_valid_package(&result.redline);
    let unknown = EditPlan::from_json(
        r#"{"schema_version":1,"author":"a","operations":[{"kind":"rewrite","paragraph":{"index":0},"text":"x","find":"y"}]}"#,
    );
    assert!(unknown.is_err());
}

#[test]
fn insert_paragraph_like_copies_another_paragraphs_properties() {
    let numbered = r#"<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr></w:pPr><w:r><w:t>an item</w:t></w:r></w:p>"#;
    let plain = r#"<w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:t>centered</w:t></w:r></w:p>"#;
    let source = docx(&format!("{plain}{numbered}"));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert_paragraph","paragraph":{"index":1},"position":"after","like":{"index":0},"runs":[{"text":"after the list"}]}]"#,
        ),
    )
    .unwrap();
    let paragraphs = paragraphs(&result.clean).unwrap();
    assert_eq!(paragraphs[2].text, "after the list");
    assert!(
        !paragraphs[2].numbered,
        "properties come from `like`, not the anchor"
    );
    let xml = part_string(&result.clean, "word/document.xml").unwrap();
    assert_eq!(xml.matches(r#"<w:jc w:val="center""#).count(), 2, "{xml}");
    // A `like` that selects nothing fails the plan.
    let err = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"kind":"insert_paragraph","paragraph":{"index":1},"like":{"index":9},"runs":[{"text":"x"}]}]"#,
        ),
    )
    .unwrap_err();
    assert!(err.message.starts_with("like:"), "{err:?}");
}

/// The edit report counts what `jubarte changes` lists: on the keep path,
/// this edit's changes beside another author's; on the comparer path, every
/// change of the redline, as `summary` counts it.
#[test]
fn the_report_counts_the_listed_changes() {
    let body = r#"<w:p><w:r><w:t xml:space="preserve">keep this </w:t></w:r><w:ins w:id="1" w:author="a" w:date="2020-01-01T00:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>Second line</w:t></w:r></w:p>"#;
    let source = docx(body);
    let ops = r#"[{"kind":"replace","paragraph":{"index":0},"find":"keep","replacement":"hold"},{"kind":"replace","paragraph":{"index":1},"find":"Second","replacement":"Next"}]"#;
    let json = |existing: &str| {
        format!(
            r#"{{"schema_version":1,"source_sha256":"{}","author":"Claude","date":"2026-10-01T09:00:00Z","existing_revisions":"{existing}","operations":{ops}}}"#,
            source_sha256(&source)
        )
    };
    // Counted as `jubarte changes` lists them: logical changes (plan 11).
    let kept = apply_plan_json(&source, &json("keep")).unwrap();
    let listed = jubarte::changes::list_logical_changes(&kept.redline).unwrap();
    let ours = listed
        .iter()
        .filter(|c| {
            c.author.as_deref() == Some("Claude")
                && c.date.as_deref() == Some("2026-10-01T09:00:00Z")
        })
        .count();
    assert!(ours > 0 && ours < listed.len(), "{listed:?}");
    assert_eq!(kept.report.revisions.total, ours);

    let compared = apply_plan_json(&source, &json("accept")).unwrap();
    let total = compared.report.revisions.total;
    assert!(total > 0);
    assert_eq!(total, summary(&compared.redline).unwrap().revisions);
    assert_eq!(
        total,
        jubarte::changes::list_logical_changes(&compared.redline)
            .unwrap()
            .len()
    );
    let counts = compared.report.revisions;
    assert_eq!(
        counts.inserted + counts.deleted + counts.moved + counts.format_changed + counts.replaced,
        total
    );
}

#[test]
fn a_blank_optional_comment_means_no_comment() {
    // An agent that has nothing to say writes "comment": "" (or spaces);
    // that is no comment, not an invalid edit.
    let source = docx(&(para("Heading") + &para(SIGNATURE)));
    let result = apply_plan(
        &source,
        &plan(
            &source,
            r#"[{"id":"op-1","kind":"replace","paragraph":"p1","find":"his or her","replacement":"an","comment":""},
                {"id":"op-2","kind":"insert_paragraph","paragraph":"p1","runs":[{"text":"New."}],"comment":"   "},
                {"id":"op-4","kind":"delete_paragraph","paragraph":"p0","comment":" \n "}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&result.clean),
        [
            "The individual signing above also signs in an individual capacity.",
            "New."
        ]
    );
    assert!(
        jubarte::comments::list_comments(&result.redline)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_preview_counts_the_comments_the_plan_adds() {
    // The dry run's summary is what applying the plan reports.
    let source = docx(&(para("Heading") + &para(SIGNATURE)));
    let plan = plan(
        &source,
        r#"[{"id":"op-1","kind":"replace","paragraph":"p1","find":"his or her","replacement":"an","comment":"Neutral."},
            {"id":"op-2","kind":"comment","paragraph":"p0","text":"Rename?"},
            {"id":"op-3","kind":"insert_paragraph","paragraph":"p1","runs":[{"text":"New."}]}]"#,
    );
    let applied = apply_plan(&source, &plan).unwrap();
    assert_eq!(applied.report.comments_added, 2);
    assert_eq!(preview_plan(&source, &plan).unwrap().comments_added, 2);
}

#[test]
fn a_preview_counts_the_revisions_and_paragraphs_applying_makes() {
    // bench 2026-10-15: one insert dry-ran as `revs: total 0` and
    // `paras: 257 → 257`; applying it made a tracked insertion.
    let source = docx(&(para("Heading") + &para(SIGNATURE)));
    let plan = plan(
        &source,
        r#"[{"id":"op-1","kind":"insert","paragraph":"p0","position":"end","text":" One"},
            {"id":"op-2","kind":"insert_paragraph","paragraph":"p1","runs":[{"text":"New."}]}]"#,
    );
    let applied = apply_plan(&source, &plan).unwrap().report;
    let preview = preview_plan(&source, &plan).unwrap();
    assert!(applied.revisions.inserted > 0, "{:?}", applied.revisions);
    assert_eq!(preview.revisions, applied.revisions);
    assert_eq!(preview.paragraphs, applied.paragraphs);
    assert_eq!(preview.paragraphs.to, preview.paragraphs.from + 1);
}

#[test]
fn check_plan_resolves_without_applying() {
    // `add --plan` checks each recorded operation with it: the same
    // refusals as a preview, without the preview's in-memory apply.
    let source = docx(&para("alpha beta"));
    let good = plan(
        &source,
        r#"[{"kind":"replace","paragraph":"p0","find":"beta","replacement":"gamma"}]"#,
    );
    jubarte::edit::check_plan(&source, &good).unwrap();
    let bad = plan(
        &source,
        r#"[{"kind":"replace","paragraph":"p0","find":"zzz","replacement":"gamma"}]"#,
    );
    let checked = jubarte::edit::check_plan(&source, &bad).unwrap_err();
    let previewed = preview_plan(&source, &bad).unwrap_err();
    assert_eq!(checked.code, "ANCHOR_NOT_FOUND");
    assert_eq!(checked.code, previewed.code);
}
