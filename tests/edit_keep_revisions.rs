// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `existing_revisions: "keep"`: the other party's tracked changes stay
//! theirs, the plan's become new revisions, and the four invariants hold.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeFilter, ChangeKind, accept_changes, list_changes, reject_changes};
use jubarte::edit::{EditPlan, apply_plan};
use jubarte::inspect::paragraphs;

const OTHER: &str = r#"<w:p><w:r><w:t xml:space="preserve">Payment within </w:t></w:r><w:ins w:id="7" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins><w:del w:id="8" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:delText>30</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> days.</w:t></w:r></w:p>"#;

fn source() -> Vec<u8> {
    docx(&(OTHER.to_string() + &para("Governing law: Delaware.")))
}

fn texts(docx: &[u8]) -> Vec<String> {
    paragraphs(docx)
        .unwrap()
        .into_iter()
        .map(|p| p.text)
        .collect()
}

fn mine() -> ChangeFilter {
    ChangeFilter {
        authors: Some(vec!["Me".to_string()]),
        ..ChangeFilter::default()
    }
}

#[test]
fn keep_leaves_their_changes_and_adds_mine() {
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","date":"2026-10-02T00:00:00Z","existing_revisions":"keep","operations":[
        {"kind":"replace","paragraph":"body:p:1","find":"Delaware","replacement":"New York"}]}"#).unwrap();
    let out = apply_plan(&source(), &plan).unwrap();
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    assert_eq!(out.report.base_sha256, out.report.source_sha256);

    // clean: my edit applied, theirs still tracked (visible text shows the accepted view of theirs).
    assert_eq!(
        texts(&out.clean),
        ["Payment within 45 days.", "Governing law: New York."]
    );
    assert_eq!(
        list_changes(&out.clean).unwrap().len(),
        2,
        "their two changes are still tracked in the clean copy"
    );

    // redline: theirs by id and author, mine new.
    let changes = list_changes(&out.redline).unwrap();
    let theirs: Vec<_> = changes
        .iter()
        .filter(|c| c.author.as_deref() == Some("Other"))
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(theirs, ["body:rev:7", "body:rev:8"]);
    let mine_kinds: Vec<_> = changes
        .iter()
        .filter(|c| c.author.as_deref() == Some("Me"))
        .map(|c| (c.kind, c.text.as_str()))
        .collect();
    assert_eq!(
        mine_kinds,
        [
            (ChangeKind::Deletion, "Delaware"),
            (ChangeKind::Insertion, "New York")
        ]
    );
    assert!(
        changes
            .iter()
            .filter(|c| c.author.as_deref() == Some("Me"))
            .all(|c| c.id != "body:rev:7" && c.id != "body:rev:8")
    );

    // The invariants.
    assert_eq!(
        texts(&accept_changes(&out.redline, &mine()).unwrap()),
        texts(&out.clean)
    );
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        texts(&source())
    );
    // Their markup is byte-identical in the redline.
    let red = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(red.contains(r#"<w:ins w:id="7" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>45</w:t></w:r></w:ins>"#), "{red}");
}

#[test]
fn an_edit_inside_their_insertion_is_still_refused() {
    let plan = EditPlan::from_json(
        r#"{"schema_version":1,"author":"Me","existing_revisions":"keep","operations":[
        {"kind":"replace","paragraph":"body:p:0","find":"45","replacement":"60"}]}"#,
    )
    .unwrap();
    let e = apply_plan(&source(), &plan).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_STRUCTURE");
    assert!(e.message.contains("revision"));
}

#[test]
fn paragraph_operations_emit_marks_under_keep() {
    // The engine refuses an insertion anchored on a paragraph the same plan
    // deletes, under every policy, so the new paragraph follows body:p:0.
    // body:p:1 is the last paragraph: its content joins the paragraph before
    // it, the inserted one, whose mark is then both inserted and deleted.
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","existing_revisions":"keep","operations":[
        {"kind":"insert_paragraph","paragraph":"body:p:0","position":"after","runs":[{"text":"Venue: New York County."}]},
        {"kind":"delete_paragraph","paragraph":"body:p:1"},
        {"kind":"rewrite","paragraph":"body:p:0","text":"Payment within 45 days of invoice."}]}"#).unwrap();
    let out = apply_plan(&source(), &plan).unwrap();
    assert_word_valid_package(&out.redline);
    let changes = list_changes(&out.redline).unwrap();
    assert!(changes.iter().any(|c| c.author.as_deref() == Some("Me")
        && c.kind == ChangeKind::Insertion
        && c.target == "paragraph_mark"));
    assert!(changes.iter().any(|c| c.author.as_deref() == Some("Me")
        && c.kind == ChangeKind::Deletion
        && c.target == "paragraph_mark"));
    assert!(changes.iter().any(|c| c.author.as_deref() == Some("Me")
        && c.kind == ChangeKind::Insertion
        && c.text == " of invoice"));
    assert_eq!(
        texts(&accept_changes(&out.redline, &mine()).unwrap()),
        texts(&out.clean)
    );
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        texts(&source())
    );
}

#[test]
fn keep_without_existing_revisions_equals_the_compare_redline_in_text() {
    // On a source with no revisions, keep and the default produce the same
    // visible result, so an agent can always send keep.
    let plain = docx(&para("Fee is 10."));
    let keep = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","existing_revisions":"keep","operations":[{"kind":"replace","paragraph":"body:p:0","find":"10","replacement":"12"}]}"#).unwrap();
    let default = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","operations":[{"kind":"replace","paragraph":"body:p:0","find":"10","replacement":"12"}]}"#).unwrap();
    let a = apply_plan(&plain, &keep).unwrap();
    let b = apply_plan(&plain, &default).unwrap();
    assert_eq!(texts(&a.clean), texts(&b.clean));
    assert_eq!(
        texts(&accept_changes(&a.redline, &mine()).unwrap()),
        texts(&accept_changes(&b.redline, &mine()).unwrap())
    );
}

// Beyond the plan's four: every operation kind under keep, the keep-only
// refusals, the report, and the other stories.

fn keep(operations: &str) -> EditPlan {
    EditPlan::from_json(&format!(
        r#"{{"schema_version":1,"author":"Me","date":"2026-10-02T00:00:00Z","existing_revisions":"keep","operations":{operations}}}"#
    ))
    .unwrap()
}

/// Valid redline; accepting mine gives clean, rejecting mine gives the
/// source; their changes keep their ids. Returns the redline's changes.
fn invariants(source: &[u8], out: &jubarte::edit::EditResult) -> Vec<jubarte::changes::Change> {
    assert_word_valid_package(&out.clean);
    assert_word_valid_package(&out.redline);
    assert_eq!(
        texts(&accept_changes(&out.redline, &mine()).unwrap()),
        texts(&out.clean)
    );
    assert_eq!(
        texts(&reject_changes(&out.redline, &mine()).unwrap()),
        texts(source)
    );
    let theirs = |changes: &[jubarte::changes::Change]| -> Vec<String> {
        changes
            .iter()
            .filter(|c| c.author.as_deref() != Some("Me"))
            .map(|c| c.id.clone())
            .collect()
    };
    let changes = list_changes(&out.redline).unwrap();
    assert_eq!(theirs(&changes), theirs(&list_changes(source).unwrap()));
    changes
}

#[test]
fn text_edits_carry_comments_and_formatting() {
    let source = source();
    let out = apply_plan(&source, &keep(r#"[
        {"kind":"insert","paragraph":"body:p:1","position":"start","text":"(a) ","format":{"bold":true}},
        {"kind":"replace","paragraph":"body:p:1","find":"Delaware","replacement":"New York","comment":"Our home court."},
        {"kind":"comment","paragraph":"body:p:1","find":"Governing","text":"Check."},
        {"kind":"delete","paragraph":"body:p:0","find":" days"}]"#)).unwrap();
    assert_eq!(
        texts(&out.clean),
        ["Payment within 45.", "(a) Governing law: New York."]
    );
    let changes = invariants(&source, &out);
    let mine: Vec<_> = changes
        .iter()
        .filter(|c| c.author.as_deref() == Some("Me"))
        .map(|c| (c.kind, c.text.as_str()))
        .collect();
    assert_eq!(
        mine,
        [
            (ChangeKind::Deletion, " days"),
            (ChangeKind::Insertion, "(a) "),
            (ChangeKind::Deletion, "Delaware"),
            (ChangeKind::Insertion, "New York")
        ]
    );
    let r = &out.report.revisions;
    assert_eq!(
        (r.inserted, r.deleted, r.total),
        (2, 2, 4),
        "the report counts my changes only"
    );
    let red = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(red.contains("<w:b />"), "the inserted text is bold: {red}");
    let comments = part_string(&out.redline, "word/comments.xml").unwrap();
    assert!(
        comments.contains("Our home court.") && comments.contains("Check."),
        "{comments}"
    );
    assert_eq!(red.matches("<w:commentReference").count(), 2, "{red}");
    // New ids continue after the highest id already in the document.
    for c in changes.iter().filter(|c| c.author.as_deref() == Some("Me")) {
        let id: u64 = c.id.rsplit(':').next().unwrap().parse().unwrap();
        assert!(id > 8, "{}", c.id);
    }
    let json = serde_json::to_value(&out.report).unwrap();
    assert_eq!(json["existing_revisions"], "keep");
}

#[test]
fn merge_format_and_delete_mark_paragraphs() {
    let source = docx(
        &(OTHER.to_string()
            + &para("First.")
            + &para("Second.")
            + &para("Third.")
            + &para("Fourth.")),
    );
    let out = apply_plan(
        &source,
        &keep(
            r#"[
        {"kind":"merge_paragraphs","paragraph":"body:p:1","separator":" "},
        {"kind":"delete_paragraph","paragraph":"body:p:3","comment":"Redundant."},
        {"kind":"format_paragraph","paragraph":"body:p:4","alignment":"center"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(
        texts(&out.clean),
        ["Payment within 45 days.", "First. Second.", "Fourth."]
    );
    let changes = invariants(&source, &out);
    let mine = |kind: ChangeKind, target: &str| {
        changes
            .iter()
            .filter(|c| c.author.as_deref() == Some("Me") && c.kind == kind && c.target == target)
            .count()
    };
    assert_eq!(
        mine(ChangeKind::Deletion, "paragraph_mark"),
        2,
        "the merged head's mark and the deleted paragraph's own mark"
    );
    assert_eq!(mine(ChangeKind::Insertion, "text"), 1, "the separator");
    assert_eq!(
        mine(ChangeKind::Formatting, "properties"),
        1,
        "{changes:#?}"
    );
    let red = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(
        red.contains(r#"<w:jc w:val="center" />"#) && red.contains("<w:pPrChange"),
        "{red}"
    );
    let comments = part_string(&out.redline, "word/comments.xml").unwrap();
    assert!(comments.contains("Redundant."), "{comments}");
    assert!(
        red.contains("<w:commentRangeStart"),
        "the deletion comment is anchored: {red}"
    );
}

#[test]
fn edits_beside_a_tab_keep_the_tab() {
    let source =
        docx(&(OTHER.to_string() + r#"<w:p><w:r><w:t>Fee</w:t><w:tab/><w:t>10</w:t></w:r></w:p>"#));
    let out = apply_plan(
        &source,
        &keep(
            r#"[
        {"kind":"replace","paragraph":"body:p:1","find":"Fee","replacement":"Price"},
        {"kind":"replace","paragraph":"body:p:1","find":"10","replacement":"12"}]"#,
        ),
    )
    .unwrap();
    assert_eq!(texts(&out.clean)[1], "Price\t12");
    invariants(&source, &out);
}

#[test]
fn keep_refuses_paragraph_changes_on_tracked_paragraphs() {
    let tracked_mark = r#"<w:p><w:pPr><w:rPr><w:del w:id="3" w:author="Other" w:date="2026-09-01T00:00:00Z"/></w:rPr></w:pPr><w:r><w:t>Marked.</w:t></w:r></w:p>"#;
    let reformatted = r#"<w:p><w:pPr><w:jc w:val="right"/><w:pPrChange w:id="4" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>Moved right.</w:t></w:r></w:p>"#;
    let source = docx(&(OTHER.to_string() + tracked_mark + reformatted + &para("Last.")));
    for operations in [
        r#"[{"kind":"delete_paragraph","paragraph":"body:p:0"}]"#,
        r#"[{"kind":"merge_paragraphs","paragraph":"body:p:1"}]"#,
        r#"[{"kind":"format_paragraph","paragraph":"body:p:2","alignment":"left"}]"#,
    ] {
        let e = apply_plan(&source, &keep(operations)).unwrap_err();
        assert_eq!(e.code, "UNSUPPORTED_STRUCTURE", "{operations}: {e:?}");
        assert!(e.message.contains("keep"), "{}", e.message);
        let e = jubarte::edit::preview_plan(&source, &keep(operations)).unwrap_err();
        assert_eq!(
            e.code, "UNSUPPORTED_STRUCTURE",
            "preview refuses too: {operations}"
        );
    }
    // The same operations on untracked paragraphs go through.
    let out = apply_plan(
        &source,
        &keep(r#"[{"kind":"format_paragraph","paragraph":"body:p:3","alignment":"left"}]"#),
    )
    .unwrap();
    invariants(&source, &out);
}

#[test]
fn keep_edits_a_header_beside_their_revision() {
    use common::docx::{Part, R_NS, W_NS, docx_with_sect};
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:hdr xmlns:w="{W_NS}"><w:p><w:r><w:t xml:space="preserve">Draft of </w:t></w:r><w:ins w:id="40" w:author="Other" w:date="2026-09-01T00:00:00Z"><w:r><w:t>October</w:t></w:r></w:ins></w:p></w:hdr>"#
    );
    let rel = format!("{R_NS}/header");
    let source = docx_with_sect(
        &para("Body."),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: &rel,
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    let out = apply_plan(&source, &keep(r#"[{"kind":"replace","paragraph":"header1:p:0","find":"Draft","replacement":"Final"}]"#)).unwrap();
    assert_word_valid_package(&out.redline);
    let changes = list_changes(&out.redline).unwrap();
    assert!(
        changes.iter().any(|c| c.id == "header1:rev:40"),
        "{changes:#?}"
    );
    let my_ids: Vec<_> = changes
        .iter()
        .filter(|c| c.author.as_deref() == Some("Me"))
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(
        my_ids,
        ["header1:rev:41", "header1:rev:42"],
        "ids follow the highest in any part"
    );
    let accepted = accept_changes(&out.redline, &mine()).unwrap();
    assert!(
        part_string(&accepted, "word/header1.xml")
            .unwrap()
            .contains("Final")
    );
    let rejected = reject_changes(&out.redline, &mine()).unwrap();
    let header_after = part_string(&rejected, "word/header1.xml").unwrap();
    assert!(
        header_after.contains("Draft") && header_after.contains(r#"w:id="40""#),
        "{header_after}"
    );
}

#[test]
fn the_patch_shows_only_my_changes() {
    use jubarte::markdown::{
        Attribution, DEFAULT_COLUMNS, PatchOptions, patch_own_changes, patch_redline,
    };
    let plan = EditPlan::from_json(r#"{"schema_version":1,"author":"Me","date":"2026-10-02T00:00:00Z","existing_revisions":"keep","operations":[
        {"kind":"replace","paragraph":"body:p:1","find":"Delaware","replacement":"New York"}]}"#).unwrap();
    let out = apply_plan(&source(), &plan).unwrap();
    let options = PatchOptions {
        old_name: "a.docx".into(),
        new_name: "a.docx".into(),
        owner: Attribution {
            author: "Me".into(),
            date: "2026-10-02T00:00:00Z".into(),
        },
    };
    let all = patch_redline(&out.redline, &options)
        .unwrap()
        .render(DEFAULT_COLUMNS);
    assert!(
        all.contains("30"),
        "the unfiltered patch shows their change too: {all}"
    );
    let own = patch_own_changes(&out.redline, &options)
        .unwrap()
        .render(DEFAULT_COLUMNS);
    assert!(
        own.contains("New York") && own.contains("Delaware"),
        "{own}"
    );
    assert!(
        !own.contains("30") && !own.contains("Other"),
        "their change is not in my patch: {own}"
    );
}

#[test]
fn the_cli_writes_my_patch_under_keep() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("deal.docx");
    std::fs::write(&file, source()).unwrap();
    let plan = dir.path().join("plan.json");
    std::fs::write(
        &plan,
        r#"{"schema_version":1,"author":"Me","date":"2026-10-02T00:00:00Z","existing_revisions":"keep","operations":[
            {"kind":"replace","paragraph":"body:p:1","find":"Delaware","replacement":"New York"}]}"#,
    )
    .unwrap();
    let out_dir = dir.path().join("review");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args([
            "edit",
            file.to_str().unwrap(),
            "--plan",
            plan.to_str().unwrap(),
            "--out-dir",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let patch = std::fs::read_to_string(out_dir.join("patch.diff")).unwrap();
    assert!(
        patch.contains("New York") && !patch.contains("30"),
        "{patch}"
    );
    let redline = std::fs::read(out_dir.join("redline.docx")).unwrap();
    assert_eq!(
        list_changes(&redline).unwrap().len(),
        4,
        "theirs and mine are both tracked"
    );
}

#[test]
fn whole_replacements_and_insertions_beside_their_changes() {
    let source = source();
    let out = apply_plan(&source, &keep(r#"[
        {"kind":"replace","paragraph":"body:p:1","find":"law: Delaware","replacement":"law: New York","whole":true},
        {"kind":"insert","paragraph":"body:p:0","before":" days","text":" business"}]"#)).unwrap();
    assert_eq!(
        texts(&out.clean),
        [
            "Payment within 45 business days.",
            "Governing law: New York."
        ]
    );
    let changes = invariants(&source, &out);
    let mine: Vec<_> = changes
        .iter()
        .filter(|c| c.author.as_deref() == Some("Me"))
        .map(|c| (c.kind, c.text.as_str()))
        .collect();
    assert_eq!(
        mine,
        [
            (ChangeKind::Insertion, " business"),
            (ChangeKind::Deletion, "law: Delaware"),
            (ChangeKind::Insertion, "law: New York")
        ],
        "whole is one deletion then one insertion"
    );
}

#[test]
fn deleting_the_paragraph_before_a_table_keeps_the_invariants() {
    let table = r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:r><w:t>Cell.</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
    let source = docx(&(OTHER.to_string() + &para("Before the table.") + table + &para("After.")));
    let out = apply_plan(
        &source,
        &keep(r#"[{"kind":"delete_paragraph","paragraph":"body:p:1"}]"#),
    )
    .unwrap();
    invariants(&source, &out);
}

#[test]
fn thread_operations_and_spans_ride_the_redline_under_keep() {
    use jubarte::comments::list_comments;
    let source = source();
    // Two comments by Ann on the tracked document, written with keep.
    let first = apply_plan(
        &source,
        &EditPlan::from_json(
            r#"{"schema_version":1,"author":"Ann","existing_revisions":"keep","operations":[
        {"kind":"comment","paragraph":"body:p:1","find":"Governing","text":"Which law?"},
        {"kind":"comment","paragraph":"body:p:1","find":"Delaware","text":"Drop this one"}]}"#,
        )
        .unwrap(),
    )
    .unwrap();
    let ids: Vec<u32> = list_comments(&first.clean)
        .unwrap()
        .iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(ids.len(), 2);
    let plan = keep(&format!(
        r#"[{{"kind":"reply_comment","comment_id":{a},"text":"New York"}},
            {{"kind":"resolve_comment","comment_id":{a}}},
            {{"kind":"delete_comment","comment_id":{b}}},
            {{"kind":"comment","paragraph":"body:p:0","through":"body:p:1","text":"Both"}},
            {{"kind":"replace","paragraph":"body:p:1","find":"Delaware","replacement":"New York"}}]"#,
        a = ids[0],
        b = ids[1]
    ));
    let out = apply_plan(&first.clean, &plan).unwrap();
    invariants(&first.clean, &out);
    for doc in [&out.clean, &out.redline] {
        let comments = list_comments(doc).unwrap();
        let texts: Vec<_> = comments
            .iter()
            .map(|c| (c.text.as_str(), c.parent.is_some(), c.done))
            .collect();
        assert_eq!(texts.len(), 3, "{texts:?}");
        assert!(texts.contains(&("Which law?", false, true)), "{texts:?}");
        assert!(texts.contains(&("New York", true, true)), "{texts:?}");
        assert!(texts.contains(&("Both", false, false)), "{texts:?}");
        let both = comments.iter().find(|c| c.text == "Both").unwrap();
        assert!(
            both.anchor_text.contains("Payment") && both.anchor_text.contains("Governing"),
            "{:?}",
            both.anchor_text
        );
    }
    let red = part_string(&out.redline, "word/document.xml").unwrap();
    assert!(
        !red.contains(&format!(r#"w:id="{}" /><w:r><w:commentReference"#, ids[1]))
            && !red.contains(&format!(r#"<w:commentReference w:id="{}""#, ids[1])),
        "the deleted comment's markers are gone: {red}"
    );
}
