// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! Unit tests for the public agent wire contracts; no document I/O needed.

use jubarte::capabilities::{capabilities, capabilities_json};
use jubarte::edit::EditPlan;
use jubarte::xmllinq::parse::validate_xml;
use serde_json::json;

#[test]
fn every_advertised_edit_kind_accepts_its_wire_representation() {
    let operations = [
        json!({"kind":"replace", "paragraph":{"index":0}, "find":"a", "replacement":"b", "whole":true}),
        json!({"kind":"insert", "paragraph":{"index":0}, "before":"a", "text":"b"}),
        json!({"kind":"delete", "paragraph":{"index":0}, "find":"a"}),
        json!({"kind":"comment", "paragraph":{"index":0}, "text":"note"}),
        json!({"kind":"insert_paragraph", "paragraph":{"index":0}, "runs":[{"text":"b"}], "like":{"index":1}}),
        json!({"kind":"delete_paragraph", "paragraph":{"index":0}}),
        json!({"kind":"format_paragraph", "paragraph":{"index":0}, "alignment":"center", "line_spacing":1.15, "space_after":6}),
        json!({"kind":"merge_paragraphs", "paragraph":{"index":0}, "separator":" "}),
        json!({"kind":"rewrite", "paragraph":{"index":0}, "text":"new text"}),
        json!({"kind":"reply_comment", "comment_id":1, "text":"agreed"}),
        json!({"kind":"resolve_comment", "comment_id":1, "done":false}),
        json!({"kind":"edit_comment", "comment_id":1, "text":"reworded"}),
        json!({"kind":"delete_comment", "comment_id":1}),
        json!({"kind":"insert_table", "paragraph":{"index":0}, "position":"before", "rows":[["a","b"],["c","d"]], "header_row":true, "widths_dxa":[4680,4680], "style":"TableGrid"}),
        json!({"kind":"list", "paragraphs":[{"index":0}, "body:p:1"], "kind_of_list":"lower_letter", "level":1, "restart":false}),
    ];
    let manifest = capabilities("rust");
    let kinds: Vec<_> = operations
        .iter()
        .map(|op| op["kind"].as_str().unwrap())
        .collect();
    assert_eq!(manifest.edit_operations, kinds);
    for operation in operations {
        let wire = json!({"schema_version":1, "author":"Reviewer", "operations":[operation]});
        let plan = EditPlan::from_json(&wire.to_string()).unwrap();
        assert_eq!(EditPlan::from_json(&plan.to_json()).unwrap(), plan);
        // A misspelled field must not silently change the operation meaning.
        let mut bad = wire;
        bad["operations"][0]["replacment"] = json!("typo");
        let error = EditPlan::from_json(&bad.to_string()).unwrap_err();
        assert_eq!(error.code, "INVALID_PLAN");
        assert!(error.message.contains("replacment"));
    }
}

#[test]
fn malformed_plan_shapes_and_run_fields_are_rejected() {
    for operations in [
        json!(null),
        json!({}),
        json!([null]),
        json!(["delete"]),
        json!([{"kind":"delete", "paragraph":{"index":-1}, "find":"a"}]),
        json!([{"kind":"delete", "paragraph":{"index":true}, "find":"a"}]),
        json!([{"kind":"insert_paragraph", "paragraph":{"index":0}, "runs":[{"text":"a", "bold":"true"}]}]),
        json!([{"kind":"insert_paragraph", "paragraph":{"index":0}, "runs":[{"text":"a", "font_size":12}]}]),
    ] {
        let wire = json!({"schema_version":1, "author":"Reviewer", "operations":operations});
        assert_eq!(
            EditPlan::from_json(&wire.to_string()).unwrap_err().code,
            "INVALID_PLAN",
            "{wire}"
        );
    }
}

#[test]
fn capabilities_roundtrip_preserves_runtime_and_scope_limits() {
    for runtime in ["rust", "python", "cli", "embedded\"\nλ"] {
        let decoded = serde_json::from_str(&capabilities_json(runtime)).unwrap();
        assert_eq!(capabilities(runtime), decoded);
    }
    let manifest = capabilities("rust");
    assert_eq!(manifest.limits.stories, ["body"]);
    assert!(manifest.limits.plain_text_runs);
    assert!(manifest.limits.refuses_opaque_ranges);
    assert!(!manifest.limits.reads_legacy_doc);
}

#[test]
fn checked_xml_rejects_bad_attributes_and_invalid_numeric_entities() {
    for xml in [
        "<a value='&unknown;'/>",
        "<a value='&#xD800;'/>",
        "<a value='&#x110000;'/>",
        "<a value=no_quotes/>",
        "<a value='unterminated/>",
        "<a><b value='1' value='2'/></a>",
        "<a>&#xD800;</a>",
        "<a>&#x110000;</a>",
        "<a>&#not_a_number;</a>",
        "<a/>trailing text",
        "<a/>&amp;",
    ] {
        assert!(validate_xml(xml).is_err(), "accepted {xml}");
    }
}

#[test]
fn checked_xml_accepts_unicode_entities_and_processing_instructions() {
    for xml in [
        "\u{feff}<?xml version='1.0'?><a>東京 😀</a>",
        "<?before data?><a/><?after data?>",
        "<a value='&quot;&apos;&amp;&lt;&gt;&#233;&#x1F600;'>é😀</a>",
        "<a><![CDATA[<not-an-element>&unknown;]]></a>",
        "<a xmlns:w='urn:word'><w:b w:value='1'/></a>",
    ] {
        assert_eq!(validate_xml(xml), Ok(()), "{xml}");
    }
}
