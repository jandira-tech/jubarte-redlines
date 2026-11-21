// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Every change to another author's comment, named or not, refuses
/// under the guard and notes in its own words (#1846, #1892).
#[test]
fn others_comment_wording_covers_every_change() {
    let others = |change, author: &str| OthersComment {
        comment: 1,
        author: author.to_string(),
        change,
    };
    let cases = [
        (
            OthersChange::Reword,
            "their words were changed under their name",
            "its words were changed",
        ),
        (
            OthersChange::Delete,
            "their comment was deleted",
            "it was deleted",
        ),
        (
            OthersChange::Resolve,
            "their comment was resolved",
            "it was resolved",
        ),
        (
            OthersChange::Reopen,
            "their comment was reopened",
            "it was reopened",
        ),
    ];
    for (change, named_note, unnamed_note) in cases {
        assert_eq!(
            others(change, "Bob").refusal("R", "S"),
            r#"c1 is by "Bob"; reply instead (R)"#
        );
        assert_eq!(
            others(change, "").refusal("R", "S"),
            "c1 names no author; reply instead (R)"
        );
        assert_eq!(
            others(change, "Bob").note(),
            format!(r#"c1 is by "Bob"; {named_note}"#)
        );
        assert_eq!(
            others(change, "").note(),
            format!("c1 names no author; {unnamed_note}")
        );
    }
    let with = OthersChange::DeletedWith(0);
    assert_eq!(
        others(with, "Bob").refusal("R", "S"),
        r#"deleting c0 deletes c1, a reply by "Bob"; resolve c0 instead (S)"#
    );
    assert_eq!(
        others(with, "").refusal("R", "S"),
        "deleting c0 deletes c1, a reply that names no author; resolve c0 instead (S)"
    );
    assert_eq!(
        others(with, "Bob").note(),
        r#"c1 is by "Bob"; their reply was deleted with c0"#
    );
    assert_eq!(
        others(with, "").note(),
        "c1 names no author; it was deleted with c0"
    );
}

#[test]
fn a_copy_is_the_edits_own_only_when_it_copies_the_edits_own_record() {
    let mut ids = RevisionCounter::new(HashSet::from([4, 9]));
    let own = ids.allocate();
    let of_own = ids.copy(Some(own));
    let of_source = ids.copy(Some(4));
    let of_nothing = ids.copy(None);
    assert_eq!((own, of_own, of_source, of_nothing), (10, 11, 12, 13));
    assert_eq!(ids.own(), &BTreeSet::from([10, 11]));
}

#[test]
fn the_counter_stays_in_words_range_and_skips_used_ids() {
    let mut ids = RevisionCounter::new(HashSet::from([0, 2, u64::from(i32::MAX.unsigned_abs())]));
    assert_eq!(ids.allocate(), 1);
    assert_eq!(ids.copy(Some(0)), 3);
}

#[test]
fn plan_json_roundtrips() {
    let json = r#"{"schema_version":1,"author":"A","operations":[
            {"kind":"replace","paragraph":{"index":0},"find":"a","replacement":"b"},
            {"kind":"insert","paragraph":{"id":"p1"},"after":"x","text":"y","comment":"c"},
            {"kind":"insert","paragraph":{"index":0},"position":"end","text":"z"},
            {"kind":"comment","paragraph":{"contains":"q"},"text":"note"},
            {"kind":"insert_paragraph","paragraph":{"starts_with":"s"},"runs":[{"text":"t","bold":true}]},
            {"kind":"delete_paragraph","paragraph":{"index":2}}]}"#;
    let plan = EditPlan::from_json(json).unwrap();
    assert_eq!(plan.operations.len(), 6);
    assert_eq!(plan.existing_revisions, ExistingRevisions::Refuse);
    let again = EditPlan::from_json(&plan.to_json()).unwrap();
    assert_eq!(again, plan);
}

#[test]
fn plan_json_rejects_unknown_fields() {
    let bad = r#"{"schema_version":1,"author":"A","extra":1,"operations":[]}"#;
    assert_eq!(EditPlan::from_json(bad).unwrap_err().code, "INVALID_PLAN");
    let bad = r#"{"schema_version":1,"author":"A","operations":[{"kind":"replace","paragraph":{"index":0},"find":"a","replacement":"b","typo":1}]}"#;
    assert_eq!(EditPlan::from_json(bad).unwrap_err().code, "INVALID_PLAN");
}

#[test]
fn plan_json_rejects_malformed_json() {
    let e = EditPlan::from_json("{").unwrap_err();
    assert!(e.to_string().starts_with("INVALID_PLAN: "));
}

#[test]
fn plan_json_refuses_another_schema_version() {
    let other = SCHEMA_VERSION + 1;
    let json = format!(r#"{{"schema_version":{other},"author":"A","operations":[]}}"#);
    let e = EditPlan::from_json(&json).unwrap_err();
    assert_eq!(e.code, "UNSUPPORTED_SCHEMA");
    assert!(e.to_string().starts_with(&format!(
        "UNSUPPORTED_SCHEMA: schema_version {other} is not supported (expected {SCHEMA_VERSION})"
    )));
}

#[test]
fn plan_to_json_is_pretty_and_a_fixed_point() {
    let pretty = r#"{
  "schema_version": 1,
  "source_sha256": "sha",
  "author": "Author",
  "date": "2024-01-01T00:00:00Z",
  "initials": "A",
  "existing_revisions": "refuse",
  "operations": []
}"#;
    assert_eq!(EditPlan::from_json(pretty).unwrap().to_json(), pretty);
}

#[test]
fn plan_fields_name_every_field_a_plan_serializes() {
    let plan = EditPlan {
        schema_version: SCHEMA_VERSION,
        source_sha256: Some("0".repeat(64)),
        author: "A".into(),
        date: Some("2026-10-01T09:00:00Z".into()),
        initials: Some("A".into()),
        resolve_revisions: Some(ResolveRevisions::default()),
        removed_anchor_comments: RemovedAnchorComments::Keep,
        existing_revisions: ExistingRevisions::Keep,
        operations: Vec::new(),
        update_fields: true,
        rewrite_others_comments: true,
        others: Others::Keep,
    };
    let value = serde_json::to_value(&plan).unwrap();
    let written: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(written, PLAN_FIELDS);
    let bad = r#"{"schema_version":1,"author":"A","operatons":[]}"#;
    assert_eq!(
        EditPlan::from_json(bad).unwrap_err().to_string(),
        format!(
            r#"INVALID_PLAN: unknown field "operatons"; did you mean "operations"? fields: {}"#,
            PLAN_FIELDS.join(", ")
        )
    );
}

#[test]
fn nearest_names_the_closest_within_a_third() {
    assert_eq!(edit_distance("ocurrence", "occurrence"), 1);
    assert_eq!(edit_distance("", "find"), 4);
    assert_eq!(edit_distance("é", "e"), 1);
    let fields = ["comment_id", "text"];
    assert_eq!(nearest("txt", &fields, None), Some("text"));
    assert_eq!(nearest("comment", &fields, Some("text")), Some("text"));
    assert_eq!(nearest("urgency", &fields, None), None);
    // A meant name the operation cannot take is no suggestion.
    assert_eq!(nearest("anchor", &fields, Some("find")), None);
}

#[test]
fn context_windows_and_excerpts() {
    let text = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJ";
    let c = context(text, 24, 26, "{op}");
    assert_eq!(c, "456789abcdefghijklmn{op}qrstuvwxyzABCDEFGHIJ");
    assert_eq!(context("ab", 0, 0, "{+x}"), "{+x}ab");
    assert_eq!(excerpt("short", 10), "short");
    assert_eq!(excerpt("a longer sentence", 8), "a longer…");
}

#[test]
fn new_position_accounts_for_earlier_edits_and_insertions() {
    let edit = |start, end, op, replacement: &str| ScheduledEdit {
        start,
        end,
        op,
        replacement: replacement.to_string(),
        attach_before: true,
        comment: None,
        format: None,
    };
    let edits = vec![
        edit(2, 4, 0, "XYZ"), // +1
        edit(6, 6, 1, "++"),  // insertion at 6
        edit(8, 9, 2, ""),    // -1
    ];
    assert_eq!(new_position(&edits, 1, true, None), 1);
    assert_eq!(new_position(&edits, 5, true, None), 6);
    assert_eq!(
        new_position(&edits, 6, true, None),
        9,
        "inclusive shifts past the insertion at 6"
    );
    assert_eq!(
        new_position(&edits, 6, false, None),
        7,
        "exclusive does not"
    );
    assert_eq!(new_position(&edits, 10, true, None), 12);
    assert_eq!(
        new_position(&edits, 6, true, Some(1)),
        7,
        "an operation's own insertion is skipped"
    );
}

#[test]
fn check_text_rejects_controls() {
    assert!(check_text("plain “quotes” é").is_ok());
    assert!(check_text("tab\there").is_err());
    assert!(check_text("line\nbreak").is_err());
    assert!(check_text("\u{ffff}").is_err());
}

#[test]
fn rpr_children_are_inserted_in_schema_order() {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(&format!(
        r#"<w:rPr xmlns:w="{}"><w:rFonts w:ascii="Arial"/><w:sz w:val="22"/></w:rPr>"#,
        W::URI
    ));
    let rpr = dom.root(doc).unwrap();
    toggle(&mut dom, rpr, "b", Some(true));
    toggle(&mut dom, rpr, "i", Some(false));
    let h = dom.new_element(W::name("highlight"));
    insert_rpr_child(&mut dom, rpr, h);
    let names: Vec<String> = dom
        .elements(rpr, None)
        .iter()
        .map(|&c| dom.name(c).unwrap().local_name().to_string())
        .collect();
    assert_eq!(names, ["rFonts", "b", "i", "sz", "highlight"]);
    let i = dom.element(rpr, &W::name("i")).unwrap();
    assert_eq!(dom.attribute(i, &W::val()), Some("0"));
    toggle(&mut dom, rpr, "b", None);
    assert_eq!(dom.elements(rpr, None).len(), 5);
}

#[test]
fn error_display_names_the_operation() {
    let e = err("X", Some("op-2"), "why");
    assert_eq!(e.to_string(), "X (op-2): why");
    let e = err("X", None, "why");
    assert_eq!(e.to_string(), "X: why");
}
