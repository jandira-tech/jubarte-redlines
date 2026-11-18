// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Plan parsing and validation: field checks, operation shapes and
//! the near-miss field names a refusal suggests.

use super::*;

impl EditPlan {
    /// Parse and validate a plan's JSON. A field read as another (`anchor`
    /// as `find`, …) is read without its note: [`Self::from_json_with_notes`]
    /// returns the notes, and [`apply_plan_json`] puts them in
    /// `EditReport.notes`.
    ///
    /// # Errors
    ///
    /// As [`Self::from_json_with_notes`].
    pub fn from_json(json: &str) -> Result<Self, EditError> {
        Self::from_json_with_notes(json).map(|(plan, _)| plan)
    }

    /// Parse and validate a plan's JSON, with a note for each field read
    /// as another: `anchor` as `find` on `replace`, `delete` and `comment`,
    /// and `comment` as `text` on `comment` and `reply_comment`.
    ///
    /// # Errors
    ///
    /// `INVALID_PLAN` when the JSON is malformed or more than one value,
    /// lacks `schema_version`, or names an unknown field or operation kind:
    /// the message names the operation, its kind, the fields the kind takes
    /// and the nearest one. `UNSUPPORTED_SCHEMA` when `schema_version`
    /// differs from [`SCHEMA_VERSION`].
    pub fn from_json_with_notes(json: &str) -> Result<(Self, Vec<String>), EditError> {
        let json = without_byte_order_mark(json);
        let mut value = plan_value(json)?;
        let Some(version) = value.get("schema_version") else {
            return Err(err(
                "INVALID_PLAN",
                None,
                format!("schema_version is required ({SCHEMA_VERSION} for this build)"),
            ));
        };
        if let Some(version) = version.as_u64().filter(|v| *v != u64::from(SCHEMA_VERSION)) {
            return Err(err(
                "UNSUPPORTED_SCHEMA",
                None,
                format!("schema_version {version} is not supported (expected {SCHEMA_VERSION})"),
            ));
        }
        check_plan_keys(&value)?;
        check_change_filters(&value)?;
        let notes = check_operations(&mut value)?;
        // Each operation is read on its own, so its error names it.
        let operations = match value.get_mut("operations") {
            Some(serde_json::Value::Array(operations)) => std::mem::take(operations),
            _ => Vec::new(),
        };
        let mut plan: Self =
            serde_json::from_value(value).map_err(|e| err("INVALID_PLAN", None, e.to_string()))?;
        plan.operations = operations
            .into_iter()
            .enumerate()
            .map(|(i, op)| {
                let label = operation_label(i, &op);
                let kind = op["kind"].as_str().unwrap_or_default().to_string();
                serde_json::from_value(op)
                    .map_err(|e| err("INVALID_PLAN", Some(label.as_str()), format!("{kind}: {e}")))
            })
            .collect::<Result<_, _>>()?;
        Ok((plan, notes))
    }

    /// Serialize the plan.
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("plan serializes")
    }
}

pub(super) fn at_least_one(fields: &[&str]) -> serde_json::Value {
    serde_json::json!(
        fields
            .iter()
            .map(|field| serde_json::json!({"required": [field]}))
            .collect::<Vec<_>>()
    )
}

pub(super) fn exactly_one(fields: &[&str]) -> serde_json::Value {
    // Each alternative requires one field and excludes every other field.
    serde_json::json!(
        fields
            .iter()
            .map(|field| serde_json::json!({
                "required": [field],
                "not": {"anyOf": fields.iter().filter(|other| *other != field)
                    .map(|other| serde_json::json!({"required": [other]})).collect::<Vec<_>>()}
            }))
            .collect::<Vec<_>>()
    )
}

/// The top-level fields of a plan, as [`EditPlan`] reads them.
pub(super) const PLAN_FIELDS: &[&str] = &[
    "author",
    "date",
    "existing_revisions",
    "initials",
    "operations",
    "others",
    "removed_anchor_comments",
    "resolve_revisions",
    "rewrite_others_comments",
    "schema_version",
    "source_sha256",
    "update_fields",
];

/// A field one kind's operations take under another name, read as meant
/// with a note: (written, read as, kinds).
const ALIASES: &[(&str, &str, &[&str])] = &[
    ("comment", "text", &["comment", "reply_comment"]),
    ("anchor", "find", &["replace", "delete", "comment"]),
];

/// `json` without a leading byte order mark, which RFC 8259 (§8.1) lets a
/// parser ignore: editors that save UTF-8 with one write it first.
pub(super) fn without_byte_order_mark(json: &str) -> &str {
    json.strip_prefix('\u{feff}').unwrap_or(json)
}

/// The plan's JSON: one value, an object, with nothing after it.
fn plan_value(json: &str) -> Result<serde_json::Value, EditError> {
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| {
        // One whole value parses: what the parser refused follows it.
        let whole = serde_json::Deserializer::from_str(json)
            .into_iter::<serde::de::IgnoredAny>()
            .next()
            .is_some_and(|first| first.is_ok());
        let message = if whole {
            format!(
                "the plan must be exactly one JSON object; characters follow it at line {} column {}",
                e.line(),
                e.column()
            )
        } else {
            e.to_string()
        };
        err("INVALID_PLAN", None, message)
    })?;
    if !value.is_object() {
        return Err(err(
            "INVALID_PLAN",
            None,
            format!(
                "the plan must be exactly one JSON object, not {}",
                json_kind(&value)
            ),
        ));
    }
    Ok(value)
}

/// What a JSON value is, for a message.
fn json_kind(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "a boolean",
        serde_json::Value::Number(_) => "a number",
        serde_json::Value::String(_) => "a string",
        serde_json::Value::Array(_) => "an array",
        serde_json::Value::Object(_) => "an object",
    }
}

/// The name an error gives operation `i`: its `id`, else `op-N` as the
/// report names it.
fn operation_label(i: usize, op: &serde_json::Value) -> String {
    op.get("id")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| format!("op-{}", i + 1), str::to_string)
}

/// The plan's own fields: an unknown one names the fields and the nearest.
fn check_plan_keys(plan: &serde_json::Value) -> Result<(), EditError> {
    let map = plan.as_object().expect("plan_value checked an object");
    match map.keys().find(|key| !PLAN_FIELDS.contains(&key.as_str())) {
        Some(key) => Err(err(
            "INVALID_PLAN",
            None,
            unknown_field(key, PLAN_FIELDS, None),
        )),
        None => Ok(()),
    }
}

/// `unknown field "key"; did you mean "nearest"? fields: …`.
pub(super) fn unknown_field(key: &str, fields: &[&str], meant: Option<&str>) -> String {
    let hint = nearest(key, fields, meant)
        .map(|name| format!("did you mean {name:?}? "))
        .unwrap_or_default();
    format!("unknown field {key:?}; {hint}fields: {}", fields.join(", "))
}

/// `resolve_revisions.accept.ids` (and `authors`, `kinds`) are arrays; a
/// single string is the slip worth naming.
fn check_change_filters(plan: &serde_json::Value) -> Result<(), EditError> {
    for side in ["accept", "reject"] {
        for key in ["ids", "authors", "kinds"] {
            if let Some(one) = plan
                .pointer(&format!("/resolve_revisions/{side}/{key}"))
                .and_then(serde_json::Value::as_str)
            {
                return Err(err(
                    "INVALID_PLAN",
                    None,
                    format!(
                        "resolve_revisions.{side}.{key} is an array: write {}",
                        serde_json::json!([one])
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Each operation's fields against its kind's schema (the table
/// `capabilities` lists): an error names the operation, its kind, the
/// fields the kind takes and the nearest one. Serde's `flatten` cannot
/// deny unknown fields itself, and a misspelled key must not silently
/// change meaning. The [`ALIASES`] are read as meant, each with a note,
/// and a failed plan's `why` is dropped: a note for whoever fixes it.
fn check_operations(plan: &mut serde_json::Value) -> Result<Vec<String>, EditError> {
    const COMMON: &[&str] = &["id", "kind", "group", "why"];
    let mut notes = Vec::new();
    let Some(ops) = plan
        .get_mut("operations")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return Ok(notes);
    };
    let kinds = dispatched_operation_kinds();
    let kind_names: Vec<&str> = kinds.iter().map(String::as_str).collect();
    for (i, op) in ops.iter_mut().enumerate() {
        let label = operation_label(i, op);
        let invalid = |message: String| err("INVALID_PLAN", Some(label.as_str()), message);
        let what = json_kind(op);
        let Some(map) = op.as_object_mut() else {
            return Err(invalid(format!(
                "an operation is a JSON object, not {what}"
            )));
        };
        match map.get("id") {
            Some(serde_json::Value::Number(n)) => {
                return Err(invalid(format!(
                    "id is a string, not a number: write \"{n}\""
                )));
            }
            Some(id) if !id.is_string() => {
                return Err(invalid(format!("id is a string, not {}", json_kind(id))));
            }
            _ => {}
        }
        let kind = match map.get("kind") {
            Some(serde_json::Value::String(kind)) => kind.clone(),
            None => {
                return Err(invalid(format!(
                    "kind is required; kinds: {}",
                    kinds.join(", ")
                )));
            }
            Some(other) => {
                return Err(invalid(format!(
                    "kind is a string, not {}; kinds: {}",
                    json_kind(other),
                    kinds.join(", ")
                )));
            }
        };
        let Some(schema) = operation_schemas()
            .iter()
            .find(|schema| schema["properties"]["kind"]["const"] == kind.as_str())
        else {
            let hint = nearest(&kind, &kind_names, None)
                .map(|name| format!("did you mean {name:?}? "))
                .unwrap_or_default();
            return Err(invalid(format!(
                "unknown kind {kind:?}; {hint}kinds: {}",
                kinds.join(", ")
            )));
        };
        for (written, read, takes) in ALIASES {
            if takes.contains(&kind.as_str())
                && !map.contains_key(*read)
                && let Some(value) = map.remove(*written)
            {
                map.insert((*read).to_string(), value);
                notes.push(format!(
                    "{label} ({kind}): field {written:?} is read as {read:?}"
                ));
            }
        }
        let refusal = match kind.as_str() {
            "list" => Some("takes paragraphs, not paragraph"),
            "watermark" => Some("a watermark takes no paragraph"),
            "settings" => Some("settings take no paragraph"),
            "fill_control" => Some("selects a control, not a paragraph"),
            "page_setup" => Some("sections take no paragraph"),
            _ => None,
        };
        if let Some(message) = refusal.filter(|_| map.contains_key("paragraph")) {
            return Err(invalid(format!("{kind}: {message}")));
        }
        let allowed = schema["properties"]
            .as_object()
            .expect("operation properties");
        let fields: Vec<&str> = allowed
            .keys()
            .map(String::as_str)
            .filter(|field| !COMMON.contains(field))
            .collect();
        if let Some(key) = map
            .keys()
            .find(|key| !COMMON.contains(&key.as_str()) && !allowed.contains_key(*key))
        {
            if let Some((_, read, _)) = ALIASES.iter().find(|(written, read, takes)| {
                written == key && takes.contains(&kind.as_str()) && map.contains_key(*read)
            }) {
                return Err(invalid(format!(
                    "{kind}: {key:?} is read as {read:?}, which the operation gives too: drop one"
                )));
            }
            // A name the operation already gives is no suggestion.
            let absent: Vec<&str> = fields
                .iter()
                .copied()
                .filter(|field| !map.contains_key(*field))
                .collect();
            let hint = nearest(key, &absent, meant_field(&kind, key))
                .map(|name| format!("did you mean {name:?}? "))
                .unwrap_or_default();
            return Err(invalid(format!(
                "{kind}: unknown field {key:?}; {hint}fields: {}",
                fields.join(", ")
            )));
        }
        for key in ["paragraph", "like", "through"] {
            if let Some(n) = map.get(key).filter(|v| v.is_number()) {
                return Err(invalid(format!(
                    "{kind}: {key} is a selector, not a number: write {}",
                    index_selector(n)
                )));
            }
        }
        if let Some(n) = map
            .get("paragraphs")
            .and_then(serde_json::Value::as_array)
            .and_then(|list| list.iter().find(|v| v.is_number()))
        {
            return Err(invalid(format!(
                "{kind}: paragraphs holds selectors, not numbers: write {}",
                index_selector(n)
            )));
        }
        map.remove("why");
    }
    Ok(notes)
}

/// The selector a bare paragraph number meant: `{"index": n}` for a whole
/// number from 0, else what an index is.
fn index_selector(n: &serde_json::Value) -> String {
    n.as_u64().map_or_else(
        || r#"{"index": N}, N a whole number from 0"#.to_string(),
        |n| format!(r#"{{"index": {n}}}"#),
    )
}

/// The field a plan gives another one's name: `text` for a replacement,
/// `comment` for a comment's text, `anchor` for `find`.
/// `text` on `replace` is no alias, only a hint: `text` is what `insert`
/// and `rewrite` take, so the slip is named rather than read.
fn meant_field(kind: &str, key: &str) -> Option<&'static str> {
    match (kind, key) {
        ("replace", "text") => Some("replacement"),
        (_, "comment") => Some("text"),
        (_, "anchor") => Some("find"),
        _ => None,
    }
}

/// The name a slip most likely meant: `meant` when it is one of `names`,
/// else the closest by edit distance within a third of the slip's length.
pub(super) fn nearest<'a>(key: &str, names: &[&'a str], meant: Option<&str>) -> Option<&'a str> {
    if let Some(name) = meant.and_then(|m| names.iter().copied().find(|name| *name == m)) {
        return Some(name);
    }
    let limit = (key.chars().count() / 3).max(1);
    names
        .iter()
        .map(|name| (edit_distance(key, name), *name))
        .filter(|(distance, _)| *distance <= limit)
        .min_by_key(|(distance, _)| *distance)
        .map(|(_, name)| name)
}

/// Levenshtein distance between two names, by characters.
pub(super) fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (above + 1)
                .min(row[j] + 1)
                .min(diagonal + usize::from(ca != *cb));
            diagonal = above;
        }
    }
    row[b.len()]
}
