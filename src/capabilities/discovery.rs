// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Focused syntax discovery and executable edit support. V2 parsing does not
//! imply dispatch: until a runtime has an adapter, all its V2 rows refuse.
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
            profiles.push(json!({
                "operation": kind, "plan_version": version, "fields": fields,
                "required": schema["required"], "constraints": constraints,
                "description": schema["description"],
                "dispatched": version == crate::inspect::SCHEMA_VERSION,
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
                let refusal = if version != crate::inspect::SCHEMA_VERSION {
                    Some("PlanV2 is parsed by the library but is not dispatched by this runtime")
                } else if track == "off" {
                    Some(
                        "v1 has no track policy; selecting its clean derivative is a separate output effect",
                    )
                } else if existing == "keep" && !crate::edit::operation_supports_keep(kind) {
                    Some("the legacy emitter cannot preserve existing revisions for this operation")
                } else {
                    None
                };
                let conditions = if let Some(reason) = refusal {
                    reason.to_string()
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
/// `dispatched` and the support rows describe execution. No full plan schema
/// is embedded. All three hosts share the same v1 edit dispatcher today.
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
        return json!({"runtime": runtime, "supported": false, "operation": operation,
            "reason": "unsupported operation", "nearest": nearest(operation.unwrap_or_default(), kinds)});
    }
    let fields = selected
        .iter()
        .flat_map(|p| p["fields"].as_object().expect("fields").keys().cloned())
        .collect();
    let mut selected: Vec<_> = selected
        .into_iter()
        .filter(|p| field.is_none_or(|f| p["fields"].get(f).is_some()))
        .collect();
    if selected.is_empty() {
        return json!({"runtime": runtime, "supported": false, "operation": operation, "field": field,
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
    let mut answer = json!({"runtime": runtime, "supported": true, "profiles": selected});
    if operation.is_none() && field.is_none() {
        answer["edit_operations"] = json!(crate::edit::dispatched_operation_kinds());
        answer["edit_plan_versions"] = json!([crate::inspect::SCHEMA_VERSION]);
    }
    answer
}

/// Text presentation for focused queries; the default manifest renderer stays
/// unchanged. Unsupported names are answers, with suggestions, not usage errors.
#[must_use]
pub fn discovery_text(answer: &Value) -> String {
    if answer["supported"] == false {
        return format!(
            "{}: operation {}, field {}\nnearest valid names: {}\n",
            answer["reason"].as_str().unwrap_or("unsupported"),
            answer["operation"],
            answer["field"],
            answer["nearest"]
        );
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
            .filter(|p| p["dispatched"] == true)
            .map(|p| p["operation"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(dispatched, actual);
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
    fn edit_distance_handles_unicode_and_empty_names() {
        assert_eq!(distance("", "text"), 4);
        assert_eq!(distance("é", "e"), 1);
        assert_eq!(distance("reply_coment", "reply_comment"), 1);
    }
}
