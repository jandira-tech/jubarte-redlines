// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Focused syntax discovery and executable edit support. V2 parsing does
//! not imply dispatch: a runtime without a PlanV2 adapter keeps all its V2
//! rows refused.
use std::collections::BTreeSet;
use std::sync::LazyLock;

use serde_json::{Value, json};

static V2: LazyLock<Value> = LazyLock::new(|| {
    serde_json::from_str(crate::edit::v2::SCHEMA_JSON).expect("production V2 schema")
});

fn expand(schema: &Value, root: &Value) -> Value {
    match schema {
        Value::Object(map) => {
            if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
                return expand(
                    root.pointer(reference.strip_prefix('#').expect("local ref"))
                        .expect("schema definition"),
                    root,
                );
            }
            Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), expand(v, root)))
                    .collect(),
            )
        }
        Value::Array(values) => Value::Array(values.iter().map(|v| expand(v, root)).collect()),
        other => other.clone(),
    }
}

fn profiles(runtime: &str) -> Vec<Value> {
    let mut profiles = Vec::new();
    for (version, schemas) in [
        (
            crate::inspect::SCHEMA_VERSION,
            crate::edit::operation_schemas().to_vec(),
        ),
        (
            2,
            V2["properties"]["operations"]["items"]["oneOf"]
                .as_array()
                .expect("V2 kinds")
                .iter()
                .map(|s| expand(s, &V2))
                .collect(),
        ),
    ] {
        for schema in schemas {
            let kind = schema["properties"]["kind"]["const"]
                .as_str()
                .expect("kind");
            let fields = schema["properties"].clone();
            let mut constraints = schema.as_object().expect("operation object").clone();
            for key in [
                "properties",
                "required",
                "description",
                "type",
                "x-jubarte-targets",
                "x-jubarte-keep",
            ] {
                constraints.remove(key);
            }
            let dispatched =
                version == crate::inspect::SCHEMA_VERSION || super::dispatches_v2(runtime);
            profiles.push(json!({
                "operation": kind, "plan_version": version, "fields": fields,
                "required": schema["required"], "constraints": constraints,
                "description": schema["description"],
                "dispatched": dispatched,
                "support": support(runtime, version, kind, &schema),
            }));
        }
    }
    profiles
}

fn targets(version: u32, schema: &Value) -> Vec<String> {
    if version == 2 {
        return vec!["source_target".into()];
    }
    if let Some(targets) = schema["x-jubarte-targets"].as_array() {
        return targets
            .iter()
            .map(|t| t.as_str().expect("target").to_string())
            .collect();
    }
    let fields = &schema["properties"];
    let names: &[&str] = if fields.get("paragraph").is_some() {
        &[
            "body",
            "table_cell",
            "header",
            "footer",
            "footnotes",
            "endnotes",
        ]
    } else if fields.get("comment_id").is_some() {
        &["comment"]
    } else if fields.get("control").is_some() {
        &["body_control"]
    } else if fields.get("section").is_some() {
        &["section"]
    } else {
        &["document"]
    };
    names.iter().map(|t| (*t).to_string()).collect()
}

fn support(runtime: &str, version: u32, kind: &str, schema: &Value) -> Vec<Value> {
    let mut rows = Vec::new();
    for target in targets(version, schema) {
        for track in ["on", "off"] {
            for existing in ["refuse", "accept", "reject", "keep"] {
                let refusal = if version == 2 && super::dispatches_v2(runtime) {
                    // V2 dispatches every combination through the preserving engine.
                    // Source, anchor and revision checks still refuse per apply_plan's
                    // contract; dispatch does not guarantee preparation succeeds.
                    None
                } else if version == 2 {
                    Some("PlanV2 is parsed by the library but is not dispatched by this runtime")
                } else if track == "off" {
                    Some(
                        "v1 has no track policy; selecting its clean derivative is a separate output effect",
                    )
                } else if existing == "keep" && !crate::edit::operation_supports_keep(kind) {
                    Some("the legacy emitter cannot preserve existing revisions for this operation")
                } else if existing != "keep" && crate::edit::operation_needs_keep(kind) {
                    Some(
                        "the revision it answers stays beside our change, so it needs existing_revisions keep",
                    )
                } else {
                    None
                };
                let mut conditions = if let Some(reason) = refusal {
                    reason.to_string()
                } else if version == 2 {
                    let mut conditions = String::from(
                        "requires the bound source digest, target pairs naming one entity, nonconflicting edits and supported source structure; composed edits may not remove a comment's whole anchor",
                    );
                    if existing == "refuse" {
                        conditions.push_str(
                            "; no remaining revision anywhere after selected decisions, including unknown revision forms",
                        );
                    }
                    if existing == "keep" {
                        conditions.push_str("; remaining revisions stay beside the new edits");
                    }
                    conditions
                } else {
                    let mut conditions = String::from(
                        "requires valid selectors, nonconflicting edits and supported source structure; opaque edit ranges refuse",
                    );
                    if existing == "refuse" {
                        conditions
                            .push_str("; no remaining revisions anywhere after selected decisions");
                    }
                    if existing == "keep" {
                        conditions.push_str("; update_fields must be false; revisions on the edited ranges, paragraph marks or formatting may refuse");
                    }
                    if schema["properties"].get("comment").is_some() {
                        conditions.push_str("; attached comments require a body anchor");
                    }
                    if matches!(kind, "watermark" | "settings" | "redact") {
                        conditions.push_str("; this effect is untracked in both outputs");
                    }
                    conditions
                };
                // Both dialects refuse another author's comment (#1264,
                // #1846), and a delete that would take their reply.
                if refusal.is_none()
                    && matches!(kind, "edit_comment" | "delete_comment" | "resolve_comment")
                {
                    conditions
                        .push_str("; another author's comment requires rewrite_others_comments");
                    if kind == "delete_comment" {
                        conditions.push_str(", as does deleting a thread that holds their reply");
                    }
                }
                rows.push(json!({"runtime": runtime, "target": target, "track": track,
                    "existing_revisions": existing,
                    "status": if refusal.is_some() { "unsupported" } else { "conditional" },
                    "conditions": conditions}));
            }
        }
    }
    rows
}

fn nearest(name: &str, candidates: BTreeSet<String>) -> Vec<String> {
    let mut ranked: Vec<_> = candidates
        .into_iter()
        .map(|candidate| (distance(name, &candidate), candidate))
        .collect();
    ranked.sort();
    ranked.into_iter().take(3).map(|(_, name)| name).collect()
}

fn distance(a: &str, b: &str) -> usize {
    let mut row: Vec<_> = (0..=b.chars().count()).collect();
    for (i, a) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, b) in b.chars().enumerate() {
            let previous = row[j + 1];
            row[j + 1] = (diagonal + usize::from(a != b))
                .min(row[j] + 1)
                .min(previous + 1);
            diagonal = previous;
        }
    }
    *row.last().expect("distance row")
}

/// Discover an operation, a field (optionally within an operation), or the
/// full edit matrix when both are absent. `supported` means known syntax;
/// `dispatched` and the support rows describe execution. `schema_version`
/// is `1`, as the manifest's is, so an agent can date a build from any
/// focused answer. No full plan schema is embedded. All three hosts share
/// the same v1 edit dispatcher; PlanV2 dispatch differs per host (the CLI
/// routes it through `edit::prepare_v2`, the bindings have no entry point
/// for it yet).
#[must_use]
pub fn discovery(runtime: &str, operation: Option<&str>, field: Option<&str>) -> Value {
    let all = profiles(runtime);
    let kinds = all
        .iter()
        .map(|p| p["operation"].as_str().expect("kind").to_string())
        .collect();
    let selected: Vec<_> = all
        .into_iter()
        .filter(|p| operation.is_none_or(|op| p["operation"] == op))
        .collect();
    if selected.is_empty() {
        return json!({"runtime": runtime, "schema_version": 1, "supported": false,
            "operation": operation,
            "reason": "unsupported operation",
            "nearest": nearest(operation.unwrap_or_default(), kinds)});
    }
    // `why` is an annotation `--skip-failed` writes and the parser ignores:
    // never the field a misspelled name meant.
    let fields = selected
        .iter()
        .flat_map(|p| p["fields"].as_object().expect("fields").keys().cloned())
        .filter(|f| f != "why")
        .collect();
    let mut selected: Vec<_> = selected
        .into_iter()
        .filter(|p| field.is_none_or(|f| p["fields"].get(f).is_some()))
        .collect();
    if selected.is_empty() {
        return json!({"runtime": runtime, "schema_version": 1, "supported": false,
            "operation": operation, "field": field,
            "reason": "unsupported field", "nearest": nearest(field.unwrap_or_default(), fields)});
    }
    if let Some(field) = field {
        for profile in &mut selected {
            profile["fields"]
                .as_object_mut()
                .expect("fields")
                .retain(|name, _| name == field);
        }
    }
    let mut answer =
        json!({"runtime": runtime, "schema_version": 1, "supported": true, "profiles": selected});
    if operation.is_none() && field.is_none() {
        answer["edit_operations"] = json!(crate::edit::dispatched_operation_kinds());
        answer["edit_plan_versions"] = json!(super::dispatched_plan_versions(runtime));
    }
    answer
}

/// Text presentation for focused queries; the default manifest renderer stays
/// unchanged. Unsupported names are answers, with suggestions, not usage
/// errors: the operation and field names print as prose, and the field
/// clause is left out when no field was asked.
#[must_use]
pub fn discovery_text(answer: &Value) -> String {
    if answer["supported"] == false {
        let reason = answer["reason"].as_str().unwrap_or("unsupported");
        let operation = answer["operation"].as_str().filter(|name| !name.is_empty());
        let named = match (answer["field"].as_str(), operation) {
            (Some(field), Some(operation)) => format!("{reason}: {field} (operation {operation})"),
            (Some(field), None) => format!("{reason}: {field}"),
            (None, operation) => format!("{reason}: {}", operation.unwrap_or_default()),
        };
        let nearest: Vec<&str> = answer["nearest"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect();
        return format!("{named}\nnearest valid names: {}\n", nearest.join(", "));
    }
    let mut text = format!(
        "runtime: {}\nsupported syntax; execution depends on the rows below\n",
        answer["runtime"]
    );
    for profile in answer["profiles"].as_array().expect("profiles") {
        text.push_str(&format!(
            "{} (plan v{}; dispatched: {})\n",
            profile["operation"], profile["plan_version"], profile["dispatched"]
        ));
        text.push_str(&format!("  required: {}\n", profile["required"]));
        for (name, facts) in profile["fields"].as_object().expect("fields") {
            text.push_str(&format!("  {name}: {facts}\n"));
        }
        text.push_str(&format!("  constraints: {}\n", profile["constraints"]));
        for row in profile["support"].as_array().expect("support rows") {
            text.push_str(&format!(
                "  target={} track={} existing_revisions={}: {} ({})\n",
                row["target"],
                row["track"],
                row["existing_revisions"],
                row["status"],
                row["conditions"]
            ));
        }
    }
    text
}

#[cfg(test)]
#[cfg_attr(all(coverage_nightly, test), coverage(off))]
mod tests {
    use super::*;

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
            _ => {
                let mut value = json!({});
                for field in schema["required"].as_array().into_iter().flatten() {
                    let field = field.as_str().unwrap();
                    value[field] = minimal(&schema["properties"][field]);
                }
                value
            }
        }
    }

    fn assert_runtime_dispatch(runtime: &str) {
        // Each wrapper calls the same apply_plan_json/apply_plan dispatcher.
        // Probe the actual typed variants and its exhaustive kind match, rather
        // than comparing two copies of a string catalog.
        let actual: BTreeSet<_> = crate::edit::operation_schemas()
            .iter()
            .map(|schema| {
                let wire = minimal(schema);
                let op: crate::edit::OperationKind = serde_json::from_value(wire).unwrap();
                crate::edit::kind_name(&op).to_string()
            })
            .collect();
        let advertised: BTreeSet<_> = crate::capabilities::capabilities(runtime)
            .edit_operations
            .into_iter()
            .collect();
        assert_eq!(advertised, actual);
        let matrix = discovery(runtime, None, None);
        let dispatched: BTreeSet<_> = matrix["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["plan_version"] == 1 && p["dispatched"] == true)
            .map(|p| p["operation"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(dispatched, actual);
        // V2 dispatches as a whole runtime property, not per kind: every
        // v2 row is dispatched, or none is.
        let v2 = matrix["profiles"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["plan_version"] == 2);
        for profile in v2 {
            assert_eq!(
                profile["dispatched"],
                json!(super::super::dispatches_v2(runtime))
            );
        }
    }

    #[test]
    fn native_advertised_equals_dispatched() {
        assert_runtime_dispatch("cli");
    }
    #[test]
    fn python_advertised_equals_dispatched() {
        assert_runtime_dispatch("python");
    }
    #[test]
    fn wasm_advertised_equals_dispatched() {
        assert_runtime_dispatch("wasm");
    }

    #[test]
    fn policy_rows_keep_untrackable_operations_and_direct_v1_explicit() {
        for kind in [
            "fill_control",
            "insert_footnote",
            "insert_image",
            "page_setup",
        ] {
            let answer = discovery("cli", Some(kind), None);
            let rows = answer["profiles"][0]["support"].as_array().unwrap();
            assert!(
                rows.iter()
                    .filter(|r| r["existing_revisions"] == "keep")
                    .all(|r| r["status"] == "unsupported")
            );
            assert!(
                rows.iter()
                    .filter(|r| r["track"] == "off")
                    .all(|r| r["status"] == "unsupported")
            );
            assert!(rows.iter().any(|r| r["track"] == "on"
                && r["existing_revisions"] == "accept"
                && r["status"] == "conditional"));
        }
    }

    #[test]
    fn strike_and_restore_are_offered_under_keep_only() {
        for kind in ["strike", "restore"] {
            let answer = discovery("cli", Some(kind), None);
            let rows: Vec<&Value> = answer["profiles"][0]["support"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|r| r["track"] == "on")
                .collect();
            assert!(!rows.is_empty(), "{kind}: {answer}");
            for row in rows {
                let keep = row["existing_revisions"] == "keep";
                let want = if keep { "conditional" } else { "unsupported" };
                assert_eq!(row["status"], want, "{kind}: {row}");
                if !keep {
                    assert!(
                        row["conditions"]
                            .as_str()
                            .unwrap()
                            .contains("existing_revisions keep"),
                        "{kind}: {row}"
                    );
                }
            }
        }
    }

    #[test]
    fn edit_distance_handles_unicode_and_empty_names() {
        assert_eq!(distance("", "text"), 4);
        assert_eq!(distance("é", "e"), 1);
        assert_eq!(distance("reply_coment", "reply_comment"), 1);
    }

    #[test]
    fn every_discovery_answer_carries_a_schema_version() {
        for answer in [
            discovery("cli", None, None),
            discovery("cli", Some("reply_comment"), None),
            discovery("cli", Some("reply_comment"), Some("text")),
            discovery("cli", Some("reply_coment"), None),
            discovery("cli", Some("reply_comment"), Some("tex")),
        ] {
            assert_eq!(answer["schema_version"], 1, "{answer}");
        }
    }

    #[test]
    fn unsupported_names_render_as_prose() {
        // No `field null`, no JSON quoting, no JSON array in prose: the
        // clause names what was asked and nothing else.
        assert_eq!(
            discovery_text(&discovery("cli", Some("reply_coment"), None)),
            "unsupported operation: reply_coment\nnearest valid names: reply_comment, edit_comment, resolve_comment\n"
        );
        assert_eq!(
            discovery_text(&discovery("cli", Some("reply_comment"), Some("tex"))),
            "unsupported field: tex (operation reply_comment)\nnearest valid names: text, id, kind\n"
        );
        let text = discovery_text(&discovery("cli", None, Some("imaginary")));
        assert!(
            text.starts_with("unsupported field: imaginary\nnearest valid names: ")
                && !text.contains('"'),
            "{text}"
        );
    }
}
