// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! `read`'s options with `--json` and `--tables` (#1158): the view's own
//! options are no-ops there, `--track-changes` settles the text, `-p tN`
//! picks a table of the grids, and with `--json` an error is a JSON object
//! on stdout as well as text on stderr.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use common::docx::{Part, docx, docx_with_sect, para};
use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_jubarte");

const INS: &str = r#"<w:ins w:id="1" w:author="Ann" w:date="2026-10-01T00:00:00Z"><w:r><w:t>new</w:t></w:r></w:ins>"#;
const DEL: &str = r#"<w:del w:id="2" w:author="Ann" w:date="2026-10-01T00:00:00Z"><w:r><w:delText>old</w:delText></w:r></w:del>"#;

/// `keep ` then Ann's insertion `new` and her deletion `old`.
fn revised(id: u32) -> String {
    let ins = INS.replace(r#"w:id="1""#, &format!(r#"w:id="{id}""#));
    let del = DEL.replace(r#"w:id="2""#, &format!(r#"w:id="{}""#, id + 1));
    format!(r#"<w:p><w:r><w:t xml:space="preserve">keep </w:t></w:r>{ins}{del}</w:p>"#)
}

fn grid(cells: &str) -> String {
    format!(
        r#"<w:tbl><w:tblPr><w:tblW w:w="4000" w:type="dxa"/></w:tblPr><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr>{cells}</w:tr></w:tbl>"#
    )
}

/// p0, t0 (whose cell holds a nested table), p3, t1, and a header whose
/// paragraph is revised too.
fn document() -> Vec<u8> {
    let inner = grid(&format!("<w:tc>{}</w:tc>", para("inner")));
    let outer = grid(&format!("<w:tc>{}{inner}<w:p/></w:tc>", revised(10)));
    let second = grid(&format!("<w:tc>{}</w:tc>", para("second")));
    let header = format!(
        r#"<w:hdr xmlns:w="{}">{}</w:hdr>"#,
        common::docx::W_NS,
        revised(20)
    );
    docx_with_sect(
        &format!("{}{outer}{}{second}", revised(1), para("between")),
        &[Part {
            name: "word/header1.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
            xml: &header,
        }],
        r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
    )
}

fn fixture(name: &str, bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    (dir, path)
}

fn jubarte(args: &[&str], dir: &Path) -> Output {
    Command::new(BIN)
        .args(args)
        .current_dir(dir)
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).unwrap()
}

fn ok(out: &Output) -> String {
    assert!(out.status.success(), "{}", stderr(out));
    stdout(out)
}

fn json(out: &Output) -> Value {
    serde_json::from_str(&ok(out)).unwrap()
}

#[test]
fn no_page_markers_with_json_is_byte_identical() {
    let (dir, file) = fixture("d.docx", &document());
    let file = file.to_str().unwrap();
    let plain = ok(&jubarte(&["read", file, "--json"], dir.path()));
    for extra in [
        &["--no-page-markers"][..],
        &["--comments", "none"],
        &["--comments", "inline"],
        &["--dates"],
        &["--no-page-markers", "--comments", "none", "--dates"],
    ] {
        let args = [&["read", file, "--json"][..], extra].concat();
        assert_eq!(ok(&jubarte(&args, dir.path())), plain, "{extra:?}");
        // The one-document shorthand reads alike.
        let args = [&[file, "--json"][..], extra].concat();
        assert_eq!(ok(&jubarte(&args, dir.path())), plain, "{extra:?}");
    }
    let grids = ok(&jubarte(&["read", file, "--tables"], dir.path()));
    for extra in [
        &["--no-page-markers"][..],
        &["--comments", "none"],
        &["--dates"],
    ] {
        let args = [&["read", file, "--tables"][..], extra].concat();
        assert_eq!(ok(&jubarte(&args, dir.path())), grids, "{extra:?}");
    }
    // `-o FILE.json` writes the bytes --json prints.
    let out = jubarte(
        &["read", file, "-o", "s.json", "--no-page-markers", "--dates"],
        dir.path(),
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("s.json")).unwrap(),
        plain
    );
}

#[test]
fn tables_with_paragraph_selects_the_table() {
    let (dir, file) = fixture("d.docx", &document());
    let file = file.to_str().unwrap();
    let all = ok(&jubarte(&["read", file, "--tables"], dir.path()));
    let heads: Vec<&str> = all.lines().filter(|l| l.starts_with("table ")).collect();
    assert_eq!(heads.len(), 3, "{all}");
    // The view's t1 is the second top-level table: the grids' table 2,
    // the nested table counted before it.
    let second = ok(&jubarte(
        &["read", file, "--tables", "-p", "t1"],
        dir.path(),
    ));
    assert!(second.starts_with("table 2: "), "{second}");
    assert!(second.contains("second"), "{second}");
    assert!(!second.contains("inner"), "{second}");
    // t0 is the outer table with the one nested in it.
    let first = ok(&jubarte(
        &["read", file, "--tables", "-p", "t0"],
        dir.path(),
    ));
    let heads: Vec<&str> = first.lines().filter(|l| l.starts_with("table ")).collect();
    assert_eq!(heads.len(), 2, "{first}");
    assert!(heads[0].starts_with("table 0: ") && heads[1].starts_with("table 1: "));
    assert!(!first.contains("second"), "{first}");
    // Both, in document order, whatever order they are named in.
    let both = ok(&jubarte(
        &["read", file, "--tables", "--select", "t1,t0"],
        dir.path(),
    ));
    assert_eq!(both, all);
    // A table the view does not have fails as the view's `-p t5` does; a
    // paragraph pick is a usage error that says what -p takes here.
    let out = jubarte(&["read", file, "--tables", "-p", "t5"], dir.path());
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("t5 is not a table of this document (2 tables)"),
        "{}",
        stderr(&out)
    );
    let view = jubarte(&["read", file, "-p", "t5", "--no-page-markers"], dir.path());
    assert_eq!(view.status.code(), Some(1), "{}", stderr(&view));
    let out = jubarte(&["read", file, "--tables", "-p", "p0"], dir.path());
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("--tables -p takes tables (t0, t1)"),
        "{}",
        stderr(&out)
    );
    assert!(stdout(&out).is_empty());
}

#[test]
fn track_changes_accepted_with_json_settles_text() {
    let (dir, file) = fixture("d.docx", &document());
    let file = file.to_str().unwrap();
    let plain = json(&jubarte(&["read", file, "--json"], dir.path()));
    assert_eq!(plain["paragraphs"][0]["text"], "keep new");
    assert!(plain.get("track_changes").is_none(), "{plain}");
    for (spelling, choice, text) in [
        ("accept", "accept", "keep new"),
        ("accepted", "accept", "keep new"),
        ("reject", "reject", "keep old"),
        ("rejected", "reject", "keep old"),
    ] {
        let snapshot = json(&jubarte(
            &["read", file, "--json", "--track-changes", spelling],
            dir.path(),
        ));
        assert_eq!(snapshot["track_changes"], choice, "{spelling}");
        assert_eq!(snapshot["paragraphs"][0]["text"], text, "{spelling}");
        // Cells, nested tables and stories settle alike; the ids hold.
        assert_eq!(snapshot["paragraphs"][1]["text"], text, "{spelling}");
        assert_eq!(
            snapshot["paragraphs"][1]["id"],
            plain["paragraphs"][1]["id"]
        );
        assert_eq!(
            snapshot["tables"][0]["rows"][0][0]["text"],
            format!("{text}\n"),
            "{spelling}"
        );
        assert_eq!(
            snapshot["stories"][0]["paragraphs"][0]["text"], text,
            "{spelling}"
        );
        // The document's own facts stay: its changes and its settings.
        assert_eq!(snapshot["changes"], plain["changes"], "{spelling}");
        assert_eq!(snapshot["summary"], plain["summary"], "{spelling}");
    }
    // A selection keeps its paragraphs under the choice.
    let picked = json(&jubarte(
        &[
            "read",
            file,
            "--json",
            "--track-changes",
            "reject",
            "-p",
            "p0",
        ],
        dir.path(),
    ));
    let texts: Vec<&Value> = picked["paragraphs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| &p["text"])
        .collect();
    assert_eq!(texts, [&Value::from("keep old")]);
    // `-o FILE.json` writes what --json prints.
    let printed = ok(&jubarte(
        &["read", file, "--json", "--track-changes", "reject"],
        dir.path(),
    ));
    let out = jubarte(
        &["read", file, "-o", "r.json", "--track-changes", "reject"],
        dir.path(),
    );
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("r.json")).unwrap(),
        printed
    );
    // The grids settle too.
    let grids = ok(&jubarte(
        &["read", file, "--tables", "--track-changes", "rejected"],
        dir.path(),
    ));
    assert!(grids.contains("keep old"), "{grids}");
    assert!(!grids.contains("keep new"), "{grids}");
    // `all` keeps the snapshot as it is.
    assert_eq!(
        json(&jubarte(
            &["read", file, "--json", "--track-changes", "all"],
            dir.path()
        )),
        plain
    );
}

#[test]
fn usage_error_with_json_is_an_object_on_stdout() {
    let (dir, file) = fixture("d.docx", &docx(&para("One.")));
    let file = file.to_str().unwrap();
    let out = jubarte(&["read", file, "--json", "--bogus"], dir.path());
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let printed = stdout(&out);
    assert_eq!(printed.lines().count(), 1, "{printed}");
    let error: Value = serde_json::from_str(&printed).unwrap();
    assert_eq!(error["error"]["code"], "USAGE", "{error}");
    let message = error["error"]["message"].as_str().unwrap();
    assert!(message.contains("--bogus"), "{error}");
    assert!(!message.starts_with("error:"), "{error}");
    assert_eq!(error["error"]["usage"], "jubarte read [OPTIONS] <FILE>");
    // clap's own text stays on stderr (its usage line names the flags
    // given), and without --json stdout stays empty.
    assert!(
        stderr(&out).starts_with("error: unexpected argument '--bogus' found\n"),
        "{}",
        stderr(&out)
    );
    assert!(stderr(&out).contains("\nUsage: jubarte read --json <FILE>\n"));
    let plain = jubarte(&["read", file, "--bogus"], dir.path());
    assert_eq!(plain.status.code(), Some(2));
    assert!(stdout(&plain).is_empty());
    assert!(stderr(&plain).starts_with("error: unexpected argument '--bogus' found\n"));
    // A selection that does not parse is a usage error too.
    let out = jubarte(&["read", file, "--json", "-p", "q7"], dir.path());
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let error: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(error["error"]["code"], "USAGE", "{error}");
    // A runtime error keeps its exit code and its text, with a code.
    let out = jubarte(&["read", "missing.docx", "--json"], dir.path());
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let error: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(error["error"]["code"], "ERROR", "{error}");
    let message = error["error"]["message"].as_str().unwrap();
    assert!(message.contains("missing.docx"), "{error}");
    assert!(error["error"].get("usage").is_none(), "{error}");
    assert_eq!(stderr(&out), format!("error: {message}\n"));
    // Other commands' --json alike.
    let out = jubarte(&["changes", file, "--json", "--bogus"], dir.path());
    assert_eq!(out.status.code(), Some(2));
    let error: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(error["error"]["usage"], "jubarte changes [OPTIONS] <FILE>");
    let out = jubarte(&["comments", "missing.docx", "--json"], dir.path());
    assert_eq!(out.status.code(), Some(1));
    let error: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(error["error"]["code"], "ERROR", "{error}");
    // Every line clap writes reaches the object: a message spread over
    // lines (the list it names), and each form of the usage.
    let out = jubarte(&["a.docx", "b.docx", "c.docx", "--json"], dir.path());
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let error: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(
        error["error"]["message"],
        "the subcommand 'c.docx' cannot be used with: [ORIGINAL] [MODIFIED]"
    );
    assert_eq!(
        error["error"]["usage"],
        "jubarte [OPTIONS]\njubarte <COMMAND>"
    );
    // Help and version are not errors.
    let out = jubarte(&["read", "--json", "--help"], dir.path());
    assert!(out.status.success());
    assert!(
        stdout(&out).starts_with("Read the agent view"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn help_says_ignored_with_json() {
    let out = jubarte(&["read", "--help"], Path::new("."));
    let help = ok(&out);
    for flag in ["--comments", "--dates", "--no-page-markers"] {
        let at = help
            .find(&format!("      {flag}"))
            .unwrap_or_else(|| panic!("{flag} in {help}"));
        let entry = &help[at..];
        let end = entry[1..]
            .find("\n      -")
            .or_else(|| entry[1..].find("\n  -"))
            .map_or(entry.len(), |e| e + 1);
        let entry = entry[..end]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            entry.contains("ignored with --json, --tables and --for-edit"),
            "{flag}: {entry}"
        );
    }
    assert!(
        help.contains("accepted") || help.contains("--track-changes"),
        "{help}"
    );
}

#[test]
fn the_host_parser_hands_its_host_the_json_line() {
    let parse = |argv: &[&str]| -> Value {
        let argv: Vec<String> = argv.iter().map(|a| (*a).to_string()).collect();
        serde_json::from_str(&jubarte::cli::parse_json(&argv, "jubarte-redlines", &[])).unwrap()
    };
    let parsed = parse(&["read", "a.docx", "--json", "--bogus"]);
    assert_eq!(parsed["exit_code"], 2, "{parsed}");
    assert_eq!(parsed["stream"], "stderr");
    let line = parsed["json"].as_str().unwrap();
    assert!(
        line.ends_with('\n') && line.lines().count() == 1,
        "{line:?}"
    );
    let line: Value = serde_json::from_str(line).unwrap();
    assert_eq!(line["error"]["code"], "USAGE");
    assert_eq!(
        line["error"]["usage"],
        "jubarte-redlines read [OPTIONS] <FILE>"
    );
    // A removed spelling is a usage error too.
    let parsed = parse(&["revisions", "a.docx", "--json"]);
    assert_eq!(parsed["exit_code"], 2, "{parsed}");
    let line: Value = serde_json::from_str(parsed["json"].as_str().unwrap()).unwrap();
    assert_eq!(line["error"]["code"], "USAGE", "{line}");
    // Without --json (or after `--`), none.
    assert!(parse(&["read", "a.docx", "--bogus"]).get("json").is_none());
    assert!(
        parse(&["read", "a.docx", "--bogus", "--", "--json"])
            .get("json")
            .is_none()
    );
    // Help is no error.
    assert!(parse(&["read", "--help", "--json"]).get("json").is_none());
}

#[test]
fn a_runtime_error_takes_its_leading_code() {
    let code = |message: &str, exit: u8| -> Value {
        let line = jubarte::cli::runtime_json_error(message, exit);
        let value: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(value["error"]["message"], message);
        assert!(value["error"].get("usage").is_none());
        value["error"]["code"].clone()
    };
    assert_eq!(
        code("STALE_SOURCE: the plan was read from another file", 3),
        "STALE_SOURCE"
    );
    assert_eq!(code("OUTPUT_EXISTS: out.docx exists", 1), "OUTPUT_EXISTS");
    assert_eq!(
        code("reading x.docx: No such file or directory", 1),
        "ERROR"
    );
    assert_eq!(code("Contract.docx: not a Word document", 1), "ERROR");
    assert_eq!(code("IO: short", 1), "ERROR");
    assert_eq!(code("the range runs backwards", 2), "USAGE");
}
