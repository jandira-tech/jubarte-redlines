// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `jubarte validate`: exit 0 clean, 2 on a word or edits finding (or an
//! error, or a warning under `--strict`), 1 unreadable; `--json`,
//! `--repair`, `--original --author`, `--rules`.

mod common;

use std::path::Path;
use std::process::Command;

use common::docx::{docx, para};

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");
const DELETED: &str = r#"<w:p><w:del w:id="1" w:author="A" w:date="2026-01-01T00:00:00Z"><w:r><w:t>gone</w:t></w:r></w:del></w:p>"#;
/// Two insertions sharing an id: a lead, not Word-fatal.
const SAME_ID: &str = r#"<w:p><w:ins w:id="5" w:author="A"><w:r><w:t>a</w:t></w:r></w:ins><w:ins w:id="5" w:author="A"><w:r><w:t>b</w:t></w:r></w:ins></w:p>"#;

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

#[test]
fn validate_exit_codes_prose_json_and_repair() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("broken.docx"), docx(DELETED)).unwrap();
    std::fs::write(dir.path().join("clean.docx"), docx(&para("fine"))).unwrap();

    let (code, out, _) = run(dir.path(), &["validate", "clean.docx"]);
    assert_eq!((code, out.trim()), (0, "no findings"));

    let (code, out, _) = run(dir.path(), &["validate", "broken.docx"]);
    assert_eq!(code, 2, "{out}");
    assert!(
        out.starts_with(
            "* TEXT_INSIDE_DELETION\terror\tword/document.xml#\
             w:body[0]/w:p[0]/w:del[0]/w:r[0]/w:t[0]\t"
        ),
        "{out}"
    );
    assert!(
        out.ends_with("1 finding(s): 1 error, 0 warning, 0 info\n"),
        "{out}"
    );

    let (code, out, _) = run(dir.path(), &["validate", "broken.docx", "--json"]);
    assert_eq!(code, 2);
    let row: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    assert_eq!(row["code"], "TEXT_INSIDE_DELETION");
    assert_eq!(row["set"], "word");
    assert_eq!(row["severity"], "error");
    assert_eq!(row["location"], "w:body[0]/w:p[0]/w:del[0]/w:r[0]/w:t[0]");
    assert_eq!(row["ref"], "w:body[0]/w:p[0]/w:del[0]/w:r[0]/w:t[0]");
    assert_eq!(row["word_fatal"], true);
    assert!(row.get("path").is_none(), "{row}");

    let (code, out, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--repair", "fixed.docx"],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        out,
        "repaired 1 finding(s), structure only; history not checked (--original, --author)\nno findings\n"
    );
    assert!(err.starts_with("wrote fixed.docx (docx, "), "{err}");
    let (_, out, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--repair", "quiet.docx", "-q"],
    );
    assert_eq!(
        (out.as_str(), err.as_str()),
        (
            "repaired 1 finding(s), structure only; history not checked (--original, --author)\nno findings\n",
            ""
        )
    );
    let (code, _, _) = run(dir.path(), &["validate", "fixed.docx"]);
    assert_eq!(code, 0);

    let (code, _, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--repair", "fixed.docx"],
    );
    assert_eq!(code, 2);
    assert!(err.contains("already exists"), "{err}");
    let (code, _, _) = run(
        dir.path(),
        &[
            "validate",
            "broken.docx",
            "--repair",
            "fixed.docx",
            "--force",
        ],
    );
    assert_eq!(code, 0);

    let (code, _, err) = run(dir.path(), &["validate", "missing.docx"]);
    assert_eq!(code, 1);
    assert!(err.starts_with("error:"), "{err}");

    let (code, _, err) = run(
        dir.path(),
        &["validate", "broken.docx", "--original", "clean.docx"],
    );
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("--author"), "{err}");
}

#[test]
fn validate_audits_tracked_edits_against_an_original() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.docx"), docx(&para("Fee is 10."))).unwrap();
    std::fs::write(dir.path().join("b.docx"), docx(&para("Fee is 12."))).unwrap();
    let (code, out, _) = run(
        dir.path(),
        &[
            "validate",
            "b.docx",
            "--original",
            "a.docx",
            "--author",
            "Legal",
            "--json",
        ],
    );
    assert_eq!(code, 2, "{out}");
    let row: serde_json::Value = serde_json::from_str(out.lines().next().unwrap()).unwrap();
    assert_eq!(row["code"], "UNTRACKED_EDIT");
    assert_eq!(row["set"], "edits");
    assert_eq!(row["severity"], "error");
    assert_eq!(row["location"], "p0");
    assert_eq!(row["ref"], "p0");
    // The JSON location keeps the long id; the message prints the short one.
    assert!(
        row["message"].as_str().unwrap().starts_with("p0 differs"),
        "{row}"
    );

    let (code, _, _) = run(
        dir.path(),
        &[
            "a.docx",
            "b.docx",
            "-o",
            "r.docx",
            "-d",
            "1970-01-01T00:00:00Z",
            "--author",
            "Legal",
        ],
    );
    assert_eq!(code, 0);
    let (code, out, _) = run(
        dir.path(),
        &[
            "validate",
            "r.docx",
            "--original",
            "a.docx",
            "--author",
            "Legal",
        ],
    );
    assert_eq!((code, out.trim()), (0, "no findings"));
    let (code, out, _) = run(
        dir.path(),
        &[
            "validate",
            "r.docx",
            "--original",
            "a.docx",
            "--author",
            "Other",
        ],
    );
    assert_eq!(code, 2);
    assert!(out.contains("FOREIGN_AUTHOR"), "{out}");
}

#[test]
fn an_ooxml_revision_error_fails_but_audit_warnings_fail_only_under_strict() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("lead.docx"), docx(SAME_ID)).unwrap();
    std::fs::write(dir.path().join("bullets.docx"), docx(&para("• item"))).unwrap();

    let (code, out, _) = run(dir.path(), &["validate", "lead.docx"]);
    assert_eq!(code, 2, "{out}");
    assert!(out.starts_with("  DUPLICATE_REVISION_ID\terror\t"), "{out}");

    let (code, out, _) = run(
        dir.path(),
        &["validate", "bullets.docx", "--rules", "style"],
    );
    assert_eq!(code, 0, "{out}");
    assert!(out.starts_with("  LITERAL_BULLET\twarning\tp0\t"), "{out}");
    assert!(
        out.ends_with("1 finding(s): 0 error, 1 warning, 0 info\n"),
        "{out}"
    );
    let (code, _, _) = run(
        dir.path(),
        &["validate", "bullets.docx", "--rules", "style", "--strict"],
    );
    assert_eq!(code, 2);
    // word,style: the Word check runs too and finds nothing.
    let (code, out, _) = run(
        dir.path(),
        &["validate", "bullets.docx", "--rules", "word,style"],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(out.lines().count(), 2, "{out}");
}

#[test]
fn rules_usage_errors_exit_two() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.docx"), docx(&para("a"))).unwrap();
    let (code, _, err) = run(dir.path(), &["validate", "a.docx", "--rules", "edits"]);
    assert_eq!(code, 2, "{err}");
    assert!(
        err.contains("--rules edits needs --author, and --original or --baseline"),
        "{err}"
    );
    let (code, _, err) = run(dir.path(), &["validate", "a.docx", "--rules", "BOGUS"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("BOGUS"), "{err}");
    assert!(err.contains("word, edits, a11y, style, structure"), "{err}");
}

#[test]
fn issue_1137_preexisting_duplicate_does_not_block_edit_publication() {
    let source = docx(&format!(
        "{}{}",
        para("old words"),
        r#"<w:p><w:ins w:id="5" w:author="Alice"><w:r><w:t>a</w:t></w:r></w:ins><w:r><w:t xml:space="preserve"> x </w:t></w:r><w:ins w:id="5" w:author="Alice"><w:r><w:t>b</w:t></w:r></w:ins></w:p>"#
    ));
    let baseline = jubarte::validate::validate(&source).unwrap();
    let duplicate = baseline
        .iter()
        .find(|f| f.code == "DUPLICATE_REVISION_ID")
        .unwrap();
    assert_eq!(duplicate.severity, "error");
    assert!(!duplicate.word_fatal);
    let json = r#"{"schema_version":1,"author":"Carol Diaz","date":"2026-10-01T09:00:00Z","existing_revisions":"keep","operations":[{"kind":"replace","paragraph":"p0","find":"old","replacement":"new"}]}"#;
    let plan = jubarte::edit::EditPlan::from_json(json).unwrap();
    let prepared = jubarte::edit::prepare(
        &source,
        &plan,
        &jubarte::edit::PrepareOptions {
            role: jubarte::edit::PrimaryRole::Redline,
            ..jubarte::edit::PrepareOptions::default()
        },
    )
    .unwrap();
    assert!(
        prepared
            .checks
            .baseline
            .iter()
            .any(|f| f.code == "DUPLICATE_REVISION_ID")
    );
    assert!(
        prepared
            .checks
            .introduced
            .iter()
            .all(|f| f.code != "DUPLICATE_REVISION_ID")
    );
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), source).unwrap();
    std::fs::write(dir.path().join("plan.json"), json).unwrap();
    let (code, out, err) = run(dir.path(), &["edit", "source.docx", "--plan", "plan.json"]);
    assert_eq!(code, 0, "{out}\n{err}");
    let published = std::fs::read(dir.path().join("source_modified.docx")).unwrap();
    assert_eq!(
        jubarte::inspect::paragraphs(&published).unwrap()[0].text,
        "new words"
    );
    assert!(
        jubarte::validate::validate(&published)
            .unwrap()
            .iter()
            .any(|f| f.code == "DUPLICATE_REVISION_ID")
    );
}

#[test]
fn received_word_warnings_fail_with_and_without_strict() {
    let dir = tempfile::tempdir().unwrap();
    let received =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/agent-view/received.docx");
    std::fs::copy(received, dir.path().join("received.docx")).unwrap();
    for strict in [false, true] {
        let mut args = vec!["validate", "received.docx"];
        if strict {
            args.push("--strict");
        }
        let (code, out, err) = run(dir.path(), &args);
        assert_eq!(code, 2, "{args:?}: {out}{err}");
        assert!(
            out.contains("COMMENT_METADATA_MISMATCH\twarning\t"),
            "{out}"
        );
        assert!(
            out.ends_with("2 finding(s): 0 error, 2 warning, 0 info\n"),
            "{out}"
        );
    }
}

#[test]
fn strict_help_explains_the_additional_warning_threshold() {
    let dir = tempfile::tempdir().unwrap();
    let (code, out, err) = run(dir.path(), &["validate", "--help"]);
    assert_eq!(code, 0, "{err}");
    let normalized = out.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized.contains(
        "--strict Also fail (exit 2) on warnings from other rule sets; word and edits findings fail without it"
    ), "{out}");
}

const FOREIGN_HISTORY: &str = r#"<w:p><w:ins w:id="1" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:t>alpha beta gamma</w:t></w:r></w:ins><w:del w:id="2" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>old</w:delText></w:r></w:del><w:r><w:t xml:space="preserve"> tail</w:t></w:r></w:p>"#;

fn foreign_audit(edited: &[u8]) -> (i32, Vec<serde_json::Value>) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("original.docx"), docx(FOREIGN_HISTORY)).unwrap();
    std::fs::write(dir.path().join("edited.docx"), edited).unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "validate",
            "edited.docx",
            "--rules",
            "edits",
            "--original",
            "original.docx",
            "--author",
            "Bob",
            "--json",
        ],
    );
    assert!(err.is_empty(), "{err}");
    let findings = out
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (code, findings)
}

fn foreign_plan(author: &str, kind: &str) -> Vec<u8> {
    let operation = if kind == "replace" {
        serde_json::json!({"kind":"replace", "paragraph":"p0", "find":"beta", "replacement":"new"})
    } else {
        serde_json::json!({"kind":"insert", "paragraph":"p0", "after":"tail", "text":" extra"})
    };
    let plan = jubarte::edit::EditPlan::from_json(
        &serde_json::json!({
            "schema_version":1, "author":author, "date":"2026-02-01T00:00:00Z",
            "existing_revisions":"keep", "operations":[operation]
        })
        .to_string(),
    )
    .unwrap();
    jubarte::edit::apply_plan(&docx(FOREIGN_HISTORY), &plan)
        .unwrap()
        .redline
}

#[test]
fn foreign_history_self_audit_exits_zero() {
    let (code, findings) = foreign_audit(&docx(FOREIGN_HISTORY));
    assert_eq!((code, findings), (0, vec![]));
}

#[test]
fn foreign_history_bobs_split_insertion_exits_zero() {
    let (code, findings) = foreign_audit(&foreign_plan("Bob", "replace"));
    assert_eq!((code, findings), (0, vec![]));
}

#[test]
fn foreign_history_alices_new_revision_exits_two_with_one_finding() {
    let (code, findings) = foreign_audit(&foreign_plan("Alice", "insert"));
    assert_eq!(code, 2);
    let foreign: Vec<_> = findings
        .iter()
        .filter(|f| f["code"] == "FOREIGN_AUTHOR")
        .collect();
    assert_eq!(foreign.len(), 1, "{findings:?}");
    assert!(
        foreign[0]["message"]
            .as_str()
            .unwrap()
            .ends_with(" by Alice is new")
    );
}

#[test]
fn foreign_history_rejected_insertion_exits_two_and_names_the_gone_revision() {
    let edited = jubarte::changes::reject_changes(
        &docx(FOREIGN_HISTORY),
        &jubarte::changes::ChangeFilter::ids(["r1"]),
    )
    .unwrap();
    let (code, findings) = foreign_audit(&edited);
    assert_eq!(code, 2);
    let foreign: Vec<_> = findings
        .iter()
        .filter(|f| f["code"] == "FOREIGN_AUTHOR")
        .collect();
    assert_eq!(foreign.len(), 1, "{findings:?}");
    assert_eq!(foreign[0]["message"], "Alice's r1 in the original is gone");
}

#[test]
fn foreign_history_carols_new_revision_exits_two_with_one_finding() {
    let (code, findings) = foreign_audit(&foreign_plan("Carol Diaz", "insert"));
    assert_eq!(code, 2);
    let foreign: Vec<_> = findings
        .iter()
        .filter(|f| f["code"] == "FOREIGN_AUTHOR")
        .collect();
    assert_eq!(foreign.len(), 1, "{findings:?}");
    assert!(
        foreign[0]["message"]
            .as_str()
            .unwrap()
            .ends_with(" by Carol Diaz is new")
    );
}

#[test]
fn foreign_swapped_deletions_exit_two() {
    let carrier = |id, text| {
        format!(
            r#"<w:del w:id="{id}" w:author="Alice" w:date="2026-01-01T00:00:00Z"><w:r><w:delText>{text}</w:delText></w:r></w:del>"#
        )
    };
    let body = |left, right| {
        format!(
            r#"<w:p>{}<w:ins w:id="2" w:author="Carol Diaz"><w:r><w:t>new rate</w:t></w:r></w:ins>{}</w:p>"#,
            carrier(1, left),
            carrier(3, right)
        )
    };
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("original.docx"),
        docx(&body("penalty", "bonus")),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("edited.docx"),
        docx(&body("bonus", "penalty")),
    )
    .unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "validate",
            "edited.docx",
            "--rules",
            "edits",
            "--original",
            "original.docx",
            "--author",
            "Bob",
            "--json",
        ],
    );
    assert!(err.is_empty(), "{err}");
    assert_eq!(code, 2, "{out}");
    assert!(
        out.lines().any(
            |line| serde_json::from_str::<serde_json::Value>(line).unwrap()["code"]
                == "FOREIGN_AUTHOR"
        ),
        "{out}"
    );
}

/// Carol's tracked change of `10` to `12` in `Fee is 10.`, her insertion
/// in two pieces that share w:id 6; `by` wrote the second (KI-8).
fn duplicated_ids(by: &str) -> Vec<u8> {
    docx(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">Fee is </w:t></w:r><w:del w:id="5" w:author="Carol"><w:r><w:delText>10</w:delText></w:r></w:del><w:ins w:id="6" w:author="Carol"><w:r><w:t>1</w:t></w:r></w:ins><w:ins w:id="6" w:author="{by}"><w:r><w:t>2</w:t></w:r></w:ins><w:r><w:t>.</w:t></w:r></w:p>"#
    ))
}

/// `--repair` mends structure; it never says the edits' history was
/// checked, and with `--original` a history that still fails fails the run
/// (KI-8, #1665).
#[test]
fn repair_tells_structure_from_history() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("original.docx"), docx(&para("Fee is 10."))).unwrap();
    std::fs::write(dir.path().join("carol.docx"), duplicated_ids("Carol")).unwrap();
    std::fs::write(dir.path().join("alice.docx"), duplicated_ids("Alice")).unwrap();
    let (code, out, _) = run(
        dir.path(),
        &["validate", "carol.docx", "--repair", "a.docx", "-q"],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        out,
        "repaired 1 finding(s), structure only; history not checked (--original, --author)\nno findings\n"
    );
    let history = ["--original", "original.docx", "--author", "Carol", "-q"];
    let (code, out, _) = run(
        dir.path(),
        &[
            &["validate", "carol.docx", "--repair", "b.docx"][..],
            &history,
        ]
        .concat(),
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        out,
        "repaired 1 finding(s), structure only; history check against original.docx follows\nno findings\n"
    );
    let (code, out, _) = run(
        dir.path(),
        &[
            &["validate", "alice.docx", "--repair", "c.docx"][..],
            &history,
        ]
        .concat(),
    );
    assert_eq!(code, 2, "{out}");
    assert!(
        out.starts_with(
            "repaired 1 finding(s), structure only; history check against original.docx follows\n"
        ),
        "{out}"
    );
    assert!(out.contains("Alice"), "{out}");
    // An original that cannot be read stops the run before any repair.
    let (code, out, err) = run(
        dir.path(),
        &[
            "validate",
            "carol.docx",
            "--repair",
            "d.docx",
            "--original",
            "missing.docx",
            "--author",
            "Carol",
        ],
    );
    assert_eq!(code, 1, "{out}{err}");
    assert!(!out.contains("history"), "{out}");
    assert!(!dir.path().join("d.docx").exists());
    // So does one that is no Word package (Codex on #1666).
    std::fs::write(dir.path().join("garbage.docx"), b"not a zip").unwrap();
    let (code, out, err) = run(
        dir.path(),
        &[
            "validate",
            "carol.docx",
            "--repair",
            "e.docx",
            "--original",
            "garbage.docx",
            "--author",
            "Carol",
        ],
    );
    assert_eq!(code, 1, "{out}{err}");
    assert!(!out.contains("history"), "{out}");
    assert!(!dir.path().join("e.docx").exists());
    let (_, help, _) = run(dir.path(), &["validate", "--help"]);
    assert!(
        help.contains("A structural repair: it does not check the edits' history"),
        "{help}"
    );
}

/// Settling keeps every id (#1264); `--repair` is where ids still change,
/// and only the duplicated ones: two insertions sharing w:id 5 come apart,
/// while the comment and bookmark beside them keep theirs.
#[test]
fn repair_still_renumbers_duplicates() {
    use common::docx::{Part, W_NS, docx_with};
    let dir = tempfile::tempdir().unwrap();
    let body = format!(
        r#"{SAME_ID}<w:p><w:commentRangeStart w:id="7"/><w:bookmarkStart w:id="42" w:name="kept"/><w:r><w:t>Noted.</w:t></w:r><w:bookmarkEnd w:id="42"/><w:commentRangeEnd w:id="7"/><w:r><w:commentReference w:id="7"/></w:r></w:p>"#
    );
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="7" w:author="Bob" w:date="2026-10-18T12:00:00Z" w:initials="B"><w:p><w:r><w:t>Note.</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    let source = docx_with(
        &body,
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    );
    std::fs::write(dir.path().join("same.docx"), source).unwrap();
    let (code, out, _) = run(
        dir.path(),
        &["validate", "same.docx", "--repair", "fixed.docx", "-q"],
    );
    assert_eq!(code, 0, "{out}");
    let fixed = std::fs::read(dir.path().join("fixed.docx")).unwrap();
    let xml = common::docx::part_string(&fixed, "word/document.xml").unwrap();
    let ins: Vec<&str> = xml
        .match_indices("<w:ins w:id=\"")
        .map(|(at, m)| {
            let id = &xml[at + m.len()..];
            &id[..id.find('"').unwrap()]
        })
        .collect();
    assert_eq!(ins.len(), 2, "{xml}");
    assert_ne!(ins[0], ins[1], "{xml}");
    for marker in [
        r#"<w:commentRangeStart w:id="7""#,
        r#"<w:commentReference w:id="7""#,
        r#"<w:bookmarkStart w:id="42""#,
    ] {
        assert!(xml.contains(marker), "{marker} in {xml}");
    }
    let comments = common::docx::part_string(&fixed, "word/comments.xml").unwrap();
    assert!(comments.contains(r#"w:id="7""#), "{comments}");
}
