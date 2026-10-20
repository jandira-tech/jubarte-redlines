// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Plan 11 (#1261): one notion of a change. One author's touching marks of
//! one kind are one logical change, listed once with a composite id
//! (`r7+8+9+10`) and its members. A deletion run followed by the same
//! author's insertion run is one replacement. Original text, unchanged or
//! deleted by someone else, separates them; text another author inserted
//! inside them does not. Settling a logical change takes the whole of each
//! run it names.

mod common;

use common::docx::docx;
use common::validity::assert_word_valid_package;
use jubarte::changes::{
    ChangeError, ChangeFilter, ChangeKind, accept_changes, accept_elements, list_changes,
    list_logical_changes, reject_changes, reject_elements,
};
use std::process::Command;

const BOB: &str = "Bob";
const DATE: &str = "2026-10-01T10:00:00Z";

fn ins(id: u32, text: &str, author: &str, date: &str) -> String {
    format!(
        r#"<w:ins w:id="{id}" w:author="{author}" w:date="{date}"><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:ins>"#
    )
}

fn del(id: u32, text: &str, author: &str, date: &str) -> String {
    format!(
        r#"<w:del w:id="{id}" w:author="{author}" w:date="{date}"><w:r><w:delText xml:space="preserve">{text}</w:delText></w:r></w:del>"#
    )
}

fn kept(text: &str) -> String {
    format!(r#"<w:r><w:t xml:space="preserve">{text}</w:t></w:r>"#)
}

fn p(content: &[String]) -> String {
    format!("<w:p>{}</w:p>", content.concat())
}

/// A paragraph whose own mark is inserted by Bob as change `mark`.
fn p_with_inserted_mark(mark: u32, content: &[String]) -> String {
    format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="{mark}" w:author="{BOB}" w:date="{DATE}"/></w:rPr></w:pPr>{}</w:p>"#,
        content.concat()
    )
}

/// One insertion that Word wrote as four marks: the plan's r7+8+9+10.
fn split_insertion(dates: [&str; 4]) -> Vec<u8> {
    docx(&p(&[
        kept("The licence "),
        ins(7, "shall re", BOB, dates[0]),
        ins(8, "main in ", BOB, dates[1]),
        ins(9, "effect until ", BOB, dates[2]),
        ins(10, "terminated.", BOB, dates[3]),
        kept(" End."),
    ]))
}

fn ids(doc: &[u8]) -> Vec<String> {
    list_logical_changes(doc)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect()
}

fn text_of(doc: &[u8]) -> String {
    let xml = common::docx::part_string(doc, "word/document.xml").unwrap();
    let mut out = String::new();
    let mut rest = xml.as_str();
    while let Some(at) = rest.find("<w:t") {
        rest = &rest[at..];
        let open_end = rest.find('>').unwrap();
        if rest[..open_end].ends_with('/') {
            rest = &rest[open_end + 1..];
            continue;
        }
        let close = rest.find("</w:t>").unwrap();
        out.push_str(&rest[open_end + 1..close]);
        rest = &rest[close..];
    }
    out
}

#[test]
fn changes_lists_one_logical_change_for_adjacent_same_author_runs() {
    let doc = split_insertion([DATE; 4]);
    let listed = list_logical_changes(&doc).unwrap();
    assert_eq!(listed.len(), 1, "{listed:#?}");
    let change = &listed[0];
    assert_eq!(change.id, "r7+8+9+10");
    assert_eq!(change.r#ref, "r7+8+9+10");
    assert_eq!(change.kind, ChangeKind::Insertion);
    assert_eq!(change.target, "text");
    assert_eq!(change.author.as_deref(), Some(BOB));
    assert_eq!(change.paragraph.as_deref(), Some("p0"));
    assert_eq!(change.text, "shall remain in effect until terminated.");
    let members: Vec<(&str, ChangeKind, &str)> = change
        .members
        .iter()
        .map(|m| (m.id.as_str(), m.kind, m.text.as_str()))
        .collect();
    assert_eq!(
        members,
        [
            ("r7", ChangeKind::Insertion, "shall re"),
            ("r8", ChangeKind::Insertion, "main in "),
            ("r9", ChangeKind::Insertion, "effect until "),
            ("r10", ChangeKind::Insertion, "terminated."),
        ]
    );
    let json = serde_json::to_value(change).unwrap();
    assert_eq!(json["members"].as_array().unwrap().len(), 4, "{json}");
    assert_eq!(json["members"][0]["id"], "r7");
    // The per-element listing stays what it was: every mark on its own.
    assert_eq!(list_changes(&doc).unwrap().len(), 4);
}

#[test]
fn a_change_of_one_mark_lists_no_members() {
    let doc = docx(&p(&[kept("Pay "), ins(5, "now", BOB, DATE)]));
    let listed = list_logical_changes(&doc).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, "r5");
    assert!(listed[0].members.is_empty());
    let json = serde_json::to_value(&listed[0]).unwrap();
    assert!(json.get("members").is_none(), "{json}");
}

/// Word stamps each typing session: the real r7..r10 carry four minutes.
/// The group takes the earliest; each member keeps its own.
#[test]
fn dates_do_not_split_a_group() {
    let doc = split_insertion([
        "2026-06-03T14:26:00Z",
        "2026-06-03T14:27:00Z",
        "2026-06-03T14:28:00Z",
        "2026-06-03T14:27:00Z",
    ]);
    let listed = list_logical_changes(&doc).unwrap();
    assert_eq!(listed.len(), 1, "{listed:#?}");
    assert_eq!(listed[0].date.as_deref(), Some("2026-06-03T14:26:00Z"));
    let dates: Vec<_> = listed[0]
        .members
        .iter()
        .map(|m| m.date.as_deref().unwrap())
        .collect();
    assert_eq!(
        dates,
        [
            "2026-06-03T14:26:00Z",
            "2026-06-03T14:27:00Z",
            "2026-06-03T14:28:00Z",
            "2026-06-03T14:27:00Z"
        ]
    );
}

#[test]
fn unchanged_text_splits_a_group() {
    let doc = docx(&p(&[
        ins(1, "alpha ", BOB, DATE),
        kept("kept "),
        ins(2, "beta.", BOB, DATE),
    ]));
    assert_eq!(ids(&doc), ["r1", "r2"]);
}

#[test]
fn another_author_splits_a_group() {
    let doc = docx(&p(&[
        ins(1, "alpha ", BOB, DATE),
        ins(2, "beta.", "Carol", DATE),
    ]));
    assert_eq!(ids(&doc), ["r1", "r2"]);
}

#[test]
fn a_replacement_is_one_change_with_members() {
    let doc = docx(&p(&[
        kept("Pay "),
        del(3, "thirty", BOB, DATE),
        ins(4, "forty-five", BOB, DATE),
        kept(" days."),
    ]));
    let listed = list_logical_changes(&doc).unwrap();
    assert_eq!(listed.len(), 1, "{listed:#?}");
    assert_eq!(listed[0].id, "r3+4");
    assert_eq!(listed[0].kind, ChangeKind::Replacement);
    assert_eq!(listed[0].text, "thirty → forty-five");
    let members: Vec<(&str, ChangeKind)> = listed[0]
        .members
        .iter()
        .map(|m| (m.id.as_str(), m.kind))
        .collect();
    assert_eq!(
        members,
        [("r3", ChangeKind::Deletion), ("r4", ChangeKind::Insertion)]
    );
    let json = serde_json::to_value(&listed[0]).unwrap();
    assert_eq!(json["kind"], "replacement");
}

/// The view never pairs an insertion with the deletion after it.
#[test]
fn an_insertion_then_a_deletion_stays_two() {
    let doc = docx(&p(&[ins(1, "new ", BOB, DATE), del(2, "old", BOB, DATE)]));
    assert_eq!(ids(&doc), ["r1", "r2"]);
}

/// A change takes its own paragraph's inserted mark and ends there: the
/// next paragraph's insertion is a change of its own. Word writes a mark
/// for every paragraph, so a change running on would make one change of a
/// whole deleted clause list (decided 20 October 2026).
#[test]
fn a_change_takes_its_own_paragraph_mark_and_ends_there() {
    let doc = docx(&format!(
        "{}{}",
        p_with_inserted_mark(6, &[kept("First "), ins(5, "new text", BOB, DATE)]),
        p(&[ins(11, "continues here", BOB, DATE), kept(" and old.")]),
    ));
    let listed = list_logical_changes(&doc).unwrap();
    let ids: Vec<&str> = listed.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["r5+6", "r11"], "{listed:#?}");
    let targets: Vec<&str> = listed[0].members.iter().map(|m| m.target).collect();
    assert_eq!(targets, ["text", "paragraph_mark"]);
}

#[test]
fn each_deleted_paragraph_is_a_change_of_its_own() {
    let deleted = |text: u32, mark: u32, words: &str| {
        format!(
            r#"<w:p><w:pPr><w:rPr><w:del w:id="{mark}" w:author="{BOB}" w:date="{DATE}"/></w:rPr></w:pPr>{}</w:p>"#,
            del(text, words, BOB, DATE)
        )
    };
    let doc = docx(
        &[
            deleted(1, 2, "One."),
            deleted(3, 4, "Two."),
            deleted(5, 6, "Three."),
            p(&[kept("Kept.")]),
        ]
        .concat(),
    );
    assert_eq!(ids(&doc), ["r1+2", "r3+4", "r5+6"]);
}

/// A change is listed where its first mark reads, not where the XML puts
/// its paragraph mark (before the paragraph's text).
#[test]
fn a_change_is_listed_where_it_reads() {
    let doc = docx(&p_with_inserted_mark(
        9,
        &[
            kept("A "),
            ins(3, "x ", "Carol", DATE),
            ins(4, "y", BOB, DATE),
        ],
    ));
    assert_eq!(ids(&doc), ["r3", "r4+9"]);
}

/// A `w:id` the story repeats names two elements: they stay apart, as no
/// composite (`r5+5`) could say which.
#[test]
fn a_repeated_id_stays_apart() {
    let doc = docx(&p(&[
        kept("A "),
        ins(5, "one ", BOB, DATE),
        ins(5, "two", BOB, DATE),
    ]));
    assert_eq!(ids(&doc), ["r5", "r5"]);
}

/// Bob deleted two pieces of Alice's insertion, her text between them:
/// that text was there when he deleted, so they are two changes.
#[test]
fn deletions_made_inside_an_insertion_around_its_text_are_two_changes() {
    let doc = docx(&p(&[
        kept("Start "),
        format!(
            r#"<w:ins w:id="3" w:author="Alice" w:date="{DATE}">{}{}{}{}</w:ins>"#,
            kept("one "),
            del(4, "two ", BOB, DATE),
            kept("three "),
            del(5, "four", BOB, DATE)
        ),
    ]));
    let listed = list_logical_changes(&doc).unwrap();
    let rows: Vec<(&str, Option<&str>)> = listed
        .iter()
        .map(|c| (c.id.as_str(), c.inside.as_deref()))
        .collect();
    assert_eq!(rows, [("r3", None), ("r4", Some("r3")), ("r5", Some("r3"))]);
}

/// The element API settles one mark of a change, which the change API
/// refuses: what is left groups again.
#[test]
fn the_element_api_settles_one_mark_of_a_change() {
    let doc = split_insertion([DATE; 4]);
    let r8 = ChangeFilter::ids(["r8"]);
    assert!(matches!(
        accept_changes(&doc, &r8),
        Err(ChangeError::PartOfChange { .. })
    ));
    let accepted = accept_elements(&doc, &r8).unwrap();
    assert_word_valid_package(&accepted);
    assert_eq!(ids(&accepted), ["r7", "r9+10"]);
    let rejected = reject_elements(&doc, &r8).unwrap();
    assert_word_valid_package(&rejected);
    assert_eq!(ids(&rejected), ["r7+9+10"]);
}

#[test]
fn an_unchanged_mark_splits_a_group() {
    let doc = docx(&format!(
        "{}{}",
        p(&[kept("First "), ins(5, "new text", BOB, DATE)]),
        p(&[ins(11, "continues here", BOB, DATE), kept(" and old.")]),
    ));
    assert_eq!(ids(&doc), ["r5", "r11"]);
}

/// Plan 1 splits Bob's insertion around Alice's edit inside it: Bob's two
/// pieces stay one change, and Alice's deletion and insertion are hers.
#[test]
fn pieces_split_around_another_authors_edit_stay_one_change() {
    let alice = "2026-10-20T07:33:05Z";
    let doc = docx(&p(&[
        kept("Start "),
        format!(
            r#"<w:ins w:id="7" w:author="{BOB}" w:date="{DATE}"><w:r><w:t xml:space="preserve">alpha </w:t></w:r>{}</w:ins>"#,
            del(8, "beta", "Alice", alice)
        ),
        ins(9, "BETA", "Alice", alice),
        ins(10, " gamma.", BOB, DATE),
    ]));
    let listed = list_logical_changes(&doc).unwrap();
    let rows: Vec<(&str, ChangeKind, Option<&str>)> = listed
        .iter()
        .map(|c| (c.id.as_str(), c.kind, c.inside.as_deref()))
        .collect();
    assert_eq!(
        rows,
        [
            ("r7+10", ChangeKind::Insertion, None),
            ("r8+9", ChangeKind::Replacement, Some("r7+10")),
        ]
    );
}

/// Text Carol deleted was there before: Bob's insertions on either side
/// of it are two changes.
#[test]
fn another_authors_deletion_of_original_text_splits_a_group() {
    let doc = docx(&p(&[
        ins(1, "foo ", BOB, DATE),
        del(2, "bar ", "Carol", DATE),
        ins(3, "baz", BOB, DATE),
    ]));
    assert_eq!(ids(&doc), ["r1", "r2", "r3"]);
}

#[test]
fn r87_plus_r88_is_accepted() {
    let doc = docx(&p(&[
        kept("Keep "),
        ins(87, "main in ", BOB, DATE),
        ins(88, "effect.", BOB, DATE),
    ]));
    let out = accept_changes(&doc, &ChangeFilter::ids(["r87+r88"])).unwrap();
    assert_word_valid_package(&out);
    assert!(list_changes(&out).unwrap().is_empty());
    assert_eq!(text_of(&out), "Keep main in effect.");
}

#[test]
fn accept_of_a_composite_resolves_every_member() {
    let doc = split_insertion([DATE; 4]);
    let accepted = accept_changes(&doc, &ChangeFilter::ids(["r7+8+9+10"])).unwrap();
    assert_word_valid_package(&accepted);
    assert!(list_changes(&accepted).unwrap().is_empty());
    assert_eq!(
        text_of(&accepted),
        "The licence shall remain in effect until terminated. End."
    );
    let rejected = reject_changes(&doc, &ChangeFilter::ids(["r7+8+9+10"])).unwrap();
    assert!(list_changes(&rejected).unwrap().is_empty());
    assert_eq!(text_of(&rejected), "The licence  End.");
    // The members given one by one name the whole change too.
    let each = ChangeFilter::ids(["r7", "r8", "r9", "r10"]);
    assert!(
        list_changes(&reject_changes(&doc, &each).unwrap())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn refusal_hint_shows_the_composite_in_the_file() {
    let doc = split_insertion([DATE; 4]);
    for given in ["r7", "r7+8", "r10"] {
        let err = reject_changes(&doc, &ChangeFilter::ids([given])).unwrap_err();
        assert!(
            matches!(&err, ChangeError::PartOfChange { whole, .. } if whole == "r7+8+9+10"),
            "{given}: {err:?}"
        );
        let shown = err.to_string();
        assert!(
            shown.contains("is part of r7+8+9+10") && shown.contains("name the whole change"),
            "{given}: {shown}"
        );
    }
    let err = reject_changes(&doc, &ChangeFilter::ids(["r7+8"])).unwrap_err();
    assert_eq!(
        err.to_string(),
        "r7+8 is part of r7+8+9+10; name the whole change (r7+8+9+10)"
    );
}

/// A replacement's two runs are separate decisions: rejecting only the
/// deletion keeps both texts, as Word allows.
#[test]
fn a_replacement_side_may_be_resolved_alone() {
    let doc = docx(&p(&[
        kept("Pay "),
        del(3, "thirty", BOB, DATE),
        ins(4, "forty-five", BOB, DATE),
        kept(" days."),
    ]));
    let out = reject_changes(&doc, &ChangeFilter::ids(["r3"])).unwrap();
    assert_eq!(text_of(&out), "Pay thirtyforty-five days.");
    let left: Vec<String> = list_changes(&out)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect();
    assert_eq!(left, ["r4"]);
}

#[test]
fn half_a_replacement_run_is_refused() {
    let doc = docx(&p(&[
        kept("Keep "),
        del(20, "one ", BOB, DATE),
        del(21, "two", BOB, DATE),
        ins(22, "three", BOB, DATE),
    ]));
    assert_eq!(ids(&doc), ["r20+21+22"]);
    let err = reject_changes(&doc, &ChangeFilter::ids(["r20"])).unwrap_err();
    assert_eq!(
        err.to_string(),
        "r20 is part of r20+21, the deleted side of r20+21+22; name that side (r20+21) or the whole change (r20+21+22)"
    );
}

fn jubarte(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn the_changes_command_lists_the_logical_change() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), split_insertion([DATE; 4])).unwrap();
    let out = jubarte(dir.path(), &["changes", "in.docx"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{stdout}");
    assert_eq!(
        stdout,
        "r7+8+9+10\tinsertion\ttext\tBob\tp0\t\"shall remain in effect until terminated.\"\t(4 elements)\n1 change(s)\n"
    );
    let out = jubarte(dir.path(), &["changes", "in.docx", "--json"]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let first: serde_json::Value = serde_json::from_str(stdout.lines().next().unwrap()).unwrap();
    let json = &first;
    assert_eq!(first["ref"], "r7+8+9+10", "{json}");
    assert_eq!(first["members"].as_array().unwrap().len(), 4, "{json}");
}

#[test]
fn the_cli_refuses_a_member_and_names_the_whole_change() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), split_insertion([DATE; 4])).unwrap();
    let out = jubarte(
        dir.path(),
        &["reject", "in.docx", "--id", "r7", "-o", "out.docx"],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("error: r7 is part of r7+8+9+10; name the whole change (r7+8+9+10)"),
        "{stderr}"
    );
    assert!(!dir.path().join("out.docx").exists());
    let out = jubarte(
        dir.path(),
        &[
            "reject",
            "in.docx",
            "--id",
            "r7+r8+r9+r10",
            "-o",
            "out.docx",
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// `read --json` lists changes as `jubarte changes` does.
#[test]
fn read_json_lists_the_logical_change() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), split_insertion([DATE; 4])).unwrap();
    let out = jubarte(dir.path(), &["read", "in.docx", "--json"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(json["schema_version"], 3, "{json}");
    let changes = json["changes"].as_array().unwrap();
    assert_eq!(changes.len(), 1, "{json}");
    assert_eq!(changes[0]["ref"], "r7+8+9+10");
    assert_eq!(changes[0]["members"].as_array().unwrap().len(), 4);
    assert_eq!(json["summary"]["revisions"], 1, "{json}");
}

/// No element is a replacement: a filter of that kind would select
/// nothing, so it is refused, naming what to select instead.
#[test]
fn a_filter_of_the_replacement_kind_is_refused() {
    let doc = docx(&p(&[del(3, "old", BOB, DATE), ins(4, "new", BOB, DATE)]));
    let filter: ChangeFilter = serde_json::from_str(r#"{"kinds":["replacement"]}"#).unwrap();
    let err = accept_changes(&doc, &filter).unwrap_err();
    assert!(
        matches!(&err, ChangeError::InvalidId(m) if m.contains("by its id (r3+4)")),
        "{err}"
    );
}
