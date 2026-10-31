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
use jubarte::markdown::{
    DocxOptions, MarkdownOptions, TrackChanges, docx_to_markdown, markdown_to_docx,
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

/// Word, and jubarte's comparer, usually write a replacement's insertion
/// first: one author's insertion then deletion is one replacement too,
/// its id in reading order (#1359).
#[test]
fn an_insertion_then_the_same_author_s_deletion_is_one_replacement() {
    let doc = docx(&p(&[
        kept("signs in "),
        ins(1, "an", BOB, DATE),
        del(2, "his or her", BOB, DATE),
        kept(" capacity"),
    ]));
    let listed = list_logical_changes(&doc).unwrap();
    assert_eq!(listed.len(), 1, "{listed:#?}");
    assert_eq!(listed[0].id, "r1+2");
    assert_eq!(listed[0].kind, ChangeKind::Replacement);
    assert_eq!(listed[0].text, "his or her → an");
    let members: Vec<(&str, ChangeKind)> = listed[0]
        .members
        .iter()
        .map(|m| (m.id.as_str(), m.kind))
        .collect();
    assert_eq!(
        members,
        [("r1", ChangeKind::Insertion), ("r2", ChangeKind::Deletion)]
    );
}

/// A replacement holds one run of each kind, in either order: a third
/// run starts a change of its own, and two authors never share one.
#[test]
fn a_replacement_ends_at_a_third_run_or_another_author() {
    let ins_del_ins = docx(&p(&[
        ins(1, "a", BOB, DATE),
        del(2, "b", BOB, DATE),
        ins(3, "c", BOB, DATE),
    ]));
    assert_eq!(ids(&ins_del_ins), ["r1+2", "r3"]);
    let del_ins_del = docx(&p(&[
        del(1, "a", BOB, DATE),
        ins(2, "b", BOB, DATE),
        del(3, "c", BOB, DATE),
    ]));
    assert_eq!(ids(&del_ins_del), ["r1+2", "r3"]);
    let two_authors = docx(&p(&[
        ins(1, "new", "Carol", DATE),
        del(2, "old", BOB, DATE),
    ]));
    assert_eq!(ids(&two_authors), ["r1", "r2"]);
    // Another author's insertion between them parts an insertion from the
    // deletion after it: the deletion is the next replacement's old side.
    let apart = docx(&p(&[
        ins(1, "and affiliates ", BOB, DATE),
        ins(2, "agree to pay the ", "Carol", DATE),
        del(3, "bankers", BOB, DATE),
        ins(4, "insurers", BOB, DATE),
    ]));
    assert_eq!(ids(&apart), ["r1", "r2", "r3+4"]);
}

/// The view keeps document order (only a deletion then an insertion
/// prints as `{~~old~>new~~}`, #1135) and tags both pieces with the
/// replacement; writing it back gives each mark its own id again.
#[test]
fn an_insertion_first_replacement_prints_in_order_and_writes_back() {
    let doc = docx(&p(&[
        kept("signs in "),
        ins(1, "an", BOB, DATE),
        del(2, "his or her", BOB, DATE),
        kept(" capacity"),
    ]));
    assert_eq!(
        body(&agent(&doc)),
        "<!-- p0 -->\nsigns in {++an++}{r1+2 @B}{--his or her--}{r1+2 @B} capacity"
    );
    keeps_every_mark(&doc);
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
    for verb in ["accept", "reject"] {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("in.docx"), split_insertion([DATE; 4])).unwrap();
        let out = jubarte(
            dir.path(),
            &[verb, "in.docx", "--id", "r7", "-o", "out.docx"],
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(2), "{stderr}");
        assert!(
            stderr
                .contains("error: r7 is part of r7+8+9+10; name the whole change: --id r7+8+9+10"),
            "{stderr}"
        );
        assert!(!dir.path().join("out.docx").exists());
        for (id, output) in [("r7+8+9+10", "out.docx"), ("r7+r8+r9+r10", "alias.docx")] {
            let out = jubarte(dir.path(), &[verb, "in.docx", "--id", id, "-o", output]);
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
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

// ------------------------------------------------------------- the view
//
// The view prints each piece of a logical change apart, as Word marked
// it, and tags every piece with the whole change's id (Q8 B): `read` and
// `changes` name the same change. Writing the view back hands the ids
// out to the pieces in turn, so each mark keeps its own `w:id`.

fn agent(doc: &[u8]) -> String {
    docx_to_markdown(
        doc,
        &MarkdownOptions {
            ids: true,
            comments: true,
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown
}

fn plain(doc: &[u8]) -> String {
    docx_to_markdown(
        doc,
        &MarkdownOptions {
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown
}

/// The view's paragraphs: what follows its YAML header.
fn body(view: &str) -> &str {
    view.strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map_or(view, |(_, body)| body)
        .trim()
}

/// Each mark Word wrote, element by element: its id, kind and text.
fn marks(doc: &[u8]) -> Vec<(String, ChangeKind, String)> {
    list_changes(doc)
        .unwrap()
        .into_iter()
        .map(|c| (c.id, c.kind, c.text))
        .collect()
}

/// `markdown` written back: the package, after no warning.
fn written(markdown: &str) -> Vec<u8> {
    let out = markdown_to_docx(markdown, &DocxOptions::default()).unwrap();
    assert!(out.warnings.is_empty(), "{:?}", out.warnings);
    out.docx
}

/// The agent view of `doc` writes back every mark with its own id and
/// text, Word-valid, and reads again as itself.
fn keeps_every_mark(doc: &[u8]) {
    let view = agent(doc);
    let back = written(&view);
    assert_word_valid_package(&back);
    assert_eq!(marks(&back), marks(doc), "the marks of\n{view}");
    assert_eq!(body(&agent(&back)), body(&view), "agent fixed point");
}

/// Bob replaced "thirty days" by "forty weeks", which Word wrote as four
/// marks.
fn four_mark_replacement() -> Vec<u8> {
    docx(&p(&[
        kept("Pay "),
        del(3, "thirty", BOB, DATE),
        del(4, " days", BOB, DATE),
        ins(1, "forty", BOB, DATE),
        ins(2, " weeks", BOB, DATE),
        kept(" end."),
    ]))
}

/// Alice replaced a word inside Bob's insertion, which Word wrote as two
/// marks around her change.
fn replacement_inside_an_insertion() -> Vec<u8> {
    const LATER: &str = "2026-10-20T07:33:05Z";
    docx(&p(&[
        kept("Start "),
        format!(
            r#"<w:ins w:id="7" w:author="{BOB}" w:date="{DATE}">{}{}</w:ins>"#,
            kept("alpha "),
            del(8, "beta", "Alice", LATER)
        ),
        ins(9, "BETA", "Alice", LATER),
        ins(10, " gamma.", BOB, DATE),
    ]))
}

/// A new paragraph: Bob's text and the mark that ends it.
fn inserted_paragraph() -> Vec<u8> {
    docx(&format!(
        "{}{}",
        p_with_inserted_mark(6, &[kept("First "), ins(5, "new text", BOB, DATE)]),
        p(&[kept("and old.")]),
    ))
}

#[test]
fn the_view_tags_each_piece_with_its_whole_change() {
    assert_eq!(
        body(&agent(&split_insertion([DATE; 4]))),
        "<!-- p0 -->\nThe licence {++shall re++}{r7+8+9+10 @B}\
         {++main in ++}{r7+8+9+10 @B}{++effect until ++}{r7+8+9+10 @B}\
         {++terminated.++}{r7+8+9+10 @B} End."
    );
}

#[test]
fn the_plain_view_tags_each_piece_with_its_whole_change() {
    assert_eq!(
        plain(&split_insertion([DATE; 4])),
        "The licence {++shall re++}{r7+8+9+10 Bob (2026-10-01T10:00:00Z)}\
         {++main in ++}{r7+8+9+10 Bob (2026-10-01T10:00:00Z)}\
         {++effect until ++}{r7+8+9+10 Bob (2026-10-01T10:00:00Z)}\
         {++terminated.++}{r7+8+9+10 Bob (2026-10-01T10:00:00Z)} End.\n"
    );
}

#[test]
fn each_piece_writes_back_as_its_own_mark() {
    keeps_every_mark(&split_insertion([DATE; 4]));
}

#[test]
fn pieces_dated_apart_keep_their_dates() {
    let doc = split_insertion([
        "2026-10-01T14:26:00Z",
        "2026-10-01T14:27:00Z",
        "2026-10-01T14:28:00Z",
        "2026-10-01T14:27:00Z",
    ]);
    let view = docx_to_markdown(
        &doc,
        &MarkdownOptions {
            ids: true,
            dates: true,
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(
        view.contains("{++shall re++}{r7+8+9+10 @B 2026-10-01T14:26:00Z}"),
        "{view}"
    );
    let back = written(&view);
    let dates = |doc: &[u8]| -> Vec<Option<String>> {
        list_changes(doc)
            .unwrap()
            .into_iter()
            .map(|c| c.date)
            .collect()
    };
    assert_eq!(dates(&back), dates(&doc));
    assert_eq!(marks(&back), marks(&doc));
}

#[test]
fn a_change_that_ends_its_paragraph_tags_the_mark_too() {
    let doc = inserted_paragraph();
    assert_eq!(
        body(&agent(&doc)),
        "<!-- p0 -->\nFirst {++new text++}{r5+6 @B}¶{r5+6 @B}\n\n<!-- p1 -->\nand old."
    );
    keeps_every_mark(&doc);
}

#[test]
fn a_replacement_of_four_marks_tags_every_piece() {
    let doc = four_mark_replacement();
    assert_eq!(
        body(&agent(&doc)),
        "<!-- p0 -->\nPay {--thirty--}{r3+4+1+2 @B}{~~ days~>forty~~}{r3+4+1+2 @B}\
         {++ weeks++}{r3+4+1+2 @B} end."
    );
    keeps_every_mark(&doc);
}

#[test]
fn a_change_inside_another_names_the_whole_of_it() {
    let doc = replacement_inside_an_insertion();
    assert_eq!(
        body(&agent(&doc)),
        "<!-- p0 -->\nStart {++alpha ++}{r7+10 @B}{~~beta~>BETA~~}{r8+9 @A re r7+10 @B}\
         {++ gamma.++}{r7+10 @B}"
    );
    keeps_every_mark(&doc);
}

/// Text another author inserted among Bob's marks leaves them one change
/// (the rule `changes` follows): the pieces either side of it share the id.
#[test]
fn pieces_either_side_of_another_authors_text_share_the_id() {
    let doc = docx(&p(&[
        kept("Pay "),
        ins(7, "within ", BOB, DATE),
        ins(8, "thirty ", "Alice", DATE),
        ins(9, "days", BOB, DATE),
        kept(" end."),
    ]));
    assert_eq!(ids(&doc), ["r7+9", "r8"]);
    assert_eq!(
        body(&agent(&doc)),
        "<!-- p0 -->\nPay {++within ++}{r7+9 @B}{++thirty ++}{r8 @A}{++days++}{r7+9 @B} end."
    );
    keeps_every_mark(&doc);
}

/// The header counts what `changes` lists: one change of four marks.
#[test]
fn the_header_counts_logical_changes() {
    let view = agent(&split_insertion([DATE; 4]));
    let line = view
        .lines()
        .find(|line| line.starts_with("revisions:"))
        .unwrap();
    assert!(
        line.contains("4 marks shown as 1 change: 1 insertion"),
        "{line}"
    );
}

/// A moved word is a change of its own: replaced by Bob's new paragraph,
/// it prints apart from the paragraph's change, never one substitution of
/// two changes (#1363), and every mark keeps its id.
#[test]
fn a_move_beside_a_change_keeps_its_own_tag() {
    let range = |tag: &str, id: u32| {
        format!(r#"<w:{tag} w:id="{id}" w:author="{BOB}" w:date="{DATE}" w:name="move1"/>"#)
    };
    let doc = docx(&format!(
        "{}{}",
        p_with_inserted_mark(
            6,
            &[
                kept("Pay "),
                range("moveFromRangeStart", 20),
                format!(
                    r#"<w:moveFrom w:id="3" w:author="{BOB}" w:date="{DATE}">{}</w:moveFrom>"#,
                    kept("General")
                ),
                r#"<w:moveFromRangeEnd w:id="20"/>"#.to_string(),
                ins(5, "CONSENT", BOB, DATE),
            ],
        ),
        p(&[
            kept("Then "),
            range("moveToRangeStart", 21),
            format!(
                r#"<w:moveTo w:id="4" w:author="{BOB}" w:date="{DATE}">{}</w:moveTo>"#,
                kept("General")
            ),
            r#"<w:moveToRangeEnd w:id="21"/>"#.to_string(),
            kept(" end."),
        ]),
    ));
    assert_eq!(ids(&doc), ["r3", "r5+6", "r4"]);
    let view = agent(&doc);
    assert!(
        body(&view)
            .starts_with("<!-- p0 -->\nPay {--General--}{r3 @B}{++CONSENT++}{r5+6 @B}¶{r5+6 @B}"),
        "{view}"
    );
    let back = written(&view);
    let wids = |doc: &[u8]| -> Vec<String> { marks(doc).into_iter().map(|(id, ..)| id).collect() };
    assert_eq!(wids(&back), wids(&doc));
}

/// #1353 review: Carol's word inside Alice's insertion, between
/// Bob's deletions there, came after them, as at the top level.
#[test]
fn another_authors_insertion_inside_the_insertion_the_marks_sit_in_separates_nothing() {
    let doc = docx(&p(&[
        kept("Start "),
        format!(
            r#"<w:ins w:id="3" w:author="Alice" w:date="{DATE}">{}{}{}{}</w:ins>"#,
            kept("one "),
            del(4, "two ", BOB, DATE),
            ins(9, "X", "Carol", DATE),
            del(5, "three", BOB, DATE)
        ),
    ]));
    let listed = list_logical_changes(&doc).unwrap();
    let rows: Vec<(&str, Option<&str>)> = listed
        .iter()
        .map(|c| (c.id.as_str(), c.inside.as_deref()))
        .collect();
    assert_eq!(
        rows,
        [("r3", None), ("r4+5", Some("r3")), ("r9", Some("r3"))]
    );
}

/// #1353 review: a replacement whose deletion sits in Alice's
/// insertion and whose insertion follows it is inside nothing.
#[test]
fn a_change_is_inside_another_only_when_all_of_it_is() {
    let insides = |after: String| {
        let doc = docx(&p(&[
            format!(
                r#"<w:ins w:id="3" w:author="Alice" w:date="{DATE}">{}{}</w:ins>"#,
                kept("one "),
                del(4, "x", BOB, DATE)
            ),
            after,
        ]));
        list_logical_changes(&doc)
            .unwrap()
            .into_iter()
            .map(|c| (c.id, c.inside))
            .collect::<Vec<_>>()
    };
    let named = |id: &str, inside: Option<&str>| (id.to_string(), inside.map(str::to_string));
    // Two deletions, one in Alice's insertion and one after it: not all of
    // the change is inside r3.
    assert_eq!(
        insides(del(5, "y", BOB, DATE)),
        [named("r3", None), named("r4+5", None)]
    );
    // A replacement is inside what its old side is inside, as the view's
    // `re` names it (`{~~x~>y~~}{r4+5 @B re r3 @A}`).
    assert_eq!(
        insides(ins(5, "y", BOB, DATE)),
        [named("r3", None), named("r4+5", Some("r3"))]
    );
}

/// #1353 review: a filter whose kinds select none of a change's
/// marks selects nothing of it, and settles nothing.
#[test]
fn a_filter_that_selects_no_mark_of_a_change_settles_nothing() {
    let doc = split_insertion([DATE; 4]);
    let filter = ChangeFilter {
        kinds: Some(vec![ChangeKind::Deletion]),
        ..ChangeFilter::ids(["r7"])
    };
    let out = reject_changes(&doc, &filter).unwrap();
    assert_eq!(ids(&out), ["r7+8+9+10"]);
    let insertions = ChangeFilter {
        kinds: Some(vec![ChangeKind::Insertion]),
        ..ChangeFilter::ids(["r7"])
    };
    assert!(matches!(
        reject_changes(&doc, &insertions),
        Err(ChangeError::PartOfChange { .. })
    ));
}

/// #1353 review: `changes` prints the kind replacement, so
/// `--kind replacement` reaches the refusal that says how to select one.
#[test]
fn the_cli_refuses_the_replacement_kind_with_how_to_select_one() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), four_mark_replacement()).unwrap();
    let out = jubarte(
        dir.path(),
        &[
            "reject",
            "in.docx",
            "--kind",
            "replacement",
            "-o",
            "out.docx",
        ],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("select a replacement by its id"),
        "{stderr}"
    );
    assert!(!dir.path().join("out.docx").exists());
    let help = jubarte(dir.path(), &["reject", "--help"]);
    assert!(!String::from_utf8_lossy(&help.stdout).contains("replacement"));
}

/// `jubarte read` of `docx` as the round trip compares it: no page
/// markers, and without the lines that name the file.
fn read_back(dir: &std::path::Path, docx: &str) -> String {
    let out = jubarte(dir, &["read", docx, "--no-page-markers"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .filter(|line| !line.starts_with("source: ") && !line.starts_with("sha256: "))
        .map(|line| format!("{line}\n"))
        .collect()
}

/// The `w:id` of every `w:ins` and `w:del` in `part`, in order.
fn part_ids(docx: &[u8], part: &str) -> Vec<String> {
    use std::io::Read as _;
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(docx)).unwrap();
    let mut xml = String::new();
    zip.by_name(part).unwrap().read_to_string(&mut xml).unwrap();
    regex::Regex::new(r#"<w:(?:ins|del) w:id="(\d+)""#)
        .unwrap()
        .captures_iter(&xml)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// A header's change printed on two pieces, and a note whose paragraphs'
/// text and mark are one change (`rev footnotes.r32+31`), write back as
/// the marks they came from: the view reads the same, every id kept.
#[test]
fn a_story_change_of_several_pieces_writes_back_each_mark() {
    for (fixture, part) in [
        ("edge-authorless.docx", "word/header1.xml"),
        ("edge-notes-tracked.docx", "word/footnotes.xml"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let source = std::fs::read(format!(
            "{}/tests/fixtures/md-roundtrip/edges/{fixture}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        std::fs::write(dir.path().join("a.docx"), &source).unwrap();
        let view = read_back(dir.path(), "a.docx");
        assert!(view.contains("r92+93") || view.contains("r32+31"), "{view}");
        std::fs::write(dir.path().join("a.md"), &view).unwrap();
        let out = jubarte(
            dir.path(),
            &[
                "convert",
                "a.md",
                "-o",
                "b.docx",
                "-d",
                "1970-01-01T00:00:00Z",
            ],
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "{stderr}");
        assert!(
            !stderr.contains("names two separate changes"),
            "{fixture}: {stderr}"
        );
        assert_eq!(read_back(dir.path(), "b.docx"), view, "{fixture}");
        let back = std::fs::read(dir.path().join("b.docx")).unwrap();
        assert_eq!(part_ids(&back, part), part_ids(&source, part), "{fixture}");
    }
}

/// #1353 review: a comment that goes with a change of
/// several marks is reported as removed by that change, named as the
/// selection named it, not by the member that held its anchor.
#[test]
fn a_removed_comment_names_the_whole_change_that_held_it() {
    let anchored = format!(
        r#"<w:ins w:id="8" w:author="{BOB}" w:date="{DATE}"><w:commentRangeStart w:id="0"/><w:r><w:t xml:space="preserve">main in </w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:ins>"#
    );
    let body = p(&[
        kept("The licence "),
        ins(7, "shall re", BOB, DATE),
        anchored,
        ins(9, "effect until ", BOB, DATE),
        ins(10, "terminated.", BOB, DATE),
        kept(" End."),
    ]);
    let comments = format!(
        r#"<w:comments xmlns:w="{}"><w:comment w:id="0" w:author="Ann"><w:p><w:r><w:t>Why?</w:t></w:r></w:p></w:comment></w:comments>"#,
        common::docx::W_NS
    );
    let doc = common::docx::docx_with(
        &body,
        &[common::docx::Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    );
    assert_eq!(ids(&doc), ["r7+8+9+10"]);
    let out = jubarte::changes::reject_changes_with_report(&doc, &ChangeFilter::ids(["r7+8+9+10"]))
        .unwrap();
    assert_eq!(
        out.removed_why,
        ["anchor removed by rejecting r7+8+9+10"],
        "{:?}",
        out.removed_comments
    );
}

/// Carol's edit inside Bob's insertion, which Bob made among Ann's marks,
/// is Bob's text edited: it separates Ann's marks no more than Bob's
/// insertion does.
#[test]
fn another_authors_edit_inside_an_insertion_among_the_marks_separates_nothing() {
    let doc = docx(&p(&[
        ins(1, "The parties ", "Ann", DATE),
        format!(
            r#"<w:ins w:id="2" w:author="{BOB}" w:date="{DATE}"><w:r><w:t xml:space="preserve">and </w:t></w:r>{}</w:ins>"#,
            del(7, "their", "Carol", DATE)
        ),
        ins(8, "its", "Carol", DATE),
        ins(9, " affiliates ", BOB, DATE),
        ins(3, "agree to pay.", "Ann", DATE),
    ]));
    let rows: Vec<(String, Option<String>)> = list_logical_changes(&doc)
        .unwrap()
        .into_iter()
        .map(|c| (c.id, c.inside))
        .collect();
    assert_eq!(
        rows,
        [
            ("r1+3".to_string(), None),
            ("r2+9".to_string(), None),
            ("r7+8".to_string(), Some("r2+9".to_string())),
        ]
    );
}

/// A paragraph whose own mark Bob inserted (`ins`) or deleted (`del`) as
/// change `mark`.
fn p_with_mark(kind: &str, mark: u32, content: &[String]) -> String {
    format!(
        r#"<w:p><w:pPr><w:rPr><w:{kind} w:id="{mark}" w:author="{BOB}" w:date="{DATE}"/></w:rPr></w:pPr>{}</w:p>"#,
        content.concat()
    )
}

/// The view header's `revisions:` line.
fn revisions_line(view: &str) -> String {
    view.lines()
        .find(|line| line.starts_with("revisions:"))
        .unwrap_or_default()
        .to_string()
}

/// #1362: a replacement one of whose sides is its paragraph's mark counts
/// as the substitution `changes` lists, in the body and in a cell; a mark
/// alone still counts as a paragraph mark.
#[test]
fn a_replacement_with_a_paragraph_mark_side_counts_as_a_substitution() {
    let page_break = r#"<w:ins w:id="1" w:author="Bob" w:date="2026-10-01T10:00:00Z"><w:r><w:br w:type="page"/></w:r></w:ins>"#;
    // A cell prints no page break (the table line has no clause for one),
    // so the break's shape is the body's only.
    let shapes = [
        (p_with_mark("ins", 2, &[del(1, "x", BOB, DATE)]), true),
        (p_with_mark("del", 2, &[ins(1, "x", BOB, DATE)]), true),
        (p_with_mark("del", 2, &[page_break.to_string()]), false),
    ];
    for (shape, in_cell) in &shapes {
        let cell = format!("<w:tbl><w:tr><w:tc>{shape}</w:tc></w:tr></w:tbl>");
        for block in std::iter::once(shape).chain(in_cell.then_some(&cell)) {
            let doc = docx(&format!("{block}{}", p(&[kept("After.")])));
            let listed: Vec<(String, ChangeKind)> = list_logical_changes(&doc)
                .unwrap()
                .into_iter()
                .map(|c| (c.id, c.kind))
                .collect();
            assert_eq!(
                listed,
                [("r1+2".to_string(), ChangeKind::Replacement)],
                "{block}"
            );
            let view = agent(&doc);
            assert!(
                revisions_line(&view).ends_with("# 2 marks shown as 1 change: 1 substitution"),
                "{block}\n{view}"
            );
        }
    }
    for kind in ["ins", "del"] {
        let doc = docx(&format!(
            "{}{}",
            p_with_mark(kind, 2, &[kept("Kept.")]),
            p(&[kept("After.")])
        ));
        assert!(
            revisions_line(&agent(&doc)).ends_with("# 1 mark shown as 1 change: 1 paragraph mark"),
            "{kind}\n{}",
            agent(&doc)
        );
    }
}

/// A text box anchored where it stands, `content` its `w:txbxContent`.
fn text_box(content: &str) -> String {
    format!(
        r#"<wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" distT="0" distB="0" distL="114300" distR="114300" simplePos="0" relativeHeight="1" behindDoc="0" locked="0" layoutInCell="1" allowOverlap="1"><wp:simplePos x="0" y="0"/><wp:positionH relativeFrom="column"><wp:align>center</wp:align></wp:positionH><wp:positionV relativeFrom="paragraph"><wp:posOffset>0</wp:posOffset></wp:positionV><wp:extent cx="2374265" cy="1403985"/><wp:effectExtent l="0" t="0" r="0" b="0"/><wp:wrapNone/><wp:docPr id="307" name="Text Box 2"/><wp:cNvGraphicFramePr/><a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:graphicData uri="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:wsp xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><wps:cNvSpPr txBox="1"/><wps:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="2374265" cy="1403985"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></wps:spPr><wps:txbx><w:txbxContent>{content}</w:txbxContent></wps:txbx><wps:bodyPr/></wps:wsp></a:graphicData></a:graphic></wp:anchor>"#
    )
}

/// A paragraph holding Bob's inserted text box (`w:ins` 12, its own text
/// `w:ins` 14 and paragraph mark `w:ins` 13 inside the box), then his
/// deletion 17 of the paragraph's text and of its mark (11).
fn inserted_text_box_replacing_text() -> Vec<u8> {
    let at = format!(r#"w:author="{BOB}" w:date="{DATE}""#);
    let anchor = text_box(&format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="13" {at}/></w:rPr></w:pPr><w:ins w:id="14" {at}><w:r><w:t>Datum plane</w:t></w:r></w:ins></w:p>"#
    ));
    docx(&format!(
        r#"<w:p><w:pPr><w:rPr><w:del w:id="11" {at}/></w:rPr></w:pPr><w:ins w:id="12" {at}><w:r><w:drawing>{anchor}</w:drawing></w:r></w:ins>{}</w:p>{}"#,
        del(17, "Gone. ", BOB, DATE),
        p(&[kept("After.")])
    ))
}

/// The text box's text prints its anchor's tag, the replacement's whole id
/// (#1359); written back, that copy of the tag takes the anchor's id, so the
/// deletion and the paragraph mark keep theirs.
#[test]
fn a_text_box_in_an_insertion_first_replacement_keeps_every_id() {
    let doc = inserted_text_box_replacing_text();
    assert_eq!(ids(&doc)[0], "r12+17+11", "{:?}", ids(&doc));
    let view = agent(&doc);
    assert!(
        view.contains("{--Gone. --}{r12+17+11 @B}")
            && view.contains("++}{r12+17+11 @B}\n\n<!-- /text-box -->"),
        "{view}"
    );
    let out = markdown_to_docx(&view, &DocxOptions::default()).unwrap();
    for id in ["r17 ", "r11 "] {
        assert!(
            !out.warnings
                .iter()
                .any(|w| w.starts_with(&format!("warning: {id}")) || w.starts_with(id)),
            "{:?}",
            out.warnings
        );
    }
    assert_word_valid_package(&out.docx);
    assert_eq!(marks(&out.docx), marks(&doc), "the marks of\n{view}");
    assert_eq!(body(&agent(&out.docx)), body(&view), "agent fixed point");
}

/// A paragraph holding Bob's insertion 7 of `a` and of a text box whose
/// text has no change of its own, then `host`.
fn text_box_in_a_single_mark() -> Vec<u8> {
    let at = format!(r#"w:author="{BOB}" w:date="{DATE}""#);
    let anchor = text_box(r#"<w:p><w:r><w:t xml:space="preserve">box </w:t></w:r></w:p>"#);
    docx(&format!(
        r#"<w:p><w:ins w:id="7" {at}><w:r><w:t>a</w:t></w:r><w:r><w:drawing>{anchor}</w:drawing></w:r></w:ins><w:r><w:t xml:space="preserve">host</w:t></w:r></w:p>"#
    ))
}

/// A text box with no change of its own prints its anchor's tag. A copy of
/// a single-id tag is the anchor's mark, which holds the box already, so
/// written back it takes no id of its own and warns of nothing, the box
/// unchanged or edited (#1353 review).
#[test]
fn a_text_box_copy_of_a_single_id_tag_writes_no_mark_of_its_own() {
    let doc = text_box_in_a_single_mark();
    assert_eq!(ids(&doc), ["r7"]);
    let view = agent(&doc);
    assert!(view.contains("{++box ++}{r7 @B}"), "{view}");
    let edited = view.replace("{++box ++}{r7 @B}", "{++box, edited ++}{r7 @B}");
    for (name, view) in [("unchanged", &view), ("edited", &edited)] {
        let out = markdown_to_docx(view, &DocxOptions::default()).unwrap();
        assert!(
            !out.warnings
                .iter()
                .any(|w| w.contains("names two separate changes")),
            "{name}: {:?}",
            out.warnings
        );
        assert_word_valid_package(&out.docx);
        let kinds = |doc: &[u8]| -> Vec<(String, ChangeKind)> {
            marks(doc)
                .into_iter()
                .map(|(id, kind, _)| (id, kind))
                .collect()
        };
        assert_eq!(
            kinds(&out.docx),
            kinds(&doc),
            "{name}: the marks of\n{view}"
        );
        // The drawing's payload records the box text's hash, which an edit
        // changes; the text before it is the fixed point.
        let text = |view: &str| body(view).split("\n\n[d1]:").next().unwrap().to_string();
        assert_eq!(
            text(&agent(&out.docx)),
            text(view),
            "{name}: agent fixed point"
        );
    }
}

/// #1363: content the view prints nothing for (a hidden run, a soft
/// hyphen, a symbol, a page break) still keeps a deletion and the
/// insertion after it two changes, as `changes` lists them; with nothing
/// between them they are one substitution.
#[test]
fn a_deletion_and_an_insertion_apart_print_as_the_two_changes_they_are() {
    let between = [
        r#"<w:r><w:rPr><w:vanish/></w:rPr><w:t>note</w:t></w:r>"#,
        r#"<w:r><w:softHyphen/></w:r>"#,
        r#"<w:r><w:sym w:font="Wingdings" w:char="F0FC"/></w:r>"#,
        r#"<w:r><w:br w:type="page"/></w:r>"#,
    ];
    for gap in between {
        let doc = docx(&p(&[
            kept("pay "),
            del(1, "old", BOB, DATE),
            gap.to_string(),
            ins(2, "new", BOB, DATE),
            kept(" now"),
        ]));
        assert_eq!(ids(&doc), ["r1", "r2"], "{gap}");
        let view = agent(&doc);
        let line = body(&view).lines().nth(1).unwrap_or_default();
        assert_eq!(line, "pay {--old--}{r1 @B}{++new++}{r2 @B} now", "{gap}");
        assert!(
            view.contains("2 marks shown as 2 changes: 1 insertion, 1 deletion"),
            "{gap}\n{view}"
        );
    }
    let touching = docx(&p(&[
        kept("pay "),
        del(1, "old", BOB, DATE),
        ins(2, "new", BOB, DATE),
        kept(" now"),
    ]));
    assert_eq!(ids(&touching), ["r1+2"]);
    assert_eq!(
        body(&agent(&touching)),
        "<!-- p0 -->\npay {~~old~>new~~}{r1+2 @B} now"
    );
}

/// A deletion and the insertion after it that repeat one `w:id` are two
/// changes, as `changes` lists them (a repeated id never joins), so the
/// view prints them apart, not as the composite `r7+7` no listing names
/// (#1353 review).
#[test]
fn a_deletion_and_an_insertion_repeating_one_id_print_apart() {
    let doc = docx(&p(&[
        kept("pay "),
        del(7, "old", BOB, DATE),
        ins(7, "new", BOB, DATE),
        kept(" now"),
    ]));
    assert_eq!(ids(&doc), ["r7", "r7"]);
    let view = agent(&doc);
    assert_eq!(
        body(&view),
        "<!-- p0 -->\npay {--old--}{r7 @B}{++new++}{r7 @B} now"
    );
    assert!(
        view.contains("2 marks shown as 2 changes: 1 insertion, 1 deletion"),
        "{view}"
    );
}

/// #1363 in a resolved view: the id line names each change `changes`
/// lists, not a pair of them.
#[test]
fn a_resolved_line_names_two_changes_apart_as_two() {
    let doc = docx(&p(&[
        kept("pay "),
        del(1, "old", BOB, DATE),
        r#"<w:r><w:rPr><w:vanish/></w:rPr><w:t>note</w:t></w:r>"#.to_string(),
        ins(2, "new", BOB, DATE),
        kept(" now"),
    ]));
    let accepted = docx_to_markdown(
        &doc,
        &MarkdownOptions {
            ids: true,
            track_changes: TrackChanges::Accept,
            page_markers: false,
            ..MarkdownOptions::default()
        },
    )
    .unwrap()
    .markdown;
    assert!(
        body(&accepted).starts_with("<!-- p0 rev r1 @B; r2 @B -->\npay new now"),
        "{accepted}"
    );
}
