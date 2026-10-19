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
        &["read", "in.docx", "--json"],
        &["in.docx", "--no-page-markers"],
        &["changes", "in.docx"],
        &["changes", "in.docx", "--json"],
        &["comments", "in.docx"],
        &["comments", "in.docx", "--json"],
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
    assert_eq!(out, "new: none\nalready in the input: 1 (not yours)\n");
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
