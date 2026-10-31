// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Revision refusals name the revision and every route around it in the
//! caller's grammar (#1108), beside the structured review choices (#1147).
//! Library cases use in-memory packages and fixed dates; process cases are
//! integration tests.
mod common;
use common::docx::{docx, para};
use common::validity::assert_word_valid_package;
use jubarte::changes::{ChangeFilter, accept_changes, list_changes, reject_changes};
use jubarte::comments::list_comments;
use jubarte::edit::{EditPlan, apply_plan, v2::PlanV2};
use jubarte::inspect::source_sha256;
use serde_json::{Value, json};
use std::path::Path;
use std::process::{Command, Output};

const DATE: &str = "2026-10-17T00:00:00Z";
const STAMP: &str = r#"w:id="7" w:author="Alice" w:date="2026-10-17T00:00:00Z""#;

/// The accept and reject routes' labels and costs, by what the revision did.
type Routes = [(&'static str, &'static str); 2];
const MOVED: Routes = [
    (
        r#"keep the move (erases "Alice"'s move)"#,
        "the words stay where they moved, unmarked",
    ),
    (
        r#"undo it silently (erases "Alice"'s move)"#,
        "the words go back where they were, no trace",
    ),
];
const DELETED: Routes = [
    (
        r#"let it go (erases "Alice"'s deletion)"#,
        "their words leave the document for good",
    ),
    (
        r#"put it back silently (erases "Alice"'s deletion)"#,
        "their deletion disappears, no trace",
    ),
];
const INSERTED: Routes = [
    (
        r#"keep it (erases "Alice"'s insertion)"#,
        "their words stay, unmarked",
    ),
    (
        r#"take it out silently (erases "Alice"'s insertion)"#,
        "their insertion disappears, no trace",
    ),
];
const FORMATTED: Routes = [
    (
        r#"keep their formatting (erases "Alice"'s formatting)"#,
        "their formatting stays, unmarked",
    ),
    (
        r#"undo it silently (erases "Alice"'s formatting)"#,
        "the old formatting comes back, no trace",
    ),
];

/// The hyperlink refusal: structural, so it names no revision.
const HYPERLINK: &str =
    "sits inside a hyperlink; edit the text on either side of it, or comment on it";

struct Case {
    body: String,
    story: &'static str,
    operation: Value,
    flags: Vec<&'static str>,
    typed: Vec<&'static str>,
    v2: Value,
    /// What the refusal says up to the revision's id.
    lead: &'static str,
    /// The revision element, for its long id.
    local: &'static str,
    /// The text the revision holds; a paragraph mark holds none.
    text: &'static str,
    /// What the revision did.
    verb: &'static str,
    routes: Routes,
    /// The paragraph a v1 or command-line comment route names: the one
    /// that holds the revision.
    paragraph: &'static str,
    /// The structured choices' consequences (#1147).
    accept: &'static str,
    reject: &'static str,
}
fn case(site: &str) -> Case {
    let target = json!({"short_id":"p0","long_id":"document.w:p.j:short.p0"});
    let mut c = Case {
        story: "body",
        body: format!(
            r#"<w:p><w:moveTo {STAMP}><w:r><w:t>beta</w:t></w:r></w:moveTo></w:p>{}"#,
            para("Tail.")
        ),
        operation: json!({"kind":"replace","paragraph":"p0","find":"beta","replacement":"new"}),
        flags: vec!["-p", "p0", "--anchor", "beta", "--content", "new"],
        typed: vec!["replace", "--at", "p0", "--find", "beta", "--text", "new"],
        v2: json!({"kind":"replace","target":target,"find":"beta","text":"new"}),
        lead: "the text is inside",
        local: "moveTo",
        text: "beta",
        verb: "moved here",
        routes: MOVED,
        paragraph: "p0",
        accept: "keeps the moved content at its destination",
        reject: "restores the moved content at its origin",
    };
    let insert = |c: &mut Case| {
        c.operation = json!({"kind":"insert","paragraph":"p0","position":"end","text":" new"});
        c.flags = vec!["-p", "p0", "--anchor", "beta", "--content", "beta new"];
        c.typed = vec![
            "insert", "--kind", "text", "--at", "p0", "--edge", "end", "--text", " new",
        ];
        c.v2 = json!({"kind":"insert","target":target,"edge":"end","text":" new"});
    };
    let delete = |c: &mut Case, paragraph: &'static str| {
        c.operation = json!({"kind":"delete_paragraph","paragraph":paragraph});
        c.flags = vec!["-p", paragraph, "--delete"];
        c.typed = vec!["delete", "--at", paragraph, "--whole-block"];
        c.v2 = json!({"kind":"delete_paragraph","target":{"short_id":paragraph,"long_id":format!("document.w:p.j:short.{paragraph}")}});
    };
    match site {
        "range_move" => {}
        "header_move" => {
            c.story = "header1";
            c.paragraph = "header1";
            c.operation["paragraph"] = json!("header1");
            c.flags[1] = "header1";
            c.typed[2] = "header1";
            c.v2["target"] = json!({"short_id":"header1","long_id":"header1.w:p.j:short.header1"});
        }
        "insert_move" => {
            c.lead = "the insertion point is inside";
            insert(&mut c);
        }
        "delete_move" => {
            c.lead = "under keep, a paragraph that holds a move cannot be deleted yet: the move is";
            delete(&mut c, "p0");
        }
        // Text inside a hyperlink, inside another author's insertion.
        "range_insertion" | "insert_insertion" => {
            c.body = format!(
                r#"<w:p><w:ins {STAMP}><w:hyperlink w:anchor="terms"><w:r><w:t>beta</w:t></w:r></w:hyperlink></w:ins></w:p>{}"#,
                para("Tail.")
            );
            c.lead = "the text";
            if site == "insert_insertion" {
                c.lead = "the insertion point";
                insert(&mut c);
            }
        }
        // A paragraph holding another author's revisions, deleted (#1136).
        "delete_insertion" | "delete_deletion" | "delete_multiple" => {
            delete(&mut c, "p0");
            let carrier = |local: &str, text: &str, stamp: &str, words: &str| {
                format!(r#"<w:{local} {stamp}><w:r><w:{text}>{words}</w:{text}></w:r></w:{local}>"#)
            };
            let content = match site {
                "delete_insertion" => carrier("ins", "t", STAMP, "beta"),
                "delete_deletion" => carrier("del", "delText", STAMP, "beta"),
                _ => {
                    carrier("ins", "t", STAMP, "beta")
                        + &carrier(
                            "del",
                            "delText",
                            r#"w:id="8" w:author="Bob" w:date="2026-10-17T00:00:00Z""#,
                            "gamma",
                        )
                }
            };
            // Bob's insertion outside the deleted paragraph stays.
            c.body = format!(
                r#"<w:p>{content}</w:p><w:p><w:ins w:id="9" w:author="Bob" w:date="2026-10-17T00:00:00Z"><w:r><w:t>Tail.</w:t></w:r></w:ins></w:p>"#
            );
        }
        "last_paragraph" | "merge_mark" | "merge_insertion_mark" => {
            c.body = format!(
                r#"<w:p><w:pPr><w:rPr><w:del {STAMP}/></w:rPr></w:pPr><w:r><w:t>First.</w:t></w:r></w:p>{}"#,
                para("beta")
            );
            c.local = "del";
            c.text = "";
            c.verb = "deleted";
            c.routes = DELETED;
            c.accept = "removes the deleted paragraph break";
            c.reject = "restores the deleted paragraph break";
            if site == "merge_insertion_mark" {
                c.body = c.body.replace("<w:del ", "<w:ins ");
                c.local = "ins";
                c.verb = "inserted";
                c.routes = INSERTED;
                c.accept = "keeps the proposed paragraph break";
                c.reject = "removes the proposed paragraph break";
            }
            if site == "last_paragraph" {
                // The last paragraph of a content control, not its
                // container's: a container's final paragraph is emptied or
                // carried away instead (#1670).
                c.body = format!(
                    r#"<w:sdt><w:sdtPr><w:id w:val="1"/></w:sdtPr><w:sdtContent>{}</w:sdtContent></w:sdt>{}"#,
                    c.body,
                    para("Tail.")
                );
                c.lead = "under keep, the last paragraph of a content control or a custom XML element cannot be deleted when the paragraph before it already has a deleted mark: that mark is";
                delete(&mut c, "p1");
            } else {
                c.lead = "under keep, a paragraph whose mark is already tracked cannot be merged: that mark is";
                c.operation = json!({"kind":"merge_paragraphs","paragraph":"p0"});
                c.flags.clear();
                c.typed.clear();
                c.v2 = Value::Null;
            }
        }
        "format_paragraph" | "list_paragraph" | "format_run" => {
            c.verb = "reformatted";
            c.routes = FORMATTED;
            c.accept = "keeps the proposed formatting";
            c.reject = "restores the previous formatting";
            c.typed.clear();
            c.v2 = Value::Null;
            if site == "format_run" {
                c.local = "rPrChange";
                c.body = format!(
                    r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange {STAMP}><w:rPr/></w:rPrChange></w:rPr><w:t>beta</w:t></w:r></w:p>"#
                );
                c.lead = "under keep, text whose formatting change is still tracked cannot be formatted again: that change is";
                c.operation = json!({"kind":"format_run","paragraph":"p0","find":"beta","format":{"italic":true}});
                c.flags = vec!["-p", "p0", "--anchor", "beta", "--style", "italic"];
            } else {
                c.local = "pPrChange";
                c.body = format!(
                    r#"<w:p><w:pPr><w:jc w:val="right"/><w:pPrChange {STAMP}><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>beta</w:t></w:r></w:p>"#
                );
                if site == "format_paragraph" {
                    c.lead = "under keep, a paragraph whose formatting change is still tracked cannot be formatted again: that change is";
                    c.operation =
                        json!({"kind":"format_paragraph","paragraph":"p0","style":"Heading1"});
                    c.flags = vec!["-p", "p0", "--style", "Heading1"];
                } else {
                    c.lead = "under keep, a paragraph whose formatting change is still tracked cannot be numbered: that change is";
                    c.operation = json!({"kind":"list","paragraphs":["p0"]});
                    c.flags.clear();
                }
            }
        }
        _ => panic!("unknown site {site}"),
    }
    c
}
/// The package `c` edits: its body, or its header; Heading1 is styled.
fn source(c: &Case) -> Vec<u8> {
    use common::docx::{Part, W_NS, docx_with, docx_with_sect};
    if c.story != "body" {
        let header = format!(r#"<w:hdr xmlns:w="{W_NS}">{}</w:hdr>"#, c.body);
        return docx_with_sect(
            &para("Body."),
            &[Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: &header,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        );
    }
    if c.operation["style"] != json!("Heading1") {
        return docx(&c.body);
    }
    let styles = format!(
        r#"<w:styles xmlns:w="{W_NS}"><w:style w:type="paragraph" w:styleId="Heading1"><w:name w:val="Heading1"/></w:style></w:styles>"#
    );
    docx_with(
        &c.body,
        &[Part {
            name: "word/styles.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles",
            xml: &styles,
        }],
    )
}
fn plan(c: &Case, source: &[u8], v2: bool) -> Value {
    if v2 {
        json!({"schema_version":2,"source":{"sha256":source_sha256(source)},"policy":{"track":"on","existing_revisions":"keep","text_view":"final"},"resolve_revisions":{"accept":[],"reject":[]},"author":"Carol Diaz","date":DATE,"scope":{"kind":"all"},"operations":[c.v2]})
    } else {
        json!({"schema_version":1,"source_sha256":source_sha256(source),"author":"Carol Diaz","date":DATE,"existing_revisions":"keep","operations":[c.operation]})
    }
}
/// `c`'s request in `syntax` as `jubarte` arguments: a dry run, or
/// published to out.docx.
fn args(c: &Case, syntax: &str, publish: bool) -> Vec<&'static str> {
    let mut args = vec!["edit"];
    match syntax {
        "flags" => {
            args.push("in.docx");
            args.extend(c.flags.iter().copied());
            args.extend([
                "-a",
                "Carol Diaz",
                "-d",
                DATE,
                "--existing-revisions",
                "keep",
            ]);
        }
        "typed" => {
            args.extend(c.typed.iter().copied());
            args.extend([
                "in.docx",
                "--author",
                "Carol Diaz",
                "--datetime",
                DATE,
                "--json",
            ]);
        }
        _ => args.extend(["apply", "in.docx", "--plan", "plan.json", "--json"]),
    }
    if publish {
        args.extend(["-o", "out.docx"]);
    } else {
        args.push("--dry-run");
    }
    args
}
fn run(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}
/// A refused process's error: the flags' last JSONL row, or the
/// envelope's `error`.
fn refused(output: &Output, syntax: &str, args: &[&str]) -> Value {
    assert!(!output.status.success(), "{args:?}");
    let stdout = String::from_utf8(output.stdout.clone()).unwrap();
    let response: Value = serde_json::from_str(stdout.lines().last().unwrap_or_default())
        .unwrap_or_else(|e| panic!("{e}: {stdout}: {}", String::from_utf8_lossy(&output.stderr)));
    if syntax == "flags" {
        response
    } else {
        response["error"].clone()
    }
}
fn short(c: &Case) -> &'static str {
    if c.story == "body" {
        "r7"
    } else {
        "header1.r7"
    }
}
/// The refusal's first line: what was refused, then the revision by id,
/// text, verb and author.
fn header(c: &Case, syntax: &str) -> String {
    let lead = match (c.lead, syntax) {
        // The flags spell an insertion as a replacement of its anchor.
        ("the insertion point is inside", "flags") => "the text is inside".to_string(),
        // V2 insertion-side errors keep their diagnostic prefix.
        ("the insertion point is inside", "typed" | "v2") => {
            format!("requested insertion side cannot be preserved: {}", c.lead)
        }
        _ => c.lead.to_string(),
    };
    let text = if c.text.is_empty() {
        String::new()
    } else {
        format!("{:?} ", c.text)
    };
    format!("{lead} {}, {text}{} by \"Alice\".", short(c), c.verb)
}
/// The `verb` (`accept` or `reject`) route, as `syntax` spells it.
fn settle(c: &Case, verb: &str, syntax: &str) -> String {
    let id = short(c);
    match syntax {
        "flags" => format!("-p {id} --{verb}"),
        "typed" => format!("--{verb}-change {id}"),
        "v2" | "template" => {
            let part = if c.story == "body" {
                "document"
            } else {
                c.story
            };
            let target = format!(
                r#"[{{"short_id":"{id}","long_id":"{part}.w:{}.w:id.7"}}]"#,
                c.local
            );
            let (accept, reject) = if verb == "accept" {
                (target.as_str(), "[]")
            } else {
                ("[]", target.as_str())
            };
            format!(r#""resolve_revisions":{{"accept":{accept},"reject":{reject}}}"#)
        }
        _ => format!(r#""resolve_revisions":{{"{verb}":{{"ids":["{id}"]}}}}"#),
    }
}
/// The comment route, as `syntax` spells it. A v2 route names the refused
/// operation's own target.
fn comment(c: &Case, syntax: &str) -> String {
    match syntax {
        "flags" | "typed" => format!(r#"add -p {} --anchor "…" --content "…""#, c.paragraph),
        "v2" | "template" => format!(
            r#"{{"kind":"comment","target":{{"short_id":{},"long_id":{}}},"find":"…","text":"…"}}"#,
            c.v2["target"]["short_id"], c.v2["target"]["long_id"]
        ),
        _ => format!(
            r#"{{"kind":"comment","paragraph":"{}","find":"…","text":"…"}}"#,
            c.paragraph
        ),
    }
}
/// A route line as its label, its fragment, its cost, and the columns the
/// fragment and the cost start at.
fn route(line: &str) -> (&str, &str, Option<&str>, usize, Option<usize>) {
    let body = line
        .strip_prefix("  ")
        .unwrap_or_else(|| panic!("indented: {line:?}"));
    let (label, rest) = body.split_once(':').unwrap();
    let column = line.len() - rest.trim_start().len();
    let rest = rest.trim_start();
    match rest.strip_suffix(')').and_then(|r| r.rsplit_once(" (")) {
        Some((fragment, cost)) => (
            label,
            fragment.trim_end(),
            Some(cost),
            column,
            Some(line.len() - cost.len() - 2),
        ),
        None => (label, rest, None, column, None),
    }
}
fn check(site: &str, syntax: &str) {
    let c = case(site);
    let source = source(&c);
    let mut q = plan(&c, &source, matches!(syntax, "v2" | "typed" | "template"));
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), &source).unwrap();
    let mut error = Value::Null;
    let message = if syntax == "library" {
        let e = apply_plan(&source, &EditPlan::from_json(&q.to_string()).unwrap()).unwrap_err();
        let preview =
            jubarte::edit::preview_plan(&source, &EditPlan::from_json(&q.to_string()).unwrap())
                .unwrap_err();
        assert_eq!(preview.message, e.message);
        e.message
    } else {
        if syntax == "template" {
            let generated = run(
                dir.path(),
                &[
                    "edit",
                    "template",
                    "in.docx",
                    "--select",
                    "p0",
                    "--author",
                    "Carol Diaz",
                    "--datetime",
                    DATE,
                    "-o",
                    "-",
                ],
            );
            assert!(
                generated.status.success(),
                "{}",
                String::from_utf8_lossy(&generated.stderr)
            );
            q = serde_json::from_slice(&generated.stdout).unwrap();
            assert_eq!(q["operations"].as_array().unwrap().len(), 1);
            q["operations"][0]["text"] = json!("new");
        }
        std::fs::write(dir.path().join("plan.json"), q.to_string()).unwrap();
        let args = args(&c, syntax, false);
        error = refused(&run(dir.path(), &args), syntax, &args);
        error["message"].as_str().unwrap().to_string()
    };
    // The revision, then accept, reject and a comment, each with its cost,
    // lined up in two columns. An insertion also offers strike first, where
    // the syntax has it: their insertion stays, with your deletion showing.
    let lines: Vec<&str> = message.lines().collect();
    let strike_line = c.verb == "inserted" && matches!(syntax, "library" | "v1" | "flags");
    assert_eq!(lines.len(), if strike_line { 5 } else { 4 }, "{message}");
    assert_eq!(lines[0], header(&c, syntax), "{message}");
    let route_lines: Vec<&str> = if strike_line {
        let (label, fragment, cost, _, _) = route(lines[1]);
        assert_eq!(
            (label, cost),
            (
                "strike it",
                Some("their insertion stays, your deletion shows")
            ),
            "{message}"
        );
        assert!(fragment.contains("strike"), "{message}");
        lines[2..].to_vec()
    } else {
        lines[1..].to_vec()
    };
    let [accept, reject] = c.routes;
    let wanted = [
        (accept.0, settle(&c, "accept", syntax), Some(accept.1)),
        (reject.0, settle(&c, "reject", syntax), Some(reject.1)),
        ("or comment on it", comment(&c, syntax), None),
    ];
    let routes: Vec<_> = route_lines.iter().map(|line| route(line)).collect();
    for ((label, fragment, cost, _, _), (want_label, want_fragment, want_cost)) in
        routes.iter().zip(&wanted)
    {
        assert_eq!(
            (*label, *fragment, *cost),
            (*want_label, want_fragment.as_str(), *want_cost),
            "{message}"
        );
    }
    assert!(routes.iter().all(|r| r.3 == routes[0].3), "{message}");
    assert_eq!(routes[0].4, routes[1].4, "{message}");
    if matches!(syntax, "v2" | "template") {
        assert!(
            !message.contains("\"ids\"")
                && !message.contains("--accept")
                && !message.contains("--reject"),
            "{message}"
        );
    }
    if syntax != "library" {
        let actions = error["next_actions"]
            .as_array()
            .expect("both structured review choices");
        // Another author's deletion or insertion lists the keeping move
        // first. Accept and reject stay, and each is marked destructive.
        let keeping = match c.verb {
            "deleted" => Some("restore"),
            "inserted" => Some("strike"),
            _ => None,
        };
        let settle_from = usize::from(keeping.is_some());
        assert_eq!(actions.len(), 2 + settle_from, "{actions:?}");
        if let Some(flag) = keeping {
            let argv = actions[0]["argv"]
                .as_array()
                .expect("argv array, never shell text");
            assert!(argv.contains(&json!(format!("--{flag}"))), "{actions:?}");
            assert!(!argv.iter().any(|arg| arg == "template"), "{actions:?}");
        }
        let settle = &actions[settle_from..];
        for (action, key) in settle.iter().zip(["accept", "reject"]) {
            let argv = action["argv"]
                .as_array()
                .expect("argv array, never shell text");
            assert!(argv.iter().all(Value::is_string));
            assert!(argv.contains(&json!(format!("--{key}-change"))));
            assert!(argv.contains(&json!(short(&c))));
            assert_eq!(action["destructive"], true, "{action}");
        }
        if c.verb == "deleted" {
            let purpose = settle[1]["purpose"].as_str().unwrap();
            assert!(
                purpose.contains("rejecting erases their change"),
                "{purpose}"
            );
            assert!(!purpose.contains("restores the deleted"), "{purpose}");
        }
    }
    match syntax {
        // The JSON routes parse in the caller's parser and settle exactly r7.
        "library" | "v1" | "v2" | "template" => {
            for (_, fragment, _) in &wanted[..2] {
                let route: Value = serde_json::from_str(&format!("{{{fragment}}}")).unwrap();
                let mut settlement = q.clone();
                settlement["operations"] = json!([]);
                settlement["resolve_revisions"] = route["resolve_revisions"].clone();
                let output = if matches!(syntax, "v2" | "template") {
                    jubarte::edit::v2_apply::apply_plan(
                        &source,
                        &PlanV2::from_json(&settlement.to_string()).unwrap(),
                    )
                    .unwrap()
                    .redline
                } else {
                    apply_plan(
                        &source,
                        &EditPlan::from_json(&settlement.to_string()).unwrap(),
                    )
                    .unwrap()
                    .redline
                };
                assert!(list_changes(&output).unwrap().is_empty());
            }
            serde_json::from_str::<Value>(&wanted[2].1).unwrap();
        }
        // The flags settle r7 on their own.
        "flags" => {
            for (_, fragment, _) in &wanted[..2] {
                let mut args = vec!["edit", "in.docx"];
                args.extend(fragment.split(' '));
                args.extend(["-a", "Carol Diaz", "-d", DATE, "-o", "settled.docx"]);
                let output = run(dir.path(), &args);
                assert!(
                    output.status.success(),
                    "{args:?}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let settled = std::fs::read(dir.path().join("settled.docx")).unwrap();
                assert!(list_changes(&settled).unwrap().is_empty(), "{args:?}");
                std::fs::remove_file(dir.path().join("settled.docx")).unwrap();
            }
        }
        // The typed options run on the refused command itself and settle
        // the revision: the refusal never comes back (#1308 review).
        _ => {
            for (_, fragment, _) in &wanted[..2] {
                let mut args: Vec<&str> = args(&c, syntax, false);
                args.extend(fragment.split(' '));
                let output = run(dir.path(), &args);
                let said = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                assert_ne!(output.status.code(), Some(2), "{args:?}: {said}");
                assert!(
                    !said.contains(&header(&c, syntax)),
                    "{args:?} left the refusal in place: {said}"
                );
            }
        }
    }
}
/// A structural refusal (text inside a hyperlink) names no revision and
/// carries no review choices, even inside another author's insertion.
fn structural(site: &str, syntax: &str) {
    let c = case(site);
    let source = source(&c);
    let message = if syntax == "library" {
        apply_plan(
            &source,
            &EditPlan::from_json(&plan(&c, &source, false).to_string()).unwrap(),
        )
        .unwrap_err()
        .message
    } else {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("in.docx"), &source).unwrap();
        let mut q = plan(&c, &source, matches!(syntax, "v2" | "typed" | "template"));
        if syntax == "template" {
            let generated = run(
                dir.path(),
                &[
                    "edit",
                    "template",
                    "in.docx",
                    "--select",
                    "p0",
                    "--author",
                    "Carol Diaz",
                    "--datetime",
                    DATE,
                    "-o",
                    "-",
                ],
            );
            assert!(generated.status.success());
            q = serde_json::from_slice(&generated.stdout).unwrap();
            q["operations"][0]["text"] = json!("new");
        }
        std::fs::write(dir.path().join("plan.json"), q.to_string()).unwrap();
        let args = args(&c, syntax, false);
        let error = refused(&run(dir.path(), &args), syntax, &args);
        assert!(error.get("next_actions").is_none(), "{error}");
        error["message"].as_str().unwrap().to_string()
    };
    let wanted = match (c.lead, syntax) {
        ("the insertion point", "flags") => format!("the text {HYPERLINK}"),
        ("the insertion point", "typed" | "v2") => {
            format!("requested insertion side cannot be preserved: the insertion point {HYPERLINK}")
        }
        (lead, _) => format!("{lead} {HYPERLINK}"),
    };
    assert_eq!(message, wanted);
}
/// Under keep, deleting a paragraph that holds other authors' insertions
/// or deletions succeeds in every syntax (#1136): their revisions stay,
/// attributed, beside ours, and the package stays Word-valid.
fn deletes(site: &str, syntax: &str) {
    let c = case(site);
    let source = source(&c);
    let redline = if syntax == "library" {
        apply_plan(
            &source,
            &EditPlan::from_json(&plan(&c, &source, false).to_string()).unwrap(),
        )
        .unwrap()
        .redline
    } else {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("in.docx"), &source).unwrap();
        std::fs::write(
            dir.path().join("plan.json"),
            plan(&c, &source, syntax == "v2").to_string(),
        )
        .unwrap();
        let args = args(&c, syntax, true);
        let output = run(dir.path(), &args);
        assert!(
            output.status.success(),
            "{args:?}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        std::fs::read(dir.path().join("out.docx")).unwrap()
    };
    assert_word_valid_package(&redline);
    let after = list_changes(&redline).unwrap();
    for old in list_changes(&source).unwrap() {
        assert!(
            after.iter().any(|new| new.kind == old.kind
                && new.target == old.target
                && new.author == old.author
                && new.text == old.text),
            "{old:?} lost: {after:?}"
        );
    }
    let texts = |bytes: &[u8]| -> Vec<String> {
        jubarte::inspect::paragraphs(bytes)
            .unwrap()
            .into_iter()
            .map(|p| p.text)
            .collect()
    };
    let all = ChangeFilter::default();
    assert_eq!(texts(&accept_changes(&redline, &all).unwrap()), ["Tail."]);
    assert_eq!(
        texts(&reject_changes(&redline, &all).unwrap()),
        texts(&reject_changes(&source, &all).unwrap())
    );
    if syntax == "library" {
        // The clean output loses the deleted paragraph with its revisions,
        // and keeps every revision outside it (#1308 review).
        let outside: Vec<_> = list_changes(&source)
            .unwrap()
            .into_iter()
            .filter(|old| old.paragraph.as_deref() == Some("p1"))
            .map(|old| (old.kind, old.author, old.text))
            .collect();
        assert_eq!(outside.len(), 1, "{outside:?}");
        let v1 = EditPlan::from_json(&plan(&c, &source, false).to_string()).unwrap();
        let v2 = PlanV2::from_json(&plan(&c, &source, true).to_string()).unwrap();
        for clean in [
            apply_plan(&source, &v1).unwrap().clean,
            jubarte::edit::v2_apply::apply_plan(&source, &v2)
                .unwrap()
                .clean,
        ] {
            assert_word_valid_package(&clean);
            let kept: Vec<_> = list_changes(&clean)
                .unwrap()
                .into_iter()
                .map(|new| (new.kind, new.author, new.text))
                .collect();
            assert_eq!(kept, outside);
            assert_eq!(texts(&clean), ["Tail."]);
        }
    }
}
macro_rules! site_tests {
    ($check:ident, $site:literal; $($test:ident => $syntax:literal),+ $(,)?) => {$ (#[test] fn $test(){$check($site,$syntax);})+};
}
site_tests!(check, "range_move"; range_move_library=>"library",range_move_flags=>"flags",range_move_v1=>"v1",range_move_typed=>"typed",range_move_v2=>"v2");
site_tests!(check, "header_move"; header_move_library=>"library",header_move_flags=>"flags",header_move_v1=>"v1",header_move_typed=>"typed",header_move_v2=>"v2");
site_tests!(check, "insert_move"; insert_move_library=>"library",insert_move_flags=>"flags",insert_move_v1=>"v1",insert_move_typed=>"typed",insert_move_v2=>"v2");
site_tests!(check, "delete_move"; delete_move_library=>"library",delete_move_flags=>"flags",delete_move_v1=>"v1",delete_move_typed=>"typed",delete_move_v2=>"v2");
site_tests!(check, "last_paragraph"; last_paragraph_library=>"library",last_paragraph_flags=>"flags",last_paragraph_v1=>"v1",last_paragraph_typed=>"typed",last_paragraph_v2=>"v2");
site_tests!(check, "merge_mark"; merge_mark_library=>"library",merge_mark_v1=>"v1");
site_tests!(check, "merge_insertion_mark"; merge_insertion_mark_library=>"library",merge_insertion_mark_v1=>"v1");
site_tests!(check, "format_paragraph"; format_paragraph_library=>"library",format_paragraph_flags=>"flags",format_paragraph_v1=>"v1");
site_tests!(check, "list_paragraph"; list_paragraph_library=>"library",list_paragraph_v1=>"v1");
site_tests!(check, "format_run"; format_run_library=>"library",format_run_flags=>"flags",format_run_v1=>"v1");
site_tests!(check, "range_move"; template_move=>"template");

site_tests!(structural, "range_insertion"; range_insertion_library=>"library",range_insertion_flags=>"flags",range_insertion_v1=>"v1",range_insertion_typed=>"typed",range_insertion_v2=>"v2");
site_tests!(structural, "insert_insertion"; insert_insertion_library=>"library",insert_insertion_flags=>"flags",insert_insertion_v1=>"v1",insert_insertion_typed=>"typed",insert_insertion_v2=>"v2");
site_tests!(structural, "range_insertion"; template_insertion=>"template");

site_tests!(deletes, "delete_insertion"; delete_insertion_library=>"library",delete_insertion_flags=>"flags",delete_insertion_v1=>"v1",delete_insertion_typed=>"typed",delete_insertion_v2=>"v2");
site_tests!(deletes, "delete_deletion"; delete_deletion_library=>"library",delete_deletion_flags=>"flags",delete_deletion_v1=>"v1",delete_deletion_typed=>"typed",delete_deletion_v2=>"v2");
site_tests!(deletes, "delete_multiple"; delete_multiple_library=>"library",delete_multiple_flags=>"flags",delete_multiple_v1=>"v1",delete_multiple_typed=>"typed",delete_multiple_v2=>"v2");

fn structured_choices(syntax: &str) {
    let c = case("range_move");
    let source = docx(&c.body);
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), &source).unwrap();
    std::fs::write(
        dir.path().join("plan.json"),
        plan(&c, &source, syntax == "v2").to_string(),
    )
    .unwrap();
    let mut args = vec!["edit"];
    match syntax {
        "flags" => {
            args.push("in.docx");
            args.extend(&c.flags);
            args.extend(["--dry-run"]);
        }
        "typed" => {
            args.extend(&c.typed);
            args.extend(["in.docx", "--dry-run", "--json"]);
        }
        _ => args.extend([
            "apply",
            "in.docx",
            "--plan",
            "plan.json",
            "--dry-run",
            "--json",
        ]),
    }
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args(args)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let response: Value = serde_json::from_str(stdout.lines().last().unwrap()).unwrap();
    let error = if syntax == "flags" {
        &response
    } else {
        &response["error"]
    };
    assert!(
        error["next_actions"].is_array(),
        "both decisions must be argument arrays: {error}"
    );
    let actions = error["next_actions"].as_array().unwrap();
    assert_eq!(actions.len(), 2);
    for (action, key) in actions.iter().zip(["accept", "reject"]) {
        let argv: Vec<String> = serde_json::from_value(action["argv"].clone()).unwrap();
        assert_eq!(argv[0], "jubarte");
        assert!(argv.contains(&format!("--{key}-change")));
        assert!(
            action["purpose"]
                .as_str()
                .unwrap()
                .contains(if key == "accept" { c.accept } else { c.reject })
        );
        assert_eq!(action["destructive"], true, "{action}");
        let generated = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .current_dir(dir.path())
            .args(&argv[1..])
            .output()
            .unwrap();
        assert!(
            generated.status.success(),
            "{}",
            String::from_utf8_lossy(&generated.stderr)
        );
        let generated = PlanV2::from_json(std::str::from_utf8(&generated.stdout).unwrap()).unwrap();
        let decisions = if key == "accept" {
            &generated.resolve_revisions.accept
        } else {
            &generated.resolve_revisions.reject
        };
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].short_id, "r7");
        assert_eq!(decisions[0].long_id, "document.w:moveTo.w:id.7");
    }
    assert_eq!(std::fs::read(dir.path().join("in.docx")).unwrap(), source);
}
#[test]
fn structured_flags_choices() {
    structured_choices("flags");
}
#[test]
fn structured_v1_choices() {
    structured_choices("v1");
}
#[test]
fn structured_typed_choices() {
    structured_choices("typed");
}
#[test]
fn structured_v2_choices() {
    structured_choices("v2");
}

#[test]
fn every_failed_flag_operation_uses_flags() {
    let dir = tempfile::tempdir().unwrap();
    let body = format!(
        r#"<w:p><w:moveTo {STAMP}><w:r><w:t>beta</w:t></w:r></w:moveTo></w:p><w:p><w:moveTo w:id="8" w:author="Bob" w:date="2026-10-17T00:00:00Z"><w:r><w:t>gamma</w:t></w:r></w:moveTo></w:p>"#
    );
    std::fs::write(dir.path().join("in.docx"), docx(&body)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args([
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--anchor",
            "beta",
            "--content",
            "new",
            "-p",
            "p1",
            "--anchor",
            "gamma",
            "--content",
            "other",
            "--dry-run",
            "--author",
            "Carol Diaz",
            "--datetime",
            DATE,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let rows: Vec<Value> = stdout
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let ops: Vec<_> = rows.iter().filter(|r| r["ev"] == "op").collect();
    assert_eq!(ops.len(), 2);
    for (row, id) in ops.iter().zip(["r7", "r8"]) {
        let message = row["message"].as_str().unwrap();
        assert!(message.contains(&format!("-p {id} --accept")), "{message}");
        assert!(message.contains(&format!("-p {id} --reject")), "{message}");
        assert!(!message.contains("resolve_revisions"), "{message}");
    }
}

/// A rewrite's insertion that first touches another author's move and
/// then lands on the run after it was not refused by that move: a later
/// range's refusal in the same rewrite carries no revision data from it
/// (#1308 review).
#[test]
fn a_retried_insertion_leaves_no_revision_data_on_a_later_refusal() {
    let body = format!(
        r#"<w:p><w:moveTo w:id="8" w:author="Bob" w:date="{DATE}"><w:r><w:t>a,</w:t></w:r></w:moveTo><w:r><w:t>b;</w:t></w:r><w:hyperlink w:anchor="dest"><w:r><w:t>c</w:t></w:r></w:hyperlink></w:p>"#
    );
    let source = docx(&body);
    let plan = json!({"schema_version":1,"source_sha256":source_sha256(&source),"author":"Carol Diaz","date":DATE,"existing_revisions":"keep","operations":[{"kind":"rewrite","paragraph":{"index":0},"text":"a,x b;d"}]});
    let e = apply_plan(&source, &EditPlan::from_json(&plan.to_string()).unwrap()).unwrap_err();
    let error = serde_json::to_value(&e).unwrap();
    assert!(
        error["message"].as_str().unwrap().contains(HYPERLINK),
        "the later range is refused for the link: {error}"
    );
    assert!(
        error
            .get("revision_refusals")
            .is_none_or(|r| r.as_array().is_some_and(Vec::is_empty)),
        "no revision refused this rewrite: {error}"
    );
}

#[test]
fn structured_engine_error_retains_the_real_revision_pair() {
    let c = case("range_move");
    let source = docx(&c.body);
    let e = apply_plan(
        &source,
        &EditPlan::from_json(&plan(&c, &source, false).to_string()).unwrap(),
    )
    .unwrap_err();
    let error = serde_json::to_value(e).unwrap();
    assert_eq!(
        error["revision_refusals"][0]["revisions"][0]["target"],
        json!({"short_id":"r7","long_id":"document.w:moveTo.w:id.7"})
    );
}

#[test]
fn recording_a_flag_request_uses_flag_choices() {
    let c = case("range_move");
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), docx(&c.body)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args([
            "edit",
            "in.docx",
            "--plan",
            "recorded.json",
            "-p",
            "p0",
            "--anchor",
            "beta",
            "--content",
            "new",
            "--dry-run",
            "--author",
            "Carol Diaz",
            "--datetime",
            DATE,
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let error: Value = serde_json::from_str(stdout.lines().last().unwrap()).unwrap();
    let message = error["message"].as_str().unwrap();
    assert_eq!(message.lines().next(), Some(header(&c, "flags").as_str()));
    for verb in ["accept", "reject"] {
        assert!(message.contains(&settle(&c, verb, "flags")), "{message}");
    }
    assert!(!dir.path().join("recorded.json").exists());
}

#[test]
fn primary_nonrevision_error_has_no_secondary_review_choices() {
    let c = case("range_move");
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), docx(&c.body)).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args([
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--anchor",
            "missing",
            "--content",
            "new",
            "-p",
            "p0",
            "--anchor",
            "beta",
            "--content",
            "new",
            "--author",
            "Carol Diaz",
            "--datetime",
            DATE,
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let error: Value = serde_json::from_str(stdout.lines().last().unwrap()).unwrap();
    assert_eq!(error["code"], "ANCHOR_NOT_FOUND");
    assert!(
        error.get("next_actions").is_none(),
        "a secondary revision must not supply the primary error's review choices: {error}"
    );
}

/// An anchor the caller typed may read like a refusal's header: the error
/// it gets is still no revision refusal, and carries no review choices
/// (#1308 review).
#[test]
fn an_anchor_quoting_a_refusal_header_has_no_review_choices() {
    let c = case("range_move");
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("in.docx"), docx(&c.body)).unwrap();
    let quoted = format!("{} {}", c.lead, short(&c));
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args([
            "edit",
            "in.docx",
            "-p",
            "p0",
            "--anchor",
            &quoted,
            "--content",
            "new",
            "-p",
            "p0",
            "--anchor",
            "beta",
            "--content",
            "new",
            "--author",
            "Carol Diaz",
            "--datetime",
            DATE,
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let error: Value = serde_json::from_str(stdout.lines().last().unwrap()).unwrap();
    assert_eq!(error["code"], "ANCHOR_NOT_FOUND");
    let first = error["message"].as_str().unwrap().lines().next().unwrap();
    assert!(first.contains(&quoted), "{error}");
    assert!(error.get("next_actions").is_none(), "{error}");
}

/// The refusal's data names the syntax its message quotes, whoever builds
/// the transaction (#1308 review): a typed command's prepared plan records
/// `typed_action` without its caller restating it.
#[test]
fn the_refusal_data_records_the_quoted_syntax() {
    let c = case("range_move");
    let source = source(&c);
    let syntax_of = |e: jubarte::edit::EditError| {
        serde_json::to_value(&e).unwrap()["revision_refusals"][0]["syntax"].clone()
    };
    let v2 = PlanV2::from_json(&plan(&c, &source, true).to_string()).unwrap();
    let typed = jubarte::edit::prepare_v2(
        &source,
        &v2,
        &jubarte::edit::PrepareOptions {
            syntax: jubarte::edit::PlanSyntax::Typed,
            ..jubarte::edit::PrepareOptions::default()
        },
    )
    .expect_err("refused");
    assert_eq!(syntax_of(typed), "typed_action");
    let plain = jubarte::edit::v2_apply::apply_plan(&source, &v2).expect_err("refused");
    assert_eq!(syntax_of(plain), "v2_plan");
    let v1 = EditPlan::from_json(&plan(&c, &source, false).to_string()).unwrap();
    let flags = jubarte::edit::check_plan_quoting(&source, &v1, jubarte::edit::PlanSyntax::Flags)
        .expect_err("refused");
    assert_eq!(syntax_of(flags), "legacy_flags");
    let v1 = apply_plan(&source, &v1).expect_err("refused");
    assert_eq!(syntax_of(v1), "v1_plan");
}

fn loss_source() -> Vec<u8> {
    let source = docx(
        &(para("Services")
            + &para("The Provider shall perform the Services.")
            + &para("Next paragraph.")),
    );
    let bob = EditPlan::from_json(r#"{"schema_version":1,"author":"Bob","date":"2026-10-18T12:00:00Z","operations":[{"kind":"insert","paragraph":"p1","position":"end","text":" Bob added this sentence.","comment":"Bob's reason."}]}"#).unwrap();
    let bob = apply_plan(&source, &bob).unwrap();
    let alice = EditPlan::from_json(r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:05:00Z","existing_revisions":"keep","operations":[{"kind":"reply_comment","comment_id":"c0","text":"Alice disagrees."}]}"#).unwrap();
    apply_plan(&bob.redline, &alice).unwrap().redline
}
fn loss_cli(flags: &[&str], channel: &str, mode: &str) -> (std::process::Output, Option<Value>) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), loss_source()).unwrap();
    let plan = json!({"schema_version":1,"author":"Alice","date":"2026-10-18T12:06:00Z","existing_revisions":"keep","removed_anchor_comments":mode,"resolve_revisions":{"reject":{"ids":["r1"]}},"operations":[]});
    std::fs::write(dir.path().join("plan.json"), plan.to_string()).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jubarte"));
    cmd.current_dir(dir.path())
        .args(["edit", "source.docx"])
        .args(flags);
    match channel {
        "bundle" => {
            cmd.args(["--out-dir", "bundle"]);
        }
        "preview" => {
            cmd.arg("--dry-run");
        }
        "single" => {
            cmd.args(["-o", "out.docx"]);
        }
        _ => panic!("unknown report channel"),
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report = match channel {
        "bundle" => Some(std::fs::read_to_string(dir.path().join("bundle/report.jsonl")).unwrap()),
        "preview" => Some(String::from_utf8(out.stdout.clone()).unwrap()),
        _ => None,
    };
    if channel != "preview" {
        let path = if channel == "bundle" {
            "bundle/redline.docx"
        } else {
            "out.docx"
        };
        let comments = list_comments(&std::fs::read(dir.path().join(path)).unwrap()).unwrap();
        assert_eq!(comments.len(), if mode == "keep" { 2 } else { 0 });
    } else {
        assert!(!dir.path().join("out.docx").exists());
        assert!(!dir.path().join("bundle").exists());
    }
    let summary = report.map(|text| {
        text.lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|event| event["ev"] == "summary")
            .expect("summary event")
    });
    (out, summary)
}
fn assert_loss_text(flags: &[&str]) {
    let (out, _) = loss_cli(flags, "single", "remove");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("removed c0 (Bob) and its reply c1: anchor removed by rejecting r1"),
        "{text}"
    );
}
fn assert_loss_json(flags: &[&str]) {
    for channel in ["bundle", "preview"] {
        let (_, summary) = loss_cli(flags, channel, "remove");
        let summary = summary.unwrap();
        assert_eq!(
            summary["removed_comments"],
            json!([{"id":"c0","author":"Bob","replies":["c1"],"why":"anchor removed by rejecting r1"}]),
            "{channel}: {summary}"
        );
        assert_eq!(summary["kept_comments"], json!([]));
    }
}
#[test]
fn removed_anchor_keep_reports_in_existing_json_channels() {
    for channel in ["bundle", "preview"] {
        let (_, summary) = loss_cli(&["--plan", "plan.json"], channel, "keep");
        let summary = summary.unwrap();
        assert_eq!(summary["removed_comments"], json!([]));
        assert_eq!(
            summary["kept_comments"],
            json!([{"id":"c0","replies":["c1"],"anchor":"empty"}])
        );
    }
}
#[test]
fn removed_anchor_plan_discloses_loss_in_text() {
    assert_loss_text(&["--plan", "plan.json"]);
}
#[test]
fn removed_anchor_flags_disclose_loss_in_text() {
    assert_loss_text(&["-p", "r1", "--reject"]);
}
#[test]
fn removed_anchor_plan_discloses_loss_in_json() {
    assert_loss_json(&["--plan", "plan.json"]);
}
#[test]
fn removed_anchor_flags_disclose_loss_in_json() {
    assert_loss_json(&["-p", "r1", "--reject"]);
}

#[test]
fn removed_anchor_typed_envelope_and_text_disclose_threads() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), loss_source()).unwrap();
    for mode in ["remove", "keep"] {
        let plan = json!({"schema_version":1,"author":"Alice","date":DATE,"existing_revisions":"keep","source_sha256":source_sha256(&loss_source()),"removed_anchor_comments":mode,"resolve_revisions":{"reject":{"ids":["r1"]}},"operations":[]});
        std::fs::write(dir.path().join("plan.json"), plan.to_string()).unwrap();
        for format in ["text", "json", "jsonl"] {
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_jubarte"));
            cmd.current_dir(dir.path()).args([
                "edit",
                "apply",
                "source.docx",
                "--plan",
                "plan.json",
                "-o",
                "out.docx",
                "--force",
            ]);
            if format != "text" {
                cmd.arg(format!("--{format}"));
            }
            let out = cmd.output().unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let text = String::from_utf8_lossy(&out.stdout);
            if format == "text" {
                let text = format!("{text}{}", String::from_utf8_lossy(&out.stderr));
                assert!(
                    text.contains(&format!(
                        "{} c0",
                        if mode == "keep" { "kept" } else { "removed" }
                    )) && text.contains("c1"),
                    "{text}"
                );
            } else {
                let envelope: Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
                let data = &envelope["data"];
                let key = if mode == "keep" {
                    "kept_comments"
                } else {
                    "removed_comments"
                };
                assert_eq!(data[key][0]["id"], "c0", "{text}");
                assert_eq!(data[key][0]["replies"], json!(["c1"]), "{text}");
            }
        }
    }
}

#[test]
fn legacy_edit_and_add_have_no_json_flag() {
    for verb in ["edit", "add"] {
        let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args([verb, "--help"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        // Option headings have six spaces; wrapped descriptions have ten.
        // `jubarte capabilities --json` can wrap onto a continuation line.
        assert!(
            !help.lines().any(|line| line.starts_with("      --json")),
            "{verb} must use --dry-run or --out-dir for JSONL: {help}"
        );
        let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args([verb, "source.docx", "--json"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("unexpected argument '--json'"), "{stderr}");
    }
}
