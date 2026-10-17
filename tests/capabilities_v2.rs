// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Integration checks for focused discovery through the shipped CLI.
use serde_json::Value;
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
    assert!(
        v2["support"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["status"] == "unsupported")
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
        let advertised: std::collections::BTreeSet<_> = matrix["edit_operations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let dispatched: std::collections::BTreeSet<_> =
            jubarte::capabilities::capabilities(runtime)
                .edit_operations
                .into_iter()
                .collect();
        assert_eq!(advertised, dispatched);
        assert_eq!(advertised.len(), 24);
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
