// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `validate OUT --baseline IN`: the findings IN already had print as not
//! the editor's, and only new ones fail the run; a finding is new by
//! identity, never by count. `read`, `changes` and `comments` name the
//! input's validity findings in one stderr line, which `--quiet` drops.

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::{Part, docx, docx_with, para};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");
const W14: &str = "http://schemas.microsoft.com/office/word/2010/wordml";
/// A Word-fatal shape: text inside a deletion as `w:t`.
const DELETED: &str = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#;
/// A paragraph a comment covers, then a plain one.
const COMMENTED: &str = r#"<w:p><w:commentRangeStart w:id="0"/><w:r><w:t>Hello world.</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p><w:p><w:r><w:t>Second paragraph.</w:t></w:r></w:p>"#;

fn run(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(BIN)
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// One comment whose `commentsExtensible` entry names `durable` where
/// `commentsIds` says `10000001`: two `COMMENT_METADATA_MISMATCH` warnings
/// (`durable` matches no comment paragraph, and `10000001` is missing),
/// which Word opens without a prompt.
fn comment_graph(durable: &str) -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{}" xmlns:w14="{W14}"><w:comment w:id="0" w:author="A" w:date="2026-01-01T00:00:00Z"><w:p w14:paraId="11111111"><w:r><w:t>note</w:t></w:r></w:p></w:comment></w:comments>"#,
        common::docx::W_NS
    );
    let extended = r#"<w15:commentsEx xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml"><w15:commentEx w15:paraId="11111111"/></w15:commentsEx>"#;
    let ids = r#"<w16cid:commentsIds xmlns:w16cid="http://schemas.microsoft.com/office/word/2016/wordml/cid"><w16cid:commentId w16cid:paraId="11111111" w16cid:durableId="10000001"/></w16cid:commentsIds>"#;
    let extensible = format!(
        r#"<w16cex:commentsExtensible xmlns:w16cex="http://schemas.microsoft.com/office/word/2018/wordml/cex"><w16cex:commentExtensible w16cex:durableId="{durable}"/></w16cex:commentsExtensible>"#
    );
    docx_with(
        COMMENTED,
        &[
            Part {
                name: "word/comments.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
                xml: &comments,
            },
            Part {
                name: "word/commentsExtended.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtended+xml",
                rel_type: "http://schemas.microsoft.com/office/2011/relationships/commentsExtended",
                xml: extended,
            },
            Part {
                name: "word/commentsIds.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsIds+xml",
                rel_type: "http://schemas.microsoft.com/office/2016/09/relationships/commentsIds",
                xml: ids,
            },
            Part {
                name: "word/commentsExtensible.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.commentsExtensible+xml",
                rel_type: "http://schemas.microsoft.com/office/2018/08/relationships/commentsExtensible",
                xml: &extensible,
            },
        ],
    )
}

fn unmatched(durable: &str) -> String {
    format!(
        "  COMMENT_METADATA_MISMATCH\twarning\tword/commentsExtensible.xml#\t\
         commentsExtensible durableId '{durable}' has no matching comment paragraph\n"
    )
}

const MISSING: &str = "  COMMENT_METADATA_MISMATCH\twarning\tword/commentsExtensible.xml#\t\
     commentsExtensible durableId is missing '10000001'\n";

/// `in.docx` with the two inherited warnings, and `out.docx`, jubarte's
/// tracked edit of it.
fn edited(dir: &Path) {
    std::fs::write(dir.join("in.docx"), comment_graph("20000002")).unwrap();
    let (code, _, err) = run(
        dir,
        &[
            "edit", "replace", "in.docx", "--find", "Second", "--text", "Third", "-o", "out.docx",
            "--author", "Claude", "--force",
        ],
    );
    assert_eq!(code, 0, "{err}");
}

fn json_lines(stdout: &str) -> Vec<serde_json::Value> {
    stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|e| panic!("{line}: {e}")))
        .collect()
}

#[test]
fn baseline_hides_the_inputs_findings() {
    let dir = tempfile::tempdir().unwrap();
    edited(dir.path());
    // Without a baseline the inherited warnings fail the edited file.
    let (code, out, _) = run(dir.path(), &["validate", "out.docx"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("2 finding(s)"), "{out}");
    assert!(out.contains(MISSING), "{out}");
    assert!(out.contains(&unmatched("20000002")), "{out}");

    let (code, out, err) = run(
        dir.path(),
        &["validate", "out.docx", "--baseline", "in.docx"],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(out, "new: none\nalready in the input: 2 (not yours)\n");
    assert_eq!(err, "");
}

#[test]
fn baseline_reports_a_new_finding_even_at_the_same_count() {
    let dir = tempfile::tempdir().unwrap();
    // Two findings each: '10000001' is missing from both; '20000002' is
    // gone from the output and '30000003' is new in it.
    std::fs::write(dir.path().join("in.docx"), comment_graph("20000002")).unwrap();
    std::fs::write(dir.path().join("out.docx"), comment_graph("30000003")).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &["validate", "out.docx", "--baseline", "in.docx"],
    );
    assert_eq!(code, 2, "{out}{err}");
    assert_eq!(
        out,
        format!(
            "new: 1\n{}already in the input: 1 (not yours)\n\
             resolved: 1 (in the input, not in this file)\n{}",
            unmatched("30000003"),
            unmatched("20000002")
        )
    );
}

#[test]
fn baseline_json_marks_status() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), comment_graph("20000002")).unwrap();
    std::fs::write(dir.path().join("out.docx"), comment_graph("30000003")).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &["validate", "out.docx", "--baseline", "in.docx", "--json"],
    );
    assert_eq!(code, 2, "{out}{err}");
    let rows = json_lines(&out);
    assert_eq!(rows.len(), 3, "{out}");
    assert_eq!(rows[0]["status"], "new");
    assert!(rows[0]["message"].as_str().unwrap().contains("'30000003'"));
    assert_eq!(rows[1]["status"], "baseline");
    assert!(rows[1]["message"].as_str().unwrap().contains("'10000001'"));
    // The finding keeps every field it has without a baseline.
    assert_eq!(rows[1]["code"], "COMMENT_METADATA_MISMATCH");
    assert_eq!(rows[1]["part"], "word/commentsExtensible.xml");
    assert_eq!(rows[1]["word_fatal"], false);
    let resolved = rows[2]["resolved"]
        .as_array()
        .expect("trailing resolved line");
    assert_eq!(resolved.len(), 1, "{out}");
    assert!(
        resolved[0]["message"]
            .as_str()
            .unwrap()
            .contains("'20000002'")
    );
    assert!(resolved[0].get("status").is_none(), "{out}");

    // Nothing new and nothing resolved: every line is the baseline's, and
    // the trailing line is still there.
    edited(dir.path());
    let (code, out, _) = run(
        dir.path(),
        &["validate", "out.docx", "--baseline", "in.docx", "--json"],
    );
    assert_eq!(code, 0, "{out}");
    let rows = json_lines(&out);
    assert_eq!(rows.len(), 3, "{out}");
    assert!(rows[..2].iter().all(|row| row["status"] == "baseline"));
    assert_eq!(rows[2], serde_json::json!({"resolved": []}));
}

#[test]
fn baseline_fails_only_on_new_findings_under_strict_too() {
    let dir = tempfile::tempdir().unwrap();
    edited(dir.path());
    let (code, out, _) = run(
        dir.path(),
        &["validate", "out.docx", "--baseline", "in.docx", "--strict"],
    );
    assert_eq!(code, 0, "{out}");
    // A new Word-fatal error is listed with its star and fails the run.
    std::fs::write(
        dir.path().join("broken.docx"),
        docx(&format!("{DELETED}{}", para("x"))),
    )
    .unwrap();
    std::fs::write(dir.path().join("clean.docx"), docx(&para("x"))).unwrap();
    let (code, out, _) = run(
        dir.path(),
        &["validate", "broken.docx", "--baseline", "clean.docx"],
    );
    assert_eq!(code, 2, "{out}");
    assert!(
        out.starts_with("new: 1\n* TEXT_INSIDE_DELETION\terror\t"),
        "{out}"
    );
    assert!(out.ends_with("already in the input: none\n"), "{out}");
}

#[test]
fn author_with_baseline_checks_the_edits_against_it() {
    let dir = tempfile::tempdir().unwrap();
    edited(dir.path());
    // The edit is Claude's tracked replacement: nothing new.
    let (code, out, err) = run(
        dir.path(),
        &[
            "validate",
            "out.docx",
            "--baseline",
            "in.docx",
            "--author",
            "Claude",
        ],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(out, "new: none\nalready in the input: 2 (not yours)\n");
    // Under another author's name the same edit is foreign, and new.
    let (code, out, _) = run(
        dir.path(),
        &[
            "validate",
            "out.docx",
            "--baseline",
            "in.docx",
            "--author",
            "Legal",
        ],
    );
    assert_eq!(code, 2, "{out}");
    assert!(out.starts_with("new: "), "{out}");
    assert!(out.contains("FOREIGN_AUTHOR"), "{out}");
    assert!(out.contains("UNTRACKED_EDIT"), "{out}");
}

#[test]
fn baseline_usage_errors_exit_two() {
    let dir = tempfile::tempdir().unwrap();
    edited(dir.path());
    let (code, _, err) = run(
        dir.path(),
        &[
            "validate",
            "out.docx",
            "--baseline",
            "in.docx",
            "--repair",
            "fixed.docx",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("'--baseline <FILE>' cannot be used with '--repair <FILE>'"),
        "{err}"
    );
    assert!(!dir.path().join("fixed.docx").exists());
    // The edits rule still needs an author.
    let (code, _, err) = run(
        dir.path(),
        &[
            "validate",
            "out.docx",
            "--baseline",
            "in.docx",
            "--rules",
            "edits",
        ],
    );
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("rules edits needs --author, and --original or --baseline"),
        "{err}"
    );
    // A baseline that cannot be read is a runtime failure.
    let (code, _, err) = run(
        dir.path(),
        &["validate", "out.docx", "--baseline", "missing.docx"],
    );
    assert_eq!(code, 1, "{err}");
    assert!(err.contains("missing.docx"), "{err}");
}

#[test]
fn read_prints_warnings_once() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), comment_graph("20000002")).unwrap();
    let line = "input has 2 validity warnings; jubarte validate in.docx lists them\n";
    for args in [
        &["read", "in.docx", "--no-page-markers"][..],
        &["in.docx", "--no-page-markers"],
        &["changes", "in.docx"],
        &["comments", "in.docx"],
    ] {
        let (code, out, err) = run(dir.path(), args);
        assert_eq!(code, 0, "{args:?}: {err}");
        assert_eq!(err, line, "{args:?}");
        assert!(!out.contains("validity"), "{args:?}: {out}");
    }
    // One error, no warnings: the line counts what there is.
    std::fs::write(
        dir.path().join("broken.docx"),
        docx(&format!("{DELETED}{}", para("x"))),
    )
    .unwrap();
    let (_, _, err) = run(dir.path(), &["changes", "broken.docx"]);
    assert_eq!(
        err,
        "input has 1 validity error; jubarte validate broken.docx lists it\n"
    );
}

#[test]
fn quiet_drops_it() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), comment_graph("20000002")).unwrap();
    std::fs::write(dir.path().join("clean.docx"), docx(&para("x"))).unwrap();
    for args in [
        &["-q", "read", "in.docx", "--no-page-markers"][..],
        &["read", "in.docx", "--no-page-markers", "--quiet"],
        &["-q", "changes", "in.docx"],
        &["comments", "in.docx", "-q"],
        // A clean input has nothing to say.
        &["read", "clean.docx", "--no-page-markers"],
        &["changes", "clean.docx"],
        &["comments", "clean.docx"],
    ] {
        let (code, _, err) = run(dir.path(), args);
        assert_eq!(code, 0, "{args:?}: {err}");
        assert_eq!(err, "", "{args:?}");
    }
}

/// Snapshot validity remains data; JSON Lines validity is a quietable stderr status.
fn json_validity(dir: &Path, file: &str, quiet: bool) -> [serde_json::Value; 3] {
    let parse = |line: &str| -> serde_json::Value {
        serde_json::from_str(line).unwrap_or_else(|e| panic!("{e}: {line}"))
    };
    let mut reads = Vec::new();
    for task in ["read", "changes", "comments"] {
        let mut args = vec![task, file, "--json"];
        if quiet {
            args.push("-q");
        }
        let (code, out, err) = run(dir, &args);
        assert_eq!(code, 0, "{args:?}: {err}");
        let lines: Vec<&str> = out.lines().collect();
        reads.push(if task == "read" {
            assert_eq!(err, "", "{args:?}");
            assert_eq!(lines.len(), 1, "{out}");
            parse(lines[0])["summary"]["input_validity"].clone()
        } else {
            let fields = if task == "changes" {
                ["kind", "id", "paragraph"]
            } else {
                ["text", "ref", "author"]
            };
            for line in lines {
                let record = parse(line);
                for field in fields {
                    assert!(
                        record.get(field).is_some(),
                        "{task}: missing {field}: {record}"
                    );
                }
            }
            if quiet {
                assert_eq!(err, "", "{args:?}");
                serde_json::Value::Null
            } else {
                assert_eq!(err.lines().count(), 1, "{args:?}: {err}");
                let status = parse(err.trim_end());
                assert_eq!(status.as_object().unwrap().len(), 1, "{status}");
                status["input_validity"].clone()
            }
        });
    }
    reads.try_into().unwrap()
}

#[test]
fn json_reads_carry_the_input_validity() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), comment_graph("20000002")).unwrap();
    let wanted = serde_json::json!({"errors":0,"warnings":2,"next":"jubarte validate in.docx"});
    for quiet in [false, true] {
        assert_eq!(
            json_validity(dir.path(), "in.docx", quiet),
            if quiet {
                [
                    wanted.clone(),
                    serde_json::Value::Null,
                    serde_json::Value::Null,
                ]
            } else {
                [wanted.clone(), wanted.clone(), wanted.clone()]
            },
            "quiet {quiet}"
        );
    }
    // The comment is still present on stdout; status is on stderr.
    let (_, out, _) = run(dir.path(), &["comments", "in.docx", "--json"]);
    assert_eq!(out.lines().count(), 1, "{out}");
    // A path is spelled for a shell, as the stderr line spells it.
    std::fs::write(dir.path().join("my file.docx"), comment_graph("20000002")).unwrap();
    let [read, ..] = json_validity(dir.path(), "my file.docx", false);
    assert_eq!(read["next"], "jubarte validate 'my file.docx'");
    // An error counts as an error.
    std::fs::write(
        dir.path().join("broken.docx"),
        docx(&format!("{DELETED}{}", para("x"))),
    )
    .unwrap();
    let [read, changes, _] = json_validity(dir.path(), "broken.docx", false);
    assert_eq!(
        read,
        serde_json::json!({"errors":1,"warnings":0,"next":"jubarte validate broken.docx"})
    );
    assert_eq!(changes, read);
}

/// `read -o FILE.json` writes the bytes `--json` prints, the validity
/// included; stdout stays empty, so the stderr notice stays too.
#[test]
fn a_json_file_carries_the_validity_and_keeps_the_notice() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), comment_graph("20000002")).unwrap();
    let (_, printed, _) = run(dir.path(), &["read", "in.docx", "--json"]);
    let (code, out, err) = run(dir.path(), &["read", "in.docx", "-o", "s.json"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(out, "");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("s.json")).unwrap(),
        printed
    );
    assert!(
        err.starts_with("input has 2 validity warnings; jubarte validate in.docx lists them\n"),
        "{err}"
    );
    let (code, _, err) = run(dir.path(), &["read", "in.docx", "-o", "q.json", "-q"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(err, "");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("q.json")).unwrap(),
        printed
    );
}

#[test]
fn a_clean_input_reads_zero_findings() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("clean.docx"), docx(&para("x"))).unwrap();
    let zero = serde_json::json!({"errors":0,"warnings":0});
    assert_eq!(
        json_validity(dir.path(), "clean.docx", false),
        [zero.clone(), zero.clone(), zero]
    );
    // No changes means no stdout records.
    let (_, out, _) = run(dir.path(), &["changes", "clean.docx", "--json"]);
    assert_eq!(out, "");
}

/// `read --for-edit` prints JSON, so it is a JSON read too: its result
/// carries the input's validity and it prints no stderr line, `-q` or
/// not. `-o FILE.json` writes the bytes it prints and keeps the notice,
/// as `read -o FILE.json` does. The library's result has no such field;
/// only the CLI read the input (#1340).
#[test]
fn the_for_edit_read_carries_the_input_validity() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), comment_graph("20000002")).unwrap();
    std::fs::write(dir.path().join("clean.docx"), docx(&para("x"))).unwrap();
    let for_edit = |file: &str, quiet: bool| {
        let mut args = vec!["read", file, "--for-edit"];
        if quiet {
            args.push("-q");
        }
        let (code, out, err) = run(dir.path(), &args);
        assert_eq!(code, 0, "{args:?}: {err}");
        assert_eq!(err, "", "{args:?}");
        let json: serde_json::Value =
            serde_json::from_str(&out).unwrap_or_else(|e| panic!("{e}: {out}"));
        (out, json["input_validity"].clone())
    };
    let wanted = serde_json::json!({"errors":0,"warnings":2,"next":"jubarte validate in.docx"});
    for quiet in [false, true] {
        assert_eq!(for_edit("in.docx", quiet).1, wanted, "quiet {quiet}");
    }
    assert_eq!(
        for_edit("clean.docx", false).1,
        serde_json::json!({"errors":0,"warnings":0})
    );
    let (printed, _) = for_edit("in.docx", false);
    let (code, out, err) = run(
        dir.path(),
        &["read", "in.docx", "--for-edit", "-o", "e.json"],
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(out, "");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("e.json")).unwrap(),
        printed
    );
    assert!(
        err.starts_with("input has 2 validity warnings; jubarte validate in.docx lists them\n"),
        "{err}"
    );
    // -q drops the notice from a file too; the file is unchanged.
    let (code, _, err) = run(
        dir.path(),
        &["read", "in.docx", "--for-edit", "-o", "q.json", "-q"],
    );
    assert_eq!(code, 0, "{err}");
    assert_eq!(err, "");
    assert_eq!(
        std::fs::read_to_string(dir.path().join("q.json")).unwrap(),
        printed
    );
    // An error counts as an error.
    std::fs::write(
        dir.path().join("broken.docx"),
        docx(&format!("{DELETED}{}", para("x"))),
    )
    .unwrap();
    assert_eq!(
        for_edit("broken.docx", false).1,
        serde_json::json!({"errors":1,"warnings":0,"next":"jubarte validate broken.docx"})
    );
    let library = jubarte::edit::for_edit::for_edit(
        &comment_graph("20000002"),
        &jubarte::edit::for_edit::ForEditRequest::default(),
    )
    .unwrap();
    let library = serde_json::to_value(&library).unwrap();
    assert!(library.get("input_validity").is_none(), "{library}");
}

/// IN and OUT in a fresh directory.
fn pair(input: &[u8], output: &[u8]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), input).unwrap();
    std::fs::write(dir.path().join("out.docx"), output).unwrap();
    dir
}

const END: &str = r#"<w:r><w:fldChar w:fldCharType="end"/></w:r>"#;

/// A field end with no begin carries no place in its message; an edit that
/// fixes one and makes another, elsewhere, still made a new error (#1291
/// review).
#[test]
fn a_new_finding_of_a_placeless_kind_is_new() {
    let input = docx(&format!(
        r#"<w:p><w:r><w:t>One.</w:t></w:r>{END}</w:p>{}"#,
        para("Two.")
    ));
    let output = docx(&format!(
        r#"{}<w:p><w:r><w:t>Two.</w:t></w:r>{END}</w:p>"#,
        para("One.")
    ));
    let dir = pair(&input, &output);
    let (code, out, err) = run(
        dir.path(),
        &["validate", "out.docx", "--baseline", "in.docx"],
    );
    assert_eq!(code, 2, "{out}{err}");
    assert!(out.starts_with("new: 1\n"), "{out}");
    assert!(out.contains("FIELD_UNBALANCED"), "{out}");
    assert!(
        out.contains("resolved: 1 (in the input, not in this file)"),
        "{out}"
    );
}

/// A paragraph added above an inherited finding renumbers its element
/// chain, not what it is: it stays the input's (#1291 review).
#[test]
fn an_inherited_finding_stays_inherited_when_paragraphs_move() {
    let input = docx(&format!("{DELETED}{}", para("Kept.")));
    let output = docx(&format!("{}{DELETED}{}", para("Inserted."), para("Kept.")));
    let dir = pair(&input, &output);
    let (code, out, err) = run(
        dir.path(),
        &["validate", "out.docx", "--baseline", "in.docx"],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.starts_with("new: none\n"), "{out}");
    assert!(!out.contains("already in the input: none"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// The other party's tracked changes, inherited, are no `FOREIGN_AUTHOR`
/// of the edit's, though the edit's own change renumbers them; a change
/// under a third name is (#1291 review).
#[test]
fn inherited_revisions_are_the_inputs() {
    let ann = r#"<w:ins w:id="1" w:author="Ann" w:date="2026-01-01T00:00:00Z"><w:r><w:t xml:space="preserve"> two</w:t></w:r></w:ins>"#;
    let ours = r#"<w:ins w:id="5" w:author="US" w:date="2026-02-01T00:00:00Z"><w:r><w:t xml:space="preserve">Zero </w:t></w:r></w:ins>"#;
    let bob = r#"<w:ins w:id="6" w:author="Bob" w:date="2026-02-01T00:00:00Z"><w:r><w:t xml:space="preserve"> three</w:t></w:r></w:ins>"#;
    let input = docx(&format!(r#"<w:p><w:r><w:t>One</w:t></w:r>{ann}</w:p>"#));
    let output = docx(&format!(
        r#"<w:p>{ours}<w:r><w:t>One</w:t></w:r>{ann}</w:p>"#
    ));
    let dir = pair(&input, &output);
    let args = [
        "validate",
        "out.docx",
        "--baseline",
        "in.docx",
        "--author",
        "US",
    ];
    let (code, out, err) = run(dir.path(), &args);
    assert_eq!(code, 0, "{out}{err}");
    // The input holds no finding of its own: Ann's change, untouched, is
    // no FOREIGN_AUTHOR of the input against itself (#1192, #1197).
    assert_eq!(out, "new: none\nalready in the input: none\n");
    let third = docx(&format!(
        r#"<w:p>{ours}<w:r><w:t>One</w:t></w:r>{ann}</w:p><w:p><w:r><w:t>Two</w:t></w:r>{bob}</w:p>"#
    ));
    std::fs::write(dir.path().join("out.docx"), third).unwrap();
    let (code, out, _) = run(dir.path(), &args);
    assert_eq!(code, 2, "{out}");
    assert!(out.starts_with("new: "), "{out}");
    assert!(out.contains("by Bob"), "{out}");
    assert!(!out.contains("by Ann"), "{out}");
}

#[test]
fn the_rules_default_names_the_baseline() {
    let (_, out, _) = run(Path::new("."), &["validate", "--help"]);
    let words = out.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        words.contains("with --original, or --baseline and --author, word and edits"),
        "{out}"
    );
}

/// A Word-fatal deletion of `text` as change `id`: `w:t` inside `w:del`.
fn deleted(id: u32, text: &str) -> String {
    format!(
        r#"<w:p><w:del w:id="{id}" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t xml:space="preserve">{text}</w:t></w:r></w:del></w:p>"#
    )
}

/// `validate out.docx --baseline in.docx`, then `extra`, over `input` and
/// `output`.
fn baselined(input: &[u8], output: &[u8], extra: &[&str]) -> (i32, String, String) {
    let dir = pair(input, output);
    let mut args = vec!["validate", "out.docx", "--baseline", "in.docx"];
    args.extend_from_slice(extra);
    run(dir.path(), &args)
}

/// The finding lines `validate --baseline` prints as new.
fn new_lines(out: &str) -> Vec<&str> {
    out.lines()
        .skip(1)
        .take_while(|line| !line.starts_with("already in the input:"))
        .collect()
}

/// A package whose default header holds `paragraphs`.
fn with_header(body: &str, paragraphs: &str) -> Vec<u8> {
    let xml = format!(
        r#"<w:hdr xmlns:w="{}" xmlns:r="{}">{paragraphs}</w:hdr>"#,
        common::docx::W_NS,
        common::docx::R_NS
    );
    common::docx::docx_with_sect(
        body,
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &xml,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

/// A same-text paragraph inserted above an inherited finding leaves it the
/// input's (#1291 review, 1).
#[test]
fn an_inherited_finding_stays_inherited_when_a_same_text_paragraph_is_inserted_above_it() {
    let input = docx(&format!("{DELETED}{}", para("Kept.")));
    let output = docx(&format!("{}{DELETED}{}", para("gone"), para("Kept.")));
    let (code, out, err) = baselined(&input, &output, &[]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.starts_with("new: none\n"), "{out}");
    assert!(out.contains("already in the input: 1 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// A new finding that repeats an inherited one, inserted above it, is the
/// one printed as new: it is a change the input did not have (#1291
/// review, 2).
#[test]
fn a_new_finding_beside_a_same_text_one_takes_no_inherited_place() {
    let input = docx(&format!(
        "{}{}{}",
        para("Pre."),
        deleted(1, "Hello"),
        para("Post.")
    ));
    let output = docx(&format!(
        "{}{}{}{}",
        para("Pre."),
        deleted(2, "Hello"),
        deleted(1, "Hello"),
        para("Post.")
    ));
    let (code, out, err) = baselined(&input, &output, &[]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(out.starts_with("new: 1\n"), "{out}");
    let new = new_lines(&out);
    assert!(new[0].contains("#w:body[0]/w:p[1]/"), "{out}");
    assert!(out.contains("already in the input: 1 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// The same in a header: the paragraph added at its top is the new one,
/// and the header's inherited findings stay the input's (#1291 review, 3).
#[test]
fn a_header_finding_stays_inherited_when_a_same_text_header_paragraph_is_inserted() {
    let header = |top: &str| {
        with_header(
            &para("Body."),
            &format!(
                "{top}{}{}{}",
                para("footer"),
                deleted(100, "footer"),
                deleted(101, "footer")
            ),
        )
    };
    let (code, out, err) = baselined(&header(""), &header(&deleted(102, "footer")), &[]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(out.starts_with("new: 1\n"), "{out}");
    let new = new_lines(&out);
    assert!(new[0].contains("word/header1.xml#w:p[0]/"), "{out}");
    assert!(out.contains("already in the input: 2 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// A table added between two inherited tables, its cell holding the same
/// finding, is the new one (#1291 review, 4).
#[test]
fn a_table_cell_finding_stays_inherited_when_a_same_text_cell_is_inserted() {
    let cell = |id: u32| {
        format!(
            "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
            deleted(id, "Hello")
        )
    };
    let input = docx(&format!("{}{}{}", cell(1), para("Trailer."), cell(2)));
    let output = docx(&format!(
        "{}{}{}{}",
        cell(1),
        para("Trailer."),
        cell(3),
        cell(2)
    ));
    let (code, out, err) = baselined(&input, &output, &[]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(out.starts_with("new: 1\n"), "{out}");
    let new = new_lines(&out);
    assert!(new[0].contains("#w:body[0]/w:tbl[1]/"), "{out}");
    assert!(out.contains("already in the input: 2 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// A finding outside any paragraph keys by its element and what holds it:
/// a style added above three inherited `STYLE_NAME_COMMA` styles of the
/// same name is the new one (#1291 review, 5).
#[test]
fn a_style_finding_stays_inherited_when_a_same_name_style_is_inserted() {
    let styles = |ids: &[u32]| {
        let styles: String = ids
            .iter()
            .map(|id| {
                format!(
                    r#"<w:style w:type="paragraph" w:styleId="X{id}"><w:name w:val="Has, comma"/></w:style>"#
                )
            })
            .collect();
        let xml = format!(
            r#"<w:styles xmlns:w="{}">{styles}</w:styles>"#,
            common::docx::W_NS
        );
        docx_with(
            &para("Body."),
            &[Part {
                name: "word/styles.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
                xml: &xml,
            }],
        )
    };
    let (code, out, err) = baselined(&styles(&[1, 2, 3]), &styles(&[4, 1, 2, 3]), &[]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(out.starts_with("new: 1\n"), "{out}");
    let new = new_lines(&out);
    assert!(new[0].contains("word/styles.xml#w:style[0]/"), "{out}");
    assert!(out.contains("already in the input: 3 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// Empty paragraphs share their text, so text cannot tell them apart; the
/// change each finding sits in still does (#1291 review, 6).
#[test]
fn an_empty_paragraph_finding_keys_by_the_change_it_sits_in() {
    let input = docx(&format!("{}{}", deleted(1, ""), deleted(2, "")));
    let output = docx(&format!(
        "{}{}{}",
        deleted(3, ""),
        deleted(1, ""),
        deleted(2, "")
    ));
    let (code, out, err) = baselined(&input, &output, &[]);
    assert_eq!(code, 2, "{out}{err}");
    assert!(out.starts_with("new: 1\n"), "{out}");
    let new = new_lines(&out);
    assert!(new[0].contains("#w:body[0]/w:p[0]/"), "{out}");
    assert!(out.contains("already in the input: 2 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// The other party's change the edit added beside two identical inherited
/// ones is the new one, by its id (#1291 review, 8).
#[test]
fn a_foreign_change_keys_by_its_id_among_identical_ones() {
    let ann = |id: u32| {
        format!(
            r#"<w:ins w:id="{id}" w:author="Ann" w:date="2026-01-01T00:00:00Z"><w:r><w:t xml:space="preserve"> two</w:t></w:r></w:ins>"#
        )
    };
    let line =
        |text: &str, id: u32| format!(r#"<w:p><w:r><w:t>{text}</w:t></w:r>{}</w:p>"#, ann(id));
    let input = docx(&format!("{}{}", line("One", 1), line("Three", 2)));
    let output = docx(&format!(
        "{}{}{}",
        line("One", 1),
        line("Two", 3),
        line("Three", 2)
    ));
    let (code, out, err) = baselined(&input, &output, &["--author", "US"]);
    assert_eq!(code, 2, "{out}{err}");
    let new = new_lines(&out).join("\n");
    assert!(new.contains("r3 by Ann"), "{out}");
    assert!(
        !new.contains("r1 by Ann") && !new.contains("r2 by Ann"),
        "{out}"
    );
    // Ann's untouched r1 and r2 are no finding of the input's (#1197).
    assert!(out.contains("already in the input: none"), "{out}");
}

/// A header paragraph's audit finding keeps its identity when a paragraph
/// added above renumbers its id (`header1` → `header1.p1`; #1291
/// review, 9).
#[test]
fn a_header_paragraph_finding_stays_inherited_when_its_id_moves() {
    let input = with_header(&para("Body."), &para("• one"));
    let output = with_header(
        &para("Body."),
        &format!("{}{}", para("Intro."), para("• one")),
    );
    let (code, out, err) = baselined(&input, &output, &["--rules", "style"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.starts_with("new: none\n"), "{out}");
    assert!(out.contains("already in the input: 1 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// An edit that rewrites the paragraph an inherited finding sits in, its
/// neighbours kept, leaves the finding the input's.
#[test]
fn an_inherited_finding_stays_inherited_when_its_paragraph_is_rewritten() {
    let unbalanced = |text: &str| {
        format!(
            "{}<w:p><w:r><w:t>{text}</w:t></w:r>{END}</w:p>{}",
            para("Before."),
            para("After.")
        )
    };
    let (code, out, err) = baselined(
        &docx(&unbalanced("One.")),
        &docx(&unbalanced("One, rewritten.")),
        &[],
    );
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.starts_with("new: none\n"), "{out}");
    assert!(out.contains("already in the input: 1 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// A paragraph's `w14:paraId` keeps an inherited finding the input's when
/// the edit both rewrites the paragraph and adds one above it.
#[test]
fn an_inherited_finding_keys_by_its_paragraph_id() {
    let unbalanced = |text: &str| {
        format!(
            r#"<w:p xmlns:w14="{W14}" w14:paraId="0A0A0A0A"><w:r><w:t>{text}</w:t></w:r>{END}</w:p>"#
        )
    };
    let input = docx(&unbalanced("One."));
    let output = docx(&format!("{}{}", para("Inserted."), unbalanced("Changed.")));
    let (code, out, err) = baselined(&input, &output, &[]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.starts_with("new: none\n"), "{out}");
    assert!(out.contains("already in the input: 1 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// A tracked insertion into the paragraph an inherited deletion sits in
/// changes the paragraph's text, not the deletion: the finding inside it
/// is the input's.
#[test]
fn an_inherited_finding_keys_by_the_change_it_sits_in() {
    let ours = r#"<w:ins w:id="5" w:author="US" w:date="2026-02-01T00:00:00Z"><w:r><w:t xml:space="preserve">Zero </w:t></w:r></w:ins>"#;
    let gone = r#"<w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del>"#;
    let input = docx(&format!(r#"<w:p><w:r><w:t>Keep </w:t></w:r>{gone}</w:p>"#));
    let output = docx(&format!(
        r#"<w:p>{ours}<w:r><w:t>Keep </w:t></w:r>{gone}</w:p>"#
    ));
    let (code, out, err) = baselined(&input, &output, &[]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(out.starts_with("new: none\n"), "{out}");
    assert!(out.contains("already in the input: 1 (not yours)"), "{out}");
    assert!(!out.contains("resolved:"), "{out}");
}

/// A clean document with both kinds of records never ends stdout with a status (#1925).
#[test]
fn quiet_json_listings_end_with_a_record() {
    let dir = tempfile::tempdir().unwrap();
    let source = comment_graph("10000001");
    let revision = r#"<w:p><w:ins w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>added</w:t></w:r></w:ins></w:p>"#;
    let xml = common::docx::part_string(&source, "word/document.xml").unwrap();
    let xml = xml.replace(COMMENTED, &format!("{COMMENTED}{revision}"));
    let source = common::docx::replace_entry(&source, "word/document.xml", xml.as_bytes());
    let validity = jubarte::validate::input_validity_of(&source, "clean.docx");
    assert_eq!((validity.errors, validity.warnings), (0, 0));
    std::fs::write(dir.path().join("clean.docx"), source).unwrap();
    for task in ["changes", "comments"] {
        let (code, out, err) = run(dir.path(), &[task, "clean.docx", "--json", "-q"]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(err, "");
        let record: serde_json::Value = serde_json::from_str(out.lines().last().unwrap()).unwrap();
        let field = if task == "changes" { "kind" } else { "text" };
        assert!(record.get(field).is_some(), "{record}");
    }
}
