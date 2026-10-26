// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration checks for focused discovery through the shipped CLI.
mod common;

use common::docx::{Part, W_NS, docx, docx_with, para};
use jubarte::edit::for_edit::{ForEdit, ForEditRequest, for_edit};
use serde_json::{Value, json};
use std::process::Command;

fn query(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["capabilities", "--json"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

/// `capabilities` without `--json`: the prose an agent reads.
fn text(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .args(["capabilities"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Two body paragraphs, `p0` "Zero" and `p1` "Six", on a Letter page with
/// one-inch margins: enough document for every operation to reach its own
/// checks and be refused there, never at the kind gate.
fn tiny_source() -> Vec<u8> {
    docx(&format!("{}{}", para("Zero"), para("Six")))
}

/// One operation through the real v1 dispatcher; the error text on refusal.
fn dispatch(operation: &Value) -> Result<(), String> {
    let plan = json!({
        "schema_version": 1,
        "author": "Ann Counsel",
        "date": "2026-10-01T09:00:00Z",
        "operations": [operation],
    });
    jubarte::edit::apply_plan_json(&tiny_source(), &plan.to_string())
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// A smallest operation of `kind` built from its published v1 fields.
fn sample_operation(kind: &str) -> Value {
    let answer = jubarte::capabilities::discovery("cli", Some(kind), None);
    let profile = answer["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["plan_version"] == 1)
        .expect("v1 profile");
    let mut op = json!({"kind": kind});
    for field in profile["required"].as_array().into_iter().flatten() {
        let field = field.as_str().unwrap();
        if field != "kind" {
            op[field] = minimal(&profile["fields"][field]);
        }
    }
    for (field, value) in [
        ("paragraph", json!("p0")),
        ("paragraphs", json!(["p0"])),
        ("find", json!("Zero")),
        ("after", json!("Zero")),
        ("before", json!("Zero")),
        ("replacement", json!("Nine")),
        ("text", json!("Nine")),
        ("rows", json!([["cell"]])),
        ("levels", json!(3)),
        ("level", json!(0)),
    ] {
        if op.get(field).is_some() {
            op[field] = value;
        }
    }
    op
}

/// A value that satisfies one field schema, as
/// `src/capabilities/discovery.rs` builds for its dispatch probe.
fn minimal(schema: &Value) -> Value {
    if let Some(value) = schema.get("const") {
        return value.clone();
    }
    if let Some(values) = schema["enum"].as_array() {
        return values[0].clone();
    }
    if schema["type"] != "object" {
        for key in ["anyOf", "oneOf"] {
            if let Some(alternatives) = schema[key].as_array() {
                return minimal(&alternatives[0]);
            }
        }
    }
    match schema["type"].as_str().unwrap_or("object") {
        "string" => json!(""),
        "integer" | "number" => schema.get("minimum").cloned().unwrap_or(json!(0)),
        "boolean" => json!(false),
        "array" => json!([]),
        _ => json!({}),
    }
}

#[test]
fn reply_comment_fields_from_one_focused_call() {
    let answer = query(&["--operation", "reply_comment"]);
    assert_eq!(answer["supported"], true);
    let v2 = answer["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["plan_version"] == 2)
        .unwrap();
    assert_eq!(
        v2["required"],
        serde_json::json!(["kind", "target", "text"])
    );
    assert_eq!(
        v2["fields"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["kind", "target", "text"]
    );
    let v1 = answer["profiles"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["plan_version"] == 1)
        .unwrap();
    assert!(jsonschema::is_valid(
        &v1["fields"]["comment_id"],
        &serde_json::json!("c27")
    ));
    assert!(jsonschema::is_valid(
        &v1["fields"]["comment_id"],
        &serde_json::json!(27)
    ));
    assert!(v2["fields"]["target"]["properties"]["short_id"].is_object());
    assert_eq!(v2["dispatched"], true);
    assert!(
        v2["support"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["status"] == "conditional")
    );
}

#[test]
fn unsupported_field_and_operation_offer_nearest_names() {
    let answer = query(&["--operation", "reply_comment", "--field", "tex"]);
    assert_eq!(answer["supported"], false);
    assert_eq!(answer["nearest"][0], "text");
    let answer = query(&["--operation", "reply_coment"]);
    assert_eq!(answer["supported"], false);
    assert_eq!(answer["nearest"][0], "reply_comment");
    let answer = query(&["--field", "imaginary"]);
    assert_eq!(answer["supported"], false);
}

#[test]
fn v2_discovery_equals_schema_and_parser() {
    let schema: Value = serde_json::from_str(jubarte::edit::v2::SCHEMA_JSON).unwrap();
    let mut kinds = Vec::new();
    for reference in schema["properties"]["operations"]["items"]["oneOf"]
        .as_array()
        .unwrap()
    {
        let definition = schema
            .pointer(
                reference["$ref"]
                    .as_str()
                    .unwrap()
                    .strip_prefix('#')
                    .unwrap(),
            )
            .unwrap();
        let kind = definition["properties"]["kind"]["const"].as_str().unwrap();
        kinds.push(kind);
        let answer = query(&["--operation", kind]);
        let profile = answer["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["plan_version"] == 2)
            .unwrap();
        assert_eq!(
            profile["fields"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>(),
            definition["properties"]
                .as_object()
                .unwrap()
                .keys()
                .collect::<Vec<_>>(),
            "{kind}"
        );
        assert_eq!(profile["required"], definition["required"], "{kind}");
    }
    let mut plan: Value =
        serde_json::from_str(include_str!("fixtures/edit-v2/template.generated.json")).unwrap();
    plan["operations"] = parser_operations();
    let parsed = jubarte::edit::v2::PlanV2::from_json(&plan.to_string()).unwrap();
    let parsed: Value = serde_json::from_str(&parsed.to_json()).unwrap();
    let parser_kinds: std::collections::BTreeSet<_> = parsed["operations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|op| op["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds.into_iter().collect::<std::collections::BTreeSet<_>>(),
        parser_kinds
    );
}

#[test]
fn each_runtime_advertises_only_dispatched_operations() {
    let cli = query(&["--matrix"]);
    assert_eq!(cli["runtime"], "cli");
    for runtime in ["cli", "python", "wasm"] {
        let matrix = jubarte::capabilities::discovery(runtime, None, None);
        assert_eq!(matrix["runtime"], runtime);
        let advertised: std::collections::BTreeSet<String> = matrix["edit_operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(advertised.len(), 26);
        // The advertised list is checked against the real dispatcher, not a
        // second copy of the same catalog: every advertised kind must clear
        // the dispatcher's kind gate on a real document, and the gate must
        // refuse a name the list does not carry.
        for kind in &advertised {
            match dispatch(&sample_operation(kind)) {
                Ok(()) => {}
                Err(e) => assert!(
                    !e.contains("unknown kind"),
                    "{runtime} advertises {kind}, but the dispatcher refuses it: {e}"
                ),
            }
        }
        let stranger = json!({"kind": "definitely_not_an_operation", "text": "x"});
        let refused = dispatch(&stranger).expect_err("the dispatcher refuses an unknown kind");
        assert!(refused.contains("unknown kind"), "{refused}");
        assert!(!advertised.contains("definitely_not_an_operation"));
        // PlanV2 dispatch is a runtime property: the CLI routes it end to
        // end, the bindings have no entry point for it. The v2 rows
        // themselves are probed against prepare_v2 below.
        if runtime == "cli" {
            assert_eq!(matrix["edit_plan_versions"], serde_json::json!([1, 2]));
        } else {
            assert_eq!(matrix["edit_plan_versions"], serde_json::json!([1]));
            assert!(
                matrix["profiles"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|p| p["plan_version"] == 2)
                    .all(|p| p["support"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|r| r["status"] == "unsupported"))
            );
        }
    }
}

fn parser_operations() -> Value {
    let t = serde_json::json!({"short_id":"p0","long_id":"document.w:p.j:short.p0"});
    let c = serde_json::json!({"short_id":"c0","long_id":"comments.w:comment.w:id.0"});
    serde_json::json!([
        {"kind":"replace","target":t,"find":"old","text":"","occurrence":1},
        {"kind":"insert","target":t,"text":"X","before_text":"old","occurrence":2,"attached_comment":{"text":"note"}},
        {"kind":"delete","target":t,"find":"old"},
        {"kind":"rewrite","target":t,"before_sha256":"a".repeat(64),"text":""},
        {"kind":"insert_paragraph","target":t,"position":"before","text":""},
        {"kind":"delete_paragraph","target":t},
        {"kind":"comment","target":t,"text":"note","find":"old","occurrence":1},
        {"kind":"reply_comment","target":c,"text":"reply"},
        {"kind":"resolve_comment","target":c,"done":false},
        {"kind":"edit_comment","target":c,"text":"new"},
        {"kind":"delete_comment","target":c,"include_replies":false}
    ])
}

/// Two body paragraphs with one existing comment: `p0` "Zero" carries
/// `c0`, `p1` "Six" is plain. Every v2 kind resolves a target on it; `c0`
/// is the plan author's, so `edit_comment` may change it (#1264).
fn commented_source() -> Vec<u8> {
    let comments = format!(
        r#"<w:comments xmlns:w="{W_NS}"><w:comment w:id="0" w:author="Ann Counsel"><w:p><w:r><w:t>Note</w:t></w:r></w:p></w:comment></w:comments>"#
    );
    docx_with(
        &format!(
            r#"<w:p><w:commentRangeStart w:id="0"/><w:r><w:t>Zero</w:t></w:r><w:commentRangeEnd w:id="0"/><w:r><w:commentReference w:id="0"/></w:r></w:p>{}"#,
            para("Six")
        ),
        &[Part {
            name: "word/comments.xml",
            content_type: "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
            rel_type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/comments",
            xml: &comments,
        }],
    )
}

/// One v2 plan wrapping `operation` under the row's track and
/// remaining-revision policy, bound to `source`'s digest.
fn v2_plan(source: &[u8], track: &str, existing: &str, operation: &Value) -> String {
    json!({
        "schema_version": 2,
        "source": {"sha256": jubarte::inspect::source_sha256(source)},
        "policy": {"track": track, "existing_revisions": existing, "text_view": "final"},
        "resolve_revisions": {"accept": [], "reject": []},
        "author": "Ann Counsel",
        "date": "2026-10-01T09:00:00Z",
        "scope": {"kind": "all"},
        "operations": [operation],
    })
    .to_string()
}

/// One smallest operation of each v2 kind against `commented_source`,
/// addressed by the paired ids `read --for-edit` prints for it. Text edits
/// target `p1`, which carries no comment: erasing a comment's whole
/// anchor is a source-structure refusal, not a policy one.
fn v2_operation(kind: &str, view: &ForEdit) -> Value {
    let row = |short: &str| {
        view.paragraphs
            .iter()
            .find(|row| row.target.short_id == short)
            .unwrap_or_else(|| panic!("no for-edit row {short}"))
    };
    let p0 = row("p0");
    let p1 = row("p1");
    let paragraph = |row: &jubarte::edit::for_edit::ForEditRow| json!({"short_id": row.target.short_id, "long_id": row.target.long_id});
    let before_sha256 = || {
        p1.working
            .as_ref()
            .and_then(|working| working.before_sha256.clone())
            .expect("p1 working hash")
    };
    let comment = || {
        let comment = p0.comments.first().expect("c0 anchored on p0");
        json!({"short_id": comment.short_id, "long_id": comment.long_id})
    };
    match kind {
        "replace" => {
            json!({"kind": "replace", "target": paragraph(p1), "find": "Six", "text": "Nine"})
        }
        "insert" => json!({"kind": "insert", "target": paragraph(p1), "text": "!",
            "after_text": "Six"}),
        "delete" => json!({"kind": "delete", "target": paragraph(p1), "find": "Six"}),
        "rewrite" => json!({"kind": "rewrite", "target": paragraph(p1),
            "before_sha256": before_sha256(), "text": "Seven"}),
        "insert_paragraph" => json!({"kind": "insert_paragraph", "target": paragraph(p1),
            "position": "after", "text": "New paragraph."}),
        "delete_paragraph" => {
            json!({"kind": "delete_paragraph", "target": paragraph(p1)})
        }
        "comment" => json!({"kind": "comment", "target": paragraph(p0), "text": "A note."}),
        "reply_comment" => {
            json!({"kind": "reply_comment", "target": comment(), "text": "Reply."})
        }
        "resolve_comment" => json!({"kind": "resolve_comment", "target": comment(), "done": true}),
        "edit_comment" => {
            json!({"kind": "edit_comment", "target": comment(), "text": "Edited."})
        }
        "delete_comment" => {
            json!({"kind": "delete_comment", "target": comment(), "include_replies": false})
        }
        other => panic!("no v2 base operation for {other}"),
    }
}

/// The v2 counterpart of the v1 probe above: every v2 row the CLI
/// advertises is probed through `jubarte::edit::prepare_v2`, the entry
/// point the CLI's `edit apply --plan` and inline routes dispatch. A row
/// marked supported must be accepted on a real fixture; a row marked
/// unsupported must refuse with the code it carries. Python and wasm have
/// no PlanV2 entry point, so they stay on `[1]` with every v2 row
/// unsupported.
#[test]
fn cli_v2_rows_match_prepare_v2_and_bindings_stay_on_v1() {
    let matrix = jubarte::capabilities::discovery("cli", None, None);
    assert_eq!(matrix["edit_plan_versions"], json!([1, 2]));
    assert_eq!(query(&[])["edit_plan_versions"], json!([1, 2]));
    for runtime in ["python", "wasm"] {
        let bound = jubarte::capabilities::discovery(runtime, None, None);
        assert_eq!(bound["edit_plan_versions"], json!([1]));
        for profile in bound["profiles"].as_array().unwrap() {
            if profile["plan_version"] == 2 {
                assert_eq!(profile["dispatched"], false, "{runtime}");
                assert!(
                    profile["support"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .all(|r| r["status"] == "unsupported"),
                    "{runtime}"
                );
            }
        }
    }
    let source = commented_source();
    let view = for_edit(
        &source,
        &ForEditRequest {
            all: true,
            ..Default::default()
        },
    )
    .unwrap();
    for profile in matrix["profiles"].as_array().unwrap() {
        if profile["plan_version"] != 2 {
            continue;
        }
        let kind = profile["operation"].as_str().unwrap();
        let operation = v2_operation(kind, &view);
        for row in profile["support"].as_array().unwrap() {
            // v2 has a track policy: a v1-only refusal must not leak into
            // a v2 row.
            let conditions = row["conditions"].as_str().unwrap();
            assert!(
                !conditions.contains("v1 has no track policy")
                    && !conditions.contains("the legacy emitter"),
                "{kind}: {conditions}"
            );
            assert!(
                conditions.contains("requires the bound source digest, target pairs naming one entity, nonconflicting edits and supported source structure; composed edits may not remove a comment's whole anchor"),
                "{kind}: {conditions}"
            );
            if row["existing_revisions"] == "refuse" {
                assert!(
                    conditions.contains("; no remaining revision anywhere after selected decisions, including unknown revision forms"),
                    "{kind} refuse: {conditions}"
                );
            }
            if row["existing_revisions"] == "keep" {
                assert!(
                    conditions.contains("; remaining revisions stay beside the new edits"),
                    "{kind} keep: {conditions}"
                );
            }
            let plan = v2_plan(
                &source,
                row["track"].as_str().unwrap(),
                row["existing_revisions"].as_str().unwrap(),
                &operation,
            );
            let parsed = jubarte::edit::v2::PlanV2::from_json(&plan).unwrap();
            // Match a v2 CLI request without --require-change.
            let probed = jubarte::edit::prepare_v2(
                &source,
                &parsed,
                &jubarte::edit::PrepareOptions {
                    require_change: false,
                    exact_text: true,
                    role: jubarte::edit::PrimaryRole::Clean,
                    syntax: jubarte::edit::PlanSyntax::V1,
                    skip_failed: false,
                },
            );
            let row_name = format!(
                "{kind} target={} track={} existing_revisions={}",
                row["target"], row["track"], row["existing_revisions"]
            );
            match probed {
                Ok(_) => assert_ne!(
                    row["status"], "unsupported",
                    "{row_name} is marked unsupported but prepare_v2 accepts it"
                ),
                Err(e) => {
                    assert_eq!(
                        row["status"], "unsupported",
                        "{row_name} is advertised but prepare_v2 refuses it: {e}"
                    );
                    let code = row["code"].as_str().unwrap_or_else(|| {
                        panic!("{row_name} is unsupported without a refusal code")
                    });
                    assert_eq!(e.code, code, "{row_name}");
                }
            }
        }
    }
}

#[test]
fn focused_field_filters_and_v1_only_fields() {
    let answer = query(&["--operation", "insert", "--field", "edge"]);
    assert_eq!(answer["supported"], true);
    assert_eq!(answer["profiles"].as_array().unwrap().len(), 1);
    assert!(answer["profiles"][0]["constraints"]["oneOf"].is_array());
    let answer = query(&["--field", "include_replies"]);
    assert_eq!(answer["profiles"].as_array().unwrap().len(), 1);
    assert_eq!(answer["profiles"][0]["operation"], "delete_comment");
    let answer = query(&["--operation", "insert_image"]);
    assert_eq!(answer["profiles"].as_array().unwrap().len(), 1);
    let profile = &answer["profiles"][0];
    assert_eq!(profile["plan_version"], 1);
    assert!(profile["fields"]["image_base64"].is_object());
    assert!(
        profile["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("image_base64"))
    );
    assert!(
        profile["support"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["existing_revisions"] == "keep" && r["status"] == "unsupported")
    );
}

#[test]
fn focused_text_and_unsupported_text_are_actionable() {
    for (args, expected) in [
        (
            vec!["capabilities", "--operation", "reply_comment"],
            vec!["reply_comment", "target", "text", "required", "unsupported"],
        ),
        (
            vec![
                "capabilities",
                "--operation",
                "reply_comment",
                "--field",
                "tex",
            ],
            vec!["unsupported", "text"],
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        for word in expected {
            assert!(text.contains(word), "missing {word}: {text}");
        }
    }
}

#[test]
fn default_output_preserves_the_existing_manifest() {
    let answer = query(&[]);
    assert_eq!(
        answer,
        serde_json::to_value(jubarte::capabilities::capabilities("cli")).unwrap()
    );
    assert!(answer.get("profiles").is_none());
    let output = Command::new(env!("CARGO_BIN_EXE_jubarte"))
        .arg("capabilities")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        jubarte::capabilities::capabilities_text(&jubarte::capabilities::capabilities("cli"))
    );
}

/// One bound the dispatcher enforces on a numeric (or text-length) field of
/// a v1 operation: probes inside the bound and one step outside it, plus the
/// bounds the profile must publish for that field.
struct Bound {
    kind: &'static str,
    path: &'static [&'static str],
    /// How a probe value is written into the operation.
    shape: Shape,
    inside: &'static [f64],
    outside: &'static [f64],
    /// What the dispatcher says one step past the bound.
    refusal: &'static str,
    /// `(minimum, maximum)` the profile must publish; `None` is an open end.
    published: (Option<f64>, Option<f64>),
}

enum Shape {
    /// The number itself.
    Number,
    /// A string of that many characters.
    Length,
    /// The only item of a one-column `widths_dxa`.
    ArrayItem,
}

const BOUNDS: &[Bound] = &[
    Bound {
        kind: "insert_toc",
        path: &["levels"],
        shape: Shape::Number,
        inside: &[1.0, 9.0],
        outside: &[0.0, 10.0],
        refusal: "is outside 1..=9",
        published: (Some(1.0), Some(9.0)),
    },
    Bound {
        kind: "list",
        path: &["level"],
        shape: Shape::Number,
        inside: &[0.0, 8.0],
        outside: &[9.0],
        refusal: "is outside 0..=8",
        published: (Some(0.0), Some(8.0)),
    },
    Bound {
        kind: "page_setup",
        path: &["page", "width_dxa"],
        shape: Shape::Number,
        inside: &[1.0, 31_680.0],
        outside: &[0.0, 31_681.0],
        refusal: "is outside 1..=31680 (22 inches)",
        published: (Some(1.0), Some(31_680.0)),
    },
    Bound {
        kind: "page_setup",
        path: &["page", "height_dxa"],
        shape: Shape::Number,
        inside: &[1.0, 31_680.0],
        outside: &[0.0, 31_681.0],
        refusal: "is outside 1..=31680 (22 inches)",
        published: (Some(1.0), Some(31_680.0)),
    },
    Bound {
        kind: "page_setup",
        path: &["margins_dxa", "top"],
        shape: Shape::Number,
        inside: &[-31_680.0, 31_680.0],
        outside: &[-31_681.0, 31_681.0],
        refusal: "is outside -31680..=31680",
        published: (Some(-31_680.0), Some(31_680.0)),
    },
    Bound {
        kind: "page_setup",
        path: &["margins_dxa", "bottom"],
        shape: Shape::Number,
        inside: &[-31_680.0, 31_680.0],
        outside: &[-31_681.0, 31_681.0],
        refusal: "is outside -31680..=31680",
        published: (Some(-31_680.0), Some(31_680.0)),
    },
    Bound {
        kind: "page_setup",
        path: &["margins_dxa", "left"],
        shape: Shape::Number,
        inside: &[0.0, 31_680.0],
        outside: &[31_681.0],
        refusal: "is outside -31680..=31680",
        published: (Some(0.0), Some(31_680.0)),
    },
    Bound {
        kind: "page_setup",
        path: &["margins_dxa", "right"],
        shape: Shape::Number,
        inside: &[0.0, 31_680.0],
        outside: &[31_681.0],
        refusal: "is outside -31680..=31680",
        published: (Some(0.0), Some(31_680.0)),
    },
    Bound {
        kind: "page_setup",
        path: &["margins_dxa", "header"],
        shape: Shape::Number,
        inside: &[0.0, 31_680.0],
        outside: &[31_681.0],
        refusal: "is outside -31680..=31680",
        published: (Some(0.0), Some(31_680.0)),
    },
    Bound {
        kind: "page_setup",
        path: &["margins_dxa", "footer"],
        shape: Shape::Number,
        inside: &[0.0, 31_680.0],
        outside: &[31_681.0],
        refusal: "is outside -31680..=31680",
        published: (Some(0.0), Some(31_680.0)),
    },
    Bound {
        kind: "insert_image",
        path: &["width_emu"],
        shape: Shape::Number,
        inside: &[1.0, 20_116_800.0],
        outside: &[0.0, 20_116_801.0],
        refusal: "is outside 1..=20116800 (22 inches)",
        published: (Some(1.0), Some(20_116_800.0)),
    },
    Bound {
        kind: "format_paragraph",
        path: &["line_spacing"],
        shape: Shape::Number,
        inside: &[0.25, 10.0],
        outside: &[0.24, 11.0],
        refusal: "is outside 0.25..=10",
        published: (Some(0.25), Some(10.0)),
    },
    Bound {
        kind: "format_paragraph",
        path: &["space_before"],
        shape: Shape::Number,
        inside: &[0.0, 1584.0],
        outside: &[-1.0, 1585.0],
        refusal: "is outside 0..=1584",
        published: (Some(0.0), Some(1584.0)),
    },
    Bound {
        kind: "format_paragraph",
        path: &["space_after"],
        shape: Shape::Number,
        inside: &[0.0, 1584.0],
        outside: &[-1.0, 1585.0],
        refusal: "is outside 0..=1584",
        published: (Some(0.0), Some(1584.0)),
    },
    Bound {
        kind: "replace",
        path: &["format", "size_pt"],
        shape: Shape::Number,
        inside: &[0.0, 1638.0],
        outside: &[-1.0, 1639.0],
        refusal: "is outside 0..=1638",
        published: (Some(0.0), Some(1638.0)),
    },
    Bound {
        kind: "insert",
        path: &["format", "size_pt"],
        shape: Shape::Number,
        inside: &[0.0, 1638.0],
        outside: &[-1.0, 1639.0],
        refusal: "is outside 0..=1638",
        published: (Some(0.0), Some(1638.0)),
    },
    Bound {
        kind: "format_run",
        path: &["format", "size_pt"],
        shape: Shape::Number,
        inside: &[0.0, 1638.0],
        outside: &[-1.0, 1639.0],
        refusal: "is outside 0..=1638",
        published: (Some(0.0), Some(1638.0)),
    },
    Bound {
        kind: "watermark",
        path: &["text"],
        shape: Shape::Length,
        inside: &[1.0, 64.0],
        outside: &[0.0, 65.0],
        refusal: "watermark text must be 1 to 64 characters",
        published: (Some(1.0), Some(64.0)),
    },
    Bound {
        kind: "insert_table",
        path: &["widths_dxa"],
        shape: Shape::ArrayItem,
        inside: &[1.0, 31_680.0],
        outside: &[0.0, 31_681.0],
        refusal: "widths_dxa must lie in 1..=31680",
        published: (Some(1.0), Some(31_680.0)),
    },
    Bound {
        kind: "replace",
        path: &["occurrence"],
        shape: Shape::Number,
        inside: &[1.0],
        outside: &[0.0],
        refusal: "occurrence is 1-based",
        published: (Some(1.0), None),
    },
    Bound {
        kind: "insert",
        path: &["occurrence"],
        shape: Shape::Number,
        inside: &[1.0],
        outside: &[0.0],
        refusal: "occurrence is 1-based",
        published: (Some(1.0), None),
    },
    Bound {
        kind: "delete",
        path: &["occurrence"],
        shape: Shape::Number,
        inside: &[1.0],
        outside: &[0.0],
        refusal: "occurrence is 1-based",
        published: (Some(1.0), None),
    },
    Bound {
        kind: "comment",
        path: &["occurrence"],
        shape: Shape::Number,
        inside: &[1.0],
        outside: &[0.0],
        refusal: "occurrence is 1-based",
        published: (Some(1.0), None),
    },
    Bound {
        kind: "format_run",
        path: &["occurrence"],
        shape: Shape::Number,
        inside: &[1.0],
        outside: &[0.0],
        refusal: "occurrence is 1-based",
        published: (Some(1.0), None),
    },
    Bound {
        kind: "insert_footnote",
        path: &["occurrence"],
        shape: Shape::Number,
        inside: &[1.0],
        outside: &[0.0],
        refusal: "occurrence is 1-based",
        published: (Some(1.0), None),
    },
    Bound {
        kind: "redact",
        path: &["occurrence"],
        shape: Shape::Number,
        inside: &[1.0],
        outside: &[0.0],
        refusal: "occurrence is 1-based",
        published: (Some(1.0), None),
    },
];

fn base_operation(kind: &str) -> Value {
    match kind {
        "insert_toc" => {
            json!({"kind": "insert_toc", "paragraph": "p0", "position": "after", "levels": 3})
        }
        "list" => {
            json!({"kind": "list", "paragraphs": ["p0"], "kind_of_list": "bullet", "level": 0})
        }
        "page_setup" => {
            json!({"kind": "page_setup", "page": {"width_dxa": 12240, "height_dxa": 12240}, "margins_dxa": {}})
        }
        "insert_image" => {
            json!({"kind": "insert_image", "paragraph": "p0", "position": "after", "image_base64": ""})
        }
        "format_paragraph" => {
            json!({"kind": "format_paragraph", "paragraph": "p0", "line_spacing": 1.0, "space_before": 0.0, "space_after": 0.0})
        }
        "format_run" => {
            json!({"kind": "format_run", "paragraph": "p0", "find": "Zero", "format": {"size_pt": 12}, "occurrence": 1})
        }
        "replace" => {
            json!({"kind": "replace", "paragraph": "p0", "find": "Zero", "replacement": "Nine", "occurrence": 1})
        }
        "insert" => {
            json!({"kind": "insert", "paragraph": "p0", "after": "Zero", "text": "Nine", "occurrence": 1})
        }
        "delete" => json!({"kind": "delete", "paragraph": "p0", "find": "Zero", "occurrence": 1}),
        "comment" => {
            json!({"kind": "comment", "paragraph": "p0", "find": "Zero", "text": "note", "occurrence": 1})
        }
        "insert_footnote" => {
            json!({"kind": "insert_footnote", "paragraph": "p0", "after": "Zero", "text": "note", "occurrence": 1})
        }
        "redact" => json!({"kind": "redact", "paragraph": "p0", "find": "Zero", "occurrence": 1}),
        "watermark" => json!({"kind": "watermark", "text": "DRAFT"}),
        "insert_table" => {
            json!({"kind": "insert_table", "paragraph": "p0", "position": "after", "rows": [["cell"]], "widths_dxa": [1000]})
        }
        other => panic!("no base operation for {other}"),
    }
}

fn bound_number(value: f64) -> Value {
    if value.fract() == 0.0 {
        if value >= 0.0 {
            json!(value as u64)
        } else {
            json!(value as i64)
        }
    } else {
        json!(value)
    }
}

fn bound_operation(bound: &Bound, value: f64) -> Value {
    let mut op = base_operation(bound.kind);
    let leaf = match bound.shape {
        Shape::Number => bound_number(value),
        Shape::Length => json!("a".repeat(value.max(0.0) as usize)),
        Shape::ArrayItem => json!([bound_number(value)]),
    };
    let mut at = &mut op;
    for segment in bound.path {
        at = &mut at[*segment];
    }
    *at = leaf;
    op
}

type Published = std::collections::BTreeMap<(String, String), (Option<f64>, Option<f64>, bool)>;

/// Every numeric and text-length bound the v1 profiles publish, keyed by
/// operation and dotted field path; `true` marks a string length bound.
fn published_bounds() -> Published {
    fn number(value: Option<&Value>) -> Option<f64> {
        value.and_then(Value::as_f64)
    }
    fn walk(schema: &Value, path: &mut Vec<String>, kind: &str, out: &mut Published) {
        let bounds = (number(schema.get("minimum")), number(schema.get("maximum")));
        if bounds != (None, None) {
            out.insert(
                (kind.to_string(), path.join(".")),
                (bounds.0, bounds.1, false),
            );
        }
        let length = (
            number(schema.get("minLength")),
            number(schema.get("maxLength")),
        );
        if length != (None, None) {
            out.insert(
                (kind.to_string(), path.join(".")),
                (length.0, length.1, true),
            );
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for (name, child) in properties {
                path.push(name.clone());
                walk(child, path, kind, out);
                path.pop();
            }
        }
        if let Some(items) = schema.get("items") {
            walk(items, path, kind, out);
        }
        for key in ["anyOf", "oneOf"] {
            if let Some(branches) = schema.get(key).and_then(Value::as_array) {
                for branch in branches {
                    walk(branch, path, kind, out);
                }
            }
        }
    }
    let mut out = Published::new();
    let matrix = jubarte::capabilities::discovery("cli", None, None);
    for profile in matrix["profiles"].as_array().unwrap() {
        if profile["plan_version"] == 1 {
            let kind = profile["operation"].as_str().unwrap();
            // `fields` is the operation schema's properties map itself.
            for (name, field) in profile["fields"].as_object().unwrap() {
                let mut path = vec![name.clone()];
                walk(field, &mut path, kind, &mut out);
            }
        }
    }
    out
}

#[test]
fn v1_profiles_publish_the_dispatchers_numeric_bounds() {
    let mut expected = Published::new();
    for bound in BOUNDS {
        expected.insert(
            (bound.kind.to_string(), bound.path.join(".")),
            (
                bound.published.0,
                bound.published.1,
                matches!(bound.shape, Shape::Length),
            ),
        );
    }
    // `index` and `comment_id` carry only their type's floor: 0 is a valid
    // zero-based index and a syntactically valid comment id, so the profiles
    // publish the unsigned minimum with no dispatcher rule to pair it with.
    let mut published = Published::new();
    for (key, bounds) in published_bounds() {
        if (key.1.ends_with(".index") || key.1 == "comment_id")
            && bounds == (Some(0.0), None, false)
        {
            continue;
        }
        published.insert(key, bounds);
    }
    assert_eq!(published, expected);
    // Every published bound is the dispatcher's own: probe at the bound and
    // one step outside, through a real plan on a real document.
    for bound in BOUNDS {
        for value in bound.inside {
            if let Err(e) = dispatch(&bound_operation(bound, *value)) {
                assert!(
                    !e.contains(bound.refusal),
                    "{} at its bound {value} is refused: {e}",
                    bound.path.join(".")
                );
            }
        }
        for value in bound.outside {
            let e = dispatch(&bound_operation(bound, *value)).expect_err(&format!(
                "{} past its bound {value} is accepted",
                bound.path.join(".")
            ));
            assert!(
                e.contains(bound.refusal),
                "{} past its bound {value} refused differently: {e}",
                bound.path.join(".")
            );
        }
    }
}

#[test]
fn v1_profiles_are_closed_and_unknown_keys_are_refused() {
    let matrix = jubarte::capabilities::discovery("cli", None, None);
    for profile in matrix["profiles"].as_array().unwrap() {
        if profile["plan_version"] == 1 {
            assert_eq!(
                profile["constraints"]["additionalProperties"],
                json!(false),
                "{}",
                profile["operation"]
            );
        }
    }
    // What the closed profiles forbid, the dispatcher refuses: a misspelled
    // key on a dispatched kind, and a paragraph key on a kind that takes
    // none (silently ignoring it would change meaning).
    let misspelled = json!({"kind": "insert_toc", "paragraph": "p0", "position": "after", "levels": 3, "levils": 3});
    assert!(
        dispatch(&misspelled)
            .unwrap_err()
            .contains("unknown field \"levils\"")
    );
    let stray =
        json!({"kind": "reply_comment", "comment_id": 0, "text": "note", "paragraph": "p0"});
    assert!(
        dispatch(&stray)
            .unwrap_err()
            .contains("unknown field \"paragraph\""),
        "a paragraph key on reply_comment must not be silently ignored"
    );
}

/// Both dialects refuse `edit_comment`, `delete_comment` and
/// `resolve_comment` on another author's comment, so every supported row
/// of either version names the opt-in (#1264, #1846); a delete also names
/// the thread that holds their reply.
#[test]
fn comment_rows_name_the_ownership_opt_in_in_both_versions() {
    let matrix = jubarte::capabilities::discovery("cli", None, None);
    for operation in ["edit_comment", "delete_comment", "resolve_comment"] {
        let mut versions = std::collections::BTreeSet::new();
        for profile in matrix["profiles"].as_array().unwrap() {
            if profile["operation"] != operation {
                continue;
            }
            versions.insert(profile["plan_version"].as_u64().unwrap());
            for row in profile["support"].as_array().unwrap() {
                if row["status"] == "unsupported" {
                    continue;
                }
                let conditions = row["conditions"].as_str().unwrap();
                assert!(
                    conditions
                        .contains("; another author's comment requires rewrite_others_comments"),
                    "{operation} v{} {row}",
                    profile["plan_version"]
                );
                assert_eq!(
                    conditions.contains(", as does deleting a thread that holds their reply"),
                    operation == "delete_comment",
                    "{operation} v{} {row}",
                    profile["plan_version"]
                );
            }
        }
        assert_eq!(
            versions,
            std::collections::BTreeSet::from([1, 2]),
            "{operation}"
        );
    }
}

#[test]
fn focused_answers_carry_the_discovery_schema_version() {
    for args in [
        vec!["--matrix"],
        vec!["--operation", "reply_comment"],
        vec!["--operation", "reply_coment"],
        vec!["--operation", "reply_comment", "--field", "tex"],
        vec!["--field", "include_replies"],
    ] {
        let answer = query(&args);
        assert_eq!(answer["schema_version"], 1, "{args:?}");
    }
}

#[test]
fn unsupported_names_render_as_prose() {
    assert_eq!(
        text(&["--operation", "reply_coment"]),
        "unsupported operation: reply_coment\nnearest valid names: reply_comment, edit_comment, resolve_comment\n"
    );
    assert_eq!(
        text(&["--operation", "reply_comment", "--field", "tex"]),
        "unsupported field: tex (operation reply_comment)\nnearest valid names: text, id, kind\n"
    );
}

/// Publishing the `widths_dxa` item bound must not change the library type:
/// a caller still reads and writes the widths as plain integers.
#[test]
fn insert_table_widths_stay_plain_integers_in_the_library_api() {
    let mut plan = jubarte::edit::EditPlan::from_json(
        r#"{"schema_version": 1, "author": "Reviewer", "operations": [{"kind": "insert_table",
            "paragraph": "p0", "position": "after", "rows": [["a", "b"]],
            "widths_dxa": [1440, 2880]}]}"#,
    )
    .expect("plan");
    let jubarte::edit::OperationKind::InsertTable { widths_dxa, .. } = &mut plan.operations[0].kind
    else {
        panic!("insert_table")
    };
    assert_eq!(widths_dxa.as_deref(), Some(&[1440_u32, 2880][..]));
    *widths_dxa = Some(vec![720]);
    let written: Value = serde_json::from_str(&plan.to_json()).expect("plan json");
    assert_eq!(written["operations"][0]["widths_dxa"], json!([720]));
}
