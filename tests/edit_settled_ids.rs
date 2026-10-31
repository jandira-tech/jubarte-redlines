// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! A whole-paragraph operation names a source paragraph that settling the
//! plan's revisions merged (#1269). It follows the merge when the settled
//! paragraph has that one source. When several source paragraphs share it,
//! the refusal names each id in its view and a route that reaches the
//! settled paragraph, and no message repeats one id.

mod common;

use common::docx::docx;
use common::validity::assert_word_valid_package;
use jubarte::edit::{EditPlan, PlanSyntax, PrepareOptions, apply_plan, prepare};
use jubarte::inspect::paragraphs;

const REV: &str = r#"w:author="Other" w:date="2026-09-01T00:00:00Z""#;
/// A second author: their marks are changes apart from `REV`'s even where
/// they touch (plan 11).
const REV_B: &str = r#"w:author="Second" w:date="2026-09-01T00:00:00Z""#;

/// As read: p0 "Zero.", p1 deleted whole, p2 "Two.", p3 "Three" whose mark
/// is deleted (it merges with p4 " four."), p5 "Five.". Settled under
/// accept: p0 "Zero.", p1 "Two.", p2 "Three four.", p3 "Five.".
fn merging() -> Vec<u8> {
    merging_with("Three", " four.")
}

/// `merging()` with p3's and p4's words given.
fn merging_with(three: &str, four: &str) -> Vec<u8> {
    docx(&[
        "<w:p><w:r><w:t>Zero.</w:t></w:r></w:p>".to_string(),
        format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="2" {REV}/></w:rPr></w:pPr><w:del w:id="3" {REV}><w:r><w:delText>One gone.</w:delText></w:r></w:del></w:p>"#
        ),
        "<w:p><w:r><w:t>Two.</w:t></w:r></w:p>".to_string(),
        format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>{three}</w:t></w:r></w:p>"#
        ),
        format!(r#"<w:p><w:r><w:t xml:space="preserve">{four}</w:t></w:r></w:p>"#),
        "<w:p><w:r><w:t>Five.</w:t></w:r></w:p>".to_string(),
    ]
    .concat())
}

fn plan(settle: &str, operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Ann Counsel","date":"2026-10-01T09:00:00Z",{settle},"operations":[{operations}]}}"#
    ))
    .unwrap()
}

const ACCEPT: &str = r#""existing_revisions":"accept""#;
const REJECT: &str = r#""existing_revisions":"reject""#;

fn texts(bytes: &[u8]) -> Vec<String> {
    paragraphs(bytes)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn refusal(source: &[u8], settle: &str, operation: &str) -> String {
    let e = apply_plan(source, &plan(settle, operation)).unwrap_err();
    assert_eq!(e.code, "PARAGRAPH_RESOLVED", "{operation}: {e}");
    e.message
}

/// Whether `message` names one paragraph id twice. A selector it quotes
/// (`({"contains":"See p3"})`) holds the document's words, not ids, and is
/// left out.
fn repeats_an_id(message: &str) -> bool {
    let mut prose = String::new();
    let mut rest = message;
    while let Some(at) = rest.find("({") {
        prose.push_str(&rest[..at]);
        let mut selector =
            serde_json::Deserializer::from_str(&rest[at + 1..]).into_iter::<serde_json::Value>();
        selector.next().unwrap().unwrap();
        rest = &rest[at + 1 + selector.byte_offset()..];
    }
    prose.push_str(rest);
    let ids: Vec<&str> = prose
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() > 1 && w.starts_with('p') && w[1..].bytes().all(|b| b.is_ascii_digit()))
        .collect();
    ids.iter()
        .enumerate()
        .any(|(i, id)| ids[i + 1..].contains(id))
}

/// p3's mark is deleted and p4 is empty, so accepting the mark leaves p3's
/// words alone in one settled paragraph; rejecting the deletion of "!"
/// inside p3 changes its text. That paragraph has one source: p3. The "!"
/// is another author's deletion: one author's deleted "!" and mark would be
/// one change, which a plan settles whole.
fn merging_alone() -> Vec<u8> {
    docx(&[
        "<w:p><w:r><w:t>Zero.</w:t></w:r></w:p>".to_string(),
        "<w:p><w:r><w:t>Two.</w:t></w:r></w:p>".to_string(),
        format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>Three</w:t></w:r><w:del w:id="5" {REV_B}><w:r><w:delText>!</w:delText></w:r></w:del></w:p>"#
        ),
        "<w:p/>".to_string(),
        "<w:p><w:r><w:t>Five.</w:t></w:r></w:p>".to_string(),
    ]
    .concat())
}

const ACCEPT_MARK_REJECT_BANG: &str =
    r#""resolve_revisions":{"accept":{"ids":["r1"]},"reject":{"ids":["r5"]}}"#;

#[test]
fn one_to_one_merge_resolves_without_a_refusal() {
    let source = merging_alone();
    for op in [
        r#"{"kind":"delete_paragraph","paragraph":"p2"}"#,
        r#"{"kind":"rewrite","paragraph":"p2","text":"Three, rewritten."}"#,
    ] {
        let out = apply_plan(&source, &plan(ACCEPT_MARK_REJECT_BANG, op)).unwrap();
        for report in &out.report.operations {
            assert_eq!(report.status, "ok", "{op}: {report:?}");
        }
        let clean = texts(&out.clean);
        assert!(!clean.iter().any(|t| t == "Three!"), "{op}: {clean:?}");
        assert_eq!(clean.first().map(String::as_str), Some("Zero."), "{op}");
        assert_eq!(clean.last().map(String::as_str), Some("Five."), "{op}");
        assert_word_valid_package(&out.clean);
        assert_word_valid_package(&out.redline);
    }
}

#[test]
fn many_to_one_merge_is_refused_naming_both_ids_and_views() {
    let source = merging();
    assert_eq!(
        refusal(
            &source,
            ACCEPT,
            r#"{"kind":"delete_paragraph","paragraph":"p3"}"#
        ),
        r#"p3 (as read) merges into p2 (after the plan's revisions settle) together with p4; an operation on a whole paragraph needs a paragraph with one source, so select it by its text instead ({"contains":"Three four."}), or accept the revisions first (`jubarte accept FILE -o OUT.docx`) and edit OUT.docx"#
    );
    assert_eq!(
        refusal(
            &source,
            ACCEPT,
            r#"{"kind":"rewrite","paragraph":"p4","text":"Four."}"#
        ),
        r#"p4 (as read) merges into p2 (after the plan's revisions settle) together with p3; an operation on a whole paragraph needs a paragraph with one source, so select it by its text instead ({"contains":"Three four."}), or accept the revisions first (`jubarte accept FILE -o OUT.docx`) and edit OUT.docx"#
    );
}

/// p0's mark is deleted: p0 "Alpha" and p1 "Beta" settle under accept into
/// p0 "AlphaBeta".
fn alpha_beta() -> Vec<u8> {
    docx(&[
        format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>Alpha</w:t></w:r></w:p>"#
        ),
        "<w:p><w:r><w:t>Beta</w:t></w:r></w:p>".to_string(),
        "<w:p><w:r><w:t>Gamma</w:t></w:r></w:p>".to_string(),
    ]
    .concat())
}

/// The settled paragraph's id is left out when one of the source ids
/// already in the message is the same string.
#[test]
fn a_settled_id_equal_to_a_source_id_is_not_repeated() {
    let source = alpha_beta();
    let route = r#"an operation on a whole paragraph needs a paragraph with one source, so select it by its text instead ({"contains":"AlphaBeta"}), or accept the revisions first (`jubarte accept FILE -o OUT.docx`) and edit OUT.docx"#;
    assert_eq!(
        refusal(
            &source,
            ACCEPT,
            r#"{"kind":"delete_paragraph","paragraph":"p0"}"#
        ),
        format!("p0 (as read) merges with p1 when the plan's revisions settle; {route}")
    );
    assert_eq!(
        refusal(
            &source,
            ACCEPT,
            r#"{"kind":"delete_paragraph","paragraph":"p1"}"#
        ),
        format!("p1 (as read) merges with p0 when the plan's revisions settle; {route}")
    );
}

/// The route the refusal gives reaches the settled paragraph.
#[test]
fn the_text_route_the_refusal_gives_reaches_the_merged_paragraph() {
    let source = merging();
    let out = apply_plan(
        &source,
        &plan(
            ACCEPT,
            r#"{"kind":"delete_paragraph","paragraph":{"contains":"Three four."}}"#,
        ),
    )
    .unwrap();
    for report in &out.report.operations {
        assert_eq!(report.status, "ok", "{report:?}");
    }
    assert_eq!(texts(&out.clean), ["Zero.", "Two.", "Five."]);
    assert_word_valid_package(&out.redline);
}

/// A flags-built plan has no text selector: its refusal gives only the
/// settle-first route. A plan that settles some revisions only cannot
/// name one command.
#[test]
fn the_route_fits_the_plan_and_its_settlement() {
    let source = merging();
    let options = PrepareOptions {
        syntax: PlanSyntax::Flags,
        ..PrepareOptions::default()
    };
    let flags = plan(ACCEPT, r#"{"kind":"delete_paragraph","paragraph":"p3"}"#);
    let e = prepare(&source, &flags, &options).unwrap_err();
    assert_eq!(e.code, "PARAGRAPH_RESOLVED", "{e}");
    assert_eq!(
        e.message,
        "p3 (as read) merges into p2 (after the plan's revisions settle) together with p4; an operation on a whole paragraph needs a paragraph with one source, so accept the revisions first (`jubarte accept FILE -o OUT.docx`) and edit OUT.docx"
    );
    let some = r#""resolve_revisions":{"accept":{"ids":["r1"]}},"existing_revisions":"keep""#;
    let message = refusal(
        &source,
        some,
        r#"{"kind":"delete_paragraph","paragraph":"p3"}"#,
    );
    assert!(
        message.ends_with(
            r#"select it by its text instead ({"contains":"Three four."}), or settle the plan's revisions first and edit the result"#
        ),
        "{message}"
    );
}

#[test]
fn gone_paragraph_keeps_its_message() {
    let source = merging();
    for settle in [ACCEPT, REJECT] {
        let op = r#"{"kind":"delete_paragraph","paragraph":"p1"}"#;
        if settle == REJECT {
            // Rejecting keeps p1: nothing to refuse.
            apply_plan(&source, &plan(settle, op)).unwrap();
            continue;
        }
        assert_eq!(
            refusal(&source, settle, op),
            "p1 is gone once the plan's revisions are settled: its text was removed"
        );
    }
}

/// `merging()` whose merged paragraph cites ids in its words: "See p3"
/// and " and p4." settle into p2 "See p3 and p4.".
fn citing() -> Vec<u8> {
    merging_with("See p3", " and p4.")
}

#[test]
fn no_refusal_repeats_an_id() {
    assert!(repeats_an_id("p0 merged into p0"));
    assert!(!repeats_an_id(
        r#"p3 merges into p2 together with p4; select it ({"contains":"See p3 and p4 \"})\""}), or p5"#
    ));
    for (source, count) in [(merging(), 6), (alpha_beta(), 3), (citing(), 6)] {
        for p in 0..count {
            for op in [
                format!(r#"{{"kind":"delete_paragraph","paragraph":"p{p}"}}"#),
                format!(r#"{{"kind":"rewrite","paragraph":"p{p}","text":"New."}}"#),
            ] {
                if let Err(e) = apply_plan(&source, &plan(ACCEPT, &op)) {
                    assert!(!repeats_an_id(&e.message), "{op}: {e}");
                }
            }
        }
    }
}

/// Rejecting p1's inserted mark merges it with p2, and rejecting p2's
/// inserted "Y" changes p2's text: settled under reject, p1 "Three four"
/// holds p2's words too.
fn changed_neighbour() -> Vec<u8> {
    docx(&[
        "<w:p><w:r><w:t>Zero.</w:t></w:r></w:p>".to_string(),
        format!(
            r#"<w:p><w:pPr><w:rPr><w:ins w:id="1" {REV}/></w:rPr></w:pPr><w:r><w:t>Three</w:t></w:r></w:p>"#
        ),
        format!(
            r#"<w:p><w:r><w:t xml:space="preserve"> four</w:t></w:r><w:ins w:id="2" {REV}><w:r><w:t>Y</w:t></w:r></w:ins></w:p>"#
        ),
        "<w:p><w:r><w:t>Five.</w:t></w:r></w:p>".to_string(),
    ]
    .concat())
}

/// A source paragraph whose own text the settle changes is still a source
/// of the merge that holds its words: an operation on its neighbour is
/// refused, not applied to both, and it is not called gone.
#[test]
fn a_neighbour_whose_text_the_settle_changes_is_still_a_source() {
    let source = changed_neighbour();
    let route = r#"an operation on a whole paragraph needs a paragraph with one source, so select it by its text instead ({"contains":"Three four"}), or reject the revisions first (`jubarte reject FILE -o OUT.docx`) and edit OUT.docx"#;
    for (op, id, with) in [
        (
            r#"{"kind":"delete_paragraph","paragraph":"p1"}"#,
            "p1",
            "p2",
        ),
        (
            r#"{"kind":"rewrite","paragraph":"p1","text":"Three."}"#,
            "p1",
            "p2",
        ),
        (
            r#"{"kind":"delete_paragraph","paragraph":"p2"}"#,
            "p2",
            "p1",
        ),
    ] {
        assert_eq!(
            refusal(&source, REJECT, op),
            format!("{id} (as read) merges with {with} when the plan's revisions settle; {route}"),
            "{op}"
        );
    }
}

/// p1 is deleted whole, its mark and its words; p2's mark is deleted and
/// p3 is empty. Settled under accept, p2's words stand alone: p1 left
/// nothing in that paragraph, so p2 is still its one source.
fn removed_neighbour() -> Vec<u8> {
    docx(&[
        "<w:p><w:r><w:t>Zero.</w:t></w:r></w:p>".to_string(),
        format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="1" {REV}/></w:rPr></w:pPr><w:del w:id="2" {REV}><w:r><w:delText>One gone.</w:delText></w:r></w:del></w:p>"#
        ),
        format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="3" {REV}/></w:rPr></w:pPr><w:r><w:t>Three</w:t></w:r></w:p>"#
        ),
        "<w:p/>".to_string(),
        "<w:p><w:r><w:t>Five.</w:t></w:r></w:p>".to_string(),
    ]
    .concat())
}

#[test]
fn a_neighbour_the_settle_removes_whole_is_no_source() {
    let source = removed_neighbour();
    let out = apply_plan(
        &source,
        &plan(ACCEPT, r#"{"kind":"delete_paragraph","paragraph":"p2"}"#),
    )
    .unwrap();
    for report in &out.report.operations {
        assert_eq!(report.status, "ok", "{report:?}");
    }
    assert_eq!(texts(&out.clean), ["Zero.", "Five."]);
    assert_word_valid_package(&out.redline);
}

/// The selector a refusal quotes, as the plan takes it.
fn quoted_selector(message: &str) -> serde_json::Value {
    let at = message.find("({").unwrap_or_else(|| panic!("{message}"));
    serde_json::Deserializer::from_str(&message[at + 1..])
        .into_iter::<serde_json::Value>()
        .next()
        .unwrap()
        .unwrap()
}

/// The selector the refusal quotes reaches the merged paragraph when
/// pasted into the plan, whatever characters its text holds and however
/// long it is.
#[test]
fn the_quoted_selector_round_trips() {
    for (three, four) in [
        (
            r#"Say "no" back\slash"#,
            r#"</w:t><w:tab/><w:t xml:space="preserve"> “fine” for €5 &amp; more."#,
        ),
        (
            "A clause long enough that the selector is cut before",
            " sixty characters of its text, where the cut is unique.",
        ),
    ] {
        let source = merging_with(three, four);
        let message = refusal(
            &source,
            ACCEPT,
            r#"{"kind":"delete_paragraph","paragraph":"p3"}"#,
        );
        let selector = quoted_selector(&message);
        let quoted = selector["contains"].as_str().unwrap().chars().count();
        if three.len() + four.len() > 60 {
            assert!(quoted <= 60, "{selector}");
        }
        let op = serde_json::json!({"kind": "delete_paragraph", "paragraph": selector});
        let out = apply_plan(&source, &plan(ACCEPT, &op.to_string()))
            .unwrap_or_else(|e| panic!("{selector}: {e}"));
        assert_eq!(texts(&out.clean), ["Zero.", "Two.", "Five."], "{selector}");
        assert_word_valid_package(&out.redline);
    }
}

#[test]
fn anchor_candidates_retry_the_source_id_after_settlement() {
    let source = merging();
    for (anchor, expected) in [("Two.", "p2"), ("Three four.", "p3"), ("Five.", "p5")] {
        let op = |at: &str| {
            format!(
                r#"{{"kind":"replace","paragraph":"{at}","find":"{anchor}","replacement":"Replaced."}}"#
            )
        };
        let error = apply_plan(&source, &plan(ACCEPT, &op("p0"))).unwrap_err();
        assert_eq!(error.code, "ANCHOR_NOT_FOUND");
        let candidate = &error.outcomes[0].candidates[0];
        assert_eq!(
            candidate.at, expected,
            "candidate must name a retryable source paragraph"
        );
        let retried = apply_plan(&source, &plan(ACCEPT, &op(&candidate.at))).unwrap();
        assert!(texts(&retried.clean).iter().any(|t| t == "Replaced."));
    }
}
