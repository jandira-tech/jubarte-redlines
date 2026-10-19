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
    let run = |args: &[&str]| jubarte(&[&["read", file][..], args].concat(), dir.path());
    // The grids of `--tables`, one `table N:` line each with the rows under
    // it.
    let grids = |extra: &[&str]| {
        let mut grids: Vec<String> = Vec::new();
        for line in ok(&run(&[&["--tables"][..], extra].concat())).lines() {
            if line.starts_with("table ") {
                grids.push(String::new());
            }
            let grid = grids.last_mut().unwrap();
            grid.push_str(line);
            grid.push('\n');
        }
        grids
    };
    let all = grids(&[]);
    assert_eq!(all.len(), 3, "{all:?}");
    assert!(
        all[1].contains("inner") && all[2].contains("second"),
        "{all:?}"
    );
    // `--tables -p SPEC`: the grids of the tables the view's selection
    // reaches, each with the tables nested in it, in document order and
    // once, an empty line, then `read -p SPEC` unchanged (#1161).
    let picked = |spec: &str, kept: &[usize], extra: &[&str]| {
        let out = ok(&run(&[&["--tables", "-p", spec][..], extra].concat()));
        let view = ok(&run(&[&["-p", spec][..], extra].concat()));
        let all = grids(extra);
        let grids: String = kept.iter().map(|&g| all[g].as_str()).collect();
        assert_eq!(out, format!("{grids}\n{view}"), "-p {spec}");
    };
    // The view's t1 is the second top-level table: the grids' table 2,
    // the nested table counted before it.
    picked("t1", &[2], &[]);
    // t0 is the outer table with the one nested in it.
    picked("t0", &[0, 1], &[]);
    picked("t1,t0", &[0, 1, 2], &[]);
    // A paragraph inside a table reaches the table that holds it, nested
    // or not; a range reaches every table it touches.
    picked("p1", &[0, 1], &[]);
    picked("p2", &[0, 1], &[]);
    picked("t0,p1", &[0, 1], &[]);
    picked("p5", &[2], &[]);
    picked("p0-p1", &[0, 1], &[]);
    picked("p1-p5", &[0, 1, 2], &[]);
    // The view's options shape the view after the grids.
    picked("t1", &[2], &["--no-page-markers", "--comments", "none"]);
    picked("p1", &[0, 1], &["--track-changes", "reject"]);
    // A pick that reaches no table is a usage error that says what -p
    // takes here, in its own spelling.
    for (spec, spelled) in [("p0", "p0"), ("t0,p0", "p0"), ("p4", "p4")] {
        let out = run(&["--tables", "-p", spec]);
        assert_eq!(out.status.code(), Some(2), "{spec}: {}", stderr(&out));
        assert!(
            stderr(&out).contains(&format!(
                "{spelled} is not in a table; --tables -p takes tN, or a pN inside a table"
            )),
            "{spec}: {}",
            stderr(&out)
        );
        assert!(stdout(&out).is_empty(), "{spec}");
    }
    // An error of the view itself is the view's: its text, its exit code.
    for spec in ["t5", "p99"] {
        let out = run(&["--tables", "-p", spec]);
        let view = run(&["-p", spec]);
        assert_eq!(out.status.code(), view.status.code(), "{spec}");
        assert_eq!(stderr(&out), stderr(&view), "{spec}");
        assert!(stdout(&out).is_empty(), "{spec}");
    }
    // The text alias and the shorthand read it the same.
    let read = run(&["--tables", "-p", "p1"]);
    for alias in [
        jubarte(&["text", file, "--tables", "-p", "p1"], dir.path()),
        jubarte(&[file, "--tables", "-p", "p1"], dir.path()),
    ] {
        assert_eq!(
            (alias.status.code(), stdout(&alias)),
            (read.status.code(), stdout(&read))
        );
    }
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
            entry.contains("ignored by --json, --for-edit and the --tables grids"),
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

/// Reject takes back what a formatting change did, as Word's Reject All
/// does: a run's old properties (`w:rPrChange`), a paragraph's old style
/// and numbering (`w:pPrChange`, its mark's properties and section kept),
/// and a move goes back where it came from. Accept reads them as they are.
#[test]
fn reject_takes_back_formatting_and_moves() {
    let by = r#"w:author="Ann" w:date="2026-10-01T00:00:00Z""#;
    let body = [
        format!(
            r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange w:id="1" {by}><w:rPr><w:i/></w:rPr></w:rPrChange></w:rPr><w:t>styled</w:t></w:r></w:p>"#
        ),
        format!(
            r#"<w:p><w:pPr><w:pStyle w:val="Heading1"/><w:numPr><w:ilvl w:val="0"/><w:numId w:val="1"/></w:numPr><w:rPr><w:b/></w:rPr><w:pPrChange w:id="2" {by}><w:pPr><w:pStyle w:val="Title"/></w:pPr></w:pPrChange></w:pPr><w:r><w:t>restyled</w:t></w:r></w:p>"#
        ),
        format!(
            r#"<w:p><w:moveFrom w:id="3" {by}><w:r><w:t>moved</w:t></w:r></w:moveFrom><w:r><w:t xml:space="preserve"> from</w:t></w:r></w:p>"#
        ),
        format!(
            r#"<w:p><w:r><w:t xml:space="preserve">to </w:t></w:r><w:moveTo w:id="4" {by}><w:r><w:t>moved</w:t></w:r></w:moveTo></w:p>"#
        ),
    ]
    .concat();
    let (dir, path) = fixture("formatted.docx", &docx(&body));
    let file = path.to_str().unwrap();
    let read = |choice: &str| {
        json(&jubarte(
            &["read", file, "--json", "--track-changes", choice],
            dir.path(),
        ))
    };
    let (accepted, rejected) = (read("accept"), read("reject"));
    let run = |snapshot: &Value, flag: &str| snapshot["paragraphs"][0]["runs"][0][flag].clone();
    assert_eq!(
        (run(&accepted, "bold"), run(&accepted, "italic")),
        (true.into(), false.into())
    );
    assert_eq!(
        (run(&rejected, "bold"), run(&rejected, "italic")),
        (false.into(), true.into())
    );
    let restyled = |snapshot: &Value| {
        let p = &snapshot["paragraphs"][1];
        (p["style"].clone(), p["numbered"].clone(), p["text"].clone())
    };
    assert_eq!(
        restyled(&accepted),
        ("Heading1".into(), true.into(), "restyled".into())
    );
    assert_eq!(
        restyled(&rejected),
        ("Title".into(), false.into(), "restyled".into())
    );
    let texts = |snapshot: &Value| {
        [2, 3].map(|i| {
            snapshot["paragraphs"][i]["text"]
                .as_str()
                .unwrap()
                .to_string()
        })
    };
    assert_eq!(texts(&accepted), [" from", "to moved"]);
    assert_eq!(texts(&rejected), ["moved from", "to "]);
    // The ids hold either way.
    for i in 0..4 {
        assert_eq!(
            accepted["paragraphs"][i]["id"],
            rejected["paragraphs"][i]["id"]
        );
    }
}

/// A removed spelling's JSON error carries the task's usage, as every
/// other usage error under --json does (#1290 review).
#[test]
fn a_removed_spelling_under_json_carries_the_usage() {
    let dir = tempfile::tempdir().unwrap();
    let out = jubarte(
        &["compare", "a.docx", "b.docx", "--date=foo", "--json"],
        dir.path(),
    );
    assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    let line: Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(line["error"]["code"], "USAGE");
    assert_eq!(
        line["error"]["message"],
        "--date is removed; use --datetime"
    );
    assert!(
        line["error"]["usage"]
            .as_str()
            .is_some_and(|usage| usage.starts_with("jubarte compare ")),
        "{line}"
    );
    // The host parser hands its host the same line.
    let args: Vec<String> = ["compare", "a.docx", "b.docx", "--date=foo", "--json"]
        .map(String::from)
        .to_vec();
    let parsed: Value =
        serde_json::from_str(&jubarte::cli::parse_json(&args, "jubarte", &[])).unwrap();
    assert_eq!(parsed["json"].as_str().unwrap(), stdout(&out));
}
