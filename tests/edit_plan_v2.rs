// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use jubarte::edit::v2::{PlanV2, before_sha256, logical_text};
use jubarte::namespaces::W;
use jubarte::xmllinq::{Dom, NodeId};
use serde_json::{Value, json};
use std::io::{Cursor, Read};
use std::sync::LazyLock;

const GENERATED: &str = include_str!("fixtures/edit-v2/template.generated.json");
const EXAMPLES: &[&str] = &[
    GENERATED,
    include_str!("fixtures/edit-v2/template.edited.json"),
    include_str!("fixtures/edit-v2/template.exact-replace.json"),
    include_str!("fixtures/edit-v2/template.composed.json"),
    include_str!("fixtures/edit-v2/template.with-comment.json"),
];
static SCHEMA: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
    let schema: Value =
        serde_json::from_str(include_str!("../schemas/edit-plan-v2.schema.json")).unwrap();
    jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&schema)
        .unwrap()
});
fn base() -> Value {
    serde_json::from_str(GENERATED).unwrap()
}
fn agrees(value: &Value, valid: bool) {
    assert_eq!(SCHEMA.is_valid(value), valid, "schema: {value}");
    let parsed = PlanV2::from_json(&value.to_string());
    assert_eq!(parsed.is_ok(), valid, "parser: {value}: {parsed:?}");
    if let Ok(plan) = parsed {
        assert_eq!(
            serde_json::from_str::<Value>(&plan.to_json()).unwrap(),
            *value
        );
    }
}
fn with_ops(operations: Value) -> Value {
    let mut v = base();
    v["operations"] = operations;
    v
}
fn paragraph(inner: &str) -> (Dom, NodeId) {
    let mut dom = Dom::new();
    let doc=dom.parse_xdocument(&format!(r#"<w:p xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">{inner}</w:p>"#));
    let p = dom.root(doc).unwrap();
    (dom, p)
}

#[test]
fn production_schema_and_parser_accept_every_example() {
    for example in EXAMPLES {
        agrees(&serde_json::from_str(example).unwrap(), true);
    }
}
#[test]
fn production_schema_and_parser_reject_design_malformed_cases() {
    let cases: Value =
        serde_json::from_str(include_str!("fixtures/edit-v2/malformed-plans.json")).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 16);
    for case in cases.as_array().unwrap() {
        eprintln!("{}", case["name"]);
        agrees(&case["plan"], false);
        assert_eq!(
            PlanV2::from_json(&case["plan"].to_string())
                .unwrap_err()
                .code,
            "INVALID_PLAN"
        );
    }
}
#[test]
fn eleven_kinds_and_explicit_policies_round_trip() {
    let t = json!({"short_id":"p0","long_id":"document.w:p.j:short.p0"});
    let c = json!({"short_id":"c0","long_id":"comments.w:comment.w:id.0"});
    let ops = all_operations();
    agrees(&with_ops(ops), true);
    for track in ["on", "off"] {
        for policy in ["keep", "accept", "reject", "refuse"] {
            let mut v = with_ops(json!([]));
            v["policy"]["track"] = json!(track);
            v["policy"]["existing_revisions"] = json!(policy);
            v["resolve_revisions"]["accept"] =
                json!([{"short_id":"r12","long_id":"document.w:ins.w:id.12"}]);
            v["scope"] = json!({"kind":"selected","targets":[t]});
            v["source_context"] = json!([{"kind":"opaque","target":t,"reason_codes":["FIELD"]},{"kind":"comment","target":c,"text":"old","reason_codes":["CONTEXT_ONLY"]}]);
            agrees(&v, true);
        }
    }
}
#[test]
fn strict_shapes_versions_and_cross_field_rules() {
    let t = base()["operations"][0]["target"].clone();
    for (pointer, bad) in [
        ("/source/sha256", json!("A".repeat(64))),
        ("/operations/0/target/short_id", json!("")),
        ("/operations/0/target/long_id", json!("")),
        ("/policy/track", json!("yes")),
        ("/policy/text_view", json!("original")),
        ("/coverage/eligible_paragraphs", json!(-1)),
        ("/scope", json!({"kind":"selected","targets":[]})),
        ("/resolve_revisions/accept", json!([t, t])),
    ] {
        let mut v = base();
        *v.pointer_mut(pointer).unwrap() = bad;
        agrees(&v, false);
    }
    // Every optional member still has a type; explicit null is not omission.
    for example in EXAMPLES {
        let v: Value = serde_json::from_str(example).unwrap();
        for key in v.as_object().unwrap().keys() {
            let mut bad = v.clone();
            bad[key] = Value::Null;
            agrees(&bad, false);
        }
        for (i, op) in v["operations"].as_array().unwrap().iter().enumerate() {
            for key in op.as_object().unwrap().keys() {
                let mut bad = v.clone();
                bad["operations"][i][key] = Value::Null;
                agrees(&bad, false);
            }
        }
    }
    for kind in ["replace", "rewrite", "insert", "insert_paragraph"] {
        let mut op = match kind {
            "replace" => json!({"kind":kind,"target":t,"find":"X","text":"X"}),
            "rewrite" => json!({"kind":kind,"target":t,"before_sha256":"a".repeat(64),"text":"X"}),
            "insert" => json!({"kind":kind,"target":t,"edge":"end","text":"X"}),
            _ => json!({"kind":kind,"target":t,"position":"after","text":"X"}),
        };
        op["attached_comment"] = json!({"text":"note"});
        agrees(&with_ops(json!([op])), true);
        op["text"] = json!("");
        agrees(&with_ops(json!([op])), false);
    }
    for version in [1, 3] {
        let mut v = base();
        v["schema_version"] = json!(version);
        agrees(&v, false);
        assert_eq!(
            PlanV2::from_json(&v.to_string()).unwrap_err().code,
            "UNSUPPORTED_SCHEMA"
        );
    }
    for invalid in ["{", "[]", "null", "{}"] {
        assert_eq!(PlanV2::from_json(invalid).unwrap_err().code, "INVALID_PLAN");
    }
}
#[test]
fn timestamp_calendar_and_exact_offsets() {
    for date in [
        "2024-02-29T23:59:59.123+05:30",
        "2026-10-17T07:00:00-04:00",
        "2000-02-29T00:00:00Z",
    ] {
        let mut v = base();
        v["date"] = json!(date);
        agrees(&v, true);
    }
    for date in [
        "1900-02-29T00:00:00Z",
        "2026-13-01T00:00:00Z",
        "2026-00-01T00:00:00Z",
        "2026-01-00T00:00:00Z",
        "2026-01-32T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T00:60:00Z",
        "2026-01-01T00:00:61Z",
        "2026-01-01T00:00:00+24:00",
        "2026-01-01T00:00:00+00:60",
        "2026-01-01T00:00:00Z\n",
        "2026-01-01t00:00:00z",
        "2026-01-01T00:00:00.Z",
    ] {
        let mut v = base();
        v["date"] = json!(date);
        agrees(&v, false);
    }
}
#[test]
fn exact_logical_text_codec_v1() {
    let (dom, p) = paragraph(
        "<w:pPr/><w:r><w:rPr/><w:t xml:space=\"preserve\">  A\u{a0}&amp;e\u{301} </w:t><w:tab/><w:t>B</w:t><w:br/><w:t>C</w:t><w:br w:type=\"textWrapping\"/></w:r>",
    );
    let text = "  A\u{a0}&e\u{301} \tB\nC\n";
    assert_eq!(logical_text(&dom, p).unwrap(), text);
    assert_eq!(
        before_sha256(&dom, p).unwrap(),
        jubarte::inspect::source_sha256(text.as_bytes())
    );
    let (dom, p) = paragraph("");
    assert_eq!(logical_text(&dom, p).unwrap(), "");
    assert_eq!(
        before_sha256(&dom, p).unwrap(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}
#[test]
fn opaque_content_never_becomes_editable_placeholder_text() {
    for opaque in [
        "<w:br w:type=\"page\"/>",
        "<w:br w:type=\"column\"/>",
        "<w:br w:type=\"unknown\"/>",
        "<w:sym w:char=\"F020\"/>",
        "<w:drawing/>",
        "<w:fldChar w:fldCharType=\"begin\"/>",
        "<w:instrText>PAGE</w:instrText>",
        "<w:footnoteReference w:id=\"1\"/>",
        "<w:ptab/>",
    ] {
        let (dom, p) = paragraph(&format!(
            "<w:r><w:t>before</w:t>{opaque}<w:t>after</w:t></w:r>"
        ));
        assert_eq!(
            logical_text(&dom, p).unwrap_err().code,
            "UNSUPPORTED_TEXT_CODEC"
        );
        assert_eq!(
            before_sha256(&dom, p).unwrap_err().code,
            "UNSUPPORTED_TEXT_CODEC"
        );
    }
}
#[test]
fn example_source_and_before_hashes_match_exact_codec() {
    let bytes = include_bytes!("fixtures/edit-v2/example-source.docx");
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&xml);
    let paragraphs = dom.descendants(doc, Some(&W::p()));
    assert_eq!(paragraphs.len(), 4);
    let texts: Vec<_> = paragraphs
        .iter()
        .map(|p| logical_text(&dom, *p).unwrap())
        .collect();
    assert_eq!(
        texts,
        [
            "Payment is due within 30 days.",
            "The fee is due; the fee is payable.",
            "One  space.",
            "Notice is due within 30\u{a0}days."
        ]
    );
    for example in EXAMPLES {
        let plan: Value = serde_json::from_str(example).unwrap();
        assert_eq!(
            plan["source"]["sha256"],
            jubarte::inspect::source_sha256(bytes)
        );
        for op in plan["operations"].as_array().unwrap() {
            if op["kind"] == "rewrite" {
                let i = op["target"]["short_id"].as_str().unwrap()[1..]
                    .parse::<usize>()
                    .unwrap();
                assert_eq!(
                    op["before_sha256"],
                    before_sha256(&dom, paragraphs[i]).unwrap()
                );
            }
        }
    }
}

#[test]
fn v1_plan_and_v2_schema_do_not_change_each_other() {
    let legacy = r#"{"schema_version":1,"author":"Legacy","operations":[]}"#;
    let v1 = jubarte::edit::EditPlan::from_json(legacy).unwrap();
    assert_eq!(
        v1.existing_revisions,
        jubarte::edit::ExistingRevisions::Refuse
    );
    assert_eq!(v1.source_sha256, None);
    assert_eq!(v1.date, None);
    assert!(!SCHEMA.is_valid(&serde_json::from_str(legacy).unwrap()));
    assert_eq!(
        jubarte::edit::EditPlan::from_json(GENERATED)
            .unwrap_err()
            .code,
        "INVALID_PLAN"
    );
    agrees(&base(), true);
}

macro_rules! malformed_case {
    ($name:ident, $index:expr) => {
        #[test]
        fn $name() {
            let cases: Value =
                serde_json::from_str(include_str!("fixtures/edit-v2/malformed-plans.json"))
                    .unwrap();
            agrees(&cases[$index]["plan"], false);
        }
    };
}
malformed_case!(rejects_run_operation, 0);
malformed_case!(rejects_missing_before_hash, 1);
malformed_case!(rejects_missing_selected_decisions, 2);
malformed_case!(rejects_unknown_root_field, 3);
malformed_case!(rejects_third_identity_field, 4);
malformed_case!(rejects_zero_occurrence, 5);
malformed_case!(rejects_empty_attached_anchor, 6);
malformed_case!(rejects_timezone_free_date, 7);
malformed_case!(rejects_invalid_calendar_date, 8);
malformed_case!(rejects_source_hash_trailing_newline, 9);
malformed_case!(rejects_before_hash_trailing_newline, 10);
malformed_case!(rejects_whitespace_author, 11);
malformed_case!(rejects_two_insert_positions, 12);
malformed_case!(rejects_occurrence_on_edge, 13);
malformed_case!(rejects_comment_occurrence_without_find, 14);
malformed_case!(rejects_comment_find_plus_through, 15);

#[test]
fn codec_uses_final_text_without_review_or_markdown_markers() {
    let (dom, p) = paragraph(
        r#"<w:bookmarkStart w:id="0"/><w:del><w:r><w:delText>old</w:delText></w:r></w:del><w:ins><w:r><w:t>{++literal++}</w:t></w:r></w:ins><w:hyperlink><w:r><w:t> linked</w:t></w:r></w:hyperlink><w:bookmarkEnd w:id="0"/>"#,
    );
    assert_eq!(logical_text(&dom, p).unwrap(), "{++literal++} linked");
}

#[test]
fn schema_and_parser_agree_on_recursive_shape_mutations() {
    fn visit(root: &Value, value: &Value, pointer: &str) {
        let check = |changed: Value| {
            let expected = SCHEMA.is_valid(&changed);
            let result = PlanV2::from_json(&changed.to_string());
            assert_eq!(result.is_ok(), expected, "{pointer}: {changed}: {result:?}");
        };
        for replacement in [
            Value::Null,
            json!(false),
            json!(0),
            json!(-1),
            json!(1.5),
            json!(""),
            json!([]),
            json!({}),
        ] {
            let mut changed = root.clone();
            *changed.pointer_mut(pointer).unwrap() = replacement;
            check(changed);
        }
        if let Some(object) = value.as_object() {
            let mut changed = root.clone();
            changed.pointer_mut(pointer).unwrap()["unknown_field"] = json!(true);
            check(changed);
            for (key, child) in object {
                let mut changed = root.clone();
                changed
                    .pointer_mut(pointer)
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
                    .remove(key);
                check(changed);
                visit(root, child, &format!("{pointer}/{key}"));
            }
        } else if let Some(array) = value.as_array() {
            for (index, child) in array.iter().enumerate() {
                visit(root, child, &format!("{pointer}/{index}"));
            }
        }
    }
    let mut roots: Vec<Value> = EXAMPLES
        .iter()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    let mut mixed = with_ops(all_operations());
    let t = mixed["operations"][0]["target"].clone();
    mixed["scope"] = json!({"kind":"selected","targets":[t]});
    mixed["resolve_revisions"]["accept"] =
        json!([{"short_id":"r1","long_id":"document.w:ins.w:id.1"}]);
    mixed["source_context"] = json!([{"kind":"paragraph","target":t,"text":"old","reason_codes":["CONTEXT_ONLY"]},{"kind":"opaque","target":t,"reason_codes":["FIELD"]}]);
    roots.push(mixed);
    for root in roots {
        visit(&root, &root, "");
    }
}

fn all_operations() -> Value {
    let t = json!({"short_id":"p0","long_id":"document.w:p.j:short.p0"});
    let c = json!({"short_id":"c0","long_id":"comments.w:comment.w:id.0"});
    json!([
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
fn schema_integer_and_timestamp_boundaries_agree() {
    for date in [
        "0000-01-01T00:00:00Z",
        "2016-12-31T23:59:60Z",
        "2017-01-01T00:59:60+01:00",
        "2016-12-31T22:59:60-01:00",
        "2026-01-01T00:00:00+23:59",
    ] {
        let mut v = base();
        v["date"] = json!(date);
        agrees(&v, true);
    }
    for date in [
        "2016-12-31T23:59:60+00:30",
        "2016-12-31T00:00:60Z",
        "2026-01-01T00:00:00.123",
        "é026-01-01T00:00:00Z",
        "2026-01-01T00:00:00+é0:00",
    ] {
        let mut v = base();
        v["date"] = json!(date);
        agrees(&v, false);
    }
    for number in [json!(0.0), json!(1.0), json!(1e10), json!(u64::MAX)] {
        let mut v = base();
        v["schema_version"] = json!(2.0);
        v["coverage"]["eligible_paragraphs"] = number;
        assert!(SCHEMA.is_valid(&v));
        let plan = PlanV2::from_json(&v.to_string()).unwrap();
        assert_eq!(plan.schema_version, 2);
        assert!(SCHEMA.is_valid(&serde_json::from_str::<Value>(&plan.to_json()).unwrap()));
    }
    for number in [json!(18446744073709551616.0_f64), json!(-1.0), json!(0.5)] {
        let mut v = base();
        v["coverage"]["eligible_paragraphs"] = number;
        agrees(&v, false);
    }
    let mut v = with_ops(all_operations());
    v["operations"][0]["occurrence"] = json!(1.0);
    assert!(SCHEMA.is_valid(&v));
    assert!(PlanV2::from_json(&v.to_string()).is_ok());
    let mut v = base();
    v["source_context"] = json!([{"kind":"opaque","target":v["operations"][0]["target"],"reason_codes":["FIELD","FIELD"]}]);
    agrees(&v, false);
}

#[test]
fn enum_fields_require_schema_string_spellings() {
    for pointer in [
        "/policy/track",
        "/policy/existing_revisions",
        "/policy/text_view",
    ] {
        let mut v = base();
        let name = v.pointer(pointer).unwrap().as_str().unwrap().to_owned();
        *v.pointer_mut(pointer).unwrap() = json!({name:Value::Null});
        agrees(&v, false);
    }
    let t = base()["operations"][0]["target"].clone();
    agrees(
        &with_ops(json!([{"kind":"insert","target":t,"text":"X","edge":{"end":null}}])),
        false,
    );
    agrees(
        &with_ops(
            json!([{"kind":"insert_paragraph","target":t,"text":"X","position":{"after":null}}]),
        ),
        false,
    );
    let mut v = base();
    v["source_context"] = json!([{"kind":{"opaque":null},"target":t,"reason_codes":["FIELD"]}]);
    agrees(&v, false);
}

#[test]
fn codec_refuses_unrepresented_element_nesting() {
    for inner in [
        "<w:t>lost</w:t>",
        "<w:tab/>",
        "<w:r><w:r><w:t>lost</w:t></w:r></w:r>",
        "<w:r><w:ins><w:r><w:t>lost</w:t></w:r></w:ins></w:r>",
    ] {
        let (dom, p) = paragraph(inner);
        assert_eq!(
            logical_text(&dom, p).unwrap_err().code,
            "UNSUPPORTED_TEXT_CODEC"
        );
    }
}
