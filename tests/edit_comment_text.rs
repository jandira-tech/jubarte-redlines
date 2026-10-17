// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Editing the text a comment covers (Task 23): the comment keeps covering
//! both sides of a tracked replacement, so Accept All leaves it on the new
//! text and Reject All on the old.

mod common;

use common::docx::{docx, para, part_string};
use common::validity::assert_word_valid_package;
use jubarte::comments::list_comments;
use jubarte::document_comparer::{accept_revisions, reject_revisions};
use jubarte::edit::{EditPlan, apply_plan};

fn plan(json: &str) -> EditPlan {
    EditPlan::from_json(json).unwrap()
}

/// "The fee is ten dollars per hour." with comment 0 on exactly "ten
/// dollars".
fn commented() -> Vec<u8> {
    let source = docx(&para("The fee is ten dollars per hour."));
    apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
            {"kind":"comment","paragraph":"p0","find":"ten dollars","text":"Check the rate"}]}"#,
        ),
    )
    .unwrap()
    .clean
}

/// The anchor text of the only comment in `docx`.
fn covered(docx: &[u8]) -> String {
    let comments = list_comments(docx).unwrap();
    assert_eq!(comments.len(), 1, "{comments:?}");
    comments[0].anchor_text.clone()
}

/// The comment's range markers and the revisions between them, in document
/// order: `start`, `end`, `ref`, `del:TEXT`, `ins:TEXT`.
fn shape(docx: &[u8]) -> Vec<String> {
    let xml = part_string(docx, "word/document.xml").unwrap();
    let mut out = Vec::new();
    let mut rest = xml.as_str();
    while let Some(at) = rest.find('<') {
        rest = &rest[at..];
        let tag_end = rest.find('>').unwrap();
        let tag = &rest[..=tag_end];
        let name = tag
            .trim_start_matches(['<', '/'])
            .split([' ', '>', '/'])
            .next()
            .unwrap();
        let text_of = |close: &str| {
            let body = &rest[..rest.find(close).unwrap()];
            body.split("<w:t")
                .skip(1)
                .chain(body.split("<w:delText").skip(1))
                .filter_map(|t| t.split_once('>').map(|(_, v)| v.split('<').next().unwrap()))
                .collect::<String>()
        };
        match name {
            "w:commentRangeStart" => out.push("start".into()),
            "w:commentRangeEnd" => out.push("end".into()),
            "w:commentReference" => out.push("ref".into()),
            "w:del" if !tag.starts_with("</") => out.push(format!("del:{}", text_of("</w:del>"))),
            "w:ins" if !tag.starts_with("</") => out.push(format!("ins:{}", text_of("</w:ins>"))),
            _ => {}
        }
        rest = &rest[tag_end + 1..];
    }
    out
}

#[test]
fn replacing_the_whole_commented_text_keeps_the_deletion_inside_the_comment() {
    let source = commented();
    assert_eq!(covered(&source), "ten dollars");
    let edited = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Bob","date":"2026-10-12T00:00:00Z","operations":[
            {"kind":"replace","paragraph":"p0","find":"ten dollars","replacement":"twelve euros"}]}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&edited.redline);
    assert_eq!(
        shape(&edited.redline),
        ["start", "del:ten dollars", "ins:twelve euros", "end", "ref"],
        "Word's order: the deletion, then the insertion, both inside the range"
    );
    let accepted = accept_revisions(&edited.redline).unwrap();
    let rejected = reject_revisions(&edited.redline).unwrap();
    assert_word_valid_package(&accepted);
    assert_word_valid_package(&rejected);
    assert_eq!(covered(&accepted), "twelve euros");
    assert_eq!(covered(&rejected), "ten dollars");
}

/// Bob's plan of `ops` on the commented source.
fn bob(ops: &str) -> Result<jubarte::edit::EditResult, jubarte::edit::EditError> {
    apply_plan(
        &commented(),
        &plan(&format!(
            r#"{{"schema_version":1,"author":"Bob","date":"2026-10-12T00:00:00Z","operations":[{ops}]}}"#
        )),
    )
}

#[test]
fn rewriting_c0_text_replaces_what_the_comment_covers() {
    let edited = bob(r#"{"kind":"rewrite","paragraph":"c0.text","text":"twelve euros"}"#).unwrap();
    assert_word_valid_package(&edited.redline);
    assert_eq!(
        shape(&edited.redline),
        ["start", "del:ten dollars", "ins:twelve euros", "end", "ref"]
    );
    assert_eq!(covered(&edited.clean), "twelve euros");
    assert_eq!(
        covered(&accept_revisions(&edited.redline).unwrap()),
        "twelve euros"
    );
    assert_eq!(
        covered(&reject_revisions(&edited.redline).unwrap()),
        "ten dollars"
    );
    assert_eq!(edited.report.operations[0].paragraph.as_deref(), Some("p0"));
}

/// A plan that rewrites what a comment covers is emitted directly, and the
/// emitter once replayed only text and paragraph operations: a `format_run`
/// beside `c0.text` reached the clean copy but not the redline, reported
/// `ok` (ledger review of #451). It is a tracked format change now.
#[test]
fn a_format_run_beside_c0_text_is_tracked_in_the_redline() {
    let edited = bob(
        r#"{"kind":"rewrite","paragraph":"c0.text","text":"twelve euros"},
        {"kind":"format_run","paragraph":"p0","find":"per hour","format":{"italic":true}}"#,
    )
    .unwrap();
    assert_word_valid_package(&edited.redline);
    assert_eq!(
        shape(&edited.redline),
        ["start", "del:ten dollars", "ins:twelve euros", "end", "ref"]
    );
    assert_eq!(edited.report.revisions.format_changed, 1);
    let italic = |docx: &[u8]| {
        part_string(docx, "word/document.xml")
            .unwrap()
            .contains("<w:i />")
    };
    assert!(italic(&edited.clean));
    let accepted = accept_revisions(&edited.redline).unwrap();
    let rejected = reject_revisions(&edited.redline).unwrap();
    assert!(italic(&accepted));
    assert!(!italic(&rejected));
    assert_eq!(covered(&rejected), "ten dollars");
}

/// An operation the emitter cannot track (a footnote) sends the plan
/// through the comparer, which writes it, instead of dropping it.
#[test]
fn a_footnote_beside_c0_text_reaches_the_redline() {
    let edited = bob(
        r#"{"kind":"rewrite","paragraph":"c0.text","text":"twelve euros"},
        {"kind":"insert_footnote","paragraph":"p0","after":"per hour","text":"Why."}"#,
    )
    .unwrap();
    assert_word_valid_package(&edited.redline);
    let red = part_string(&edited.redline, "word/document.xml").unwrap();
    assert!(red.contains("<w:footnoteReference "), "{red}");
    assert_eq!(
        covered(&accept_revisions(&edited.redline).unwrap()),
        "twelve euros"
    );
}

/// The comment keeps both sides of a whole-range replacement whatever else
/// the plan holds: paragraph formatting, or `whole` on the edit (ledger
/// review of #451, finding 2). The comparer would put the deletion after
/// the range, so Reject All left the comment on nothing.
#[test]
fn a_whole_range_replacement_keeps_the_comment_in_mixed_plans() {
    for ops in [
        r#"{"kind":"replace","paragraph":"p0","find":"ten dollars","replacement":"twelve euros"},
        {"kind":"format_paragraph","paragraph":"p0","alignment":"center"}"#,
        r#"{"kind":"replace","paragraph":"p0","find":"ten dollars","replacement":"twelve euros","whole":true}"#,
    ] {
        let edited = bob(ops).unwrap();
        assert_word_valid_package(&edited.redline);
        assert_eq!(
            shape(&edited.redline),
            ["start", "del:ten dollars", "ins:twelve euros", "end", "ref"],
            "{ops}"
        );
        assert_eq!(
            covered(&reject_revisions(&edited.redline).unwrap()),
            "ten dollars",
            "{ops}"
        );
    }
}

/// `update_fields` refreshes the clean copy only, so the emitted redline
/// would not accept to it: refused, as under keep, and both refusals name
/// the command that refreshes fields now (`jubarte fields update` is gone).
#[test]
fn update_fields_beside_a_whole_range_replacement_is_refused() {
    let commented_plan = |extra: &str| {
        plan(&format!(
            r#"{{"schema_version":1,"author":"Bob",{extra}"operations":[
            {{"kind":"replace","paragraph":"p0","find":"ten dollars","replacement":"twelve euros"}}]}}"#
        ))
    };
    for extra in [
        r#""update_fields":true,"#,
        r#""update_fields":true,"existing_revisions":"keep","#,
    ] {
        let e = apply_plan(&commented(), &commented_plan(extra)).unwrap_err();
        assert_eq!(e.code, "INVALID_PLAN", "{extra}: {e:?}");
        assert!(
            e.message.contains("convert") && e.message.contains("--update-fields"),
            "{extra}: {}",
            e.message
        );
        assert!(!e.message.contains("fields update"), "{}", e.message);
        let e = jubarte::edit::preview_plan(&commented(), &commented_plan(extra)).unwrap_err();
        assert_eq!(e.code, "INVALID_PLAN", "preview refuses too: {extra}");
    }
}

#[test]
fn an_anchor_in_c0_text_is_looked_up_in_the_covered_text_only() {
    let edited =
        bob(r#"{"kind":"replace","paragraph":"c0.text","find":"ten","replacement":"twelve"}"#)
            .unwrap();
    assert_word_valid_package(&edited.redline);
    assert_eq!(covered(&edited.clean), "twelve dollars");
    let refused =
        bob(r#"{"kind":"replace","paragraph":"c0.text","find":"fee","replacement":"rate"}"#)
            .unwrap_err();
    assert_eq!(refused.code, "ANCHOR_NOT_FOUND");
    assert!(
        refused.message.contains("not in the text c0 covers"),
        "{}",
        refused.message
    );
}

#[test]
fn a_repeated_anchor_is_unique_inside_c0_text() {
    let source = docx(&para("ten dollars or ten euros"));
    let source = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
            {"kind":"comment","paragraph":"p0","find":"ten euros","text":"Which?"}]}"#,
        ),
    )
    .unwrap()
    .clean;
    let edited = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Bob","operations":[
            {"kind":"replace","paragraph":"c0.text","find":"ten","replacement":"nine"}]}"#,
        ),
    )
    .unwrap();
    assert_eq!(covered(&edited.clean), "nine euros");
    assert!(
        jubarte::inspect::paragraphs(&edited.clean).unwrap()[0]
            .text
            .starts_with("ten dollars or nine"),
    );
}

#[test]
fn an_unknown_comment_text_is_refused() {
    let refused = bob(r#"{"kind":"rewrite","paragraph":"c7.text","text":"x"}"#).unwrap_err();
    assert_eq!(refused.code, "ANCHOR_NOT_FOUND");
    assert!(refused.message.contains("c7"), "{}", refused.message);
}

fn jubarte(args: &[&str], dir: &std::path::Path) -> std::process::Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

#[test]
fn the_cli_edits_c0_text_and_c0_keeps_meaning_the_comment() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("src.docx"), commented()).unwrap();
    let whole = jubarte(
        &[
            "edit",
            "src.docx",
            "-p",
            "c0.text",
            "--content",
            "twelve euros",
            "--datetime",
            "1970-01-01T00:00:00Z",
            "--out-dir",
            "whole",
        ],
        dir.path(),
    );
    assert!(
        whole.status.success(),
        "{}",
        String::from_utf8_lossy(&whole.stderr)
    );
    let redline = std::fs::read(dir.path().join("whole/redline.docx")).unwrap();
    assert_eq!(
        shape(&redline),
        ["start", "del:ten dollars", "ins:twelve euros", "end", "ref"]
    );
    let outside = jubarte(
        &[
            "edit",
            "src.docx",
            "-p",
            "c0.text",
            "--anchor",
            "fee",
            "--content",
            "rate",
            "--out-dir",
            "outside",
        ],
        dir.path(),
    );
    assert_eq!(outside.status.code(), Some(3), "{outside:?}");
    assert!(
        String::from_utf8_lossy(&outside.stderr).contains("not in the text c0 covers"),
        "{}",
        String::from_utf8_lossy(&outside.stderr)
    );
    // `-p c0 --content` still rewrites the comment's own text.
    let note = jubarte(
        &[
            "edit",
            "src.docx",
            "-p",
            "c0",
            "--content",
            "Rate checked",
            "--out-dir",
            "note",
        ],
        dir.path(),
    );
    assert!(
        note.status.success(),
        "{}",
        String::from_utf8_lossy(&note.stderr)
    );
    let clean = std::fs::read(dir.path().join("note/clean.docx")).unwrap();
    let comments = list_comments(&clean).unwrap();
    assert_eq!(comments[0].text, "Rate checked");
    assert_eq!(comments[0].anchor_text, "ten dollars");
}

/// "Alpha beta" / "gamma delta", with comment 0 from "beta" through "gamma".
fn spanning() -> Vec<u8> {
    let source = docx(&format!(
        "{}{}",
        para("Alpha beta"),
        r#"<w:p><w:r><w:t>gamma</w:t></w:r><w:r><w:t xml:space="preserve"> delta</w:t></w:r></w:p>"#
    ));
    let one = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Ann","operations":[
            {"kind":"comment","paragraph":"p0","find":"beta","text":"Span"}]}"#,
        ),
    )
    .unwrap()
    .clean;
    let xml = part_string(&one, "word/document.xml").unwrap();
    let end = xml.find("<w:commentRangeEnd").unwrap();
    let end_tag = &xml[end..end + xml[end..].find("/>").unwrap() + 2];
    let moved = xml.replacen(end_tag, "", 1);
    let gamma = moved.find("gamma</w:t></w:r>").unwrap() + "gamma</w:t></w:r>".len();
    let moved = format!("{}{end_tag}{}", &moved[..gamma], &moved[gamma..]);
    common::docx::replace_entry(&one, "word/document.xml", moved.as_bytes())
}

#[test]
fn rewriting_c0_text_over_two_paragraphs_deletes_the_mark_between() {
    let source = spanning();
    assert_eq!(covered(&source), "beta\ngamma");
    let edited = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Bob","operations":[
            {"kind":"rewrite","paragraph":"c0.text","text":"new"}]}"#,
        ),
    )
    .unwrap();
    assert_word_valid_package(&edited.redline);
    assert_word_valid_package(&edited.clean);
    let xml = part_string(&edited.redline, "word/document.xml").unwrap();
    // The mark between is the first paragraph's (the one holding "beta");
    // the last paragraph keeps its mark.
    let paragraphs: Vec<&str> = xml.split("</w:p>").collect();
    let holding = |text: &str| {
        paragraphs
            .iter()
            .find(|p| p.contains(text))
            .unwrap_or_else(|| panic!("no paragraph holds {text}: {xml}"))
    };
    assert!(
        holding(">beta<").contains("<w:pPr><w:rPr><w:del "),
        "the mark between is deleted: {xml}"
    );
    assert!(
        !holding(" delta<").contains("<w:rPr><w:del "),
        "the last mark stays: {xml}"
    );
    let order: Vec<String> = shape(&edited.redline)
        .into_iter()
        .filter(|s| s != "ref" && !s.starts_with("del:Alpha"))
        .collect();
    assert_eq!(order, ["start", "del:beta", "ins:new", "del:gamma", "end"]);
    let texts = |docx: &[u8]| -> Vec<String> {
        jubarte::inspect::paragraphs(docx)
            .unwrap()
            .into_iter()
            .map(|p| p.text)
            .collect()
    };
    assert_eq!(texts(&edited.clean), ["Alpha new delta"]);
    assert_eq!(covered(&edited.clean), "new");
    let accepted = accept_revisions(&edited.redline).unwrap();
    let rejected = reject_revisions(&edited.redline).unwrap();
    assert_word_valid_package(&accepted);
    assert_word_valid_package(&rejected);
    assert_eq!(texts(&accepted), ["Alpha new delta"]);
    assert_eq!(covered(&accepted), "new");
    assert_eq!(texts(&rejected), ["Alpha beta", "gamma delta"]);
    assert_eq!(covered(&rejected), "beta\ngamma");
    // An anchor cannot say which of the two paragraphs it means.
    let refused = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Bob","operations":[
            {"kind":"replace","paragraph":"c0.text","find":"beta","replacement":"x"}]}"#,
        ),
    )
    .unwrap_err();
    assert_eq!(refused.code, "INVALID_EDIT");
    assert!(
        refused.message.contains("more than one paragraph"),
        "{}",
        refused.message
    );
}

#[test]
fn deleting_c0_text_removes_only_what_the_comment_covers() {
    let edited = bob(r#"{"kind":"delete_paragraph","paragraph":"c0.text"}"#).unwrap();
    assert_word_valid_package(&edited.redline);
    assert_eq!(
        jubarte::inspect::paragraphs(&edited.clean).unwrap()[0].text,
        "The fee is  per hour."
    );
    assert_eq!(
        covered(&reject_revisions(&edited.redline).unwrap()),
        "ten dollars"
    );
    let refused = bob(r#"{"kind":"insert_paragraph","paragraph":"c0.text","runs":[{"text":"x"}]}"#)
        .unwrap_err();
    assert!(
        refused.message.contains("names text, not a paragraph"),
        "{}",
        refused.message
    );
}

#[test]
fn a_point_comment_covers_no_text_to_rewrite() {
    let source = std::fs::read("tests/fixtures/agent-view/received.docx").unwrap();
    let refused = apply_plan(
        &source,
        &plan(
            r#"{"schema_version":1,"author":"Bob","existing_revisions":"keep","operations":[
            {"kind":"rewrite","paragraph":"c11.text","text":"private"}]}"#,
        ),
    )
    .unwrap_err();
    assert_eq!(refused.code, "INVALID_EDIT");
    assert!(
        refused
            .message
            .contains("c11 covers no text; edit its paragraph (p18) instead"),
        "{}",
        refused.message
    );
}

/// A settled view numbers each note's paragraphs as the settled story does,
/// so the printed id is the one `edit -p` takes on the settled document.
#[test]
fn note_definitions_carry_settled_ids_in_settled_views() {
    let dir = tempfile::tempdir().unwrap();
    let source = "tests/fixtures/from-docx/critic/synthetic/hard-notes.docx";
    std::fs::copy(source, dir.path().join("notes.docx")).unwrap();
    let ids_of = |args: &[&str]| -> Vec<String> {
        let out = jubarte(args, dir.path());
        assert!(
            out.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout)
            .unwrap()
            .lines()
            .filter(|line| {
                line.starts_with("<!-- footnotes.") || line.starts_with("<!-- endnotes.")
            })
            .map(str::to_string)
            .collect()
    };
    for choice in ["accept", "reject"] {
        let settled = format!("settled-{choice}.docx");
        let done = jubarte(&[choice, "notes.docx", "-o", &settled], dir.path());
        assert!(
            done.status.success(),
            "{choice}: {}",
            String::from_utf8_lossy(&done.stderr)
        );
        assert_eq!(
            ids_of(&["read", "notes.docx", "--track-changes", choice]),
            ids_of(&["read", &settled]),
            "{choice}: the settled view and the settled file agree"
        );
    }
    // The printed id is actionable: the rejected file's "Still here" is
    // footnotes.p1 in its own view.
    let edited = jubarte(
        &[
            "edit",
            "settled-reject.docx",
            "-p",
            "footnotes.p1",
            "--anchor",
            "Still",
            "--content",
            "STILL",
            "-o",
            "edited.docx",
            "--force",
        ],
        dir.path(),
    );
    assert!(
        edited.status.success(),
        "{}",
        String::from_utf8_lossy(&edited.stderr)
    );
    let view = ids_of(&["read", "edited.docx"]);
    assert!(
        view.contains(&"<!-- footnotes.p1 -->".to_string()),
        "{view:?}"
    );
}

/// `read` names each note's paragraphs on the line before its definition,
/// numbered as `edit -p` takes them (separator notes are not numbered).
#[test]
fn note_definitions_carry_the_ids_edit_takes() {
    let dir = tempfile::tempdir().unwrap();
    let source = "tests/fixtures/from-docx/critic/synthetic/hard-notes.docx";
    std::fs::copy(source, dir.path().join("notes.docx")).unwrap();
    let read = jubarte(&["read", "notes.docx"], dir.path());
    assert!(read.status.success());
    let view = String::from_utf8(read.stdout).unwrap();
    // The id lines, whatever a definition's own line count.
    assert_eq!(
        view.lines()
            .filter(|line| line.starts_with("<!-- footnotes.") || line.starts_with("<!-- endnotes."))
            .collect::<Vec<_>>(),
        [
            "<!-- footnotes.p0 -->",
            "<!-- footnotes.p1 -->",
            "<!-- footnotes.p2 -->",
            "<!-- endnotes.p0-p2 -->",
            "<!-- footnotes.p3 -->",
        ],
        "{view}"
    );
    for (id, anchor) in [
        ("footnotes.p0", "2024"),
        ("footnotes.p2", "Still"),
        ("endnotes.p2", "joined"),
        ("footnotes.p3", "Inside"),
    ] {
        let out = jubarte(
            &[
                "edit",
                "notes.docx",
                "-p",
                id,
                "--anchor",
                anchor,
                "--content",
                "X",
                "--existing-revisions",
                "keep",
                "-o",
                "out.docx",
                "--force",
            ],
            dir.path(),
        );
        assert!(
            out.status.success(),
            "{id}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    // Plain Markdown (convert --commonmark) prints no id lines.
    let md = jubarte(
        &["convert", "notes.docx", "-o", "n.md", "--commonmark"],
        dir.path(),
    );
    assert!(
        md.status.success(),
        "{}",
        String::from_utf8_lossy(&md.stderr)
    );
    let text = std::fs::read_to_string(dir.path().join("n.md")).unwrap();
    assert!(text.contains("[^3]: Still here."), "{text}");
    assert!(!text.contains("<!-- footnotes"), "{text}");
}

#[test]
fn settled_note_numbering_ignores_references_removed_from_headers() {
    use common::docx::{Part, W_NS, docx_with_sect};
    use jubarte::markdown::{ReadOptions, TrackChanges, read};
    for kind in ["footnote", "endnote"] {
        let notes = format!(
            r#"<w:{kind}s xmlns:w="{W_NS}"><w:{kind} w:id="1"><w:p><w:r><w:t>Header note.</w:t></w:r></w:p></w:{kind}><w:{kind} w:id="2"><w:p><w:r><w:t>Body note.</w:t></w:r></w:p></w:{kind}></w:{kind}s>"#
        );
        let header = format!(
            r#"<w:hdr xmlns:w="{W_NS}"><w:p><w:ins w:id="9" w:author="Ann"><w:r><w:{kind}Reference w:id="1"/></w:r></w:ins></w:p></w:hdr>"#
        );
        let note_part = format!("word/{kind}s.xml");
        let content_type =
            format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}s+xml");
        let rel_type =
            format!("http://schemas.openxmlformats.org/officeDocument/2006/relationships/{kind}s");
        let source = docx_with_sect(
            &format!(
                r#"<w:p><w:r><w:t>Body.</w:t></w:r><w:r><w:{kind}Reference w:id="2"/></w:r></w:p>"#
            ),
            &[
                Part {
                    name: &note_part,
                    content_type: &content_type,
                    rel_type: &rel_type,
                    xml: &notes,
                },
                Part {
                    name: "word/header1.xml",
                    content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                    rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                    xml: &header,
                },
            ],
            r#"<w:headerReference w:type="default" r:id="rIdX1"/>"#,
        );
        let ids = |bytes: &[u8], track_changes| -> Vec<String> {
            read(
                bytes,
                &ReadOptions {
                    page_markers: false,
                    track_changes,
                    ..ReadOptions::default()
                },
            )
            .unwrap()
            .markdown
            .lines()
            .filter(|line| line.starts_with(&format!("<!-- {kind}s.")))
            .map(str::to_string)
            .collect()
        };
        for track in [TrackChanges::Accept, TrackChanges::Reject] {
            let settled = match track {
                TrackChanges::Accept => accept_revisions(&source).unwrap(),
                _ => reject_revisions(&source).unwrap(),
            };
            let expected = ids(&settled, TrackChanges::All);
            assert!(!expected.is_empty());
            assert_eq!(ids(&source, track), expected, "{kind} {track:?}");
        }
    }
}
