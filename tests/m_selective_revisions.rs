// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Accepting or rejecting some tracked changes and leaving the rest tracked,
//! as Word's Accept/Reject This Change does.

mod common;

use common::docx::{Part, docx, docx_with_sect, part_string};
use common::validity::{assert_word_valid_package, check_word_valid_package};
use jubarte::changes::{
    Change, ChangeError, ChangeFilter, ChangeKind, MoveSide, accept_changes, list_changes,
    reject_changes,
};
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use std::collections::HashSet;

const A: &str = r#"w:author="Ann" w:date="2026-01-01T00:00:00Z""#;
const B: &str = r#"w:author="Bob" w:date="2026-01-02T00:00:00Z""#;

/// One insertion by Ann, one deletion by Bob, one paragraph formatting
/// change by Ann.
fn three_changes() -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">Keep </w:t></w:r><w:ins w:id="1" {A}><w:r><w:t>new</w:t></w:r></w:ins><w:del w:id="2" {B}><w:r><w:delText>old</w:delText></w:r></w:del></w:p><w:p><w:pPr><w:jc w:val="center"/><w:pPrChange w:id="3" {A}><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>Centred</w:t></w:r></w:p>"#
    ))
}

fn ids(pkg: &[u8]) -> Vec<String> {
    list_changes(pkg)
        .unwrap()
        .into_iter()
        .map(|c| c.id)
        .collect()
}

fn filter_ids(ids: &[&str]) -> ChangeFilter {
    ChangeFilter::ids(ids.iter().copied())
}

#[test]
fn list_changes_names_each_change_by_story_and_id() {
    let changes = list_changes(&three_changes()).unwrap();
    let got: Vec<_> = changes
        .iter()
        .map(|c| (c.id.as_str(), c.kind, c.author.as_deref(), c.text.as_str()))
        .collect();
    assert_eq!(
        got,
        [
            ("body:rev:1", ChangeKind::Insertion, Some("Ann"), "new"),
            ("body:rev:2", ChangeKind::Deletion, Some("Bob"), "old"),
            ("body:rev:3", ChangeKind::Formatting, Some("Ann"), "Centred"),
        ]
    );
}

#[test]
fn accepting_one_insertion_leaves_the_others_tracked() {
    let out = accept_changes(&three_changes(), &filter_ids(&["body:rev:1"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["body:rev:2", "body:rev:3"]);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(
        xml.contains(">new</w:t>") && !xml.contains(r#"<w:ins w:id="1""#),
        "{xml}"
    );
    assert!(xml.contains("<w:delText>old</w:delText>"), "{xml}");
}

#[test]
fn rejecting_by_author_leaves_the_other_authors_changes_tracked() {
    let filter = ChangeFilter {
        authors: Some(vec!["Bob".into()]),
        ..Default::default()
    };
    let out = reject_changes(&three_changes(), &filter).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["body:rev:1", "body:rev:3"]);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(
        xml.contains(">old</w:t>") && !xml.contains("delText"),
        "{xml}"
    );
}

#[test]
fn a_kind_filter_resolves_only_that_kind() {
    let filter = ChangeFilter {
        kinds: Some(vec![ChangeKind::Formatting]),
        ..Default::default()
    };
    let out = reject_changes(&three_changes(), &filter).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["body:rev:1", "body:rev:2"]);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(!xml.contains("<w:jc"), "{xml}");
}

#[test]
fn filters_combine_as_and() {
    let filter = ChangeFilter {
        authors: Some(vec!["Ann".into()]),
        kinds: Some(vec![ChangeKind::Insertion]),
        ..Default::default()
    };
    let out = accept_changes(&three_changes(), &filter).unwrap();
    assert_eq!(ids(&out), ["body:rev:2", "body:rev:3"]);
}

#[test]
fn selecting_every_change_is_accept_or_reject_all() {
    let pkg = three_changes();
    let all = ChangeFilter::default();
    assert_eq!(
        accept_changes(&pkg, &all).unwrap(),
        accept_revisions(&pkg).unwrap()
    );
    assert_eq!(
        reject_changes(&pkg, &all).unwrap(),
        reject_revisions(&pkg).unwrap()
    );
}

#[test]
fn an_unknown_id_is_refused() {
    let err = accept_changes(&three_changes(), &filter_ids(&["body:rev:9"])).unwrap_err();
    assert!(
        matches!(&err, ChangeError::UnknownChange(id) if id == "body:rev:9"),
        "{err:?}"
    );
}

/// Word accepts or rejects a move as one change: selecting either side
/// resolves both, range markers included.
#[test]
fn a_move_resolves_both_sides() {
    let pkg = docx(&format!(
        r#"<w:p><w:moveFromRangeStart w:id="10" w:name="move1" {A}/><w:moveFrom w:id="11" {A}><w:r><w:t>Moved</w:t></w:r></w:moveFrom><w:moveFromRangeEnd w:id="10"/></w:p><w:p><w:r><w:t>Stays</w:t></w:r></w:p><w:p><w:moveToRangeStart w:id="12" w:name="move1" {A}/><w:moveTo w:id="13" {A}><w:r><w:t>Moved</w:t></w:r></w:moveTo><w:moveToRangeEnd w:id="12"/><w:ins w:id="14" {B}><w:r><w:t xml:space="preserve"> too</w:t></w:r></w:ins></w:p>"#
    ));
    let listed = list_changes(&pkg).unwrap();
    assert_eq!(
        listed.iter().map(|c| c.kind).collect::<Vec<_>>(),
        [ChangeKind::Move, ChangeKind::Move, ChangeKind::Insertion]
    );
    assert_eq!(listed[0].move_name.as_deref(), Some("move1"));
    assert_eq!(
        listed.iter().map(|c| c.move_side).collect::<Vec<_>>(),
        [Some(MoveSide::From), Some(MoveSide::To), None]
    );
    for side in ["body:rev:11", "body:rev:13"] {
        let out = accept_changes(&pkg, &filter_ids(&[side])).unwrap();
        assert_word_valid_package(&out);
        assert_eq!(ids(&out), ["body:rev:14"], "accepting {side}");
        let xml = part_string(&out, "word/document.xml").unwrap();
        assert_eq!(xml.matches(">Moved<").count(), 1, "{xml}");
        assert!(
            !xml.contains("moveFrom") && !xml.contains("moveTo"),
            "{xml}"
        );
    }
    let out = reject_changes(&pkg, &filter_ids(&["body:rev:13"])).unwrap();
    assert_eq!(ids(&out), ["body:rev:14"]);
}

/// A paragraph mark's own insertion is a change of its own.
#[test]
fn an_inserted_paragraph_mark_is_its_own_change() {
    let pkg = docx(&format!(
        r#"<w:p><w:pPr><w:rPr><w:ins w:id="5" {A}/></w:rPr></w:pPr><w:ins w:id="6" {A}><w:r><w:t>First</w:t></w:r></w:ins></w:p><w:p><w:r><w:t>Second</w:t></w:r></w:p>"#
    ));
    let listed = list_changes(&pkg).unwrap();
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0].target, "paragraph_mark");
    assert_eq!(listed[1].target, "text");
    let out = reject_changes(&pkg, &filter_ids(&["body:rev:6"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["body:rev:5"]);
}

#[test]
fn header_changes_carry_their_story() {
    let header = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:ins w:id="7" {B}><w:r><w:t>Head</w:t></w:r></w:ins></w:p></w:hdr>"#
    );
    let header: &'static str = Box::leak(header.into_boxed_str());
    let pkg = docx_with_sect(
        &format!(r#"<w:p><w:ins w:id="8" {A}><w:r><w:t>Body</w:t></w:r></w:ins></w:p>"#),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    );
    assert_eq!(ids(&pkg), ["body:rev:8", "header1:rev:7"]);
    let out = reject_changes(&pkg, &filter_ids(&["header1:rev:7"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["body:rev:8"]);
    let hx = part_string(&out, "word/header1.xml").unwrap();
    assert!(!hx.contains("Head"), "{hx}");
}

#[test]
fn a_filter_reads_from_json() {
    let f: ChangeFilter = serde_json::from_str(
        r#"{"ids":["body:rev:1"],"authors":["Ann"],"kinds":["insertion","move"]}"#,
    )
    .unwrap();
    assert_eq!(f.kinds, Some(vec![ChangeKind::Insertion, ChangeKind::Move]));
    assert!(serde_json::from_str::<ChangeFilter>(r#"{"author":"Ann"}"#).is_err());
}

/// A list given empty selects nothing (an agent's empty id list must not
/// resolve every change); `{}` selects every change.
#[test]
fn an_empty_list_selects_nothing() {
    let pkg = three_changes();
    for filter in [r#"{"ids":[]}"#, r#"{"authors":[]}"#, r#"{"kinds":[]}"#] {
        let f: ChangeFilter = serde_json::from_str(filter).unwrap();
        assert_eq!(accept_changes(&pkg, &f).unwrap(), pkg, "{filter}");
        assert_eq!(reject_changes(&pkg, &f).unwrap(), pkg, "{filter}");
    }
    let all: ChangeFilter = serde_json::from_str("{}").unwrap();
    assert!(ids(&accept_changes(&pkg, &all).unwrap()).is_empty());
}

/// The bench beside this checkout, or beside the main checkout from a
/// worktree.
fn bench() -> Option<std::path::PathBuf> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    [
        root.join("../neurotic_docx_bench"),
        root.join("../../../neurotic_docx_bench"),
    ]
    .into_iter()
    .find(|p| {
        p.join("corpus/word/notices/rejected_tracking_selection.csv")
            .is_file()
    })
}

/// On Word's own redlines (the bench's `rejected_tracking` set; skipped
/// without it): resolving every other change first and the rest after ends
/// where resolving them all at once does, and the half-way document is
/// Word-valid with every unselected id still listed.
#[test]
fn resolving_in_two_halves_ends_where_resolving_all_does() {
    use jubarte::debug::{Check, Options, report};
    let Some(bench) = bench() else {
        eprintln!("skip: neurotic_docx_bench missing");
        return;
    };
    let selection =
        std::fs::read_to_string(bench.join("corpus/word/notices/rejected_tracking_selection.csv"))
            .unwrap();
    let mut rows = selection.lines();
    let header: Vec<&str> = rows.next().unwrap().split(',').collect();
    let docx_col = header.iter().position(|h| *h == "docx").unwrap();
    let opts = Options {
        checks: vec![Check::Text],
        ..Default::default()
    };
    let (mut pairs, mut differ) = (0, Vec::new());
    // A malformed row or a missing redline fails; nothing listed is skipped.
    for row in rows.filter(|r| !r.trim().is_empty()) {
        let Some(docx) = row.split(',').nth(docx_col) else {
            differ.push(format!("malformed row: {row}"));
            continue;
        };
        let Ok(redline) = std::fs::read(bench.join("corpus/word").join(docx)) else {
            differ.push(format!("{docx}: missing"));
            continue;
        };
        let listed = list_changes(&redline).unwrap();
        // What Word's own redline already breaks is not the resolution's.
        let inherited: HashSet<String> = check_word_valid_package(&redline)
            .errors
            .into_iter()
            .collect();
        let half: Vec<String> = listed.iter().step_by(2).map(|c| c.id.clone()).collect();
        let first = ChangeFilter::ids(half);
        for (name, one, all) in [
            (
                "accept",
                accept_changes as fn(&[u8], &ChangeFilter) -> Result<Vec<u8>, ChangeError>,
                accept_revisions as fn(&[u8]) -> Result<Vec<u8>, _>,
            ),
            ("reject", reject_changes, reject_revisions),
        ] {
            pairs += 1;
            let partial = one(&redline, &first).unwrap();
            let broken: Vec<String> = check_word_valid_package(&partial)
                .errors
                .into_iter()
                .filter(|e| !inherited.contains(e))
                .collect();
            if !broken.is_empty() {
                differ.push(format!("{docx} [{name}]: invalid half-way: {broken:?}"));
                continue;
            }
            let left: HashSet<String> = ids(&partial).into_iter().collect();
            let selected = |c: &Change| {
                first.matches(c)
                    || c.move_name.as_ref().is_some_and(|m| {
                        listed
                            .iter()
                            .any(|o| o.move_name.as_ref() == Some(m) && first.matches(o))
                    })
            };
            // Resolving a change so its content goes takes what it holds.
            let removes = |c: &Change| match (name, c.kind) {
                ("accept", ChangeKind::Deletion) | ("reject", ChangeKind::Insertion) => true,
                (_, ChangeKind::Move) => {
                    (c.move_side == Some(MoveSide::From)) == (name == "accept")
                }
                _ => false,
            };
            let by_id: std::collections::HashMap<&str, &Change> =
                listed.iter().map(|c| (c.id.as_str(), c)).collect();
            let swept = |c: &Change| {
                let mut at = c.inside.as_deref();
                while let Some(id) = at {
                    let holder = by_id[id];
                    if selected(holder) && removes(holder) {
                        return true;
                    }
                    at = holder.inside.as_deref();
                }
                false
            };
            let expected: HashSet<String> = listed
                .iter()
                .filter(|c| !selected(c) && !swept(c))
                .map(|c| c.id.clone())
                .collect();
            // A note goes with a reference its resolved change removed, a
            // header or footer with the section a resolved mark ended.
            let story_went = |id: &str| {
                let story = id.split(':').next().unwrap_or("");
                matches!(story, "footnotes" | "endnotes")
                    || story != "body"
                        && part_string(&partial, &format!("word/{story}.xml")).is_none()
            };
            if !left.is_subset(&expected) || expected.difference(&left).any(|id| !story_went(id)) {
                let mut lost: Vec<_> = expected.difference(&left).collect();
                let mut stayed: Vec<_> = left.difference(&expected).collect();
                lost.sort();
                stayed.sort();
                differ.push(format!("{docx} [{name}]: lost {lost:?}, stayed {stayed:?}"));
                continue;
            }
            let finished = one(&partial, &ChangeFilter::default()).unwrap();
            let out = report(&all(&redline).unwrap(), Some(&finished), &opts).unwrap();
            // A paragraph mark resolved on its own before a table (or at a
            // story's end) cannot merge away and stays, as in Word; resolving
            // the table later leaves the paragraph empty or split where
            // resolving both at once joins it (A.5a). The text may not differ.
            let side = |sign: &str| -> String {
                out.lines()
                    .filter(|l| l.starts_with(sign))
                    .filter_map(|l| l.split_once('¶'))
                    .map(|(_, t)| t.strip_prefix("  ").unwrap_or(t))
                    .collect()
            };
            let only_breaks = out.lines().all(|l| {
                l.ends_with("differ")
                    || l.ends_with("differs")
                    || l.trim() == "~"
                    || l.starts_with("-A   ¶")
                    || l.starts_with("+B   ¶")
            }) && side("-A") == side("+B");
            if out != "text identical\n" && !only_breaks {
                differ.push(format!("{docx} [{name}]:\n{out}"));
            }
        }
    }
    assert!(pairs > 0);
    assert!(
        differ.is_empty(),
        "{} of {pairs} differ:\n{}",
        differ.len(),
        differ.join("\n")
    );
}

/// Rejecting an inserted row that holds a comment's start, while the text
/// holding its end and reference stays tracked, keeps the comment: its
/// range collapses onto the surviving end instead of losing its start
/// (Word-invalid; bench 7787357743).
#[test]
fn a_comment_cut_by_a_resolved_row_keeps_its_range() {
    let comments = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:comment w:id="0" w:author="Ann" w:date="2026-01-01T00:00:00Z" w:initials="A"><w:p><w:r><w:t>Note</w:t></w:r></w:p></w:comment></w:comments>"#;
    let pkg = common::docx::docx_with(
        &format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:trPr><w:ins w:id="1" {A}/></w:trPr><w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p><w:commentRangeStart w:id="0"/><w:ins w:id="3" {A}><w:r><w:t>Cell</w:t></w:r></w:ins></w:p></w:tc></w:tr></w:tbl><w:p><w:ins w:id="2" {B}><w:r><w:t>Kept</w:t></w:r></w:ins><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>"#
        ),
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: comments,
        }],
    );
    let out = reject_changes(&pkg, &filter_ids(&["body:rev:1"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["body:rev:2"]);
    let xml = part_string(&out, "word/document.xml").unwrap();
    assert!(!xml.contains("Cell"), "{xml}");
    assert!(
        xml.contains(r#"<w:commentRangeStart w:id="0" /><w:commentRangeEnd w:id="0" />"#),
        "{xml}"
    );
}

/// Two adjacent tables alike in every whole-table property, the second
/// carrying a table grid change. Word's save holds such tables as one, but a
/// grid change left tracked keeps its own table.
#[test]
fn a_kept_grid_change_keeps_its_table() {
    let cell = |t: &str| {
        format!(r#"<w:tc><w:tcPr><w:tcW w:w="2000" w:type="dxa"/></w:tcPr><w:p>{t}</w:p></w:tc>"#)
    };
    let tbl = |grid_change: &str, t: &str| {
        format!(
            r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/>{grid_change}</w:tblGrid><w:tr>{}</w:tr></w:tbl>"#,
            cell(t)
        )
    };
    let pkg = docx(&format!(
        r#"{}{}<w:p/>"#,
        tbl(
            "",
            &format!(r#"<w:ins w:id="1" {A}><w:r><w:t>One</w:t></w:r></w:ins>"#)
        ),
        tbl(
            r#"<w:tblGridChange w:id="2"><w:tblGrid><w:gridCol w:w="1500"/></w:tblGrid></w:tblGridChange>"#,
            "<w:r><w:t>Two</w:t></w:r>"
        ),
    ));
    for out in [
        accept_changes(&pkg, &filter_ids(&["body:rev:1"])).unwrap(),
        reject_changes(&pkg, &filter_ids(&["body:rev:1"])).unwrap(),
    ] {
        assert_word_valid_package(&out);
        assert_eq!(ids(&out), ["body:rev:2"]);
    }
}

/// A moved paragraph's mark sits in its `pPr`, before the range start Word
/// writes after it: the mark is part of the move and resolves with it.
#[test]
fn a_moved_paragraph_mark_resolves_with_its_move() {
    let mark = |side: &str, id: u32| {
        format!(r#"<w:pPr><w:rPr><w:{side} w:id="{id}" {A}/></w:rPr></w:pPr>"#)
    };
    let pkg = docx(&format!(
        r#"<w:p>{}<w:moveFromRangeStart w:id="10" w:name="move1" {A}/><w:moveFrom w:id="11" {A}><w:r><w:t>Moved</w:t></w:r></w:moveFrom></w:p><w:p><w:moveFromRangeEnd w:id="10"/><w:r><w:t>Stays</w:t></w:r></w:p><w:p>{}<w:moveToRangeStart w:id="12" w:name="move1" {A}/><w:moveTo w:id="13" {A}><w:r><w:t>Moved</w:t></w:r></w:moveTo></w:p><w:p><w:moveToRangeEnd w:id="12"/><w:ins w:id="14" {B}><w:r><w:t>Tail</w:t></w:r></w:ins></w:p>"#,
        mark("moveFrom", 20),
        mark("moveTo", 21),
    ));
    let listed = list_changes(&pkg).unwrap();
    assert!(
        listed
            .iter()
            .filter(|c| c.kind == ChangeKind::Move)
            .all(|c| c.move_name.as_deref() == Some("move1")),
        "{listed:?}"
    );
    for resolve in [accept_changes, reject_changes] {
        for side in ["body:rev:20", "body:rev:21", "body:rev:13"] {
            let out = resolve(&pkg, &filter_ids(&[side])).unwrap();
            assert_word_valid_package(&out);
            assert_eq!(ids(&out), ["body:rev:14"], "resolving {side}");
        }
    }
}

/// A revision mark inside a formatting change's recorded properties is
/// history, not a change of its own; the paragraph's formatting changes go
/// with its mark when resolving the mark removes it.
#[test]
fn recorded_marks_are_history_and_mark_formatting_goes_with_the_mark() {
    let pkg = docx(&format!(
        r#"<w:p><w:pPr><w:jc w:val="both"/><w:rPr><w:del w:id="1" {A}/><w:sz w:val="22"/><w:rPrChange w:id="2" {A}><w:rPr><w:del w:id="3" {A}/><w:sz w:val="28"/></w:rPr></w:rPrChange></w:rPr><w:pPrChange w:id="4" {A}><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>Joined</w:t></w:r></w:p><w:p><w:r><w:t>Next</w:t></w:r></w:p>"#
    ));
    let listed = list_changes(&pkg).unwrap();
    let got: Vec<_> = listed
        .iter()
        .map(|c| (c.id.as_str(), c.inside.as_deref()))
        .collect();
    assert_eq!(
        got,
        [
            ("body:rev:1", None),
            ("body:rev:2", Some("body:rev:1")),
            ("body:rev:4", Some("body:rev:1")),
        ]
    );
    let out = accept_changes(&pkg, &filter_ids(&["body:rev:1"])).unwrap();
    assert_word_valid_package(&out);
    assert!(ids(&out).is_empty(), "{:?}", ids(&out));
    let out = reject_changes(&pkg, &filter_ids(&["body:rev:2"])).unwrap();
    assert_word_valid_package(&out);
    assert_eq!(ids(&out), ["body:rev:1", "body:rev:4"]);
}
