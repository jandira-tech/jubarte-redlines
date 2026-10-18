// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Revision refusals retain both review decisions in the caller's grammar.
//! Library cases use in-memory packages and fixed dates; process cases are integration tests.
mod common;
use common::docx::{docx, para};
use jubarte::edit::{EditPlan, apply_plan, v2::PlanV2};
use jubarte::inspect::source_sha256;
use serde_json::{Value, json};
use std::process::Command;

const DATE: &str = "2026-10-17T00:00:00Z";
const STAMP: &str = r#"w:id="7" w:author="Alice" w:date="2026-10-17T00:00:00Z""#;

struct Case {
    body: String,
    multi: bool,
    story: &'static str,
    operation: Value,
    reason: &'static str,
    local: &'static str,
    kind: &'static str,
    accept: &'static str,
    reject: &'static str,
    flags: Vec<&'static str>,
    typed: Vec<&'static str>,
    v2: Value,
}
fn case(site: &str) -> Case {
    let target = json!({"short_id":"p0","long_id":"document.w:p.j:short.p0"});
    let mut c = Case {
        multi: false,
        story: "body",
        body: format!(
            r#"<w:p><w:moveTo {STAMP}><w:r><w:t>beta</w:t></w:r></w:moveTo></w:p>{}"#,
            para("Tail.")
        ),
        operation: json!({"kind":"replace","paragraph":"p0","find":"beta","replacement":"new"}),
        reason: "the text sits inside",
        local: "moveTo",
        kind: "move",
        accept: "keeps the moved content at its destination",
        reject: "restores the moved content at its origin",
        flags: vec!["-p", "p0", "--anchor", "beta", "--content", "new"],
        typed: vec!["replace", "--at", "p0", "--find", "beta", "--text", "new"],
        v2: json!({"kind":"replace","target":target,"find":"beta","text":"new"}),
    };
    match site {
        "range_move" => {}
        "header_move" => {
            c.story = "header1";
            c.operation["paragraph"] = json!("header1");
            c.flags[1] = "header1";
            c.typed[2] = "header1";
            c.v2["target"] = json!({"short_id":"header1","long_id":"header1.w:p.j:short.header1"});
        }
        "range_insertion" | "insert_insertion" => {
            c.body = format!(
                r#"<w:p><w:ins {STAMP}><w:hyperlink w:anchor="terms"><w:r><w:t>beta</w:t></w:r></w:hyperlink></w:ins></w:p>{}"#,
                para("Tail.")
            );
            c.local = "ins";
            c.kind = "insertion";
            c.accept = "keeps the proposed content as plain text";
            c.reject = "removes the proposed content";
            if site == "insert_insertion" {
                c.reason = "the insertion point sits inside";
                c.operation =
                    json!({"kind":"insert","paragraph":"p0","position":"end","text":" new"});
                c.flags = vec!["-p", "p0", "--anchor", "beta", "--content", "beta new"];
                c.typed = vec![
                    "insert", "--kind", "text", "--at", "p0", "--edge", "end", "--text", " new",
                ];
                c.v2 = json!({"kind":"insert","target":target,"edge":"end","text":" new"});
            }
        }
        "insert_move" => {
            c.reason = "the insertion point sits inside";
            c.operation = json!({"kind":"insert","paragraph":"p0","position":"end","text":" new"});
            c.flags = vec!["-p", "p0", "--anchor", "beta", "--content", "beta new"];
            c.typed = vec![
                "insert", "--kind", "text", "--at", "p0", "--edge", "end", "--text", " new",
            ];
            c.v2 = json!({"kind":"insert","target":target,"edge":"end","text":" new"});
        }
        "delete_insertion" | "delete_deletion" | "delete_move" | "delete_multiple" => {
            c.reason = "under keep, a paragraph that holds tracked changes cannot be deleted";
            c.operation = json!({"kind":"delete_paragraph","paragraph":"p0"});
            c.flags = vec!["-p", "p0", "--delete"];
            c.typed = vec!["delete", "--at", "p0", "--whole-block"];
            c.v2 = json!({"kind":"delete_paragraph","target":target});
            if site != "delete_move" {
                let (local, text, kind, accept, reject) =
                    if matches!(site, "delete_insertion" | "delete_multiple") {
                        (
                            "ins",
                            "t",
                            "insertion",
                            "keeps the proposed content as plain text",
                            "removes the proposed content",
                        )
                    } else {
                        (
                            "del",
                            "delText",
                            "deletion",
                            "removes the deleted content",
                            "restores the deleted content",
                        )
                    };
                c.body = format!(
                    r#"<w:p><w:{local} {STAMP}><w:r><w:{text}>beta</w:{text}></w:r></w:{local}></w:p>{}"#,
                    para("Tail.")
                );
                c.local = local;
                c.kind = kind;
                c.accept = accept;
                c.reject = reject;
                if site == "delete_multiple" {
                    c.multi = true;
                    c.body = format!(
                        r#"<w:p><w:ins {STAMP}><w:r><w:t>beta</w:t></w:r></w:ins><w:del w:id="8" w:author="Bob" w:date="2026-10-17T00:00:00Z"><w:r><w:delText>gamma</w:delText></w:r></w:del></w:p>{}"#,
                        para("Tail.")
                    );
                }
            }
        }
        "last_paragraph" | "merge_mark" | "merge_insertion_mark" => {
            c.body = format!(
                r#"<w:p><w:pPr><w:rPr><w:del {STAMP}/></w:rPr></w:pPr><w:r><w:t>First.</w:t></w:r></w:p>{}"#,
                para("beta")
            );
            c.local = "del";
            c.kind = "deletion";
            c.accept = "removes the deleted paragraph break";
            c.reject = "restores the deleted paragraph break";
            if site == "merge_insertion_mark" {
                c.body = c.body.replace("<w:del ", "<w:ins ");
                c.local = "ins";
                c.kind = "insertion";
                c.accept = "keeps the proposed paragraph break";
                c.reject = "removes the proposed paragraph break";
            }
            if site == "last_paragraph" {
                c.reason = "under keep, the last paragraph cannot be deleted when the paragraph before it already has a deleted mark";
                c.operation = json!({"kind":"delete_paragraph","paragraph":"p1"});
                c.flags = vec!["-p", "p1", "--delete"];
                c.typed = vec!["delete", "--at", "p1", "--whole-block"];
                c.v2 = json!({"kind":"delete_paragraph","target":{"short_id":"p1","long_id":"document.w:p.j:short.p1"}});
            } else {
                c.reason = "under keep, a paragraph whose mark is already tracked cannot be merged";
                c.operation = json!({"kind":"merge_paragraphs","paragraph":"p0"});
                c.flags.clear();
                c.typed.clear();
                c.v2 = Value::Null;
            }
        }
        "format_paragraph" | "list_paragraph" | "format_run" => {
            c.kind = "formatting change";
            c.accept = "keeps the proposed formatting";
            c.reject = "restores the previous formatting";
            c.typed.clear();
            c.v2 = Value::Null;
            if site == "format_run" {
                c.local = "rPrChange";
                c.body = format!(
                    r#"<w:p><w:r><w:rPr><w:b/><w:rPrChange {STAMP}><w:rPr/></w:rPrChange></w:rPr><w:t>beta</w:t></w:r></w:p>"#
                );
                c.reason = "under keep, text whose formatting change is still tracked cannot be formatted again";
                c.operation = json!({"kind":"format_run","paragraph":"p0","find":"beta","format":{"italic":true}});
                c.flags = vec!["-p", "p0", "--anchor", "beta", "--style", "italic"];
            } else {
                c.local = "pPrChange";
                c.body = format!(
                    r#"<w:p><w:pPr><w:jc w:val="right"/><w:pPrChange {STAMP}><w:pPr/></w:pPrChange></w:pPr><w:r><w:t>beta</w:t></w:r></w:p>"#
                );
                if site == "format_paragraph" {
                    c.reason = "under keep, a paragraph whose formatting change is still tracked cannot be formatted again";
                    c.operation =
                        json!({"kind":"format_paragraph","paragraph":"p0","style":"Heading1"});
                    c.flags = vec!["-p", "p0", "--style", "Heading1"];
                } else {
                    c.reason = "under keep, a paragraph whose formatting change is still tracked cannot be numbered";
                    c.operation = json!({"kind":"list","paragraphs":["p0"]});
                    c.flags.clear();
                }
            }
        }
        _ => panic!("unknown site {site}"),
    }
    c
}
fn plan(c: &Case, source: &[u8], v2: bool) -> Value {
    if v2 {
        json!({"schema_version":2,"source":{"sha256":source_sha256(source)},"policy":{"track":"on","existing_revisions":"keep","text_view":"final"},"resolve_revisions":{"accept":[],"reject":[]},"author":"Carol Diaz","date":DATE,"scope":{"kind":"all"},"operations":[c.v2]})
    } else {
        json!({"schema_version":1,"source_sha256":source_sha256(source),"author":"Carol Diaz","date":DATE,"existing_revisions":"keep","operations":[c.operation]})
    }
}
fn decision(c: &Case, accept: bool, syntax: &str) -> String {
    let key = if accept { "accept" } else { "reject" };
    let short = if c.story == "body" {
        "r7"
    } else {
        "header1.r7"
    };
    let ids = if c.multi {
        vec![short, "r8"]
    } else {
        vec![short]
    };
    match syntax {
        "flags" => ids
            .iter()
            .map(|id| format!("-p {id} --{key}"))
            .collect::<Vec<_>>()
            .join(" "),
        "typed" => ids
            .iter()
            .map(|id| format!("--{key}-change {id}"))
            .collect::<Vec<_>>()
            .join(" "),
        "v2" | "template" => {
            let part = if c.story == "body" {
                "document"
            } else {
                c.story
            };
            let target = json!({"short_id":short,"long_id":format!("{part}.w:{}.w:id.7",c.local)});
            let targets = if c.multi {
                vec![
                    target,
                    json!({"short_id":"r8","long_id":"document.w:del.w:id.8"}),
                ]
            } else {
                vec![target]
            };
            let decisions = if accept {
                json!({"accept":targets,"reject":[]})
            } else {
                json!({"accept":[],"reject":targets})
            };
            json!({"resolve_revisions":decisions}).to_string()
        }
        _ => json!({"resolve_revisions":{key:{"ids":ids}}}).to_string(),
    }
}
fn expected(c: &Case, syntax: &str) -> String {
    let short = if c.story == "body" {
        "r7"
    } else {
        "header1.r7"
    };
    // Legacy flags validate an insertion's anchor before testing its edge.
    let reason = if syntax == "flags" && c.reason == "the insertion point sits inside" {
        "the text sits inside"
    } else {
        c.reason
    };
    let prefix = if reason.starts_with("the ") {
        format!("{reason} {short}")
    } else {
        format!("{reason}: {short}")
    };
    let more = if c.multi {
        ", r8, a tracked deletion by \"Bob\""
    } else {
        ""
    };
    let accept = if c.multi {
        "keeps the proposed content as plain text for r7 and removes the deleted content for r8"
    } else {
        c.accept
    };
    let reject = if c.multi {
        "removes the proposed content for r7 and restores the deleted content for r8"
    } else {
        c.reject
    };
    format!(
        "{prefix}, a tracked {} by \"Alice\"{more}; {} {accept}; {} {reject}; refresh the edit against the resulting document, or comment on the revision",
        c.kind,
        decision(c, true, syntax),
        decision(c, false, syntax)
    )
}
fn check(site: &str, syntax: &str) {
    let c = case(site);
    let source = if site == "format_paragraph" {
        use common::docx::{Part, W_NS, docx_with};
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
    } else if c.story == "body" {
        docx(&c.body)
    } else {
        use common::docx::{Part, W_NS, docx_with_sect};
        let header = format!(r#"<w:hdr xmlns:w="{W_NS}">{}</w:hdr>"#, c.body);
        docx_with_sect(
            &para("Body."),
            &[Part {
                name: "word/header1.xml",
                content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml",
                rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/header",
                xml: &header,
            }],
            r#"<w:headerReference w:type="default" r:id="rIdX0"/>"#,
        )
    };
    let mut q = plan(&c, &source, matches!(syntax, "v2" | "typed" | "template"));
    let actual_message;
    if syntax == "library" {
        let e = apply_plan(&source, &EditPlan::from_json(&q.to_string()).unwrap()).unwrap_err();
        assert_eq!(e.message, expected(&c, syntax));
        let preview =
            jubarte::edit::preview_plan(&source, &EditPlan::from_json(&q.to_string()).unwrap())
                .unwrap_err();
        assert_eq!(preview.message, e.message);
        actual_message = e.message;
    } else {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("in.docx"), &source).unwrap();
        if syntax == "template" {
            let generated = Command::new(env!("CARGO_BIN_EXE_jubarte"))
                .current_dir(dir.path())
                .args([
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
                ])
                .output()
                .unwrap();
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
        let mut args = vec!["edit"];
        match syntax {
            "flags" => {
                args.push("in.docx");
                args.extend(&c.flags);
                args.extend([
                    "-a",
                    "Carol Diaz",
                    "-d",
                    DATE,
                    "--existing-revisions",
                    "keep",
                    "--dry-run",
                ]);
            }
            "typed" => {
                args.extend(&c.typed);
                args.extend([
                    "in.docx",
                    "--author",
                    "Carol Diaz",
                    "--datetime",
                    DATE,
                    "--dry-run",
                    "--json",
                ]);
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
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        let stdout = String::from_utf8(output.stdout).unwrap();
        let response: Value = serde_json::from_str(stdout.lines().last().unwrap_or_default())
            .unwrap_or_else(|e| {
                panic!("{e}: {stdout}: {}", String::from_utf8_lossy(&output.stderr))
            });
        let error = if syntax == "flags" {
            &response
        } else {
            &response["error"]
        };
        let actual = error["message"].as_str().unwrap();
        let wanted = expected(&c, syntax);
        // V2 insertion-side errors preserve their existing diagnostic prefix.
        let wanted = if site.starts_with("insert_") && matches!(syntax, "typed" | "v2") {
            format!("requested insertion side cannot be preserved: {wanted}")
        } else {
            wanted
        };
        assert_eq!(actual, wanted);
        actual_message = actual.to_string();
        if matches!(syntax, "v2" | "template") {
            assert!(
                !actual.contains("\"ids\"")
                    && !actual.contains("--accept")
                    && !actual.contains("--reject")
            );
        }
        let actions = error["next_actions"]
            .as_array()
            .expect("both structured review choices");
        assert_eq!(actions.len(), 2);
        for (action, key) in actions.iter().zip(["accept", "reject"]) {
            let argv = action["argv"]
                .as_array()
                .expect("argv array, never shell text");
            assert!(argv.iter().all(Value::is_string));
            assert!(argv.contains(&json!(format!("--{key}-change"))));
            assert!(argv.contains(&json!(if c.story == "body" {
                "r7"
            } else {
                "header1.r7"
            })));
        }
    }
    // The JSON suggestions parse in the caller's parser and settle exactly r7.
    if matches!(syntax, "library" | "v1" | "v2") {
        let fragments: Vec<Value> = actual_message
            .split("; ")
            .filter(|s| s.starts_with('{'))
            .map(|s| {
                serde_json::Deserializer::from_str(s)
                    .into_iter::<Value>()
                    .next()
                    .unwrap()
                    .unwrap()
            })
            .collect();
        assert_eq!(fragments.len(), 2);
        for fragment in fragments {
            let mut settlement = q.clone();
            settlement["operations"] = json!([]);
            settlement["resolve_revisions"] = fragment["resolve_revisions"].clone();
            let output = if syntax == "v2" {
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
            assert!(jubarte::changes::list_changes(&output).unwrap().is_empty());
        }
    }
}
macro_rules! route_tests {
    ($site:literal; $($test:ident => $syntax:literal),+ $(,)?) => {$ (#[test] fn $test(){check($site,$syntax);})+};
}
route_tests!("range_move"; range_move_library=>"library",range_move_flags=>"flags",range_move_v1=>"v1",range_move_typed=>"typed",range_move_v2=>"v2");
route_tests!("insert_move"; insert_move_library=>"library",insert_move_flags=>"flags",insert_move_v1=>"v1",insert_move_typed=>"typed",insert_move_v2=>"v2");
route_tests!("delete_insertion"; delete_insertion_library=>"library",delete_insertion_flags=>"flags",delete_insertion_v1=>"v1",delete_insertion_typed=>"typed",delete_insertion_v2=>"v2");
route_tests!("delete_deletion"; delete_deletion_library=>"library",delete_deletion_flags=>"flags",delete_deletion_v1=>"v1",delete_deletion_typed=>"typed",delete_deletion_v2=>"v2");
route_tests!("delete_move"; delete_move_library=>"library",delete_move_flags=>"flags",delete_move_v1=>"v1",delete_move_typed=>"typed",delete_move_v2=>"v2");
route_tests!("last_paragraph"; last_paragraph_library=>"library",last_paragraph_flags=>"flags",last_paragraph_v1=>"v1",last_paragraph_typed=>"typed",last_paragraph_v2=>"v2");
route_tests!("merge_mark"; merge_mark_library=>"library",merge_mark_v1=>"v1");
route_tests!("format_paragraph"; format_paragraph_library=>"library",format_paragraph_flags=>"flags",format_paragraph_v1=>"v1");
route_tests!("list_paragraph"; list_paragraph_library=>"library",list_paragraph_v1=>"v1");
route_tests!("format_run"; format_run_library=>"library",format_run_flags=>"flags",format_run_v1=>"v1");

route_tests!("range_insertion"; range_insertion_library=>"library",range_insertion_flags=>"flags",range_insertion_v1=>"v1",range_insertion_typed=>"typed",range_insertion_v2=>"v2");
route_tests!("insert_insertion"; insert_insertion_library=>"library",insert_insertion_flags=>"flags",insert_insertion_v1=>"v1",insert_insertion_typed=>"typed",insert_insertion_v2=>"v2");

route_tests!("range_move"; template_move=>"template");
route_tests!("range_insertion"; template_insertion=>"template");

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

route_tests!("delete_multiple"; delete_multiple_library=>"library",delete_multiple_flags=>"flags",delete_multiple_v1=>"v1",delete_multiple_typed=>"typed",delete_multiple_v2=>"v2");

route_tests!("header_move"; header_move_library=>"library",header_move_flags=>"flags",header_move_v1=>"v1",header_move_typed=>"typed",header_move_v2=>"v2");

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

route_tests!("merge_insertion_mark"; merge_insertion_mark_library=>"library",merge_insertion_mark_v1=>"v1");

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
    assert_eq!(error["message"], expected(&c, "flags"));
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
fn loss_cli(flags: &[&str], json: bool) -> std::process::Output {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("source.docx"), loss_source()).unwrap();
    std::fs::write(dir.path().join("plan.json"),r#"{"schema_version":1,"author":"Alice","date":"2026-10-18T12:06:00Z","existing_revisions":"keep","resolve_revisions":{"reject":{"ids":["r1"]}},"operations":[]}"#).unwrap();
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_jubarte"));
    cmd.current_dir(dir.path())
        .args(["edit", "source.docx"])
        .args(flags)
        .args(["-o", "out.docx"]);
    if json {
        cmd.arg("--json");
    }
    let out = cmd.output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let comments = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .current_dir(dir.path())
        .args(["comments", "out.docx"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&comments.stdout).contains("0 comment(s)"));
    out
}
fn assert_loss_text(flags: &[&str]) {
    let out = loss_cli(flags, false);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        text.contains("removed c0 (Bob) and its reply c1:"),
        "{text}"
    );
}
fn assert_loss_json(flags: &[&str]) {
    let out = loss_cli(flags, true);
    let text = String::from_utf8_lossy(&out.stdout);
    let summary: Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
    assert_eq!(
        summary["removed_comments"],
        json!([{"id":"c0","author":"Bob","replies":["c1"]}]),
        "{text}"
    );
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
